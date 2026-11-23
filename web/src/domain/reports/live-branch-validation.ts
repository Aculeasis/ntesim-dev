import type { ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import type { LiveBranch, LiveEnergyWait } from '../contracts/scenario';

function activeActorAtBoundary(request: ScenarioRequest, actionIndex: number) {
  let active = request.active_actor_index;
  for (const action of request.actions.slice(0, actionIndex))
    if (action.kind === 'switch_actor') active = action.target_actor_index;
  return active;
}

export function validateLiveBranchReceipts(
  request: ScenarioRequest,
  branches: LiveBranch[],
  energyWaits: LiveEnergyWait[],
  report: ScenarioReport,
) {
  const receipts = report.live_branch_receipts ?? [];
  if (!Array.isArray(report.live_branch_receipts ?? []))
    throw new Error('Invalid live branch receipt ledger.');
  if (receipts.length > branches.length)
    throw new Error('Report contains an undeclared live branch receipt.');
  for (const [index, branch] of branches.entries()) {
    const receipt = receipts[index];
    if (!receipt) {
      if (report.execution_completed && branch.timestamp_us <= report.absolute_elapsed_us)
        throw new Error('Completed live program is missing its branch receipt.');
      continue;
    }
    const expectedCondition = branch.condition,
      observedCondition = receipt.condition;
    if (
      ![
        receipt.time_us,
        receipt.observer_actor_index,
        receipt.observed_value,
        receipt.selected_wait_us,
        receipt.baseline_wait_us,
        receipt.after_action_index,
        receipt.source_line,
        receipt.source_column,
      ].every((value) => Number.isSafeInteger(value) && value >= 0) ||
      receipt.time_us !== branch.timestamp_us ||
      receipt.time_us > report.absolute_elapsed_us ||
      !observedCondition ||
      observedCondition.kind !== expectedCondition.kind ||
      observedCondition.threshold !== expectedCondition.threshold ||
      (expectedCondition.kind === 'target_hp_below' &&
        (observedCondition.kind !== 'target_hp_below' ||
          observedCondition.target_index !== expectedCondition.target_index)) ||
      receipt.baseline_wait_us !== branch.wait_if_below_us ||
      receipt.after_action_index !== branch.after_action_index ||
      receipt.source_line !== branch.source_line ||
      receipt.source_column !== branch.source_column ||
      (expectedCondition.kind === 'active_actor_energy_below' &&
        receipt.observer_actor_index !==
          activeActorAtBoundary(request, branch.after_action_index)) ||
      receipt.selected_wait_us !==
        (receipt.observed_value < expectedCondition.threshold
          ? branch.wait_if_below_us
          : branch.wait_otherwise_us)
    )
      throw new Error('Live branch receipt differs from the compiled program or observation.');
  }
  const energyReceipts = report.live_energy_wait_receipts ?? [];
  if (!Array.isArray(report.live_energy_wait_receipts ?? []))
    throw new Error('Invalid live energy wait receipt ledger.');
  if (energyReceipts.length > energyWaits.length)
    throw new Error('Report contains an undeclared live energy wait receipt.');
  let accumulatedWaitUs = 0,
    previousWaitResolved = true;
  for (const [index, wait] of energyWaits.entries()) {
    const receipt = energyReceipts[index];
    const expectedStartTime = wait.timestamp_us + accumulatedWaitUs;
    if (!Number.isSafeInteger(expectedStartTime))
      throw new Error('Live energy wait schedule exceeds the safe clock range.');
    if (!receipt) {
      if (report.execution_completed && expectedStartTime <= report.absolute_elapsed_us)
        throw new Error('Completed live program is missing its energy wait receipt.');
      previousWaitResolved = false;
      continue;
    }
    if (!previousWaitResolved)
      throw new Error('A later energy wait started before an earlier wait resolved.');
    if (
      ![
        receipt.started_time_us,
        receipt.actor_index,
        receipt.energy_at_start,
        receipt.energy_at_least,
        receipt.after_action_index,
        receipt.source_line,
        receipt.source_column,
      ].every((value) => Number.isSafeInteger(value) && value >= 0) ||
      receipt.started_time_us !== expectedStartTime ||
      receipt.started_time_us > report.absolute_elapsed_us ||
      receipt.actor_index !== wait.actor_index ||
      receipt.energy_at_least !== wait.energy_at_least ||
      receipt.after_action_index !== wait.after_action_index ||
      receipt.source_line !== wait.source_line ||
      receipt.source_column !== wait.source_column
    )
      throw new Error('Live energy wait receipt differs from the compiled program.');
    const resumed = receipt.resumed_time_us,
      energyAtResume = receipt.energy_at_resume;
    if ((resumed === null) !== (energyAtResume === null))
      throw new Error('Live energy wait has an incomplete resume observation.');
    if (resumed === null) {
      if (report.execution_completed)
        throw new Error('Completed live program contains an unresolved energy wait.');
      if (index + 1 < energyWaits.length && energyReceipts[index + 1])
        throw new Error('A later energy wait started before an earlier wait resumed.');
      previousWaitResolved = false;
      continue;
    }
    if (energyAtResume === null)
      throw new Error('Live energy wait has no settled energy at resume.');
    if (
      !Number.isSafeInteger(resumed) ||
      resumed < receipt.started_time_us ||
      resumed > report.absolute_elapsed_us ||
      !Number.isSafeInteger(energyAtResume) ||
      energyAtResume < receipt.energy_at_least
    )
      throw new Error('Live energy wait resumed without a valid settled threshold.');
    accumulatedWaitUs += resumed - receipt.started_time_us;
    if (!Number.isSafeInteger(accumulatedWaitUs))
      throw new Error('Live energy wait delays exceed the safe clock range.');
  }
}
