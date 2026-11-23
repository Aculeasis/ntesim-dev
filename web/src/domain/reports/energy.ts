import type { EnergyGrant, ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import { exactFraction, equalExact, expectedActionTime } from './validation-support';
import { energySourceDetails } from './energy-sources';

const zeroCarry = { numerator: 0n, denominator: 1n };

interface EnergyPoolState {
  energy: number;
  capacity: number;
  initial: number;
  granted: number;
  overflow: number;
  spent: number;
  chargeRemainder: { numerator: bigint; denominator: bigint };
}

function chargeRemainder(value: EnergyGrant['charge_remainder_before'] | undefined) {
  try {
    const parsed = value ? exactFraction(value) : zeroCarry;
    if (parsed.numerator < 0n || parsed.numerator >= parsed.denominator)
      throw new Error('outside [0, 1)');
    return parsed;
  } catch {
    throw new Error('Invalid exact energy charge remainder.');
  }
}

function nonnegativeExact(value: EnergyGrant['charge_basis'], message: string) {
  if (value === undefined) return;
  let parsed: { numerator: bigint; denominator: bigint };
  try {
    parsed = exactFraction(value);
  } catch {
    throw new Error(message);
  }
  if (parsed.numerator < 0n) throw new Error(message);
}

function applyGrantReceipt(
  pools: Map<number, EnergyPoolState>,
  request: ScenarioRequest,
  grant: EnergyGrant,
  eventKind: string,
) {
  energySourceDetails(grant.source, eventKind);
  const actor = pools.get(grant.recipient_actor_index);
  const sourceActor = request.actors.find((a) => a.slot === grant.source_actor_index);
  const recipientActor = request.actors.find((a) => a.slot === grant.recipient_actor_index);
  const carryBefore = chargeRemainder(grant.charge_remainder_before),
    carryAfter = chargeRemainder(grant.charge_remainder_after);
  if (
    !actor ||
    !sourceActor ||
    !recipientActor ||
    ![
      grant.source_actor_index,
      grant.recipient_actor_index,
      grant.energy_before,
      grant.gross,
      grant.retained,
      grant.overflow,
      grant.energy_after,
    ].every((value) => Number.isSafeInteger(value) && value >= 0)
  )
    throw new Error('Invalid typed energy ownership or units.');
  nonnegativeExact(grant.charge_basis, 'Invalid packet energy basis.');
  if (grant.hit_basis) {
    nonnegativeExact(grant.hit_basis.base, 'Invalid packet energy basis.');
    nonnegativeExact(grant.hit_basis.distribution, 'Invalid packet energy basis.');
  }
  if (
    grant.energy_before !== actor.energy ||
    !equalExact(carryBefore, actor.chargeRemainder) ||
    grant.retained + grant.overflow !== grant.gross ||
    grant.energy_after > actor.capacity ||
    grant.energy_after !== actor.energy + grant.retained
  )
    throw new Error('Typed energy transition does not reconcile with its pool.');
  actor.energy = grant.energy_after;
  actor.chargeRemainder = carryAfter;
  actor.granted += grant.gross;
  actor.overflow += grant.overflow;
}

export function validateEnergy(request: ScenarioRequest, report: ScenarioReport) {
  const pools = new Map<number, EnergyPoolState>(
    request.actors.map((actor) => [
      actor.slot,
      {
        energy: actor.initial_energy,
        capacity: actor.energy_capacity,
        initial: actor.initial_energy,
        granted: 0,
        overflow: 0,
        spent: 0,
        chargeRemainder: zeroCarry,
      },
    ]),
  );
  const admissionKey = (commandIndex: number, abilityId: string) => `${commandIndex}:${abilityId}`;
  const admittedCommands = new Set<string>();
  const entryTimes = new Map<string, number>();
  for (const event of report.trace ?? []) {
    if (
      event.event_kind === 'owner_entry' &&
      event.data?.command_index !== undefined &&
      event.data.ability_id !== undefined
    )
      entryTimes.set(admissionKey(event.data.command_index, event.data.ability_id), event.time_us);
  }
  const occurrences = new Map<
    string,
    NonNullable<ScenarioReport['action_metadata_ledger']>[number]
  >();
  for (const entry of report.action_metadata_ledger ?? []) {
    for (const commandIndex of entry.command_indices) {
      occurrences.set(admissionKey(commandIndex, entry.ability_id), entry);
    }
  }
  const branchReceipts = report.live_branch_receipts ?? [],
    energyWaitReceipts = report.live_energy_wait_receipts ?? [],
    resumedEnergyWaitReceipts = energyWaitReceipts.filter(
      (receipt) => receipt.resumed_time_us !== null,
    ),
    hasTargetHpBranch = branchReceipts.some(
      (receipt) => receipt.condition.kind === 'target_hp_below',
    ),
    targetHp = hasTargetHpBranch
      ? new Map(request.targets.map((target) => [target.target_index, target.hp_current]))
      : undefined;
  let branchMarkerIndex = 0,
    energyWaitStartMarkerIndex = 0,
    energyWaitResumeMarkerIndex = 0,
    hitMarkerIndex = 0;
  let pendingEnergyWait: (typeof energyWaitReceipts)[number] | undefined;
  for (const event of report.trace) {
    const hasEnergyReceipt = Boolean(event.energy_change || event.energy_fanout);
    if (event.energy_change && event.energy_fanout)
      throw new Error('Trace event cannot carry both energy_change and energy_fanout.');
    if (event.event_kind === 'live_branch') {
      const receipt = branchReceipts[branchMarkerIndex++];
      if (!receipt) throw new Error('Live branch trace marker has no matching receipt.');
      if (
        event.phase !== 'Observation' ||
        hasEnergyReceipt ||
        event.time_us !== receipt.time_us ||
        event.active_actor !== receipt.observer_actor_index
      )
        throw new Error('Live branch receipt differs from its observation trace.');
      if (receipt.condition.kind === 'active_actor_energy_below') {
        const actor = pools.get(event.active_actor);
        if (!actor || receipt.observed_value !== actor.energy)
          throw new Error('Live energy branch observation differs from the typed energy ledger.');
      } else {
        const hp = targetHp?.get(receipt.condition.target_index);
        if (hp === undefined || receipt.observed_value !== hp)
          throw new Error('Live target HP observation differs from the hit receipt ledger.');
      }
    } else if (event.event_kind === 'live_energy_wait_started') {
      if (pendingEnergyWait)
        throw new Error('Live energy wait start marker overlaps an unresolved wait.');
      const receipt = energyWaitReceipts[energyWaitStartMarkerIndex++],
        actor = receipt && pools.get(receipt.actor_index);
      if (!receipt) throw new Error('Live energy wait start marker has no matching receipt.');
      if (
        event.phase !== 'Observation' ||
        hasEnergyReceipt ||
        event.time_us !== receipt.started_time_us ||
        !actor ||
        receipt.energy_at_start !== actor.energy
      )
        throw new Error('Live energy wait start receipt differs from its trace observation.');
      pendingEnergyWait = receipt;
    } else if (event.event_kind === 'live_energy_wait_resumed') {
      const receipt = resumedEnergyWaitReceipts[energyWaitResumeMarkerIndex++],
        actor = receipt && pools.get(receipt.actor_index);
      if (!receipt) throw new Error('Live energy wait resume marker has no matching receipt.');
      if (pendingEnergyWait !== receipt)
        throw new Error('Live energy wait resume marker has no matching start.');
      if (
        event.phase !== 'Observation' ||
        hasEnergyReceipt ||
        event.time_us !== receipt.resumed_time_us ||
        !actor ||
        receipt.energy_at_resume !== actor.energy
      )
        throw new Error('Live energy wait resume receipt differs from its trace observation.');
      pendingEnergyWait = undefined;
    } else if (
      hasTargetHpBranch &&
      ['hit_damage', 'break_damage', 'reaction_damage'].includes(event.event_kind)
    ) {
      const hit = report.hit_receipts[hitMarkerIndex++],
        hp = hit && targetHp?.get(hit.target_index);
      if (!hit || hit.time_us !== event.time_us || hp === undefined)
        throw new Error('Target HP branch cannot be reconciled with ordered hit markers.');
      targetHp!.set(hit.target_index, hp - hit.applied_hp_loss);
    }
    const change = event.energy_change;
    if (event.event_kind === 'invoke_ability' || event.event_kind === 'utility_ability') {
      if (!change || change.source !== 'ability_cost' || event.energy_fanout)
        throw new Error('Missing typed admitted energy cost receipt.');
      const actor = pools.get(change.recipient_actor_index);
      const carryBefore = chargeRemainder(change.charge_remainder_before),
        carryAfter = chargeRemainder(change.charge_remainder_after);
      const command = request.actions[change.command_index];
      const key = admissionKey(change.command_index, change.ability_id);
      const occurrence = occurrences.get(key);
      const supportSwitch =
        command?.kind === 'switch_actor' && occurrence?.switch_origin?.kind === 'support';
      const commandActor =
        command?.kind === 'switch_actor'
          ? command.target_actor_index
          : command && 'actor_index' in command
            ? command.actor_index
            : undefined;
      if (
        !actor ||
        ![
          change.source_actor_index,
          change.recipient_actor_index,
          change.command_index,
          change.energy_before,
          change.spent,
          change.energy_after,
        ].every((value) => Number.isSafeInteger(value) && value >= 0) ||
        !command ||
        (command.kind !== 'invoke_ability' && command.kind !== 'player_input' && !supportSwitch) ||
        admittedCommands.has(key) ||
        commandActor !== change.recipient_actor_index ||
        change.source_actor_index !== change.recipient_actor_index ||
        event.active_actor !== change.recipient_actor_index ||
        (command.kind === 'invoke_ability' && command.ability_id !== change.ability_id) ||
        (command.kind === 'invoke_ability' ? command.energy_cost : 0) !== change.spent ||
        event.time_us !==
          ((supportSwitch ? entryTimes.get(key) : undefined) ??
            expectedActionTime(request, report, change.command_index)) ||
        !occurrence ||
        occurrence.actor_index !== change.recipient_actor_index ||
        occurrence.ability_id !== change.ability_id ||
        change.energy_before !== actor.energy ||
        !equalExact(carryBefore, actor.chargeRemainder) ||
        !equalExact(carryAfter, actor.chargeRemainder) ||
        change.spent > actor.energy ||
        change.energy_after !== actor.energy - change.spent
      )
        throw new Error('Invalid admitted energy cost receipt.');
      admittedCommands.add(key);
      actor.energy = change.energy_after;
      actor.spent += change.spent;
      continue;
    }
    if (change) {
      if (change.source === 'ability_cost')
        throw new Error('Energy cost outside action admission.');
      applyGrantReceipt(pools, request, change, event.event_kind);
    } else if (event.energy_fanout) {
      const fanout = event.energy_fanout;
      if (!Array.isArray(fanout.recipients) || fanout.recipients.length === 0)
        throw new Error('Invalid typed energy fanout.');
      nonnegativeExact(fanout.base, 'Invalid packet energy basis.');
      const seenRecipients = new Set<number>();
      for (const recipient of fanout.recipients) {
        if (seenRecipients.has(recipient.recipient_actor_index))
          throw new Error('Duplicate recipient in typed energy fanout.');
        seenRecipients.add(recipient.recipient_actor_index);
        nonnegativeExact(recipient.distribution, 'Invalid packet energy basis.');
        applyGrantReceipt(
          pools,
          request,
          {
            source: fanout.source,
            source_actor_index: fanout.source_actor_index,
            recipient_actor_index: recipient.recipient_actor_index,
            energy_before: recipient.energy_before,
            gross: recipient.gross,
            retained: recipient.retained,
            overflow: recipient.overflow,
            energy_after: recipient.energy_after,
            charge_remainder_before: recipient.charge_remainder_before,
            charge_remainder_after: recipient.charge_remainder_after,
            charge_basis: recipient.charge_basis,
          },
          event.event_kind,
        );
      }
    }
  }
  if (branchMarkerIndex !== branchReceipts.length)
    throw new Error('Live branch markers do not match receipt count.');
  if (energyWaitStartMarkerIndex !== energyWaitReceipts.length)
    throw new Error('Live energy wait start markers do not match receipt count.');
  if (energyWaitResumeMarkerIndex !== resumedEnergyWaitReceipts.length)
    throw new Error('Live energy wait resume markers do not match receipt count.');
  if (hasTargetHpBranch && hitMarkerIndex !== report.hit_receipts.length)
    throw new Error('Target HP branch hit markers do not match hit receipt count.');
  if (
    report.action_metadata_ledger?.some((entry) =>
      entry.command_indices.some(
        (commandIndex) => !admittedCommands.has(admissionKey(commandIndex, entry.ability_id)),
      ),
    )
  )
    throw new Error('Missing admitted energy cost observation.');
  for (const state of report.final_actor_states) {
    const actor = pools.get(state.slot)!;
    const finalCarry = chargeRemainder(state.charge_remainder);
    for (const value of [
      state.energy,
      state.granted_energy,
      state.overflow_energy,
      state.spent_energy,
    ])
      if (!Number.isSafeInteger(value) || value < 0)
        throw new Error('Invalid energy ledger units.');
    if (
      actor.initial !== state.initial_energy ||
      actor.capacity !== state.energy_capacity ||
      actor.energy !== state.energy ||
      actor.granted !== state.granted_energy ||
      actor.overflow !== state.overflow_energy ||
      actor.spent !== state.spent_energy ||
      !equalExact(actor.chargeRemainder, finalCarry)
    )
      throw new Error('Typed energy receipts do not reconcile with the final ledger.');
  }
}
