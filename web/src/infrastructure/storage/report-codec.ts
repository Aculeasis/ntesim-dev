export interface CodecRequest {
  id: number;
  operation: 'encode' | 'decode';
  buffer: ArrayBuffer;
}
export interface CodecResponse {
  id: number;
  buffer?: ArrayBuffer;
  error?: string;
}
let sequence = 0,
  queue: Promise<unknown> = Promise.resolve();

// One isolated Worker per operation releases the grown WASM heap even on error.
// Transfer ownership and never share the simulation Worker pool.
export function reportCodec(
  operation: CodecRequest['operation'],
  bytes: Uint8Array<ArrayBuffer>,
  signal?: AbortSignal,
) {
  const task = queue.then(
    () =>
      new Promise<Uint8Array<ArrayBuffer>>((resolve, reject) => {
        if (signal?.aborted) {
          reject(new Error('Report operation cancelled.'));
          return;
        }
        const worker = new Worker(new URL('./report-codec.worker.ts', import.meta.url), {
          type: 'module',
        });
        const id = ++sequence;
        const finish = (error?: string, buffer?: ArrayBuffer) => {
          worker.terminate();
          clearTimeout(timer);
          signal?.removeEventListener('abort', abort);
          if (error || !buffer) reject(new Error(error || 'Report codec returned no data.'));
          else resolve(new Uint8Array(buffer));
        };
        const abort = () => finish('Report operation cancelled.');
        const timer = setTimeout(() => finish('Report codec timed out.'), 120_000);
        signal?.addEventListener('abort', abort, { once: true });
        worker.onmessage = ({ data }: MessageEvent<CodecResponse>) => {
          if (data.id === id) finish(data.error, data.buffer);
        };
        worker.onerror = (event) => {
          event.preventDefault();
          finish('Report codec failed: ' + event.message);
        };
        worker.onmessageerror = () => finish('Report codec returned invalid data.');
        try {
          worker.postMessage({ id, operation, buffer: bytes.buffer } satisfies CodecRequest, [
            bytes.buffer,
          ]);
        } catch (error) {
          finish(String(error));
        }
      }),
  );
  queue = task.catch(() => {});
  return task;
}
