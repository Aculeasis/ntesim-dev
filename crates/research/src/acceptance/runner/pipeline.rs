use super::*;

#[derive(Clone)]
enum ScreeningOutcome {
    Success(CompositionScreeningSummary),
    Failure(CompositionScreeningFailure),
}

fn checkpoint_hash(checkpoint: &AcceptanceCheckpoint) -> Result<String, String> {
    let mut payload = serde_json::to_value(checkpoint).map_err(|error| error.to_string())?;
    payload
        .as_object_mut()
        .ok_or("checkpoint must be an object")?
        .remove("checkpoint_id");
    Ok(sha256(
        &serde_json::to_vec(&payload).map_err(|error| error.to_string())?,
    ))
}

fn actor_overrides(
    composition: &str,
) -> Result<BTreeMap<String, crate::published_teams::ActorOverride>, String> {
    let scenario = build_pilot_scenario(
        composition,
        AcceptanceScenarioKind::SingleTargetReal120,
        Ratio {
            numerator: 1,
            denominator: 1,
        },
        0,
        None,
    )?;
    Ok(composition
        .split('+')
        .zip(scenario.actors)
        .map(|(slug, actor)| {
            (
                slug.to_string(),
                crate::published_teams::ActorOverride {
                    weapon_id: Some(actor.weapon_id),
                    weapon_tier: Some(actor.weapon_tier),
                    set_id: actor.set_id,
                    static_stats: actor.static_stats,
                },
            )
        })
        .collect())
}

pub fn run_acceptance_pipeline(
    coverage: &Value,
    conditions: &Value,
    variants_csv: &str,
    request: &AcceptanceRunRequest,
    limits: &ExecutionLimits,
) -> Result<AcceptanceRunReport, String> {
    if request.schema_version != 1 || request.content_id != "ntesim-current" {
        return Err("unsupported schema version or content_id".into());
    }
    if request.hypothesis.trim().is_empty()
        || request.scenarios.is_empty()
        || request
            .scenarios
            .iter()
            .map(|kind| kind.code())
            .collect::<BTreeSet<_>>()
            .len()
            != request.scenarios.len()
    {
        return Err("acceptance requires a hypothesis and distinct nonempty scenarios".into());
    }
    for (name, seeds) in [
        ("screening", &request.screening_seeds),
        ("finalist", &request.finalist_seeds),
    ] {
        if seeds.is_empty()
            || seeds.len() > 1000
            || seeds.iter().collect::<BTreeSet<_>>().len() != seeds.len()
        {
            return Err(format!("{name} requires 1..=1000 distinct seeds"));
        }
    }
    if request.timing_multipliers.is_empty()
        || request.timing_multipliers.len() > 3
        || request
            .timing_multipliers
            .iter()
            .any(|ratio| ratio.denominator == 0 || ratio.numerator < ratio.denominator)
        || !request
            .timing_multipliers
            .iter()
            .any(|ratio| ratio.numerator == ratio.denominator)
    {
        return Err("acceptance requires 1..=3 timing multipliers >=1, including nominal".into());
    }
    if request.worker_config.max_workers == 0
        || request.worker_config.max_workers > 64
        || request.worker_config.chunk_size == 0
    {
        return Err("worker config requires 1..=64 workers and positive chunk size".into());
    }
    if request
        .scenarios
        .iter()
        .any(AcceptanceScenarioKind::is_mode_clock)
    {
        return Err(
            "unsupported mode-clock acceptance scenario: game runner has no mode-clock horizon"
                .into(),
        );
    }

    let comp_report: CompositionReport =
        composition::enumerate_compositions(coverage, variants_csv)?;
    let all_compositions = comp_report.compositions;
    let total_enumerated = all_compositions.len();
    let max_to_screen = request
        .max_compositions_to_evaluate
        .unwrap_or(total_enumerated);
    if max_to_screen == 0 || max_to_screen > total_enumerated {
        return Err("composition limit must be within the enumerated roster".into());
    }

    // Resource/parallel execution controls are not semantic inputs; extension of a
    // canonical composition prefix may reuse screening, but never stale finalists.
    let mut semantic = serde_json::to_value(request).map_err(|error| error.to_string())?;
    let fields = semantic
        .as_object_mut()
        .ok_or("request must be an object")?;
    fields.remove("resume_checkpoint");
    fields.remove("worker_config");
    fields.remove("max_compositions_to_evaluate");
    let request_sha256 = sha256(
        &serde_json::to_vec(&serde_json::json!({
            "request": semantic, "coverage": coverage, "conditions": conditions,
            "variants_csv": variants_csv, "limits": limits,
            "engine_version": ENGINE_VERSION, "engine_source_sha256": ENGINE_SOURCE_SHA256,
        }))
        .map_err(|error| error.to_string())?,
    );

    let (
        mut processed_indices,
        mut screening_results,
        mut screening_failures,
        mut cumulative_sha256,
        resume_count,
    ) = if let Some(cp) = &request.resume_checkpoint {
        if cp.schema_version != 1
            || cp.request_sha256 != request_sha256
            || checkpoint_hash(cp)? != cp.checkpoint_id
        {
            return Err("corrupted checkpoint or execution identity mismatch".into());
        }
        if cp.current_phase != AcceptancePhase::Deliverables
            || cp.processed_composition_indices.len() > max_to_screen
            || cp.completed_composition_indices.len() > cp.processed_composition_indices.len()
            || cp.screening_results.len() + cp.screening_failures.len()
                != cp.processed_composition_indices.len() * request.scenarios.len()
            || cp.is_complete
                != (cp.processed_composition_indices.len() == total_enumerated
                    && cp.screening_failures.is_empty())
        {
            return Err("corrupted checkpoint progress metadata".into());
        }
        let mut verify_hash = sha256(b"acceptance-start-seed");
        for (offset, index) in cp.processed_composition_indices.iter().enumerate() {
            if *index != offset + 1 {
                return Err("checkpoint indices must be a canonical unique prefix".into());
            }
            let comp = &all_compositions[offset];
            for scenario in &request.scenarios {
                let key = screening::result_key(*scenario, &comp.composition);
                if let Some(summary) = cp.screening_results.get(&key) {
                    if summary.composition_index != *index
                        || summary.composition != comp.composition
                        || summary.scenario != *scenario
                        || summary.seeds != request.screening_seeds
                        || summary.is_published_variant != comp.is_published_variant
                        || summary.matched_archetype
                            != comp
                                .matched_variant
                                .as_ref()
                                .map(|matched| matched.archetype.clone())
                        || !summary.invariants_passed
                        || !summary.energy_conserved
                        || summary.nominal_damage_exact.denominator == 0
                        || !summary.nominal_dps.is_finite()
                        || summary.nominal_dps
                            != summary.nominal_damage_exact.to_f64()
                                / (scenario.horizon_us() as f64 / 1_000_000.0)
                        || screening::summary_hash(summary)? != summary.result_sha256
                    {
                        return Err("corrupted screening checkpoint receipt".into());
                    }
                    verify_hash =
                        sha256(format!("{verify_hash}:{}", summary.result_sha256).as_bytes());
                } else if let Some(failure) = cp.screening_failures.get(&key) {
                    if failure.composition_index != *index
                        || failure.composition != comp.composition
                        || failure.scenario != *scenario
                        || failure.seeds != request.screening_seeds
                        || failure.is_published_variant != comp.is_published_variant
                        || failure.matched_archetype
                            != comp
                                .matched_variant
                                .as_ref()
                                .map(|matched| matched.archetype.clone())
                        || failure.reason.trim().is_empty()
                        || screening::failure_hash(failure)? != failure.failure_sha256
                    {
                        return Err("corrupted screening failure receipt".into());
                    }
                    verify_hash =
                        sha256(format!("{verify_hash}:{}", failure.failure_sha256).as_bytes());
                } else {
                    return Err("missing screening outcome in checkpoint".into());
                }
            }
        }
        let expected_completed_indices = (1..=cp.processed_composition_indices.len())
            .filter(|index| {
                let composition = &all_compositions[*index - 1].composition;
                request.scenarios.iter().all(|scenario| {
                    cp.screening_results
                        .contains_key(&screening::result_key(*scenario, composition))
                })
            })
            .collect::<Vec<_>>();
        if cp.completed_composition_indices != expected_completed_indices {
            return Err(
                "checkpoint completed indices differ from successful screening outcomes".into(),
            );
        }
        if verify_hash != cp.cumulative_sha256 {
            return Err("corrupted checkpoint cumulative hash".into());
        }
        (
            cp.processed_composition_indices.clone(),
            cp.screening_results.clone(),
            cp.screening_failures.clone(),
            cp.cumulative_sha256.clone(),
            cp.resume_count
                .checked_add(1)
                .ok_or("checkpoint resume counter overflow")?,
        )
    } else {
        (
            Vec::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            sha256(b"acceptance-start-seed"),
            0,
        )
    };

    let pending = &all_compositions[processed_indices.len()..max_to_screen];
    let screen =
        |comp: &crate::composition::CompositionRecord| -> Result<Vec<ScreeningOutcome>, String> {
            request
                .scenarios
                .iter()
                .map(|scenario| {
                    match screen_single_composition(
                        comp,
                        *scenario,
                        &request.screening_seeds,
                        limits,
                    ) {
                        Ok(summary) => Ok(ScreeningOutcome::Success(summary)),
                        Err(reason) => screening::failure_summary(
                            comp,
                            *scenario,
                            &request.screening_seeds,
                            reason,
                        )
                        .map(ScreeningOutcome::Failure),
                    }
                })
                .collect()
        };
    let mut newly_screened = Vec::new();
    if request.worker_config.max_workers == 1 || cfg!(target_arch = "wasm32") {
        for comp in pending {
            newly_screened.extend(screen(comp)?);
        }
    } else {
        // Bounded worker waves use the declared chunk size. Scoped threads read the
        // same immutable inputs; output order is canonical irrespective of scheduling.
        for wave in pending.chunks(
            request
                .worker_config
                .chunk_size
                .saturating_mul(request.worker_config.max_workers),
        ) {
            let results = thread::scope(|scope| {
                let handles: Vec<_> = wave
                    .chunks(request.worker_config.chunk_size)
                    .map(|chunk| {
                        let screen_ref = &screen;
                        scope.spawn(move || {
                            let mut results = Vec::new();
                            for comp in chunk {
                                results.extend(screen_ref(comp)?);
                            }
                            Ok::<_, String>(results)
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|handle| {
                        handle
                            .join()
                            .map_err(|_| "worker thread panicked".to_string())?
                    })
                    .collect::<Result<Vec<_>, String>>()
            })?;
            newly_screened.extend(results.into_iter().flatten());
        }
    }
    newly_screened.sort_by_key(|outcome| {
        let (composition_index, scenario) = match outcome {
            ScreeningOutcome::Success(summary) => (summary.composition_index, summary.scenario),
            ScreeningOutcome::Failure(failure) => (failure.composition_index, failure.scenario),
        };
        (
            composition_index,
            request
                .scenarios
                .iter()
                .position(|candidate| *candidate == scenario)
                .unwrap(),
        )
    });
    for outcome in newly_screened {
        match outcome {
            ScreeningOutcome::Success(summary) => {
                cumulative_sha256 =
                    sha256(format!("{cumulative_sha256}:{}", summary.result_sha256).as_bytes());
                screening_results.insert(
                    screening::result_key(summary.scenario, &summary.composition),
                    summary,
                );
            }
            ScreeningOutcome::Failure(failure) => {
                cumulative_sha256 =
                    sha256(format!("{cumulative_sha256}:{}", failure.failure_sha256).as_bytes());
                screening_failures.insert(
                    screening::result_key(failure.scenario, &failure.composition),
                    failure,
                );
            }
        }
    }
    processed_indices = (1..=max_to_screen).collect();
    let completed_indices = (1..=max_to_screen)
        .filter(|index| {
            let composition = &all_compositions[*index - 1].composition;
            request.scenarios.iter().all(|scenario| {
                screening_results.contains_key(&screening::result_key(*scenario, composition))
            })
        })
        .collect();

    let mut top_entries = Vec::new();
    let mut finalist_results = BTreeMap::new();
    for scenario in &request.scenarios {
        let mut ranked: Vec<_> = screening_results
            .values()
            .filter(|summary| summary.scenario == *scenario)
            .collect();
        ranked.sort_by(|left, right| {
            right
                .nominal_dps
                .total_cmp(&left.nominal_dps)
                .then(left.composition_index.cmp(&right.composition_index))
        });
        let baseline = ranked
            .iter()
            .find(|summary| summary.is_published_variant)
            .copied();
        let mut seen = BTreeSet::new();
        let mut promoted: Vec<_> = ranked
            .iter()
            .take(request.top_candidates_per_scenario)
            .copied()
            .collect();
        for summary in &promoted {
            seen.insert(summary.composition.clone());
        }
        let mut archetypes = BTreeSet::new();
        for summary in &ranked {
            if summary
                .matched_archetype
                .as_ref()
                .is_some_and(|archetype| archetypes.insert(archetype.clone()))
                && seen.insert(summary.composition.clone())
            {
                promoted.push(summary);
            }
        }
        for (index, summary) in ranked.iter().enumerate() {
            top_entries.push(TopCandidateEntry {
                scenario: *scenario,
                rank: index + 1,
                composition: summary.composition.clone(),
                nominal_dps: summary.nominal_dps,
                is_published_variant: summary.is_published_variant,
                matched_archetype: summary.matched_archetype.clone(),
                superiority_vs_baseline_pct: baseline
                    .filter(|baseline| baseline.nominal_dps > 0.0)
                    .map(|baseline| (summary.nominal_dps / baseline.nominal_dps - 1.0) * 100.0),
            });
        }
        let finalist_request = |composition: &str,
                                baseline_damage,
                                baseline_archetype|
         -> Result<FinalistValidationRequest, String> {
            Ok(FinalistValidationRequest {
                schema_version: 1,
                content_id: request.content_id.clone(),
                hypothesis: request.hypothesis.clone(),
                composition: composition.into(),
                scenario_kind: *scenario,
                seeds: request.finalist_seeds.clone(),
                timing_multipliers: request.timing_multipliers.clone(),
                research_crit: None,
                actor_overrides: actor_overrides(composition)?,
                baseline_damage,
                baseline_archetype,
            })
        };
        // Compare damage with a baseline evaluated under the same finalist seeds,
        // crit policy, scenario and builds. An absent/zero baseline stays absent.
        let baseline_damage = if let Some(baseline) = baseline {
            let report = evaluate_finalist(
                coverage,
                variants_csv,
                &finalist_request(&baseline.composition, None, None)?,
                limits,
            )?;
            (report.nominal_mean_damage > 0.0).then_some(report.nominal_mean_damage)
        } else {
            None
        };
        for summary in promoted {
            let report = evaluate_finalist(
                coverage,
                variants_csv,
                &finalist_request(
                    &summary.composition,
                    baseline_damage,
                    baseline.and_then(|summary| summary.matched_archetype.clone()),
                )?,
                limits,
            )?;
            finalist_results.insert(
                screening::result_key(*scenario, &summary.composition),
                report,
            );
        }
    }

    let (deliverables, _, _, _, _) =
        compile_acceptance_deliverables(coverage, conditions, variants_csv)?;
    let screening_complete =
        screening_failures.is_empty() && processed_indices.len() == total_enumerated;
    let failed_screening_count = screening_failures.len();
    let mut checkpoint = AcceptanceCheckpoint {
        checkpoint_id: String::new(),
        schema_version: 1,
        request_sha256: request_sha256.clone(),
        current_phase: AcceptancePhase::Deliverables,
        completed_composition_indices: completed_indices,
        processed_composition_indices: processed_indices,
        screening_results,
        screening_failures,
        finalist_results: finalist_results.clone(),
        cumulative_sha256,
        is_complete: screening_complete,
        resume_count,
    };
    checkpoint.checkpoint_id = checkpoint_hash(&checkpoint)?;
    Ok(AcceptanceRunReport {
        schema_version: 1, report_kind: VERSION.into(), content_id: request.content_id.clone(),
        status: "INCOMPLETE".into(), quality: Quality::Incomplete, game_dps_computed: false, acceptance_ready: false,
        request_sha256, engine_version: ENGINE_VERSION.into(), engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
        current_phase: checkpoint.current_phase, compositions_enumerated: total_enumerated,
        compositions_processed: checkpoint.processed_composition_indices.len(),
        compositions_screened: checkpoint.completed_composition_indices.len(), finalists_evaluated: finalist_results.len(),
        published_variants_evaluated: finalist_results.values().filter(|report| report.is_published_variant).count(),
        unexpected_finalists_found: finalist_results.values().filter(|report| report.classification == FinalistClassification::UnexpectedFinalist).count(),
        top_candidates: top_entries, finalists: finalist_results.into_values().collect(), checkpoint,
        deliverables: Some(deliverables), blockers: [
            "Fixed preview openings do not establish complete kits, break timelines, live rotations or continuing cycles".into(),
            "Mode-clock scenarios are unsupported; resource sustainability remains unverified".into(),
        ]
        .into_iter()
        .chain((failed_screening_count > 0).then(|| format!(
            "{} composition-scenario executions failed and were excluded from ranking",
            failed_screening_count
        )))
        .collect(), input_hashes: BTreeMap::from([
            ("coverage".into(), sha256(&serde_json::to_vec(coverage).map_err(|error| error.to_string())?)),
            ("conditions".into(), sha256(&serde_json::to_vec(conditions).map_err(|error| error.to_string())?)),
            ("variants_csv".into(), sha256(variants_csv.as_bytes())),
        ]),
    })
}
