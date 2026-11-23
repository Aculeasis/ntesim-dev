import type { AutonomousEventReceipt, HitReceipt, ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import { exactFraction } from './validation-support';
import { hasInitiatingOccurrence } from './break-contribution';
import { validateDamageParent } from './damage-parent';

const index = (value: number) => Number.isSafeInteger(value) && value >= 0;

export function validateAutonomousEvents(request: ScenarioRequest, report: ScenarioReport) {
  const events = report.autonomous_events ?? [];
  if (!Array.isArray(events)) throw new Error('Invalid autonomous event ledger.');
  for (const [ordinal, event] of events.entries()) {
    if (
      !index(event.actor_index) ||
      !request.actors[event.actor_index] ||
      !index(event.active_actor_index) ||
      !request.actors[event.active_actor_index] ||
      !index(event.time_us) ||
      event.time_us > report.absolute_elapsed_us ||
      typeof event.ability_id !== 'string' ||
      !event.ability_id.length
    )
      throw new Error('Invalid autonomous event identity.');
    const trigger = event.trigger;
    let valid = false;
    switch (trigger?.kind) {
      case 'initial_state':
        valid = true;
        break;
      case 'coordinated_hit':
      case 'settled_hit': {
        const hit = report.hit_receipts[trigger.hit_index];
        valid =
          index(trigger.hit_index) &&
          !!hit &&
          hit.time_us === event.time_us &&
          index(hit.actor_index) &&
          !!request.actors[hit.actor_index] &&
          index(hit.target_index) &&
          !!request.targets[hit.target_index] &&
          (hit.autonomous_event_index === undefined || hit.autonomous_event_index < ordinal);
        if (trigger.kind === 'coordinated_hit') {
          const cast = request.actions[trigger.cast_command_index];
          valid =
            valid &&
            hit.applied_hp_loss > 0 &&
            index(trigger.cast_command_index) &&
            cast?.kind === 'invoke_ability' &&
            cast.actor_index === event.actor_index &&
            cast.timestamp_us <= event.time_us;
        }
        break;
      }
      case 'field_entry':
      case 'action': {
        const command = request.actions[trigger.command_index];
        valid =
          index(trigger.command_index) &&
          !!command &&
          (trigger.kind !== 'field_entry' || command.kind === 'switch_actor');
        break;
      }
      case 'incoming_damage':
        valid =
          index(trigger.event_index) && !!request.incoming_damage_events?.[trigger.event_index];
        break;
      case 'hp_cost':
        valid = index(trigger.event_index) && !!request.hp_cost_events?.[trigger.event_index];
        break;
      case 'vital_change':
        valid = index(trigger.event_sequence);
        break;
      case 'owner_deadline':
        valid =
          index(trigger.autonomous_event_index) &&
          trigger.autonomous_event_index < ordinal &&
          events[trigger.autonomous_event_index].time_us <= event.time_us;
        break;
    }
    if (!valid) throw new Error('Autonomous event has no real causal parent.');
    const heal = event.recipient_healing;
    if (!heal) continue;
    if (
      !index(heal.recipient_actor_index) ||
      !request.actors[heal.recipient_actor_index] ||
      ![heal.gross, heal.applied, heal.excess, heal.hp_before, heal.hp_after].every(index) ||
      heal.gross - heal.applied !== heal.excess ||
      heal.hp_after - heal.hp_before !== heal.applied ||
      exactFraction(heal.base).numerator < 0n
    )
      throw new Error('Invalid autonomous healing settlement.');
  }
}

export function hasAutonomousOrigin(hit: HitReceipt, events: AutonomousEventReceipt[]) {
  if (hit.autonomous_event_index === undefined) return false;
  const source = events[hit.autonomous_event_index];
  return (
    index(hit.autonomous_event_index) &&
    hit.command_index === undefined &&
    !hit.switch_origin &&
    !!source &&
    source.time_us <= hit.time_us &&
    (!!hit.break_contribution || source.actor_index === hit.actor_index)
  );
}

export function validateInitiatingOccurrences(report: ScenarioReport) {
  if (
    !report.action_metadata_ledger &&
    !report.autonomous_events &&
    !report.player_input_receipts?.some((input) => input.ability_id)
  )
    return;
  for (const [hitIndex, hit] of report.hit_receipts.entries()) {
    if (validateDamageParent(report, hitIndex)) continue;
    if (
      !hasAutonomousOrigin(hit, report.autonomous_events ?? []) &&
      !hasInitiatingOccurrence(
        hit,
        report.action_metadata_ledger ?? [],
        report.metadata_catalog ?? [],
        report.player_input_receipts,
      )
    )
      throw new Error(
        `Hit receipt has no admitted action metadata occurrence (${hit.damage_effect_id}, command ${hit.command_index ?? 'none'}).`,
      );
  }
}
