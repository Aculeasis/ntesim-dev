import assert from 'node:assert/strict';
import { after, test } from 'node:test';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { Worker } from 'node:worker_threads';
import { NtesimWasm } from '../src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../src/infrastructure/storage/lossless-json.ts';
import { createReplay } from '../src/domain/replay/model.ts';
import { eventLocation } from '../src/domain/replay/source.ts';

const root = fileURLToPath(new URL('../../', import.meta.url));
const cache = path.join(root, 'cache/verification');
fs.mkdirSync(cache, { recursive: true });
const directory = fs.mkdtempSync(path.join(cache, 'source-calibration-'));
const input = path.join(directory, 'input.nte');
const experiment = path.join(directory, 'experiment.json');
const seeds = [42, 43];
const budgets = {
  schema_version: 1,
  seeds,
  max_events_per_run: 10000,
  max_total_events: 100000,
  max_receipt_bytes: 64 * 1024 * 1024,
};
fs.writeFileSync(experiment, JSON.stringify(budgets));
const bytes = fs.readFileSync(path.join(root, 'target/wasm-bindgen/ntesim_wasm_bg.wasm'));
const engine = await NtesimWasm.create(bytes);
const worker = new Worker(new URL('./node_worker_bridge.mjs', import.meta.url), {
  workerData: { script: new URL('../src/infrastructure/worker/worker.ts', import.meta.url).href },
});
let id = 0;
function request(type, payload) {
  const requestId = ++id;
  return new Promise((resolve, reject) => {
    const receive = (message) => {
      if (message.id !== requestId || !['SUCCESS', 'ERROR'].includes(message.type)) return;
      worker.off('message', receive);
      worker.off('error', reject);
      resolve(message);
    };
    worker.on('message', receive);
    worker.once('error', reject);
    worker.postMessage({ id: requestId, type, payload, wasmSource: bytes });
  });
}
async function success(type, payload) {
  const message = await request(type, payload);
  assert.equal(message.type, 'SUCCESS', message.error);
  return message.result;
}
function native(args, expectedStatus = 0) {
  const output = path.join(directory, 'native.json');
  const fd = fs.openSync(output, 'w');
  let result;
  try {
    result = spawnSync(
      path.join(root, 'target/debug', process.platform === 'win32' ? 'ntesim.exe' : 'ntesim'),
      args,
      { encoding: 'utf8', timeout: 60000, stdio: ['ignore', fd, 'pipe'] },
    );
  } finally {
    fs.closeSync(fd);
  }
  assert.ifError(result.error);
  assert.equal(result.status, expectedStatus, `${args[0]}: ${result.stderr}; ${directory}`);
  return parseLosslessJson(fs.readFileSync(output, 'utf8'));
}
function observations(aggregate) {
  const { elapsed_ms, trials_per_second, ...result } = aggregate;
  assert.ok(elapsed_ms >= 0);
  return result;
}
after(async () => {
  await worker.terminate();
  fs.rmSync(directory, { recursive: true, force: true });
});

const baselines = fs
  .readdirSync(path.join(root, 'examples/baselines'))
  .filter((name) => name.endsWith('.nte'))
  .map((name) => `examples/baselines/${name}`)
  .concat([
    'examples/mint-native-cycles.nte',
    'examples/chaos-native-cycles.nte',
    'presets/native/nanally-full-kit.nte',
  ]);
const sources = baselines.map((fixture) => [
  fixture,
  fs.readFileSync(path.join(root, fixture), 'utf8'),
]);
sources.push([
  'early victory',
  `options duration=10 stop=victory crit=fixed;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
target hp=1 def=0 resist=0; active mint; mint attack;`,
]);
for (const [fixture, original] of sources) {
  test(`Continuing source native/WASM/Studio Worker calibration: ${fixture}`, async () => {
    const source = engine.editOptions(original, {
      iteration: seeds.length,
      seed: seeds[0],
    });
    fs.writeFileSync(input, source);
    const compiled = native(['compile-script', input]);
    assert.deepEqual(engine.compileSource(source), compiled);
    const batch = native(['batch-script', input, experiment]);
    assert.equal(batch.eligible_runs, seeds.length);
    const result = await success('SIMULATE_NTE', { source });
    assert.equal(result.partial, false);
    assert.deepEqual(result.compiled, compiled.compiled);
    assert.deepEqual(result.party, compiled.party);
    const reports = [];
    for (const [index, seed] of seeds.entries()) {
      const execution = batch.runs[index].execution;
      assert.deepEqual(engine.runSourceSample({ compiled: compiled.compiled, seed }), execution);
      if (index === seeds.length - 1)
        assert.deepEqual(
          await success('SOURCE_SAMPLE', { compiled: compiled.compiled, seed }),
          execution,
        );
      if (index === 0) {
        assert.deepEqual(result.request, execution.request);
        assert.deepEqual(result.report, execution.report);
        assert.deepEqual(result.actionLocations, execution.action_locations);
        assert.deepEqual(result.occurrences, execution.occurrences);
      }
      const replay = createReplay(execution.request, execution.report);
      assert.equal(
        replay.stateAt(execution.report.absolute_elapsed_us).hits,
        execution.report.hits_settled,
      );
      if (index === 0) {
        const checked = new Set();
        for (const event of execution.report.autonomous_events ?? []) {
          if (
            !['coordinated_hit', 'settled_hit'].includes(event.trigger.kind) ||
            checked.has(event.trigger.kind)
          )
            continue;
          checked.add(event.trigger.kind);
          const altered = structuredClone(execution.report);
          const ordinal = execution.report.autonomous_events.indexOf(event);
          const child = replay.events.find(
            (entry) => entry.hit?.autonomous_event_index === ordinal,
          );
          const command =
            event.trigger.kind === 'coordinated_hit'
              ? event.trigger.cast_command_index
              : execution.report.hit_receipts[event.trigger.hit_index].command_index;
          if (command !== undefined) {
            assert.ok(child);
            assert.deepEqual(eventLocation(execution, child), execution.action_locations[command]);
          }
          altered.autonomous_events[ordinal].trigger.hit_index = altered.hit_receipts.length;
          assert.throws(() => createReplay(execution.request, altered), /causal parent/);
          const cycle = structuredClone(execution.report);
          cycle.hit_receipts[event.trigger.hit_index].autonomous_event_index = ordinal;
          assert.throws(() => createReplay(execution.request, cycle), /causal parent/);
          if (event.trigger.kind === 'coordinated_hit') {
            const missingCast = structuredClone(execution.report);
            missingCast.autonomous_events[ordinal].trigger.cast_command_index =
              execution.request.actions.length;
            assert.throws(() => createReplay(execution.request, missingCast), /causal parent/);
          }
        }
      }
      reports.push(execution.report);
    }
    const summary = engine.invoke('aggregate_trials', {
      request: compiled.compiled.request,
      total: seeds.length,
      reports,
      elapsed_ms: 0,
    }).data;
    assert.deepEqual(result.samples, summary.samples);
    // Wall-clock throughput is host metadata; every gameplay/quality field is exact.
    assert.deepEqual(observations(result.aggregate), observations(summary.aggregate));
  });
}

test('Native exact exports, CSV, replay, comparisons and rejected source trials cross the Worker', async () => {
  const source = `options duration=10 iteration=2 seed=42 crit=random;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0; active mint; mint attack; wait(1);`;
  const candidate = source.replace('mint attack;', 'mint skill;');
  for (const [index, text] of [source, source.replace('mint attack;', 'mint burst;')].entries()) {
    fs.writeFileSync(input, text);
    const complete = index === 0;
    const status = complete ? 0 : 2;
    const run = native(['run-script', input], status);
    assert.deepEqual(engine.invoke('run_script', { source: text }).data, run);
    const batch = native(['batch-script', input, experiment], status);
    const compiled = engine.compileSource(text).compiled;
    const trial = await request('SIMULATE_NTE', { source: text });
    if (complete) {
      assert.equal(trial.type, 'SUCCESS');
    } else {
      assert.equal(trial.type, 'ERROR');
      assert.deepEqual(trial.source_failure.execution, run.execution);
      assert.equal(batch.eligible_runs, 0);
      assert.equal(batch.mean_damage, null);
      assert.throws(
        () =>
          engine.invoke('aggregate_trials', {
            request: compiled.request,
            total: 2,
            reports: [run.execution.report],
            elapsed_ms: 0,
          }),
        /failed or incomplete/,
      );
    }
    const output = path.join(directory, `export-${index}`);
    const manifest = native(['export-script', input, output], status);
    const exported = await success('EXPORT_SOURCE_REPORT', { compiled, seed: 42 });
    assert.deepEqual(
      exported.report,
      parseLosslessJson(fs.readFileSync(path.join(output, 'report.json'), 'utf8')),
    );
    assert.equal(exported.report_json, fs.readFileSync(path.join(output, 'report.json'), 'utf8'));
    assert.deepEqual(
      Object.keys(exported.csv_tables).sort(),
      Object.keys(manifest.files)
        .filter((name) => name.endsWith('.csv'))
        .sort(),
    );
    for (const [name, csv] of Object.entries(exported.csv_tables)) {
      assert.equal(csv, fs.readFileSync(path.join(output, name), 'utf8'));
    }
    const reportPath = path.join(directory, `worker-report-${index}.json`);
    fs.writeFileSync(reportPath, exported.report_json);
    assert.deepEqual(
      await success('REPLAY_SOURCE_REPORT', { report_json: exported.report_json }),
      native(['replay-script', reportPath]),
    );
  }
  fs.writeFileSync(input, source);
  const candidatePath = path.join(directory, 'candidate.nte');
  fs.writeFileSync(candidatePath, candidate);
  const comparison = await success('COMPARE_SOURCE_REPORTS', {
    baseline: source,
    candidate,
    request: budgets,
  });
  const nativeComparison = native(['compare-scripts', input, candidatePath, experiment]);
  assert.deepEqual(comparison.comparison, nativeComparison);
  assert.deepEqual(parseLosslessJson(comparison.report_json), nativeComparison);
  assert.notEqual(comparison.comparison.mean_damage_delta.numerator, '0');
  const failedCandidate = source.replace('mint attack;', 'mint burst;');
  fs.writeFileSync(candidatePath, failedCandidate);
  const failedComparison = await success('COMPARE_SOURCE_REPORTS', {
    baseline: source,
    candidate: failedCandidate,
    request: budgets,
  });
  assert.deepEqual(
    failedComparison.comparison,
    native(['compare-scripts', input, candidatePath, experiment], 2),
  );
  assert.equal(failedComparison.comparison.complete, false);
  assert.equal(failedComparison.comparison.mean_dps_delta, null);

  const invalid = '# 🦀\r\noptions duration=10;\r\nwait(1 2);';
  fs.writeFileSync(input, invalid);
  const diagnostics = native(['compile-script', input], 2);
  assert.deepEqual(engine.compileSource(invalid), diagnostics);
  assert.deepEqual(
    (await success('INVOKE', { action: 'compile_source', payload: { source: invalid } })).data,
    diagnostics,
  );
  const rejected = await request('SIMULATE_NTE', { source: invalid });
  assert.equal(rejected.type, 'ERROR');
  assert.match(rejected.error, /Line 3, column 8/);
});
