use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{GameScenarioRequest, state::GameSimulatorState};

const WINDOW_US: u64 = 5_000_000;
const COOLDOWN_US: u64 = 5_000_000;
const MODEL: crate::HypothesisId = crate::HypothesisId::LacrimosaA6FreeReactionEntryV1;

pub(super) fn arm(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx.actor_index != ctx.active_actor_index || !ctx.config.awakening.has_effect(6) {
        return Ok(());
    }
    let state = ctx.actor.character.lacrimosa_mut()?;
    state.free_support_until_us = ctx
        .world_time_us
        .checked_add(WINDOW_US)
        .ok_or("Lacrimosa A6 Support window overflow")?;
    ctx.record_hypothesis(MODEL)
}

pub(crate) fn ready(
    request: &GameScenarioRequest,
    state: &GameSimulatorState,
    incoming: usize,
) -> bool {
    let actor = &state.actors[incoming];
    let now = state.mode_clock.relative_us();
    request.actors[incoming].awakening.has_effect(6)
        && actor.locked_until_us <= state.current_time_us
        && matches!(&actor.character, crate::characters::RuntimeState::Lacrimosa(owner)
            if owner.free_support_until_us > now && owner.free_support_cd_until_us <= now)
}

pub(crate) fn consume(ctx: &mut CallbackContext<'_>, incoming: usize) -> Result<(), String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let cooldown = ctx
        .state
        .mode_clock
        .relative_us()
        .checked_add(COOLDOWN_US)
        .ok_or("Lacrimosa A6 Support cooldown overflow")?;
    let state = ctx.state.actors[incoming].character.lacrimosa_mut()?;
    state.free_support_until_us = 0;
    state.free_support_cd_until_us = cooldown;
    Ok(())
}
