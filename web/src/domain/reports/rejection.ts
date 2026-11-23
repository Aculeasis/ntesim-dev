import type { ScenarioRequest } from '../contracts/scenario';
import type { ScenarioReport } from '../contracts/report';
import { english } from '../localization';
import { exactFraction, expectedActionTime } from './validation-support';
import { fraction } from './statistics';

export class TrialHaltError extends Error {}

export function validateRejectedAction(request: ScenarioRequest, report: ScenarioReport) {
  const rejected = report.rejected_action;
  if (!rejected) return;
  const command = request.actions[rejected.command_index];
  const actor = request.actors.find((actor) => actor.slot === rejected.actor_index);
  const active =
    report.trace.filter((event) => event.event_kind === 'switch_actor').at(-1)?.active_actor ??
    request.active_actor_index;
  if (
    report.success ||
    report.execution_completed ||
    !actor ||
    !command ||
    command.kind === 'wait' ||
    ![rejected.command_index, rejected.time_us, rejected.actor_index].every(
      (value) => Number.isSafeInteger(value) && value >= 0,
    ) ||
    rejected.time_us !== expectedActionTime(request, report, rejected.command_index) ||
    (command.kind === 'invoke_ability' || command.kind === 'player_input'
      ? command.actor_index !== rejected.actor_index
      : rejected.actor_index !== active) ||
    report.action_metadata_ledger?.some((entry) =>
      entry.command_indices.includes(rejected.command_index),
    ) ||
    report.player_input_receipts?.some((entry) => entry.command_index === rejected.command_index)
  )
    throw new Error('Invalid rejected action identity.');
  const reason = rejected.reason;
  if (!reason || typeof reason !== 'object') throw new Error('Missing rejected gate reason.');
  const nonnegative = (value: number) => Number.isSafeInteger(value) && value >= 0;
  switch (reason.kind) {
    case 'stamina': {
      const available = exactFraction(reason.available);
      const current = report.combat_stamina && exactFraction(report.combat_stamina.current);
      if (
        command.kind !== 'player_input' ||
        command.input !== 'dash' ||
        !request.combat_stamina ||
        !current ||
        available.numerator !== current.numerator ||
        available.denominator !== current.denominator ||
        exactFraction(reason.required).numerator <= 0n
      )
        throw new Error('Invalid rejected stamina facts.');
      break;
    }
    case 'energy': {
      const state = report.final_actor_states.find((state) => state.slot === rejected.actor_index);
      if (
        command.kind !== 'invoke_ability' ||
        !nonnegative(reason.required) ||
        !nonnegative(reason.available) ||
        reason.required !== command.energy_cost ||
        reason.available >= reason.required ||
        reason.available !== state?.energy
      )
        throw new Error('Invalid rejected energy facts.');
      break;
    }
    case 'recovery':
    case 'cooldown':
      if (!nonnegative(reason.until_us) || reason.until_us <= rejected.time_us)
        throw new Error('Invalid rejected timing facts.');
      break;
    case 'actor_not_active':
      if (
        (command.kind !== 'invoke_ability' && command.kind !== 'player_input') ||
        reason.active_actor_index !== active ||
        active === rejected.actor_index
      )
        throw new Error('Invalid rejected field owner.');
      break;
    case 'time_reversal':
      if (!nonnegative(reason.last_us) || reason.last_us <= rejected.time_us)
        throw new Error('Invalid rejected clock facts.');
      break;
    case 'timing_overflow':
      break;
    case 'other':
      if (typeof reason.message !== 'string' || !reason.message)
        throw new Error('Invalid rejected gate message.');
      break;
    default:
      throw new Error('Unknown rejected action reason.');
  }
}

export function rejectedActionMessage(request: ScenarioRequest, report: ScenarioReport) {
  const rejected = report.rejected_action;
  if (!rejected) return report.halt_diagnostic || 'Execution verification failed.';
  const command = request.actions[rejected.command_index];
  const actor = request.actors.find((actor) => actor.slot === rejected.actor_index);
  if (!command || !actor) throw new Error('Invalid rejected action identity.');
  const input =
    command.kind === 'invoke_ability'
      ? english.skill(command.ability_id)
      : command.kind === 'player_input'
        ? command.input
        : 'Swap';
  const reason = rejected.reason;
  let detail: string;
  switch (reason.kind) {
    case 'stamina':
      detail = `needs ${fraction(reason.required)} combat stamina; has ${fraction(reason.available)}`;
      break;
    case 'energy':
      detail = `needs ${reason.required} energy; has ${reason.available}`;
      break;
    case 'recovery':
      detail = `actor recovery ends at ${reason.until_us / 1e6}s`;
      break;
    case 'cooldown':
      detail = `cooldown ends at ${reason.until_us / 1e6}s`;
      break;
    case 'actor_not_active':
      detail = `${english.character(request.actors[reason.active_actor_index].character_id)} is on field`;
      break;
    case 'time_reversal':
      detail = `input precedes the actor's last action at ${reason.last_us / 1e6}s`;
      break;
    case 'timing_overflow':
      detail = 'action timing exceeds the supported range';
      break;
    case 'other':
      detail = reason.message;
      break;
  }
  return `${english.character(actor.character_id)} ${input} blocked at ${rejected.time_us / 1e6}s: ${detail}.`;
}
