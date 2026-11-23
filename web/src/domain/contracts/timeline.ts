import type { ActorShieldState, ActorState, ScenarioReport } from './report';
import type { ExactFraction, Ratio } from './ratio';

export type BuffRecipient =
  | { type: 'self_actor' | 'active_actor' | 'team_all' }
  | { type: 'specific_actor'; target: number }
  | { type: 'specific_target'; target: string | number };

export type BuffDuration =
  { kind: 'permanent' } | { kind: 'timed'; duration_us: number; expires_at_us: number };

export type BuffRefreshPolicy =
  | { policy: 'reset' | 'independent' | 'disallow' }
  | { policy: 'extend'; max_duration_us: number | null };

export interface BuffCatalogEntry {
  clock_basis: import('./clocks').ClockDomain;
  id: string;
  source: string;
  owner_actor: number;
  recipient: BuffRecipient;
  stacking_scope?: 'per_source' | 'shared';
  source_lifetime?: 'snapshot' | 'while_alive';
  semantic_tags?: string[];
  refresh_policy: BuffRefreshPolicy;
  max_stacks: number;
  decay_policy: 'all_at_once' | 'one_by_one';
  stat_modifiers: Record<string, Ratio>;
}

export interface ActiveBuffRef {
  instance_id: number;
  buff_def_id: number;
  stacks: number;
  duration: BuffDuration;
}

export interface BuffTimelineDiff {
  upsert?: ActiveBuffRef[];
  remove?: number[];
}

export interface ActorStateDelta {
  slot: number;
  character_id?: string;
  attack_mode?: ActorState['attack_mode'];
  hp?: number;
  hp_max?: number;
  energy?: number;
  energy_capacity?: number;
  active_shields?: ActorShieldState[];
  fireflies_stacks?: number;
  active_fireflies_stacks?: unknown[];
  energy_conserved?: boolean;
  initial_energy?: number;
  granted_energy?: number;
  spent_energy?: number;
  overflow_energy?: number;
  charge_remainder?: ExactFraction | null;
  cycle?: ActorState['cycle'] | null;
  resources?: Record<string, unknown>;
  fractional_resources?: Record<string, unknown>;
}

export interface StateFrame {
  timers?: { timer_def_id: number; expires_at_us: number }[] | null;
  time_us: number;
  boundary: { kind: 'initial' | 'final' } | { kind: 'event'; ordinal: number };
  active_actor_index: number;
  actors?: ActorStateDelta[];
  targets?: ScenarioReport['final_target_states'];
  buffs?: ActiveBuffRef[] | null;
  buff_diff?: BuffTimelineDiff | null;
  stamina?: {
    current: ExactFraction;
    spent: ExactFraction;
    recovered: ExactFraction;
    recovery_ready_at_us: number;
  } | null;
  hits_end: number;
  healing_end: number;
  trace_end: number;
}
export interface HealingReceipt {
  time_us: number;
  source_actor_index: number;
  recipient_actor_index: number;
  base: ExactFraction;
  outgoing_bonus: ExactFraction;
  incoming_bonus: ExactFraction;
  gross: number;
  applied: number;
  excess: number;
  hp_before: number;
  hp_after: number;
}
