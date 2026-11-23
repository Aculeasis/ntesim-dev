//! Owner-only finite montage; World timers and persistent effects use their
//! declared shared policy independently of this local montage's dilation.
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
pub(super) mod profile;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::DaffodilOrdinaryClockV1;
const SKILL: &str = "GA_Daffodill_Skill";
// GA_Daffodill_QTE leads into the AppearAttack section at its guarded
// JumpSection_CustomCheck delay. This is an owner montage coordinate.
const SUPPORT_STEP_TRANSITION_US: u64 = 1_150_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Normal(usize),
    Skill(bool),
    Burst,
    Plunge,
    Riposte,
    Support,
    Step,
    SupportParry,
    Dodge(bool),
    Branch,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    active: Option<Action>,
    displaced: Option<(u64, u64)>,
}
#[derive(Clone, Debug)]
struct Action {
    owner: usize,
    command: usize,
    start: u64,
    kind: Kind,
    groups: BTreeMap<String, u64>,
    contacts: BTreeSet<String>,
    skill_started: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    SkillCooldown {
        owner: usize,
        command: usize,
    },
    FinisherCheck {
        owner: usize,
        command: usize,
    },
    PhantomExpiry {
        owner: usize,
        generation: u64,
    },
    DiscordExpiry {
        owner: usize,
        target_index: usize,
        generation: u64,
    },
}

pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
fn schedule_group(ctx: &mut CastContext<'_>, kind: Kind, effect: &str) -> Result<String, String> {
    if kind == Kind::SupportParry && effect == super::phantom::APPEAR_SUCCESS {
        let coordinate =
            crate::simulation::inputs::normal_coordinate(ctx.config, "GA_Daffodill_Melee")?;
        let previous = ctx.native_array_index;
        ctx.native_array_index = Some(coordinate);
        let result = ctx.schedule_native_aoe_group(effect, ctx.time_us);
        ctx.native_array_index = previous;
        return result;
    }
    ctx.schedule_native_aoe_group(effect, ctx.time_us)
}
fn due(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|e| format!("Daffodil montage deadline: {e:?}"))
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|group| action.groups.contains_key(group)),
        GameEventData::Content(crate::callbacks::ContentEvent::Daffodil(
            Event::SkillCooldown { owner, command } | Event::FinisherCheck { owner, command },
        ))
        | GameEventData::DaffodilStepAfterSupport { owner, command } => {
            *owner == action.owner && *command == action.command
        }
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
    let qualified_finisher =
        action.kind == Kind::SupportParry && super::phantom::parry_succeeded(actor, action.command);
    let chained_step = action.kind == Kind::SupportParry;
    let transition = if qualified_finisher {
        super::phantom::FINISHER_US
    } else {
        SUPPORT_STEP_TRANSITION_US
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
                .and_then(|group| action.groups.get(group))
                .copied(),
            GameEventData::Content(crate::callbacks::ContentEvent::Daffodil(
                Event::SkillCooldown { .. },
            )) => Some(if action.kind == Kind::Skill(true) {
                110_328
            } else {
                182_290
            }),
            GameEventData::Content(crate::callbacks::ContentEvent::Daffodil(
                Event::FinisherCheck { .. },
            )) => Some(super::phantom::FINISHER_US),
            GameEventData::DaffodilStepAfterSupport { .. } => Some(transition),
            _ => None,
        };
        if let Some(progress) = progress {
            event.time_us = due(clock, action, progress)?;
        }
    }
    let recovery = if chained_step {
        transition
    } else {
        profile::recovery(action.kind)
    };
    actor.locked_until_us = due(clock, action, recovery)?;
    if matches!(action.kind, Kind::Normal(_)) {
        actor.character.daffodil_mut()?.normal_started_us = Some(actor.locked_until_us);
    }
    queue.replace(events);
    if chained_step {
        crate::runtime_api::defer_current_admissions(queue, actor.locked_until_us);
    }
    Ok(())
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let mut cancel_phantom = None;
    {
        let Some(clock) = &mut actor.character.daffodil_mut()?.clock else {
            return Ok(());
        };
        if let Some((start, end)) = clock.displaced.take() {
            let mut retained = HitlagTracker::default();
            for window in clock
                .lag
                .windows()
                .iter()
                .filter(|w| !(w.start_real_us == start && w.duration_real_us == end - start))
            {
                retained
                    .add_window(window.clone())
                    .map_err(|e| format!("Daffodil cancelled suspension: {e:?}"))?;
            }
            clock.lag = retained;
        }
        if let Some(action) = clock.active.take() {
            queue.retain(|event| !belongs(event, &action));
            if matches!(action.kind, Kind::Step | Kind::SupportParry) {
                cancel_phantom = Some(action.command);
            }
            if matches!(action.kind, Kind::Skill(_)) && !action.skill_started {
                actor.ability_cooldowns.insert(
                    SKILL.into(),
                    now.checked_add(3_000_000)
                        .ok_or("Daffodil cancel cooldown overflow")?,
                );
            }
        }
    }
    if let Some(command) = cancel_phantom {
        super::phantom::cancel_active(actor, command)?;
    }
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let held = super::held::kind(ctx)?;
    let secondary = if held.is_some() {
        held
    } else {
        super::secondary::select_cast(ctx)?
    };
    let phantom = if held.is_none() {
        super::phantom::plan(
            ctx,
            secondary.is_some_and(|(kind, _, _)| kind == Kind::Support),
        )?
    } else {
        None
    };
    if secondary.is_none()
        && phantom.is_none()
        && ctx.packet_metadata.map(|p| p.damage_effect_id.as_str())
            != super::primary_packet(ctx.ability_id)
    {
        return Err(
            "Daffodil ordinary clock requires its canonical Normal/Skill/Finale primary".into(),
        );
    }
    ctx.record_hypothesis(MODEL)?;
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let (kind, hits, landing) = if let Some(plan) = phantom {
        match plan {
            super::phantom::Plan::Step(hits) => (Kind::Step, hits, 0),
            super::phantom::Plan::QualifiedSupportParry(hits) => (Kind::SupportParry, hits, 0),
        }
    } else if let Some((kind, hits, landing)) = secondary {
        (kind, hits.to_vec(), landing)
    } else {
        let (kind, hits) = match ctx.ability_id {
            "GA_Daffodill_Melee" => {
                ctx.record_hypothesis(super::skill::CHAIN_HYPOTHESIS_ID)?;
                let state = ctx.actor.character.daffodil_mut()?;
                state.normal_stage = crate::characters::normal::after_gap(
                    state.normal_stage,
                    state.normal_started_us,
                    ctx.time_us,
                );
                let stage = state.normal_stage;
                state.normal_stage = (stage + 1) % 5;
                (Kind::Normal(stage), profile::NORMAL[stage].to_vec())
            }
            SKILL => {
                let echo = super::resonance::capture(ctx)? == 2;
                (
                    Kind::Skill(echo),
                    if echo {
                        profile::ECHOES.to_vec()
                    } else {
                        profile::SKILL.to_vec()
                    },
                )
            }
            "GA_Daffodill_UltraSkill" => {
                super::insight::on_burst(ctx)?;
                super::phantom::burst(ctx)?;
                ctx.actor.ability_cooldowns.insert(
                    ctx.ability_id.into(),
                    ctx.time_us
                        .checked_add(20_000_000)
                        .ok_or("Daffodil Finale cooldown overflow")?,
                );
                (Kind::Burst, profile::BURST.to_vec())
            }
            _ => return Err("Daffodil ordinary clock has no selected action".into()),
        };
        (kind, hits, 0)
    };
    if !matches!(kind, Kind::Normal(_)) {
        let state = ctx.actor.character.daffodil_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
        if matches!(kind, Kind::Skill(_) | Kind::Burst) {
            ctx.record_hypothesis(super::skill::PACKETS_HYPOTHESIS_ID)?;
        }
    }
    if ctx.targets.len() > 1 {
        ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != Some(hits[0].0)
    {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let mut groups = BTreeMap::from([(ctx.scope_primary_aoe_group(hits[0].0)?, hits[0].1)]);
    for &(effect, progress) in &hits[1..] {
        groups.insert(schedule_group(ctx, kind, effect)?, progress);
    }
    let mut clock = ctx
        .actor
        .character
        .daffodil_mut()?
        .clock
        .take()
        .ok_or("Daffodil clock not initialized")?;
    let mut inherited = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        inherited
            .add_window(window.clone())
            .map_err(|e| format!("Daffodil inherited pause: {e:?}"))?;
    }
    clock.lag = inherited;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx
            .time_us
            .checked_add(landing)
            .ok_or("Daffodil landing deadline overflow")?,
        kind,
        groups,
        contacts: BTreeSet::new(),
        skill_started: false,
    });
    if matches!(kind, Kind::Step | Kind::SupportParry)
        && super::phantom::finisher_scheduled(ctx.actor, ctx.command_index)
    {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(super::phantom::FINISHER_US)
                .ok_or("Daffodil finisher marker overflow")?,
            priority: crate::execution::CausalPriority::ActionAdmission,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Daffodil(
                Event::FinisherCheck {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                },
            )),
        });
    }
    if kind == Kind::SupportParry {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx
                .time_us
                .checked_add(SUPPORT_STEP_TRANSITION_US)
                .ok_or("Daffodil Support transition overflow")?,
            priority: crate::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::DaffodilStepAfterSupport {
                owner: ctx.actor_index,
                command: ctx.command_index,
            },
        });
    }
    if matches!(kind, Kind::Skill(_)) {
        ctx.actor.ability_cooldowns.remove(SKILL);
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: crate::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::Daffodil(
                Event::SkillCooldown {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                },
            )),
        });
    }
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.daffodil_mut()?.clock = Some(clock);
    if kind == Kind::Burst {
        ctx.actor.switch_action_locked_until_us = ctx
            .time_us
            .checked_add(5_087_548)
            .ok_or("Daffodil Finale switch lock overflow")?;
    } else if matches!(kind, Kind::Support | Kind::SupportParry) {
        ctx.actor.switch_action_locked_until_us = ctx
            .time_us
            .checked_add(863_029)
            .ok_or("Daffodil Support switch lock overflow")?;
    }
    Ok(())
}

pub(crate) fn step_after_support(
    request: &GameScenarioRequest,
    state: &mut crate::simulation::state::GameSimulatorState,
    queue: &mut BinaryHeap,
    owner: usize,
    command: usize,
) -> Result<Option<crate::runtime_api::OwnerEntryAttack>, String> {
    let actor = &state.actors[owner];
    if state.active_actor_index != owner
        || actor.hp == 0
        || !matches!(&actor.character, crate::characters::RuntimeState::Daffodil(kit)
            if kit.clock.as_ref().and_then(|clock| clock.active.as_ref()).is_some_and(|action|
                action.command == command && action.kind == Kind::SupportParry))
    {
        return Ok(None);
    }
    let step = super::phantom::step_entry(request, state, owner)?;
    if step.is_some() {
        stop(&mut state.actors[owner], queue, state.current_time_us)?;
    }
    Ok(step)
}

pub(crate) fn dodge(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    back: bool,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if super::held::accepted(ctx.request)
        && !back
        && normal_jump_gate(&ctx.state.actors[owner], now).is_some()
        && now < ctx.state.actors[owner].locked_until_us
    {
        crate::runtime_api::record_hypothesis_id(
            ctx.request,
            &mut ctx.state.execution_model_assumptions,
            super::held::MODEL,
        )?;
        let actor = &mut ctx.state.actors[owner];
        let mut clock = actor
            .character
            .daffodil_mut()?
            .clock
            .take()
            .ok_or("Daffodil suspended clock missing")?;
        let until = now
            .checked_add(1_014_541)
            .ok_or("Daffodil Displaced Edge overflow")?;
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: owner,
                start_real_us: now,
                duration_real_us: 1_014_541,
                dilated_rate_ppm: 0,
            })
            .map_err(|e| format!("Daffodil suspension: {e:?}"))?;
        clock.displaced = Some((now, until));
        project(&clock, actor, ctx.queue, now, false)?;
        actor.character.daffodil_mut()?.clock = Some(clock);
        return Ok(());
    }
    let actor = &mut ctx.state.actors[owner];
    stop(actor, ctx.queue, now)?;
    actor.character.daffodil_mut()?.normal_stage = 0;
    actor.character.daffodil_mut()?.normal_started_us = None;
    let mut clock = actor
        .character
        .daffodil_mut()?
        .clock
        .take()
        .ok_or("Daffodil Dodge clock missing")?;
    clock.active = Some(Action {
        owner,
        command,
        start: now,
        kind: Kind::Dodge(back),
        groups: Default::default(),
        contacts: Default::default(),
        skill_started: false,
    });
    project(&clock, actor, ctx.queue, now, true)?;
    actor.character.daffodil_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn dodge_immune(actor: &ActorRuntime, now: u64) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Daffodil(state) if state.clock.as_ref().is_some_and(|c|c.displaced.is_some_and(|(_,end)|now<end)||c.active.as_ref().is_some_and(|a|matches!(a.kind,Kind::Dodge(_)))&&now<actor.locked_until_us))
}
pub(crate) fn normal_jump_gate(actor: &ActorRuntime, now: u64) -> Option<u64> {
    let crate::characters::RuntimeState::Daffodil(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let action = clock.active.as_ref()?;
    if !matches!(action.kind, Kind::Normal(_))
        || now >= actor.locked_until_us
        || clock.displaced.is_some_and(|(_, end)| now < end)
    {
        return None;
    }
    due(clock, action, cancel::gate(action.kind, "jump")?).ok()
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let pause = profile::pause(ctx.damage_effect_id);
    if !accepted(ctx.request) || pause == 0 || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let mut clock = ctx
        .actor
        .character
        .daffodil_mut()?
        .clock
        .take()
        .ok_or("Daffodil contact has no clock")?;
    let group = ctx
        .packet_group_id
        .ok_or("Daffodil contact has no causal group")?;
    let active = clock
        .active
        .as_mut()
        .is_some_and(|a| a.groups.contains_key(group) && a.contacts.insert(group.into()));
    if active {
        clock
            .lag
            .add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx.time_us,
                duration_real_us: pause,
                dilated_rate_ppm: 0,
            })
            .map_err(|e| format!("Daffodil lost progress: {e:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.daffodil_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn after_parry_success(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let Some(clock) = actor.character.daffodil_mut()?.clock.take() else {
        return Ok(());
    };
    if clock
        .active
        .as_ref()
        .is_some_and(|action| action.kind == Kind::SupportParry)
    {
        project(&clock, actor, queue, now, false)?;
    }
    actor.character.daffodil_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::SkillCooldown { owner, command } => {
            let actor = &mut ctx.state.actors[owner];
            if actor.hp == 0 {
                return Ok(());
            }
            if let Some(clock) = &mut actor.character.daffodil_mut()?.clock
                && let Some(action) = &mut clock.active
                && action.command == command
            {
                action.skill_started = true;
                actor.ability_cooldowns.insert(
                    SKILL.into(),
                    ctx.state
                        .current_time_us
                        .checked_add(16_000_000)
                        .ok_or("Daffodil Skill cooldown overflow")?,
                );
            }
        }
        Event::FinisherCheck { owner, command } => {
            super::phantom::finish_check(ctx, owner, command)?;
        }
        Event::PhantomExpiry { owner, generation } => {
            let now = ctx.state.current_time_us;
            let phantom = &mut ctx.state.actors[owner].character.daffodil_mut()?.phantom;
            if phantom.generation == generation
                && phantom.expires_at_us.is_some_and(|until| until <= now)
            {
                phantom.charges = 0;
                phantom.expires_at_us = None;
            }
        }
        Event::DiscordExpiry {
            owner,
            target_index,
            generation,
        } => {
            super::discord::expire(
                ctx.request,
                ctx.state,
                ctx.queue,
                ctx.sequence,
                ctx.trace,
                owner,
                target_index,
                generation,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn phantom_marker_deadline(
    actor: &ActorRuntime,
    command: usize,
) -> Result<Option<u64>, String> {
    let crate::characters::RuntimeState::Daffodil(owner) = &actor.character else {
        return Ok(None);
    };
    let Some(clock) = owner.clock.as_ref() else {
        return Ok(None);
    };
    let Some(action) = clock.active.as_ref().filter(|action| {
        action.command == command && matches!(action.kind, Kind::Step | Kind::SupportParry)
    }) else {
        return Ok(None);
    };
    due(clock, action, super::phantom::FINISHER_US).map(Some)
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
