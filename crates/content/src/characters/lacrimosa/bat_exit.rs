//! Native ordinary-Evade carrier creates the short Cang tag at role-out begin.
use crate::runtime_api::CallbackContext;
use crate::simulation::state::ActorRuntime;

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::LacrimosaBatEvadeExitV1;

pub(crate) fn recovery_override(
    request: &crate::simulation::GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    let super::super::RuntimeState::Lacrimosa(state) = &actor.character else {
        return None;
    };
    (ability == "switch_actor"
        && state.bat_evade_until_us > now
        && actor.hp > 0
        && actor.locked_until_us > now
        && request
            .model_assumptions
            .get(HYPOTHESIS_ID.as_str())
            .map(String::as_str)
            == Some(HYPOTHESIS_ID.description()))
    .then_some((now, HYPOTHESIS_ID))
}

pub(crate) fn is_marked(actor: &ActorRuntime, now: u64) -> bool {
    let super::super::RuntimeState::Lacrimosa(state) = &actor.character else {
        return false;
    };
    actor.hp > 0
        && (state.baicang_tag_until_us > now
            || (state.bat_evade_until_us > now && actor.locked_until_us > now))
}

pub(crate) fn on_exit(ctx: &mut CallbackContext<'_>, actor: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let runtime = &mut ctx.state.actors[actor];
    let recovering = runtime.locked_until_us > now;
    let state = runtime.character.lacrimosa_mut()?;
    if state.bat_evade_until_us > now && recovering {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            HYPOTHESIS_ID,
        )?;
        state.baicang_tag_until_us = now
            .checked_add(1_500_000)
            .ok_or("Lacrimosa Cang tag deadline overflow")?;
        runtime.locked_until_us = now;
    }
    state.bat_evade_until_us = 0;
    Ok(())
}
