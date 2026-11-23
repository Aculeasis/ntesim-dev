//! Paid held Full continuation, retaining actual initial/later input identity.
use super::*;
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::ChizHeldFullV1;
pub(super) const GATE: u64 = 268_039;

pub(crate) fn accepted(request: &GameScenarioRequest) -> bool {
    request
        .model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}

pub(in crate::characters::chiz::pressed) fn pending(hold: &Hold) -> bool {
    hold.finisher.is_some() && !hold.finished
}

pub(crate) fn recovery(
    request: &GameScenarioRequest,
    actor: &ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<(u64, crate::HypothesisId)> {
    if ability != ABILITY || !accepted(request) {
        return None;
    }
    let crate::characters::RuntimeState::Chiz(state) = &actor.character else {
        return None;
    };
    let pressed = state.pressed.as_ref()?;
    let action = pressed
        .active
        .and_then(|command| pressed.actions.get(&command))?;
    let hold = action.hold.as_ref()?;
    if hold.finished || hold.reserved != 2 {
        return None;
    }
    if hold.finisher.is_some() {
        return Some((now, MODEL));
    }
    if hold.cost.is_none() && hold.release_at <= now {
        return None;
    }
    let gate = if hold.cost.is_some() {
        now
    } else if hold.half_started {
        action.start.checked_add(669_036)?
    } else {
        super::super::deadline(action, action.owner, 632_043).ok()?
    };
    Some((gate, MODEL))
}

pub(crate) fn on_cast(ctx: &mut CastContext<'_>) -> Result<bool, String> {
    if !recovery(ctx.request, ctx.actor, ctx.ability_id, ctx.time_us)
        .is_some_and(|(gate, _)| ctx.time_us >= gate)
    {
        return Ok(false);
    }
    if ctx
        .request
        .hold_inputs
        .get(&ctx.command_index)
        .is_some_and(|hold| hold.duration_us > 0)
    {
        return Err(
            "Chiz pending two-stock loop accepts a separate tap for held Full, not a new hold"
                .into(),
        );
    }
    ctx.record_hypothesis(MODEL)?;
    let source = Source {
        command: ctx.command_index,
        coordinate: ctx
            .native_array_index
            .ok_or("Chiz held Full coordinate missing")?,
        action: ctx.action_metadata.clone(),
    };
    let state = ctx
        .actor
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz held Full state missing")?;
    let old = state.active.ok_or("Chiz active hold missing")?;
    state
        .stocks
        .spend(1)
        .map_err(|error| format!("Chiz third stock unavailable: {error:?}"))?;
    super::super::arm_refill(
        ctx.request,
        state,
        ctx.actor_index,
        ctx.time_us,
        ctx.queue,
        ctx.sequence,
    )?;
    let action = state
        .actions
        .get_mut(&old)
        .ok_or("Chiz active hold cast missing")?;
    let hold = action
        .hold
        .as_mut()
        .ok_or("Chiz active hold capture missing")?;
    if hold.finisher.is_some() {
        return Err("Chiz held Full already pending".into());
    }
    hold.finisher = Some(source);
    let due = if hold.cost.is_some() {
        Some(
            ctx.time_us.max(
                action
                    .start
                    .checked_add(GATE)
                    .ok_or("Chiz held Full gate overflow")?,
            ),
        )
    } else {
        None
    };
    ctx.actor.locked_until_us = due.unwrap_or(super::super::deadline(
        action,
        action.owner,
        action.recovery,
    )?);
    ctx.actor
        .ability_cooldowns
        .insert(ABILITY.into(), ctx.time_us);
    ctx.queue.retain(|event|!matches!(&event.data,GameEventData::HitDelivery {command_index,actor_index,..} if *command_index==ctx.command_index && *actor_index==ctx.actor_index));
    if let Some(due) = due {
        enqueue(
            ctx.queue,
            ctx.sequence,
            due,
            super::super::super::events::Event::HeldPhase {
                owner: ctx.actor_index,
                command: old,
                phase: Phase::Finish,
                progress: 0,
            },
        )?;
    }
    Ok(true)
}

pub(super) fn apply(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let state = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz held Full finish state missing")?;
    let activate = state.active == Some(command);
    let Some(action) = state.actions.get_mut(&command) else {
        return Ok(());
    };
    let Some(hold) = &mut action.hold else {
        return Ok(());
    };
    if hold.finished {
        return Ok(());
    }
    let source = hold
        .finisher
        .take()
        .ok_or("Chiz held Full finish has no paid input")?;
    hold.finished = true;
    ctx.queue.retain(|event|!matches!(&event.data,GameEventData::Content(crate::callbacks::ContentEvent::Chiz(super::super::super::events::Event::HeldPhase {owner:source_owner,command:source_command,..})) if *source_owner==owner && *source_command==command));
    start_full(ctx, owner, source, activate)
}

/// A released initial hold still pays its two committed segments, but never
/// enters a loop or cashes unrequested loan credit. A separately paid third
/// input may retain its finite Full gate without fabricating a loop pulse.
pub(super) fn release_before_loop(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<bool, String> {
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz before-loop release state missing")?;
    let Some(action) = state.actions.get_mut(&command) else {
        return Ok(false);
    };
    let Some(hold) = &mut action.hold else {
        return Ok(false);
    };
    if hold.release_at > now || !accepted(ctx.request) {
        return Ok(false);
    }
    if hold.finisher.is_some() {
        action.start = now;
        action.recovery = GATE;
        action.tracker = HitlagTracker::default();
        action.groups.clear();
        enqueue(
            ctx.queue,
            ctx.sequence,
            now.checked_add(GATE)
                .ok_or("Chiz released Full gate overflow")?,
            super::super::super::events::Event::HeldPhase {
                owner,
                command,
                phase: Phase::Finish,
                progress: 0,
            },
        )?;
        project(&mut ctx.state.actors[owner], owner, command, ctx.queue, now)?;
    } else {
        hold.finished = true;
        if state.active == Some(command) {
            state.stage = 0;
            state.combo_ready = None;
            state.active = None;
        }
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "chiz_release_before_loop".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            ..Default::default()
        }),
    });
    Ok(true)
}

pub(super) fn start_full(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    source: Source,
    activate: bool,
) -> Result<(), String> {
    let mut groups = BTreeMap::new();
    for (effect, progress) in [
        ("GE_Player_Chiichan073_Skill3_Damage", 252_173),
        ("GE_Player_Chiichan073_Skill4_Damage", 751_256),
        ("GE_Player_Chiichan073_Skill4_Damage", 865_237),
        ("GE_Player_Chiichan073_Skill4_Damage", 949_538),
    ] {
        groups.insert(source.emit(ctx, owner, effect)?, progress);
    }
    let now = ctx.state.current_time_us;
    let state = ctx.state.actors[owner]
        .character
        .chiz_mut()?
        .pressed
        .as_mut()
        .ok_or("Chiz held Full clock missing")?;
    let previous = state.actions.remove(&source.command);
    let (coin, coin_group, loop_coins, coin_market, coin_force) = previous.map_or_else(
        || (None, String::new(), BTreeMap::new(), None, false),
        |action| {
            (
                action.coin,
                action.coin_group,
                action.loop_coins,
                action.coin_market,
                action.coin_force,
            )
        },
    );
    state.actions.insert(
        source.command,
        super::super::Action {
            kind: super::super::Kind::HeldFull,
            owner,
            start: now,
            recovery: 1_499_861,
            tracker: HitlagTracker::default(),
            groups,
            connected: BTreeSet::new(),
            coin,
            coin_market,
            coin_force,
            coin_group,
            hold: None,
            loop_coins,
        },
    );
    if activate {
        state.active = Some(source.command);
        state.stage = 0;
    }
    project(
        &mut ctx.state.actors[owner],
        owner,
        source.command,
        ctx.queue,
        now,
    )?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "chiz_held_full_start".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(source.command),
            ..Default::default()
        }),
    });
    Ok(())
}
