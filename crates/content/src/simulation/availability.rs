//! Read-only combat prerequisites. Field/recovery/switch admission remains
//! mandatory when the caller later submits an actual input.
use super::{GameScenarioRequest, state::GameSimulatorState};

pub(super) struct AbilityAvailability {
    pub(super) ready: bool,
    pub(super) cooldown_us: u64,
}

pub(super) fn ability(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
    energy_cost: u64,
) -> AbilityAvailability {
    let actor = &state.actors[actor_index];
    let config = &request.actors[actor_index];
    let utility =
        crate::registry::utility_invocation(config, actor, ability_id, state.current_time_us);
    let energy_cost = if utility.is_some() { 0 } else { energy_cost };
    let cooldown_us = utility
        .map(|action| action.ready_at_us)
        .or_else(|| crate::registry::cooldown_override(config, actor, ability_id, damage_effect_id))
        .unwrap_or_else(|| {
            actor
                .ability_cooldowns
                .get(ability_id)
                .copied()
                .unwrap_or(0)
        })
        .saturating_sub(state.current_time_us);
    let ready = utility.is_none_or(|action| action.validate(request).is_ok())
        && actor.hp > 0
        && cooldown_us == 0
        && actor.energy.current() >= energy_cost
        && crate::registry::validate_ability_admission(
            request,
            config,
            actor,
            ability_id,
            state.current_time_us,
        )
        .is_ok()
        && crate::registry::validate_packet_admission(
            config,
            actor,
            ability_id,
            damage_effect_id,
            state.current_time_us,
        )
        .is_ok()
        && crate::registry::resource_admission_failure(
            request,
            request.actions.len(),
            config,
            actor,
            ability_id,
            state.current_time_us,
            state.combat_stamina.as_ref(),
        )
        .is_none();
    AbilityAvailability { ready, cooldown_us }
}
