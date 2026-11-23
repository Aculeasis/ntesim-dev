import type { PlayerInputReceipt, ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import { exactFraction, expectedActionTime } from './validation-support';

export function validateStaminaTransport(
  request: ScenarioRequest,
  report: ScenarioReport,
  inputs: PlayerInputReceipt[],
) {
  const state = report.combat_stamina;
  if (Boolean(request.combat_stamina) !== Boolean(state))
    throw new Error('Combat stamina profile/report presence differs.');
  if (!state || !request.combat_stamina) return;
  for (const key of ['capacity', 'initial', 'use_efficiency', 'recovery_rate'] as const) {
    const expected = exactFraction(request.combat_stamina[key]);
    const actual = exactFraction(state.profile[key]);
    if (actual.numerator !== expected.numerator || actual.denominator !== expected.denominator)
      throw new Error('Combat stamina profile differs from its request.');
  }
  for (const value of [state.current, state.total_spent, state.total_recovered]) {
    if (exactFraction(value).numerator < 0n) throw new Error('Invalid exact stamina observation.');
  }
  if (
    !Number.isSafeInteger(state.recovery_ready_at_us) ||
    state.recovery_ready_at_us < 0 ||
    state.updated_at_us !== report.final_buff_state.time_us ||
    !Array.isArray(state.spends)
  )
    throw new Error('Invalid combat stamina clock/receipt ledger.');
  const indices = new Set<number>();
  const byCommand = new Map(inputs.map((receipt) => [receipt.command_index, receipt]));
  for (const spend of state.spends) {
    const input = byCommand.get(spend.command_index);
    if (
      !input ||
      input.input !== 'dash' ||
      input.actor_index !== spend.actor_index ||
      indices.has(spend.command_index) ||
      expectedActionTime(request, report, spend.command_index) !== spend.time_us
    )
      throw new Error('Combat stamina spend differs from its admitted input.');
    for (const value of [spend.amount, spend.before, spend.after]) {
      if (exactFraction(value).numerator < 0n) throw new Error('Invalid exact stamina receipt.');
    }
    indices.add(spend.command_index);
  }
  if (inputs.some((receipt) => receipt.input === 'dash' && !indices.has(receipt.command_index)))
    throw new Error('Missing combat stamina spend for admitted dash.');
}
