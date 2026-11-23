import type { ScenarioRequest } from '../contracts/scenario';
import type {
  ActionPacketMetadataReceipt,
  EnergyFanout,
  EnergyGrant,
  EnergySpend,
  HitReceipt,
  InputActionRole,
  ScenarioReport,
  TraceEvent,
} from '../contracts/report';
import { english } from '../localization';

function actionLabel(ability: string, role?: InputActionRole | null) {
  if (role === 'critical_riposte') return 'Critical Riposte';
  if (role === 'parry_attack') return 'Parry Attack';
  return english.skill(ability);
}

export interface ReplayEvent {
  index: number;
  time: number;
  kind: string;
  owner: number | null;
  frameIndex: number;
  hit?: HitReceipt;
  trace?: TraceEvent;
  admitted?: EnergySpend;
  grant?: EnergyGrant;
  fanout?: EnergyFanout;
  primarySlot?: 'normal' | 'skill' | 'ultimate' | null;
  inputRole?: InputActionRole | null;
  actor: string;
  detail: string;
  action: string;
}

export function replayEvents(report: ScenarioReport): ReplayEvent[] {
  const catalog = report.metadata_catalog ?? [];
  const metadata = new Map<number, ActionPacketMetadataReceipt['action']>();
  for (const entry of report.action_metadata_ledger ?? []) {
    const item = catalog[entry.metadata_id];
    if (!item) continue;
    for (const commandIndex of entry.command_indices) {
      if (!metadata.has(commandIndex)) metadata.set(commandIndex, item.action);
    }
  }
  const inputs = new Map(report.player_input_receipts?.map((e) => [e.command_index, e]));
  const inputLabel = (input: 'arc' | 'special' | 'dash') =>
    input === 'arc' ? 'Arc' : input === 'special' ? 'Special (G)' : 'Dash';
  const events: ReplayEvent[] = [];
  let traceEnd = 0,
    hitsEnd = 0;
  for (const [frameIndex, frame] of report.state_timeline.entries()) {
    if (frameIndex === 0) continue;
    // Trace and hit arrays have separate native prefix identities. Do not pair
    // them by parsing diagnostic prose or claiming callback order within a frame.
    for (const trace of report.trace.slice(traceEnd, frame.trace_end)) {
      const change = trace.energy_change;
      const admitted = change?.source === 'ability_cost' ? change : undefined;
      const grant = change?.source !== 'ability_cost' ? change : undefined;
      const fanout = trace.energy_fanout;
      const linkedHit =
        trace.hit_index === undefined ? undefined : report.hit_receipts[trace.hit_index];
      const action = admitted ? metadata.get(admitted.command_index) : undefined;
      const input = admitted ? inputs.get(admitted.command_index) : undefined;
      const owner =
        trace.event_kind === 'switch_actor'
          ? trace.active_actor
          : (change?.recipient_actor_index ??
            linkedHit?.actor_index ??
            trace.from_actor ??
            trace.to_actor ??
            fanout?.source_actor_index ??
            null);
      events.push({
        frameIndex,
        index: events.length,
        time: trace.time_us,
        kind: trace.event_kind,
        owner,
        trace,
        admitted,
        grant,
        fanout,
        primarySlot: action?.primary_slot,
        inputRole: action?.input_role,
        ...english.event(trace, report.final_actor_states, linkedHit),
        ...(input
          ? { action: inputLabel(input.input) }
          : admitted
            ? { action: actionLabel(admitted.ability_id, action?.input_role) }
            : {}),
      });
    }
    for (const hit of report.hit_receipts.slice(hitsEnd, frame.hits_end)) {
      const input = hit.command_index === undefined ? undefined : inputs.get(hit.command_index);
      const hitMetadata = catalog[hit.metadata_id];
      events.push({
        frameIndex,
        index: events.length,
        time: hit.time_us,
        kind: 'damage_receipt',
        owner: hit.actor_index,
        hit,
        actor: english.actor(hit.actor_index, report.final_actor_states),
        action: hit.break_contribution
          ? 'Break'
          : input
            ? inputLabel(input.input)
            : actionLabel(hit.ability_id, hitMetadata?.action.input_role),
        primarySlot: hitMetadata?.action.primary_slot,
        inputRole: hitMetadata?.action.input_role,
        detail: `${hit.damage_effect_id} · target ${hit.target_index + 1} · exact ${hit.damage}`,
      });
    }
    events.push({
      index: events.length,
      time: frame.time_us,
      kind: 'state_boundary',
      owner: null,
      frameIndex,
      actor: 'Team / system',
      action: `State ${frameIndex}`,
      detail: `${frame.boundary.kind} · ${frame.hits_end} hits · ${frame.healing_end} healing receipts`,
    });
    traceEnd = frame.trace_end;
    hitsEnd = frame.hits_end;
  }
  return events;
}

export function fieldSegments(request: ScenarioRequest, events: ReplayEvent[], horizon: number) {
  const segments: { slot: number; start: number; end: number }[] = [];
  let active = request.active_actor_index,
    start = 0;
  for (const e of events.filter((e) => e.kind === 'switch_actor')) {
    if (e.time > start) segments.push({ slot: active, start, end: e.time });
    active = e.trace!.active_actor;
    start = e.time;
  }
  if (start < horizon) segments.push({ slot: active, start, end: horizon });
  return segments;
}
