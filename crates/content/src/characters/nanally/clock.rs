//! Finite owner progress and causal cancellation on the shared real axis.
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{events::GameEventData, state::ActorRuntime};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) mod held;
mod notifications;
mod profile;
pub(crate) use notifications::{Event, apply as notify};
pub(crate) use profile::Kind;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::NanallyActionClockV1;

pub(crate) fn skill_timing() -> (u64, u64, u64) {
    (
        profile::windows(Kind::Skill).recovery,
        super::removal::SKILL_COOLDOWN_US,
        profile::hits(Kind::Skill)[0].1,
    )
}

pub(crate) fn burst_timing() -> (u64, u64, u64) {
    (
        profile::windows(Kind::Burst).recovery,
        super::underboss::BURST_COOLDOWN_US,
        profile::hits(Kind::Burst)[0].1,
    )
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    action: Option<Box<Action>>,
    input_command: Option<usize>,
}
#[derive(Clone, Debug)]
struct Action {
    kind: Kind,
    owner: usize,
    command: usize,
    start: u64,
    recovery: u64,
    jump: u64,
    movement: u64,
    switch: u64,
    skill: u64,
    owned_action: Option<crate::runtime_api::OwnedAction>,
    held_until: Option<u64>,
    groups: BTreeMap<String, u64>,
    contacts: BTreeSet<String>,
}

fn deadline(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|error| format!("Nanally actor clock: {error}"))
}

fn project(
    clock: &Clock,
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    fresh: bool,
) -> Result<(), String> {
    let Some(action) = &clock.action else {
        return Ok(());
    };
    let times = action
        .groups
        .iter()
        .map(|(group, progress)| Ok((group.clone(), deadline(clock, action, *progress)?)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if fresh || event.time_us > now {
            match &event.data {
                GameEventData::HitDelivery { metadata, .. } => {
                    if let Some(time) = metadata
                        .binding
                        .energy_group_id
                        .as_ref()
                        .and_then(|group| times.get(group))
                    {
                        event.time_us = *time;
                    }
                }
                GameEventData::Content(crate::callbacks::ContentEvent::NanallyClock(marker))
                    if marker.owner == action.owner && marker.command == action.command =>
                {
                    event.time_us = deadline(clock, action, marker.progress())?;
                }
                _ => {}
            }
        }
    }
    actor.locked_until_us =
        deadline(clock, action, action.recovery)?.max(action.held_until.unwrap_or(0));
    queue.replace(events);
    Ok(())
}

pub(crate) fn end(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
) -> Result<Option<usize>, String> {
    let clock = &mut actor.character.nanally_mut()?.clock;
    let input = clock.input_command.take();
    let Some(action) = clock.action.take() else {
        return Ok(input);
    };
    queue.retain(|event| {
        !matches!(&event.data, GameEventData::HitDelivery {metadata,..} if metadata.binding.energy_group_id.as_ref().is_some_and(|group|action.groups.contains_key(group)))
        && !matches!(&event.data, GameEventData::Content(crate::callbacks::ContentEvent::NanallyUnderboss(group)) if group.belongs_to_command(action.owner,action.command))
        && !matches!(&event.data, GameEventData::Content(crate::callbacks::ContentEvent::NanallyClock(marker)) if marker.owner==action.owner && marker.command==action.command)
    });
    Ok(Some(action.command))
}

pub(crate) fn cast(ctx: &mut CastContext<'_>, kind: Kind) -> Result<(), String> {
    cast_after(ctx, kind, 0)
}

pub(crate) fn cast_after(
    ctx: &mut CastContext<'_>,
    kind: Kind,
    start_delay_us: u64,
) -> Result<(), String> {
    let release = held::release(ctx, kind)?;
    cast_phase(ctx, kind, start_delay_us, false, release)
}

fn continue_held(ctx: &mut CastContext<'_>, kind: Kind, release: u64) -> Result<(), String> {
    cast_phase(ctx, kind, 0, true, Some(release))
}

fn cast_phase(
    ctx: &mut CastContext<'_>,
    kind: Kind,
    start_delay_us: u64,
    continuation: bool,
    held_until: Option<u64>,
) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    end(ctx.actor, ctx.queue)?;
    let hits = profile::hits(kind);
    if !continuation
        && ctx
            .packet_metadata
            .map(|packet| packet.damage_effect_id.as_str())
            != Some(hits[0].0)
    {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let mut groups = BTreeMap::new();
    // Aim captures one recipient. Each ordinary melee packet hits all living in-range dummies.
    let primary = if continuation {
        ctx.schedule_native_aoe_group(hits[0].0, ctx.time_us)?
    } else if !matches!(kind, Kind::Aim) {
        ctx.scope_primary_aoe_group(hits[0].0)?
    } else {
        ctx.scope_primary_target_group(hits[0].0)?
    };
    groups.insert(primary, hits[0].1);
    for &(effect, progress) in &hits[1..] {
        groups.insert(
            ctx.schedule_native_aoe_group(effect, ctx.time_us)?,
            progress,
        );
    }
    let windows = profile::windows(kind);
    let owned_action = (held_until.is_some() || matches!(kind, Kind::Skill | Kind::Burst))
        .then(|| crate::runtime_api::OwnedAction::capture(ctx))
        .transpose()?;
    let mut clock = std::mem::take(&mut ctx.actor.character.nanally_mut()?.clock);
    // Only an unexpired contact interval can affect progress from this new input.
    let mut carried = HitlagTracker::default();
    for window in clock.lag.windows() {
        if window.start_real_us.saturating_add(window.duration_real_us) > ctx.time_us {
            carried
                .add_window(window.clone())
                .map_err(|error| format!("Nanally carried contact pause: {error}"))?;
        }
    }
    clock.lag = carried;
    clock.action = Some(Box::new(Action {
        kind,
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx
            .time_us
            .checked_add(start_delay_us)
            .ok_or("Nanally action start overflow")?,
        recovery: windows.recovery,
        jump: windows.jump,
        movement: windows.movement,
        switch: windows.switch,
        skill: windows.skill,
        owned_action,
        held_until,
        groups,
        contacts: Default::default(),
    }));
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.hit_time_us = deadline(&clock, clock.action.as_ref().unwrap(), hits[0].1)?;
    ctx.actor.character.nanally_mut()?.clock = clock;
    notifications::schedule(ctx, kind)?;
    Ok(())
}

pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let (Some(group), Some((delay, duration, rate))) =
        (ctx.packet_group_id, profile::pause(ctx.damage_effect_id))
    else {
        return Ok(());
    };
    let mut clock = std::mem::take(&mut ctx.actor.character.nanally_mut()?.clock);
    if clock.action.as_mut().is_some_and(|action| {
        action.command == ctx.command_index
            && action.groups.contains_key(group)
            && action.contacts.insert(group.into())
    }) {
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Nanally contact delay overflow")?,
                duration_real_us: duration,
                dilated_rate_ppm: rate,
            })
            .map_err(|error| format!("Nanally contact pause: {error}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.nanally_mut()?.clock = clock;
    Ok(())
}

pub(crate) fn recovery(actor: &ActorRuntime, input: &str) -> Option<(u64, crate::HypothesisId)> {
    let super::super::RuntimeState::Nanally(state) = &actor.character else {
        return None;
    };
    let clock = &state.clock;
    let action = clock.action.as_ref()?;
    let progress = match input {
        "jump" => action.jump,
        "dash" | "special" => action.movement,
        "switch_actor" => action.switch,
        "GA_Nanally_Skill" => action.skill,
        "GA_Nanally_Melee" | "GA_Nanally_UltraSkill" => action.recovery,
        _ => return None,
    };
    deadline(clock, action, progress)
        .ok()
        .map(|time| {
            if input == "GA_Nanally_Melee" {
                time.max(action.held_until.unwrap_or(0))
            } else {
                time
            }
        })
        .map(|time| (time, MODEL))
}

pub(crate) fn input(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.nanally_mut()?;
    let fresh = held::fresh(state, now);
    let kind = state.clock.action.as_ref().map(|action| action.kind);
    end(&mut ctx.state.actors[owner], ctx.queue)?;
    let state = ctx.state.actors[owner].character.nanally_mut()?;
    state.clock.input_command = Some(command);
    if input == crate::simulation::PlayerInput::Dash && fresh {
        state.heavy_resume = match kind {
            Some(Kind::Heavy(stage @ 0..=1)) => Some(stage + 1),
            Some(Kind::Heavy(_)) => None,
            _ => state.heavy_resume,
        };
        // The dodge preserves the next stage; its real recovery still consumes time.
        state.normal_started_us = Some(now);
    } else {
        state.normal_stage = 0;
        state.normal_started_us = None;
        state.heavy_resume = None;
    }
    Ok(())
}

pub(crate) fn validate_interrupt(
    request: &crate::simulation::GameScenarioRequest,
) -> Result<(), String> {
    if request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        != Some(MODEL.description())
    {
        return Err("Nanally interruption requires action_clock_v1".into());
    }
    Ok(())
}

pub(crate) fn interrupt(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before = actor.locked_until_us;
    let clock = &actor.character.nanally_mut()?.clock;
    if before > now && clock.action.is_none() && clock.input_command.is_none() {
        return Err("Nanally interruption reached an unbound pending action".into());
    }
    let cooldown = actor.ability_cooldowns.get("GA_Nanally_Skill").copied();
    let count = ctx
        .queue
        .iter()
        .filter(|event| matches!(event.data, GameEventData::HitDelivery { .. }))
        .count();
    let command = end(actor, ctx.queue)?;
    actor.locked_until_us = now;
    actor.character.nanally_mut()?.normal_stage = 0;
    actor.character.nanally_mut()?.normal_started_us = None;
    actor.character.nanally_mut()?.heavy_resume = None;
    let cancelled = count
        - ctx
            .queue
            .iter()
            .filter(|event| matches!(event.data, GameEventData::HitDelivery { .. }))
            .count();
    Ok(crate::simulation::GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: command.filter(|_| before > now),
        cancelled_hit_deliveries: cancelled,
        recovery_before_us: before,
        recovery_after_us: now,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: cooldown,
        field_cancelled: false,
    })
}

impl Clock {
    pub(crate) fn global_freeze(
        &mut self,
        owner: usize,
        start: u64,
        duration: u64,
    ) -> Result<(), String> {
        self.lag
            .add_freeze(owner, start, duration)
            .map_err(|error| error.to_string())
    }
}
