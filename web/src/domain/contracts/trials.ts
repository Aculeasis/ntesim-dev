import type { ScenarioRequest } from './scenario';
import type { EnergyGrant, ScenarioReport } from './report';
import type { Statistics } from '../reports/statistics';
import type { CompiledSourceProgram, PartyInspection, SourceLocation } from './script';
import type { SourceOccurrence } from './source';
import type { TimeBasisPair } from './clocks';
export type { TimeBasisPair } from './clocks';

export interface TrialSample {
  seed: number;
  damage: number;
  dps: TimeBasisPair<number>;
  eps: TimeBasisPair<number>;
  granted_eps: TimeBasisPair<number>;
  hps: TimeBasisPair<number>;
  shp: TimeBasisPair<number>;
  relative_elapsed_us: number;
  absolute_elapsed_us: number;
  hits: number;
  events: number;
}
export interface ActorAggregate {
  dps: TimeBasisPair<Statistics>;
  slot: number;
  character_id: string;
  damage: Statistics;
  hits: Statistics;
  energy: Statistics;
  granted: Statistics;
  effective: Statistics;
  field: TimeBasisPair<Statistics> & {
    relative: Statistics;
    absolute: Statistics;
  };
  actions: Record<string, Statistics>;
}
export interface DamageAggregate {
  dps: TimeBasisPair<Statistics>;
  actor_index: number;
  ability_id: string;
  effect_id: string;
  damage: Statistics;
  hits: Statistics;
}
export interface EnergyAggregate {
  actor_index: number;
  source: EnergyGrant['source'];
  kind: string;
  energy: Statistics;
  effective: Statistics;
  overflow: Statistics;
}
export interface TimelinePoint {
  time: number;
  dps: Statistics;
  cumulative: Statistics;
}
export interface TargetAggregate {
  target_index: number;
  damage: Statistics;
  dps: TimeBasisPair<Statistics>;
}
export interface ReactionAggregate extends Statistics {
  name: string;
}
export interface AggregateMetrics {
  dps: TimeBasisPair<Statistics>;
  eps: TimeBasisPair<Statistics>;
  granted_eps: TimeBasisPair<Statistics>;
  hps: TimeBasisPair<Statistics>;
  shp: TimeBasisPair<Statistics>;
  damage: Statistics;
  duration: Statistics;
  absolute_duration: Statistics;
  hits: Statistics;
  events: Statistics;
  gross_healing: Statistics;
  healing: Statistics;
  excess_healing: Statistics;
  shield_absorbed: Statistics;
}
export interface TrialAggregate {
  completed: number;
  requested: number;
  elapsed_ms: number;
  trials_per_second: number | null;
  metrics: AggregateMetrics;
  actors: ActorAggregate[];
  sources: DamageAggregate[];
  energy_sources: EnergyAggregate[];
  targets: TargetAggregate[];
  timeline: TimelinePoint[];
  bucket_seconds: number;
  quality: ScenarioReport['quality'];
  game_dps_computed: boolean;
  reactions: ReactionAggregate[];
}
export interface TrialResult {
  party: PartyInspection;
  report: ScenarioReport;
  samples: TrialSample[];
  request: ScenarioRequest;
  compiled: CompiledSourceProgram;
  actionLocations: SourceLocation[];
  occurrences: SourceOccurrence[];
  trials: number;
  aggregate: TrialAggregate;
  warnings: string[];
  partial: boolean;
}
