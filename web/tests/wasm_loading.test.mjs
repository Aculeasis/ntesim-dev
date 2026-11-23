import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import { Worker } from 'node:worker_threads';
import { readFileSync } from 'node:fs';
import { nativeSourceFixtures } from './native_source_fixtures.mjs';
import { NtesimWasm } from '../src/infrastructure/wasm/client.ts';
import { installFetchFixture, wasmBytes as bytes } from './wasm_loading_fixture.mjs';
import { validateObservations } from '../src/domain/reports/validation.ts';
import { runNativeSuite } from './native-suite.mjs';

const expected = (await NtesimWasm.create(bytes)).version();
const loopback = process.env.NTESIM_TEST_LOOPBACK === '1';
const server = loopback
  ? createServer((request, response) => {
      if (request.url === '/missing') {
        response.writeHead(404).end();
        return;
      }
      response.setHeader(
        'Content-Type',
        request.url === '/fallback'
          ? 'application/octet-stream'
          : request.url === '/mime-parameters'
            ? 'application/wasm; charset=utf-8'
            : 'application/wasm',
      );
      response.end(request.url === '/corrupt' ? 'invalid wasm' : bytes);
    })
  : null;
if (server) {
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
}
const base = server ? `http://127.0.0.1:${server.address().port}` : 'http://wasm.test';
const fixture = server ? null : installFetchFixture(base);
const streaming = WebAssembly.instantiateStreaming;
let streamed = 0;
WebAssembly.instantiateStreaming = (...args) => {
  streamed++;
  return streaming(...args);
};
let worker;
try {
  assert.deepEqual((await NtesimWasm.fromUrl(`${base}/engine.wasm`)).version(), expected);
  assert.deepEqual((await NtesimWasm.fromUrl(`${base}/fallback`)).version(), expected);
  assert.deepEqual((await NtesimWasm.fromUrl(`${base}/mime-parameters`)).version(), expected);
  const streaming = WebAssembly.instantiateStreaming;
  try {
    WebAssembly.instantiateStreaming = undefined;
    assert.deepEqual((await NtesimWasm.fromUrl(`${base}/engine.wasm`)).version(), expected);
  } finally {
    WebAssembly.instantiateStreaming = streaming;
  }
  assert.deepEqual((await NtesimWasm.create(await WebAssembly.compile(bytes))).version(), expected);
  await assert.rejects(NtesimWasm.fromUrl(`${base}/missing`), /binary \(404\)/);
  await assert.rejects(NtesimWasm.fromUrl(`${base}/corrupt`), WebAssembly.CompileError);

  worker = new Worker(new URL('./wasm_loading_worker.mjs', import.meta.url), {
    workerData: {
      script: new URL('../src/infrastructure/worker/worker.ts', import.meta.url).href,
      fixtureBase: fixture ? base : null,
    },
  });
  const result = new Promise((resolve, reject) => {
    worker.on('error', reject);
    worker.on('message', (message) => {
      if (message.type === 'SUCCESS') resolve(message.result);
      if (message.type === 'ERROR') reject(new Error(message.error));
    });
  });
  worker.postMessage({ id: 1, type: 'VERSION', wasmSource: `${base}/engine.wasm` });
  assert.deepEqual(await result, expected);
  const source = readFileSync(new URL('../../presets/studio/s1.nte', import.meta.url), 'utf8');
  const direct = await NtesimWasm.create(bytes);
  async function request(message) {
    const response = new Promise((resolve, reject) => {
      const receive = (value) => {
        if (value.id !== message.id || !['SUCCESS', 'ERROR'].includes(value.type)) return;
        worker.off('message', receive);
        if (value.type === 'ERROR') reject(new Error(value.error));
        else resolve(value.result);
      };
      worker.on('message', receive);
    });
    worker.postMessage(message);
    return response;
  }
  const inspection = await request({
    id: 2,
    type: 'INVOKE',
    payload: { action: 'inspect_script', payload: { source } },
  });
  assert.deepEqual(inspection, direct.invoke('inspect_script', { source }));
  const supportSource = `options duration=20 iteration=1 crit=fixed;
    daffodil char lvl=80/80 awakening=0 talent=1,2,3 qte=8 start_energy=120;
    fadia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    target hp=100000000 def=0 resist=0; active daffodil;
    daffodil burst; fadia swap; wait(1); daffodil swap;`;
  const missingSupportLevel = await request({
    id: 110,
    type: 'INVOKE',
    payload: {
      action: 'inspect_script',
      payload: { source: supportSource.replace(' qte=8', '') },
    },
  });
  assert.match(JSON.stringify(missingSupportLevel), /char requires qte=<level>/);
  const supportExecution = await request({
    id: 111,
    type: 'INVOKE',
    payload: { action: 'run_script', payload: { source: supportSource } },
  });
  assert.equal(supportExecution.status, 'ok');
  const { request: supportRequest, report: supportReport } = supportExecution.data.execution;
  assert.equal(supportReport.execution_completed, true);
  assert.equal(supportRequest.actors[0].purchased_qte_level, 8);
  const supportHit = supportReport.hit_receipts.find(
    (hit) => hit.damage_effect_id === 'GE_Player_Daffodill_QTE_Damage',
  );
  const stepHit = supportReport.hit_receipts.find(
    (hit) => hit.damage_effect_id === 'GE_Player_Daffodill_EntryAttack1_Damage',
  );
  assert.ok(supportHit && stepHit && supportHit.time_us < stepHit.time_us);
  assert.equal(supportHit.command_index, stepHit.command_index);
  assert.deepEqual(
    await request({ id: 112, type: 'SIMULATE_GAME', payload: supportRequest }),
    supportReport,
  );
  for (const [index, fixture] of nativeSourceFixtures.entries()) {
    const liveSource = readFileSync(
      new URL(`../../presets/native/${fixture}.nte`, import.meta.url),
      'utf8',
    );
    const liveCompiled = await request({
      id: 1001 + index * 3,
      type: 'INVOKE',
      payload: { action: 'compile_source', payload: { source: liveSource } },
    });
    assert.deepEqual(liveCompiled, direct.invoke('compile_source', { source: liveSource }));
    const liveExecution = await request({
      id: 1002 + index * 3,
      type: 'INVOKE',
      payload: { action: 'run_compiled_source', payload: liveCompiled.data },
    });
    assert.equal(liveExecution.status, 'ok');
    assert.equal(liveExecution.data.execution.report.execution_completed, true);
    assert.deepEqual(liveExecution.data, direct.invoke('run_script', { source: liveSource }).data);
    const replay = await request({
      id: 1003 + index * 3,
      type: 'SIMULATE_GAME',
      payload: liveExecution.data.execution.request,
    });
    assert.deepEqual(replay, liveExecution.data.execution.report);
  }
  const compiled = direct.compileScript(source);
  const reportSource =
    'options duration=3 iteration=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; mint attack; wait(1);';
  const artifact = direct.compileSource(reportSource).compiled;
  const execution = direct.runSourceSample({ compiled: artifact, seed: 42 });
  const exported = await request({
    id: 4000,
    type: 'INVOKE',
    payload: {
      action: 'export_source_report',
      payload: { compiled: artifact, seed: 42 },
    },
  });
  assert.deepEqual(
    exported,
    direct.invoke('export_source_report', { compiled: artifact, seed: 42 }),
  );
  assert.ok(exported.data.csv_tables['timeline.csv'].includes('frame_index'));
  assert.deepEqual(exported.data.report.execution, execution);
  const verified = await request({
    id: 4001,
    type: 'INVOKE',
    payload: {
      action: 'replay_source_report',
      payload: { report_json: exported.data.report_json },
    },
  });
  assert.equal(verified.data.reconciled, true);
  const compared = await request({
    id: 4002,
    type: 'INVOKE',
    payload: {
      action: 'compare_source_reports',
      payload: {
        baseline: reportSource,
        candidate: reportSource,
        request: {
          schema_version: 1,
          seeds: [42],
          max_events_per_run: 1000,
          max_total_events: 4000,
          max_receipt_bytes: 1048576,
        },
      },
    },
  });
  assert.equal(compared.data.comparison.complete, true);
  assert.equal(compared.data.comparison.mean_dps_delta.numerator, '0');
  const report = await request({ id: 3, type: 'SIMULATE_GAME', payload: compiled.request });
  assert.deepEqual(report, direct.simulateGame(compiled.request));
  const autonomousRequest = JSON.parse(
    readFileSync(
      new URL('../../examples/baicang-fadia-game-research.json', import.meta.url),
      'utf8',
    ),
  );
  const autonomous = await request({ id: 30, type: 'SIMULATE_GAME', payload: autonomousRequest });
  assert.deepEqual(autonomous, direct.simulateGame(autonomousRequest));
  validateObservations(autonomousRequest, autonomous);
  if (process.env.NTESIM_VERIFY_NATIVE === '1') {
    const native = runNativeSuite([autonomousRequest]);
    assert.deepEqual(autonomous, native.reports[0]);
  }
  assert.equal(autonomousRequest.actions.length, 0);
  assert.equal(autonomous.autonomous_events.length, 3);
  assert.ok(autonomous.hit_receipts.every((hit) => hit.command_index === undefined));
  for (const mutate of [
    (value) => {
      value.hit_receipts[0].command_index = 0;
    },
    (value) => {
      value.hit_receipts[0].autonomous_event_index = 99;
    },
    (value) => {
      value.autonomous_events[1].trigger.autonomous_event_index = 2;
    },
    (value) => {
      value.autonomous_events[0].recipient_healing.applied++;
    },
  ]) {
    const invalid = structuredClone(autonomous);
    mutate(invalid);
    assert.throws(() => validateObservations(autonomousRequest, invalid), /autonomous/i);
  }
  const invalidSource = '# 🦀\r\noptions duration=1;\r\nwait(1 2);';
  const invalid = await request({
    id: 4,
    type: 'INVOKE',
    payload: { action: 'inspect_script', payload: { source: invalidSource } },
  });
  assert.deepEqual(invalid, direct.invoke('inspect_script', { source: invalidSource }));
  assert.equal(streamed, 2);
  if (fixture)
    assert.deepEqual(fixture.requests, [
      '/engine.wasm',
      '/fallback',
      '/mime-parameters',
      '/engine.wasm',
      '/missing',
      '/corrupt',
    ]);
} finally {
  await worker?.terminate();
  fixture?.restore();
  WebAssembly.instantiateStreaming = streaming;
  if (server) {
    server.closeAllConnections();
    await new Promise((resolve) => server.close(resolve));
  }
}
console.log(
  'WASM URL loading: streaming, MIME fallback, module input, HTTP/compile errors and Worker',
);
