use super::parameters::PARAMETERS;
use crate::HypothesisId;
use crate::simulation::{
    ExactFraction as F, GameActorConfig, GameScenarioRequest, state::ActorRuntime,
};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffManager, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::CharacterZankouCrimsonReverieV1;
pub(crate) const KINDLING_CAP: u32 = 1;

pub(crate) fn initial_kindling(initial: Option<crate::formula::Ratio>) -> Result<u32, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(KINDLING_CAP) {
        return Err("Zankou Kindling must be integral in0..1.".into());
    }
    u32::try_from(value.numerator).map_err(|_| "Zankou initial Kindling overflow".into())
}

/// One combat-entry initialization; subsequent switches never call this hook.
pub(crate) fn initialize_actor(actor: &mut ActorRuntime) -> Result<(), String> {
    if actor.hp > 0 {
        actor.initialize_cycle(PARAMETERS.combat_entry_cycle)?;
    }
    Ok(())
}

pub(crate) fn initialize_team(
    config: &GameActorConfig,
    actor_index: usize,
    buffs: &mut BuffManager,
    request: &GameScenarioRequest,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id != "character:1036" {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
    if config.awakening.has_effect(1) {
        crate::runtime_api::record_hypothesis_id(request, assumptions, super::modifiers::MODEL)?;
    }
    buffs
        .apply_buff(
            BuffDefinition {
                id: "zankou.crimson_reverie_ci".into(),
                source: config.character_id.clone(),
                recipient: BuffRecipient::SelfActor,
                source_lifetime: BuffSourceLifetime::Snapshot,
                stacking_scope: BuffStackingScope::PerSource,
                duration_us: None,
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Disallow,
                decay_policy: DecayPolicy::AllAtOnce,
                semantic_tags: Default::default(),
                stat_modifiers: BTreeMap::from([(
                    "cycle_intensity".into(),
                    PARAMETERS.self_cycle_intensity,
                )]),
            },
            actor_index,
            0,
        )
        .map_err(|error| format!("Zankou Crimson Reverie initialization failed: {error:?}"))
}
