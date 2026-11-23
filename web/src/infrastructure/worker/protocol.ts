import type { ScenarioProgramRequest, ScenarioRequest } from '../../domain/contracts/scenario';
import type { ScenarioReport } from '../../domain/contracts/report';
import type { FailedSourceTrial, SourceSampleRequest } from '../../domain/contracts/source';
import type { ExperimentRequest, ReportRequest } from '../../domain/contracts/source-report';
import type { TrialResult, TrialSample } from '../../domain/contracts/trials';
export type WorkerCommand =
  | { type: 'SIMULATE_NTE'; payload: { source: string; resume?: TrialResult } }
  | { type: 'SOURCE_SAMPLE'; payload: SourceSampleRequest }
  | { type: 'EXPORT_SOURCE_REPORT'; payload: ReportRequest }
  | { type: 'REPLAY_SOURCE_REPORT'; payload: { report_json: string } }
  | {
      type: 'COMPARE_SOURCE_REPORTS';
      payload: { baseline: string; candidate: string; request: ExperimentRequest };
    }
  | { type: 'SIMULATE_GAME'; payload: ScenarioRequest | ScenarioProgramRequest }
  | { type: 'SIMULATE_ROTATION'; payload: { fixture: unknown; rotation: string; seed: number } }
  | { type: 'RUN_BATCH'; payload: { scenario: unknown; batchRequest: unknown } }
  | { type: 'VERIFY_TABULAR'; payload: { cases?: unknown } }
  | { type: 'ENUMERATE_COMPOSITIONS'; payload: { variantsCsv?: string } }
  | {
      type: 'RUN_SOURCE_CHUNK';
      payload: { compiled: unknown; start_offset: number; count: number };
    }
  | {
      type: 'ADD_SOURCE_OBSERVATIONS';
      payload: { observations: unknown[]; elapsed_ms: number; snapshot?: boolean };
    }
  | { type: 'INVOKE'; payload: { action: string; payload: unknown } }
  | { type: 'INIT' | 'VERSION' | 'CANCEL'; payload?: unknown };
export type WorkerRequest = WorkerCommand & { id: number; wasmSource?: BufferSource | string };
export type WorkerResponse =
  | { id: number; type: 'PROGRESS'; progress: number; stage: string }
  | {
      id: number;
      type: 'UPDATE';
      result: Omit<TrialResult, 'samples'> & {
        samples_delta: TrialSample[];
        sample_offset: number;
      };
    }
  | { id: number; type: 'SUCCESS'; result: unknown }
  | {
      id: number;
      type: 'ERROR';
      error: string;
      taskId?: number;
      source_failure?: FailedSourceTrial;
    }
  | { id: number; type: 'CANCELLED' };
export interface RunCallbacks {
  onProgress?: (progress: number, stage: string) => void;
  onUpdate?: (result: TrialResult) => void;
}
export interface WorkerTransport {
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage(message: WorkerRequest): void;
  terminate(): void;
}
export interface RunTask {
  id: number;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  onProgress: (progress: number, stage: string) => void;
  onUpdate: (result: TrialResult) => void;
  samples: TrialSample[];
}
export type SimulationResult = TrialResult | ScenarioReport;
