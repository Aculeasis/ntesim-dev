import type { ExactFraction } from './ratio';

export interface RejectedAction {
  command_index: number;
  time_us: number;
  actor_index: number;
  reason:
    | { kind: 'actor_not_active'; active_actor_index: number }
    | { kind: 'recovery' | 'cooldown'; until_us: number }
    | { kind: 'energy'; required: number; available: number }
    | { kind: 'stamina'; required: ExactFraction; available: ExactFraction }
    | { kind: 'time_reversal'; last_us: number }
    | { kind: 'timing_overflow' }
    | { kind: 'other'; message: string };
}
