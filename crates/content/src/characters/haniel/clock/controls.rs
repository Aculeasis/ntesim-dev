//! Source cancel gates and committed paid player controls.
use super::*;
pub(crate) fn recovery(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !super::super::secondary::accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Haniel(state) = &actor.character else {
        return None;
    };
    let Some(action) = state.clock.as_ref()?.active.as_ref() else {
        return (input == "jump").then_some((now, super::super::secondary::MODEL));
    };
    let normal_recovery = match action.kind {
        Kind::Normal(stage) => RECOVERY[stage],
        Kind::Entry => 896_381,
        Kind::Skill => 1_309_354,
        Kind::Burst => 4_240_809,
        Kind::Charge => 1_211_743,
        Kind::FastCharge => 195_077,
        Kind::Plunge => 175_616,
        Kind::Riposte => 957_007,
        Kind::Support => 1_177_904,
        Kind::Dodge(back) => {
            if back {
                1_005_710
            } else {
                836_769
            }
        }
    };
    let gate = match (action.kind, input) {
        (Kind::Normal(stage), "jump") => [208125, 522308, 439603, 758749, 831093][stage],
        (Kind::Entry, "jump") => 208_125,
        (Kind::Skill, "jump") => 1_229_058,
        (Kind::Charge, "jump") => 1_220_839,
        (Kind::FastCharge, "jump") => 204_172,
        (Kind::Riposte, "jump" | SKILL) => 514_085,
        (Kind::Dodge(back), "jump") => {
            if back {
                0
            } else {
                94_234
            }
        }
        (Kind::Dodge(back), SKILL) => {
            if back {
                494_283
            } else {
                333_611
            }
        }
        (_, "jump" | "dash" | "GA_Haniel_Melee" | SKILL | BURST) => normal_recovery,
        _ => return None,
    };
    Some((
        action.start.checked_add(gate)?.checked_add(action.pause)?,
        super::super::secondary::MODEL,
    ))
}
pub(crate) fn dodging(actor: &ActorRuntime, now: u64) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Haniel(s) if s.clock.as_ref().and_then(|c|c.active.as_ref()).is_some_and(|a|matches!(a.kind,Kind::Dodge(_))) && now<actor.locked_until_us)
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    if !super::super::secondary::accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    stop(actor, ctx.queue, now)?;
    let state = actor.character.haniel_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    if input == crate::simulation::PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        state
            .clock
            .as_mut()
            .ok_or("Haniel Dodge clock missing")?
            .active = Some(Action {
            owner,
            command,
            start: now,
            kind: Kind::Dodge(back),
            pause: 0,
            contacts: Default::default(),
            groups: Default::default(),
            skill: false,
            cooldown_started: false,
            charge: None,
        });
        actor.locked_until_us = now
            .checked_add(if back { 1_005_710 } else { 836_769 })
            .ok_or("Haniel Dodge recovery overflow")?;
    }
    Ok(())
}
