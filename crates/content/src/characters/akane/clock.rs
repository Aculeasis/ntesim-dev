//! Finite ordinary montage anchors projected through the shared hitlag integrator.
use super::parameters::PARAMETERS;
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
pub(crate) mod secondary;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::AkaneOrdinaryClockV1;

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
    AppearNormal,
    MeleePre,
    Qte,
    Dodge(bool),
    Riposte,
    Plunge,
    SkillPress,
    SkillHold,
    SyncedOvertone,
    SyncedReverb,
    BurstStage1,
    BurstStage2,
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
        actor.character.akane_mut()?.clock = Some(Clock::default());
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
                .ok_or("Akane clock requires grouped primary packets")?;
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
            "Akane source clock packet count mismatch: {} groups vs {} offsets",
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
        GameEventData::Content(crate::callbacks::ContentEvent::Akane(event)) => {
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
            .map_err(|error| format!("Akane caster deadline: {error:?}"))
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
            GameEventData::Content(crate::callbacks::ContentEvent::Akane(event)) => {
                event.anchor(action.owner, action.command)
            }
            _ => None,
        };
        if let Some(offset) = offset {
            event.time_us = deadline(offset)?;
        }
    }
    actor.locked_until_us = deadline(action.recovery)?;
    if matches!(
        action.kind,
        Kind::Normal(..) | Kind::AppearNormal | Kind::MeleePre
    ) {
        actor.character.akane_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    match action.kind {
        Kind::Qte => {
            actor.switch_action_locked_until_us = deadline(profile::QTE_SWITCH_LOCK)?;
        }
        Kind::SyncedOvertone => {
            actor.switch_action_locked_until_us = deadline(profile::PROSKILL_1_SWITCH_LOCK)?;
        }
        Kind::SyncedReverb => {
            actor.switch_action_locked_until_us = deadline(profile::PROSKILL_2_SWITCH_LOCK)?;
        }
        Kind::BurstStage1 => {
            actor.switch_action_locked_until_us = deadline(profile::BURST1_SWITCH_LOCK)?;
        }
        Kind::BurstStage2 => {
            actor.switch_action_locked_until_us = deadline(profile::BURST2_SWITCH_LOCK)?;
        }
        Kind::SkillPress => {
            let ready = deadline(profile::SKILL_PRESS_CD_MARKER)?
                .checked_add(PARAMETERS.skill_cooldown_us)
                .ok_or("Akane Skill cooldown overflow")?;
            actor
                .ability_cooldowns
                .insert("GA_Akane_Skill".into(), ready);
            actor.character.akane_mut()?.rhythm_wave_ready_at_us = ready;
        }
        Kind::SkillHold => {
            let ready = deadline(profile::SKILL_HOLD_CD_MARKER)?
                .checked_add(PARAMETERS.skill_cooldown_us)
                .ok_or("Akane Skill cooldown overflow")?;
            actor
                .ability_cooldowns
                .insert("GA_Akane_Skill".into(), ready);
            actor.character.akane_mut()?.rhythm_wave_ready_at_us = ready;
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
    let (landing, offsets, recovery) = match kind {
        Kind::Normal(0) => (0, profile::MELEE1_HITS, profile::NORMAL_RECOVERY[0]),
        Kind::AppearNormal | Kind::MeleePre => {
            (0, profile::MELEE_PRE_HITS, profile::NORMAL_RECOVERY[0])
        }
        Kind::Normal(1) => (0, profile::MELEE2_HITS, profile::NORMAL_RECOVERY[1]),
        Kind::Normal(2) => (0, profile::MELEE3_HITS, profile::NORMAL_RECOVERY[2]),
        Kind::Normal(3) => (0, profile::MELEE4_HITS, profile::NORMAL_RECOVERY[3]),
        Kind::Qte => (0, profile::QTE_HITS, profile::QTE_RECOVERY),
        Kind::SkillPress => (0, profile::SKILL_PRESS_HITS, profile::SKILL_PRESS_RECOVERY),
        Kind::SkillHold => (0, profile::SKILL_HOLD_HITS, profile::SKILL_HOLD_RECOVERY),
        Kind::SyncedOvertone => (0, profile::PROSKILL_1_HITS, profile::PROSKILL_1_RECOVERY),
        Kind::SyncedReverb => (0, profile::PROSKILL_2_HITS, profile::PROSKILL_2_RECOVERY),
        Kind::BurstStage1 => (0, profile::BURST1_HITS, profile::BURST1_RECOVERY),
        Kind::BurstStage2 => (0, profile::BURST2_HITS, profile::BURST2_RECOVERY),
        Kind::Riposte => (
            0,
            std::slice::from_ref(&profile::RIPOSTE_HIT),
            profile::RIPOSTE_RECOVERY,
        ),
        Kind::Plunge => (
            ctx.request
                .plunge_inputs
                .get(&ctx.command_index)
                .ok_or("Akane Stage Dive clock profile missing")?
                .landing_delay_us,
            std::slice::from_ref(&profile::PLUNGE_HIT),
            profile::PLUNGE_RECOVERY,
        ),
        Kind::Dodge(..) => return Err("Akane dodge uses secondary input clock".into()),
        _ => return Err("Akane normal clock stage out of range".into()),
    };
    ctx.record_hypothesis(MODEL)?;
    let groups = packet_anchors(ctx, offsets)?;
    let mut clock = ctx
        .actor
        .character
        .akane_mut()?
        .clock
        .take()
        .ok_or("Akane ordinary clock not initialized")?;
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
            .map_err(|error| format!("Akane inherited hitlag: {error:?}"))?;
    }
    clock.tracker = tracker;
    clock.connected.clear();
    let start = ctx
        .time_us
        .checked_add(landing)
        .ok_or("Akane clock start overflow")?;
    clock.action = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start,
        input_start: ctx.time_us,
        recovery,
        kind,
        groups,
    });
    if matches!(
        kind,
        Kind::SkillPress | Kind::SkillHold | Kind::SyncedOvertone | Kind::SyncedReverb
    ) && crate::gear::needs_successful_skill_finish(ctx.config)
    {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(recovery)
                .ok_or("Akane Skill finish overflow")?,
            priority: crate::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Akane(
                super::events::Event::SkillFinish {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress: recovery,
                },
            )),
        });
    }
    project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.akane_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let pause = profile::pause(ctx.damage_effect_id);
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 || pause == 0 {
        return Ok(());
    }
    let Some(id) = ctx.packet_group_id else {
        return Ok(());
    };
    let mut clock = ctx
        .actor
        .character
        .akane_mut()?
        .clock
        .take()
        .ok_or("Akane clock missing at hit contact")?;
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
                start_real_us: ctx.time_us,
                duration_real_us: pause,
                dilated_rate_ppm: 0,
            })
            .map_err(|error| format!("Akane hit pause: {error:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    }
    ctx.actor.character.akane_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.akane_mut()?;
    state.entry_normal_ready = false;
    state.melee_pre_ready = false;
    state.riposte_ready = false;
    if let Some(clock) = &mut state.clock
        && let Some(action) = clock.action.take()
    {
        ctx.queue.retain(|event| !belongs(event, &action));
    }
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
