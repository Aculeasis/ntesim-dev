//! Player montage notifies and independently launched finite actors.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext, OwnedAction};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::BTreeSet;
pub(crate) mod a2;
pub(crate) mod burst;
mod events;
pub(crate) mod held;
mod profile;
mod skill;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::IroiActionClockV1;
pub(crate) const QTE: &str = "GA_Oneiroi_QTE";
pub(crate) const QTE_PACKET: &str = "GE_Player_Oneiroi_QTE_Damage";
pub(crate) const SUPPORT: crate::runtime_api::SupportEntry = crate::runtime_api::SupportEntry {
    ability_id: QTE,
    damage_effect_id: QTE_PACKET,
    hit_delay_us: 784_391,
    recovery_us: 1_340_667,
    dispatch_model: MODEL,
    packet_model: None,
    extra_damage_effects: &[],
};
#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    active: Option<Action>,
    moon_packets: BTreeSet<String>,
    pause_packets: BTreeSet<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Normal(usize),
    Skill,
    Transform,
    Branch {
        stage: held::BranchStage,
        hold_us: u64,
    },
    LucidDream {
        hold_us: u64,
    },
    Burst,
    Support,
}
#[derive(Clone, Debug)]
struct Action {
    owner: usize,
    command: usize,
    start: u64,
    kind: Kind,
    source: OwnedAction,
    groups: BTreeSet<String>,
    cooldown_started: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Notify {
    NormalHit,
    NormalActor,
    SkillCooldown,
    SkillHeal,
    SupportHit,
    SupportHeal,
    SkillGun,
    SkillSyringe,
    SkillTrumpet,
    BranchTag { stage: held::BranchStage },
    LucidShot(usize),
    BurstHeal,
    BurstHit,
    BurstCarrier,
    BurstRelease,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActorKind {
    NormalThree,
    SkillGun,
    SkillSyringe,
    SkillTrumpet,
    BranchGun,
    BranchSyringe,
    BranchTrumpet,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Montage {
        owner: usize,
        command: usize,
        progress: u64,
        notify: Notify,
    },
    Actor {
        owner: usize,
        source: Box<OwnedAction>,
        remaining: u8,
        kind: ActorKind,
    },
    A2Creation {
        iroi_idx: usize,
        candidate: usize,
        command_index: usize,
    },
    LambAi {
        owner: usize,
        ally_index: usize,
        lamb_type: super::regression::LambType,
        skill_number: u8,
    },
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn normal_effect(effect: &str) -> bool {
    profile::EFFECTS.contains(&effect)
}
pub(crate) fn clock_moon_effect(effect: &str) -> bool {
    normal_effect(effect) || effect == QTE_PACKET
}
pub(crate) fn admission(
    req: &GameScenarioRequest,
    _command: usize,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if !accepted(req) {
        return None;
    }
    if ability == burst::ABILITY && !super::dream::accepted(req) {
        let message = "Iroi native Burst requires its declared Dream reservation model".to_owned();
        return Some(crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        });
    }
    None
}

fn moon(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !clock_moon_effect(ctx.damage_effect_id) || ctx.actor.character.iroi_mut()?.dream_active {
        return Ok(());
    }
    let group = ctx
        .packet_group_id
        .ok_or("Iroi native Moon packet has no identity")?;
    let state = ctx.actor.character.iroi_mut()?;
    let clock = state
        .clock
        .as_mut()
        .ok_or("Iroi native Moon clock missing")?;
    if clock.moon_packets.insert(group.into()) {
        let base = super::skill::packet_moon(ctx.damage_effect_id)?
            .ok_or("Iroi normal Moon row missing")?;
        super::skill::grant_moon(ctx.config, state, base)?;
    }
    Ok(())
}

pub(crate) fn push(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    time_us: u64,
    event: Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: crate::execution::CausalPriority::BuffTickOrExpiry,
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::IroiClock(event)),
    });
    Ok(())
}
fn due(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|e| format!("Iroi montage deadline: {e:?}"))
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::Content(crate::callbacks::ContentEvent::IroiClock(Event::Montage {
            owner,
            command,
            ..
        })) => *owner == action.owner && *command == action.command,
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|group| action.groups.contains(group)),
        _ => false,
    }
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
        if let GameEventData::Content(crate::callbacks::ContentEvent::IroiClock(Event::Montage {
            progress,
            ..
        })) = &event.data
        {
            event.time_us = due(clock, action, *progress)?;
        }
    }
    actor.locked_until_us = due(clock, action, profile::recovery(action.kind))?;
    if matches!(action.kind, Kind::Normal(_)) {
        actor.character.iroi_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    queue.replace(events);
    if action.kind == Kind::Burst {
        burst::project(
            actor,
            queue,
            action.owner,
            due(clock, action, burst::RELEASE)?,
        )?;
    }
    Ok(())
}
pub(crate) fn input_lock(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Iroi(state) = &actor.character else {
        return None;
    };
    if input == "GA_Oneiroi_Melee" && state.riposte_ready {
        return Some((now, MODEL));
    }
    let Some(clock) = state.clock.as_ref() else {
        return (input == "jump").then_some((now, MODEL));
    };
    let Some(action) = clock.active.as_ref() else {
        return (input == "jump").then_some((now, MODEL));
    };
    Some((
        due(clock, action, profile::gate(action.kind, input)).ok()?,
        MODEL,
    ))
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    if let Some(clock) = &mut actor.character.iroi_mut()?.clock
        && let Some(action) = clock.active.take()
    {
        queue.retain(|event| !belongs(event, &action));
        clock
            .pause_packets
            .retain(|group| !action.groups.contains(group));
        if action.kind == Kind::Skill && !action.cooldown_started {
            actor.ability_cooldowns.insert(
                skill::ABILITY.into(),
                now.checked_add(3_000_000)
                    .ok_or("Iroi cancel cooldown overflow")?,
            );
        }
        if action.kind == Kind::Burst {
            burst::cancel(actor, queue, action.owner, now)?;
        }
    }
    Ok(())
}
/// Return false for routes whose native clock is still a tracked prerequisite.
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if !accepted(ctx.request) && ctx.ability_id != QTE {
        return Ok(false);
    }
    let command = ctx.command_index;
    let hold_duration = ctx
        .request
        .hold_inputs
        .get(&command)
        .map(|h| h.duration_us)
        .unwrap_or(0);
    let stage = {
        let state = ctx.actor.character.iroi_mut()?;
        crate::characters::normal::at_boundary(
            state.normal_stage,
            state.normal_started_us,
            ctx.time_us,
        )
    };
    stop(ctx.actor, ctx.queue, ctx.time_us)?;

    let packet_id = ctx.packet_metadata.map(|p| p.damage_effect_id.as_str());
    let is_primary = packet_id == super::primary_packet(ctx.ability_id);

    if ctx.ability_id == QTE && is_primary {
        return cast_support(ctx);
    }

    if ctx.ability_id == skill::ABILITY {
        if hold_duration >= 300_000 {
            return held::cast_transform(ctx, command);
        } else if is_primary {
            return held::cast_skill(ctx, command);
        }
    }

    if ctx.ability_id == burst::ABILITY && is_primary {
        return held::cast_burst(ctx, command);
    }

    if ctx.ability_id == "GA_Oneiroi_Melee"
        || packet_id.is_some_and(super::branch::is_branch_packet)
    {
        let current_stage = if ctx
            .actor
            .character
            .iroi()?
            .normal_started_us
            .is_some_and(|prev| ctx.time_us.saturating_sub(prev) <= 2_000_000)
        {
            ctx.actor.character.iroi()?.normal_stage
        } else {
            0
        };

        let is_lucid = packet_id == Some(super::branch::BRANCH5)
            || (current_stage == 4 && hold_duration >= 200_000);
        let is_daydream = !is_lucid
            && (packet_id.is_some_and(|p| {
                matches!(
                    p,
                    super::branch::BRANCH1 | super::branch::BRANCH2 | super::branch::BRANCH3
                )
            }) || (current_stage != 4 && hold_duration >= 300_000));

        if is_lucid {
            return held::cast_lucid_dream(ctx, command, hold_duration);
        } else if is_daydream {
            return held::cast_daydream(ctx, command, hold_duration, packet_id);
        } else if is_primary {
            return held::cast_normal(ctx, command, stage);
        }
    }

    Ok(false)
}

fn cast_support(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    ctx.record_hypothesis(MODEL)?;
    ctx.record_hypothesis(super::skill::SKILL_HEAL_HYPOTHESIS_ID)?;
    let source = OwnedAction::capture(ctx)?;
    ctx.cancel_primary_delivery()?;
    let state = ctx.actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    let mut clock = state.clock.take().unwrap_or_default();
    clock.lag = HitlagTracker::default();
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx.time_us,
        kind: Kind::Support,
        source,
        groups: Default::default(),
        cooldown_started: false,
    });
    for (progress, notify) in [
        (SUPPORT.hit_delay_us, Notify::SupportHit),
        (784_675, Notify::SupportHeal),
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
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(true)
}

pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if (!accepted(ctx.request) && ctx.damage_effect_id != QTE_PACKET) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    if ctx.damage_effect_id == QTE_PACKET && !active_support(ctx.actor, ctx.command_index)? {
        return Ok(());
    }
    moon(ctx)?;
    let Some((duration, delay)) = profile::pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    let Some(mut clock) = ctx.actor.character.iroi_mut()?.clock.take() else {
        return Ok(());
    };
    let group = ctx
        .packet_group_id
        .ok_or("Iroi player contact has no packet identity")?;
    if clock.pause_packets.remove(group) {
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Iroi delayed pause overflow")?,
                duration_real_us: duration,
                dilated_rate_ppm: if ctx.damage_effect_id == QTE_PACKET {
                    300_000
                } else {
                    100_000
                },
            })
            .map_err(|e| format!("Iroi contact pause: {e:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.iroi_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn active_support(
    actor: &crate::simulation::state::ActorRuntime,
    command: usize,
) -> Result<bool, String> {
    Ok(actor
        .character
        .iroi()?
        .clock
        .as_ref()
        .and_then(|clock| clock.active.as_ref())
        .is_some_and(|action| action.command == command && matches!(action.kind, Kind::Support)))
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    events::apply(ctx, event)
}

pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before_cd = actor.ability_cooldowns.get(skill::ABILITY).copied();
    let mut receipt = crate::simulation::GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: before_cd,
        skill_cooldown_after_us: before_cd,
        field_cancelled: false,
    };
    if actor.hp == 0 {
        return Ok(receipt);
    }
    let Some(clock) = actor.character.iroi_mut()?.clock.as_ref() else {
        return Ok(receipt);
    };
    let Some(action) = clock.active.as_ref() else {
        return Ok(receipt);
    };
    let full = [
        "dash",
        "GA_Oneiroi_UltraSkill",
        "GA_Oneiroi_Skill",
        "GA_Oneiroi_Melee",
    ]
    .iter()
    .map(|input| profile::gate(action.kind, input))
    .max()
    .ok_or("Iroi action gate missing")?;
    if now >= due(clock, action, full)? {
        return Ok(receipt);
    }
    receipt.interrupted_command_index = Some(action.command);
    let before = ctx
        .queue
        .iter()
        .filter(|event| matches!(event.data, GameEventData::HitDelivery { .. }))
        .count();
    stop(actor, ctx.queue, now)?;
    receipt.cancelled_hit_deliveries = before
        - ctx
            .queue
            .iter()
            .filter(|event| matches!(event.data, GameEventData::HitDelivery { .. }))
            .count();
    actor.locked_until_us = now;
    let state = actor.character.iroi_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.riposte_ready = false;
    receipt.recovery_after_us = now;
    receipt.skill_cooldown_after_us = actor.ability_cooldowns.get(skill::ABILITY).copied();
    Ok(receipt)
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
