import { NtesimWasm } from '../wasm/client';
import { runScript } from './runner';
import { ScriptError } from '../wasm/script';
import type { WorkerRequest, WorkerResponse } from './protocol';
import type { ScenarioProgramRequest } from '../../domain/contracts/scenario';
const ctx = globalThis as unknown as Pick<DedicatedWorkerGlobalScope, 'postMessage' | 'onmessage'>;
let wasm: NtesimWasm | null = null;
const post = (message: WorkerResponse) => ctx.postMessage(message);
async function load(source?: BufferSource | string) {
  if (wasm) return wasm;
  if (!source) throw new Error('An explicit WASM URL or byte buffer is required.');
  wasm =
    typeof source === 'string' ? await NtesimWasm.fromUrl(source) : await NtesimWasm.create(source);
  return wasm;
}
ctx.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const message = event.data,
    { id, type } = message;
  try {
    if (type === 'CANCEL') {
      post({ id, type: 'CANCELLED' });
      return;
    }
    if (!wasm) post({ id, type: 'PROGRESS', progress: 10, stage: 'Loading WebAssembly engine' });
    const engine = await load(message.wasmSource);
    let result: unknown;
    switch (message.type) {
      case 'SIMULATE_NTE':
        await runScript(engine, message, post);
        return;
      case 'SOURCE_SAMPLE':
        result = engine.runSourceSample(message.payload);
        break;
      case 'EXPORT_SOURCE_REPORT':
        result = engine.invoke('export_source_report', message.payload);
        result = (result as { data: unknown }).data;
        break;
      case 'REPLAY_SOURCE_REPORT':
        result = (engine.invoke('replay_source_report', message.payload) as { data: unknown }).data;
        break;
      case 'COMPARE_SOURCE_REPORTS':
        result = (engine.invoke('compare_source_reports', message.payload) as { data: unknown })
          .data;
        break;
      case 'INIT':
        result = { initialized: true };
        break;
      case 'VERSION':
        result = engine.version();
        break;
      case 'SIMULATE_GAME':
        result =
          'branches' in message.payload && 'energy_waits' in message.payload
            ? engine.simulateGameProgram(message.payload as ScenarioProgramRequest)
            : engine.simulateGame(message.payload);
        break;
      case 'SIMULATE_ROTATION':
        result = engine.simulateRotation(
          message.payload.fixture,
          message.payload.rotation,
          message.payload.seed,
        );
        break;
      case 'RUN_BATCH':
        result = engine.runBatch(message.payload.scenario, message.payload.batchRequest);
        break;
      case 'VERIFY_TABULAR':
        result = engine.verifyTabular(message.payload.cases);
        break;
      case 'ENUMERATE_COMPOSITIONS':
        result = engine.enumerateCompositions(message.payload.variantsCsv);
        break;
      case 'RUN_SOURCE_CHUNK':
        result = (engine.invoke('run_source_chunk', message.payload) as { data: unknown }).data;
        break;
      case 'ADD_SOURCE_OBSERVATIONS':
        result = (engine.invoke('add_source_observations', message.payload) as { data: unknown })
          .data;
        break;
      case 'INVOKE':
        result = engine.invoke(message.payload.action, message.payload.payload);
        break;
      default:
        throw new Error(`Unsupported worker request type: ${type}`);
    }
    post({ id, type: 'PROGRESS', progress: 100, stage: 'Complete' });
    post({ id, type: 'SUCCESS', result });
  } catch (error) {
    post({
      id,
      type: 'ERROR',
      error:
        error instanceof ScriptError
          ? `Line ${error.line}, column ${error.column}: ${error.message}`
          : error instanceof Error
            ? error.message
            : String(error),
    });
  }
};
