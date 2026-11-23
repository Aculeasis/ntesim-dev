use super::*;

/// Run each published preview once and report its baseline observations.
/// Legal candidate builds and optimized comparisons remain unsupported until
/// the search is connected to the frozen equipment compiler and its legality
/// receipts.
pub fn run_published_search(
    csv_content: &str,
    limits: &ExecutionLimits,
) -> Result<BuildSearchResult, String> {
    let variants = parse_published_variants_csv(csv_content)?;
    let mut comparisons = Vec::with_capacity(variants.len());
    let mut total_baseline_dmg = 0.0f64;
    let mut failed_count = 0usize;
    let mut baseline_values_complete = true;

    for variant in &variants {
        let result = (|| {
            let request = compile_variant_scenario_with_overrides(variant, None)?;
            let value = run_game_scenario(&request, limits)?;
            let report: GameScenarioReport = serde_json::from_value(value)
                .map_err(|error| format!("failed to parse baseline scenario report: {error}"))?;
            crate::trials::eligibility::validate_completed(&report)?;
            Ok::<_, String>(report)
        })();

        match result {
            Ok(report) => {
                total_baseline_dmg += report.total_damage_dealt;
                comparisons.push(VariantComparisonResult {
                    readiness: preview_readiness(),
                    variant_index: variant.variant_index,
                    archetype: variant.archetype.clone(),
                    tier: variant.tier.clone(),
                    slots: variant.slots.clone(),
                    status: "completed".into(),
                    failure_reason: None,
                    baseline_damage: Some(report.total_damage_dealt),
                    optimized_damage: None,
                    damage_delta: None,
                    percentage_gain: None,
                    settled_hits: Some(report.hits_settled),
                    invariants_verified: Some(report.invariants_passed),
                });
            }
            Err(reason) => {
                failed_count += 1;
                baseline_values_complete = false;
                comparisons.push(VariantComparisonResult {
                    readiness: preview_readiness(),
                    variant_index: variant.variant_index,
                    archetype: variant.archetype.clone(),
                    tier: variant.tier.clone(),
                    slots: variant.slots.clone(),
                    status: "failed".into(),
                    failure_reason: Some(reason),
                    baseline_damage: None,
                    optimized_damage: None,
                    damage_delta: None,
                    percentage_gain: None,
                    settled_hits: None,
                    invariants_verified: None,
                });
            }
        }
    }

    let mut readiness = preview_readiness();
    readiness.blockers.push(
        "Legal build candidates and optimized damage rankings are unsupported until the shared equipment compiler and legality receipts are connected".into(),
    );
    if failed_count > 0 {
        readiness.blockers.push(format!(
            "{failed_count} published variant baselines failed and were excluded from aggregate damage"
        ));
    }
    let total_archetypes = variants
        .iter()
        .map(|variant| variant.archetype.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    Ok(BuildSearchResult {
        readiness,
        schema_version: 1,
        content_id: "ntesim-current".into(),
        total_variants: variants.len(),
        total_archetypes,
        total_characters: 0,
        character_builds: BTreeMap::new(),
        variant_comparisons: comparisons,
        archetype_rankings: Vec::new(),
        total_baseline_damage: baseline_values_complete.then_some(total_baseline_dmg),
        total_optimized_damage: None,
        total_percentage_gain: None,
        execution_status: if failed_count == 0 {
            "BASELINE_ONLY"
        } else {
            "BASELINE_PARTIAL"
        }
        .into(),
    })
}
