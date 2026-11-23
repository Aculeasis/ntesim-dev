import type { SourceExecution } from '../contracts/source';
import type { ReplayEvent } from './events';

/** Follow only recorded causal identities; autonomous initial state has no line. */
export function eventLocation(execution: SourceExecution, event: ReplayEvent | null) {
  let command = event?.admitted?.command_index;
  let hit = event?.hit;
  let index: number | undefined;
  const autonomous = execution.report.autonomous_events ?? [];
  for (
    let remaining = execution.report.hit_receipts.length + autonomous.length + 1;
    command === undefined && remaining > 0;
    remaining--
  ) {
    if (hit) {
      command = hit.command_index;
      if (command !== undefined) break;
      if (hit.parent_hit_index !== undefined) {
        hit = execution.report.hit_receipts[hit.parent_hit_index];
        continue;
      }
      index = hit.autonomous_event_index;
      hit = undefined;
      continue;
    }
    if (index === undefined) break;
    const trigger = autonomous[index]?.trigger;
    index = undefined;
    if (trigger?.kind === 'action' || trigger?.kind === 'field_entry')
      command = trigger.command_index;
    else if (trigger?.kind === 'coordinated_hit') command = trigger.cast_command_index;
    else if (trigger?.kind === 'settled_hit')
      hit = execution.report.hit_receipts[trigger.hit_index];
    else if (trigger?.kind === 'owner_deadline') index = trigger.autonomous_event_index;
  }
  return command === undefined ? null : (execution.action_locations[command] ?? null);
}
