//! Selected finite Normal clock, prerequisite for the remaining full owner state machine.
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
mod profile;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorOrdinaryClockV1;
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
    switch: u64,
    /// Pending held-Branch lower bound in actor microseconds from `start`.
    gate: Option<u64>,
    /// Authored real Branch landing; extends the lock without actor dilation.
    landing: Option<u64>,
    groups: BTreeMap<String, u64>,
    contacts: BTreeSet<String>,
    kind: Kind,
    a4_counter: u8,
    positive: bool,
    success: bool,
    cooldown_started: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Normal(usize),
    Entry,
    Skill,
    Burst,
    Cyclone(u8),
    Dodge(bool),
    Riposte,
    Support,
    Plunge,
    Branch,
    BranchFallback,
}
impl Kind {
    fn cyclone(self) -> Option<u8> {
        if let Self::Cyclone(n) = self {
            Some(n)
        } else {
            None
        }
    }
}
pub(super) struct Plan {
    pub(super) kind: Kind,
    pub(super) hits: Vec<profile::Hit>,
    pub(super) recovery: u64,
    pub(super) jump: u64,
    pub(super) switch: u64,
    pub(super) notifies: Vec<(u64, Boundary)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boundary {
    TapPower,
    SkillCooldown,
    HeldBirth,
    HeldTick,
    Emergency,
    CycloneSuccess,
    CycloneFinish,
    BranchGate,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(super) owner: usize,
    pub(super) command: usize,
    boundary: Boundary,
    progress: u64,
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    super::power::accepted(req)
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
    if super::power::accepted(req)
        && req.actors.iter().any(|a| {
            a.character_id == "character:1025"
                && (a.energy_capacity != 100
                    || (a.awakening.has_effect(3) && !super::burst::accepted(req)))
        })
    {
        return Err("Hathor tap_power_v1 requires Ultimate capacity100; A3 live target counting requires burst_emergency_v1".into());
    }
    for (index, action) in req.actions.iter().enumerate() {
        if !super::secondary::accepted(req)
            && matches!(action,GameActionCommand::PlayerInput{actor_index,..} if req.actors[*actor_index].character_id=="character:1025")
        {
            return Err("Hathor ordinary_clock_v1 secondary controls require their remaining owner bindings".into());
        }
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            energy_cost,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1025"
            && ((!matches!(ability_id.as_str(), "GA_Hathor_Melee")
                && !(super::power::accepted(req) && ability_id == "GA_Hathor_Skill")
                && !(super::burst::accepted(req) && ability_id == "GA_Hathor_UltraSkill"))
                || !canonical_packet(req, index, ability_id, damage_effect_id)
                || *energy_cost
                    != if ability_id == "GA_Hathor_UltraSkill" {
                        100
                    } else {
                        0
                    }
                || (req.hold_inputs.contains_key(&index)
                    && !super::held::accepted(req)
                    && !(super::branch::accepted(req)
                        && ability_id == "GA_Hathor_Melee"
                        && req.branch_inputs.contains_key(&index)))
                || (req.plunge_inputs.contains_key(&index) && !super::air::accepted(req)))
        {
            return Err("Hathor selected clock requires canonical zero-cost Normal or accepted Tap Skill; other branches require their remaining owner bindings".into());
        }
    }
    Ok(())
}
fn canonical_packet(req: &GameScenarioRequest, index: usize, ability: &str, effect: &str) -> bool {
    super::primary_packet(ability) == Some(effect)
        || (ability == "GA_Hathor_Melee"
            && ((super::secondary::accepted(req) && effect == super::secondary::RIPOSTE)
                || (super::air::accepted(req)
                    && effect == super::air::PACKET
                    && req.plunge_inputs.contains_key(&index))))
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    matches!(&event.data,GameEventData::HitDelivery {metadata,..} if metadata.binding.energy_group_id.as_ref().is_some_and(|g|action.groups.contains_key(g)))
        || matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::HathorClock(e)) if e.owner==action.owner && e.command==action.command)
        || matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::HathorAir(e)) if e.owner==action.owner && e.command==action.command)
        || matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::HathorBranch(e)) if e.owner==action.owner && e.command==action.command)
}
fn project(
    clock: &Clock,
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    fresh: bool,
) -> Result<(), String> {
    let Some(action) = clock.active.as_ref() else {
        return Ok(());
    };
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if !fresh && event.time_us <= now {
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
                .map_err(|e| format!("Hathor contact clock: {e}"))?;
        } else if let GameEventData::Content(crate::callbacks::ContentEvent::HathorClock(e)) =
            &event.data
            && e.owner == action.owner
            && e.command == action.command
        {
            event.time_us = clock
                .lag
                .resolve_real_deadline(action.owner, action.start, e.progress)
                .map_err(|e| format!("Hathor notify clock: {e}"))?;
        }
    }
    let mut deadline = clock
        .lag
        .resolve_real_deadline(action.owner, action.start, action.recovery)
        .map_err(|e| format!("Hathor recovery clock: {e}"))?;
    if let Some(gate) = action.gate {
        deadline = deadline.max(
            clock
                .lag
                .resolve_real_deadline(action.owner, action.start, gate)
                .map_err(|e| format!("Hathor Branch gate clock: {e}"))?,
        );
    }
    if let Some(landing) = action.landing {
        deadline = deadline.max(landing);
    }
    actor.locked_until_us = deadline;
    actor.character.hathor_mut()?.normal_started_us =
        matches!(action.kind, Kind::Normal(_) | Kind::Entry).then_some(deadline);
    queue.replace(events);
    Ok(())
}
/// Convert the pending held Normal into Branch at its causal gate. Returns the
/// real time of the mode boundary that must observe the authored landing.
pub(super) fn enter_branch(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    command: usize,
    launchable: bool,
    landing: u64,
) -> Result<Option<u64>, String> {
    let mut clock = actor
        .character
        .hathor_mut()?
        .clock
        .take()
        .ok_or("Hathor Branch clock missing")?;
    let Some(action) = clock
        .active
        .as_ref()
        .filter(|a| a.command == command && matches!(a.kind, Kind::Normal(_) | Kind::Entry))
    else {
        actor.character.hathor_mut()?.clock = Some(clock);
        return Ok(None);
    };
    queue.retain(|event| !belongs(event, action));
    let action = clock.active.as_mut().unwrap();
    action.kind = if launchable {
        Kind::Branch
    } else {
        Kind::BranchFallback
    };
    action.start = now;
    action.recovery = 0;
    action.jump = 0;
    action.switch = 0;
    action.gate = None;
    action.landing = Some(landing);
    action.groups.clear();
    action.contacts.clear();
    let due = if launchable {
        landing
    } else {
        clock
            .lag
            .resolve_real_deadline(action.owner, now, super::branch::FALLBACK)
            .map_err(|e| format!("Hathor Branch fallback clock: {e}"))?
            .min(landing)
    };
    project(&clock, actor, queue, now, false)?;
    actor.character.hathor_mut()?.clock = Some(clock);
    Ok(Some(due))
}
/// Re-anchor the active Branch at its authored landing and register the one
/// outgoing packet group. Returns the real impact time for that group.
pub(super) struct Landing<'a> {
    pub(super) command: usize,
    pub(super) expected: Kind,
    pub(super) recovery: u64,
    pub(super) jump: u64,
    pub(super) switch: u64,
    pub(super) group: &'a str,
    pub(super) progress: u64,
}
pub(super) fn branch_landing(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    landing: Landing<'_>,
) -> Result<Option<u64>, String> {
    let mut clock = actor
        .character
        .hathor_mut()?
        .clock
        .take()
        .ok_or("Hathor Branch landing clock missing")?;
    let Some(action) = clock
        .active
        .as_mut()
        .filter(|a| a.command == landing.command && a.kind == landing.expected)
    else {
        actor.character.hathor_mut()?.clock = Some(clock);
        return Ok(None);
    };
    action.start = now;
    action.recovery = landing.recovery;
    action.jump = landing.jump;
    action.switch = landing.switch;
    action.gate = None;
    action.landing = None;
    action.groups = BTreeMap::from([(landing.group.to_owned(), landing.progress)]);
    action.contacts.clear();
    let impact = clock
        .lag
        .resolve_real_deadline(action.owner, now, landing.progress)
        .map_err(|e| format!("Hathor Branch landing clock: {e}"))?;
    project(&clock, actor, queue, now, false)?;
    actor.character.hathor_mut()?.clock = Some(clock);
    Ok(Some(impact))
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let state = actor.character.hathor_mut()?;
    if let Some(action) = state.clock.as_mut().and_then(|c| c.active.take()) {
        queue.retain(|e| !belongs(e, &action));
        if (action.kind == Kind::Skill) && !action.cooldown_started {
            actor.ability_cooldowns.insert(
                "GA_Hathor_Skill".into(),
                now.checked_add(3_000_000)
                    .ok_or("Hathor canceled Skill CD overflow")?,
            );
        }
    }
    let state = actor.character.hathor_mut()?;
    state.air = None;
    state.branch = None;
    state.normal_stage = 0;
    state.normal_started_us = None;
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    if super::power::accepted(ctx.request) {
        ctx.record_hypothesis(super::power::MODEL)?;
    }
    let state = ctx.actor.character.hathor_mut()?;
    let stage = crate::characters::normal::at_boundary(
        state.normal_stage,
        state.normal_started_us,
        ctx.time_us,
    );
    let cyclone = (ctx.ability_id == "GA_Hathor_Skill"
        && super::cyclone::accepted(ctx.request)
        && super::burst::active(ctx.actor, ctx.time_us))
    .then(|| combo_stage(ctx.actor, ctx.time_us));
    ctx.buffs
        .remove_owned_buff(super::cyclone::DEF, super::cyclone::SOURCE, ctx.actor_index);
    stop(ctx.actor, ctx.queue, ctx.time_us)?;
    let mut clock = ctx
        .actor
        .character
        .hathor_mut()?
        .clock
        .take()
        .ok_or("Hathor clock missing")?;
    let plan = profile::select(ctx, stage, cyclone)?;
    let branch = super::branch::opened(ctx, &plan)?;
    if plan.kind == Kind::Burst {
        ctx.record_hypothesis(super::burst::MODEL)?;
        ctx.actor.ability_cooldowns.insert(
            "GA_Hathor_UltraSkill".into(),
            ctx.time_us
                .checked_add(15_000_000)
                .ok_or("Hathor Burst CD overflow")?,
        );
    }
    let counter = if let Some(n) = cyclone {
        super::cyclone::commit(ctx, n)?
    } else {
        0
    };
    let hits = &plan.hits;
    if ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(hits[0].0) {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let mut groups = BTreeMap::from([(ctx.scope_primary_aoe_group(hits[0].0)?, hits[0].1)]);
    for &(effect, at) in &hits[1..] {
        groups.insert(ctx.schedule_native_aoe_group(effect, ctx.time_us)?, at);
    }
    let kind = plan.kind;
    let start = if kind == Kind::Plunge {
        ctx.time_us
            .checked_add(ctx.request.plunge_inputs[&ctx.command_index].landing_delay_us)
            .ok_or("Hathor landing overflow")?
    } else {
        ctx.time_us
    };
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start,
        recovery: plan.recovery,
        jump: plan.jump,
        switch: plan.switch,
        gate: branch.as_ref().map(|(gate, _)| *gate),
        landing: None,
        groups,
        contacts: Default::default(),
        kind,
        a4_counter: counter,
        positive: false,
        success: false,
        cooldown_started: false,
    });
    if kind == Kind::Skill {
        ctx.actor.ability_cooldowns.remove("GA_Hathor_Skill");
    }
    let mut notifies = plan.notifies.clone();
    if let Some((gate, _)) = &branch {
        notifies.push((*gate, Boundary::BranchGate));
    }
    schedule_notifies(ctx, notifies)?;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    let state = ctx.actor.character.hathor_mut()?;
    state.normal_stage = match kind {
        Kind::Normal(n) => (n + 1) % 5,
        Kind::Entry => 1,
        _ => 0,
    };
    state.branch = branch.map(|(_, state)| state);
    state.clock = Some(clock);
    if kind == Kind::Plunge {
        super::air::begin(ctx)?;
    }
    Ok(())
}

pub(super) fn air_deadline(
    actor: &ActorRuntime,
    owner: usize,
    origin: u64,
    progress: u64,
) -> Result<u64, String> {
    let crate::characters::RuntimeState::Hathor(s) = &actor.character else {
        return Err("Hathor air clock owner missing".into());
    };
    s.clock
        .as_ref()
        .ok_or("Hathor air clock missing")?
        .lag
        .resolve_real_deadline(owner, origin, progress)
        .map_err(|e| format!("Hathor airborne source clock: {e}"))
}
pub(super) fn land(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let mut clock = actor
        .character
        .hathor_mut()?
        .clock
        .take()
        .ok_or("Hathor landing clock missing")?;
    let action = clock
        .active
        .as_mut()
        .ok_or("Hathor landing action missing")?;
    if action.kind != Kind::Plunge {
        return Err("Hathor landing action changed".into());
    }
    action.start = now;
    project(&clock, actor, queue, now, false)?;
    actor.character.hathor_mut()?.clock = Some(clock);
    Ok(())
}
fn schedule_notifies(
    ctx: &mut CastContext<'_>,
    notifies: Vec<(u64, Boundary)>,
) -> Result<(), String> {
    for (progress, boundary) in notifies {
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        ctx.queue.push(ScheduledGameEvent {
            time_us: ctx.time_us,
            priority: if matches!(boundary, Boundary::CycloneFinish | Boundary::BranchGate) {
                ntesim_core::execution::CausalPriority::ModePause
            } else {
                ntesim_core::execution::CausalPriority::ActionAdmission
            },
            seq: *ctx.sequence,
            data: GameEventData::Content(crate::callbacks::ContentEvent::HathorClock(Event {
                owner: ctx.actor_index,
                command: ctx.command_index,
                boundary,
                progress,
            })),
        });
    }
    Ok(())
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if !accepted(ctx.request) || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    if let Some(action) = ctx
        .actor
        .character
        .hathor_mut()?
        .clock
        .as_mut()
        .and_then(|c| c.active.as_mut())
        .filter(|a| {
            a.command == ctx.command_index
                && ctx
                    .packet_group_id
                    .is_some_and(|g| a.groups.contains_key(g))
        })
    {
        action.positive = true;
    }
    let Some(pause) = profile::pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    let mut clock = ctx
        .actor
        .character
        .hathor_mut()?
        .clock
        .take()
        .ok_or("Hathor contact clock missing")?;
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
                rate,
            } => clock.lag.add_window(HitlagWindow {
                actor_index: ctx.actor_index,
                start_real_us: ctx
                    .time_us
                    .checked_add(delay)
                    .ok_or("Hathor pause onset overflow")?,
                duration_real_us: duration,
                dilated_rate_ppm: rate,
            }),
            profile::Pause::Curve(points, duration) => {
                clock.lag.add_curve_window(HitlagCurveWindow {
                    actor_index: ctx.actor_index,
                    start_real_us: ctx.time_us,
                    duration_real_us: duration,
                    points: points.to_vec(),
                })
            }
        }
        .map_err(|e| format!("Hathor pause: {e}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
        super::burst::project_pending(ctx.actor, ctx.actor_index, ctx.time_us, ctx.queue)?;
    }
    ctx.actor.character.hathor_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn cycle(
    req: &GameScenarioRequest,
    effect: &str,
) -> Result<Option<crate::runtime_api::PacketCycleGrant>, String> {
    if !accepted(req) {
        return Ok(None);
    }
    let Some(raw) =
        ntesim_data::native_damage::cycle_add(ntesim_data::skill_levels::frozen::tables(), effect)?
    else {
        return Ok(None);
    };
    Ok(Some(crate::runtime_api::PacketCycleGrant {
        raw: crate::simulation::ExactFraction::new(raw.numerator(), raw.denominator())?,
        hypothesis: crate::HypothesisId::CombatPacketCycleAccrualV1,
    }))
}
pub(crate) fn interrupt(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    owner: usize,
    event_index: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let before = actor.locked_until_us;
    let burst = actor
        .character
        .hathor_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref())
        .is_some_and(|a| a.kind == Kind::Burst);
    let cooldown = actor.ability_cooldowns.get("GA_Hathor_Skill").copied();
    let command = actor
        .character
        .hathor_mut()?
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
        ctx.state
            .buffs
            .remove_owned_buff(super::cyclone::DEF, super::cyclone::SOURCE, owner);
        stop(actor, ctx.queue, now)?;
        actor.character.hathor_mut()?.riposte_ready_us = None;
        actor.locked_until_us = now;
    }
    let receipt = crate::simulation::GameActorInterruptReceipt {
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
        skill_cooldown_after_us: actor.ability_cooldowns.get("GA_Hathor_Skill").copied(),
        field_cancelled: false,
    };
    if burst {
        super::burst::interrupt(ctx, owner)?;
    }
    if command.is_some() {
        super::burst::finish_cyclone(ctx, owner, "SettlementAndDeath")?;
    }
    Ok(receipt)
}

pub(crate) fn event(
    ctx: &mut crate::runtime_api::CallbackContext<'_>,
    event: Event,
) -> Result<(), String> {
    let actor = &mut ctx.state.actors[event.owner];
    if actor.hp == 0
        || actor
            .character
            .hathor_mut()?
            .clock
            .as_ref()
            .and_then(|c| c.active.as_ref())
            .is_none_or(|a| a.command != event.command)
    {
        return Ok(());
    }
    match event.boundary {
        Boundary::CycloneSuccess => success(ctx, event.owner),
        Boundary::CycloneFinish => {
            let stage = actor
                .character
                .hathor_mut()?
                .clock
                .as_ref()
                .and_then(|c| c.active.as_ref())
                .and_then(|a| a.kind.cyclone());
            if stage == Some(3) {
                success(ctx, event.owner)?;
            }
            super::cyclone::clear(ctx, event.owner)
        }
        Boundary::Emergency => super::burst::enter(ctx, event.owner),
        Boundary::HeldBirth => super::held::birth(ctx, event.owner),
        Boundary::HeldTick => super::held::tick(ctx, event.owner),
        Boundary::BranchGate => super::branch::gate(ctx, event.owner, event.command),
        Boundary::TapPower => super::power::grant(ctx, event.owner, super::power::Cause::Tap),
        Boundary::SkillCooldown => {
            actor.ability_cooldowns.insert(
                "GA_Hathor_Skill".into(),
                ctx.state
                    .current_time_us
                    .checked_add(16_000_000)
                    .ok_or("Hathor Skill CD overflow")?,
            );
            actor
                .character
                .hathor_mut()?
                .clock
                .as_mut()
                .unwrap()
                .active
                .as_mut()
                .unwrap()
                .cooldown_started = true;
            Ok(())
        }
    }
}

fn success(ctx: &mut crate::runtime_api::CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let action = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .clock
        .as_mut()
        .and_then(|c| c.active.as_mut())
        .ok_or("Hathor Cyclone action missing")?;
    if action.positive && !action.success {
        action.success = true;
        super::burst::increment_counter(&mut ctx.state.actors[owner], ctx.state.current_time_us)?;
    }
    Ok(())
}
pub(super) fn combo_gate(actor: &ActorRuntime, now: u64) -> Option<u64> {
    let crate::characters::RuntimeState::Hathor(state) = &actor.character else {
        return None;
    };
    let clock = state.clock.as_ref()?;
    let a = clock.active.as_ref()?;
    let (gate, end) = match a.kind.cyclone()? {
        1 => (1_116_684, 1_166_596),
        2 => (1_433_341, 1_483_364),
        _ => return None,
    };
    let gate = clock
        .lag
        .resolve_real_deadline(a.owner, a.start, gate)
        .ok()?;
    let end = clock
        .lag
        .resolve_real_deadline(a.owner, a.start, end)
        .ok()?;
    (now >= gate && now <= end).then_some(gate)
}
pub(super) fn combo_stage(actor: &ActorRuntime, now: u64) -> u8 {
    if combo_gate(actor, now).is_some()
        && let crate::characters::RuntimeState::Hathor(s) = &actor.character
        && let Some(stage) = s
            .clock
            .as_ref()
            .and_then(|c| c.active.as_ref())
            .and_then(|a| a.kind.cyclone())
    {
        return stage + 1;
    }
    1
}
pub(super) fn cyclone_counter(actor: &ActorRuntime, command: usize) -> u8 {
    if let crate::characters::RuntimeState::Hathor(s) = &actor.character {
        return s
            .clock
            .as_ref()
            .and_then(|c| c.active.as_ref())
            .filter(|a| a.command == command && a.kind.cyclone().is_some())
            .map_or(0, |a| a.a4_counter);
    }
    0
}
pub(super) fn cyclone_finish(actor: &ActorRuntime, now: u64) -> Option<u64> {
    if let crate::characters::RuntimeState::Hathor(s) = &actor.character
        && s.clock
            .as_ref()
            .and_then(|c| c.active.as_ref())
            .is_some_and(|a| a.kind.cyclone().is_some())
        && now < actor.locked_until_us
    {
        return Some(actor.locked_until_us);
    }
    None
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
