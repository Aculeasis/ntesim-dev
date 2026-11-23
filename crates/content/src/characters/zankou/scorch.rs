use crate::HypothesisId;
use crate::runtime_api::ScorchEnhancement;
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::CharacterZankouSilentSunsetV1;

pub(crate) fn policy(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<Option<ScorchEnhancement>, String> {
    let owner = request.actors.iter().enumerate().find(|(index, actor)| {
        actor.character_id == "character:1036" && state.actors[*index].hp > 0
    });
    let Some((owner_actor_index, config)) = owner else {
        return Ok(None);
    };
    crate::runtime_api::record_hypothesis_id(request, assumptions, HYPOTHESIS_ID)?;
    let set_max_stacks = if config.awakening.count() >= 6 {
        state.actors[owner_actor_index]
            .character
            .zankou()
            .is_ok_and(|zankou| zankou.r6_enhancement_until_us > state.current_time_us)
    } else {
        false
    };
    Ok(Some(ScorchEnhancement {
        owner_actor_index,
        stack_cap: 3,
        set_max_stacks,
    }))
}

pub(crate) fn dot_eligible(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    owner_actor_index: usize,
    damage_effect_id: &str,
    semantic_tags: &BTreeSet<String>,
) -> bool {
    request.actors[owner_actor_index].character_id == "character:1036"
        && state.actors[owner_actor_index].hp > 0
        && semantic_tags.contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG)
        && damage_effect_id != "ntesim.reaction.scorch"
}
