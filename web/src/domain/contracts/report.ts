import type { HitReceipt } from './hit';
export type { HitReceipt } from './hit';
import type { AutonomousEventReceipt } from './autonomous';
export type { AutonomousEventReceipt, CausalTrigger } from './autonomous';
import type { CombatStaminaReport } from './stamina';
import type { RejectedAction } from './rejection';
import type { InputActionRole } from './action-role';
export type { InputActionRole } from './action-role';
import type {
  ExactFraction,
  LiveBranchCondition,
  NativeScalingStat,
  Ratio,
  StochasticCrit,
} from './scenario';
import type { ActorDamageReceipt, HpCostReceipt } from './vitals';
import type { SwitchOrigin } from './effect-origin';
export type { SwitchOrigin } from './effect-origin';
import type {
  DummyCombatEvent,
  EnergyChange,
  EnergyFanout,
  EnergyFanoutRecipient,
  EnergyGrant,
  EnergySource,
  EnergySpend,
  OrdinaryHitEnergyBasis,
  ResourceDepositReceipt,
  TraceEvent,
  TraceEventPayload,
} from './trace';
export type {
  DummyCombatEvent,
  EnergyChange,
  EnergyFanout,
  EnergyFanoutRecipient,
  EnergyGrant,
  EnergySource,
  EnergySpend,
  OrdinaryHitEnergyBasis,
  ResourceDepositReceipt,
  TraceEvent,
  TraceEventPayload,
} from './trace';
export interface NativeEvidence {
  content_id: string;
  table: string;
  row: string;
  pointer: string;
  row_sha256: string;
}
export type MetadataProvenance =
  | { kind: 'native'; evidence_id: number }
  | { kind: 'accepted_model_hypothesis'; hypothesis_id: string };
export type ActionCategory = 'melee' | 'skill' | 'ultra_skill' | 'qte' | 'other';
export type DamageKind = 'elemental' | 'mental' | 'true';
export type DamageElement =
  'physical' | 'cosmos' | 'nature' | 'chaos' | 'incantation' | 'lakshana' | 'psyche';
export interface ActionMetadataReceipt {
  categories: ActionCategory[];
  gameplay_tags?: string[];
  ability_id: string;
  native_ability_index?: number | null;
  input_role?: InputActionRole | null;
  input_role_provenance?: MetadataProvenance | null;
  primary_slot?: 'normal' | 'skill' | 'ultimate' | null;
  source_evidence_id?: number | null;
  identity_evidence_id?: number | null;
  provenance: MetadataProvenance;
}
export interface PacketMetadataReceipt {
  damage_effect_id: string;
  native_ga_name?: string | null;
  native_damage_type_ex: string;
  native_damage_source_category?: string | null;
  damage_kind: DamageKind;
  element?: DamageElement | null;
  fixed_crit_rate?: ExactFraction | null;
  fixed_crit_rate_provenance?: MetadataProvenance | null;
  unbal_value?: ExactFraction | null;
  unbal_value_provenance?: MetadataProvenance | null;
  semantic_tags?: string[];
  semantic_provenance?: MetadataProvenance | null;
  source_evidence_id?: number | null;
  provenance: MetadataProvenance;
  packet_kind?: 'standard' | 'separate_parry' | 'arc_chain';
}
export interface ActionPacketMetadataReceipt {
  action: ActionMetadataReceipt;
  packet: PacketMetadataReceipt;
  binding: {
    kind: 'native_direct' | 'owner_child' | 'research_declared';
    initiating_ability_id: string;
    packet_ga_name?: string | null;
    provenance?: MetadataProvenance | null;
  };
}
export interface ActionMetadataOccurrence {
  switch_origin?: SwitchOrigin;
  execution_model_assumption_ids?: string[];
  command_indices: number[];
  actor_index: number;
  ability_id: string;
  damage_effect_id: string;
  metadata_id: number;
  energy_group_id?: string;
}
export interface ActorState {
  cycle?: {
    initial: number;
    current: number;
    capacity: number;
    granted: number;
    spent: number;
    overflow: number;
    carry: ExactFraction;
    conserved: boolean;
  };
  resources?: Record<string, unknown>;
  fractional_resources?: Record<string, unknown>;
  slot: number;
  character_id: string;
  attack_mode: 'normal' | 'aim' | 'ranged' | 'melee';
  hp: number;
  hp_max: number;
  energy: number;
  energy_capacity: number;
  fireflies_stacks: number;
  active_fireflies_stacks: unknown[];
  energy_conserved: boolean;
  initial_energy: number;
  granted_energy: number;
  spent_energy: number;
  overflow_energy: number;
  charge_remainder?: ExactFraction;
  active_shields?: ActorShieldState[];
}
export interface ActorShieldState {
  clock_basis: import('./clocks').ClockDomain;
  id: string;
  source: string;
  remaining_hp: number;
  max_hp: number;
  expires_at_us: number;
}
export interface RngDraw {
  stream: string;
  index: number;
  value: number;
}
export interface RngSample {
  bound: number;
  value: number;
  draws: RngDraw[];
}
export interface RngAudit {
  version: string;
  seed: number;
  streams: Record<string, { sequence: number | string; state: number | string; draws: number }>;
}
export interface PlayerInputReceipt {
  command_index: number;
  actor_index: number;
  ability_id?: string;
  time_us: number;
  input: 'special' | 'dash' | 'arc';
  /** Older serialized receipts omit this flag; validation treats omission as false. */
  critical_success?: boolean;
  attack_mode_before: ActorState['attack_mode'];
  attack_mode_after: ActorState['attack_mode'];
  execution_model_assumption_ids?: string[];
}
export interface BuffState {
  time_us: number;
  active_actor_index: number;
  instances: import('./timeline').ActiveBuffRef[];
}
export interface ScenarioReport {
  clock_contract: 'relative-combat-v1';
  timer_policy: 'paused-world-v1';
  absolute_limit_us: number;
  relative_elapsed_us: number;
  timer_pauses?: import('./clocks').TimerPause[];
  timer_bindings: Record<number, string>;
  timer_catalog?: {
    owner: number | null;
    key: string;
    clock_basis: import('./clocks').ClockDomain;
  }[];
  dps_unavailable_reason:
    'zero_relative_duration' | 'incomplete_execution' | 'unassessed_model' | null;
  state_timeline: import('./timeline').StateFrame[];
  healing_receipts: import('./timeline').HealingReceipt[];
  input_time: unknown;
  combat_stamina?: CombatStaminaReport;
  hp_cost_receipts?: HpCostReceipt[];
  incoming_damage_receipts?: ActorDamageReceipt[];
  player_input_receipts?: PlayerInputReceipt[];
  rejected_action?: RejectedAction;
  talent_projection_receipts?: {
    command_indices: number[];
    compiled_array_index: number;
    compiled_atk_rate: Ratio;
    compiled_scaling_stat?: NativeScalingStat;
    compiled_rate?: Ratio;
    skill_level: { effective_level: number; purchase: { purchased_level: number } };
  }[];
  schema_version: number;
  content_id: string;
  status: string;
  quality: 'confirmed' | 'bounded_hypothesis' | 'unbounded_hypothesis' | 'incomplete' | 'synthetic';
  game_dps_computed: boolean;
  success: boolean;
  execution_completed: boolean;
  pending_events: number;
  relative_duration_us: number;
  absolute_elapsed_us: number;
  available_shield_hp: import('./clocks').TimeBasisPair<number>;
  seed: number;
  stochastic_crit?: StochasticCrit;
  model_assumptions?: Record<string, string>;
  resolved_model_assumptions?: Record<string, string>;
  callback_model_assumptions?: Record<string, string>;
  evidence_catalog?: NativeEvidence[];
  metadata_catalog?: ActionPacketMetadataReceipt[];
  buff_catalog?: import('./timeline').BuffCatalogEntry[];
  rng?: RngAudit;
  reaction_rng?: RngAudit;
  proc_rng?: { actor_index: number; rng: RngAudit }[];
  proc_selection_receipts?: {
    time_us: number;
    actor_index: number;
    stream: string;
    sample: RngSample;
    outcome_index: number;
    cooldown_until_us: number;
  }[];
  invariants_passed: boolean;
  total_events_processed: number;
  hits_settled: number;
  max_proc_depth_reached: number;
  total_damage_dealt: number;
  total_damage_exact: ExactFraction;
  hit_receipts: HitReceipt[];
  autonomous_events?: AutonomousEventReceipt[];
  live_branch_receipts?: {
    time_us: number;
    condition: LiveBranchCondition;
    observer_actor_index: number;
    observed_value: number;
    selected_wait_us: number;
    baseline_wait_us: number;
    after_action_index: number;
    source_line: number;
    source_column: number;
  }[];
  live_energy_wait_receipts?: {
    started_time_us: number;
    actor_index: number;
    energy_at_start: number;
    energy_at_least: number;
    resumed_time_us: number | null;
    energy_at_resume: number | null;
    after_action_index: number;
    source_line: number;
    source_column: number;
  }[];
  action_metadata_ledger?: ActionMetadataOccurrence[];
  final_actor_states: ActorState[];
  final_buff_state: BuffState;
  final_target_states: {
    control_immune?: boolean;
    base_defense?: number;
    target_index: number;
    is_boss: boolean;
    hp_current: number;
    hp_max: number;
    defense: number;
    airborne_until_us?: number;
    statuses?: {
      clock_basis: import('./clocks').ClockDomain;
      source_actor_index: number;
      id: string;
      expires_at_us: number;
      semantic_tags?: string[];
      stacks: number;
      resistance_reductions: Record<string, ExactFraction>;
    }[];
  }[];
  trace: TraceEvent[];
  halt_diagnostic?: string;
  engine_version: string;
  engine_source_sha256: string;
}
