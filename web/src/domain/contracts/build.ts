import type { SignedFraction } from './ratio';
import type { Ratio, StaticStats, Action, GearProc, StochasticCrit } from './scenario';
export interface ActorProjectionRequest {
  slot?: number;
  content_id: string;
  character_id: string;
  level: number;
  awakening?: number[];
  max_level?: number;
  weapon_id?: string;
  weapon_level?: number;
  weapon_max_level?: number;
  refinement: number;
  set_id?: string;
  set_count: number;
  base_overrides: { atk?: number; hp?: number; def?: number };
  additive_stats: Record<string, Ratio>;
}
export interface PublicAction {
  command: Action;
  advance_us: number;
  model_assumptions: Record<string, string>;
}
export type EquipmentProjection =
  | { status: 'supported'; gear_procs: GearProc[]; model_assumptions: Record<string, string> }
  | { status: 'unsupported'; diagnostic: string };
export interface ActorProjection {
  static_stats: StaticStats;
  energy_capacity: number;
  charge_efficiency: Ratio;
  model_assumptions: Record<string, string>;
  base: { atk: number; hp: number; def: number };
  unmodeled_attributes: Record<string, SignedFraction>;
  native_action_metadata: unknown;
  public_actions: Record<string, PublicAction>;
  equipment_projection: EquipmentProjection;
  stochastic_crit: StochasticCrit;
}
export type ActorProjector = (request: ActorProjectionRequest) => ActorProjection;
