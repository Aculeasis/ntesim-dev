//! Finite ordinary montage; local dilation leaves World timers and resources
//! independent, while shared cinematics/Time Stop apply their declared policy.
use crate::runtime_api::{ActorHitContext, CallbackContext, CastContext};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::timing::{HitlagTracker, HitlagWindow};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) mod controls;
mod held;
mod profile;
pub(crate) mod secondary;
pub(crate) mod source;
pub(crate) mod tenacity;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::MintOrdinaryClockV1;
const SKILL: &str = "GA_Mint019_Skill";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Normal(usize),
    Entry,
    Approach,
    Plunge,
    Riposte,
    Support,
    Dodge(bool),
    Skill,
    Burst,
    Hold,
    End,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Clock {
    lag: HitlagTracker,
    timer_lag: HitlagTracker,
    tenacity_sources: BTreeMap<usize, HitlagWindow>,
    active: Option<Action>,
    input_command: Option<usize>,
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
    source: Option<crate::runtime_api::OwnedAction>,
    release: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Marker {
    Cooldown,
    Begin(u64),
    Wind,
    Spin(usize),
    NaturalEnd,
    Release,
    EndComplete,
    Land,
    TenacityBegin,
    TenacityEnd,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    owner: usize,
    command: usize,
    marker: Marker,
}
impl Event {
    fn progress(&self) -> Option<u64> {
        Some(match self.marker {
            Marker::Cooldown => 182791,
            Marker::Begin(progress) => progress,
            Marker::Wind => 1416667,
            Marker::Spin(index) => held::SPINS[index],
            Marker::NaturalEnd => 5410000,
            Marker::EndComplete => 616639,
            Marker::Land => 0,
            Marker::Release | Marker::TenacityBegin | Marker::TenacityEnd => return None,
        })
    }
}
fn push(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    owner: usize,
    command: usize,
    marker: Marker,
    time_us: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority: if matches!(marker, Marker::TenacityBegin | Marker::TenacityEnd) {
            crate::execution::CausalPriority::ModePause
        } else {
            crate::execution::CausalPriority::BuffTickOrExpiry
        },
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::MintClock(Event {
            owner,
            command,
            marker,
        })),
    });
    Ok(())
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    tenacity::validate(req)?;
    secondary::validate(req)?;
    if !accepted(req) {
        return held::validate(req);
    }
    for (command, action) in req.actions.iter().enumerate() {
        if let GameActionCommand::InvokeAbility {
            actor_index,
            ability_id,
            damage_effect_id,
            ..
        } = action
            && req.actors[*actor_index].character_id == "character:1019"
            && (!matches!(
                ability_id.as_str(),
                "GA_Mint019_Melee" | SKILL | "GA_Mint019_UltraSkill"
            ) || !(Some(damage_effect_id.as_str()) == super::primary_packet(ability_id)
                || (ability_id == "GA_Mint019_Melee"
                    && damage_effect_id == super::plunge::PACKET
                    && req.plunge_inputs.contains_key(&command))))
        {
            return Err(
                "Mint clock requires canonical Normal/Skill/Burst or declared native plunge inputs"
                    .into(),
            );
        }
    }
    held::validate(req)
}
fn due(clock: &Clock, action: &Action, progress: u64) -> Result<u64, String> {
    clock
        .lag
        .resolve_real_deadline(action.owner, action.start, progress)
        .map_err(|e| format!("Mint montage deadline: {e:?}"))
}
fn marker_due(clock: &Clock, action: &Action, event: &Event) -> Result<Option<u64>, String> {
    event
        .progress()
        .map(|progress| {
            if event.marker == Marker::NaturalEnd {
                clock
                    .timer_lag
                    .resolve_real_deadline(action.owner, action.start, progress)
                    .map_err(|e| format!("Mint SkillTriggerEnd deadline: {e:?}"))
            } else {
                due(clock, action, progress)
            }
        })
        .transpose()
}
pub(crate) fn input_lock(
    req: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if !accepted(req) {
        return None;
    }
    let crate::characters::RuntimeState::Mint(state) = &actor.character else {
        return None;
    };
    if ability == "GA_Mint019_Melee" && state.riposte_ready {
        return Some((now, MODEL));
    }
    let Some(clock) = state.clock.as_ref() else {
        return (ability == "jump").then_some((now, MODEL));
    };
    let Some(action) = clock.active.as_ref() else {
        return (ability == "jump").then_some((now, MODEL));
    };
    let progress = secondary::gate(action.kind, ability);
    Some((due(clock, action, progress).ok()?, MODEL))
}
fn belongs(event: &ScheduledGameEvent, action: &Action) -> bool {
    match &event.data {
        GameEventData::HitDelivery { metadata, .. } => metadata
            .binding
            .energy_group_id
            .as_ref()
            .is_some_and(|g| action.groups.contains_key(g)),
        GameEventData::Content(crate::callbacks::ContentEvent::MintClock(e)) => {
            e.owner == action.owner
                && e.command == action.command
                && !matches!(e.marker, Marker::TenacityBegin | Marker::TenacityEnd)
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
            GameEventData::Content(crate::callbacks::ContentEvent::MintClock(e)) => {
                if let Some(time) = marker_due(clock, action, e)? {
                    event.time_us = time;
                }
                None
            }
            _ => None,
        };
        if let Some(progress) = progress {
            event.time_us = due(clock, action, progress)?;
        }
    }
    actor.locked_until_us = if action.kind == Kind::Hold {
        marker_due(
            clock,
            action,
            &Event {
                owner: action.owner,
                command: action.command,
                marker: Marker::NaturalEnd,
            },
        )?
        .ok_or("Mint hold end deadline missing")?
        .checked_add(616639)
        .ok_or("Mint hold recovery overflow")?
    } else {
        due(clock, action, profile::recovery(action.kind))?
    };
    // A declared landing hold can begin after ordinary plunge recovery.
    // Keep default completion with its pending branch; explicit input gates
    // still permit a legal replacement before that threshold.
    if let Some((_, begin)) = held::pending_begin(clock, action)? {
        actor.locked_until_us = actor.locked_until_us.max(begin);
    }
    queue.replace(events);
    Ok(())
}
pub(crate) fn stop(
    actor: &mut ActorRuntime,
    queue: &mut BinaryHeap,
    now: u64,
    stamina: Option<&mut crate::simulation::CombatStaminaReport>,
) -> Result<(), String> {
    if let Some(clock) = &mut actor.character.mint_mut()?.clock {
        clock.input_command = None;
        let Some(action) = clock.active.take() else {
            return Ok(());
        };
        queue.retain(|event| !belongs(event, &action));
        if let Some(stamina) = stamina {
            stamina.suppress_recovery(action.owner, action.command, now, false)?;
        }
        if matches!(action.kind, Kind::Skill) && !action.cooldown_started {
            actor.ability_cooldowns.insert(
                SKILL.into(),
                now.checked_add(3_000_000)
                    .ok_or("Mint cancel cooldown overflow")?,
            );
        }
    }
    Ok(())
}
pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    ctx.record_hypothesis(MODEL)?;
    stop(
        ctx.actor,
        ctx.queue,
        ctx.time_us,
        ctx.stamina.as_deref_mut(),
    )?;
    let (kind, hits, delay) = if let Some(selected) = secondary::selection(ctx)? {
        selected
    } else {
        match ctx.ability_id {
            "GA_Mint019_Melee" => {
                let s = ctx.actor.character.mint_mut()?;
                if ctx.request.approach_inputs.contains(&ctx.command_index) {
                    s.normal_stage = 1;
                    s.normal_started_us = Some(ctx.time_us);
                    (Kind::Approach, secondary::APPROACH_HITS, 0)
                } else if s.entry_normal_pending {
                    s.normal_stage = 1;
                    s.normal_started_us = Some(ctx.time_us);
                    (Kind::Entry, secondary::ENTRY_HITS, 0)
                } else {
                    let stage = crate::characters::normal::after_gap(
                        s.normal_stage,
                        s.normal_started_us,
                        ctx.time_us,
                    );
                    s.normal_stage = (stage + 1) % 5;
                    s.normal_started_us = Some(ctx.time_us);
                    (Kind::Normal(stage), profile::NORMAL[stage], 0)
                }
            }
            SKILL => (Kind::Skill, profile::SKILL, 0),
            "GA_Mint019_UltraSkill" => (Kind::Burst, profile::BURST, 0),
            _ => return Err("Unsupported Mint ordinary clock input".into()),
        }
    };
    ctx.actor.character.mint_mut()?.entry_normal_pending = false;
    ctx.actor.character.mint_mut()?.riposte_ready = false;
    if !matches!(kind, Kind::Normal(_) | Kind::Entry | Kind::Approach) {
        let s = ctx.actor.character.mint_mut()?;
        s.normal_stage = 0;
        s.normal_started_us = None;
    }
    let release = if matches!(
        kind,
        Kind::Normal(_) | Kind::Entry | Kind::Approach | Kind::Plunge
    ) {
        ctx.request
            .hold_inputs
            .get(&ctx.command_index)
            .map(|h| {
                ctx.time_us
                    .checked_add(h.duration_us)
                    .ok_or_else(|| "Mint release overflow".to_string())
            })
            .transpose()?
    } else {
        None
    };
    start(ctx, kind, hits, release, false, delay)?;
    super::skill::effects(ctx)
}

fn start(
    ctx: &mut CastContext<'_>,
    kind: Kind,
    hits: &[profile::Hit],
    release: Option<u64>,
    continuation: bool,
    delay: u64,
) -> Result<(), String> {
    if ctx.targets.len() > 1 {
        ctx.record_hypothesis(super::skill::ALL_IN_RANGE_HYPOTHESIS_ID)?;
    }
    if !continuation && ctx.packet_metadata.map(|p| p.damage_effect_id.as_str()) != Some(hits[0].0)
    {
        ctx.replace_native_primary_packet(hits[0].0)?;
    }
    let first = if continuation {
        ctx.schedule_native_aoe_group(hits[0].0, ctx.time_us)?
    } else {
        ctx.scope_primary_aoe_group(hits[0].0)?
    };
    let mut groups = BTreeMap::from([(first, hits[0].1)]);
    for &(effect, offset) in &hits[1..] {
        groups.insert(ctx.schedule_native_aoe_group(effect, ctx.time_us)?, offset);
    }
    let mut clock = ctx
        .actor
        .character
        .mint_mut()?
        .clock
        .take()
        .unwrap_or_default();
    inherit(&mut clock, ctx.time_us)?;
    clock.active = Some(Action {
        owner: ctx.actor_index,
        command: ctx.command_index,
        start: ctx
            .time_us
            .checked_add(delay)
            .ok_or("Mint section origin overflow")?,
        kind,
        groups,
        contacts: BTreeSet::new(),
        cooldown_started: false,
        source: Some(crate::runtime_api::OwnedAction::capture(ctx)?),
        release,
    });
    if matches!(kind, Kind::Skill) {
        ctx.actor.ability_cooldowns.remove(SKILL);
        push(
            ctx.queue,
            ctx.sequence,
            ctx.actor_index,
            ctx.command_index,
            Marker::Cooldown,
            ctx.time_us,
        )?;
    }
    if kind == Kind::Plunge {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.actor_index,
            ctx.command_index,
            Marker::Land,
            ctx.time_us,
        )?;
    }
    held::schedule(ctx, &clock)?;
    project(&clock, ctx.actor, ctx.queue, ctx.time_us, true)?;
    ctx.actor.character.mint_mut()?.clock = Some(clock);
    Ok(())
}
fn inherit(clock: &mut Clock, now: u64) -> Result<(), String> {
    let mut lag = HitlagTracker::default();
    for window in clock
        .lag
        .windows()
        .iter()
        .filter(|w| w.start_real_us.saturating_add(w.duration_real_us) > now)
    {
        lag.add_window(window.clone())
            .map_err(|e| format!("Mint inherited pause: {e:?}"))?;
    }
    clock.lag = lag;
    clock.timer_lag = clock.lag.clone();
    for window in clock.tenacity_sources.values() {
        clock
            .timer_lag
            .add_window(HitlagWindow {
                dilated_rate_ppm: 0,
                ..window.clone()
            })
            .map_err(|e| format!("Mint inherited timer pause: {e:?}"))?;
    }
    Ok(())
}
pub(crate) fn contact(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    let (pause, rate, delay) = profile::pause(ctx.damage_effect_id);
    if !accepted(ctx.request) || pause == 0 || ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(mut clock) = ctx.actor.character.mint_mut()?.clock.take() else {
        return Ok(());
    };
    let group = ctx
        .packet_group_id
        .ok_or("Mint contact has no causal group")?;
    if clock
        .active
        .as_mut()
        .is_some_and(|a| a.groups.contains_key(group) && a.contacts.insert(group.into()))
    {
        let window = HitlagWindow {
            actor_index: ctx.actor_index,
            start_real_us: ctx
                .time_us
                .checked_add(delay)
                .ok_or("Mint delayed pause overflow")?,
            duration_real_us: pause,
            dilated_rate_ppm: rate,
        };
        clock
            .timer_lag
            .add_window(window.clone())
            .map_err(|e| format!("Mint timer contact pause: {e:?}"))?;
        clock
            .lag
            .add_window(window)
            .map_err(|e| format!("Mint contact pause: {e:?}"))?;
        project(&clock, ctx.actor, ctx.queue, ctx.time_us, false)?;
    }
    ctx.actor.character.mint_mut()?.clock = Some(clock);
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    if matches!(event.marker, Marker::TenacityBegin | Marker::TenacityEnd) {
        return tenacity::event(ctx, event);
    }
    if event.marker != Marker::Cooldown {
        return held::event(ctx, event);
    }
    let actor = &mut ctx.state.actors[event.owner];
    if actor.hp == 0 {
        return Ok(());
    }
    if let Some(clock) = &mut actor.character.mint_mut()?.clock
        && let Some(action) = &mut clock.active
        && action.command == event.command
    {
        action.cooldown_started = true;
        actor.ability_cooldowns.insert(
            SKILL.into(),
            ctx.state
                .current_time_us
                .checked_add(6_000_000)
                .ok_or("Mint Skill cooldown overflow")?,
        );
    }
    Ok(())
}
pub(crate) fn interrupt(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    event: usize,
) -> Result<crate::simulation::GameActorInterruptReceipt, String> {
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    let cooldown = actor.ability_cooldowns.get(SKILL).copied();
    let mut receipt = crate::simulation::GameActorInterruptReceipt {
        event_index: event,
        actor_index: owner,
        time_us: now,
        interrupted_command_index: None,
        cancelled_hit_deliveries: 0,
        recovery_before_us: actor.locked_until_us,
        recovery_after_us: actor.locked_until_us,
        skill_cooldown_before_us: cooldown,
        skill_cooldown_after_us: cooldown,
        field_cancelled: false,
    };
    if actor.hp == 0 || (now >= actor.locked_until_us && !actor.character.mint_mut()?.airborne) {
        return Ok(receipt);
    }
    receipt.interrupted_command_index = actor
        .character
        .mint_mut()?
        .clock
        .as_ref()
        .and_then(|c| c.active.as_ref().map(|a| a.command).or(c.input_command));
    let before = ctx
        .queue
        .iter()
        .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
        .count();
    stop(actor, ctx.queue, now, ctx.state.combat_stamina.as_mut())?;
    receipt.cancelled_hit_deliveries = before
        - ctx
            .queue
            .iter()
            .filter(|e| matches!(e.data, GameEventData::HitDelivery { .. }))
            .count();
    actor.locked_until_us = now;
    let state = actor.character.mint_mut()?;
    state.normal_stage = 0;
    state.normal_started_us = None;
    state.airborne = false;
    state.entry_normal_pending = false;
    state.riposte_ready = false;
    receipt.recovery_after_us = now;
    receipt.skill_cooldown_after_us = actor.ability_cooldowns.get(SKILL).copied();
    Ok(receipt)
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
