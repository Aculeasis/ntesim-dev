use super::*;

pub fn run_pilot(
    coverage: &Value,
    request: &PilotRequest,
    limits: &ExecutionLimits,
) -> Result<PilotReport, String> {
    if request.schema_version != 1 {
        return Err("unsupported schema version: expected 1".into());
    }
    if request.content_id != "ntesim-current" {
        return Err("unsupported content_id: expected ntesim-current".into());
    }
    if request.teams.is_empty() {
        return Err("pilot requires at least 1 team composition".into());
    }
    if request.scenarios.is_empty() {
        return Err("pilot requires at least 1 scenario".into());
    }
    if request.timing_multipliers.is_empty() {
        return Err("pilot requires at least 1 timing multiplier".into());
    }
    if request.seeds.is_empty() {
        return Err("pilot requires at least 1 seed".into());
    }

    if request.hypothesis.trim().is_empty()
        || request.seeds.len() > 1000
        || request.seeds.iter().collect::<BTreeSet<_>>().len() != request.seeds.len()
        || request
            .scenarios
            .iter()
            .map(|scenario| scenario.code())
            .collect::<BTreeSet<_>>()
            .len()
            != request.scenarios.len()
        || request.teams.iter().collect::<BTreeSet<_>>().len() != request.teams.len()
        || request.timing_multipliers.len() > 3
        || request
            .timing_multipliers
            .iter()
            .any(|ratio| ratio.denominator == 0 || ratio.numerator < ratio.denominator)
        || request
            .budget
            .screening_prune_threshold_pct
            .is_some_and(|threshold| !threshold.is_finite() || !(0.0..=100.0).contains(&threshold))
    {
        return Err("pilot requires a hypothesis, distinct inputs, 1..=1000 seeds, 1..=3 timing multipliers >=1 and a finite pruning threshold in [0,100]".into());
    }
    let total_units = request
        .teams
        .len()
        .checked_mul(request.scenarios.len())
        .and_then(|value| value.checked_mul(request.timing_multipliers.len()))
        .and_then(|value| value.checked_mul(request.seeds.len()))
        .ok_or("pilot unit count overflow")?;
    if total_units > 100_000 {
        return Err("pilot execution units exceed 100000".into());
    }
    if request
        .scenarios
        .iter()
        .any(AcceptanceScenarioKind::is_mode_clock)
    {
        return Err(
            "unsupported mode-clock pilot scenario: game runner has no mode-clock horizon".into(),
        );
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

    for team in &request.teams {
        let slugs: Vec<&str> = team.split('+').map(|s| s.trim()).collect();
        if slugs.len() != 4 {
            return Err(format!("team {team} must contain exactly 4 characters"));
        }
        for slug in slugs {
            let p = get_character_profile(slug)?;
            if !released_ids.contains(p.character_id) {
                return Err(format!("character {slug} is not released"));
            }
        }
    }

    let mut semantic_request = serde_json::to_value(request).map_err(|e| e.to_string())?;
    let fields = semantic_request
        .as_object_mut()
        .ok_or("request must be an object")?;
    fields.remove("resume_checkpoint");
    fields.remove("budget");
    let request_bytes = serde_json::to_vec(&serde_json::json!({"request":semantic_request,"coverage":coverage,"limits":limits,"engine":ENGINE_SOURCE_SHA256,"version":ENGINE_VERSION})).map_err(|e| e.to_string())?;
    let request_sha256 = sha256(&request_bytes);

    let mut planned_units = Vec::new();
    let mut unit_idx = 0;
    for team in &request.teams {
        for scenario in &request.scenarios {
            for timing in &request.timing_multipliers {
                for seed in &request.seeds {
                    planned_units.push(PilotExecutionUnit {
                        unit_index: unit_idx,
                        team: team.clone(),
                        scenario_kind: *scenario,
                        timing_multiplier: *timing,
                        seed: *seed,
                    });
                    unit_idx += 1;
                }
            }
        }
    }

    let (mut completed_units, mut failed_units, mut cumulative_sha256, resume_count) =
        if let Some(cp) = &request.resume_checkpoint {
            if cp.schema_version != 1 {
                return Err("unsupported checkpoint schema version: expected 1".into());
            }
            if cp.next_unit_index > planned_units.len() {
                return Err("checkpoint contains more units than planned".into());
            }

            // Verify every success and failure receipt against its planned unit,
            // then hash the complete processed prefix in unit order.
            let mut outcome_hashes = BTreeMap::new();
            for u in &cp.completed_units {
                let pu = planned_units
                    .get(u.unit_index)
                    .ok_or("checkpoint contains an unplanned successful unit")?;
                if u.unit_index != pu.unit_index
                    || u.team != pu.team
                    || u.scenario != pu.scenario_kind.code()
                    || u.timing_multiplier != pu.timing_multiplier
                    || u.seed != pu.seed
                {
                    return Err(format!(
                        "checkpoint unit {} differs from planned unit",
                        u.unit_index
                    ));
                }
                if u.damage_exact.denominator == 0
                    || measurement::deterministic_unit_hash(u)? != u.unit_sha256
                    || !u.damage_f64.is_finite()
                    || u.damage_f64 != u.damage_exact.to_f64()
                    || !u.dps.is_finite()
                    || u.dps
                        != u.damage_exact.to_f64()
                            / (pu.scenario_kind.horizon_us() as f64 / 1_000_000.0)
                {
                    return Err("corrupted checkpoint unit receipt".into());
                }
                if outcome_hashes
                    .insert(u.unit_index, u.unit_sha256.clone())
                    .is_some()
                {
                    return Err("checkpoint contains duplicate unit outcomes".into());
                }
            }

            for failure in &cp.failed_units {
                let pu = planned_units
                    .get(failure.unit_index)
                    .ok_or("checkpoint contains an unplanned failed unit")?;
                if failure.unit_index != pu.unit_index
                    || failure.team != pu.team
                    || failure.scenario != pu.scenario_kind.code()
                    || failure.timing_multiplier != pu.timing_multiplier
                    || failure.seed != pu.seed
                    || failure.reason.trim().is_empty()
                    || measurement::deterministic_failure_hash(failure)? != failure.failure_sha256
                {
                    return Err(format!(
                        "checkpoint failure receipt {} differs from planned unit",
                        failure.unit_index
                    ));
                }
                if outcome_hashes
                    .insert(failure.unit_index, failure.failure_sha256.clone())
                    .is_some()
                {
                    return Err("checkpoint contains duplicate unit outcomes".into());
                }
            }

            if outcome_hashes.len() != cp.next_unit_index
                || (0..cp.next_unit_index).any(|idx| !outcome_hashes.contains_key(&idx))
            {
                return Err("checkpoint outcomes do not cover the processed prefix".into());
            }
            let mut verify_hash = sha256(b"pilot-seed-start");
            for hash in outcome_hashes.values() {
                verify_hash = sha256(format!("{}:{hash}", verify_hash).as_bytes());
            }

            if verify_hash != cp.cumulative_sha256 {
                return Err("corrupted checkpoint cumulative SHA-256 hash".into());
            }
            if cp.request_sha256 != request_sha256
                || measurement::checkpoint_hash(cp)? != cp.checkpoint_id
            {
                return Err("corrupted checkpoint or execution identity mismatch".into());
            }
            if cp.next_unit_index != cp.completed_units.len() + cp.failed_units.len()
                || cp.total_planned_units != planned_units.len()
                || cp.is_complete != (cp.next_unit_index == planned_units.len())
            {
                return Err("corrupted checkpoint progress metadata".into());
            }

            (
                cp.completed_units.clone(),
                cp.failed_units.clone(),
                cp.cumulative_sha256.clone(),
                cp.resume_count
                    .checked_add(1)
                    .ok_or("checkpoint resume counter overflow")?,
            )
        } else {
            (Vec::new(), Vec::new(), sha256(b"pilot-seed-start"), 0)
        };

    let start_unit_index = completed_units.len() + failed_units.len();
    let batch_start = Instant::now();
    for (newly_completed, unit) in planned_units[start_unit_index..].iter().enumerate() {
        if request
            .budget
            .max_units_per_checkpoint
            .is_some_and(|max| newly_completed >= max)
        {
            break;
        }
        if request
            .budget
            .max_wall_time_ms
            .is_some_and(|max_ms| batch_start.elapsed().as_millis() as u64 >= max_ms)
        {
            break;
        }

        match execute_pilot_unit(unit, request, limits) {
            Ok(u_result) => {
                cumulative_sha256 =
                    sha256(format!("{}:{}", cumulative_sha256, u_result.unit_sha256).as_bytes());
                completed_units.push(u_result);
            }
            Err(reason)
                if reason.contains("unsupported preview action:")
                    || reason.starts_with(
                        "Fireflies team-hit geometry requires exactly one target; received ",
                    ) =>
            {
                let failure = measurement::failed_unit_receipt(unit, reason)?;
                cumulative_sha256 =
                    sha256(format!("{}:{}", cumulative_sha256, failure.failure_sha256).as_bytes());
                failed_units.push(failure);
            }
            Err(error) => return Err(error),
        }
    }

    let next_unit_index = completed_units.len() + failed_units.len();
    let is_complete = next_unit_index == planned_units.len();

    let mut checkpoint = PilotCheckpoint {
        checkpoint_id: String::new(),
        schema_version: 1,
        request_sha256: request_sha256.clone(),
        cumulative_sha256: cumulative_sha256.clone(),
        completed_units: completed_units.clone(),
        failed_units: failed_units.clone(),
        next_unit_index,
        total_planned_units: planned_units.len(),
        is_complete,
        resume_count,
    };
    checkpoint.checkpoint_id = measurement::checkpoint_hash(&checkpoint)?;

    let units_completed = completed_units.len();
    let units_failed = failed_units.len();
    let total_wall_time_us = completed_units.iter().try_fold(0u64, |sum, unit| {
        sum.checked_add(unit.elapsed_us)
            .ok_or("profiling time overflow")
    })?;
    let total_events_processed = completed_units.iter().try_fold(0u64, |sum, unit| {
        sum.checked_add(unit.events_processed as u64)
            .ok_or("profiling event count overflow")
    })?;

    let average_us_per_unit = if units_completed > 0 {
        total_wall_time_us as f64 / units_completed as f64
    } else {
        0.0
    };
    let min_us_per_unit = completed_units
        .iter()
        .map(|u| u.elapsed_us)
        .min()
        .unwrap_or(0);
    let max_us_per_unit = completed_units
        .iter()
        .map(|u| u.elapsed_us)
        .max()
        .unwrap_or(0);

    let events_per_second = if total_wall_time_us > 0 {
        (total_events_processed as f64 / total_wall_time_us as f64) * 1_000_000.0
    } else {
        0.0
    };
    let throughput_runs_per_second = if total_wall_time_us > 0 {
        (units_completed as f64 / total_wall_time_us as f64) * 1_000_000.0
    } else {
        0.0
    };
    let peak_memory_estimate_bytes = completed_units
        .iter()
        .map(|u| u.estimated_peak_memory_bytes)
        .max()
        .unwrap_or(0);
    let total_storage_bytes = completed_units.iter().try_fold(0usize, |sum, unit| {
        sum.checked_add(unit.storage_bytes)
            .ok_or("profiling storage overflow")
    })?;
    let average_storage_bytes_per_result = total_storage_bytes
        .checked_div(units_completed)
        .unwrap_or(0);

    let profiling = PilotProfilingSummary {
        units_completed,
        units_failed,
        total_wall_time_us,
        total_events_processed,
        average_us_per_unit,
        min_us_per_unit,
        max_us_per_unit,
        events_per_second,
        throughput_runs_per_second,
        peak_memory_estimate_bytes,
        average_storage_bytes_per_result,
    };

    let roster_size = released_ids.len();
    let total_four_character_combinations = if roster_size >= 4 {
        roster_size * (roster_size - 1) * (roster_size - 2) * (roster_size - 3) / 24
    } else {
        0
    };
    let scenarios_per_combination = 5usize;
    let screening_seeds_per_scenario = 10usize;
    let projected_screening_runs = (total_four_character_combinations
        * scenarios_per_combination
        * screening_seeds_per_scenario) as u64;

    let finalist_pool_size = 50usize;
    let finalist_timings_count = 3usize;
    let finalist_seeds_per_timing = 1000usize;
    let projected_finalist_runs = (finalist_pool_size
        * scenarios_per_combination
        * finalist_timings_count
        * finalist_seeds_per_timing) as u64;

    let total_projected_runs = projected_screening_runs + projected_finalist_runs;

    let unit_seconds = average_us_per_unit / 1_000_000.0;
    let projected_screening_cpu_hours = (projected_screening_runs as f64 * unit_seconds) / 3600.0;
    let projected_finalist_cpu_hours = (projected_finalist_runs as f64 * unit_seconds) / 3600.0;
    let projected_total_cpu_hours = projected_screening_cpu_hours + projected_finalist_cpu_hours;

    let projected_storage_mb = (projected_finalist_runs as f64
        * average_storage_bytes_per_result as f64)
        / (1024.0 * 1024.0);

    let projection = StageGWorkloadProjection {
        total_four_character_combinations,
        scenarios_per_combination,
        screening_seeds_per_scenario,
        projected_screening_runs,
        finalist_pool_size,
        finalist_timings_count,
        finalist_seeds_per_timing,
        projected_finalist_runs,
        total_projected_runs,
        projected_screening_cpu_hours,
        projected_finalist_cpu_hours,
        projected_total_cpu_hours,
        projected_storage_mb,
        uncertainty_range_pct: None,
        notes: vec![
            "Stage G workload projection assumes two-phase search: 10-seed screening over all 8,855 combinations followed by 1,000-seed trials for top 50 finalists".into(),
            "Estimated CPU hours depend on host core count and parallelization scaling".into(),
            "Multi-target (4-target) scenarios process ~4x hit events, contributing to the ±25% execution variance".into(),
        ],
    };

    let mut pruned_receipts = Vec::new();
    if let Some(threshold_pct) = request
        .budget
        .screening_prune_threshold_pct
        .filter(|_| is_complete)
    {
        let mut team_dps_map: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
        for u in &completed_units {
            if u.timing_multiplier.numerator == u.timing_multiplier.denominator {
                team_dps_map
                    .entry((u.scenario.clone(), u.team.clone()))
                    .or_default()
                    .push(u.dps);
            }
        }

        let mut team_means: Vec<((String, String), f64)> = team_dps_map
            .into_iter()
            .map(|(team, dps_vals)| {
                let mean = dps_vals.iter().sum::<f64>() / dps_vals.len() as f64;
                (team, mean)
            })
            .collect();

        if !team_means.is_empty() {
            team_means.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut baselines = BTreeMap::<String, f64>::new();
            for ((scenario, _), score) in &team_means {
                baselines
                    .entry(scenario.clone())
                    .and_modify(|best| *best = best.max(*score))
                    .or_insert(*score);
            }
            {
                for ((scenario, team), mean_dps) in team_means {
                    let best_dps = baselines[&scenario];
                    if best_dps <= 0.0 {
                        continue;
                    }
                    let deficit_pct = ((best_dps - mean_dps) / best_dps) * 100.0;
                    if deficit_pct > threshold_pct {
                        pruned_receipts.push(PrunedCandidateReceipt {
                            scenario,
                            team: team.clone(),
                            score_dps: mean_dps,
                            baseline_dps: best_dps,
                            deficit_pct,
                            reason: format!(
                                "Candidate mean DPS ({:.1}) falls below screening threshold ({:.1}% deficit vs best {:.1})",
                                mean_dps, deficit_pct, best_dps
                            ),
                        });
                    }
                }
            }
        }
    }

    Ok(PilotReport {
        schema_version: 1,
        report_kind: VERSION.into(),
        content_id: "ntesim-current".into(),
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        request_sha256,
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
        profiling,
        projection,
        checkpoint,
        pruned_receipts,
        blockers: [
            "Native animation frame dispatch and input buffer timelines remain unverified".into(),
            "Profiling measurements reflect current partial/synthetic Rust execution kernel, not final native engine".into(),
            "Extrapolated Stage G workload is a planning projection without a measured uncertainty estimate".into(),
        ]
        .into_iter()
        .chain(failed_units.iter().map(|failure| {
            format!(
                "Pilot unit {} is unsupported: {}",
                failure.unit_index, failure.reason
            )
        }))
        .collect(),
        input_hashes: BTreeMap::new(),
    })
}
