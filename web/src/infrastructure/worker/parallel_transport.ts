import type { WorkerCommand, WorkerRequest, WorkerResponse, WorkerTransport } from './protocol';
import { SourceTrialError, SimulationCancelledError } from './controller';

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

const workerQueues = new WeakMap<WorkerTransport, Map<number, PendingRequest>>();
let nextRequestId = -1;

function ensureWorkerQueue(worker: WorkerTransport): Map<number, PendingRequest> {
  const existing = workerQueues.get(worker);
  if (existing) return existing;
  const queue = new Map<number, PendingRequest>();
  workerQueues.set(worker, queue);
  const prevMsg = worker.onmessage;
  const prevErr = worker.onerror;
  worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
    const msg = event.data;
    if (!msg || msg.type === 'PROGRESS') return;
    const pending = queue.get(msg.id);
    if (!pending) {
      prevMsg?.(event);
      return;
    }
    queue.delete(msg.id);
    if (msg.type === 'SUCCESS') {
      pending.resolve(msg.result);
    } else if (msg.type === 'ERROR') {
      pending.reject(
        msg.source_failure
          ? new SourceTrialError(msg.error, msg.source_failure)
          : new Error(msg.error || 'Worker error'),
      );
    } else if (msg.type === 'CANCELLED') {
      pending.reject(new SimulationCancelledError('Simulation cancelled'));
    }
  };
  worker.onerror = (event: ErrorEvent) => {
    for (const req of queue.values()) {
      req.reject(new Error(`Worker fatal error: ${event.message || 'Worker crash'}`));
    }
    queue.clear();
    prevErr?.(event);
  };
  return queue;
}

export function sendWorkerRequest<T>(
  worker: WorkerTransport,
  command: WorkerCommand,
  wasmSource: BufferSource | string,
): Promise<T> {
  const taskId = nextRequestId--;
  const queue = ensureWorkerQueue(worker);
  return new Promise<T>((resolve, reject) => {
    queue.set(taskId, {
      resolve: resolve as (value: unknown) => void,
      reject,
    });
    try {
      worker.postMessage({
        ...command,
        id: taskId,
        wasmSource,
      } as WorkerRequest);
    } catch (error) {
      queue.delete(taskId);
      reject(error);
    }
  });
}

export function cancelParallelRequests(workers: WorkerTransport[]): void {
  for (const worker of workers) {
    const queue = workerQueues.get(worker);
    if (!queue) continue;
    for (const req of queue.values()) {
      req.reject(new SimulationCancelledError('Simulation cancelled'));
    }
    queue.clear();
  }
}
