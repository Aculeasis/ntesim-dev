//! Finite owner montage and channel deadlines on the shared real-time axis.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
pub(super) mod profile;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::EdgarFullKitV1;
pub(super) const SKILL: &str = "GA_Edgar_Skill";
pub(super) const BURST: &str = "GA_Edgar_UltraSkill";
pub(super) const SKILL_PACKET: &str = "GE_Player_Edgar_Skill_Damage";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Normal(usize),
    Appear,
    Skill(Option<u64>),
    Burst,
    Plunge,
    Riposte,
    Support,
    Dodge(bool),
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    active: Option<Action>,
}
#[derive(Clone, Debug)]
struct Action {
    owner: usize,
    command: usize,
    start: u64,
    kind: Kind,
    groups: BTreeMap<String, u64>,
    contacts: BTreeSet<String>,
    cooldown_started: bool,
    loop_entered: bool,
    locked_target: usize,
    a2: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActionEvent {
    Heal { coordinate: usize },
    FieldSpawn { coordinate: usize },
    Pulse { group: String },
    LoopEntered,
    Cooldown { additional: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Action {
        owner: usize,
        command: usize,
        progress: u64,
        kind: ActionEvent,
    },
    FieldTick {
        owner: usize,
        generation: u64,
    },
    FieldExpiry {
        owner: usize,
        generation: u64,
    },
    Refund {
        owner: usize,
        raw: u64,
    },
}

pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    for (&command, hold) in &req.hold_inputs {
        if let Some(crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            ..
        }) = req.actions.get(command)
            && req.actors[*actor_index].character_id == "character:1021"
            && (!accepted(req)
                || ability_id != SKILL
                || damage_effect_id != SKILL_PACKET
                || hold.moving
                || hold.reserved_stocks.is_some())
        {
            return Err(
                "Edgar held input requires full_kit_v1 and canonical stationary Skill".into(),
            );
        }
    }
    super::secondary::validate(req)
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|g| action.groups.contains_key(g)),
        GameEventData::Content(crate::callbacks::ContentEvent::Edgar(Event::Action {
            owner,
            command,
            ..
        })) => *owner == action.owner && *command == action.command,
        _ => false,
    }
}
fn due(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|e| format!("Edgar montage deadline: {e:?}"))
}
fn project(
    clock: &Clock,
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    fresh: bool,
) -> Result<(), String> {
    let Some(action) = &clock.active else {
        return Ok(());
    };
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if (!fresh && event.time_us <= now) || !belongs(event, action) {
            continue;
        }
        let progress = match &event.data {
            GameEventData::HitDelivery { metadata, .. } => metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|g| action.groups.get(g))
                .copied(),
            GameEventData::Content(crate::callbacks::ContentEvent::Edgar(Event::Action {
                progress,
                ..
            })) => Some(*progress),
            _ => None,
        };
        if let Some(progress) = progress {
            event.time_us = due(clock, action, progress)?;
        }
    }
    actor.locked_until_us = due(clock, action, profile::recovery(action.kind))?;
    if matches!(action.kind, Kind::Normal(_)) {
        actor.character.edgar_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    if action.kind == Kind::Riposte && !action.contacts.is_empty() {
        let state = actor.character.edgar_mut()?;
        state.normal_stage = 4;
        state.normal_started_us = Some(actor.locked_until_us);
    }
    queue.replace(events);
    Ok(())
}
fn additional(elapsed: u64, a2: bool) -> Result<u64, String> {
    let value: u64 = (u128::from(elapsed) * 1676 / 1000)
        .try_into()
        .map_err(|_| "Edgar channel cooldown overflow")?;
    Ok(if a2 { value.min(3_000_000) } else { value })
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let Some(clock) = &mut actor.character.edgar_mut()?.clock else {
        return Ok(());
    };
    if let Some(action) = clock.active.take() {
        queue.retain(|e| !belongs(e, &action));
        if matches!(action.kind, Kind::Skill(_)) && !action.cooldown_started {
            let duration = if action.loop_entered {
                16_000_000
                    + additional(
                        now.saturating_sub(action.start).saturating_sub(800_000),
                        action.a2,
                    )?
            } else {
                3_000_000
            };
            actor.ability_cooldowns.insert(
                SKILL.into(),
                now.checked_add(duration)
                    .ok_or("Edgar interrupt cooldown overflow")?,
            );
        }
    }
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    let continued = crate::characters::normal::continues_at(
        ctx.actor.character.edgar_mut()?.normal_started_us,
        ctx.time_us,
    );
    let secondary = super::secondary::select_cast(ctx)?;
    let (kind, hits, landing) = if let Some(plan) = secondary {
        plan
    } else {
        profile::plan(ctx, continued)?
    };
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    if !matches!(kind, Kind::Normal(_)) {
        let state = ctx.actor.character.edgar_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    let mut groups = BTreeMap::new();
    if let Some(&(effect, progress)) = hits.first() {
        if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(effect) {
            ctx.replace_native_primary_packet(effect)?;
        }
        groups.insert(ctx.scope_primary_aoe_group(effect)?, progress);
        for &(effect, progress) in &hits[1..] {
            groups.insert(
                ctx.schedule_native_aoe_group(effect, ctx.time_us)?,
                progress,
            );
        }
    } else {
        ctx.cancel_primary_delivery()?;
    }
    if matches!(kind, Kind::Skill(_)) {
        ctx.queue.retain(|e|!matches!(&e.data,GameEventData::HitDelivery{target_index,metadata,..} if *target_index!=ctx.target_index&&metadata.binding.energy_group_id.as_ref().is_some_and(|g|groups.contains_key(g))));
    }
    let mut clock = ctx
        .actor
        .character
        .edgar_mut()?
        .clock
        .take()
        .ok_or("Edgar clock not initialized")?;
    // Retain only a still-live owner pause from the cancelled action.
    let mut lag = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        lag.add_window(window.clone())
            .map_err(|e| format!("Edgar inherited pause: {e:?}"))?;
    }
    clock.lag = lag;
    let start = ctx
        .time_us
        .checked_add(landing)
        .ok_or("Edgar landing overflow")?;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start,
        kind,
        groups,
        contacts: Default::default(),
        cooldown_started: false,
        loop_entered: false,
        locked_target: ctx.target_index,
        a2: ctx.config.awakening.has_effect(2),
    });
    profile::schedule_controls(ctx, &mut clock, kind)?;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.edgar_mut()?.clock = Some(clock);
    Ok(())
}
pub(super) fn schedule(
    ctx: &mut CastContext<'_>,
    progress: u64,
    kind: ActionEvent,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: ctx
            .time_us
            .checked_add(progress)
            .ok_or("Edgar event overflow")?,
        priority: ntesim_core::execution::CausalPriority::ActionAdmission,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Edgar(Event::Action {
            owner: ctx.actor_index,
            command: ctx.command_index,
            progress,
            kind,
        })),
    });
    Ok(())
}
pub(crate) fn enqueue(
    ctx: &mut CallbackContext<'_>,
    time_us: u64,
    event: Event,
) -> Result<(), String> {
    if time_us > ctx.request.absolute_limit_us {
        return Ok(());
    }
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us,
        priority: ntesim_core::execution::CausalPriority::BuffTickOrExpiry,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Edgar(event)),
    });
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::FieldTick { owner, generation } => super::effects::tick(ctx, owner, generation),
        Event::FieldExpiry { owner, generation } => super::effects::expire(ctx, owner, generation),
        Event::Refund { owner, raw } => {
            let before =
                crate::simulation::energy::EnergySnapshot::capture(&ctx.state.actors[owner]);
            ctx.state.actors[owner].grant_energy(raw)?;
            let raw_exact = crate::simulation::ExactFraction::new(raw.into(), 1)?;
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "ProcCallback".into(),
                event_kind: "edgar_r6_refund".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(owner),
                to_actor: Some(owner),
                energy_change: Some(before.receipt(
                    &ctx.state.actors[owner],
                    crate::simulation::GameEnergySource::EdgarHit,
                    owner,
                    owner,
                    Some(raw_exact),
                )?),
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    exact_raw: Some(raw_exact),
                    ..Default::default()
                }),
            });
            Ok(())
        }
        Event::Action {
            owner,
            command,
            kind,
            ..
        } => {
            let live = ctx.state.actors[owner].hp > 0
                && ctx.state.actors[owner]
                    .character
                    .edgar_mut()?
                    .clock
                    .as_ref()
                    .and_then(|c| c.active.as_ref())
                    .is_some_and(|a| a.command == command);
            if !live {
                return Ok(());
            }
            match kind {
                ActionEvent::Heal { coordinate } => {
                    super::effects::skill_heal(ctx, owner, coordinate)
                }
                ActionEvent::FieldSpawn { coordinate } => {
                    super::effects::spawn(ctx, owner, coordinate)
                }
                ActionEvent::LoopEntered => {
                    ctx.state.actors[owner]
                        .character
                        .edgar_mut()?
                        .clock
                        .as_mut()
                        .unwrap()
                        .active
                        .as_mut()
                        .unwrap()
                        .loop_entered = true;
                    Ok(())
                }
                ActionEvent::Cooldown { additional } => {
                    ctx.state.actors[owner].ability_cooldowns.insert(
                        SKILL.into(),
                        ctx.state
                            .current_time_us
                            .checked_add(16_000_000 + additional)
                            .ok_or("Edgar cooldown overflow")?,
                    );
                    ctx.state.actors[owner]
                        .character
                        .edgar_mut()?
                        .clock
                        .as_mut()
                        .unwrap()
                        .active
                        .as_mut()
                        .unwrap()
                        .cooldown_started = true;
                    Ok(())
                }
                ActionEvent::Pulse { group } => retarget(ctx, owner, &group),
            }
        }
    }
}
fn retarget(ctx: &mut CallbackContext<'_>, owner: usize, group: &str) -> Result<(), String> {
    let action = ctx.state.actors[owner]
        .character
        .edgar_mut()?
        .clock
        .as_mut()
        .unwrap()
        .active
        .as_mut()
        .unwrap();
    let target = if ctx.state.targets[action.locked_target].hp_current > 0 {
        Some(action.locked_target)
    } else {
        ctx.state.targets.iter().position(|t| t.hp_current > 0)
    };
    if let Some(target) = target {
        action.locked_target = target;
    }
    let mut events = ctx.queue.drain().collect::<Vec<_>>();
    events.retain_mut(|e| {
        if let GameEventData::HitDelivery {
            target_index,
            metadata,
            ..
        } = &mut e.data
            && metadata.binding.energy_group_id.as_deref() == Some(group)
        {
            if let Some(target) = target {
                *target_index = target;
            } else {
                return false;
            }
        }
        true
    });
    ctx.queue.replace(events);
    Ok(())
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let (pause, delay) = profile::pause(ctx.damage_effect_id);
    if pause == 0 {
        return Ok(());
    }
    let mut clock = ctx
        .actor
        .character
        .edgar_mut()?
        .clock
        .take()
        .ok_or("Edgar contact clock missing")?;
    if let Some(group) = ctx.packet_group_id
        && clock
            .active
            .as_mut()
            .is_some_and(|a| a.groups.contains_key(group) && a.contacts.insert(group.into()))
    {
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Edgar hitpause onset overflow")?,
                duration_real_us: pause,
                dilated_rate_ppm: 0,
            })
            .map_err(|e| format!("Edgar hitpause: {e:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.edgar_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn recovery(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Edgar(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let Some(action) = &clock.active else {
        return (input == "jump").then_some((now, MODEL));
    };
    let progress = profile::gate(action.kind, input)?;
    Some((due(clock, action, progress).ok()?, MODEL))
}

pub(crate) fn immune(actor: &ActorRuntime, now: u64) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Edgar(s) if s.clock.as_ref().and_then(|c|c.active.as_ref()).is_some_and(|a|matches!(a.kind,Kind::Dodge(_)))&&now<actor.locked_until_us)
}
pub(crate) fn committed(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    input: crate::simulation::PlayerInput,
) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    stop(&mut ctx.state.actors[owner], ctx.queue, now)?;
    let state = ctx.state.actors[owner].character.edgar_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    if input == crate::simulation::PlayerInput::Dash {
        let back = ctx.request.dash_directions.get(&command)
            == Some(&crate::simulation::GameDashDirection::Back);
        let mut clock = state.clock.take().ok_or("Edgar Dodge clock missing")?;
        clock.active = Some(Action {
            owner,
            command,
            start: now,
            kind: Kind::Dodge(back),
            groups: Default::default(),
            contacts: Default::default(),
            cooldown_started: false,
            loop_entered: false,
            locked_target: 0,
            a2: false,
        });
        project(&clock, &mut ctx.state.actors[owner], ctx.queue, now, true)?;
        ctx.state.actors[owner].character.edgar_mut()?.clock = Some(clock);
    }
    Ok(())
}
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before = actor.locked_until_us;
    let cooldown = actor.ability_cooldowns.get(SKILL).copied();
    let command = actor
        .character
        .edgar_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .filter(|_| now < before)
        .map(|a| a.command);
    let count = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    if command.is_some() {
        stop(actor, ctx.queue, now)?;
        actor.locked_until_us = now;
        actor.character.edgar_mut()?.normal_stage = 0;
    }
    Ok(crate::simulation::GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: command,
        cancelled_hit_deliveries: count
            - ctx
                .queue
                .iter()
                .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
                .count(),
        recovery_before_us: before,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: actor.ability_cooldowns.get(SKILL).copied(),
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
