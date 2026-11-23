import type { LiveProgramDeclarations, ScenarioRequest } from '../contracts/scenario';
import type { ScenarioReport } from '../contracts/report';
import { replayEvents, fieldSegments, type ReplayEvent } from './events';
import { validateObservations } from '../reports/validation';
import { actorSummaryFromDelta, applyActorDelta, applyBuffDiff } from '../reports/timeline-fold';
import { ClockIndex } from '../reports/clocks';

export function createReplay(
  request: ScenarioRequest,
  report: ScenarioReport,
  liveProgram?: LiveProgramDeclarations,
) {
  validateObservations(request, report, liveProgram?.branches, liveProgram?.energy_waits);
  const clocks = new ClockIndex(report);
  const initial = report.state_timeline[0];
  if (!initial || initial.boundary.kind !== 'initial')
    throw new Error('Missing recorded initial state.');
  const events = replayEvents(report),
    horizon = report.absolute_elapsed_us,
    segments = fieldSegments(request, events, horizon);
  const stateAt = (time: number, limit = Infinity) => {
    const t = Math.max(0, Math.min(horizon, Number(time) || 0));
    const native = {
      time_us: initial.time_us,
      boundary: initial.boundary,
      frameIndex: 0,
      active_actor_index: initial.active_actor_index,
      actors: (initial.actors ?? []).map(actorSummaryFromDelta),
      targets: [...(initial.targets ?? [])],
      buffs: [...(initial.buffs ?? [])],
      timers: [...(initial.timers ?? [])],
      ...(initial.stamina !== undefined ? { stamina: initial.stamina } : {}),
      hits_end: initial.hits_end,
      healing_end: initial.healing_end,
      trace_end: initial.trace_end,
    };
    let count = 0,
      last: ReplayEvent | null = null;
    const lastActions = new Map<number, ReplayEvent>();
    for (const e of events) {
      if (e.time > t || count >= limit) break;
      count++;
      last = e;
      if (e.admitted && e.owner !== null) lastActions.set(e.owner, e);
      if (e.kind === 'state_boundary') {
        const frame = report.state_timeline[e.frameIndex];
        for (const actor of frame.actors ?? []) {
          native.actors[actor.slot] =
            frame.boundary.kind === 'event'
              ? applyActorDelta(native.actors[actor.slot], actor)
              : actorSummaryFromDelta(actor);
        }
        for (const target of frame.targets ?? []) native.targets[target.target_index] = target;
        if (frame.buffs) native.buffs = [...frame.buffs];
        else if (frame.buff_diff) native.buffs = applyBuffDiff(native.buffs, frame.buff_diff);
        if (frame.stamina !== undefined && frame.stamina !== null) native.stamina = frame.stamina;
        if (frame.timers) native.timers = [...frame.timers];
        Object.assign(native, {
          time_us: frame.time_us,
          boundary: frame.boundary,
          frameIndex: e.frameIndex,
          active_actor_index: frame.active_actor_index,
          hits_end: frame.hits_end,
          healing_end: frame.healing_end,
          trace_end: frame.trace_end,
        });
      }
    }
    const actors = native.actors.map((a) => ({
      slot: a.slot,
      character_id: a.character_id,
      energy: a.energy,
      capacity: a.energy_capacity,
      damage: 0,
      hits: 0,
      spent: a.spent_energy,
      granted: a.granted_energy,
      overflow: a.overflow_energy,
      charge_remainder: a.charge_remainder ?? '0/1',
      lastAction: lastActions.get(a.slot) ?? null,
    }));
    const targets = native.targets.map((a) => ({
      index: a.target_index,
      hp: a.hp_current,
      max: a.hp_max,
      damage: 0,
      hits: 0,
    }));
    let damage = 0n;
    const actorDamage = actors.map(() => 0n),
      targetDamage = targets.map(() => 0n);
    // Accumulate settled integers before the presentation conversion.
    for (const hit of report.hit_receipts.slice(0, native.hits_end)) {
      const value = BigInt(hit.rounded_damage);
      damage += value;
      actorDamage[hit.actor_index] += value;
      actors[hit.actor_index].hits++;
      targetDamage[hit.target_index] += value;
      targets[hit.target_index].hits++;
    }
    actors.forEach((actor, i) => {
      actor.damage = Number(actorDamage[i]);
    });
    targets.forEach((target, i) => {
      target.damage = Number(targetDamage[i]);
    });
    return {
      time: t,
      active: native.active_actor_index,
      actors,
      targets,
      damage: Number(damage),
      hits: native.hits_end,
      last,
      count,
      native,
    };
  };
  return { request, report, clocks, horizon, events, segments, stateAt };
}
export type ReplayModel = ReturnType<typeof createReplay>;
export type PlaybackState = ReturnType<ReplayModel['stateAt']>;
