//! Actual owner gates and remaining hit-pause inheritance across montage inputs.
use crate::runtime_api::CallbackContext;
use crate::simulation::{GameScenarioRequest, PlayerInput, state::ActorRuntime};
use ntesim_core::kernel::timing::HitlagWindow;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizActionCancelsV1;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
    if accepted(request) && !super::pressed::accepted(request) {
        return Err("Chiz action cancels require pressed_skill_v1 and ordinary_clock_v1".into());
    }
    Ok(())
}
pub(crate) fn inherited_windows(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    now: u64,
) -> Vec<HitlagWindow> {
    if accepted(request) {
        let mut windows = super::pressed::live_windows(actor, now);
        if super::secondary::accepted(request) {
            windows.extend(super::clock::live_windows(actor, now));
        }
        windows
    } else {
        Vec::new()
    }
}
pub(crate) fn recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(request) {
        return None;
    }
    if super::secondary::accepted(request)
        && let Some(gate) = super::clock::input_gate(actor, input)
    {
        return Some((gate, super::secondary::MODEL));
    }
    if let Some(gate) = super::pressed::input_gate(actor, input) {
        return Some((gate, MODEL));
    }
    if input == "jump" {
        if let Some(gate) = super::clock::jump_gate(actor) {
            return Some((gate, MODEL));
        }
        if !super::pressed::has_active(actor) && !super::clock::has_active(actor) {
            return Some((now, MODEL));
        }
    }
    None
}
pub(crate) fn player_input(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
    input: PlayerInput,
) -> Result<(), String> {
    if input == PlayerInput::Jump && accepted(request) {
        super::pressed::mark_jump(actor)?;
    }
    Ok(())
}
pub(crate) fn committed_input(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if input != PlayerInput::Jump || !accepted(ctx.request) {
        return Ok(());
    }
    super::pressed::commit_jump(&mut ctx.state.actors[owner], ctx.queue)?;
    super::clock::stop(
        &mut ctx.state.actors[owner],
        ctx.queue,
        ctx.state.current_time_us,
    )?;
    let state = ctx.state.actors[owner].character.chiz_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(())
}
