import type { HpCostEvent } from './vitals';
import type { CombatStaminaProfile } from './stamina';
import type { StochasticCrit, ActionMetadata } from './research';
export type { StochasticCrit, ActionMetadata } from './research';
import type { Target } from './target';
export type { Target } from './target';

import type { Ratio } from './ratio';
export type { Ratio, ExactFraction } from './ratio';
export type NativeScalingStat =
  'atk' | 'def' | 'hp' | { mixed: { atk: Ratio; def: Ratio; hp: Ratio } };
export interface StaticStats {
  base_atk: number;
  atk: Ratio;
  hp: number;
  base_hp?: number;
  hp_percent?: Ratio;
  hp_flat?: Ratio;
  def: number;
  base_def?: number;
  def_percent?: Ratio;
  def_flat?: Ratio;
  crit_rate: Ratio;
  crit_damage_bonus: Ratio;
  healing_bonus?: Ratio;
  shield_bonus?: Ratio;
  break_intensity?: Ratio;
  cycle_intensity?: Ratio;
  break_efficiency_multiplier?: Ratio;
  mental_damage_bonus?: Ratio;
  elemental_bonus: Ratio;
  damage_bonus?: Ratio;
  elemental_bonuses?: Partial<
    Record<'CHAOS' | 'COSMOS' | 'INCANTATION' | 'LAKSHANA' | 'NATURE' | 'PSYCHE', Ratio>
  >;
}
export type GearProc =
  | {
      kind: 'worldrain_damage';
      damage_bonus: Ratio;
      cycle_intensity_bonus: Ratio;
      duration_us: number;
    }
  | { kind: 'jiaojuan_break'; break_intensity_bonus: Ratio; broken_damage_bonus: Ratio }
  | { kind: 'snowman_shield_atk'; atk_bonus: Ratio }
  | { kind: 'yuren_hp'; hp_bonus: Ratio; shield_bonus: Ratio }
  | { kind: 'tuansanlang_team'; break_intensity_bonus: Ratio }
  | { kind: 'yaodao_parry'; parry_damage_ratio: Ratio }
  | { kind: 'thief_candy_dodge'; damage_bonus: Ratio; duration_us: number }
  | {
      kind: 'dustbin_break';
      break_intensity_bonus: Ratio;
      duration_us: number;
      cooldown_us: number;
    }
  | { kind: 'police_rat_boss'; atk_bonus: Ratio; boss_damage_bonus: Ratio }
  | {
      kind: 'rose_thorn';
      atk_bonus: Ratio;
      crit_damage_per_stack: Ratio;
      duration_us: number;
      broken_extension_us: number;
    }
  | {
      kind: 'tiger_tally_atk';
      atk_bonus: Ratio;
      normal_damage_bonus: Ratio;
      duration_us: number;
      talisman_window_us: number;
      boss_damage_bonus: Ratio;
      boss_damage_duration_us: number;
    }
  | {
      kind: 'nakupeda';
      hp_bonus: Ratio;
      lowest_hp_heal_ratio: Ratio;
      shield_ratio: Ratio;
      shield_duration_us: number;
      team_heal_ratio: Ratio;
      cooldown_us: number;
    }
  | { kind: 'black_book_break'; break_intensity_bonus: Ratio }
  | { kind: 'jingmotingyuan_crit'; crit_damage_bonus: Ratio; duration_us: number }
  | { kind: 'rishi_atk'; atk_bonus: Ratio }
  | {
      kind: 'mamen_fons';
      cosmos_bonus_per_stack: Ratio;
      fons_per_stack: number;
      max_stacks: number;
    }
  | {
      kind: 'butterfly_damage';
      anima_bonus: Ratio;
      attachment_bonus: Ratio;
      ult_attachment_bonus: Ratio;
      ult_duration_us: number;
    }
  | { kind: 'kite_atk'; atk_bonus: Ratio; lakshana_bonus: Ratio; duration_us: number }
  | {
      kind: 'whale_atk';
      atk_bonus: Ratio;
      broken_damage_bonus: Ratio;
      kill_heal: Ratio;
      kill_heal_cooldown_us: number;
    }
  | { kind: 'crowbar_break'; break_intensity_bonus: Ratio; duration_us: number }
  | { kind: 'nest_bird_gaze'; damage_reduction: Ratio; duration_us: number }
  | { kind: 'noros_atk'; atk_bonus: Ratio; duration_us: number; cooldown_us: number }
  | { kind: 'gold_wool'; lakshana_bonus: Ratio; crit_damage_bonus: Ratio; duration_us: number }
  | { kind: 'motor_candy_atk'; atk_per_stack: Ratio; interval_us: number }
  | { kind: 'shadow_creed_atk'; atk_bonus: Ratio; duration_us: number }
  | {
      kind: 'quiet_manor_mental';
      mental_bonus_per_stack: Ratio;
      duration_us: number;
      max_stacks: number;
    }
  | { kind: 'knight_candy_crit_damage'; crit_damage_per_stack: Ratio; duration_us: number }
  | { kind: 'oraora_basic_attack_damage'; damage_bonus_per_stack: Ratio; duration_us: number }
  | { kind: 'appliance_damage'; damage_bonus: Ratio }
  | { kind: 'prokaryon_damage'; damage_bonus: Ratio }
  | { kind: 'paper_plane_damage'; damage_bonus: Ratio }
  | { kind: 'boxing_candy_damage'; damage_bonus: Ratio; low_hp_damage_bonus: Ratio }
  | { kind: 'forgotten_hp_stats'; atk_bonus: Ratio; def_bonus: Ratio }
  | { kind: 'blast_candy_atk'; atk_bonus: Ratio; duration_us: number }
  | { kind: 'charge_efficiency'; multiplier: Ratio }
  | { kind: 'speedy_hedgehog_team_atk'; team_atk_bonus: Ratio; duration_us: number }
  | {
      kind: 'good_boy_team_atk';
      team_atk_bonus: Ratio;
      duration_us: number;
      control_atk_bonus?: Ratio;
    }
  | { kind: 'mind_royale_energy'; energy_grant: number; cooldown_us: number }
  | { kind: 'lost_radiance_def_ignore'; def_ignore: Ratio; duration_us: number }
  | {
      kind: 'twinbirds';
      psyche_bonus: Ratio;
      team_crit_damage_bonus: Ratio;
      self_crit_damage_bonus: Ratio;
      duration_us: number;
    }
  | { kind: 'eternal_waltz'; max_hp_bonus: Ratio; mental_bonus: Ratio; duration_us: number }
  | {
      kind: 'raging_flames';
      lakshana_bonus: Ratio;
      redirect_ultimate_bonus: Ratio;
      redirect_skill_stack_bonus: Ratio;
      duration_us: number;
    }
  | {
      kind: 'far_side_moon';
      crit_rate_bonus: Ratio;
      skill_damage_bonus: Ratio;
      basic_attack_damage_bonus: Ratio;
      lakshana_damage_bonus: Ratio;
      duration_us: number;
    }
  | {
      kind: 'lunar_phase';
      atk_bonus: Ratio;
      cosmos_bonus: Ratio;
      def_ignore: Ratio;
      duration_us: number;
    }
  | {
      kind: 'demon_blade';
      crit_rate_bonus: Ratio;
      crit_damage_per_stack: Ratio;
      duration_us: number;
      icd_us: number;
    }
  | {
      kind: 'moon';
      psyche_bonus: Ratio;
      crit_damage_per_stack: Ratio;
      duration_us: number;
      icd_us: number;
    }
  | {
      kind: 'fireflies_crit_damage';
      crit_damage_per_stack: Ratio;
      max_stacks: number;
      duration_us: number;
    }
  | {
      kind: 'bopu_damage';
      damage_bonus: Ratio;
      duration_us: number;
      cooldown_us: number;
    }
  | {
      kind: 'koinobori_stats';
      atk_bonus: Ratio;
      def_bonus: Ratio;
      hp_bonus: Ratio;
    }
  | {
      kind: 'gold_record';
      atk_bonus: Ratio;
      qte_crit_damage_bonus: Ratio;
      ult_crit_damage_per_stack: Ratio;
      duration_us: number;
    }
  | {
      kind: 'spider_team_atk';
      atk_bonus_per_stack: Ratio;
      atk_bonus_max_stack: Ratio;
      duration_us: number;
    }
  | {
      kind: 'bit_game';
      off_field_atk_base: Ratio;
      off_field_atk_per_stack: Ratio;
      off_field_icd_us: number;
      on_field_psyche_base: Ratio;
      on_field_psyche_per_stack: Ratio;
    }
  | {
      kind: 'marching_beyond_time';
      atk_bonus: Ratio;
      state_crit_damage_bonus: Ratio;
      crit_damage_per_stack: Ratio;
      def_ignore: Ratio;
      def_ignore_duration_us: number;
    }
  | {
      kind: 'street_boxer_crit';
      crit_rate_bonus: Ratio;
      powered_crit_rate_bonus: Ratio;
      duration_us: number;
    }
  | { kind: 'be_happy'; heal_ratio: Ratio; cooldown_us: number }
  | { kind: 'bitter_cake'; def_bonus: Ratio; duration_us: number; cooldown_us: number }
  | { kind: 'castle_healing_bonus'; healing_bonus: Ratio; duration_us: number }
  | {
      kind: 'hp_suit_max_hp';
      max_hp_bonus_per_stack: Ratio;
      duration_us: number;
      max_stacks: number;
    }
  | {
      kind: 'crimson_atk';
      atk_bonus_per_stack: Ratio;
      duration_us: number;
      max_stacks: number;
    };
export interface Actor {
  slot: number;
  character_id: string;
  character_level: number;
  awakening: number[];
  purchased_skill_level: number;
  purchased_qte_level?: number;
  talent_levels?: [number, number, number];
  character_max_level?: number;
  initial_hp: number;
  initial_hp_max: number;
  initial_energy: number;
  energy_capacity: number;
  /** Rust validates owner-specific resource IDs, units and capacities. */
  initial_resources?: Record<string, Ratio>;
  weapon_id: string;
  weapon_tier: number;
  set_id: string | null;
  static_stats: StaticStats;
  gear_procs: GearProc[];
}
export interface InvokeAction {
  kind: 'invoke_ability';
  timestamp_us: number;
  actor_index: number;
  ability_id: string;
  array_index: number;
  damage_effect_id: string;
  atk_rate: Ratio;
  scaling_stat?: NativeScalingStat;
  energy_cost: number;
  recovery_us: number;
  cooldown_us: number;
  hit_delay_us: number;
}
export type Action =
  | InvokeAction
  | {
      kind: 'player_input';
      timestamp_us: number;
      actor_index: number;
      input: 'special' | 'dash' | 'arc';
      /** Older serialized requests omit this flag; the runner treats omission as false. */
      critical_success?: boolean;
    }
  | { kind: 'switch_actor'; timestamp_us: number; target_actor_index: number; cooldown_us: number }
  | { kind: 'wait'; timestamp_us: number; duration_us: number };
export interface ScenarioRequest {
  clock_contract: 'relative-combat-v1';
  timer_policy: 'paused-world-v1';
  absolute_limit_us: number;
  combat_stamina?: CombatStaminaProfile;
  schema_version: number;
  content_id: string;
  relative_duration_us: number;
  stop?: 'time' | 'victory';
  start_fons?: number;
  seed: number;
  action_admission_order?: 'scheduled' | 'after_settled_events';
  active_actor_index: number;
  actors: Actor[];
  targets: Target[];
  incoming_damage_events?: ActorDamageEvent[];
  hp_cost_events?: HpCostEvent[];
  actions: Action[];
  stochastic_crit?: StochasticCrit;
  model_assumptions?: Record<string, string>;
  action_metadata?: Record<string, ActionMetadata>;
}
export type LiveBranchCondition =
  | { kind: 'target_hp_below'; target_index: number; threshold: number }
  | { kind: 'active_actor_energy_below'; threshold: number };
export interface LiveBranch {
  timestamp_us: number;
  after_action_index: number;
  condition: LiveBranchCondition;
  wait_if_below_us: number;
  wait_otherwise_us: number;
  source_line: number;
  source_column: number;
}
export interface LiveEnergyWait {
  timestamp_us: number;
  after_action_index: number;
  actor_index: number;
  energy_at_least: number;
  source_line: number;
  source_column: number;
}
export interface LiveProgramDeclarations {
  branches: LiveBranch[];
  energy_waits: LiveEnergyWait[];
}
export interface ScenarioProgramRequest extends LiveProgramDeclarations {
  request: ScenarioRequest;
}
export interface ActorDamageEvent {
  timestamp_us: number;
  actor_index: number;
  delivered_damage: number;
}
