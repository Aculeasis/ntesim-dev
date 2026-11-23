use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantExecutionSummary {
    #[serde(flatten)]
    pub readiness: ScenarioReadiness,
    pub variant_index: usize,
    pub archetype: String,
    pub tier: String,
    pub slots: [String; 4],
    pub status: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub hits_settled: Option<usize>,
    pub total_damage_dealt: Option<f64>,
    pub energy_conserved: Option<bool>,
    pub invariants_passed: Option<bool>,
    pub final_target_hp: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VariantBatchReport {
    #[serde(flatten)]
    pub readiness: ScenarioReadiness,
    pub schema_version: u32,
    pub content_id: String,
    pub total_variants: usize,
    pub passed_variants: usize,
    pub failed_variants: usize,
    pub all_invariants_passed: bool,
    pub all_energy_conserved: bool,
    pub variants: Vec<VariantExecutionSummary>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}

pub fn run_published_batch_with_actor_overrides(
    csv_content: &str,
    actor_overrides: Option<&std::collections::BTreeMap<String, ActorOverride>>,
    limits: &ExecutionLimits,
) -> Result<VariantBatchReport, String> {
    let variants = parse_published_variants_csv(csv_content)?;
    let mut results = Vec::with_capacity(variants.len());
    let mut passed_count = 0usize;
    let mut failed_count = 0usize;
    let mut all_invariants_passed = true;
    let mut all_energy_conserved = true;

    for variant in &variants {
        let scenario = match compile_variant_scenario_with_actor_overrides(variant, actor_overrides)
        {
            Ok(scenario) => scenario,
            Err(reason) => {
                failed_count += 1;
                all_invariants_passed = false;
                all_energy_conserved = false;
                results.push(failed_variant_summary(variant, "unsupported", reason));
                continue;
            }
        };
        let report_value = match run_game_scenario(&scenario, limits) {
            Ok(report) => report,
            Err(reason) => {
                failed_count += 1;
                all_invariants_passed = false;
                all_energy_conserved = false;
                results.push(failed_variant_summary(variant, "failed", reason));
                continue;
            }
        };
        let report: GameScenarioReport = match serde_json::from_value(report_value) {
            Ok(report) => report,
            Err(error) => {
                failed_count += 1;
                all_invariants_passed = false;
                all_energy_conserved = false;
                results.push(failed_variant_summary(
                    variant,
                    "failed",
                    format!("invalid scenario report: {error}"),
                ));
                continue;
            }
        };

        let variant_energy_conserved = report.final_actor_states.iter().all(|a| a.energy_conserved);
        let execution_validation = crate::trials::eligibility::validate_completed(&report);
        let variant_passed = execution_validation.is_ok() && report.hits_settled > 0;
        let failure_reason = if variant_passed {
            None
        } else {
            Some(
                execution_validation
                    .err()
                    .unwrap_or_else(|| "completed report contained no settled hits".into()),
            )
        };

        if variant_passed {
            passed_count += 1;
        } else {
            failed_count += 1;
        }

        if !report.invariants_passed {
            all_invariants_passed = false;
        }
        if !variant_energy_conserved {
            all_energy_conserved = false;
        }

        let final_hp = report
            .final_target_states
            .first()
            .map(|t| t.hp_current)
            .unwrap_or(0);

        results.push(VariantExecutionSummary {
            readiness: preview_readiness(),
            variant_index: variant.variant_index,
            archetype: variant.archetype.clone(),
            tier: variant.tier.clone(),
            slots: variant.slots.clone(),
            status: if variant_passed {
                "completed"
            } else {
                "failed"
            }
            .into(),
            success: variant_passed,
            failure_reason,
            hits_settled: Some(report.hits_settled),
            total_damage_dealt: Some(report.total_damage_dealt),
            energy_conserved: Some(variant_energy_conserved),
            invariants_passed: Some(report.invariants_passed),
            final_target_hp: Some(final_hp),
        });
    }

    Ok(VariantBatchReport {
        readiness: preview_readiness(),
        schema_version: 1,
        content_id: "ntesim-current".into(),
        total_variants: variants.len(),
        passed_variants: passed_count,
        failed_variants: failed_count,
        all_invariants_passed,
        all_energy_conserved,
        variants: results,
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
    })
}

fn failed_variant_summary(
    variant: &PublishedVariant,
    status: &str,
    reason: String,
) -> VariantExecutionSummary {
    VariantExecutionSummary {
        readiness: preview_readiness(),
        variant_index: variant.variant_index,
        archetype: variant.archetype.clone(),
        tier: variant.tier.clone(),
        slots: variant.slots.clone(),
        status: status.into(),
        success: false,
        failure_reason: Some(reason),
        hits_settled: None,
        total_damage_dealt: None,
        energy_conserved: None,
        invariants_passed: None,
        final_target_hp: None,
    }
}

pub fn run_published_batch_with_overrides(
    csv_content: &str,
    stat_overrides: Option<&std::collections::BTreeMap<String, GameActorStaticStats>>,
    limits: &ExecutionLimits,
) -> Result<VariantBatchReport, String> {
    let actor_overrides = stat_overrides.map(|m| {
        m.iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    ActorOverride {
                        weapon_id: None,
                        weapon_tier: None,
                        set_id: None,
                        static_stats: v.clone(),
                    },
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    });
    run_published_batch_with_actor_overrides(csv_content, actor_overrides.as_ref(), limits)
}

pub fn run_published_batch(
    csv_content: &str,
    limits: &ExecutionLimits,
) -> Result<VariantBatchReport, String> {
    run_published_batch_with_actor_overrides(csv_content, None, limits)
}
