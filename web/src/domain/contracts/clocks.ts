export const CLOCK_CONTRACT = 'relative-combat-v1' as const;
export const TIMER_POLICY = 'paused-world-v1' as const;

export interface TimeBasisPair<T> {
  relative: T | null;
  absolute: T | null;
}

export interface PauseOwner {
  owner: number;
  generation: number;
}
export interface TimerPause {
  owner: PauseOwner;
  kind: 'cinematic' | 'active_stop';
  start_absolute_us: number;
  end_absolute_us: number;
}
export type ClockDomain =
  | { kind: 'absolute' | 'world' | 'object_lifetime' }
  | { kind: 'autonomous_activity'; advance_during_active_stop: boolean }
  | { kind: 'replay_action'; context: PauseOwner }
  | { kind: 'action'; context: PauseOwner; advance_during_active_stop: boolean };
