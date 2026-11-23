//! Actual Skill.4 control state consumed by teammate entry selectors.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

pub(crate) const HYPOTHESIS_ID: crate::HypothesisId = crate::HypothesisId::SkiaGnawEntryStateV1;

pub(crate) fn public_skill_timing() -> (u64, u64, u64) {
    (1_335_873, 12_153_592, 153_592)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    Begin,
    Attack,
    End,
}

pub(crate) fn accepted(request: &crate::simulation::GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(HYPOTHESIS_ID.as_str())
        .map(String::as_str)
        == Some(HYPOTHESIS_ID.description())
}

pub(crate) fn is_sneaking(actor: &crate::simulation::state::ActorRuntime, now: u64) -> bool {
    actor.hp > 0
        && matches!(&actor.character, crate::characters::RuntimeState::Skia(state) if state.gnaw_until_us > now)
}

pub(crate) fn leave(ctx: &mut CallbackContext<'_>, actor: usize) {
    if let crate::characters::RuntimeState::Skia(state) = &mut ctx.state.actors[actor].character {
        state.gnaw_until_us = 0;
    }
    ctx.queue.retain(|event| !matches!(&event.data, GameEventData::Content(
        crate::callbacks::ContentEvent::SkiaEntryState { actor_index, .. }) if *actor_index == actor));
}

pub(crate) fn before_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if is_sneaking(ctx.actor, ctx.time_us)
        && matches!(ctx.ability_id, "GA_Skia_Melee" | "GA_Skia_QTE")
    {
        let state = ctx.actor.character.skia_mut()?;
        state.gnaw_until_us = 0;
        let actor = ctx.actor_index;
        let mut events = ctx.queue.drain().collect::<Vec<_>>();
        events.retain(|event| {
            !matches!(&event.data, GameEventData::Content(
                crate::callbacks::ContentEvent::SkiaEntryState { actor_index, .. }) if *actor_index == actor)
        });
        ctx.queue.replace(events);
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.ability_id != "GA_Skia_Skill" {
        return Ok(());
    }
    ctx.record_hypothesis(HYPOTHESIS_ID)?;
    let (recovery_us, cooldown_us, primary_us) = public_skill_timing();
    let mut events = ctx.queue.drain().collect::<Vec<_>>();
    let mut selected: Vec<_> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| match &event.data {
            GameEventData::HitDelivery {
                command_index,
                actor_index,
                damage_effect_id,
                ..
            } if *command_index == ctx.command_index
                && *actor_index == ctx.actor_index
                && matches!(
                    damage_effect_id.as_str(),
                    "GE_Player_Skia_Skill1_Damage"
                        | "GE_Player_Skia_Skill2_Damage"
                        | "GE_Player_Skia_Skill3_Damage"
                ) =>
            {
                Some((event.seq, index))
            }
            _ => None,
        })
        .collect();
    selected.sort_unstable();
    if selected.len() != 6 {
        return Err("Skia entry state requires the six native Skill packets".into());
    }
    for ((_, index), offset) in selected
        .into_iter()
        .zip([primary_us, 216_117, 296_382, 369_313, 871_421, 913_168])
    {
        events[index].time_us = ctx
            .time_us
            .checked_add(offset)
            .ok_or("Skia Skill impact overflow")?;
    }
    ctx.queue.replace(events);
    ctx.actor.locked_until_us = ctx
        .time_us
        .checked_add(recovery_us)
        .ok_or("Skia recovery overflow")?;
    ctx.actor.ability_cooldowns.insert(
        "GA_Skia_Skill".into(),
        ctx.time_us
            .checked_add(cooldown_us)
            .ok_or("Skia cooldown overflow")?,
    );
    let state = ctx.actor.character.skia_mut()?;
    state.gnaw_generation = state
        .gnaw_generation
        .checked_add(1)
        .ok_or("Skia generation overflow")?;
    state.gnaw_until_us = 0;
    state.gnaw_marked = false;
    state.gnaw_command = ctx.command_index;
    schedule(
        ctx.queue,
        ctx.sequence,
        ctx.actor_index,
        state.gnaw_generation,
        ctx.time_us
            .checked_add(1_048_984)
            .ok_or("Skia Gnaw marker overflow")?,
        Boundary::Begin,
    )
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if accepted(ctx.request)
        && ctx.damage_effect_id == "GE_Player_Skia_Skill3_Damage"
        && ctx.target_alive
    {
        ctx.actor.character.skia_mut()?.gnaw_marked = true;
    }
    Ok(())
}

pub(crate) fn event(
    ctx: &mut CallbackContext<'_>,
    actor: usize,
    generation: u64,
    boundary: Boundary,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let runtime = &mut ctx.state.actors[actor];
    let state = runtime.character.skia_mut()?;
    let command = state.gnaw_command;
    let target_index = ctx.request.aim_target(command);
    let marked_target_alive = ctx.state.targets[target_index].hp_current > 0;
    if runtime.hp == 0
        || generation != state.gnaw_generation
        || ctx.state.active_actor_index != actor
    {
        return Ok(());
    }
    match boundary {
        Boundary::Begin => {
            state.gnaw_until_us = now
                .checked_add(5_000_000)
                .ok_or("Skia Gnaw duration overflow")?;
            schedule(
                ctx.queue,
                ctx.sequence,
                actor,
                generation,
                now.checked_add(350_000)
                    .ok_or("Skia Gnaw attack overflow")?,
                Boundary::Attack,
            )?;
            schedule(
                ctx.queue,
                ctx.sequence,
                actor,
                generation,
                state.gnaw_until_us,
                Boundary::End,
            )?;
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: now,
                phase: "BuffTickOrExpiry".into(),
                event_kind: "skia_gnaw_enter".into(),
                active_actor: actor,
                hit_index: None,
                from_actor: Some(actor),
                to_actor: Some(actor),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    expires_at_us: Some(state.gnaw_until_us),
                    ..Default::default()
                }),
            });
            Ok(())
        }
        Boundary::Attack
            if state.gnaw_until_us > now && state.gnaw_marked && marked_target_alive =>
        {
            state.gnaw_marked = false;
            let config = &ctx.request.actors[actor];
            let has_mark = crate::simulation::state::target_has_status(
                &ctx.state.buffs,
                actor,
                &config.character_id,
                target_index,
                "skia.fang_thrust",
            );
            if has_mark {
                ctx.state.buffs.remove_target_buff(
                    "skia.fang_thrust",
                    &config.character_id,
                    &target_index.to_string(),
                );
                super::skill::schedule_shadow_atk(
                    ctx.queue,
                    ctx.sequence,
                    ctx.request,
                    actor,
                    target_index,
                    now,
                    command,
                )?;
            }
            Ok(())
        }
        Boundary::End if state.gnaw_until_us == now => {
            state.gnaw_until_us = 0;
            state.gnaw_marked = false;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn schedule(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    seq: &mut u64,
    actor_index: usize,
    generation: u64,
    time_us: u64,
    boundary: Boundary,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(seq)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
        seq: *seq,
        data: GameEventData::Content(crate::callbacks::ContentEvent::SkiaEntryState {
            actor_index,
            generation,
            boundary,
        }),
    });
    Ok(())
}
