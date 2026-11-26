//! `On the Line` / `Remembrance` off-field coordinated attacks (`3.0s` x `6` triggers).
use super::parameters::PARAMETERS;
use super::skill::Connection;
use crate::callbacks::ContentEvent;
use crate::execution::CausalPriority;
use crate::runtime_api::{CallbackContext, CastContext, OwnedAction};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ElymsUltraAndOnTheLineV1;
pub(crate) const XIETONG_A: &str = "GE_Player_Elyms_XieTongSkillA1_Damage";
pub(crate) const XIETONG_B: &str = "GE_Player_Elyms_XieTongSkillB1_Damage";

#[derive(Clone, Debug)]
pub(crate) struct OnTheLineState {
    pub(crate) source: OwnedAction,
    pub(crate) is_st: bool,
    pub(crate) snapshot_connection: Connection,
    pub(crate) remaining_triggers: u8,
    pub(crate) generation: u64,
    pub(crate) next_trigger_rel_us: Option<u64>,
    pub(crate) paused_remaining_rel_us: Option<u64>,
}

pub(crate) fn arm(
    ctx: &mut CastContext<'_>,
    is_st: bool,
    snapshot_connection: Connection,
) -> Result<(), String> {
    let source = OwnedAction::capture(ctx)?;
    let owner = ctx.actor_index;
    let queue = &mut *ctx.queue;
    let state = ctx.actor.character.elyms_mut()?;
    let next_gen = state
        .on_the_line
        .as_ref()
        .map_or(1, |s| s.generation.saturating_add(1));
    queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::OnTheLineTick { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    state.on_the_line = Some(OnTheLineState {
        source,
        is_st,
        snapshot_connection,
        remaining_triggers: PARAMETERS.on_the_line_max_triggers,
        generation: next_gen,
        next_trigger_rel_us: None,
        paused_remaining_rel_us: Some(0),
    });
    state
        .burst_snap_connection
        .insert(ctx.command_index, snapshot_connection);
    Ok(())
}

fn schedule_tick(
    queue: &mut crate::simulation::timer_queue::EventQueue,
    sequence: &mut u64,
    owner: usize,
    generation: u64,
    fire_at_abs_us: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: fire_at_abs_us,
        priority: CausalPriority::ProcCallback,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::Elyms(super::events::Event::OnTheLineTick {
            owner,
            generation,
        })),
    });
    Ok(())
}

pub(crate) fn on_field_exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let rel_now = ctx.state.mode_clock.relative_us();
    let abs_now = ctx.state.current_time_us;
    let schedule_info = {
        let state = ctx.state.actors[owner].character.elyms_mut()?;
        let Some(otl) = &mut state.on_the_line else {
            return Ok(());
        };
        if otl.remaining_triggers == 0 {
            state.on_the_line = None;
            return Ok(());
        }
        let delay = otl.paused_remaining_rel_us.take().unwrap_or(0);
        otl.generation = otl.generation.saturating_add(1);
        otl.next_trigger_rel_us = Some(
            rel_now
                .checked_add(delay)
                .ok_or("Elyms On the Line deadline overflow")?,
        );
        (otl.generation, delay)
    };
    let (generation, delay) = schedule_info;
    ctx.queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::OnTheLineTick { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    let fire_at = abs_now
        .checked_add(delay)
        .ok_or("Elyms On the Line timer overflow")?;
    schedule_tick(ctx.queue, ctx.sequence, owner, generation, fire_at)
}

pub(crate) fn on_field_entry(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let rel_now = ctx.state.mode_clock.relative_us();
    let state = ctx.state.actors[owner].character.elyms_mut()?;
    let Some(otl) = &mut state.on_the_line else {
        return Ok(());
    };
    if otl.remaining_triggers == 0 {
        state.on_the_line = None;
        return Ok(());
    }
    let rem = otl
        .next_trigger_rel_us
        .take()
        .map_or(0, |due| due.saturating_sub(rel_now));
    otl.paused_remaining_rel_us = Some(rem);
    otl.generation = otl.generation.saturating_add(1);
    ctx.queue.retain(|event| {
        !matches!(
            &event.data,
            GameEventData::Content(ContentEvent::Elyms(
                super::events::Event::OnTheLineTick { owner: ev_owner, .. }
            )) if *ev_owner == owner
        )
    });
    Ok(())
}

pub(crate) fn on_tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 || ctx.state.active_actor_index == owner {
        return Ok(());
    }
    let rel_now = ctx.state.mode_clock.relative_us();
    let abs_now = ctx.state.current_time_us;
    let next = {
        let state = ctx.state.actors[owner].character.elyms_mut()?;
        let Some(otl) = &mut state.on_the_line else {
            return Ok(());
        };
        if otl.generation != generation || otl.remaining_triggers == 0 {
            return Ok(());
        }
        otl.remaining_triggers -= 1;
        let source = otl.source.clone();
        let is_st = otl.is_st;
        let snap = otl.snapshot_connection;
        let remaining = otl.remaining_triggers;
        let next_gen = otl.generation;
        if remaining > 0 {
            otl.next_trigger_rel_us = Some(
                rel_now
                    .checked_add(PARAMETERS.on_the_line_period_us)
                    .ok_or("Elyms On the Line next trigger overflow")?,
            );
        } else {
            state.on_the_line = None;
        }
        (source, is_st, snap, remaining > 0, next_gen)
    };
    let (source, is_st, _snap, reschedule, next_gen) = next;
    if reschedule {
        let next_abs = abs_now
            .checked_add(PARAMETERS.on_the_line_period_us)
            .ok_or("Elyms On the Line reschedule overflow")?;
        schedule_tick(ctx.queue, ctx.sequence, owner, next_gen, next_abs)?;
    }

    let first_living =
        (0..ctx.request.targets.len()).find(|&t| ctx.state.targets[t].hp_current > 0);
    let Some(primary_target) = first_living else {
        return Ok(());
    };
    let (packet, first_delay) = if is_st {
        (XIETONG_B, super::clock::profile::XIETONGB_FIRST_HIT)
    } else {
        (XIETONG_A, super::clock::profile::XIETONGA_FIRST_HIT)
    };
    source.with_cast(ctx, owner, primary_target, |cast| {
        cast.record_hypothesis(MODEL)?;
        if !is_st && cast.targets.len() > 1 {
            cast.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
        }
        for hit_idx in 0..u64::from(PARAMETERS.on_the_line_hit_count) {
            let offset = first_delay
                .checked_add(
                    hit_idx
                        .checked_mul(PARAMETERS.on_the_line_hit_step_us)
                        .ok_or("Elyms Remembrance hit step overflow")?,
                )
                .ok_or("Elyms Remembrance hit offset overflow")?;
            let time_us = cast
                .time_us
                .checked_add(offset)
                .ok_or("Elyms Remembrance hit time overflow")?;
            if is_st {
                cast.schedule_native_target_group(packet, primary_target, time_us)?;
            } else {
                cast.schedule_native_aoe_group(packet, time_us)?;
            }
        }
        Ok(())
    })
}
