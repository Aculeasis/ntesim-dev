import type { ExactFraction, NativeScalingStat, Ratio } from './scenario';
import type { ControlApplication } from './control';
import type { SwitchOrigin } from './effect-origin';
import type { BreakContribution } from './break';

export interface HitReceipt {
  parent_hit_index?: number;
  autonomous_event_index?: number;
  control_applications?: ControlApplication[];
  switch_origin?: SwitchOrigin;
  break_contribution?: BreakContribution;
  execution_model_assumption_ids?: string[];
  command_index?: number;
  static_general_damage_bonus?: ExactFraction;
  static_element_damage_bonus?: ExactFraction;
  static_mental_damage_bonus?: ExactFraction;
  time_us: number;
  actor_index: number;
  target_index: number;
  ability_id: string;
  damage_effect_id: string;
  scaling_stat?: NativeScalingStat;
  scaling_rate?: Ratio;
  metadata_id: number;
  energy_group_id?: string;
  non_crit_damage: ExactFraction;
  crit_damage: ExactFraction;
  crit_rate: Ratio;
  critical: boolean;
  damage: ExactFraction;
  rounded_damage: number | string;
  applied_hp_loss: number;
  crit_sample?: unknown;
}
