//! Generation-owned cinematic pause transitions on the shared runtime clock.
use super::events::{GameEventData, ScheduledGameEvent};
use super::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use ntesim_core::kernel::timing::{ModePauseKind, PauseOwner};

#[derive(Clone, Copy, Debug)]
pub(crate) struct CinematicCast {
    pub(crate) context: PauseOwner,
    release_us: u64,
    completion_us: u64,
}

pub(crate) fn cancel_cinematic(
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    actor: usize,
    command: Option<usize>,
) -> Result<(), String> {
    if let Some(cast) = state.cinematic_casts.get(&actor).copied()
        && command.is_none_or(|command| cast.context.generation == command as u64)
    {
        state
            .mode_clock
            .end_pause(cast.context, state.current_time_us)
            .map_err(|e| e.to_string())?;
        state.cinematic_casts.remove(&actor);
        queue.retain(|event| !matches!(event.data, GameEventData::CinematicRelease { context } if context == cast.context));
    }
    Ok(())
}

pub(crate) fn committed(
    state: &mut GameSimulatorState,
    request: &super::GameScenarioRequest,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    actor: usize,
    command: usize,
    primary_slot: Option<super::metadata::PrimaryActionSlot>,
    ability: &str,
    damage_effect_id: &str,
) -> Result<(), String> {
    // A new legal input may cancel its own old finite cinematic. Buffs and
    // independent producers never inherit the old cast's exemption.
    cancel_cinematic(state, queue, actor, None)?;
    let is_ultimate = primary_slot == Some(super::metadata::PrimaryActionSlot::Ultimate);
    let context = PauseOwner {
        owner: actor,
        generation: command as u64,
    };
    let active_stop = match &state.actors[actor].character {
        crate::characters::RuntimeState::Hotori(owner) => {
            is_ultimate
                || owner.timepiece.mode_active(state.current_time_us)
                || owner.timepiece.rewinding(state.current_time_us)
        }
        _ => false,
    };
    if let Some(ntesim_core::kernel::timing::ClockDomain::Action { context, .. }) =
        state.actors[actor].action_clock
    {
        state.mode_clock.remove_action(context);
    }
    if active_stop {
        queue.allow_active_stop(context);
    }
    if is_ultimate
        && let crate::characters::RuntimeState::Hotori(owner) = &state.actors[actor].character
    {
        queue.bind_mode(owner.timepiece.mode_generation, context);
    }
    state.mode_clock.register_action(context, active_stop);
    state.actors[actor].action_clock = Some(ntesim_core::kernel::timing::ClockDomain::Action {
        context,
        advance_during_active_stop: active_stop,
    });
    let Some(binding) = crate::characters::cinematics::action_binding(
        &request.actors[actor].character_id,
        primary_slot,
        ability,
        damage_effect_id,
    )?
    else {
        return queue.sync(&state.mode_clock);
    };
    let context = PauseOwner {
        owner: actor,
        generation: command as u64,
    };
    let now = state.current_time_us;
    use crate::characters::cinematics::Release;
    let (release_us, provenance) = match binding {
        Release::TimeStop => {
            state.mode_clock.start_pause(context, ModePauseKind::Cinematic, now).map_err(|e| e.to_string())?;
            state.timer_bindings.insert(actor, "Modeled connected pause: World's Tide commitment -> sourced EnterTimeStop -> actual finisher, expiry or defeat".into());
            return queue.sync(&state.mode_clock);
        }
        Release::Notify { progress_us, montage } => (
            now.checked_add(progress_us).ok_or("cinematic release deadline overflow")?,
            format!("Retrieved {montage}: Common.EndAbilitySkill at {progress_us} montage microseconds; selected mode-timer release"),
        ),
        Release::ActionCompletion => (
            {
                let completion = crate::characters::source_choices::completion_deadline(&state.actors[actor], now);
                if completion > now { completion } else {
                    now.checked_add(crate::characters::cinematics::FALLBACK_ACTION_RECOVERY_US)
                        .ok_or("modeled owner completion overflow")?
                }
            },
            "Modeled owner action-completion fallback; no reliable Common.EndAbilitySkill release is bound".into(),
        ),
    };
    if release_us <= now {
        return Err(
            "supported Ultimate requires a positive owner cinematic release binding".into(),
        );
    }
    state
        .mode_clock
        .start_pause(context, ModePauseKind::Cinematic, now)
        .map_err(|e| e.to_string())?;
    state.timer_bindings.insert(actor, provenance);
    state.cinematic_casts.insert(
        actor,
        CinematicCast {
            context,
            release_us,
            completion_us: state.actors[actor].locked_until_us,
        },
    );
    schedule_release(queue, sequence, context, release_us)?;
    queue.sync(&state.mode_clock)
}

fn schedule_release(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    context: PauseOwner,
    time_us: u64,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        seq: *sequence,
        priority: crate::execution::CausalPriority::ModePause,
        data: GameEventData::CinematicRelease { context },
    });
    Ok(())
}

pub(crate) fn release_cinematic(
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    context: PauseOwner,
) -> Result<(), String> {
    let Some(cast) = state
        .cinematic_casts
        .get(&context.owner)
        .filter(|cast| cast.context == context)
        .copied()
    else {
        return Ok(());
    };
    // Positive local montage delay already consumed by the owner moves its end
    // marker too. The owner's unchanged recovery profile remains authoritative.
    let delay = state.actors[context.owner]
        .locked_until_us
        .saturating_sub(cast.completion_us);
    let release = cast
        .release_us
        .checked_add(delay)
        .ok_or("cinematic release projection overflow")?;
    if state.actors[context.owner].hp > 0 && release > state.current_time_us {
        return schedule_release(queue, sequence, context, release);
    }
    cancel_cinematic(
        state,
        queue,
        context.owner,
        Some(context.generation as usize),
    )
}

/// Current checked projection of the live finite cinematic endpoint. Consumers
/// composing an existing local suspension count only its uncovered action tail.
pub(crate) fn cinematic_release_deadline(
    state: &GameSimulatorState,
    owner: usize,
) -> Result<Option<u64>, String> {
    state
        .cinematic_casts
        .get(&owner)
        .map(|cast| {
            cast.release_us
                .checked_add(
                    state.actors[owner]
                        .locked_until_us
                        .saturating_sub(cast.completion_us),
                )
                .ok_or_else(|| "cinematic release projection overflow".into())
        })
        .transpose()
}
