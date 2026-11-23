//! Pure source-coordinate cancel gates; cancellation happens only after admission.
use super::*;
use crate::runtime_api::CallbackContext;
use crate::simulation::PlayerInput;
pub(crate) fn recovery(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !super::super::effects::accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Fadia(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let Some(action) = &clock.active else {
        return (input == "jump").then_some((now, super::super::effects::MODEL));
    };
    let progress = match input {
        "jump" => action.jump,
        "GA_Fadia_Skill" => action.skill_gate,
        "dash" | "GA_Fadia_Melee" | "GA_Fadia_UltraSkill" => action.recovery,
        _ => return None,
    };
    Some((
        clock
            .lag
            .resolve_real_deadline(action.owner, action.start, progress)
            .ok()?,
        super::super::effects::MODEL,
    ))
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if !super::super::effects::accepted(ctx.request) {
        return Ok(());
    }
    stop(&mut ctx.state.actors[owner], ctx.queue)?;
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.fadia_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    if input == PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        let mut clock = state.clock.take().ok_or("Fadia Dodge clock missing")?;
        clock.active = Some(Action {
            owner,
            command,
            start: now,
            recovery: if back { 1007017 } else { 1006571 },
            jump: if back { 0 } else { 82544 },
            skill_gate: if back { 494283 } else { 333754 },
            skill: false,
            enhanced: false,
            a5: false,
            groups: Default::default(),
            contacts: Default::default(),
        });
        project(&clock, &mut ctx.state.actors[owner], ctx.queue, now, true)?;
        ctx.state.actors[owner].character.fadia_mut()?.clock = Some(clock);
    }
    Ok(())
}
