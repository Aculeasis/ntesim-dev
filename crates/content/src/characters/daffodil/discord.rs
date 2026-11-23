//! Daffodil's selected Blade Draw B2 Discord break-cap mechanic.

use crate::simulation::state::{
    GameSimulatorState, TargetStatusDeclaration, apply_target_tagged_status,
};
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{ExactFraction, GameEventReceipt, GameScenarioRequest};
use ntesim_core::buffs::dispatch::{DecayPolicy, RefreshPolicy};
use std::collections::BTreeMap;

pub(crate) const STATUS_ID: &str = "ntesim.daffodil.discord_cap";
const STATUS_SOURCE: &str = "character:1054.blade_draw";
const STATUS_TAG: &str = "ntesim.daffodil.discord_cap";
const STACKS_MAX: u8 = 2;
const STACK_REDUCTION_PERCENT: u64 = 10;
const DURATION_US: u64 = 30_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CapState {
    pub(crate) stacks: u8,
    pub(crate) generation: u64,
    pub(crate) expires_at_us: u64,
}

/// Returns true when the cap step itself exhausted the target gauge.
pub(crate) fn on_discord(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    source_actor: usize,
    target_index: usize,
) -> Result<bool, String> {
    let Some(original_capacity) = state
        .targets
        .get(target_index)
        .and_then(|target| target.break_state.as_ref())
        .map(|bar| bar.original_capacity)
    else {
        return Ok(false);
    };
    if state.targets[target_index].hp_current == 0 {
        return Ok(false);
    }
    let daffodil_actor = request
        .actors
        .iter()
        .enumerate()
        .find(|(index, actor)| {
            actor.character_id == "character:1054" && state.actors[*index].hp > 0
        })
        .map(|(index, _)| index);
    let Some(daffodil_actor) = daffodil_actor else {
        return Ok(false);
    };
    crate::runtime_api::record_hypothesis_id(
        request,
        &mut state.execution_model_assumptions,
        crate::HypothesisId::DaffodilDiscordCapReductionV1,
    )?;

    let now_us = state.current_time_us;
    let prior = state.actors[daffodil_actor]
        .character
        .daffodil_mut()?
        .discord_caps
        .get(&target_index)
        .copied();
    let active = prior.filter(|entry| entry.expires_at_us > now_us);
    if prior.is_some() && active.is_none() {
        state.actors[daffodil_actor]
            .character
            .daffodil_mut()?
            .discord_caps
            .remove(&target_index);
        state
            .buffs
            .remove_target_buff(STATUS_ID, STATUS_SOURCE, &target_index.to_string());
        crate::simulation::break_state::change_capacity(
            request,
            state,
            queue,
            sequence,
            trace,
            source_actor,
            target_index,
            original_capacity,
        )?;
    }

    let stacks = active
        .map_or(0, |entry| entry.stacks)
        .saturating_add(1)
        .min(STACKS_MAX);
    let expires_at_us = now_us
        .checked_add(DURATION_US)
        .ok_or("Daffodil Discord cap expiry overflow")?;
    let owner_state = state.actors[daffodil_actor].character.daffodil_mut()?;
    let generation = owner_state.discord_generation;
    owner_state.discord_generation = generation
        .checked_add(1)
        .ok_or("Daffodil Discord generation overflow")?;

    // B2 reduces the current maximum by ten percent of original capacity per
    // stack. The shared gauge transition preserves depletion across cap shifts.
    let remaining_percent = 100 - u64::from(stacks) * STACK_REDUCTION_PERCENT;
    let capacity =
        original_capacity.mul(ExactFraction::new(u128::from(remaining_percent), 100)?)?;
    let broke = crate::simulation::break_state::change_capacity(
        request,
        state,
        queue,
        sequence,
        trace,
        source_actor,
        target_index,
        capacity,
    )?;
    apply_target_tagged_status(
        &mut state.buffs,
        TargetStatusDeclaration {
            source_actor: daffodil_actor,
            source: STATUS_SOURCE.into(),
            target_index,
            id: STATUS_ID.into(),
            now_us: state.mode_clock.relative_us(),
            duration_us: DURATION_US,
            reductions: BTreeMap::new(),
        },
        std::collections::BTreeSet::from([STATUS_TAG.into()]),
        u32::from(STACKS_MAX),
        RefreshPolicy::Reset,
        DecayPolicy::AllAtOnce,
    )?;
    state.actors[daffodil_actor]
        .character
        .daffodil_mut()?
        .discord_caps
        .insert(
            target_index,
            CapState {
                stacks,
                generation,
                expires_at_us,
            },
        );
    if expires_at_us <= request.absolute_limit_us {
        *sequence = sequence.checked_add(1).ok_or("event sequence overflow")?;
        queue.push(crate::simulation::events::ScheduledGameEvent {
            time_us: expires_at_us,
            priority: crate::execution::CausalPriority::BuffTickOrExpiry,
            seq: *sequence,
            data: crate::simulation::events::GameEventData::Content(
                crate::callbacks::ContentEvent::Daffodil(
                    crate::characters::daffodil::clock::Event::DiscordExpiry {
                        owner: daffodil_actor,
                        target_index,
                        generation,
                    },
                ),
            ),
        });
    }
    trace.push(GameEventReceipt {
        time_us: now_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "daffodil_discord_cap_applied".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(daffodil_actor),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            target_index: Some(target_index),
            stacks: Some(u32::from(stacks)),
            max_stacks: Some(u32::from(STACKS_MAX)),
            expires_at_us: Some(expires_at_us),
            ..Default::default()
        }),
    });
    Ok(broke)
}

pub(crate) fn expire(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    owner: usize,
    target_index: usize,
    generation: u64,
) -> Result<(), String> {
    let owner_runtime = state
        .actors
        .get_mut(owner)
        .ok_or("Daffodil Discord expiry owner index is out of range")?
        .character
        .daffodil_mut()?;
    let Some(current) = owner_runtime.discord_caps.get(&target_index).copied() else {
        return Ok(());
    };
    if current.generation != generation || current.expires_at_us != state.current_time_us {
        return Ok(());
    }
    owner_runtime.discord_caps.remove(&target_index);
    state
        .buffs
        .remove_target_buff(STATUS_ID, STATUS_SOURCE, &target_index.to_string());
    let Some(original) = state
        .targets
        .get(target_index)
        .and_then(|target| target.break_state.as_ref())
        .map(|bar| bar.original_capacity)
    else {
        return Ok(());
    };
    crate::simulation::break_state::change_capacity(
        request,
        state,
        queue,
        sequence,
        trace,
        owner,
        target_index,
        original,
    )?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "daffodil_discord_cap_expired".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            target_index: Some(target_index),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) fn clear_target(state: &mut GameSimulatorState, target_index: usize) {
    for actor in &mut state.actors {
        if let Ok(owner) = actor.character.daffodil_mut() {
            owner.discord_caps.remove(&target_index);
        }
    }
    state
        .buffs
        .remove_target_buff(STATUS_ID, STATUS_SOURCE, &target_index.to_string());
}
