//! Screening over every declared seed in one scenario; partial executions cannot rank.
use super::*;

pub(crate) fn result_key(scenario: AcceptanceScenarioKind, composition: &str) -> String {
    format!("{}:{composition}", scenario.code())
}

pub(crate) fn summary_hash(summary: &CompositionScreeningSummary) -> Result<String, String> {
    let mut value = serde_json::to_value(summary).map_err(|error| error.to_string())?;
    value
        .as_object_mut()
        .ok_or("summary must be an object")?
        .remove("result_sha256");
    Ok(sha256(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

pub(crate) fn screen_single_composition(
    comp_record: &crate::composition::CompositionRecord,
    scenario_kind: AcceptanceScenarioKind,
    seeds: &[u64],
    limits: &ExecutionLimits,
) -> Result<CompositionScreeningSummary, String> {
    let mut sum = ExactFraction::new(0, 1)?;
    let mut events_processed = 0usize;
    for seed in seeds {
        let scenario = build_pilot_scenario(
            &comp_record.composition,
            scenario_kind,
            Ratio {
                numerator: 1,
                denominator: 1,
            },
            *seed,
            None,
        )?;
        let value = crate::simulation::run_game_scenario(&scenario, limits)?;
        let report: crate::simulation::GameScenarioReport =
            serde_json::from_value(value).map_err(|error| error.to_string())?;
        crate::trials::eligibility::validate_completed(&report)?;
        sum = sum.add(report.total_damage_exact)?;
        events_processed = events_processed
            .checked_add(report.total_events_processed)
            .ok_or("screening event count overflow")?;
    }
    let mean = sum.div(ExactFraction::new(seeds.len() as u128, 1)?)?;
    let mut summary = CompositionScreeningSummary {
        scenario: scenario_kind,
        seeds: seeds.to_vec(),
        composition_index: comp_record.index,
        composition: comp_record.composition.clone(),
        is_published_variant: comp_record.is_published_variant,
        matched_archetype: comp_record
            .matched_variant
            .as_ref()
            .map(|matched| matched.archetype.clone()),
        nominal_dps: mean.to_f64() / (scenario_kind.horizon_us() as f64 / 1_000_000.0),
        nominal_damage_exact: mean,
        events_processed,
        invariants_passed: true,
        energy_conserved: true,
        result_sha256: String::new(),
    };
    summary.result_sha256 = summary_hash(&summary)?;
    Ok(summary)
}

pub(crate) fn failure_summary(
    comp_record: &crate::composition::CompositionRecord,
    scenario: AcceptanceScenarioKind,
    seeds: &[u64],
    reason: String,
) -> Result<CompositionScreeningFailure, String> {
    let mut failure = CompositionScreeningFailure {
        scenario,
        seeds: seeds.to_vec(),
        composition_index: comp_record.index,
        composition: comp_record.composition.clone(),
        is_published_variant: comp_record.is_published_variant,
        matched_archetype: comp_record
            .matched_variant
            .as_ref()
            .map(|matched| matched.archetype.clone()),
        reason,
        failure_sha256: String::new(),
    };
    failure.failure_sha256 = failure_hash(&failure)?;
    Ok(failure)
}

pub(crate) fn failure_hash(failure: &CompositionScreeningFailure) -> Result<String, String> {
    let payload = serde_json::json!({
        "scenario": failure.scenario,
        "seeds": failure.seeds,
        "composition_index": failure.composition_index,
        "composition": failure.composition,
        "is_published_variant": failure.is_published_variant,
        "matched_archetype": failure.matched_archetype,
        "reason": failure.reason,
    });
    Ok(sha256(
        &serde_json::to_vec(&payload).map_err(|error| error.to_string())?,
    ))
}
