import { workerData } from 'node:worker_threads';
import { installFetchFixture } from './wasm_loading_fixture.mjs';

if (workerData.fixtureBase) installFetchFixture(workerData.fixtureBase);
await import('./node_worker_bridge.mjs');
