use super::*;

pub fn compile_variant_scenario(variant: &PublishedVariant) -> Result<GameScenarioRequest, String> {
    compile_variant_scenario_with_overrides(variant, None)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActorOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_tier: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set_id: Option<String>,
    pub static_stats: GameActorStaticStats,
}

/// Construct a partial preview with optional caller actor overrides. Every
/// variant actor must have an override when a map is supplied; no fallback.
/// This function does not audit the supplied stats' source or build legality.
pub fn compile_variant_scenario_with_actor_overrides(
    variant: &PublishedVariant,
    actor_overrides: Option<&std::collections::BTreeMap<String, ActorOverride>>,
) -> Result<GameScenarioRequest, String> {
    let composition = variant.slots.join("+");
    let mut scenario = crate::profiling::build_pilot_scenario(
        &composition,
        crate::acceptance_spec::AcceptanceScenarioKind::SingleTargetReal120,
        Ratio {
            numerator: 1,
            denominator: 1,
        },
        42,
        actor_overrides,
    )?;
    // Published preview uses caller-supplied profile constants and gear proc
    // parameters, so it is a declared research hypothesis.
    scenario.model_assumptions.insert(
        "published.preview_inputs_v1".into(),
        "Published profile stats, action timing and caller-supplied gear proc parameters are preview hypotheses; build legality, native proc eligibility and complete combat coverage are unresolved.".into(),
    );
    scenario.model_assumptions.insert(
        "crit.fixed_noncrit_preview_v1".into(),
        "Published variant baseline uses fixed non-critical hit receipts; native critical eligibility and roll behavior are not evaluated.".into(),
    );
    scenario.stochastic_crit = None;
    // Keep the same post-action preview window after any fixture entry offset.
    scenario.relative_duration_us =
        20_000_000 + crate::profiling::preview_entry_offset_us(&composition);
    if let Some(GameActionCommand::Wait {
        timestamp_us,
        duration_us,
    }) = scenario.actions.last_mut()
    {
        *duration_us = scenario
            .relative_duration_us
            .checked_sub(*timestamp_us)
            .ok_or("published preview inputs exceed their horizon")?;
    }
    if actor_overrides.is_none() {
        for actor in &mut scenario.actors {
            actor.static_stats.crit_rate = Ratio {
                numerator: 1,
                denominator: 20,
            };
            actor.static_stats.crit_damage_bonus = Ratio {
                numerator: 1,
                denominator: 2,
            };
        }
    }
    Ok(scenario)
}

/// Construct a partial preview with optional caller static overrides. Every
/// variant actor must have an override when a map is supplied; no fallback.
/// This function does not audit the supplied stats' source or build legality.
pub fn compile_variant_scenario_with_overrides(
    variant: &PublishedVariant,
    stat_overrides: Option<&std::collections::BTreeMap<String, GameActorStaticStats>>,
) -> Result<GameScenarioRequest, String> {
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
    compile_variant_scenario_with_actor_overrides(variant, actor_overrides.as_ref())
}
