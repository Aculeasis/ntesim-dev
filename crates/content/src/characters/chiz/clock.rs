//! Ordinary finite montage and released-producer clocks on one real time axis.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
mod profile;
#[cfg(test)]
mod tests;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizOrdinaryClockV1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Normal(usize),
    Plunge,
    Riposte,
    Support,
    Dodge(bool),
    BarrenCue,
    Barren,
    Burst,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    tracker: HitlagTracker,
    action: Option<Action>,
    connected: BTreeSet<String>,
}

#[derive(Clone, Debug)]
struct Action {
    command: usize,
    owner: usize,
    start: u64,
    kind: Kind,
    // Montage coordinate and real producer delay for each causal group.
    groups: BTreeMap<String, (u64, u64)>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(super) fn jump_gate(actor: &ActorRuntime) -> Option<u64> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let action = clock.action.as_ref()?;
    deadline(clock, action, profile::gate(action.kind, "jump")?).ok()
}
pub(super) fn has_active(actor: &ActorRuntime) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Chiz(state) if state.clock.as_ref().is_some_and(|clock|clock.action.is_some()))
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if accepted(request) {
        actor.character.chiz_mut()?.clock = Some(Box::default());
    }
    Ok(())
}

fn deadline(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .tracker
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|error| format!("Chiz montage deadline: {error:?}"))
}

fn owns(event: &ScheduledGameEvent, action: &Action) -> bool {
    if let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
        super::events::Event::Market(super::market::Event::Enter { owner, command }),
    )) = &event.data
    {
        return *owner == action.owner && *command == action.command;
    }
    if let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
        super::events::Event::Barren(event),
    )) = &event.data
    {
        return event.owner == action.owner && event.command == action.command;
    }
    matches!(&event.data,GameEventData::HitDelivery {metadata,..} if metadata.binding.energy_group_id.as_ref().is_some_and(|group| action.groups.contains_key(group)))
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
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if event.time_us <= now && !fresh {
            continue;
        }
        if let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
            super::events::Event::Market(super::market::Event::Enter { owner, command }),
        )) = &event.data
            && *owner == action.owner
            && *command == action.command
        {
            event.time_us = deadline(clock, action, 3_420_264)?;
        }
        if let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
            super::events::Event::Barren(anchor),
        )) = &event.data
            && anchor.owner == action.owner
            && anchor.command == action.command
        {
            event.time_us = deadline(clock, action, anchor.progress)?;
        }
        if let GameEventData::HitDelivery { metadata, .. } = &event.data
            && let Some((progress, delay)) = metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|group| action.groups.get(group))
        {
            event.time_us = deadline(clock, action, *progress)?
                .checked_add(*delay)
                .ok_or("Chiz producer deadline overflow")?;
        }
    }
    actor.locked_until_us = deadline(clock, action, profile::recovery(action.kind))?;
    actor.character.chiz_mut()?.normal_started_us = Some(actor.locked_until_us);
    queue.replace(events);
    Ok(())
}

pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    if let Some(clock) = &mut actor.character.chiz_mut()?.clock
        && let Some(action) = clock.action.take()
    {
        // Only an already reached Normal4 producer survives a living field exit
        // or montage replacement. Defeat never leaves future owner pulses.
        let spawn = match action.kind {
            Kind::Normal(3) => Some(profile::HITS[3][0]),
            Kind::Burst => Some(2_346_949),
            _ => None,
        };
        let released = actor.hp > 0
            && spawn
                .map(|progress| deadline(clock, &action, progress))
                .transpose()?
                .is_some_and(|due| now >= due);
        queue.retain(|event| {
            !owns(event, &action)
                || released && matches!(event.data, GameEventData::HitDelivery { .. })
        });
    }
    Ok(())
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 {
        ctx.queue.retain(|event| !matches!(&event.data,
            GameEventData::HitDelivery {actor_index,ability_id,..} if *actor_index==owner && ability_id=="GA_Chiichan073_Melee")
            && !matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::Chiz(super::events::Event::HitPause {owner:source,..})) if *source==owner));
    }
    stop(&mut ctx.state.actors[owner], ctx.queue, now)?;
    if super::secondary::accepted(ctx.request)
        && let Some(clock) = &mut ctx.state.actors[owner].character.chiz_mut()?.clock
    {
        clock.tracker = HitlagTracker::default();
    }
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
    {
        return Err("Chiz ordinary clock requires the canonical Normal input".into());
    }
    ctx.record_hypothesis(MODEL)?;
    ctx.record_hypothesis(super::skill::CHAIN_HYPOTHESIS_ID)?;
    ctx.record_hypothesis(crate::HypothesisId::ChizNormalGrainV1)?;
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let state = ctx.actor.character.chiz_mut()?;
    state.normal_stage = crate::characters::normal::after_gap(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let stage = state.normal_stage;
    state.normal_stage = (stage + 1) % 5;
    let effect = profile::ROWS[stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(effect)?;
    }
    let mut emitted = vec![ctx.scope_primary_aoe_group(effect)?];
    for _ in &profile::HITS[stage][1..] {
        emitted.push(ctx.schedule_native_aoe_group(effect, ctx.hit_time_us)?);
    }
    let groups = emitted
        .into_iter()
        .zip(profile::HITS[stage])
        .map(|(group, progress)| {
            (
                group,
                if stage == 3 {
                    (profile::HITS[3][0], progress - profile::HITS[3][0])
                } else {
                    (*progress, 0)
                },
            )
        })
        .collect();
    start(ctx, Kind::Normal(stage), 0, groups)
}

pub(crate) fn start(
    ctx: &mut CastContext<'_>,
    kind: Kind,
    landing: u64,
    groups: BTreeMap<String, (u64, u64)>,
) -> Result<(), String> {
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let mut clock = ctx
        .actor
        .character
        .chiz_mut()?
        .clock
        .take()
        .ok_or("Chiz ordinary clock missing")?;
    let mut tracker = HitlagTracker::default();
    for window in
        clock.tracker.windows().iter().filter(|window| {
            window.start_real_us.saturating_add(window.duration_real_us) > ctx.time_us
        })
    {
        tracker
            .add_window(window.clone())
            .map_err(|error| format!("Chiz inherited pause: {error:?}"))?;
    }
    clock.tracker = tracker;
    clock.connected.clear();
    clock.action = Some(Action {
        command: ctx.command_index,
        owner: ctx.actor_index,
        start: ctx
            .time_us
            .checked_add(landing)
            .ok_or("Chiz landing deadline overflow")?,
        kind,
        groups,
    });
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.chiz_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn input_gate(actor: &ActorRuntime, input: &str) -> Option<u64> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let action = clock.action.as_ref()?;
    deadline(clock, action, profile::gate(action.kind, input)?).ok()
}
pub(crate) fn normal_gate(actor: &ActorRuntime) -> Option<u64> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let action = clock.action.as_ref()?;
    matches!(action.kind, Kind::Normal(_))
        .then(|| deadline(clock, action, profile::recovery(action.kind)).ok())
        .flatten()
}
pub(crate) fn live_windows(actor: &ActorRuntime, now: u64) -> Vec<HitlagWindow> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return Vec::new();
    };
    state.clock.as_ref().map_or_else(Vec::new, |clock| {
        clock
            .tracker
            .windows()
            .iter()
            .filter(|window| window.start_real_us.saturating_add(window.duration_real_us) > now)
            .cloned()
            .collect()
    })
}
pub(crate) fn inherit_pressed(actor: &mut ActorRuntime, now: u64) -> Result<(), String> {
    let windows = super::pressed::live_windows(actor, now);
    if let Some(clock) = &mut actor.character.chiz_mut()?.clock {
        for window in windows {
            if !clock.tracker.windows().contains(&window) {
                clock
                    .tracker
                    .add_window(window)
                    .map_err(|error| format!("Chiz inherited avatar pause: {error:?}"))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn dodge(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    back: bool,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .chiz_mut()?
        .clock
        .take()
        .ok_or("Chiz Dodge clock missing")?;
    clock.connected.clear();
    clock.action = Some(Action {
        command,
        owner,
        start: now,
        kind: Kind::Dodge(back),
        groups: BTreeMap::new(),
    });
    project(&clock, actor, ctx.queue, now, true)?;
    actor.character.chiz_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let (duration_us, rate) = profile::pause(ctx.damage_effect_id);
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 || duration_us == 0 {
        return Ok(());
    }
    let group = ctx
        .packet_group_id
        .ok_or("Chiz pause requires a causal packet group")?;
    let clock = ctx
        .actor
        .character
        .chiz_mut()?
        .clock
        .as_mut()
        .ok_or("Chiz clock missing at contact")?;
    if !clock
        .action
        .as_ref()
        .is_some_and(|action| action.groups.contains_key(group))
        || !clock.connected.insert(group.into())
    {
        return Ok(());
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(profile::PAUSE_DELAY)
            .ok_or("Chiz pause onset overflow")?,
        priority: crate::execution::CausalPriority::ModePause,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
            super::events::Event::HitPause {
                owner: ctx.actor_index,
                duration_us,
                rate,
            },
        )),
    });
    Ok(())
}

pub(crate) fn pause(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    duration_us: u64,
    rate: u32,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .chiz_mut()?
        .clock
        .take()
        .ok_or("Chiz clock missing at pause onset")?;
    clock
        .tracker
        .add_window(HitlagWindow {
            actor_index: owner,
            start_real_us: now,
            duration_real_us: duration_us,
            dilated_rate_ppm: rate,
        })
        .map_err(|error| format!("Chiz hit pause: {error:?}"))?;
    project(&clock, actor, ctx.queue, now, false)?;
    let ordinary = clock
        .action
        .as_ref()
        .is_some_and(|action| matches!(action.kind, Kind::Normal(_)));
    actor.character.chiz_mut()?.clock = Some(clock);
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: if ordinary {
            "chiz_normal_hit_pause"
        } else {
            "chiz_avatar_hit_pause"
        }
        .into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            duration_us: Some(duration_us),
            rate_ppm: Some(rate),
            ..Default::default()
        }),
    });
    Ok(())
}

impl Clock {
    pub(crate) fn global_freeze(
        &mut self,
        owner: usize,
        start: u64,
        duration: u64,
    ) -> Result<(), String> {
        self.tracker
            .add_freeze(owner, start, duration)
            .map_err(|error| error.to_string())
    }
}
