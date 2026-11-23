import type { ScenarioProgramRequest, ScenarioRequest } from '../../domain/contracts/scenario';
import type { ScenarioReport } from '../../domain/contracts/report';
import type {
  FailedSourceTrial,
  SourceExecution,
  SourceSampleRequest,
} from '../../domain/contracts/source';
import type { TrialResult } from '../../domain/contracts/trials';
import type {
  ComparisonBundle,
  ExperimentRequest,
  ReportBundle,
  ReportRequest,
  SourceReplayVerification,
} from '../../domain/contracts/source-report';
import type {
  RunCallbacks,
  RunTask,
  WorkerTransport,
  WorkerRequest,
  WorkerResponse,
  WorkerCommand,
} from './protocol';
import { WorkerPool, defaultWorkerCount } from './pool';
import { executeParallelNte } from './parallel';
export class SourceTrialError extends Error {
  constructor(
    message: string,
    readonly failure: FailedSourceTrial,
  ) {
    super(message);
  }
}
export class SimulationCancelledError extends Error {}
export class SimulationController {
  pool: WorkerPool;
  worker: WorkerTransport | null = null;
  currentTaskId = 0;
  activeTask: RunTask | null = null;
  isBusy = false;
  workerScript: string;
  wasmSource: BufferSource | string;
  timeoutMs: number;
  constructor(
    options: {
      workerScript?: string;
      wasmSource?: BufferSource | string;
      timeoutMs?: number;
      workerCount?: number;
      workerFactory?: (script: string) => WorkerTransport;
    } = {},
  ) {
    this.workerScript = options.workerScript ?? new URL('./worker.ts', import.meta.url).href;
    this.wasmSource =
      options.wasmSource ??
      new URL('../../../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url).href;
    this.timeoutMs = options.timeoutMs ?? 0;
    this.factory =
      options.workerFactory ??
      (options.workerScript
        ? (script) => new Worker(script, { type: 'module' })
        : () => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' }));
    this.pool = new WorkerPool(this.workerScript, this.factory, options.workerCount);
  }
  private factory: (script: string) => WorkerTransport;
  get workerCount(): number {
    return this.pool.count;
  }
  setWorkerCount(count: number): void {
    this.pool.setWorkerCount(count);
  }
  initWorker() {
    if (!this.worker) {
      const worker = this.pool.primary();
      this.worker = worker;
      worker.onmessage = (event) => {
        if (this.worker === worker) this.handleMessage(event);
      };
      worker.onerror = (event) => {
        if (this.worker === worker) this.handleWorkerError(event);
      };
    }
    return this.worker;
  }
  terminateWorker() {
    this.pool.terminate();
    this.worker = null;
    this.isBusy = false;
  }
  execute(
    type: 'SIMULATE_NTE',
    payload: { source: string; resume?: TrialResult },
    callbacks?: RunCallbacks,
  ): Promise<TrialResult>;
  execute(
    type: 'SOURCE_SAMPLE',
    payload: SourceSampleRequest,
    callbacks?: RunCallbacks,
  ): Promise<SourceExecution>;
  execute(
    type: 'SIMULATE_GAME',
    payload: ScenarioRequest | ScenarioProgramRequest,
    callbacks?: RunCallbacks,
  ): Promise<ScenarioReport>;
  execute(type: 'EXPORT_SOURCE_REPORT', payload: ReportRequest): Promise<ReportBundle>;
  execute(
    type: 'REPLAY_SOURCE_REPORT',
    payload: { report_json: string },
  ): Promise<SourceReplayVerification>;
  execute(
    type: 'COMPARE_SOURCE_REPORTS',
    payload: { baseline: string; candidate: string; request: ExperimentRequest },
  ): Promise<ComparisonBundle>;
  execute(
    type: WorkerCommand['type'],
    payload: unknown,
    callbacks?: RunCallbacks,
  ): Promise<unknown>;
  execute(
    type: WorkerCommand['type'],
    payload: unknown,
    callbacks: RunCallbacks = {},
  ): Promise<unknown> {
    if (this.isBusy)
      return Promise.reject(
        new Error(
          'Simulation runner is currently busy. Cancel existing task before starting a new one.',
        ),
      );
    try {
      this.initWorker();
    } catch (error) {
      return Promise.reject(error);
    }
    const id = ++this.currentTaskId;
    this.isBusy = true;
    return new Promise((resolve, reject) => {
      const timer =
        this.timeoutMs > 0
          ? setTimeout(
              () => this.cancel('Execution timed out after ' + this.timeoutMs / 1000 + 's'),
              this.timeoutMs,
            )
          : null;
      const lifecycle = new AbortController();
      const finish = () => {
        if (timer) clearTimeout(timer);
        this.isBusy = false;
        this.activeTask = null;
        lifecycle.abort();
      };
      this.activeTask = {
        id,
        resolve: (value) => {
          finish();
          resolve(value);
        },
        reject: (error) => {
          finish();
          reject(error);
        },
        onProgress: callbacks.onProgress ?? (() => {}),
        onUpdate: callbacks.onUpdate ?? (() => {}),
        samples: [],
      };
      if (
        type === 'SIMULATE_NTE' &&
        this.pool.count > 1 &&
        !(payload as { resume?: unknown })?.resume
      ) {
        void executeParallelNte({
          payload: payload as { source: string },
          callbacks,
          pool: this.pool,
          wasmSource: this.wasmSource,
          isCurrent: () => this.activeTask?.id === id,
          signal: lifecycle.signal,
          finish,
          resolve: resolve as (value: unknown) => void,
          reject,
        }).catch((error) => {
          if (this.activeTask?.id === id)
            this.activeTask.reject(error instanceof Error ? error : new Error(String(error)));
        });
        return;
      }
      try {
        this.worker!.postMessage({
          id,
          type,
          payload,
          wasmSource: this.wasmSource,
        } as WorkerRequest);
      } catch (error) {
        this.terminateWorker();
        this.activeTask?.reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  cancel(reason = 'Simulation cancelled by user') {
    const task = this.activeTask;
    if (!task) return;
    try {
      this.pool.broadcast({ id: task.id, type: 'CANCEL' });
    } catch {
      /* Termination still cancels a broken transport. */
    } finally {
      this.terminateWorker();
      task.reject(
        reason === 'Simulation cancelled by user'
          ? new SimulationCancelledError(reason)
          : new Error(reason),
      );
    }
  }
  handleMessage(event: MessageEvent<WorkerResponse>) {
    const message = event.data,
      task = this.activeTask;
    if (!task || task.id !== message.id) return;
    try {
      switch (message.type) {
        case 'PROGRESS':
          task.onProgress(message.progress, message.stage);
          break;
        case 'UPDATE': {
          const { samples_delta, sample_offset, ...snapshot } = message.result;
          if (sample_offset !== task.samples.length)
            throw new Error('Trial update sample offset mismatch');
          task.samples = task.samples.concat(samples_delta);
          task.onUpdate({ ...snapshot, samples: task.samples });
          break;
        }
        case 'SUCCESS':
          task.resolve(message.result);
          break;
        case 'ERROR':
          this.terminateWorker();
          task.reject(
            message.source_failure
              ? new SourceTrialError(message.error, message.source_failure)
              : new Error(message.error || 'Unknown simulation error'),
          );
          break;
        case 'CANCELLED':
          this.terminateWorker();
          task.reject(new SimulationCancelledError('Simulation cancelled'));
          break;
      }
    } catch (error) {
      this.terminateWorker();
      task.reject(
        new Error(
          `Report update failed: ${error instanceof Error ? error.message : String(error)}`,
        ),
      );
    }
  }
  handleWorkerError(event: ErrorEvent) {
    this.activeTask?.reject(new Error(`Worker fatal error: ${event.message || 'Worker crash'}`));
    this.terminateWorker();
  }
}
