import type { InputActionRole } from './action-role';

export interface StochasticCrit {
  roll_policy: 'independent_per_target_hit_v1';
  hypothesis: string;
  eligible_effects: Record<string, string[]>;
  actor_sequences: Record<string, number>;
}

export interface ActionMetadata {
  hypothesis_id: string;
  ability_id: string;
  damage_effect_id: string;
  action_gameplay_tags?: string[] | null;
  input_action_role?: InputActionRole | null;
  damage_type_ex?: string | null;
  damage_source_category?: string | null;
  packet_ga_name_binding?: string | null;
}
