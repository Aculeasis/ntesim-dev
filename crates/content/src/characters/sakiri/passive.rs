//! Sakiri Passive 1 (Breakthrough >= 2, `GA_Sagiri_Passive_1` -> `Buff_Reaction_ZhouAn8_1003`)
//! and Passive 2 (Breakthrough >= 4, `GA_Sagiri_Passive_2` -> `Buff_Sagiri003_Passive1`).
use crate::characters::packet_semantics::DOT_DAMAGE_TAG;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest};

const SOURCE: &str = "character:1003";
const PASSIVE1_MIN_BREAKTHROUGH: u64 = 2;
const PASSIVE2_MIN_BREAKTHROUGH: u64 = 4;
const SCORCH_DOT_MAX_HANDLES: usize = 4;

fn breakthrough(config: &GameActorConfig) -> Result<u64, String> {
    let progression = ntesim_data::skill_levels::progression(
        &config.character_id,
        config.character_level,
        config.character_max_level,
    )?;
    Ok(progression["breakthrough"].as_u64().unwrap_or(0))
}

pub(crate) fn passive1_unlocked(config: &GameActorConfig) -> Result<bool, String> {
    Ok(config.character_id == SOURCE && breakthrough(config)? >= PASSIVE1_MIN_BREAKTHROUGH)
}

pub(crate) fn passive2_unlocked(config: &GameActorConfig) -> Result<bool, String> {
    Ok(config.character_id == SOURCE && breakthrough(config)? >= PASSIVE2_MIN_BREAKTHROUGH)
}

pub(crate) fn scorch_extension_eligible(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
) -> bool {
    request.actors.iter().enumerate().any(|(index, actor)| {
        state.actors.get(index).is_some_and(|rt| rt.hp > 0)
            && passive1_unlocked(actor).unwrap_or(false)
    })
}

/// Active DoT handle count on `target_index` while Scorch is active (`Buff_Reaction_ZhouAn8_1003`
/// / `Cau_AnHunZhou8_1003`: `+0.25 * handles` FinalDamageUp, capped at `+1.0`).
pub(crate) fn scorch_dot_final_bonus(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    target_index: usize,
    now_us: u64,
) -> Result<Option<ExactFraction>, String> {
    if !scorch_extension_eligible(request, state) {
        return Ok(None);
    }
    let scorch_active = state
        .reactions
        .scorch
        .get(&target_index)
        .is_some_and(|scorch| scorch.expires_at_us > now_us);
    if !scorch_active {
        return Ok(None);
    }
    let mut handles: usize = 1; // Active Scorch carrier (`Buff_Reaction_5_new`) carries `State.Damage.Dot`.
    for (actor_index, config) in request.actors.iter().enumerate() {
        let Some(actor) = state.actors.get(actor_index) else {
            continue;
        };
        let foreign_stacks = crate::registry::lookup_kit(&config.character_id).map_or(0, |kit| {
            kit.attached_dot_stacks(actor, target_index, now_us)
        });
        if foreign_stacks > 0 {
            handles = handles.saturating_add(1);
        }
        if config.character_id == "character:1036" {
            let zankou_trains =
                crate::characters::zankou::vile_ash::carriers(actor, target_index, now_us).len();
            handles = handles.saturating_add(zankou_trains);
        }
    }
    let capped = handles.min(SCORCH_DOT_MAX_HANDLES) as u128;
    Ok(Some(ExactFraction::new(capped, 4)?))
}

pub(crate) fn packet_dot_final_bonus(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    target_index: usize,
    semantic_tags: &std::collections::BTreeSet<String>,
    now_us: u64,
) -> Result<Option<ExactFraction>, String> {
    if !semantic_tags.contains(DOT_DAMAGE_TAG) {
        return Ok(None);
    }
    scorch_dot_final_bonus(request, state, target_index, now_us)
}
