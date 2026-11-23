import type { ScenarioRequest } from './scenario';
import type { ScenarioReport } from './report';
import type { CompiledSourceProgram, ScriptDiagnostic, SourceLocation } from './script';
import type { TrialAggregate, TrialSample } from './trials';

export interface SourceOccurrence {
  index: number;
  loc: SourceLocation;
  kind: 'action' | 'target_gauge' | 'wait' | 'wait_until' | 'wait_for_energy';
  started_us: number;
  completed_us: number | null;
  command_indices: number[];
  action_window?: 'attack' | 'hold' | 'jump' | 'dash' | 'skill' | 'swap';
}
export interface SourceExecution {
  source: string;
  request: ScenarioRequest;
  report: ScenarioReport;
  action_locations: SourceLocation[];
  occurrences: SourceOccurrence[];
  diagnostic: ScriptDiagnostic | null;
  failure_context: { occurrence_index: number; command_index: number | null } | null;
}
export interface SourceTrialUpdate {
  first_execution: SourceExecution | null;
  samples_delta: TrialSample[];
  aggregate: TrialAggregate | null;
  failure: SourceExecution | null;
}
export interface SourceSampleRequest {
  compiled: CompiledSourceProgram;
  seed: number;
}
export interface FailedSourceTrial {
  compiled: CompiledSourceProgram;
  execution: SourceExecution;
}
