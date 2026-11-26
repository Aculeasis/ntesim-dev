//! Finite ordinary montage anchors projected through the shared hitlag integrator.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod cancel;
pub(crate) mod profile;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ElymsOrdinaryClockV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    pub(super) tracker: HitlagTracker,
    pub(super) action: Option<Action>,
    pub(super) connected: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub(super) struct Action {
    pub(super) owner: usize,
    pub(super) command: usize,
    pub(super) start: u64,
    pub(super) input_start: u64,
    pub(super) recovery: u64,
    pub(super) kind: Kind,
    pub(super) groups: BTreeMap<String, u64>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Kind {
    Normal(usize),
    Crosstalk(usize),
    Branch,
    Aim,
    Qte,
    Dodge(bool),
    Riposte,
    Plunge,
    Skill1,
    Skill2,
    Skill3 { extra: bool },
    BurstA,
    BurstB,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if accepted(request) {
        actor.character.elyms_mut()?.clock = Some(Clock::default());
    }
    Ok(())
}

fn packet_anchors(ctx: &CastContext<'_>, offsets: &[u64]) -> Result<BTreeMap<String, u64>, String> {
    let mut first_sequences = BTreeMap::<String, u64>::new();
    for event in ctx.queue.iter() {
        if let GameEventData::HitDelivery {
            command_index,
            actor_index,
            ability_id,
            metadata,
            ..
        } = &event.data
            && *command_index == ctx.command_index
            && *actor_index == ctx.actor_index
            && ability_id == ctx.ability_id
        {
            let id = metadata
                .binding
                .energy_group_id
                .as_ref()
                .ok_or("Elyms clock requires grouped primary packets")?;
            first_sequences
                .entry(id.clone())
                .and_modify(|seq| *seq = (*seq).min(event.seq))
                .or_insert(event.seq);
        }
    }
    let mut groups: Vec<_> = first_sequences.into_iter().collect();
    groups.sort_by_key(|(_, seq)| *seq);
    if groups.len() != offsets.len() && ctx.targets.iter().any(|target| target.hp_current > 0) {
        return Err(format!(
            "Elyms source clock packet count mismatch: {} groups vs {} offsets",
            groups.len(),
            offsets.len()
        ));
    }
    Ok(groups
        .into_iter()
        .zip(offsets)
        .map(|((id, _), offset)| (id, *offset))
        .collect())
}

fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|group| action.groups.contains_key(group)),
        GameEventData::Content(crate::callbacks::ContentEvent::Elyms(event)) => {
            event.anchor(action.owner, action.command).is_some()
        }
        _ => false,
    }
}

fn project(
    clock: &Clock,
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let Some(action) = &clock.action else {
        return Ok(());
    };
    let deadline = |offset| {
        clock
            .tracker
            .resolve_real_deadline(action.owner, action.start, offset)
            .map_err(|error| format!("Elyms caster deadline: {error:?}"))
    };
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if event.time_us <= now && now != action.input_start {
            continue;
        }
        let offset = match &event.data {
            GameEventData::HitDelivery { metadata, .. } => metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|id| action.groups.get(id))
                .copied(),
            GameEventData::Content(crate::callbacks::ContentEvent::Elyms(event)) => {
                event.anchor(action.owner, action.command)
            }
            _ => None,
        };
        if let Some(offset) = offset {
            event.time_us = deadline(offset)?;
        }
    }
    actor.locked_until_us = deadline(action.recovery)?;
    if matches!(action.kind, Kind::Normal(..) | Kind::Crosstalk(..)) {
        actor.character.elyms_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    match action.kind {
        Kind::Qte => {
            actor.switch_action_locked_until_us = deadline(profile::QTE_SWITCH_LOCK)?;
        }
        Kind::Skill2 => {
            actor.switch_action_locked_until_us = deadline(profile::SKILL2_SWITCH_LOCK)?;
        }
        Kind::Skill3 { .. } => {
            actor.switch_action_locked_until_us = deadline(profile::SKILL3_SWITCH_LOCK)?;
        }
        Kind::BurstA => {
            actor.switch_action_locked_until_us = deadline(profile::BURSTA_SWITCH_LOCK)?;
        }
        Kind::BurstB => {
            actor.switch_action_locked_until_us = deadline(profile::BURSTB_SWITCH_LOCK)?;
        }
        _ => {}
    }
    queue.replace(events);
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>, kind: Kind) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let mut offsets = Vec::new();
    let (landing, recovery) = match kind {
        Kind::Normal(stage) if stage < 5 => {
            offsets.extend(MELEE_HITS[stage].iter().map(|(_, t)| *t));
            (0, profile::NORMAL_RECOVERY[stage])
        }
        Kind::Crosstalk(stage) if stage < 3 => {
            offsets.extend(MELEE_B_HITS[stage].iter().map(|(_, t)| *t));
            (0, profile::CROSSTALK_RECOVERY[stage])
        }
        Kind::Branch => {
            offsets.extend(profile::BRANCH_HITS.iter().map(|(_, t)| *t));
            (0, profile::BRANCH_RECOVERY)
        }
        Kind::Aim => {
            offsets.extend(profile::AIM_HITS.iter().map(|(_, t)| *t));
            (0, profile::AIM_RECOVERY)
        }
        Kind::Qte => {
            offsets.push(profile::QTE_HIT);
            (0, profile::QTE_RECOVERY)
        }
        Kind::Riposte => {
            offsets.push(profile::RIPOSTE_HIT);
            (0, profile::RIPOSTE_RECOVERY)
        }
        Kind::Plunge => {
            let landing = ctx
                .request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Elyms Falling Dream clock profile missing")?
                .landing_delay_us;
            offsets.push(profile::PLUNGE_HIT);
            (landing, profile::PLUNGE_RECOVERY)
        }
        Kind::Skill1 => {
            offsets.extend(profile::SKILL1_HITS.iter().map(|(_, t)| *t));
            (0, profile::SKILL1_RECOVERY)
        }
        Kind::Skill2 => {
            offsets.extend(profile::SKILL2_HITS.iter().map(|(_, t)| *t));
            (0, profile::SKILL2_RECOVERY)
        }
        Kind::Skill3 { extra } => {
            offsets.extend(profile::SKILL3_HITS.iter().map(|(_, t)| *t));
            if extra {
                offsets.push(profile::SKILL3_EXTRA_HIT);
            }
            (0, profile::SKILL3_RECOVERY)
        }
        Kind::BurstA => {
            offsets.extend(profile::BURSTA_HITS.iter().map(|(_, t)| *t));
            (0, profile::BURSTA_RECOVERY)
        }
        Kind::BurstB => {
            offsets.extend(profile::BURSTB_HITS.iter().map(|(_, t)| *t));
            (0, profile::BURSTB_RECOVERY)
        }
        Kind::Dodge(..) => return Err("Elyms dodge uses secondary input clock".into()),
        _ => return Err("Elyms normal clock stage out of range".into()),
    };
    ctx.record_hypothesis(MODEL)?;
    let groups = packet_anchors(ctx, &offsets)?;
    let mut clock = ctx
        .actor
        .character
        .elyms_mut()?
        .clock
        .take()
        .ok_or("Elyms ordinary clock not initialized")?;
    if let Some(previous) = clock.action.take() {
        ctx.queue.retain(|event| !belongs(event, &previous));
    }
    let mut tracker = HitlagTracker::default();
    for window in
        clock.tracker.windows().iter().filter(|window| {
            window.start_real_us.saturating_add(window.duration_real_us) > ctx.time_us
        })
    {
        tracker
            .add_window(window.clone())
            .map_err(|error| format!("Elyms inherited hitlag: {error:?}"))?;
    }
    clock.tracker = tracker;
    clock.connected.clear();
    let start = ctx
        .time_us
        .checked_add(landing)
        .ok_or("Elyms clock start overflow")?;
    clock.action = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start,
        input_start: ctx.time_us,
        recovery,
        kind,
        groups,
    });
    if matches!(kind, Kind::Skill1 | Kind::Skill2 | Kind::Skill3 { .. })
        && crate::gear::needs_successful_skill_finish(ctx.config)
    {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(recovery)
                .ok_or("Elyms Skill finish overflow")?,
            priority: crate::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Elyms(
                super::events::Event::SkillFinish {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress: recovery,
                },
            )),
        });
    }
    project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.elyms_mut()?.clock = Some(clock);
    Ok(())
}

mod profile_tables {
    use super::profile;
    pub(super) const MELEE_HITS: &[&[profile::Hit]] = &[
        profile::MELEE1_HITS,
        profile::MELEE2_HITS,
        profile::MELEE3_HITS,
        profile::MELEE4_HITS,
        profile::MELEE5_HITS,
    ];
    pub(super) const MELEE_B_HITS: &[&[profile::Hit]] = &[
        profile::MELEEB1_HITS,
        profile::MELEEB2_HITS,
        profile::MELEEB3_HITS,
    ];
}
use profile_tables::{MELEE_B_HITS, MELEE_HITS};

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let Some((delay, duration, rate)) = profile::pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(id) = ctx.packet_group_id else {
        return Ok(());
    };
    let mut clock = ctx
        .actor
        .character
        .elyms_mut()?
        .clock
        .take()
        .ok_or("Elyms clock missing at hit contact")?;
    if clock
        .action
        .as_ref()
        .is_some_and(|action| action.groups.contains_key(id))
        && clock.connected.insert(id.into())
    {
        clock
            .tracker
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Elyms hit pause delay overflow")?,
                duration_real_us: duration,
                dilated_rate_ppm: rate,
            })
            .map_err(|error| format!("Elyms hit pause: {error:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    }
    ctx.actor.character.elyms_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn dodge(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    exit(ctx, owner)?;
    let now = ctx.state.current_time_us;
    let back = ctx.request.dash_directions.get(&command)
        == Some(&crate::simulation::GameDashDirection::Back);
    let actor = &mut ctx.state.actors[owner];
    let mut clock = actor
        .character
        .elyms_mut()?
        .clock
        .take()
        .ok_or("Elyms Evade clock missing")?;
    clock.connected.clear();
    clock.action = Some(Action {
        owner,
        command,
        start: now,
        input_start: now,
        recovery: if back {
            profile::EVADE_BACK_RECOVERY
        } else {
            profile::EVADE_FORWARD_RECOVERY
        },
        kind: Kind::Dodge(back),
        groups: BTreeMap::new(),
    });
    project(&clock, actor, ctx.queue, now)?;
    actor.character.elyms_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.elyms_mut()?;
    state.riposte_ready = false;
    state.entry_normal_ready = false;
    if let Some(clock) = &mut state.clock
        && let Some(action) = clock.action.take()
    {
        ctx.queue.retain(|event| !belongs(event, &action));
    }
    Ok(())
}
