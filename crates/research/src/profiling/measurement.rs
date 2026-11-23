use super::*;

pub(crate) fn execute_pilot_unit(
    unit: &PilotExecutionUnit,
    request: &PilotRequest,
    limits: &ExecutionLimits,
) -> Result<PilotUnitResult, String> {
    let scenario = build_pilot_scenario(
        &unit.team,
        unit.scenario_kind,
        unit.timing_multiplier,
        unit.seed,
        request.actor_overrides.as_ref(),
    )?;

    let start = Instant::now();
    let report_val = simulation::run_game_scenario(&scenario, limits)?;
    let elapsed_us = start.elapsed().as_micros() as u64;

    let report: GameScenarioReport = serde_json::from_value(report_val)
        .map_err(|e| format!("failed to deserialize scenario report: {e}"))?;
    crate::trials::eligibility::validate_completed(&report)
        .map_err(|error| format!("pilot unit {}: {error}", unit.unit_index))?;

    let damage_exact = report.total_damage_exact;
    let damage_f64 = damage_exact.to_f64();
    let horizon_sec = scenario.relative_duration_us as f64 / 1_000_000.0;
    let dps = if horizon_sec > 0.0 {
        damage_f64 / horizon_sec
    } else {
        0.0
    };

    let estimated_peak_memory_bytes =
        report.total_events_processed * 128 + scenario.actions.len() * 256 + 64 * 1024;

    let unit_json = serde_json::json!({
        "unit_index": unit.unit_index,
        "team": unit.team,
        "scenario": unit.scenario_kind.code(),
        "timing": format!("{}/{}", unit.timing_multiplier.numerator, unit.timing_multiplier.denominator),
        "seed": unit.seed,
        "damage_exact": {
            "numerator": damage_exact.numerator.to_string(),
            "denominator": damage_exact.denominator.to_string()
        },
        "events_processed": report.total_events_processed,
    });
    let unit_bytes = serde_json::to_vec(&unit_json).map_err(|e| e.to_string())?;
    let storage_bytes = unit_bytes.len();
    let unit_sha256 = sha256(&unit_bytes);

    Ok(PilotUnitResult {
        unit_index: unit.unit_index,
        team: unit.team.clone(),
        scenario: unit.scenario_kind.code().to_string(),
        timing_multiplier: unit.timing_multiplier,
        seed: unit.seed,
        damage_exact,
        damage_f64,
        dps,
        events_processed: report.total_events_processed,
        elapsed_us,
        estimated_peak_memory_bytes,
        storage_bytes,
        unit_sha256,
    })
}

pub(crate) fn deterministic_unit_hash(unit: &PilotUnitResult) -> Result<String, String> {
    let payload = serde_json::json!({
        "unit_index":unit.unit_index,"team":unit.team,"scenario":unit.scenario,
        "timing":format!("{}/{}",unit.timing_multiplier.numerator,unit.timing_multiplier.denominator),
        "seed":unit.seed,"damage_exact":{"numerator":unit.damage_exact.numerator.to_string(),"denominator":unit.damage_exact.denominator.to_string()},"events_processed":unit.events_processed,
    });
    Ok(sha256(
        &serde_json::to_vec(&payload).map_err(|e| e.to_string())?,
    ))
}

pub(crate) fn failed_unit_receipt(
    unit: &PilotExecutionUnit,
    reason: String,
) -> Result<PilotUnitFailure, String> {
    let payload = serde_json::json!({
        "unit_index": unit.unit_index,
        "team": unit.team,
        "scenario": unit.scenario_kind.code(),
        "timing": format!("{}/{}", unit.timing_multiplier.numerator, unit.timing_multiplier.denominator),
        "seed": unit.seed,
        "reason": reason,
    });
    let failure_sha256 = sha256(&serde_json::to_vec(&payload).map_err(|e| e.to_string())?);
    Ok(PilotUnitFailure {
        unit_index: unit.unit_index,
        team: unit.team.clone(),
        scenario: unit.scenario_kind.code().to_string(),
        timing_multiplier: unit.timing_multiplier,
        seed: unit.seed,
        reason,
        failure_sha256,
    })
}

pub(crate) fn deterministic_failure_hash(failure: &PilotUnitFailure) -> Result<String, String> {
    let payload = serde_json::json!({
        "unit_index": failure.unit_index,
        "team": failure.team,
        "scenario": failure.scenario,
        "timing": format!("{}/{}", failure.timing_multiplier.numerator, failure.timing_multiplier.denominator),
        "seed": failure.seed,
        "reason": failure.reason,
    });
    Ok(sha256(
        &serde_json::to_vec(&payload).map_err(|e| e.to_string())?,
    ))
}

pub(crate) fn checkpoint_hash(checkpoint: &PilotCheckpoint) -> Result<String, String> {
    let mut payload = serde_json::to_value(checkpoint).map_err(|e| e.to_string())?;
    payload
        .as_object_mut()
        .ok_or("checkpoint must be an object")?
        .remove("checkpoint_id");
    Ok(sha256(
        &serde_json::to_vec(&payload).map_err(|e| e.to_string())?,
    ))
}
