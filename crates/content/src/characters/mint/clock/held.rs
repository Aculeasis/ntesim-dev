//! Section-local hold thresholds, finite paid opportunities and one End lifecycle.
use super::{Clock, Event, Kind, Marker, due, profile::Hit, push};
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{GameActionCommand, GameScenarioRequest};

const OPEN: &[Hit] = &[
    (super::super::hold::EVADE1, 302511),
    (super::super::hold::EVADE1, 601565),
    (super::super::hold::EVADE1, 730626),
    (super::super::hold::EVADE4, 1196146),
];
pub(super) const SPINS: [u64; 30] = [
    1432981, 1567344, 1699972, 1831936, 1966199, 2099678, 2231232, 2366688, 2499738, 2632322,
    2767093, 2899117, 3033328, 3165678, 3299132, 3432802, 3567731, 3699915, 3833450, 3966964,
    4100661, 4232597, 4366074, 4500040, 4632957, 4767249, 4900703, 5032921, 5166700, 5299756,
];
fn threshold(kind: Kind) -> Option<u64> {
    match kind {
        Kind::Normal(stage) => Some([400000, 200000, 500000, 250000, 700000][stage]),
        Kind::Entry | Kind::Approach => Some(450000),
        Kind::Plunge => Some(530000),
        _ => None,
    }
}
pub(super) fn pending_begin(
    clock: &Clock,
    action: &super::Action,
) -> Result<Option<(u64, u64)>, String> {
    let Some(progress) = threshold(action.kind) else {
        return Ok(None);
    };
    let time = due(clock, action, progress)?;
    Ok(action
        .release
        .filter(|release| time < *release)
        .map(|_| (progress, time)))
}
pub(super) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    for (&command, hold) in &req.hold_inputs {
        let Some(GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            timestamp_us,
            ..
        }) = req.actions.get(command)
        else {
            continue;
        };
        if req.actors[*actor_index].character_id != "character:1019" {
            continue;
        }
        if !super::accepted(req)
            || ability_id != "GA_Mint019_Melee"
            || hold.moving
            || hold.reserved_stocks.is_some()
        {
            return Err(
                "Mint hold requires canonical stationary Normal without reserved stocks".into(),
            );
        }
        if hold.duration_us > 0 && req.combat_stamina.is_none() {
            return Err("Mint hold requires explicit combat stamina".into());
        }
        timestamp_us
            .checked_add(hold.duration_us)
            .and_then(|t| t.checked_add(616639))
            .ok_or("Mint held deadline overflow")?;
    }
    Ok(())
}
pub(super) fn schedule(ctx: &mut CastContext<'_>, clock: &Clock) -> Result<(), String> {
    let action = clock.active.as_ref().ok_or("Mint hold clock missing")?;
    let mut markers = Vec::new();
    if let Some((progress, _)) = pending_begin(clock, action)? {
        markers.push(Marker::Begin(progress));
    }
    if action.kind == Kind::Hold {
        markers.extend([Marker::Wind, Marker::NaturalEnd]);
        markers.extend((0..SPINS.len()).map(Marker::Spin));
        push(
            ctx.queue,
            ctx.sequence,
            action.owner,
            action.command,
            Marker::Release,
            action.release.ok_or("Mint hold release missing")?,
        )?;
    }
    for marker in markers {
        let event = Event {
            owner: action.owner,
            command: action.command,
            marker,
        };
        push(
            ctx.queue,
            ctx.sequence,
            event.owner,
            event.command,
            marker,
            super::marker_due(clock, action, &event)?.ok_or("Mint progress missing")?,
        )?;
    }
    Ok(())
}
fn end(ctx: &mut CallbackContext<'_>, event: &Event) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[event.owner];
    let old = actor
        .character
        .mint_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.clone())
        .ok_or("Mint ending action missing")?;
    super::stop(actor, ctx.queue, now, ctx.state.combat_stamina.as_mut())?;
    let mut clock = actor
        .character
        .mint_mut()?
        .clock
        .take()
        .ok_or("Mint ending clock missing")?;
    clock.active = Some(super::Action {
        kind: Kind::End,
        start: now,
        groups: Default::default(),
        contacts: Default::default(),
        release: None,
        ..old
    });
    let deadline = due(&clock, clock.active.as_ref().unwrap(), 616639)?;
    actor.locked_until_us = deadline;
    push(
        ctx.queue,
        ctx.sequence,
        event.owner,
        event.command,
        Marker::EndComplete,
        deadline,
    )?;
    actor.character.mint_mut()?.clock = Some(clock);
    Ok(())
}
pub(super) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[event.owner];
    if actor.hp == 0 {
        return Ok(());
    }
    let Some(action) = actor
        .character
        .mint_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.clone())
        .filter(|a| a.command == event.command)
    else {
        return Ok(());
    };
    match event.marker {
        Marker::Begin(_) => {
            let release = action.release.ok_or("Mint held prefix release missing")?;
            if release <= now {
                return Ok(());
            }
            action
                .source
                .as_ref()
                .ok_or("Mint held prefix source missing")?
                .with_cast(
                    ctx,
                    event.owner,
                    ctx.request.aim_target(event.command),
                    |cast| {
                        cast.record_hypothesis(super::super::hold::HYPOTHESIS_ID)?;
                        super::stop(cast.actor, cast.queue, now, cast.stamina.as_deref_mut())?;
                        let s = cast.actor.character.mint_mut()?;
                        s.normal_stage = 0;
                        s.normal_started_us = None;
                        super::start(cast, Kind::Hold, OPEN, Some(release), true, 0)
                    },
                )
        }
        Marker::Wind => ctx
            .state
            .combat_stamina
            .as_mut()
            .ok_or("Mint Wind requires stamina")?
            .suppress_recovery(event.owner, event.command, now, true),
        Marker::Spin(index) => {
            let stamina = ctx
                .state
                .combat_stamina
                .as_mut()
                .ok_or("Mint spin requires stamina")?;
            crate::runtime_api::record_hypothesis_id(
                ctx.request,
                &mut ctx.state.execution_model_assumptions,
                crate::HypothesisId::CombatStaminaV1,
            )?;
            if !stamina.try_spend(event.command, event.owner, now, 2)? {
                return end(ctx, &event);
            }
            action
                .source
                .as_ref()
                .ok_or("Mint paid spin source missing")?
                .with_cast(
                    ctx,
                    event.owner,
                    ctx.request.aim_target(event.command),
                    |cast| {
                        let group =
                            cast.schedule_native_aoe_group(super::super::hold::EVADE5, now)?;
                        cast.actor
                            .character
                            .mint_mut()?
                            .clock
                            .as_mut()
                            .ok_or("Mint spin clock missing")?
                            .active
                            .as_mut()
                            .ok_or("Mint spin action missing")?
                            .groups
                            .insert(group, SPINS[index]);
                        Ok(())
                    },
                )
        }
        Marker::NaturalEnd | Marker::Release => end(ctx, &event),
        Marker::Land => {
            actor.character.mint_mut()?.airborne = false;
            Ok(())
        }
        Marker::EndComplete => {
            super::stop(actor, ctx.queue, now, ctx.state.combat_stamina.as_mut())
        }
        Marker::Cooldown => Err("Mint cooldown routed to hold state".into()),
        Marker::TenacityBegin | Marker::TenacityEnd => {
            Err("Mint tenacity routed to hold state".into())
        }
    }
}
