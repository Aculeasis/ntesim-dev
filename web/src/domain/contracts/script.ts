import type { ExactFraction, Ratio, ScenarioRequest, StaticStats } from './scenario';
import type { CombatStaminaProfile } from './stamina';

export interface SourceLocation {
  line: number;
  column: number;
  offset: number;
  length: number;
}
export interface ScriptDiagnostic {
  code:
    | 'lexical'
    | 'syntax'
    | 'binding'
    | 'configuration'
    | 'evaluation'
    | 'arithmetic'
    | 'work_limit'
    | 'action_limit'
    | 'event_limit'
    | 'admission'
    | 'horizon'
    | 'energy_gate';
  message: string;
  loc: SourceLocation;
}
export interface BuildDecomposition {
  contributions: {
    source:
      | 'character'
      | 'weapon'
      | 'set'
      | 'authored'
      | 'character_passive'
      | 'weapon_passive'
      | 'base_override';
    attribute_id: string;
    amount: ExactFraction;
  }[];
  attributes: Record<string, ExactFraction>;
  atk: ExactFraction;
  hp: ExactFraction;
  defense: ExactFraction;
}
export interface ActorInspection {
  slot: number;
  character_id: string;
  loc: SourceLocation;
  level: number;
  max_level: number | null;
  awakening: number[];
  talent_levels: [number, number, number];
  purchased_qte_level: number;
  weapon: { id: string; level: number; max_level: number; refinement: number } | null;
  set: { id: string; count: number } | null;
  static_stats: StaticStats;
  charge_efficiency: Ratio;
  initial_hp: number;
  initial_energy: number;
  energy_capacity: number;
  initial_cycle: number;
  initial_resources: Record<string, Ratio>;
  additive_stats: Record<string, Ratio>;
  build_decomposition: BuildDecomposition;
  unmodeled_attribute_ids: string[];
}
export interface PartyInspection {
  schema_version: number;
  content_id: string;
  active_actor_index: number;
  start_fons: number | null;
  actors: ActorInspection[];
  model_assumption_ids: string[];
}
export interface CompiledOptions {
  trials: number;
  relative_duration_us: number;
  duration_specified: boolean;
  seed: number;
  crit: 'fixed' | 'random';
  stop: 'time' | 'victory';
  start_fons: number | null;
  combat_stamina?: CombatStaminaProfile;
}
export interface CompiledSourceProgram {
  schema_version: number;
  engine_version: string;
  engine_source_sha256: string;
  input: { source: string; limits: { maxSteps: number; maxActions: number } };
  request: ScenarioRequest;
  trials: number;
}
export interface SourceCompileResponse {
  compiled: CompiledSourceProgram | null;
  party: PartyInspection | null;
  options: CompiledOptions | null;
  diagnostics: ScriptDiagnostic[];
}
export interface InspectionResponse {
  party: PartyInspection | null;
  diagnostics: ScriptDiagnostic[];
}
