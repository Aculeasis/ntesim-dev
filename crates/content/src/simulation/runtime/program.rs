use crate::simulation::events::GameEventData;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use crate::simulation::{GameEventPayload, GameEventReceipt};
pub(crate) fn validate_live_program(
    program: &crate::simulation::GameScenarioProgramRequest,
) -> Result<(), String> {
    for branch in &program.branches {
        for wait in &program.energy_waits {
            if branch.after_action_index < wait.after_action_index
                && branch.timestamp_us > wait.timestamp_us
            {
                return Err(
                    "live branch preceding an energy wait must have an earlier or equal timestamp"
                        .into(),
                );
            }
            if wait.after_action_index < branch.after_action_index
                && wait.timestamp_us > branch.timestamp_us
            {
                return Err(
                    "live energy wait preceding a branch must have an earlier or equal timestamp"
                        .into(),
                );
            }
        }
    }
    let mut previous_branch: Option<&crate::simulation::GameLiveBranch> = None;
    for branch in &program.branches {
        if branch.timestamp_us > program.request.absolute_limit_us {
            return Err("live branch timestamp exceeds the scenario horizon".into());
        }
        if branch.after_action_index > program.request.actions.len() {
            return Err("live branch action boundary exceeds the action list".into());
        }
        if let crate::simulation::GameLiveBranchCondition::TargetHpBelow { target_index, .. } =
            &branch.condition
            && *target_index >= program.request.targets.len()
        {
            return Err("live HP branch references an unknown target".into());
        }
        if branch.wait_if_below_us == 0 || branch.wait_otherwise_us == 0 {
            return Err("live branch waits must both advance time".into());
        }
        if branch.source_line == 0 || branch.source_column == 0 {
            return Err("live branch is missing its source location".into());
        }
        if previous_branch.is_some_and(|previous| {
            branch.timestamp_us < previous.timestamp_us
                || branch.after_action_index < previous.after_action_index
        }) {
            return Err("live branches must follow their timestamps and action boundaries".into());
        }
        if program.request.actions[..branch.after_action_index]
            .iter()
            .any(|action| action.timestamp_us() > branch.timestamp_us)
        {
            return Err("live branch cannot precede an action in its prefix".into());
        }
        let baseline_end = branch
            .timestamp_us
            .checked_add(branch.wait_if_below_us)
            .ok_or("live branch wait exceeds the supported clock range")?;
        if program.request.actions[branch.after_action_index..]
            .iter()
            .any(|action| action.timestamp_us() < baseline_end)
        {
            return Err("live branch suffix must begin after its baseline wait".into());
        }
        previous_branch = Some(branch);
    }
    let mut previous_gate: Option<&crate::simulation::GameLiveEnergyWait> = None;
    for wait in &program.energy_waits {
        if wait.timestamp_us > program.request.absolute_limit_us {
            return Err("live energy wait timestamp exceeds the scenario horizon".into());
        }
        if wait.after_action_index > program.request.actions.len() {
            return Err("live energy wait boundary exceeds the action list".into());
        }
        let actor = program
            .request
            .actors
            .get(wait.actor_index)
            .ok_or("live energy wait references an unknown actor")?;
        if wait.energy_at_least > actor.energy_capacity {
            return Err("live energy wait exceeds the actor's declared capacity".into());
        }
        if wait.source_line == 0 || wait.source_column == 0 {
            return Err("live energy wait is missing its source location".into());
        }
        if previous_gate.is_some_and(|previous| {
            wait.timestamp_us < previous.timestamp_us
                || wait.after_action_index < previous.after_action_index
        }) {
            return Err(
                "live energy waits must follow their timestamps and action boundaries".into(),
            );
        }
        if program.request.actions[..wait.after_action_index]
            .iter()
            .any(|action| action.timestamp_us() > wait.timestamp_us)
        {
            return Err("live energy wait cannot precede an action in its prefix".into());
        }
        if program.request.actions[wait.after_action_index..]
            .iter()
            .any(|action| action.timestamp_us() < wait.timestamp_us)
        {
            return Err("live energy wait suffix must begin at or after the wait".into());
        }
        previous_gate = Some(wait);
    }
    for branch in &program.branches {
        for wait in &program.energy_waits {
            if branch.after_action_index < wait.after_action_index
                && branch.timestamp_us > wait.timestamp_us
            {
                return Err(
                    "live branch preceding an energy wait must have an earlier or equal timestamp"
                        .into(),
                );
            }
            if wait.after_action_index < branch.after_action_index
                && wait.timestamp_us > branch.timestamp_us
            {
                return Err(
                    "live energy wait preceding a branch must have an earlier or equal timestamp"
                        .into(),
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn shift_queued_actions(
    queue: &mut BinaryHeap,
    branches: &[crate::simulation::GameLiveBranch],
    energy_waits: &[crate::simulation::GameLiveEnergyWait],
    parked_actions: &mut [crate::simulation::events::ScheduledGameEvent],
    pending_energy_wait: Option<&mut PendingLiveEnergyWait>,
    after_action_index: usize,
    selected_wait_us: u64,
    baseline_wait_us: u64,
) -> Result<(), String> {
    let delta = i128::from(selected_wait_us) - i128::from(baseline_wait_us);
    if delta == 0 {
        return Ok(());
    }
    let mut pending = Vec::with_capacity(queue.len());
    for mut event in queue.drain() {
        let action_index = match &event.data {
            GameEventData::ActionAdmission { command_index }
            | GameEventData::SwitchActor { command_index, .. } => Some(*command_index),
            GameEventData::LiveBranch { branch_index } => {
                branches.get(*branch_index).map(|b| b.after_action_index)
            }
            GameEventData::LiveEnergyWait { wait_index } => {
                energy_waits.get(*wait_index).map(|w| w.after_action_index)
            }
            _ => None,
        };
        if action_index.is_some_and(|index| index >= after_action_index) {
            let shifted = i128::from(event.time_us) + delta;
            event.time_us = u64::try_from(shifted)
                .map_err(|_| "live branch shifted an action outside the supported clock range")?;
        }
        pending.push(event);
    }
    queue.extend(pending);

    for event in parked_actions.iter_mut() {
        let action_index = match &event.data {
            GameEventData::ActionAdmission { command_index }
            | GameEventData::SwitchActor { command_index, .. } => Some(*command_index),
            GameEventData::LiveBranch { branch_index } => {
                branches.get(*branch_index).map(|b| b.after_action_index)
            }
            _ => None,
        };
        if action_index.is_some_and(|index| index >= after_action_index) {
            let shifted = i128::from(event.time_us) + delta;
            event.time_us = u64::try_from(shifted)
                .map_err(|_| "live branch shifted an action outside the supported clock range")?;
        }
    }

    if let Some(wait) = pending_energy_wait {
        for event in wait.parked_actions.iter_mut() {
            let action_index = match &event.data {
                GameEventData::ActionAdmission { command_index }
                | GameEventData::SwitchActor { command_index, .. } => Some(*command_index),
                GameEventData::LiveBranch { branch_index } => {
                    branches.get(*branch_index).map(|b| b.after_action_index)
                }
                _ => None,
            };
            if action_index.is_some_and(|index| index >= after_action_index) {
                let shifted = i128::from(event.time_us) + delta;
                event.time_us = u64::try_from(shifted).map_err(
                    |_| "live branch shifted an action outside the supported clock range",
                )?;
            }
        }
    }

    Ok(())
}

pub(crate) fn process_live_branch(
    state: &GameSimulatorState,
    queue: &mut BinaryHeap,
    branches: &[crate::simulation::GameLiveBranch],
    energy_waits: &[crate::simulation::GameLiveEnergyWait],
    parked_actions: &mut [crate::simulation::events::ScheduledGameEvent],
    pending_energy_wait: Option<&mut PendingLiveEnergyWait>,
    branch_index: usize,
    receipts: &mut Vec<crate::simulation::GameLiveBranchReceipt>,
    trace: &mut Vec<GameEventReceipt>,
) -> Result<(), String> {
    let branch = &branches[branch_index];
    let observed_value = match &branch.condition {
        crate::simulation::GameLiveBranchCondition::TargetHpBelow { target_index, .. } => {
            state.targets[*target_index].hp_current
        }
        crate::simulation::GameLiveBranchCondition::ActiveActorEnergyBelow { .. } => {
            state.actors[state.active_actor_index].energy.current()
        }
    };
    let threshold = match &branch.condition {
        crate::simulation::GameLiveBranchCondition::TargetHpBelow { threshold, .. }
        | crate::simulation::GameLiveBranchCondition::ActiveActorEnergyBelow { threshold } => {
            *threshold
        }
    };
    let selected_wait_us = if observed_value < threshold {
        branch.wait_if_below_us
    } else {
        branch.wait_otherwise_us
    };
    state
        .current_time_us
        .checked_add(selected_wait_us)
        .ok_or("live branch wait exceeds the supported clock range")?;
    shift_queued_actions(
        queue,
        branches,
        energy_waits,
        parked_actions,
        pending_energy_wait,
        branch.after_action_index,
        selected_wait_us,
        branch.wait_if_below_us,
    )?;
    receipts.push(crate::simulation::GameLiveBranchReceipt {
        time_us: state.current_time_us,
        condition: branch.condition.clone(),
        observer_actor_index: state.active_actor_index,
        observed_value,
        selected_wait_us,
        baseline_wait_us: branch.wait_if_below_us,
        after_action_index: branch.after_action_index,
        source_line: branch.source_line,
        source_column: branch.source_column,
    });
    let (condition, target_index, observer_actor) = match &branch.condition {
        crate::simulation::GameLiveBranchCondition::TargetHpBelow { target_index, .. } => {
            ("target_hp_below", Some(*target_index), None)
        }
        crate::simulation::GameLiveBranchCondition::ActiveActorEnergyBelow { .. } => (
            "active_actor_energy_below",
            None,
            Some(state.active_actor_index),
        ),
    };
    trace.push(GameEventReceipt {
        time_us: state.current_time_us,
        phase: "Observation".into(),
        event_kind: "live_branch".into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: observer_actor,
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(GameEventPayload {
            condition: Some(condition.into()),
            target_index,
            amount: Some(observed_value),
            threshold: Some(threshold),
            delay_us: Some(selected_wait_us),
            ..Default::default()
        }),
    });
    Ok(())
}

pub(crate) struct PendingLiveEnergyWait {
    pub(crate) instruction: crate::simulation::GameLiveEnergyWait,
    pub(crate) wait_index: usize,
    pub(crate) started_time_us: u64,
    pub(crate) receipt_index: usize,
    pub(crate) parked_actions: Vec<crate::simulation::events::ScheduledGameEvent>,
    pub(crate) resume_queued: bool,
}

pub(crate) fn scheduled_action_index(
    event: &crate::simulation::events::ScheduledGameEvent,
    branches: &[crate::simulation::GameLiveBranch],
) -> Option<usize> {
    match &event.data {
        GameEventData::ActionAdmission { command_index }
        | GameEventData::SwitchActor { command_index, .. } => Some(*command_index),
        GameEventData::LiveBranch { branch_index } => {
            branches.get(*branch_index).map(|b| b.after_action_index)
        }
        _ => None,
    }
}

pub(crate) fn parked_energy_action_count(
    initially_parked: &[crate::simulation::events::ScheduledGameEvent],
    pending: Option<&PendingLiveEnergyWait>,
) -> usize {
    initially_parked.len() + pending.map_or(0, |wait| wait.parked_actions.len())
}

impl super::execution::PreparedExecution {
    pub(super) fn wait_end(&self, index: usize, time: i128, duration_us: u64) -> i128 {
        // The scheduled compiler emits the branch's baseline Wait immediately
        // before its observation boundary. The selected arm replaces that wait.
        let duration = self
            .live_branch_receipts
            .iter()
            .find(|branch| {
                branch.after_action_index == index + 1
                    && i128::from(branch.time_us) == time
                    && branch.baseline_wait_us == duration_us
            })
            .map_or(duration_us, |branch| branch.selected_wait_us);
        time + i128::from(duration)
    }

    /// Resolve native Wait annotations using the same reached shifts as queued
    /// inputs. Source exports already contain their actual timestamps.
    pub(super) fn action_times<'a>(
        &self,
        request: &'a crate::simulation::GameScenarioRequest,
    ) -> impl Iterator<Item = (usize, &'a crate::simulation::GameActionCommand, i128)> + use<'a>
    {
        let mut shifts = std::collections::BTreeMap::<usize, i128>::new();
        for branch in &self.live_branch_receipts {
            *shifts.entry(branch.after_action_index).or_default() +=
                i128::from(branch.selected_wait_us) - i128::from(branch.baseline_wait_us);
        }
        for wait in &self.live_energy_wait_receipts {
            if let Some(resumed) = wait.resumed_time_us {
                *shifts.entry(wait.after_action_index).or_default() +=
                    i128::from(resumed) - i128::from(wait.started_time_us);
            }
        }
        let mut shifts = shifts.into_iter().peekable();
        let mut offset = 0;
        request
            .actions
            .iter()
            .enumerate()
            .map(move |(index, action)| {
                while shifts
                    .peek()
                    .is_some_and(|(boundary, _)| *boundary <= index)
                {
                    offset += shifts.next().expect("peeked shift").1;
                }
                (index, action, i128::from(action.timestamp_us()) + offset)
            })
    }
}
