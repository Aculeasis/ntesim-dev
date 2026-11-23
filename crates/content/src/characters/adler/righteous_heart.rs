use crate::HypothesisId;
use crate::simulation::{GameActorConfig, GameScenarioRequest};
use ntesim_core::buffs::dispatch::{
    BuffDefinition, BuffManager, BuffRecipient, BuffSourceLifetime, BuffStackingScope, DecayPolicy,
    RefreshPolicy,
};
use std::collections::BTreeMap;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::AdlerRighteousHeartV1;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn initialize_team(
    config: &GameActorConfig,
    actor_index: usize,
    buffs: &mut BuffManager,
    request: &GameScenarioRequest,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    if config.character_id != "character:1033" || !accepted(request) {
        return Ok(());
    }
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    if progression["breakthrough"]
        .as_u64()
        .ok_or("Adler breakthrough missing")?
        < 4
    {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
    buffs
        .apply_buff(
            BuffDefinition {
                id: "adler.righteous_heart_def".into(),
                source: config.character_id.clone(),
                recipient: BuffRecipient::SpecificActor(actor_index),
                source_lifetime: BuffSourceLifetime::Snapshot,
                stacking_scope: BuffStackingScope::PerSource,
                duration_us: None,
                max_stacks: 1,
                refresh_policy: RefreshPolicy::Disallow,
                decay_policy: DecayPolicy::AllAtOnce,
                semantic_tags: Default::default(),
                stat_modifiers: BTreeMap::from([(
                    "def_percent".into(),
                    crate::formula::Ratio {
                        numerator: 1,
                        denominator: 5,
                    },
                )]),
            },
            actor_index,
            0,
        )
        .map_err(|error| format!("Adler Righteous Heart initialization failed: {error:?}"))
}
