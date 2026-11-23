import type { ExactFraction, SignedFraction } from './ratio';
import type { SourceExecution } from './source';
import type { PartyInspection, CompiledSourceProgram } from './script';

export interface DamageTotals {
  hits: number;
  exact: ExactFraction;
  applied_hp_loss: number | string;
  overkill: number | string;
  dps: ExactFraction | null;
}
export interface DamageAnalysis {
  total: DamageTotals;
  contributions: {
    actor_index: number;
    target_index: number;
    ability_id: string;
    damage_effect_id: string;
    on_field: boolean;
    totals: DamageTotals;
  }[];
}
export interface SourceReportExport {
  schema_version: number;
  source_sha256: string;
  request_sha256: string;
  input_sha256: string;
  engine_version: string;
  engine_source_sha256: string;
  party: PartyInspection;
  damage: DamageAnalysis;
  execution: SourceExecution;
  uncertainty_policy: string;
}
export interface ReportBundle {
  report: SourceReportExport;
  report_json: string;
  csv_tables: Record<string, string>;
}
export interface SourceReplayVerification {
  reconciled: boolean;
  source_sha256: string;
  input_sha256: string;
  request_sha256: string;
  success: boolean;
  execution_completed: boolean;
  quality: SourceExecution['report']['quality'];
  damage: DamageAnalysis;
}
export interface ReportRequest {
  compiled: CompiledSourceProgram;
  seed: number;
}
export interface ExperimentRequest {
  schema_version: number;
  seeds: number[];
  max_events_per_run: number;
  max_total_events: number;
  max_receipt_bytes: number;
}
export interface ExperimentBatch {
  source_sha256: string;
  input_sha256: string;
  engine_source_sha256: string;
  compiled: CompiledSourceProgram;
  request: ExperimentRequest;
  complete: boolean;
  eligible_runs: number;
  halt_diagnostic: string | null;
  uncertainty_policy: string;
  runs: {
    seed: number;
    eligible: boolean;
    error: string | null;
    execution: SourceExecution | null;
  }[];
}
export interface SourceComparison {
  input_sha256: string;
  direction: string;
  complete: boolean;
  baseline: ExperimentBatch;
  candidate: ExperimentBatch;
  pairs: {
    seed: number;
    eligible: boolean;
    damage_delta: SignedFraction | null;
    dps_delta: SignedFraction | null;
  }[];
  mean_damage_delta: SignedFraction | null;
  mean_dps_delta: SignedFraction | null;
}
export interface ComparisonBundle {
  comparison: SourceComparison;
  report_json: string;
}
