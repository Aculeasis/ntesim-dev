//! Root reservation, cancellable carrier installation and actual Dream release.
use super::*;
use crate::callbacks::ContentEvent;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
pub(super) const ABILITY: &str = "GA_Oneiroi_UltraSkill";
pub(super) const RELEASE: u64 = 4_004_747;
#[derive(Clone, Debug)]
pub(crate) struct Dream {
    pub(crate) source: OwnedAction,
    pub(crate) healing: bool,
    pub(crate) damage: bool,
    pending: bool,
}
pub(super) fn begin(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(super::super::skill::BURST_HYPOTHESIS_ID)?;
    let coordinate = super::super::skill::heal_coordinate(ctx, "Burst")?;
    super::super::dream::start_dream(ctx, coordinate)?;
    ctx.actor.character.iroi_mut()?.native_dream = Some(Box::new(Dream {
        source: OwnedAction::capture(ctx)?,
        healing: false,
        damage: false,
        pending: true,
    }));
    ctx.actor.ability_cooldowns.insert(
        ABILITY.into(),
        ctx.time_us
            .checked_add(16_000_000)
            .ok_or("Iroi Burst cooldown overflow")?,
    );
    super::super::skill::burst_entry(ctx)?;
    for (progress, notify) in [
        (1_925_956, Notify::BurstHeal),
        (3_385_389, Notify::BurstHit),
        (3_882_835, Notify::BurstCarrier),
        (RELEASE, Notify::BurstRelease),
    ] {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.time_us,
            Event::Montage {
                owner: ctx.actor_index,
                command: ctx.command_index,
                progress,
                notify,
            },
        )?;
    }
    Ok(())
}
/// Reproject the reserved owner clock; other clocks retain their declared basis.
pub(super) fn project(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    owner: usize,
    release: u64,
) -> Result<(), String> {
    let state = actor.character.iroi_mut()?;
    if !state.native_dream.as_ref().is_some_and(|d| d.pending) {
        return Ok(());
    }
    let end = release
        .checked_add(super::super::resource::duration_us(state.moon)?)
        .ok_or("Iroi native Dream end overflow")?;
    state.dream_release_us = release;
    state.dream_end_us = end;
    let generation = state.dream_generation;
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        match &event.data {
            GameEventData::Content(ContentEvent::IroiDreamRelease {
                actor_index,
                generation: g,
                ..
            }) if *g == generation && *actor_index == owner => event.time_us = release,
            GameEventData::Content(ContentEvent::IroiDreamEnd {
                actor_index,
                generation: g,
            }) if *g == generation && *actor_index == owner => event.time_us = end,
            GameEventData::Content(ContentEvent::IroiDreamTick {
                actor_index,
                generation: g,
                tick_index,
                ..
            }) if *g == generation && *actor_index == owner => {
                event.time_us = tick_index
                    .checked_mul(1_000_000)
                    .and_then(|offset| release.checked_add(offset))
                    .ok_or("Iroi native Dream tick overflow")?;
            }
            _ => {}
        }
    }
    queue.replace(events);
    Ok(())
}
pub(super) fn cancel(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    owner: usize,
    now: u64,
) -> Result<(), String> {
    let state = actor.character.iroi_mut()?;
    let Some(dream) = &state.native_dream else {
        return Ok(());
    };
    if !dream.pending {
        return Ok(());
    }
    if dream.healing || dream.damage {
        // Ending setup releases only already installed carriers with full lifetime.
        project(actor, queue, owner, now)?;
        actor
            .character
            .iroi_mut()?
            .native_dream
            .as_mut()
            .ok_or("Iroi native Dream lost reservation")?
            .pending = false;
    } else {
        state.dream_active = false;
        state.native_dream = None;
    }
    Ok(())
}
pub(super) fn notify(
    ctx: &mut CallbackContext<'_>,
    action: &Action,
    notify: Notify,
) -> Result<(), String> {
    match notify {
        Notify::BurstHeal => {
            ctx.state.actors[action.owner]
                .character
                .iroi_mut()?
                .native_dream
                .as_mut()
                .ok_or("Iroi native heal carrier lost Dream")?
                .healing = true;
            action
                .source
                .with_cast(ctx, action.owner, 0, super::super::skill::native_burst_heal)
        }
        Notify::BurstHit => {
            if ctx.state.actors[action.owner]
                .character
                .iroi()?
                .dream_is_ivory
            {
                let target = ctx.request.aim_target(action.command);
                let target = if target < ctx.state.targets.len()
                    && ctx.state.targets[target].hp_current > 0
                {
                    target
                } else {
                    ctx.state
                        .targets
                        .iter()
                        .position(|t| t.hp_current > 0)
                        .unwrap_or(0)
                };
                action.source.with_cast(ctx, action.owner, target, |cast| {
                    cast.schedule_native_capped_aoe_packet(
                        "GE_Player_Oneiroi_UltraSkill_Damage",
                        cast.time_us,
                        20,
                    )
                })?;
            }
            Ok(())
        }
        Notify::BurstCarrier => {
            ctx.state.actors[action.owner]
                .character
                .iroi_mut()?
                .native_dream
                .as_mut()
                .ok_or("Iroi native pulse carrier lost Dream")?
                .damage = true;
            Ok(())
        }
        Notify::BurstRelease => {
            if let Some(dream) = ctx.state.actors[action.owner]
                .character
                .iroi_mut()?
                .native_dream
                .as_mut()
            {
                dream.pending = false;
            }
            Ok(())
        }
        _ => Err("Iroi native Burst received another section's notify".into()),
    }
}
