import type { ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import { expectedActionTime, validateAssumptionIds } from './validation-support';
import { validateStaminaTransport } from './stamina';

export function validatePlayerInputs(request: ScenarioRequest, report: ScenarioReport) {
  const receipts = report.player_input_receipts ?? [];
  if (!Array.isArray(receipts)) throw new Error('Invalid player input receipt ledger.');
  const indices = new Set<number>();
  for (const receipt of receipts) {
    const command = request.actions[receipt.command_index];
    if (
      !Number.isSafeInteger(receipt.command_index) ||
      receipt.command_index < 0 ||
      indices.has(receipt.command_index) ||
      command?.kind !== 'player_input' ||
      command.actor_index !== receipt.actor_index ||
      command.input !== receipt.input ||
      (command.critical_success ?? false) !== (receipt.critical_success ?? false) ||
      expectedActionTime(request, report, receipt.command_index) !== receipt.time_us ||
      report.rejected_action?.command_index === receipt.command_index
    )
      throw new Error('Player input receipt differs from its admitted command.');
    indices.add(receipt.command_index);
    if (
      receipt.ability_id !== undefined &&
      (receipt.input !== 'arc' || typeof receipt.ability_id !== 'string' || !receipt.ability_id)
    )
      throw new Error('Invalid resolved player input ability identity.');
    const criticalSuccess = receipt.critical_success ?? false;
    if (criticalSuccess && receipt.input !== 'dash')
      throw new Error('Only a dash input may assume Critical Dodge success.');
    if (
      ![receipt.attack_mode_before, receipt.attack_mode_after].every((mode) =>
        ['normal', 'aim', 'ranged', 'melee'].includes(mode),
      )
    )
      throw new Error('Invalid player input mode metadata.');
    const resolvedAssumptions = report.resolved_model_assumptions ?? {};
    validateAssumptionIds(
      receipt.execution_model_assumption_ids,
      resolvedAssumptions,
      'player input execution assumptions',
      () => {},
    );
    const assumptionIds = receipt.execution_model_assumption_ids ?? [];
    if (
      receipt.input === 'arc'
        ? assumptionIds.length === 0
        : !assumptionIds.includes(
            receipt.input === 'dash' && request.combat_stamina
              ? 'combat.stamina_v1'
              : 'inputs.special_dodge_timing_v1',
          )
    )
      throw new Error('Player input timing has no hypothesis receipt.');
    if (criticalSuccess !== assumptionIds.includes('inputs.critical_dodge_assumed_success_v1'))
      throw new Error('Critical Dodge success flag differs from its assumption receipt.');
    if (criticalSuccess && !resolvedAssumptions['inputs.critical_dodge_assumed_success_v1']?.trim())
      throw new Error('Critical Dodge assumption differs from resolved model provenance.');
  }
  if (
    report.execution_completed &&
    request.actions.some((command, index) => command.kind === 'player_input' && !indices.has(index))
  )
    throw new Error('Missing admitted player input receipt.');
  for (const actor of report.final_actor_states) {
    if (!['normal', 'aim', 'ranged', 'melee'].includes(actor.attack_mode))
      throw new Error('Invalid final attack mode.');
  }
  validateStaminaTransport(request, report, receipts);
  return receipts;
}
