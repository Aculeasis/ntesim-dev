//! Finite pressed Skill stocks, cast clocks and non-consuming Surplus Coin1.
use crate::runtime_api::{
    ActorHitContext, CallbackContext, CastContext, DamageContext, HitModifiers,
};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{
    ExactFraction, GameActorConfig, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::ActorRuntime,
};
use ntesim_core::kernel::{
    state::Pool,
    timing::{HitlagTracker, HitlagWindow},
};
use num_traits::ToPrimitive;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) mod held;
mod profile;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizPressedSkillV1;
const ABILITY: &str = "GA_Chiichan073_Skill";
const COIN: &str = "GE_Player_Chiichan073_Skill1_CoinDamage";
const REFILL_US: u64 = 3_000_000;

#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(crate) stocks: Pool,
    next_refill: Option<u64>,
    stage: usize,
    combo_ready: Option<u64>,
    active: Option<usize>,
    actions: BTreeMap<usize, Action>,
    jump_cancel: Option<usize>,
}

impl State {
    pub(crate) fn pause_world(&mut self, from: u64, delta: u64) -> Result<(), String> {
        if let Some(next) = &mut self.next_refill {
            crate::simulation::timer_state::future(next, from, delta)?;
        }
        Ok(())
    }
    pub(crate) fn pause_action(
        &mut self,
        owner: usize,
        from: u64,
        delta: u64,
    ) -> Result<(), String> {
        if let Some(ready) = &mut self.combo_ready {
            crate::simulation::timer_state::future(ready, from, delta)?;
        }
        for action in self.actions.values_mut() {
            action
                .tracker
                .add_freeze(owner, from, delta)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Action {
    kind: Kind,
    owner: usize,
    start: u64,
    recovery: u64,
    tracker: HitlagTracker,
    groups: BTreeMap<String, u64>,
    connected: BTreeSet<String>,
    coin: Option<ExactFraction>,
    coin_market: Option<u64>,
    coin_force: bool,
    coin_group: String,
    hold: Option<held::Hold>,
    loop_coins: BTreeMap<String, ExactFraction>,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    No,
    Half,
    Full,
    HeldNo,
    HeldHalf,
    HeldLoop,
    HeldFull,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(crate) fn fixed_crit_encoding(
    effect: &str,
) -> Option<(ExactFraction, ExactFraction, crate::HypothesisId)> {
    matches!(effect, COIN | held::COIN).then_some((
        ExactFraction {
            numerator: 1,
            denominator: 1_000_000,
        },
        ExactFraction {
            numerator: 0,
            denominator: 1,
        },
        if effect == COIN { MODEL } else { held::MODEL },
    ))
}

pub(crate) const STOCKS_CAPACITY: u64 = 3;

pub(crate) fn initial_stocks(initial: Option<crate::formula::Ratio>) -> Result<u64, String> {
    let Some(initial) = initial else {
        return Ok(STOCKS_CAPACITY);
    };
    let value = ExactFraction::from_ratio(initial)?;
    if value.denominator != 1 || value.numerator > u128::from(STOCKS_CAPACITY) {
        return Err("Chiz stocks must be integral in 0..3.".into());
    }
    u64::try_from(value.numerator).map_err(|_| "Chiz initial stocks overflow".into())
}

pub(crate) fn initialize(
    request: &GameScenarioRequest,
    config: &GameActorConfig,
    actor: &mut ActorRuntime,
) -> Result<(), String> {
    if !accepted(request) {
        return Ok(());
    }
    if !super::clock::accepted(request) || !super::finance::accepted(request) {
        return Err("Chiz pressed Skill requires ordinary clock and exact finance models".into());
    }
    let stocks = initial_stocks(config.initial_resources.stocks)?;
    actor.character.chiz_mut()?.pressed = Some(Box::new(State {
        stocks: Pool::new(stocks, 3).map_err(|error| format!("Chiz stocks: {error:?}"))?,
        next_refill: None,
        stage: 0,
        combo_ready: None,
        active: None,
        actions: BTreeMap::new(),
        jump_cancel: None,
    }));
    Ok(())
}

pub(crate) fn initialize_combat(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    let state = ctx.state.actors[owner].character.chiz_mut()?;
    let Some(pressed) = state.pressed.as_mut() else {
        return Ok(());
    };
    arm_refill(
        ctx.request,
        pressed,
        owner,
        ctx.state.current_time_us,
        ctx.queue,
        ctx.sequence,
    )
}

pub(super) fn enqueue(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    time: u64,
    event: super::events::Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us: time,
        priority: if matches!(event, super::events::Event::PressSettlement { .. }) {
            crate::execution::CausalPriority::ProcCallback
        } else {
            crate::execution::CausalPriority::ModePause
        },
        seq: *sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::Chiz(event)),
    });
    Ok(())
}

fn arm_refill(
    request: &GameScenarioRequest,
    state: &mut State,
    owner: usize,
    now: u64,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
) -> Result<(), String> {
    if state.next_refill.is_some() || state.stocks.current() == 3 {
        return Ok(());
    }
    let due = now
        .checked_add(REFILL_US)
        .ok_or("Chiz stock deadline overflow")?;
    state.next_refill = Some(due);
    if due <= request.absolute_limit_us {
        enqueue(
            queue,
            sequence,
            due,
            super::events::Event::StockRefill { owner },
        )?;
    }
    Ok(())
}

pub(crate) fn refill(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz stock state missing")?;
    if state.next_refill != Some(now) {
        return Ok(());
    }
    state.next_refill = None;
    state.stocks.grant(1);
    arm_refill(ctx.request, state, owner, now, ctx.queue, ctx.sequence)?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "chiz_skill_stock_refill".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            stacks: Some(u32::try_from(state.stocks.current()).unwrap_or(u32::MAX)),
            ..Default::default()
        }),
    });
    Ok(())
}

fn deadline(action: &Action, owner: usize, progress: u64) -> Result<u64, String> {
    action
        .tracker
        .resolve_real_deadline(owner, action.start, progress)
        .map_err(|error| format!("Chiz pressed clock: {error:?}"))
}

fn project(
    actor: &mut ActorRuntime,
    owner: usize,
    command: usize,
    queue: &mut BinaryHeap,
    now: u64,
) -> Result<(), String> {
    let state = actor
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz pressed clock missing")?;
    let action = state
        .actions
        .get(&command)
        .ok_or("Chiz pressed cast missing")?;
    let mut events = queue.drain().collect::<Vec<_>>();
    for event in &mut events {
        if event.time_us <= now && now != action.start {
            continue;
        }
        if let GameEventData::Content(crate::callbacks::ContentEvent::Chiz(
            super::events::Event::HeldPhase {
                command: source,
                progress,
                ..
            },
        )) = &event.data
            && *source == command
            && *progress > 0
        {
            event.time_us = deadline(action, owner, *progress)?;
        } else if let GameEventData::HitDelivery { metadata, .. } = &event.data
            && let Some(progress) = metadata
                .binding
                .energy_group_id
                .as_ref()
                .and_then(|group| action.groups.get(group))
        {
            event.time_us = deadline(action, owner, *progress)?;
        }
    }
    let ready = deadline(action, owner, action.recovery)?;
    if state.active == Some(command) {
        state.combo_ready = Some(ready);
        actor.locked_until_us = ready;
    }
    queue.replace(events);
    Ok(())
}

pub(crate) fn reset(actor: &mut ActorRuntime) -> Result<(), String> {
    if let Some(state) = &mut actor.character.chiz_mut()?.pressed {
        state.stage = 0;
        state.combo_ready = None;
        state.active = None;
    }
    Ok(())
}

pub(crate) fn admission_failure(
    request: &GameScenarioRequest,
    command: usize,
    actor: &ActorRuntime,
    ability: &str,
) -> Option<crate::simulation::rejection::AdmissionFailure> {
    if ability != ABILITY {
        return None;
    }
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    if state
        .finance
        .as_ref()
        .and_then(|finance| finance.market.as_ref())
        .and_then(|market| market.session.as_ref())
        .is_some_and(|session| session.closing)
    {
        let message = "Chiz market is closing; no new financial input admitted".to_owned();
        return Some(crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        });
    }
    let pressed = state.pressed.as_ref()?;
    if pressed
        .active
        .and_then(|active| pressed.actions.get(&active))
        .and_then(|action| action.hold.as_ref())
        .is_some_and(held::finish::pending)
    {
        let message = "Chiz held Full finish is already reserved".to_owned();
        return Some(crate::simulation::rejection::AdmissionFailure {
            reason: crate::simulation::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        });
    }
    let required = if request
        .hold_inputs
        .get(&command)
        .is_some_and(|hold| hold.duration_us > 0)
    {
        u64::from(request.hold_inputs[&command].reserved_stocks.unwrap_or(2))
    } else {
        1
    };
    if pressed.stocks.current() >= required {
        return None;
    }
    let message = format!(
        "Chiz Skill has insufficient stocks: required{required}, available{}",
        pressed.stocks.current()
    );
    Some(crate::simulation::rejection::AdmissionFailure {
        reason: crate::simulation::GameAdmissionReason::Other {
            message: message.clone(),
        },
        message,
    })
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    if ctx
        .packet_metadata
        .map(|packet| packet.damage_effect_id.as_str())
        != super::primary_packet(ABILITY)
    {
        return Err("Chiz pressed Skill requires the canonical input; raw sections/coins cannot bypass stocks and finance".into());
    }
    if held::finish::on_cast(ctx)? {
        return Ok(());
    }
    let hold = held::capture(ctx)?;
    if hold.is_some() {
        ctx.record_hypothesis(held::MODEL)?;
    }
    if hold
        .as_ref()
        .is_some_and(|hold| held::reserved_stocks(hold) == 3)
    {
        ctx.record_hypothesis(held::finish::MODEL)?;
    }
    ctx.record_hypothesis(MODEL)?;
    super::clock::stop(ctx.actor, ctx.queue, ctx.time_us)?;
    ctx.actor.character.chiz_mut()?.normal_stage = 0;
    ctx.actor.character.chiz_mut()?.normal_started_us = None;
    let state = ctx
        .actor
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz pressed Skill missing")?;
    if state
        .combo_ready
        .is_some_and(|ready| ctx.time_us.saturating_sub(ready) > 2_000_000)
    {
        state.stage = 0;
    }
    if hold.is_some() {
        state.stage = 0;
    }
    state
        .stocks
        .spend(hold.as_ref().map_or(1, held::reserved_stocks))
        .map_err(|error| format!("Chiz Skill has insufficient stocks: {error:?}"))?;
    arm_refill(
        ctx.request,
        state,
        ctx.actor_index,
        ctx.time_us,
        ctx.queue,
        ctx.sequence,
    )?;
    let stage = state.stage;
    state.stage = (stage + 1) % 3;
    let inherited = super::cancel::inherited_windows(ctx.request, ctx.actor, ctx.time_us);
    if super::cancel::accepted(ctx.request) {
        cancel_active_montage(ctx.actor, ctx.queue);
    }
    let state = ctx.actor.character.chiz_mut()?.pressed.as_mut().unwrap();
    state.active = Some(ctx.command_index);
    emit_stage(ctx, stage, hold, inherited)?;
    super::market::capture(ctx)?;
    // Generic admission's historical cooldown does not replace the stock ledger.
    ctx.actor
        .ability_cooldowns
        .insert(ABILITY.into(), ctx.time_us);
    let capture = ctx
        .time_us
        .checked_add(200_000)
        .ok_or("Chiz settlement deadline overflow")?;
    if capture <= ctx.request.absolute_limit_us {
        enqueue(
            ctx.queue,
            ctx.sequence,
            capture,
            super::events::Event::PressSettlement {
                owner: ctx.actor_index,
                command: ctx.command_index,
            },
        )?;
    }
    held::start(ctx)?;
    project(
        ctx.actor,
        ctx.actor_index,
        ctx.command_index,
        ctx.queue,
        ctx.time_us,
    )
}

fn emit_stage(
    ctx: &mut CastContext<'_>,
    stage: usize,
    hold: Option<held::Hold>,
    inherited: Vec<HitlagWindow>,
) -> Result<(), String> {
    let mut groups = BTreeMap::new();
    let rows = profile::HITS[stage];
    if stage != 0 {
        ctx.replace_native_primary_packet(rows[0].0)?;
    }
    groups.insert(ctx.scope_primary_aoe_group(rows[0].0)?, rows[0].1);
    for (effect, progress) in &rows[1..] {
        groups.insert(
            ctx.schedule_native_aoe_group(effect, ctx.hit_time_us)?,
            *progress,
        );
    }
    let coin_group = ctx.schedule_native_aoe_group(COIN, ctx.hit_time_us)?;
    groups.insert(coin_group.clone(), profile::COIN_PROGRESS[stage]);
    let mut tracker = HitlagTracker::default();
    for window in inherited {
        tracker
            .add_window(window)
            .map_err(|error| format!("Chiz inherited pause: {error:?}"))?;
    }
    let action = Action {
        kind: if hold.is_some() {
            Kind::HeldNo
        } else {
            match stage {
                0 => Kind::No,
                1 => Kind::Half,
                _ => Kind::Full,
            }
        },
        owner: ctx.actor_index,
        start: ctx.time_us,
        recovery: profile::RECOVERY[stage],
        tracker,
        groups,
        connected: BTreeSet::new(),
        coin: None,
        coin_market: None,
        coin_force: false,
        coin_group,
        hold,
        loop_coins: BTreeMap::new(),
    };
    ctx.actor
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .unwrap()
        .actions
        .insert(ctx.command_index, action);
    Ok(())
}

pub(crate) fn settle(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let market = super::market::settle(ctx, owner, command)?;
    let actor = &mut ctx.state.actors[owner];
    let state = actor.character.chiz_mut()?;
    let finance = state
        .finance
        .as_ref()
        .ok_or("Chiz finance missing at press settlement")?;
    let amount = if let Some((amount, _, _)) = market {
        amount
    } else if finance.in_debt() {
        0
    } else {
        finance
            .wallet
            .current()
            .to_integer()
            .to_u128()
            .ok_or("Chiz settlement base exceeds u128")?
    };
    let pressed = state
        .pressed
        .as_mut()
        .ok_or("Chiz pressed settlement missing")?;
    let Some(action) = pressed.actions.get_mut(&command) else {
        return Ok(());
    };
    if action.coin.is_some() {
        return Ok(());
    }
    action.coin = Some(ExactFraction::new(amount, 1)?);
    action.coin_market = market.map(|(_, _, generation)| generation);
    action.coin_force = market.is_some_and(|(_, force, _)| force);
    if action.coin_force {
        let mut events = ctx.queue.drain().collect::<Vec<_>>();
        for event in &mut events {
            if let GameEventData::HitDelivery { metadata, .. } = &mut event.data
                && metadata.binding.energy_group_id.as_ref() == Some(&action.coin_group)
            {
                metadata.packet.fixed_crit_rate = Some(ExactFraction::new(1, 1)?);
                metadata.packet.fixed_crit_rate_provenance = Some(
                    crate::simulation::metadata::MetadataProvenance::AcceptedModelHypothesis {
                        id: super::market::MODEL.into(),
                        description: super::market::MODEL.description().into(),
                    },
                );
            }
        }
        ctx.queue.replace(events);
    }
    if amount == 0 {
        ctx.queue.retain(|event| !matches!(&event.data, GameEventData::HitDelivery {metadata,..} if metadata.binding.energy_group_id.as_ref() == Some(&action.coin_group)));
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ProcCallback".into(),
        event_kind: "chiz_press_settlement".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            exact_raw: Some(ExactFraction::new(amount, 1)?),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn on_hit(ctx: &mut ActorHitContext<'_>) -> Result<(), String> {
    if ctx.damage_effect_id == COIN && ctx.applied_hp_loss > 0 {
        let state = ctx.actor.character.chiz_mut()?;
        if let Some(action) = state
            .pressed
            .as_ref()
            .and_then(|pressed| pressed.actions.get(&ctx.command_index))
            && action.coin_force
            && let Some(market) = state
                .finance
                .as_mut()
                .and_then(|finance| finance.market.as_mut())
            && action.coin_market == Some(market.generation)
        {
            market.first_crit = false;
        }
    }
    let Some((duration, rate)) = profile::pause(ctx.damage_effect_id) else {
        return Ok(());
    };
    if ctx.applied_hp_loss == 0 {
        return Ok(());
    }
    let Some(state) = &mut ctx.actor.character.chiz_mut()?.pressed else {
        return Ok(());
    };
    let Some(action) = state.actions.get_mut(&ctx.command_index) else {
        return Ok(());
    };
    let group = ctx
        .packet_group_id
        .ok_or("Chiz pressed hit has no causal group")?;
    if !action.groups.contains_key(group) || !action.connected.insert(group.into()) {
        return Ok(());
    }
    enqueue(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(16_700)
            .ok_or("Chiz pause onset overflow")?,
        super::events::Event::PressPause {
            owner: ctx.actor_index,
            command: ctx.command_index,
            duration,
            rate,
        },
    )
}

pub(crate) fn pause(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    duration: u64,
    rate: u32,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let actor = &mut ctx.state.actors[owner];
    if actor.hp == 0 {
        return Ok(());
    }
    let state = actor
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz pressed pause state missing")?;
    let Some(action) = state.actions.get_mut(&command) else {
        return Ok(());
    };
    action
        .tracker
        .add_window(HitlagWindow {
            actor_index: owner,
            start_real_us: now,
            duration_real_us: duration,
            dilated_rate_ppm: rate,
        })
        .map_err(|error| format!("Chiz pressed pause: {error:?}"))?;
    project(actor, owner, command, ctx.queue, now)
}

pub(crate) fn damage(
    ctx: &DamageContext<'_>,
    mut modifiers: HitModifiers,
) -> Result<HitModifiers, String> {
    if !matches!(ctx.damage_effect_id, COIN | held::COIN) {
        return Ok(modifiers);
    }
    let crate::characters::RuntimeState::Chiz(state) = &ctx.actor.character else {
        return Err("Chiz Coin1 has no owner state".into());
    };
    let Some(pressed) = &state.pressed else {
        return Err("Chiz Coin1 requires an admitted financial Skill".into());
    };
    let base = pressed
        .actions
        .get(&ctx.command_index)
        .and_then(|action| {
            if ctx.damage_effect_id == COIN {
                action.coin
            } else {
                ctx.packet_group_id
                    .and_then(|group| action.loop_coins.get(group))
                    .copied()
            }
        })
        .ok_or("Chiz Coin1 has no captured proceeds")?;
    modifiers.damage_base_override = Some(base);
    if ctx.config.awakening.has_effect(6) {
        modifiers = modifiers.add_damage(super::parameters::PARAMETERS.coin_damage_bonus)?;
    }
    Ok(modifiers)
}

pub(crate) fn exit(ctx: &mut CallbackContext<'_>, owner: usize) -> Result<(), String> {
    if !accepted(ctx.request) {
        return Ok(());
    }
    reset(&mut ctx.state.actors[owner])?;
    if ctx.state.actors[owner].hp != 0 {
        return Ok(());
    }
    ctx.queue.retain(|event| !matches!(&event.data, GameEventData::HitDelivery {actor_index,ability_id,..} if *actor_index == owner && ability_id == ABILITY)
        && !matches!(&event.data, GameEventData::Content(crate::callbacks::ContentEvent::Chiz(super::events::Event::PressSettlement {owner:source,..} | super::events::Event::PressPause {owner:source,..} | super::events::Event::HeldPhase {owner:source,..})) if *source == owner));
    if let Some(state) = &mut ctx.state.actors[owner].character.chiz_mut()?.pressed {
        state.actions.clear();
    }
    Ok(())
}

pub(super) fn pause_avatar(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    source: Option<usize>,
    duration: u64,
    rate: u32,
) -> Result<(), String> {
    let active = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_ref()
        .and_then(|pressed| pressed.active);
    if let Some(command) = active
        && Some(command) != source
    {
        pause(ctx, owner, command, duration, rate)?;
    }
    Ok(())
}

pub(super) fn live_windows(actor: &ActorRuntime, now: u64) -> Vec<HitlagWindow> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return Vec::new();
    };
    state
        .pressed
        .as_ref()
        .and_then(|pressed| {
            pressed
                .active
                .and_then(|command| pressed.actions.get(&command))
        })
        .map_or_else(Vec::new, |action| {
            action
                .tracker
                .windows()
                .iter()
                .filter(|window| window.start_real_us.saturating_add(window.duration_real_us) > now)
                .cloned()
                .collect()
        })
}

pub(super) fn input_gate(actor: &ActorRuntime, input: &str) -> Option<u64> {
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let pressed = state.pressed.as_ref()?;
    let action = pressed
        .active
        .and_then(|command| pressed.actions.get(&command))?;
    let progress = match (action.kind, input) {
        (Kind::No, ABILITY) => 632_043,
        (Kind::Half, ABILITY) => 669_036,
        (Kind::No, "jump") => 714_457,
        (Kind::Half, "jump") => 659_934,
        (Kind::Full, "jump") => 1_164_855,
        (Kind::HeldFull, "jump") => 1_327_248,
        _ => return None,
    };
    deadline(action, action.owner, progress).ok()
}

fn remove_montage(queue: &mut BinaryHeap, action: &Action) {
    queue.retain(|event|!matches!(&event.data,GameEventData::HitDelivery {metadata,..} if metadata.binding.energy_group_id.as_ref().is_some_and(|group|action.groups.contains_key(group))));
}

fn cancel_active_montage(actor: &ActorRuntime, queue: &mut BinaryHeap) {
    if let crate::characters::RuntimeState::Chiz(state) = &actor.character
        && let Some(pressed) = &state.pressed
        && let Some(action) = pressed
            .active
            .and_then(|command| pressed.actions.get(&command))
        && matches!(action.kind, Kind::No | Kind::Half | Kind::Full)
    {
        remove_montage(queue, action);
    }
}

pub(super) fn mark_jump(actor: &mut ActorRuntime) -> Result<(), String> {
    if let Some(state) = &mut actor.character.chiz_mut()?.pressed {
        state.jump_cancel = state.active;
    }
    Ok(())
}

pub(super) fn commit_jump(actor: &mut ActorRuntime, queue: &mut BinaryHeap) -> Result<(), String> {
    if let Some(state) = &mut actor.character.chiz_mut()?.pressed
        && let Some(command) = state.jump_cancel.take()
        && let Some(action) = state.actions.get(&command)
        && matches!(
            action.kind,
            Kind::No | Kind::Half | Kind::Full | Kind::HeldFull
        )
    {
        remove_montage(queue, action);
    }
    Ok(())
}

pub(super) fn has_active(actor: &ActorRuntime) -> bool {
    matches!(&actor.character,crate::characters::RuntimeState::Chiz(state) if state.pressed.as_ref().is_some_and(|pressed|pressed.active.is_some()))
}

/// Virtual playback owns finite montage controls, while stocks and their World
/// refill timer remain on the living character.
#[derive(Clone, Debug)]
pub(crate) struct FiniteClock {
    stage: usize,
    combo_ready: Option<u64>,
    active: Option<usize>,
    actions: BTreeMap<usize, Action>,
    jump_cancel: Option<usize>,
}
impl State {
    pub(crate) fn take_finite(&mut self) -> FiniteClock {
        FiniteClock {
            stage: std::mem::take(&mut self.stage),
            combo_ready: self.combo_ready.take(),
            active: self.active.take(),
            actions: std::mem::take(&mut self.actions),
            jump_cancel: self.jump_cancel.take(),
        }
    }
    pub(crate) fn restore_finite(&mut self, clock: FiniteClock) {
        self.stage = clock.stage;
        self.combo_ready = clock.combo_ready;
        self.active = clock.active;
        self.actions = clock.actions;
        self.jump_cancel = clock.jump_cancel;
    }
}
impl FiniteClock {
    pub(crate) fn freeze(&mut self, owner: usize, from: u64, delta: u64) -> Result<(), String> {
        if let Some(ready) = &mut self.combo_ready {
            crate::simulation::timer_state::future(ready, from, delta)?;
        }
        for action in self.actions.values_mut() {
            action
                .tracker
                .add_freeze(owner, from, delta)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}
