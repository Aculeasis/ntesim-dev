//! Source-coordinate ordinary actions retain exact owner rate curves and real resource time.
use crate::runtime_api::{ActorHitContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagCurveWindow, HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) mod controls;
pub(super) mod profile;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::FadiaOrdinaryClockV1;
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
    recovery: u64,
    jump: u64,
    skill_gate: u64,
    skill: bool,
    enhanced: bool,
    a5: bool,
    groups: BTreeMap<String, u64>,
    contacts: BTreeSet<String>,
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    super::effects::accepted(req)
        || req
            .model_assumptions
            .get(MODEL.as_str())
            .map(String::as_str)
            == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !accepted(req) {
        return Ok(());
    }
    if super::effects::accepted(req) {
        for config in req
            .actors
            .iter()
            .filter(|a| a.character_id == "character:1039")
        {
            if config.energy_capacity != 120 {
                return Err("Fadia full-kit Ultimate capacity must be120".into());
            }
        }
        for action in &req.actions {
            if let GameActionCommand::InvokeAbility {
                actor_index,
                ability_id,
                damage_effect_id,
                energy_cost,
                ..
            } = action
                && req.actors[*actor_index].character_id == "character:1039"
            {
                let cost = if ability_id == "GA_Fadia_UltraSkill" {
                    120
                } else {
                    0
                };
                if *energy_cost != cost
                    || (super::primary_packet(ability_id) != Some(damage_effect_id.as_str())
                        && !(ability_id == "GA_Fadia_Melee"
                            && damage_effect_id == super::secondary::PLUNGE))
                {
                    return Err("Fadia full-kit input requires its canonical primary packet and native Ultimate cost".into());
                }
            }
        }
    }
    if super::effects::accepted(req) {
        super::secondary::validate(req)?;
    }
    for action in &req.actions {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            energy_cost,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1039"
            && !super::effects::accepted(req)
            && (ability_id != "GA_Fadia_Melee"
                || damage_effect_id != "GE_Player_Fadia_Melee1_Damage"
                || *energy_cost != 0)
        {
            return Err(
                "Fadia ordinary_clock_v1 currently binds canonical zero-cost Normal only".into(),
            );
        }
    }
    Ok(())
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    matches!(&event.data,GameEventData::HitDelivery{metadata,..} if metadata.binding.energy_group_id.as_ref().is_some_and(|g|action.groups.contains_key(g)))
        || matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::Fadia(e)) if e.owner == action.owner && e.command == action.command)
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
        if let GameEventData::HitDelivery { metadata, .. } = &event.data
            && let Some(progress) = metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|g| action.groups.get(g))
        {
            event.time_us = clock
                .lag
                .resolve_real_deadline(action.owner, action.start, *progress)
                .map_err(|e| format!("Fadia packet clock: {e}"))?;
        } else if let GameEventData::Content(crate::callbacks::ContentEvent::Fadia(e)) = &event.data
        {
            event.time_us = clock
                .lag
                .resolve_real_deadline(action.owner, action.start, e.boundary.offset())
                .map_err(|e| format!("Fadia notify clock: {e}"))?;
        }
    }
    actor.locked_until_us = clock
        .lag
        .resolve_real_deadline(action.owner, action.start, action.recovery)
        .map_err(|e| format!("Fadia recovery clock: {e}"))?;
    actor.character.fadia_mut()?.normal_started_us = Some(actor.locked_until_us);
    queue.replace(events);
    Ok(())
}
pub(crate) fn stop(actor: &mut ActorRuntime, queue: &mut BinaryHeap) -> Result<(), String> {
    if let Some(clock) = &mut actor.character.fadia_mut()?.clock
        && let Some(action) = clock.active.take()
    {
        queue.retain(|e| !belongs(e, &action));
    }
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let full = super::effects::accepted(ctx.request);
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str())
        != super::primary_packet(ctx.ability_id)
        && !(full
            && ctx.ability_id == "GA_Fadia_Melee"
            && matches!(
                ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()),
                Some(super::secondary::PLUNGE | super::secondary::RIPOSTE)
            ))
    {
        return Err("Fadia clock requires a canonical primary packet".into());
    }
    ctx.record_hypothesis(if full { super::effects::MODEL } else { MODEL })?;
    stop(ctx.actor, ctx.queue)?;
    let profile::Plan {
        hits,
        recovery,
        skill,
        enhanced,
        a5,
        jump,
        skill_gate,
        landing,
    } = profile::select(ctx, full)?;
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(hits[0].0) {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let mut groups = BTreeMap::from([(
        ctx.scope_primary_aoe_group(hits[0].0)?,
        hits[0]
            .1
            .checked_add(landing)
            .ok_or("Fadia landing overflow")?,
    )]);
    for &(effect, progress) in &hits[1..] {
        groups.insert(
            ctx.schedule_native_aoe_group(effect, ctx.time_us)?,
            progress
                .checked_add(landing)
                .ok_or("Fadia packet landing overflow")?,
        );
    }
    let mut clock = ctx
        .actor
        .character
        .fadia_mut()?
        .clock
        .take()
        .ok_or("Fadia ordinary clock not initialized")?;
    let mut retained = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        retained
            .add_window(window.clone())
            .map_err(|e| format!("Fadia inherited pause: {e}"))?;
    }
    for window in clock
        .lag
        .curve_windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > ctx.time_us)
    {
        retained
            .add_curve_window(window.clone())
            .map_err(|e| format!("Fadia inherited curve: {e}"))?;
    }
    clock.lag = retained;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx.time_us,
        recovery: recovery
            .checked_add(landing)
            .ok_or("Fadia landing recovery overflow")?,
        jump: jump
            .checked_add(landing)
            .ok_or("Fadia landing jump overflow")?,
        skill_gate: skill_gate
            .checked_add(landing)
            .ok_or("Fadia landing Skill gate overflow")?,
        skill,
        enhanced,
        a5,
        groups,
        contacts: Default::default(),
    });
    if skill {
        for boundary in [Boundary::SkillCooldown, Boundary::SkillMark] {
            crate::runtime_api::increment_sequence(ctx.sequence)?;
            ctx.queue.push(ScheduledGameEvent {
                time_us: ctx.time_us,
                priority: ntesim_core::execution::CausalPriority::ProcCallback,
                seq: *ctx.sequence,
                data: GameEventData::Content(crate::callbacks::ContentEvent::Fadia(Event {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    boundary,
                })),
            });
        }
    }
    if ctx.ability_id == "GA_Fadia_UltraSkill" {
        ctx.actor.ability_cooldowns.insert(
            ctx.ability_id.into(),
            ctx.time_us
                .checked_add(20000000)
                .ok_or("Fadia Burst cooldown overflow")?,
        );
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: ntesim_core::execution::CausalPriority::ProcCallback,
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::FadiaEffects(
                super::effects::Event::Heal {
                    owner: ctx.actor_index,
                    command: ctx.command_index,
                    enhanced,
                },
            )),
        });
    }
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.fadia_mut()?.clock = Some(clock);
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    SkillCooldown,
    SkillMark,
}
impl Boundary {
    fn offset(self) -> u64 {
        match self {
            Self::SkillCooldown => 198376,
            Self::SkillMark => 908426,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(crate) owner: usize,
    pub(crate) command: usize,
    pub(crate) boundary: Boundary,
}

pub(crate) fn event(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event: Event,
) -> Result<(), String> {
    if ctx.state.actors[event.owner].hp == 0 {
        return Ok(());
    }
    match event.boundary {
        Boundary::SkillCooldown => {
            let until = ctx
                .state
                .current_time_us
                .checked_add(16000000)
                .ok_or("Fadia Skill cooldown overflow")?;
            ctx.state.actors[event.owner]
                .ability_cooldowns
                .insert("GA_Fadia_Skill".into(), until);
            Ok(())
        }
        Boundary::SkillMark => super::effects::mark(ctx, event.owner, event.command),
    }
}
pub(crate) fn validate_live(
    state: &super::skill::State,
    ability: &str,
    packet: &str,
    _time: u64,
) -> Result<(), String> {
    if super::primary_packet(ability) != Some(packet)
        && !(ability == "GA_Fadia_Melee"
            && (packet == super::secondary::PLUNGE
                || packet == super::secondary::RIPOSTE && state.riposte_ready))
    {
        return Err("Fadia requires her canonical primary packet".into());
    }
    Ok(())
}
pub(crate) fn flags(actor: &ActorRuntime, command: usize) -> (bool, bool) {
    let crate::characters::RuntimeState::Fadia(state) = &actor.character else {
        return (false, false);
    };
    state
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .filter(|a| a.command == command)
        .map(|a| (a.enhanced, a.a5))
        .unwrap_or((false, false))
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(pause) = profile::pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    let mut clock = ctx
        .actor
        .character
        .fadia_mut()?
        .clock
        .take()
        .ok_or("Fadia contact clock missing")?;
    if let Some(group) = ctx.packet_group_id
        && clock.active.as_mut().is_some_and(|a| {
            a.command == ctx.command_index
                && a.groups.contains_key(group)
                && a.contacts.insert(group.into())
        })
    {
        match pause {
            profile::Pause::Flat {
                duration,
                delay,
                rate_ppm,
            } => clock.lag.add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Fadia pause onset overflow")?,
                duration_real_us: duration,
                dilated_rate_ppm: rate_ppm,
            }),
            profile::Pause::Curve { duration, points } => {
                clock.lag.add_curve_window(HitlagCurveWindow {
                    actor_index: ctx.actor_index,
                    start_real_us: ctx.time_us,
                    duration_real_us: duration,
                    points: points.to_vec(),
                })
            }
        }
        .map_err(|e| format!("Fadia connected pause: {e}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.fadia_mut()?.clock = Some(clock);
    Ok(())
}

pub(crate) fn interrupt(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before = actor.locked_until_us;
    let cooldown_before = actor.ability_cooldowns.get("GA_Fadia_Skill").copied();
    let interrupted_skill = actor
        .character
        .fadia_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .is_some_and(|a| a.skill);
    let command = actor
        .character
        .fadia_mut()?
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
        stop(actor, ctx.queue)?;
        if interrupted_skill && cooldown_before.is_none_or(|until| until <= now) {
            actor.ability_cooldowns.insert(
                "GA_Fadia_Skill".into(),
                now.checked_add(3000000)
                    .ok_or("Fadia interrupted Skill cooldown overflow")?,
            );
        }
        actor.locked_until_us = now;
        let state = actor.character.fadia_mut()?;
        state.normal_stage = 0;
        state.normal_started_us = None;
    }
    Ok(crate::simulation::GameActorInterruptReceipt {
        event_index: event,
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
        skill_cooldown_before_us: cooldown_before,
        skill_cooldown_after_us: actor.ability_cooldowns.get("GA_Fadia_Skill").copied(),
        field_cancelled: false,
    })
}

impl Event {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.command, false)
    }
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
