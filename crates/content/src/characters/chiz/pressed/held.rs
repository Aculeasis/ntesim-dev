//! Two-stock No/held Half continuation, finite loop spending and terminal cash-out.
use super::super::source::Source;
use super::*;
use num_rational::BigRational;
use num_traits::ToPrimitive;
pub(crate) mod finish;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizHeldSkillV1;
pub(super) const COIN: &str = "GE_Player_Chiichan073_Skill2_CoinDamage";
const LOOP_PACKET: &str = "GE_Player_Chiichan073_Skill2_Damage";
const LOOP_END: u64 = 3_733_333;
const EXIT: u64 = 449_393;
const PULSES: [u64; 16] = [
    200_000, 433_334, 666_667, 900_000, 1_133_334, 1_366_667, 1_600_001, 1_833_334, 2_066_667,
    2_300_001, 2_533_333, 2_766_667, 3_000_001, 3_233_333, 3_466_667, 3_683_484,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Half,
    Loop,
    Tick,
    End,
    Finish,
}

#[derive(Clone, Debug)]
pub(super) struct Hold {
    release_at: u64,
    source: Source,
    cost: Option<BigRational>,
    finished: bool,
    reserved: u8,
    natural_end: Option<u64>,
    finisher: Option<Source>,
    half_started: bool,
    switch_ready: Option<u64>,
}

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

/// The committed afterimage owns Half/Loop clocks after the initial avatar
/// segment. Those clocks must not extend that segment's switch recovery.
pub(crate) fn switch_recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    input: &str,
) -> Option<(u64, crate::HypothesisId)> {
    if input != "switch_actor" || !accepted(request) {
        return None;
    }
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let pressed = state.pressed.as_ref()?;
    let action = pressed.actions.get(&pressed.active?)?;
    let hold = action.hold.as_ref()?;
    if hold.finished || !matches!(action.kind, Kind::HeldNo | Kind::HeldHalf | Kind::HeldLoop) {
        return None;
    }
    let ready = hold
        .switch_ready
        .or_else(|| deadline(action, action.owner, action.recovery).ok())?;
    Some((ready, MODEL))
}

pub(crate) fn validate_stock_selection(
    input: &crate::simulation::GameHoldInput,
) -> Result<(), String> {
    if input
        .reserved_stocks
        .is_some_and(|stocks| !matches!(stocks, 2 | 3))
        || input.reserved_stocks.is_some() && input.duration_us == 0
    {
        return Err("Chiz held Skill reserves exactly2 or3 stocks on a positive hold".into());
    }
    Ok(())
}

pub(crate) fn validate(
    request: &GameScenarioRequest,
    packets: &[Option<crate::simulation::metadata::ActionPacketMetadata>],
) -> Result<(), String> {
    if finish::accepted(request) && !accepted(request) {
        return Err("Chiz held Full requires held_skill_v1".into());
    }
    if accepted(request) && !super::accepted(request) {
        return Err("Chiz held Skill requires pressed Skill model".into());
    }
    for (&command, input) in &request.hold_inputs {
        let Some(crate::simulation::GameActionCommand::InvokeAbility {
            actor_index,
            timestamp_us,
            ..
        }) = request.actions.get(command)
        else {
            continue;
        };
        if request.actors[*actor_index].character_id != "character:1073" {
            continue;
        }
        if let Some(stocks) = input.reserved_stocks {
            if !matches!(stocks, 2 | 3) || input.duration_us == 0 {
                return Err(
                    "Chiz held Skill reserves exactly2 or3 stocks on a positive hold".into(),
                );
            }
            if stocks == 3 && !finish::accepted(request) {
                return Err("Chiz three-stock reservation requires held_full_v1".into());
            }
        }
        let packet = packets[command]
            .as_ref()
            .ok_or("Chiz held Skill metadata missing")?;
        if packet.action.ability_id == "GA_Chiichan073_Melee"
            && super::super::barren::accepted(request)
            && input.reserved_stocks.is_none()
        {
            continue;
        }
        if packet.action.ability_id != ABILITY
            || packet.packet.damage_effect_id != super::super::primary_packet(ABILITY).unwrap()
        {
            return Err("Chiz hold requires canonical Skill input".into());
        }
        if input.moving {
            return Err("Chiz held Skill has no moving input binding".into());
        }
        if input.duration_us == 0 {
            continue;
        }
        if !accepted(request) {
            return Err("Chiz held Skill requires accepted held_skill_v1".into());
        }
        if input.duration_us < 2_313_136 && !finish::accepted(request) {
            return Err(
                "Chiz admitted two-stock hold requires stationary duration at least2313136us"
                    .into(),
            );
        }
        timestamp_us
            .checked_add(input.duration_us)
            .and_then(|time| time.checked_add(EXIT))
            .ok_or("Chiz hold deadline overflow")?;
    }
    Ok(())
}

pub(super) fn capture(ctx: &CastContext<'_>) -> Result<Option<Hold>, String> {
    let Some(input) = ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .filter(|input| input.duration_us > 0)
    else {
        return Ok(None);
    };
    if !accepted(ctx.request) {
        return Err("Chiz held Skill model missing at admission".into());
    }
    Ok(Some(Hold {
        release_at: ctx
            .time_us
            .checked_add(input.duration_us)
            .ok_or("Chiz hold release overflow")?,
        source: Source {
            command: ctx.command_index,
            coordinate: ctx
                .native_array_index
                .ok_or("Chiz held Skill coordinate missing")?,
            action: ctx.action_metadata.clone(),
        },
        cost: None,
        finished: false,
        reserved: input.reserved_stocks.unwrap_or(2),
        natural_end: None,
        finisher: None,
        half_started: false,
        switch_ready: None,
    }))
}

pub(super) fn reserved_stocks(hold: &Hold) -> u64 {
    u64::from(hold.reserved)
}

pub(super) fn start(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let state = ctx
        .actor
        .character
        .chiz_mut()?
        .pressed
        .as_ref()
        .ok_or("Chiz held state missing")?;
    if state.actions[&ctx.command_index].hold.is_none() {
        return Ok(());
    }
    enqueue(
        ctx.queue,
        ctx.sequence,
        ctx.time_us
            .checked_add(919_881)
            .ok_or("Chiz held Half deadline overflow")?,
        super::super::events::Event::HeldPhase {
            owner: ctx.actor_index,
            command: ctx.command_index,
            phase: Phase::Half,
            progress: 919_881,
        },
    )
}

pub(crate) fn apply(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    phase: Phase,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    match phase {
        Phase::Half => half(ctx, owner, command),
        Phase::Loop => loop_entry(ctx, owner, command),
        Phase::Tick => tick(ctx, owner, command, false),
        Phase::End => tick(ctx, owner, command, true),
        Phase::Finish => finish::apply(ctx, owner, command),
    }
}

fn half(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    let source = {
        let state = ctx.state.actors[owner]
            .character
            .chiz_mut()?
            .pressed
            .as_ref()
            .ok_or("Chiz held Half state missing")?;
        let Some(hold) = state
            .actions
            .get(&command)
            .and_then(|action| action.hold.as_ref())
        else {
            return Ok(());
        };
        hold.source.clone()
    };
    let mut groups = BTreeMap::new();
    for progress in [599_893, 966_861] {
        groups.insert(
            source.emit(ctx, owner, "GE_Player_Chiichan073_Skill2_2Damage")?,
            progress,
        );
    }
    let now = ctx.state.current_time_us;
    let action = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .unwrap()
        .actions
        .get_mut(&command)
        .unwrap();
    let switch_ready = deadline(action, owner, action.recovery)?;
    action.hold.as_mut().unwrap().switch_ready = Some(switch_ready);
    action.start = now;
    action.kind = super::Kind::HeldHalf;
    action.recovery = 1_049_910;
    action.tracker = HitlagTracker::default();
    action.groups = groups;
    action.connected.clear();
    action.hold.as_mut().unwrap().half_started = true;
    enqueue(
        ctx.queue,
        ctx.sequence,
        now.checked_add(1_033_255)
            .ok_or("Chiz loop entry overflow")?,
        super::super::events::Event::HeldPhase {
            owner,
            command,
            phase: Phase::Loop,
            progress: 1_033_255,
        },
    )?;
    project(&mut ctx.state.actors[owner], owner, command, ctx.queue, now)
}

fn loop_entry(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    if finish::release_before_loop(ctx, owner, command)? {
        return Ok(());
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner].character.chiz_mut()?;
    let available = state
        .finance
        .as_ref()
        .ok_or("Chiz loop finance missing")?
        .available();
    let pressed = state.pressed.as_mut().ok_or("Chiz held loop missing")?;
    let Some(action) = pressed.actions.get_mut(&command) else {
        return Ok(());
    };
    let hold = action.hold.as_mut().ok_or("Chiz held capture missing")?;
    if hold.cost.is_some() {
        return Ok(());
    }
    hold.cost = Some(available / BigRational::from_integer(16.into()));
    let natural_end = now.checked_add(LOOP_END).ok_or("Chiz loop end overflow")?;
    hold.natural_end = Some(natural_end);
    let end = hold.release_at.min(natural_end).max(now);
    action.start = now;
    action.kind = super::Kind::HeldLoop;
    action.recovery = end - now
        + if hold.reserved == 3 && end == natural_end {
            1_739_861
        } else {
            EXIT
        };
    action.tracker = HitlagTracker::default();
    action.groups.clear();
    let finishing = hold.finisher.is_some();
    if finishing {
        action.recovery = finish::GATE;
    }
    for offset in PULSES {
        let due = now.checked_add(offset).ok_or("Chiz loop pulse overflow")?;
        if due <= end {
            enqueue(
                ctx.queue,
                ctx.sequence,
                due,
                super::super::events::Event::HeldPhase {
                    owner,
                    command,
                    phase: Phase::Tick,
                    progress: 0,
                },
            )?;
        }
    }
    enqueue(
        ctx.queue,
        ctx.sequence,
        end,
        super::super::events::Event::HeldPhase {
            owner,
            command,
            phase: Phase::End,
            progress: 0,
        },
    )?;
    if finishing {
        enqueue(
            ctx.queue,
            ctx.sequence,
            now.checked_add(finish::GATE)
                .ok_or("Chiz buffered Full deadline overflow")?,
            super::super::events::Event::HeldPhase {
                owner,
                command,
                phase: Phase::Finish,
                progress: 0,
            },
        )?;
    }
    project(&mut ctx.state.actors[owner], owner, command, ctx.queue, now)
}

fn tick(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    terminal: bool,
) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.chiz_mut()?;
    let pressed = state
        .pressed
        .as_mut()
        .ok_or("Chiz loop tick state missing")?;
    let Some(action) = pressed.actions.get_mut(&command) else {
        return Ok(());
    };
    let hold = action
        .hold
        .as_mut()
        .ok_or("Chiz loop tick capture missing")?;
    if hold.finished {
        return Ok(());
    }
    let activate = pressed.active == Some(command);
    let finance = state
        .finance
        .as_mut()
        .ok_or("Chiz loop tick finance missing")?;
    let maximum = if terminal {
        finance.available()
    } else {
        hold.cost.clone().ok_or("Chiz loop cost not captured")?
    };
    let actual = finance.spend(maximum)?;
    if terminal {
        hold.finished = true;
        if pressed.active == Some(command) {
            pressed.stage = 0;
            pressed.combo_ready = None;
            pressed.active = None;
        }
    }
    let source = hold.source.clone();
    let full =
        terminal && hold.reserved == 3 && hold.natural_end == Some(ctx.state.current_time_us);
    super::super::finance::refresh(ctx, owner)?;
    if !terminal {
        source.emit(ctx, owner, LOOP_PACKET)?;
    }
    let base = actual
        .to_integer()
        .to_u128()
        .ok_or("Chiz Coin2 spent base exceeds u128")?
        .checked_mul(3)
        .ok_or("Chiz Coin2 base overflow")?;
    if base > 0 {
        let group = source.emit(ctx, owner, COIN)?;
        ctx.state.actors[owner]
            .character
            .chiz_mut()?
            .pressed
            .as_mut()
            .unwrap()
            .actions
            .get_mut(&command)
            .unwrap()
            .loop_coins
            .insert(group, ExactFraction::new(base, 1)?);
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ModePause".into(),
        event_kind: if terminal {
            "chiz_held_terminal_spend"
        } else {
            "chiz_held_loop_spend"
        }
        .into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            wide_amount: Some(ntesim_core::kernel::state::WideFraction::from(&actual)),
            exact_raw: Some(ExactFraction::new(base, 1)?),
            ..Default::default()
        }),
    });
    if full {
        finish::start_full(ctx, owner, source, activate)?;
    }
    Ok(())
}
