import { Worker as NodeWorker } from 'node:worker_threads';

export function installControlledTrialWorker() {
  const delivery = { reorderFailure: undefined, heldFirstChunk: undefined };
  globalThis.Worker = class {
    constructor(script) {
      this.worker = new NodeWorker(new URL('../node_worker_bridge.mjs', import.meta.url), {
        workerData: { script: String(script) },
      });
      this.worker.on('message', (data) => {
        this.onmessage?.({ data });
        if (data.type !== 'SUCCESS') return;
        const { reorderFailure, heldFirstChunk } = delivery;
        if (reorderFailure?.postFirstChunk && data.result?.failure) {
          const post = reorderFailure.postFirstChunk;
          reorderFailure.postFirstChunk = undefined;
          post();
        }
        if (heldFirstChunk?.tail.delete(data.id)) {
          if (++heldFirstChunk.received === 5) heldFirstChunk.ready();
        }
      });
      this.worker.on('error', (error) => this.onerror?.(error));
    }
    postMessage(message) {
      if (message.type === 'RUN_SOURCE_CHUNK') {
        const isFirst = message.payload.start_offset === 0;
        const { reorderFailure, heldFirstChunk } = delivery;
        if (reorderFailure && isFirst) {
          reorderFailure.postFirstChunk = () => this.worker.postMessage(message);
          return;
        }
        if (heldFirstChunk) {
          heldFirstChunk.sent++;
          if (isFirst) return;
          heldFirstChunk.tail.add(message.id);
        }
      }
      this.worker.postMessage(message);
    }
    terminate() {
      return this.worker.terminate();
    }
  };
  return delivery;
}
