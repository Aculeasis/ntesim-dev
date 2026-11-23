//! Source gates for admitted paid grounded controls.
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
    if let Some(gate) = super::super::cyclone::recovery(actor, input, now) {
        return Some(gate);
    }
    let crate::characters::RuntimeState::Hathor(s) = &actor.character else {
        return None;
    };
    let c = s.clock.as_ref()?;
    let Some(a) = c.active.as_ref() else {
        return (input == "jump").then_some((now, super::super::secondary::MODEL));
    };
    if matches!(a.kind, Kind::Branch | Kind::BranchFallback) {
        return (input == "switch_actor").then_some((now, super::super::branch::MODEL));
    }
    let progress = match input {
        "jump" => a.jump,
        "switch_actor" => a.switch,
        "GA_Hathor_Skill" if a.kind == Kind::Riposte => 0,
        "GA_Hathor_Skill" => {
            if let Kind::Dodge(back) = a.kind {
                if back { 331_208 } else { 267_501 }
            } else {
                a.recovery
            }
        }
        "dash" | "GA_Hathor_Melee" | "GA_Hathor_UltraSkill" => a.recovery,
        _ => return None,
    };
    c.lag
        .resolve_real_deadline(a.owner, a.start, progress)
        .ok()
        .map(|deadline| (deadline, super::super::secondary::MODEL))
}
pub(crate) fn dodging(actor: &ActorRuntime, now: u64) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Hathor(s)
        if s.clock.as_ref().and_then(|c|c.active.as_ref()).is_some_and(|a|matches!(a.kind,Kind::Dodge(_)))
        && now<actor.locked_until_us)
}
pub(crate) fn committed(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    if !super::super::secondary::accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    ctx.state.buffs.remove_owned_buff(
        super::super::cyclone::DEF,
        super::super::cyclone::SOURCE,
        owner,
    );
    stop(&mut ctx.state.actors[owner], ctx.queue, now)?;
    super::super::burst::finish_cyclone(ctx, owner, "ActionAdmission")?;
    if input == crate::simulation::PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        let recovery = if back { 1_009_564 } else { 996_277 };
        let a = &mut ctx.state.actors[owner];
        let clock = a
            .character
            .hathor_mut()?
            .clock
            .as_mut()
            .ok_or("Hathor Dodge clock missing")?;
        clock.active = Some(Action {
            owner,
            command,
            start: now,
            recovery,
            jump: if back { 0 } else { 82_544 },
            switch: recovery,
            gate: None,
            landing: None,
            groups: Default::default(),
            contacts: Default::default(),
            kind: Kind::Dodge(back),
            a4_counter: 0,
            positive: false,
            success: false,
            cooldown_started: false,
        });
        a.locked_until_us = clock
            .lag
            .resolve_real_deadline(owner, now, recovery)
            .map_err(|e| format!("Hathor Dodge recovery: {e}"))?;
    }
    Ok(())
}
