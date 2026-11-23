import {
  CLOCK_CONTRACT,
  TIMER_POLICY,
  type ClockDomain,
  type TimerPause,
} from '../contracts/clocks';
import type { ScenarioReport } from '../contracts/report';

type Interval = readonly [number, number];
function union(pauses: TimerPause[]): Interval[] {
  const sorted = pauses
    .map((p) => [p.start_absolute_us, p.end_absolute_us] as [number, number])
    .sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  const result: [number, number][] = [];
  for (const [start, end] of sorted) {
    const previous = result[result.length - 1];
    if (previous && start <= previous[1]) previous[1] = Math.max(previous[1], end);
    else result.push([start, end]);
  }
  return result;
}

class IntervalIndex {
  readonly intervals: Interval[];
  private readonly prefix: number[];
  constructor(pauses: TimerPause[]) {
    this.intervals = union(pauses);
    let total = 0;
    this.prefix = this.intervals.map(([start, end]) => (total += end - start));
  }
  private preceding(time: number): number {
    let low = 0,
      high = this.intervals.length;
    while (low < high) {
      const middle = (low + high) >>> 1;
      if (this.intervals[middle][0] <= time) low = middle + 1;
      else high = middle;
    }
    return low - 1;
  }
  pausedThrough(time: number): number {
    const index = this.preceding(time);
    if (index < 0) return 0;
    const [start, end] = this.intervals[index];
    return (this.prefix[index - 1] ?? 0) + Math.max(0, Math.min(time, end) - start);
  }
  pausedAt(time: number): boolean {
    const index = this.preceding(time);
    return index >= 0 && time < this.intervals[index][1];
  }
}

/** Presentation indexes native pause records; it never identifies abilities or
 * infers a pause from an actor name. Playback geometry stays absolute. */
export class ClockIndex {
  private readonly world: IntervalIndex;
  private readonly cinematic: IntervalIndex;
  readonly absoluteEnd: number;
  readonly intervals: Interval[];
  constructor(report: ScenarioReport) {
    if (report.clock_contract !== CLOCK_CONTRACT || report.timer_policy !== TIMER_POLICY)
      throw new Error('Unsupported combat clock contract or timer policy.');
    const integer = (value: number) => Number.isSafeInteger(value) && value >= 0;
    if (
      !integer(report.absolute_elapsed_us) ||
      !integer(report.relative_elapsed_us) ||
      !integer(report.absolute_limit_us) ||
      report.absolute_limit_us <= 0 ||
      report.absolute_elapsed_us > report.absolute_limit_us ||
      report.relative_elapsed_us > report.absolute_elapsed_us ||
      !integer(report.relative_duration_us) ||
      report.relative_duration_us <= 0 ||
      !Array.isArray(report.final_actor_states) ||
      !Array.isArray(report.timer_pauses) ||
      !Array.isArray(report.timer_catalog)
    )
      throw new Error('Invalid or missing native combat clock observations.');
    const definitions = new Set<string>();
    for (const timer of report.timer_catalog) {
      const identity = JSON.stringify([timer.owner, timer.key]);
      if (
        (timer.owner !== null &&
          (!integer(timer.owner) || timer.owner >= report.final_actor_states.length)) ||
        typeof timer.key !== 'string' ||
        !timer.key ||
        !['world', 'object_lifetime'].includes(timer.clock_basis?.kind) ||
        definitions.has(identity)
      )
        throw new Error('Invalid native timer catalog.');
      definitions.add(identity);
    }
    const owners = new Map<string, number>();
    let start = 0;
    for (const pause of report.timer_pauses) {
      const key = `${pause.owner?.owner}:${pause.owner?.generation}`;
      if (
        !integer(pause.start_absolute_us) ||
        !integer(pause.end_absolute_us) ||
        pause.start_absolute_us >= pause.end_absolute_us ||
        pause.end_absolute_us > report.absolute_elapsed_us ||
        pause.start_absolute_us < start ||
        !integer(pause.owner?.owner) ||
        pause.owner.owner >= report.final_actor_states.length ||
        !integer(pause.owner.generation) ||
        (pause.kind !== 'cinematic' && pause.kind !== 'active_stop') ||
        (owners.get(key) ?? 0) > pause.start_absolute_us ||
        !report.timer_bindings?.[pause.owner.owner]
      )
        throw new Error('Invalid native timer-pause interval or ownership.');
      start = pause.start_absolute_us;
      owners.set(key, pause.end_absolute_us);
    }
    this.absoluteEnd = report.absolute_elapsed_us;
    this.world = new IntervalIndex(report.timer_pauses);
    this.cinematic = new IntervalIndex(report.timer_pauses.filter((p) => p.kind === 'cinematic'));
    this.intervals = this.world.intervals;
    if (this.relativeAt(this.absoluteEnd) !== report.relative_elapsed_us)
      throw new Error('Relative duration plus union pause does not equal absolute duration.');
    if (
      typeof report.game_dps_computed !== 'boolean' ||
      ![null, 'zero_relative_duration', 'incomplete_execution', 'unassessed_model'].includes(
        report.dps_unavailable_reason,
      ) ||
      (report.relative_elapsed_us === 0 &&
        (report.game_dps_computed || report.dps_unavailable_reason !== 'zero_relative_duration')) ||
      report.game_dps_computed !== (report.dps_unavailable_reason === null)
    )
      throw new Error('Invalid relative DPS availability for the native clock observation.');
  }
  private cursor(absoluteUs: number): number {
    if (!Number.isFinite(absoluteUs)) throw new Error('Invalid absolute playback cursor.');
    return Math.max(0, Math.min(absoluteUs, this.absoluteEnd));
  }
  relativeAt(absoluteUs: number): number {
    const cursor = this.cursor(absoluteUs);
    return cursor - this.world.pausedThrough(cursor);
  }
  objectAt(absoluteUs: number): number {
    const cursor = this.cursor(absoluteUs);
    return cursor - this.cinematic.pausedThrough(cursor);
  }
  elapsedAt(domain: ClockDomain, absoluteUs: number): number {
    switch (domain.kind) {
      case 'absolute':
        return this.cursor(absoluteUs);
      case 'world':
        return this.relativeAt(absoluteUs);
      case 'object_lifetime':
        return this.objectAt(absoluteUs);
      case 'replay_action':
        return this.objectAt(absoluteUs);
      case 'autonomous_activity':
        return domain.advance_during_active_stop
          ? this.objectAt(absoluteUs)
          : this.relativeAt(absoluteUs);
      case 'action':
        throw new Error('Action timer inspection requires native action-clock observations.');
    }
  }
  pausedAt(absoluteUs: number): boolean {
    return this.world.pausedAt(this.cursor(absoluteUs));
  }
}
