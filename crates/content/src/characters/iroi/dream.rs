use super::parameters::{HealKind, heal_ratio};
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    ExactFraction,
    events::{GameEventData, ScheduledGameEvent},
};
use ntesim_core::execution::CausalPriority;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::IroiDreamStateV1;
pub(crate) const PULSE_PACKET: &str = "GE_Player_Oneiroi_UltraSkill1_Damage";

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn defeat(actor: &mut crate::simulation::state::ActorRuntime) {
    if let crate::characters::RuntimeState::Iroi(state) = &mut actor.character
        && state.dream_active
    {
        state.dream_active = false;
        state.moon = ExactFraction::default();
        state.revive_used_in_dream = false;
        state.revived_ally = None;
        state.native_dream = None;
    }
}

pub(crate) fn start_dream(ctx: &mut CastContext<'_>, coordinate: u32) -> Result<bool, String> {
    let state = ctx.actor.character.iroi_mut()?;
    // Reserve the current balance, including a continuing Dream's actual remainder.
    let moon = super::resource::remaining(state, ctx.time_us)?;
    let duration_us = super::resource::duration_us(moon)?;
    let release_us = ctx
        .time_us
        .checked_add(4_004_747)
        .ok_or("Iroi Dream release overflow")?;
    let end_us = release_us
        .checked_add(duration_us)
        .ok_or("Iroi Dream deadline overflow")?;
    let generation_id = state
        .dream_generation
        .checked_add(1)
        .ok_or("Iroi Dream generation overflow")?;
    state.moon = moon;
    let is_ivory = state.moon.numerator >= 30 * state.moon.denominator;
    state.dream_active = true;
    state.dream_is_ivory = is_ivory;
    state.revive_used_in_dream = false;
    state.revived_ally = None;
    state.dream_generation = generation_id;
    state.dream_release_us = release_us;
    state.dream_end_us = end_us;

    ctx.record_hypothesis(HYPOTHESIS_ID)?;

    let queue = &mut *ctx.queue;
    let seq = &mut *ctx.sequence;

    // Schedule Dream end at end_us
    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us: end_us,
        priority: CausalPriority::BuffTickOrExpiry,
        seq: *seq,
        data: GameEventData::Content(ContentEvent::IroiDreamEnd {
            actor_index: ctx.actor_index,
            generation: generation_id,
        }),
    });

    // Schedule first periodic tick at release_us + 1_000_000 if release_us + 1_000_000 < end_us
    if let Some(first_tick_us) = release_us
        .checked_add(1_000_000)
        .filter(|time| *time < end_us)
    {
        crate::runtime_api::increment_sequence(seq)?;
        queue.push(ScheduledGameEvent {
            time_us: first_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *seq,
            data: GameEventData::Content(ContentEvent::IroiDreamTick {
                actor_index: ctx.actor_index,
                generation: generation_id,
                tick_index: 1,
                coordinate,
            }),
        });
    }

    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us: release_us,
        priority: CausalPriority::ProcCallback,
        seq: *seq,
        data: GameEventData::Content(ContentEvent::IroiDreamRelease {
            actor_index: ctx.actor_index,
            command_index: ctx.command_index,
            generation: generation_id,
        }),
    });
    Ok(is_ivory)
}

pub(crate) fn release(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let state = actor.character.iroi_mut()?;
    if actor.hp == 0 || !state.dream_active || state.dream_generation != generation {
        return Ok(());
    }
    state.dream_release_us = now;
    state.dream_end_us = now
        .checked_add(super::resource::duration_us(state.moon)?)
        .ok_or("Iroi deployed Dream lifetime overflow")?;
    let end = state.dream_end_us;
    let events = ctx.queue.drain().collect::<Vec<_>>();
    for mut event in events {
        match &event.data {
            GameEventData::Content(ContentEvent::IroiDreamEnd {
                actor_index,
                generation: live,
            }) if *actor_index == owner && *live == generation => {
                event.time_us = end;
                ctx.queue.push_on(
                    ntesim_core::kernel::timing::ClockDomain::ObjectLifetime,
                    event,
                );
            }
            GameEventData::Content(ContentEvent::IroiDreamTick {
                actor_index,
                generation: live,
                tick_index,
                ..
            }) if *actor_index == owner && *live == generation => {
                event.time_us = now
                    .checked_add(
                        tick_index
                            .checked_mul(1_000_000)
                            .ok_or("Iroi tick phase overflow")?,
                    )
                    .ok_or("Iroi tick deadline overflow")?;
                ctx.queue
                    .push_on(ntesim_core::kernel::timing::ClockDomain::World, event);
            }
            _ => ctx.queue.push(event),
        }
    }
    Ok(())
}

pub(crate) fn on_tick(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
    tick_index: u64,
    coordinate: u32,
) -> Result<(), String> {
    let (is_ivory, release_us, end_us) = {
        let source = &ctx.state.actors[actor_index];
        if source.hp == 0 {
            return Ok(());
        }
        let state = source.character.iroi()?;
        if !state.dream_active
            || state.dream_generation != generation
            || ctx.state.current_time_us >= state.dream_end_us
        {
            return Ok(());
        }
        (
            state.dream_is_ivory,
            state.dream_release_us,
            state.dream_end_us,
        )
    };

    let native = ctx.state.actors[actor_index]
        .character
        .iroi()?
        .native_dream
        .clone();
    if native.as_ref().is_none_or(|dream| dream.healing) {
        let atk_ratio = heal_ratio(HealKind::BurstPeriodic, coordinate)?;
        let mut recipients = (0..ctx.request.actors.len()).collect::<Vec<_>>();
        recipients.sort_by_key(|&index| (ctx.request.actors[index].slot, index));
        for recipient_actor_index in recipients {
            super::callback::apply_team_heal(
                ctx,
                actor_index,
                recipient_actor_index,
                atk_ratio,
                "iroi_dream_periodic_heal",
                "Burst",
            )?;
        }
    }

    // If Ivory: deliver pulse damage GE_Player_Oneiroi_UltraSkill1_Damage
    if is_ivory && native.as_ref().is_none_or(|dream| dream.damage) {
        if let Some(dream) = &native {
            dream.source.with_cast(ctx, actor_index, 0, |cast| {
                cast.schedule_native_autonomous_capped_aoe_packet(PULSE_PACKET, cast.time_us, 20)
            })?;
        } else {
            deliver_pulse_damage(ctx, actor_index)?;
        }
    }

    // Schedule next tick if next_tick_us < end_us
    let next_tick_us = release_us + (tick_index + 1) * 1_000_000;
    if next_tick_us < end_us {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: next_tick_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *ctx.sequence,
            data: GameEventData::Content(ContentEvent::IroiDreamTick {
                actor_index,
                generation,
                tick_index: tick_index + 1,
                coordinate,
            }),
        });
    }

    Ok(())
}

fn deliver_pulse_damage(ctx: &mut CallbackContext<'_>, actor_index: usize) -> Result<(), String> {
    let metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &ctx.request.actors[actor_index].character_id,
        "GA_Oneiroi_UltraSkill",
        PULSE_PACKET,
        None,
        &ctx.state.execution_model_assumptions,
    )?;

    let target_count = ctx.request.targets.len().min(20);
    for target_index in 0..target_count {
        if ctx.state.targets[target_index].hp_current == 0 {
            continue;
        }
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.state.current_time_us,
            priority: CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index: 0,
                actor_index,
                target_index,
                ability_id: "GA_Oneiroi_UltraSkill".into(),
                damage_effect_id: PULSE_PACKET.into(),
                atk_rate: ntesim_core::formula::Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                metadata: Box::new(metadata.clone()),
            },
        });
    }
    Ok(())
}

pub(crate) fn on_dream_end(
    ctx: &mut CallbackContext<'_>,
    actor_index: usize,
    generation: u64,
) -> Result<(), String> {
    let revived_ally = {
        let state = ctx.state.actors[actor_index].character.iroi_mut()?;
        if state.dream_active && state.dream_generation == generation {
            state.dream_active = false;
            state.moon = ExactFraction::default();
            state.native_dream = None;
            ctx.queue.retain(|event| {
                !matches!(&event.data, GameEventData::Content(
                ContentEvent::IroiDreamTick { actor_index: owner, generation: live, .. })
                if *owner == actor_index && *live == generation)
            });
            state.revived_ally.take()
        } else {
            None
        }
    };
    if let Some(ally_idx) = revived_ally {
        super::regression::re_regress_ally(ctx, actor_index, ally_idx)?;
    }
    Ok(())
}
