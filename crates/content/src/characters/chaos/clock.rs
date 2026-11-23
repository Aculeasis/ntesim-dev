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
pub(super) mod cancel;
mod profile;
pub(super) mod secondary;
pub(super) mod verdict;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChaosOrdinaryClockV1;

#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    tracker: HitlagTracker,
    action: Option<Action>,
    connected: BTreeSet<String>,
}

impl Clock {
    pub(super) fn end_action(&mut self) {
        self.action = None;
    }
}

#[derive(Clone, Debug)]
struct Action {
    owner: usize,
    command: usize,
    start: u64,
    input_start: u64,
    recovery: u64,
    kind: Kind,
    groups: BTreeMap<String, u64>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Normal(usize, bool),
    AppearNormal,
    Support,
    Dodge(bool),
    Riposte,
    Plunge,
    Skill,
    Burst,
    VerdictPlain(bool),
    VerdictFull(bool),
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
    if !accepted(request) {
        return Ok(());
    }
    if !super::resources::accepted(request) {
        return Err("Chaos ordinary clock requires ordinary_crime_v1".into());
    }
    let config = request
        .actors
        .iter()
        .find(|config| config.character_id == "character:1071")
        .ok_or("Chaos clock config missing")?;
    let bonus = super::parameters::warrant_bonus(config)?;
    let state = actor.character.chaos_mut()?;
    state.clock = Some(Clock::default());
    state.warrant_bonus = Some(bonus);
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
                .ok_or("Chaos clock requires grouped primary packets")?;
            first_sequences
                .entry(id.clone())
                .and_modify(|seq| *seq = (*seq).min(event.seq))
                .or_insert(event.seq);
        }
    }
    let mut groups: Vec<_> = first_sequences.into_iter().collect();
    groups.sort_by_key(|(_, seq)| *seq);
    if groups.len() != offsets.len() {
        return Err("Chaos source clock packet count mismatch".into());
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
        GameEventData::Content(crate::callbacks::ContentEvent::Chaos(event)) => {
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
            .map_err(|error| format!("Chaos caster deadline: {error:?}"))
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
            GameEventData::Content(crate::callbacks::ContentEvent::Chaos(event)) => {
                event.anchor(action.owner, action.command)
            }
            _ => None,
        };
        if let Some(offset) = offset {
            event.time_us = deadline(offset)?;
        }
    }
    actor.locked_until_us = deadline(action.recovery)?;
    if matches!(action.kind, Kind::Normal(..) | Kind::AppearNormal) {
        actor.character.chaos_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    if matches!(action.kind, Kind::Burst) {
        let state = actor.character.chaos_mut()?;
        state.burst_switch_until_us = deadline(super::form::SWITCH_LOCK_US)?;
        state.burst_immunity_until_us = deadline(super::form::IMMUNITY_END_US)?;
    }
    if matches!(action.kind, Kind::Skill) {
        actor.ability_cooldowns.insert(
            "GA_Chaos071_Skill".into(),
            deadline(profile::SKILL_CD_MARKER)?
                .checked_add(15_000_000)
                .ok_or("Chaos Skill cooldown overflow")?,
        );
    }
    if matches!(action.kind, Kind::VerdictFull(..)) {
        actor.character.chaos_mut()?.verdict_immunity_until_us = actor.locked_until_us.min(
            action
                .start
                .checked_add(1_100_000)
                .ok_or("Chaos Verdict immunity overflow")?,
        );
    }
    queue.replace(events);
    Ok(())
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let normal = ctx.ability_id == "GA_Chaos071_Melee";
    let burst = ctx.ability_id == "GA_Chaos071_UltraSkill";
    let support =
        ctx.ability_id == super::support::ABILITY && super::support::accepted(ctx.request);
    let appear = normal && ctx.actor.character.chaos_mut()?.entry_normal_ready;
    let (offsets, recovery) = if normal {
        let stage = (ctx.actor.character.chaos_mut()?.normal_stage + 4) % 5;
        if appear {
            (profile::APPEAR_HITS, profile::RECOVERY[0])
        } else if stage == 4 && super::form::active(ctx.actor) {
            (profile::TRANSFORMED_LAST, 1_090_035)
        } else {
            (profile::HITS[stage], profile::RECOVERY[stage])
        }
    } else if ctx.ability_id == "GA_Chaos071_Skill" {
        (profile::SKILL_HITS, profile::SKILL_RECOVERY)
    } else if burst && super::form::accepted(ctx.request) {
        (profile::BURST_HITS, profile::BURST_RECOVERY)
    } else if support {
        (profile::SUPPORT_HITS, super::support::RECOVERY)
    } else {
        return Err(
            "UNSUPPORTED_CHAOS_FORM: Burst with the ordinary clock requires its form binding"
                .into(),
        );
    };
    ctx.record_hypothesis(MODEL)?;
    let groups = packet_anchors(ctx, offsets)?;
    ctx.actor.character.chaos_mut()?.verdict = None;
    let mut clock = ctx
        .actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos ordinary clock not initialized")?;
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
            .map_err(|error| format!("Chaos inherited hitlag: {error:?}"))?;
    }
    clock.tracker = tracker;
    clock.connected.clear();
    clock.action = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx.time_us,
        input_start: ctx.time_us,
        recovery,
        kind: if appear {
            Kind::AppearNormal
        } else if normal {
            Kind::Normal(
                (ctx.actor.character.chaos_mut()?.normal_stage + 4) % 5,
                super::form::active(ctx.actor),
            )
        } else if support {
            Kind::Support
        } else if burst {
            Kind::Burst
        } else {
            Kind::Skill
        },
        groups,
    });
    if !normal && !burst && !support && crate::gear::needs_successful_skill_finish(ctx.config) {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(recovery)
                .ok_or("Chaos Skill finish overflow")?,
            priority: crate::execution::CausalPriority::ModePause,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Chaos(
                super::events::Event::SkillFinish {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    progress: recovery,
                },
            )),
        });
    }
    project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.chaos_mut()?.clock = Some(clock);
    ctx.actor.character.chaos_mut()?.entry_normal_ready = false;
    if burst {
        ctx.actor.ability_cooldowns.insert(
            ctx.ability_id.into(),
            ctx.time_us
                .checked_add(15_000_000)
                .ok_or("Chaos Skill cooldown overflow")?,
        );
    }
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let pause = profile::pause(ctx.damage_effect_id);
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 || pause == 0 {
        return Ok(());
    }
    let id = ctx
        .packet_group_id
        .ok_or("Chaos hit pause requires a causal group")?;
    let mut clock = ctx
        .actor
        .character
        .chaos_mut()?
        .clock
        .take()
        .ok_or("Chaos clock missing at contact")?;
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
            .map_err(|error| format!("Chaos hit pause: {error:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us)?;
    }
    ctx.actor.character.chaos_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.chaos_mut()?;
    state.verdict = None;
    state.verdict_immunity_until_us = 0;
    state.entry_normal_ready = false;
    state.riposte_ready = false;
    if let Some(clock) = &mut ctx.state.actors[owner].character.chaos_mut()?.clock
        && let Some(action) = clock.action.take()
    {
        ctx.queue.retain(|event| !belongs(event, &action));
    }
    Ok(())
}

pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let model = crate::HypothesisId::ChaosForcedInterruptV1;
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        model,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let mut receipt = crate::simulation::GameActorInterruptReceipt {
        event_index,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: None,
        skill_cooldown_after_us: None,
        field_cancelled: false,
    };
    if actor.hp == 0 {
        return Ok(receipt);
    }
    let state = actor.character.chaos_mut()?;
    let Some(action) = state.clock.as_ref().and_then(|clock| clock.action.as_ref()) else {
        return Ok(receipt);
    };
    let live_continuation = if super::cancel::full_lifetime(ctx.request) {
        cancel::continuation_end(
            state
                .clock
                .as_ref()
                .ok_or("Chaos continuation clock missing")?,
            action,
        )?
        .is_some_and(|end| now < end)
    } else {
        false
    };
    if now >= actor.locked_until_us && !live_continuation {
        return Ok(receipt);
    }
    let cooldown = if matches!(action.kind, Kind::Skill) {
        Some(
            now.checked_add(if now.saturating_sub(action.start) < 200_000 {
                3_000_000
            } else {
                15_000_000
            })
            .ok_or("Chaos interrupted Skill cooldown overflow")?,
        )
    } else {
        None
    };
    let action = state
        .clock
        .as_mut()
        .and_then(|clock| clock.action.take())
        .ok_or("Chaos interrupted action missing")?;
    ctx.queue.retain(|event| {
        if belongs(event, &action) {
            receipt.cancelled_hit_deliveries +=
                usize::from(matches!(event.data, GameEventData::HitDelivery { .. }));
            false
        } else {
            true
        }
    });
    if let Some(deadline) = cooldown {
        receipt.skill_cooldown_before_us =
            actor.ability_cooldowns.get("GA_Chaos071_Skill").copied();
        actor
            .ability_cooldowns
            .insert("GA_Chaos071_Skill".into(), deadline);
        receipt.skill_cooldown_after_us = Some(deadline);
    }
    actor.locked_until_us = now;
    let state = actor.character.chaos_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.burst_switch_until_us = now;
    state.burst_immunity_until_us = now;
    state.verdict_immunity_until_us = now;
    state.verdict = None;
    state.entry_normal_ready = false;
    state.riposte_ready = false;
    receipt.interrupted_command_index = Some(action.command);
    receipt.recovery_after_us = now;
    Ok(receipt)
}

pub(crate) fn validate_interrupt(request: &GameScenarioRequest) -> Result<(), String> {
    let model = crate::HypothesisId::ChaosForcedInterruptV1;
    if !accepted(request)
        || request
            .model_assumptions
            .get(model.as_str())
            .map(String::as_str)
            != Some(model.description())
    {
        return Err(
            "Chaos interrupt_events require ordinary_clock_v1 and forced_interrupt_v1".into(),
        );
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
