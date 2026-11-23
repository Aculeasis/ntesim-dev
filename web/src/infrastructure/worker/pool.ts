import type { WorkerRequest, WorkerTransport } from './protocol';

export const PIPELINE_DEPTH = 2;
export const MIN_FLUSH_INTERVAL = 100;
export const MAX_FLUSH_INTERVAL = 1000;
export const FLUSH_COST_RATIO = 10;

/**
 * Default number of simulation workers:
 * clamps (cores - 2) to [2, 8] based on hardwareConcurrency and available memory.
 * Falls back to 1 if hardwareConcurrency is unavailable or <= 2.
 */
export function defaultWorkerCount(): number {
  if (typeof window === 'undefined' || typeof navigator === 'undefined') return 1;
  const cores = navigator.hardwareConcurrency;
  if (!cores || cores < 2) return 1;
  const memory = (navigator as { deviceMemory?: number }).deviceMemory;
  if (memory != null && memory < 4) return 2;
  const max = memory != null && memory < 8 ? 4 : 8;
  return Math.min(Math.max(cores - 2, 2), max);
}

/**
 * WorkerPool manages a pool of Web Worker instances, supporting request pipelining,
 * broadcast cancellation, and graceful lifecycle management.
 */
export class WorkerPool {
  private workers: WorkerTransport[] = [];
  private workerCount: number;
  private factory: (script: string) => WorkerTransport;
  private script: string;

  constructor(
    script: string,
    factory: (script: string) => WorkerTransport,
    workerCount = defaultWorkerCount(),
  ) {
    this.script = script;
    this.factory = factory;
    this.workerCount = Math.max(1, workerCount);
  }

  public get count(): number {
    return this.workerCount;
  }

  public setWorkerCount(count: number): void {
    this.workerCount = Math.max(1, count);
  }

  public ensure(count = this.workerCount): WorkerTransport[] {
    while (this.workers.length < count) {
      this.workers.push(this.factory(this.script));
    }
    return this.workers;
  }

  public primary(): WorkerTransport {
    return this.ensure(1)[0];
  }

  public broadcast(message: WorkerRequest): void {
    for (const worker of this.workers) {
      try {
        worker.postMessage(message);
      } catch {
        /* Ignore broken transport during broadcast */
      }
    }
  }

  public terminate(): void {
    for (const worker of this.workers) {
      try {
        worker.terminate();
      } catch {
        /* Ignore broken transport during termination */
      }
    }
    this.workers = [];
  }
}
