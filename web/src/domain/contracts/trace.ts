import type { ExactFraction } from './ratio';

export interface OrdinaryHitEnergyBasis {
  base: ExactFraction;
  distribution: ExactFraction;
}

export type EnergySource =
  | 'mind_royale'
  | 'mint_a6'
  | 'mint_hit'
  | 'sakiri_hit'
  | 'zero_hit'
  | 'iroi_hit'
  | 'nanally_hit'
  | 'chaos_hit'
  | 'edgar_hit'
  | 'hotori_hit'
  | 'daffodil_hit'
  | 'blackbird_hit'
  | 'aurelia_hit'
  | 'haniel_hit'
  | 'linko_hit'
  | 'skia_hit'
  | 'hathor_hit'
  | 'hathor_power'
  | 'jiuyuan_hit'
  | 'jiuyuan_bud_boom'
  | 'baicang_hit'
  | 'fadia_hit'
  | 'fadia_threshold'
  | 'zankou_hit'
  | 'chiz_hit'
  | 'adler_hit'
  | 'lacrimosa_hit'
  | 'shinku_hit'
  | 'zero_a2'
  | 'nanally_a1'
  | 'sakiri_a5'
  | 'adler_a5'
  | 'rishi_eclipse'
  | 'reaction_charge';

export interface EnergyGrant {
  source: EnergySource;
  source_actor_index: number;
  recipient_actor_index: number;
  energy_before: number;
  gross: number;
  retained: number;
  overflow: number;
  energy_after: number;
  charge_remainder_before?: ExactFraction;
  charge_remainder_after?: ExactFraction;
  charge_basis?: ExactFraction;
  hit_basis?: OrdinaryHitEnergyBasis;
}

export interface EnergyFanoutRecipient {
  recipient_actor_index: number;
  distribution?: ExactFraction;
  energy_before: number;
  gross: number;
  retained: number;
  overflow: number;
  energy_after: number;
  charge_remainder_before?: ExactFraction;
  charge_remainder_after?: ExactFraction;
  charge_basis?: ExactFraction;
}

export interface EnergyFanout {
  source: EnergySource;
  source_actor_index: number;
  base?: ExactFraction;
  recipients: EnergyFanoutRecipient[];
}

export interface EnergySpend {
  source: 'ability_cost';
  source_actor_index: number;
  recipient_actor_index: number;
  command_index: number;
  ability_id: string;
  energy_before: number;
  spent: number;
  energy_after: number;
  charge_remainder_before?: ExactFraction;
  charge_remainder_after?: ExactFraction;
}

export type EnergyChange = EnergyGrant | EnergySpend;

export interface ResourceDepositReceipt {
  input: ExactFraction;
  repaid: ExactFraction;
  credited: ExactFraction;
  overflow: ExactFraction;
}

export type DummyCombatEvent =
  | {
      kind: 'attack_start';
      timestamp_us: number;
      target_index: number;
      actor_index: number;
    }
  | {
      kind: 'proximity';
      timestamp_us: number;
      target_index: number;
      in_proximity: boolean;
    };

export interface TraceEventPayload {
  ability_id?: string;
  damage_effect_id?: string;
  utility_id?: string;
  buff_id?: string;
  shield_id?: string;
  packet_group_id?: string;
  input?: string;
  attack_mode?: string;
  source?: string;
  reaction?: string;
  element?: string;
  reason?: string;
  cause?: string;
  condition?: string;
  outcome?: string;
  mark?: string;
  action?: string;
  lamb_type?: string;
  dummy_event?: DummyCombatEvent;
  target_index?: number;
  to_target_index?: number;
  next_target_index?: number;
  command_index?: number;
  event_index?: number;
  parent_hit_index?: number;
  slot?: number;
  ordinal?: number;
  stacks?: number;
  stacks_before?: number;
  max_stacks?: number;
  level?: number;
  count?: number;
  consumed?: number;
  remaining?: number;
  generation?: number;
  discord_generation?: number;
  scorch_generation?: number;
  nova_generation?: number;
  bud_id?: number;
  word_generation?: number;
  price_index?: number;
  price_pct?: number;
  recipients?: number[];
  resource_deposits?: Record<string, ResourceDepositReceipt>;
  duration_us?: number;
  expires_at_us?: number;
  completes_at_us?: number;
  cooldown_until_us?: number;
  recovery_before_us?: number;
  recovery_after_us?: number;
  leave_grant_due_us?: number;
  phase_origin_us?: number;
  delay_us?: number;
  rate_ppm?: number;
  admitted?: boolean;
  active?: boolean;
  all?: boolean;
  full?: boolean;
  leaving?: boolean;
  enhanced?: boolean;
  copied?: boolean;
  fixed?: boolean;
  defeated?: boolean;
  death?: boolean;
  launchable?: boolean;
  first_crit?: boolean;
  critical?: boolean;
  field_cancelled?: boolean;
  target_alive?: boolean;
  cancelled?: boolean;
  amount?: number | string;
  threshold?: number | string;
  gross?: number | string;
  applied?: number | string;
  overflow?: number | string;
  hp_before?: number | string;
  hp_after?: number | string;
  hp_max?: number | string;
  hp_max_before?: number | string;
  delivered_damage?: number | string;
  absorbed_by_shields?: number | string;
  absorbed_by_effects?: number | string;
  energy?: number | string;
  pool?: number | string;
  atk_add?: ExactFraction;
  atk_percent?: ExactFraction;
  def_percent?: ExactFraction;
  def_ignore?: ExactFraction;
  crit_rate_bonus?: ExactFraction;
  crit_damage_bonus?: ExactFraction;
  damage_bonus?: ExactFraction;
  mental_damage_bonus?: ExactFraction;
  atk_ratio?: ExactFraction;
  exact_amount?: ExactFraction;
  exact_before?: ExactFraction;
  exact_after?: ExactFraction;
  exact_delta?: ExactFraction;
  exact_raw?: ExactFraction;
  exact_scaled?: ExactFraction;
  exact_carry?: ExactFraction;
  exact_efficiency?: ExactFraction;
  exact_capacity?: ExactFraction;
  exact_overflow?: ExactFraction;
  wide_basis?: ExactFraction;
  wide_amount?: ExactFraction;
  wide_overflow?: ExactFraction;
}

export interface TraceEvent {
  time_us: number;
  phase: string;
  event_kind: string;
  active_actor: number;
  hit_index?: number;
  from_actor?: number;
  to_actor?: number;
  energy_change?: EnergyChange;
  energy_fanout?: EnergyFanout;
  data?: TraceEventPayload;
}
