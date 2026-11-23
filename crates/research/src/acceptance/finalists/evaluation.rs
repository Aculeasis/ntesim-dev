use super::*;

pub fn evaluate_finalist(
    coverage: &Value,
    variants_csv: &str,
    request: &FinalistValidationRequest,
    limits: &ExecutionLimits,
) -> Result<FinalistEvaluationReport, String> {
    if request.schema_version != 1 {
        return Err("unsupported schema version: expected 1".into());
    }
    if request.content_id != "ntesim-current" {
        return Err("unsupported content_id: expected ntesim-current".into());
    }
    if request.hypothesis.trim().is_empty() {
        return Err("finalist evaluation requires a non-empty hypothesis".into());
    }
    if request.seeds.is_empty() || request.seeds.len() > 1000 {
        return Err("finalist evaluation requires between 1 and 1000 seeds".into());
    }
    if request.seeds.iter().collect::<BTreeSet<_>>().len() != request.seeds.len() {
        return Err("finalist evaluation seeds must be distinct".into());
    }
    if request.timing_multipliers.is_empty() || request.timing_multipliers.len() > 3 {
        return Err("finalist evaluation requires 1..=3 timing multipliers".into());
    }

    if request
        .baseline_damage
        .is_some_and(|baseline| !baseline.is_finite() || baseline <= 0.0)
    {
        return Err("baseline damage must be finite and positive".into());
    }
    if !request
        .timing_multipliers
        .iter()
        .any(|ratio| ratio.denominator > 0 && ratio.numerator == ratio.denominator)
    {
        return Err("finalist evaluation requires a nominal timing multiplier".into());
    }

    let slots: Vec<&str> = request.composition.split('+').map(|s| s.trim()).collect();
    if slots.len() != 4 {
        return Err(format!(
            "composition must contain exactly 4 slots, found {}",
            slots.len()
        ));
    }
    let slot_set: BTreeSet<&str> = slots.iter().copied().collect();
    if slot_set.len() != 4 {
        return Err("composition slots must be 4 distinct characters".into());
    }

    let entities = coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?;
    let mut released_ids = BTreeSet::new();
    for e in entities {
        if let (Some("character"), Some("released"), Some(id)) = (
            e["kind"].as_str(),
            e["availability"].as_str(),
            e["id"].as_str(),
        ) {
            released_ids.insert(id.to_string());
        }
    }

    for slug in &slots {
        let profile = get_character_profile(slug)?;
        if !released_ids.contains(profile.character_id) {
            return Err(format!("character {slug} is not released"));
        }
    }

    let variants = parse_published_variants_csv(variants_csv)?;
    let is_published = variants
        .iter()
        .any(|v| v.slots.iter().map(|s| s.as_str()).collect::<BTreeSet<_>>() == slot_set);

    let mut scenario = crate::profiling::build_pilot_scenario(
        &request.composition,
        request.scenario_kind,
        Ratio {
            numerator: 1,
            denominator: 1,
        },
        request.seeds[0],
        Some(&request.actor_overrides),
    )?;
    if let Some(policy) = &request.research_crit {
        // Empty eligibility is explicit: do not silently invent caller selections.
        scenario.stochastic_crit = Some(policy.clone());
    }

    let pilot_report_val = simulation::run_game_scenario(&scenario, limits)?;
    let pilot_report: GameScenarioReport = serde_json::from_value(pilot_report_val)
        .map_err(|e| format!("failed to parse pilot report: {e}"))?;

    crate::trials::eligibility::validate_completed(&pilot_report)?;
    let all_invariants_passed = pilot_report.invariants_passed;
    let all_energy_conserved = pilot_report
        .final_actor_states
        .iter()
        .all(|a| a.energy_conserved);
    // A single fixed opening cannot establish continuing-cycle sustainability.
    let resource_sustainable = false;

    let batch_req = game_batch::Request {
        schema_version: 1,
        seeds: request.seeds.clone(),
        timing_policy: TimingPolicy::ScaleRecoveryAndHitDelayV1,
        timing_hypothesis: format!(
            "Finalist batch evaluation over {} seeds and {} timing multipliers",
            request.seeds.len(),
            request.timing_multipliers.len()
        ),
        timing_multipliers: request.timing_multipliers.clone(),
        max_events_per_run: 500,
        max_total_events: 1_000_000,
        max_run_receipt_bytes: 32 * 1024 * 1024,
    };

    let scenario_bytes = serde_json::to_vec(&scenario).map_err(|e| e.to_string())?;
    let batch_req_bytes = serde_json::to_vec(&batch_req).map_err(|e| e.to_string())?;
    let batch_report = game_batch::run_with_limits(&scenario_bytes, &batch_req_bytes, limits)?;

    let timing_stats = batch_report
        .timing_statistics
        .as_ref()
        .ok_or_else(|| "Batch execution produced no timing statistics".to_string())?;

    let mut timing_summaries = Vec::with_capacity(timing_stats.len());
    let mut batch_stats = Vec::with_capacity(timing_stats.len());

    for (t_idx, ts) in timing_stats.iter().enumerate() {
        let s = &ts.statistics;
        let mean = (s.mean.numerator as f64) / (s.mean.denominator as f64);
        let min = (s.minimum.numerator as f64) / (s.minimum.denominator as f64);
        let max = (s.maximum.numerator as f64) / (s.maximum.denominator as f64);

        let adv_pct = request.baseline_damage.map(|base| {
            if base > 0.0 {
                ((mean - base) / base) * 100.0
            } else {
                0.0
            }
        });

        timing_summaries.push(FinalistTimingSummary {
            timing_index: t_idx,
            multiplier: ts.multiplier,
            multiplier_label: format!("{}/{}x", ts.multiplier.numerator, ts.multiplier.denominator),
            count: s.count,
            mean_damage: mean,
            min_damage: min,
            max_damage: max,
            advantage_vs_baseline_pct: adv_pct,
        });

        batch_stats.push(s.clone());
    }

    let nominal_mean = timing_summaries
        .iter()
        .find(|t| t.multiplier.numerator == t.multiplier.denominator)
        .ok_or("nominal timing statistics missing")?
        .mean_damage;

    let nominal_adv_pct = request.baseline_damage.map(|base| {
        if base > 0.0 {
            ((nominal_mean - base) / base) * 100.0
        } else {
            0.0
        }
    });

    let beats_5pct = nominal_adv_pct.map(|pct| pct >= 5.0).unwrap_or(false);

    let retains_under_timing = timing_summaries.iter().all(|t| {
        t.advantage_vs_baseline_pct
            .map(|p| p > 0.0)
            .unwrap_or(false)
    });

    let superiority_confirmed =
        !is_published && beats_5pct && retains_under_timing && resource_sustainable;

    let classification = if is_published {
        FinalistClassification::PublishedVariant
    } else if superiority_confirmed {
        FinalistClassification::UnexpectedFinalist
    } else if nominal_adv_pct.map(|p| p > 0.0).unwrap_or(false) {
        FinalistClassification::PromisingCandidate
    } else {
        FinalistClassification::Underperforming
    };

    Ok(FinalistEvaluationReport {
        schema_version: 1,
        report_kind: "validate-finalist-evaluation-v1".into(),
        content_id: "ntesim-current".into(),
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        composition: request.composition.clone(),
        slots: [
            slots[0].to_string(),
            slots[1].to_string(),
            slots[2].to_string(),
            slots[3].to_string(),
        ],
        scenario: request.scenario_kind.code().to_string(),
        scenario_description: request.scenario_kind.description().to_string(),
        is_published_variant: is_published,
        classification,
        seed_count: request.seeds.len(),
        resource_sustainable,
        all_invariants_passed,
        all_energy_conserved,
        nominal_mean_damage: nominal_mean,
        baseline_damage: request.baseline_damage,
        baseline_archetype: request.baseline_archetype.clone(),
        nominal_advantage_pct: nominal_adv_pct,
        beats_5pct_threshold: beats_5pct,
        retains_advantage_under_timing_degradation: retains_under_timing,
        superiority_confirmed,
        timing_summaries,
        batch_statistics: batch_stats,
        hypothesis: request.hypothesis.clone(),
        blockers: vec![
            "Direct in-game damage validation is unavailable; results are calculated model estimates".into(),
            "Complete kit proc and enemy break timelines are unresolved".into(),
            "Continuing cycles were not executed; resource sustainability is unverified".into(),
        ],
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
    })
}
