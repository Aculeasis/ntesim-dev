use super::{
    ExactFraction, GameEventPayload, GameEventReceipt, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
    state::{GameSimulatorState, TargetStatusDeclaration, apply_target_tagged_status},
};
use crate::execution::CausalPriority;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use ntesim_core::buffs::dispatch::{DecayPolicy, RefreshPolicy};
use std::collections::{BTreeMap, BTreeSet};

use ntesim_contracts::hypotheses::HypothesisId;

pub(crate) const HYPOTHESIS_ID: HypothesisId = HypothesisId::CombatUnbalValueAsTargetGaugeV1;

pub(crate) const BROKEN_STATUS_ID: &str = "ntesim.status.broken";
pub(crate) const BROKEN_STATUS_TAG: &str = "ntesim.status.broken";
const BROKEN_STATUS_SOURCE: &str = "content:break_model";

/// Authored scenario state, without a physical attack or outgoing efficiency.
pub(super) fn admit_authored(
    ctx: &mut super::context::EventContext<'_>,
    command: usize,
    target_index: usize,
    gauge: crate::formula::Ratio,
) -> Result<Option<super::rejection::AdmissionFailure>, String> {
    let after = ExactFraction::from_ratio(gauge)?;
    let target = ctx
        .state
        .targets
        .get(target_index)
        .ok_or("target gauge index out of bounds")?;
    let bar = target
        .break_state
        .as_ref()
        .ok_or("target gauge input requires a modeled gauge")?;
    let message = if target.hp_current == 0 {
        Some(format!("target t{target_index} is defeated"))
    } else if bar.broken_until_us.is_some() {
        Some(format!("target t{target_index} is already Broken"))
    } else if after
        .try_cmp(bar.gauge)
        .map_err(|error| error.to_string())?
        .is_gt()
    {
        Some(format!(
            "target t{target_index} authored gauge cannot increase its live gauge"
        ))
    } else {
        None
    };
    if let Some(message) = message {
        return Ok(Some(super::rejection::AdmissionFailure {
            reason: super::GameAdmissionReason::Other {
                message: message.clone(),
            },
            message,
        }));
    }
    let before = bar.gauge;
    if after.is_zero() {
        ctx.state
            .current_time_us
            .checked_add(bar.broken_duration_us)
            .ok_or("target Broken expiry overflow")?;
        bar.generation
            .checked_add(1)
            .ok_or("target break generation overflow")?;
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;
    ctx.state.targets[target_index]
        .break_state
        .as_mut()
        .unwrap()
        .gauge = after;
    ctx.trace.push(GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ActionAdmission".into(),
        event_kind: "target_break_gauge_authored".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            exact_before: Some(before),
            exact_after: Some(after),
            ..Default::default()
        }),
    });
    // No actor dealt gauge damage: do not manufacture Dustbin or hit callbacks.
    if enter_broken_if_depleted(
        ctx.request,
        ctx.state,
        ctx.queue,
        ctx.sequence,
        ctx.trace,
        ctx.state.active_actor_index,
        target_index,
    )? {
        super::break_damage::schedule(
            ctx.request,
            ctx.state,
            ctx.queue,
            ctx.sequence,
            command,
            target_index,
        )?;
    }
    Ok(None)
}

#[derive(Clone, Copy)]
pub(crate) struct GaugeChange {
    pub unbal_value: ExactFraction,
    pub owner_efficiency_bonus: ExactFraction,
}

impl GaugeChange {
    pub(crate) fn for_packet(
        config: &super::GameActorConfig,
        actor: &super::state::ActorRuntime,
        command_index: usize,
        ability: &str,
        value: Option<ExactFraction>,
    ) -> Result<Option<Self>, String> {
        value
            .map(|unbal_value| {
                Ok(Self {
                    unbal_value,
                    owner_efficiency_bonus: crate::registry::break_efficiency_bonus(
                        config,
                        actor,
                        command_index,
                        ability,
                    )?,
                })
            })
            .transpose()
    }
}

impl From<ExactFraction> for GaugeChange {
    fn from(unbal_value: ExactFraction) -> Self {
        Self {
            unbal_value,
            owner_efficiency_bonus: ExactFraction {
                numerator: 0,
                denominator: 1,
            },
        }
    }
}

pub(crate) fn schedule_unbalance(
    state: &GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    command_index: usize,
    actor_index: usize,
    target_index: usize,
    change: Option<GaugeChange>,
    connected: bool,
    target_defeated: bool,
) -> Result<(), String> {
    if !connected || target_defeated || state.targets[target_index].break_state.is_none() {
        return Ok(());
    }
    let Some(GaugeChange {
        unbal_value,
        owner_efficiency_bonus,
    }) = change.filter(|value| !value.unbal_value.is_zero())
    else {
        return Ok(());
    };
    ExactFraction::new(unbal_value.numerator, unbal_value.denominator)?;
    *sequence = sequence.checked_add(1).ok_or("event sequence overflow")?;
    queue.push(ScheduledGameEvent {
        time_us: state.current_time_us,
        priority: CausalPriority::ReactionOrBreak,
        seq: *sequence,
        data: GameEventData::TargetUnbalance {
            command_index,
            actor_index,
            target_index,
            unbal_value,
            owner_efficiency_bonus,
        },
    });
    Ok(())
}

pub(crate) fn apply_unbalance(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    actor_index: usize,
    target_index: usize,
    change: impl Into<GaugeChange>,
) -> Result<bool, String> {
    let GaugeChange {
        unbal_value,
        owner_efficiency_bonus,
    } = change.into();
    let target = state
        .targets
        .get(target_index)
        .ok_or("break event target index is out of range")?;
    if target.hp_current == 0 {
        return Ok(false);
    }
    let Some(break_state) = target.break_state.as_ref() else {
        return Ok(false);
    };
    if break_state.broken_until_us.is_some() {
        return Ok(false);
    }

    crate::runtime_api::record_hypothesis_id(
        request,
        &mut state.execution_model_assumptions,
        HYPOTHESIS_ID,
    )?;

    // Sample the outgoing multiplier at ReactionOrBreak settlement. The buff
    // service owns stacking, expiry, recipients and living-source eligibility.
    let living_sources = state
        .actors
        .iter()
        .enumerate()
        .filter_map(|(index, actor)| (actor.hp > 0).then_some(index))
        .collect();
    let modifiers = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(
            actor_index,
            state.active_actor_index,
            state.mode_clock.relative_us(),
            &living_sources,
        )
        .map_err(|error| format!("Break efficiency buff query failed: {error:?}"))?;
    let multiplier = ExactFraction::from_ratio(
        request.actors[actor_index]
            .static_stats
            .break_efficiency_multiplier,
    )?
    .add(ExactFraction::from_ratio(
        modifiers
            .get("break_efficiency")
            .copied()
            .unwrap_or(crate::formula::Ratio::zero()),
    )?)?;
    let multiplier = multiplier.add(owner_efficiency_bonus)?;
    let incoming_multiplier = break_state.incoming_efficiency_multiplier.add(
        crate::runtime_api::target_stat_modifier(
            &state.buffs,
            target_index,
            state.mode_clock.relative_us(),
            "incoming_break_efficiency",
        )?,
    )?;
    let unbal_value = unbal_value.mul(multiplier)?.mul(incoming_multiplier)?;
    if unbal_value.is_zero() {
        return Ok(false);
    }
    let break_state = state.targets[target_index].break_state.as_mut().unwrap();
    let before = break_state.gauge;
    let lhs = unbal_value
        .numerator
        .checked_mul(before.denominator)
        .ok_or("break gauge comparison overflow")?;
    let rhs = before
        .numerator
        .checked_mul(unbal_value.denominator)
        .ok_or("break gauge comparison overflow")?;
    let after = if lhs >= rhs {
        ExactFraction::new(0, 1)?
    } else {
        before.sub(unbal_value)?
    };
    break_state.gauge = after;

    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "target_break_gauge_reduced".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            exact_before: Some(before),
            exact_delta: Some(unbal_value),
            exact_after: Some(after),
            exact_efficiency: Some(incoming_multiplier),
            ..Default::default()
        }),
    });

    // A zero starting gauge can enter Broken but has not lost any gauge.
    // Grant before the new generation captures its contributors' live BI.
    if !before.is_zero() {
        crate::weapons::dustbin::on_break_reduced(request, state, actor_index, trace)?;
    }
    enter_broken_if_depleted(
        request,
        state,
        queue,
        sequence,
        trace,
        actor_index,
        target_index,
    )
}

/// Change the target's effective maximum while preserving the amount of gauge
/// already depleted. A live Broken target updates only the cap used at recovery.
pub(crate) fn change_capacity(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    actor_index: usize,
    target_index: usize,
    capacity: ExactFraction,
) -> Result<bool, String> {
    if capacity.is_zero() {
        return Err("target Break capacity must remain positive".into());
    }
    let target = state
        .targets
        .get_mut(target_index)
        .ok_or("Break capacity target index is out of range")?;
    if target.hp_current == 0 {
        return Ok(false);
    }
    let Some(break_state) = target.break_state.as_mut() else {
        return Ok(false);
    };
    let old_capacity = break_state.capacity;
    let old_gauge = break_state.gauge;
    let is_broken = break_state.broken_until_us.is_some();
    let new_gauge = if is_broken {
        old_gauge
    } else {
        shift_gauge_for_capacity(old_gauge, old_capacity, capacity)?
    };
    break_state.capacity = capacity;
    if !is_broken {
        break_state.gauge = new_gauge;
    }
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "target_break_capacity_changed".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            exact_raw: Some(old_capacity),
            exact_capacity: Some(capacity),
            exact_before: Some(old_gauge),
            exact_after: Some(new_gauge),
            ..Default::default()
        }),
    });
    if !is_broken && new_gauge.is_zero() {
        enter_broken_if_depleted(
            request,
            state,
            queue,
            sequence,
            trace,
            actor_index,
            target_index,
        )
    } else {
        Ok(false)
    }
}

/// Apply the target-profile fraction for one admitted Discord generation pair.
/// The loss basis is original capacity even after a temporary cap reduction.
pub(crate) fn apply_discord_gauge_loss(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    actor_index: usize,
    target_index: usize,
    fraction: ExactFraction,
) -> Result<bool, String> {
    let target = state
        .targets
        .get_mut(target_index)
        .ok_or("Discord target index is out of range")?;
    if target.hp_current == 0 {
        return Ok(false);
    }
    let Some(break_state) = target.break_state.as_mut() else {
        return Ok(false);
    };
    let original_capacity = break_state.original_capacity;
    if break_state.broken_until_us.is_some() {
        trace.push(GameEventReceipt {
            time_us: state.current_time_us,
            phase: "ReactionOrBreak".into(),
            event_kind: "target_discord_gauge_ignored_broken".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(actor_index),
            to_actor: None,
            energy_change: None,
            energy_fanout: None,
            data: Some(GameEventPayload {
                target_index: Some(target_index),
                ..Default::default()
            }),
        });
        return Ok(false);
    }
    let incoming_multiplier = break_state.incoming_efficiency_multiplier.add(
        crate::runtime_api::target_stat_modifier(
            &state.buffs,
            target_index,
            state.mode_clock.relative_us(),
            "incoming_break_efficiency",
        )?,
    )?;
    let loss = original_capacity.mul(fraction)?.mul(incoming_multiplier)?;
    let before = break_state.gauge;
    let after = if before.try_cmp(loss).map_err(|error| error.to_string())?
        != std::cmp::Ordering::Greater
    {
        ExactFraction::new(0, 1)?
    } else {
        before.sub(loss)?
    };
    break_state.gauge = after;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "target_discord_gauge_reduced".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            exact_before: Some(before),
            exact_delta: Some(loss),
            exact_capacity: Some(original_capacity),
            exact_after: Some(after),
            ..Default::default()
        }),
    });
    if after.is_zero() {
        enter_broken_if_depleted(
            request,
            state,
            queue,
            sequence,
            trace,
            actor_index,
            target_index,
        )
    } else {
        Ok(false)
    }
}

fn shift_gauge_for_capacity(
    gauge: ExactFraction,
    old_capacity: ExactFraction,
    new_capacity: ExactFraction,
) -> Result<ExactFraction, String> {
    let shifted = if new_capacity
        .try_cmp(old_capacity)
        .map_err(|error| error.to_string())?
        == std::cmp::Ordering::Less
    {
        let reduction = old_capacity.sub(new_capacity)?;
        if gauge
            .try_cmp(reduction)
            .map_err(|error| error.to_string())?
            != std::cmp::Ordering::Greater
        {
            ExactFraction::new(0, 1)?
        } else {
            gauge.sub(reduction)?
        }
    } else {
        gauge.add(new_capacity.sub(old_capacity)?)?
    };
    if shifted
        .try_cmp(new_capacity)
        .map_err(|error| error.to_string())?
        == std::cmp::Ordering::Greater
    {
        Ok(new_capacity)
    } else {
        Ok(shifted)
    }
}

fn enter_broken_if_depleted(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    actor_index: usize,
    target_index: usize,
) -> Result<bool, String> {
    let target = state
        .targets
        .get(target_index)
        .ok_or("Break transition target index is out of range")?;
    if target.hp_current == 0 {
        return Ok(false);
    }
    let Some(break_state) = target.break_state.as_ref() else {
        return Ok(false);
    };
    if !break_state.gauge.is_zero() || break_state.broken_until_us.is_some() {
        return Ok(false);
    }

    let break_state = state.targets[target_index].break_state.as_mut().unwrap();
    let duration_us = break_state.broken_duration_us;
    let expires_at_us = state
        .current_time_us
        .checked_add(duration_us)
        .ok_or("target Broken expiry overflow")?;
    let generation = break_state
        .generation
        .checked_add(1)
        .ok_or("target break generation overflow")?;
    break_state.generation = generation;
    break_state.broken_until_us = Some(expires_at_us);
    break_state.broken_by_actor = Some(actor_index);
    break_state.rose_extension_used = false;

    apply_target_tagged_status(
        &mut state.buffs,
        TargetStatusDeclaration {
            source_actor: actor_index,
            source: BROKEN_STATUS_SOURCE.into(),
            target_index,
            id: BROKEN_STATUS_ID.into(),
            now_us: state.mode_clock.relative_us(),
            duration_us,
            reductions: BTreeMap::new(),
        },
        BTreeSet::from([BROKEN_STATUS_TAG.into()]),
        1,
        RefreshPolicy::Extend {
            max_duration_us: None,
        },
        DecayPolicy::AllAtOnce,
    )?;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "target_broken".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(actor_index),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            expires_at_us: Some(expires_at_us),
            ..Default::default()
        }),
    });

    crate::characters::team::on_target_broken(request, state, target_index, trace)?;
    if expires_at_us <= request.absolute_limit_us {
        *sequence = sequence.checked_add(1).ok_or("event sequence overflow")?;
        queue.push(ScheduledGameEvent {
            time_us: expires_at_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *sequence,
            data: GameEventData::TargetBreakRecovery {
                target_index,
                generation,
            },
        });
    }
    Ok(true)
}

pub(crate) fn extend_broken_for_rose(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    trace: &mut Vec<GameEventReceipt>,
    target_index: usize,
    extension_us: u64,
) -> Result<bool, String> {
    if extension_us == 0 {
        return Err("Rose Broken extension must be positive".into());
    }
    let target = state
        .targets
        .get(target_index)
        .ok_or("Rose Broken extension target index is out of range")?;
    if target.hp_current == 0 {
        return Ok(false);
    }
    let Some(break_state) = target.break_state.as_ref() else {
        return Ok(false);
    };
    let Some(previous_deadline_us) = break_state.broken_until_us else {
        return Ok(false);
    };
    if previous_deadline_us <= state.current_time_us || break_state.rose_extension_used {
        return Ok(false);
    }
    let source_actor = break_state
        .broken_by_actor
        .ok_or("active Broken state is missing its source actor")?;
    let generation = break_state.generation;
    let extended_deadline_us = previous_deadline_us
        .checked_add(extension_us)
        .ok_or("Rose Broken expiry overflow")?;

    apply_target_tagged_status(
        &mut state.buffs,
        TargetStatusDeclaration {
            source_actor,
            source: BROKEN_STATUS_SOURCE.into(),
            target_index,
            id: BROKEN_STATUS_ID.into(),
            now_us: state.mode_clock.relative_us(),
            duration_us: extension_us,
            reductions: BTreeMap::new(),
        },
        BTreeSet::from([BROKEN_STATUS_TAG.into()]),
        1,
        RefreshPolicy::Extend {
            max_duration_us: None,
        },
        DecayPolicy::AllAtOnce,
    )?;

    let break_state = state.targets[target_index]
        .break_state
        .as_mut()
        .expect("active break state was checked above");
    break_state.broken_until_us = Some(extended_deadline_us);
    break_state.rose_extension_used = true;
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: "target_broken_duration_extended".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            cooldown_until_us: Some(previous_deadline_us),
            expires_at_us: Some(extended_deadline_us),
            cause: Some("bohemian_rose".into()),
            ..Default::default()
        }),
    });

    crate::characters::team::on_broken_extended(state, target_index, trace)?;
    if extended_deadline_us <= request.absolute_limit_us {
        *sequence = sequence.checked_add(1).ok_or("event sequence overflow")?;
        queue.push(ScheduledGameEvent {
            time_us: extended_deadline_us,
            priority: CausalPriority::BuffTickOrExpiry,
            seq: *sequence,
            data: GameEventData::TargetBreakRecovery {
                target_index,
                generation,
            },
        });
    }
    Ok(true)
}

pub(crate) fn recover(
    state: &mut GameSimulatorState,
    trace: &mut Vec<GameEventReceipt>,
    target_index: usize,
    generation: u64,
) -> Result<(), String> {
    let target = state
        .targets
        .get_mut(target_index)
        .ok_or("break recovery target index is out of range")?;
    let Some(break_state) = target.break_state.as_mut() else {
        return Ok(());
    };
    if break_state.generation != generation
        || break_state.broken_until_us != Some(state.current_time_us)
    {
        return Ok(());
    }
    break_state.gauge = break_state.capacity;
    break_state.broken_until_us = None;
    break_state.broken_by_actor = None;
    break_state.rose_extension_used = false;
    crate::characters::team::on_target_recovered(state, target_index);
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "BuffTickOrExpiry".into(),
        event_kind: "target_break_recovered".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: None,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            target_index: Some(target_index),
            ..Default::default()
        }),
    });
    Ok(())
}
