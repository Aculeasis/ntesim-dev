import { parentPort, workerData } from 'node:worker_threads';
import { register } from '../../cache/web/node_modules/tsx/dist/esm/api/index.mjs';
register({
  tsconfig: new URL('../tsconfig.runtime.json', import.meta.url).pathname.replace(
    /^\/([A-Z]:)/,
    '$1',
  ),
});

globalThis.self = globalThis;
self.postMessage = (message) => parentPort.postMessage(message);
await import(workerData.script);
parentPort.on('message', (data) => self.onmessage({ data }));
