//! Committed native Dodge/Jump and explicit successful parry.
use super::*;
use crate::simulation::PlayerInput;

pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: PlayerInput,
) -> Result<(), String> {
    if !super::accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    super::stop(actor, ctx.queue, now, ctx.state.combat_stamina.as_mut())?;
    let state = actor.character.mint_mut()?;
    state.riposte_ready = false;
    state.entry_normal_pending = false;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.airborne = input == PlayerInput::Jump;
    if input == PlayerInput::Jump {
        state
            .clock
            .get_or_insert_with(Default::default)
            .input_command = Some(command);
        actor.locked_until_us = now;
        return Ok(());
    }
    if input != PlayerInput::Dash {
        return Err("Unsupported Mint native control".into());
    }
    let back = ctx.request.dash_directions.get(&command)
        == Some(&crate::simulation::GameDashDirection::Back);
    dodge(ctx, owner, command, now, back)
}
fn dodge(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    start: u64,
    back: bool,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor.character.mint_mut()?.clock.take().unwrap_or_default();
    super::inherit(&mut clock, now)?;
    clock.active = Some(Action {
        owner,
        command,
        start,
        kind: Kind::Dodge(back),
        groups: Default::default(),
        contacts: Default::default(),
        cooldown_started: false,
        source: None,
        release: None,
    });
    super::project(&clock, actor, ctx.queue, now, true)?;
    actor.character.mint_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn validate_parry(req: &GameScenarioRequest) -> Result<(), String> {
    if !super::accepted(req) || req.combat_stamina.is_none() {
        return Err("Mint parry requires ordinary_clock_v1 and paid Dodge stamina".into());
    }
    Ok(())
}
pub(crate) fn parry(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorParryReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 || ctx.state.active_actor_index != owner || now >= actor.locked_until_us {
        return Err("Mint parry requires an active living owner during paid Dodge".into());
    }
    let action = actor
        .character
        .mint_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .filter(|a| matches!(a.kind, Kind::Dodge(_)))
        .ok_or("Mint parry requires paid Dodge")?;
    let command = action.command;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    actor.character.mint_mut()?.riposte_ready = false;
    super::stop(actor, ctx.queue, now, ctx.state.combat_stamina.as_mut())?;
    dodge(
        ctx,
        owner,
        command,
        now.checked_add(20000).ok_or("Mint delayed Back overflow")?,
        true,
    )?;
    Ok(crate::simulation::GameActorParryReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        malice_granted: crate::simulation::ExactFraction::new(0, 1)?,
        skill_permission_until_us: None,
    })
}
