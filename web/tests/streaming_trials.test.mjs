import assert from 'node:assert/strict';
import fs from 'node:fs';
import { SimulationController } from '../src/infrastructure/worker/controller.ts';
import { engine } from './native-compiler.ts';
import { createReplay } from '../src/domain/replay/model.ts';
import { installControlledTrialWorker } from './fixtures/controlled_trial_worker.mjs';

const bytes = fs.readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);
const source = `options duration=10 iteration=7 seed=42 crit=random;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
target hp=100000000 def=0 resist=0; active mint;
while .time < 3 { mint attack; wait(1); }`;
const compiled = engine.compileSource(source).compiled;
assert.ok(compiled);
const direct = Array.from({ length: 7 }, (_, i) =>
  engine.runSourceSample({ compiled, seed: 42 + i }),
);
const summary = engine.invoke('aggregate_trials', {
  request: compiled.request,
  total: 7,
  reports: direct.map((e) => e.report),
  elapsed_ms: 0,
}).data;

const delivery = installControlledTrialWorker();
const controller = new SimulationController({ wasmSource: bytes, workerCount: 4 });
try {
  const updates = [];
  const result = await controller.execute(
    'SIMULATE_NTE',
    { source },
    { onUpdate: (r) => updates.push(r) },
  );
  assert.ok(updates.length >= 2);
  assert.equal(updates[0].aggregate.completed, 1);
  assert.equal(updates[0].partial, true);
  assert.equal(updates[0].samples.length, 1, 'Deltas cannot mutate an earlier delivered prefix');
  for (const update of updates) assert.equal(update.samples.length, update.aggregate.completed);
  assert.equal(result.partial, false);
  assert.deepEqual(result.samples, summary.samples);
  assert.deepEqual(result.aggregate.metrics, summary.aggregate.metrics);
  assert.equal(result.samples[0].hps.relative, 0);
  assert.equal(result.samples[0].hps.absolute, 0);
  assert.equal(result.samples[0].shp.relative, 0);
  assert.equal(result.samples[0].shp.absolute, 0);
  assert.equal('sps' in result.samples[0], false);
  assert.equal('sps' in result.aggregate.metrics, false);
  assert.equal(result.aggregate.metrics.shp.relative.mean, 0);
  assert.equal(result.aggregate.metrics.shp.absolute.mean, 0);
  assert.equal(result.aggregate.metrics.gross_healing.mean, 0);
  assert.equal(result.aggregate.metrics.shield_absorbed.mean, 0);
  assert.equal(result.aggregate.quality, direct[0].report.quality);
  assert.equal(result.aggregate.game_dps_computed, direct[0].report.game_dps_computed);
  assert.deepEqual(result.request, direct[0].request);
  assert.deepEqual(result.report, direct[0].report);
  assert.deepEqual(result.occurrences, direct[0].occurrences);
  assert.equal(result.compiled.input.source, source);
  const edited = engine.editOptions(source, { seed: 100, duration: 15 });
  assert.notEqual(edited, result.compiled.input.source);
  const sample = await controller.execute('SOURCE_SAMPLE', { compiled: result.compiled, seed: 48 });
  assert.deepEqual(sample, direct[6]);
  const replay = createReplay(sample.request, sample.report);
  assert.equal(replay.stateAt(sample.report.absolute_elapsed_us).hits, sample.report.hits_settled);
  const bundle = await controller.execute('EXPORT_SOURCE_REPORT', {
    compiled: result.compiled,
    seed: sample.report.seed,
  });
  assert.equal(bundle.report.execution.report.seed, 48);
  assert.equal(bundle.report.execution.source, source);
  assert.deepEqual(bundle.report.execution, sample);
  assert.equal(
    bundle.report_json,
    engine.invoke('export_source_report', { compiled, seed: sample.report.seed }).data.report_json,
  );
  assert.equal(
    (await controller.execute('REPLAY_SOURCE_REPORT', { report_json: bundle.report_json }))
      .reconciled,
    true,
  );
  await assert.rejects(
    controller.execute('REPLAY_SOURCE_REPORT', {
      report_json: bundle.report_json.replace(bundle.report.source_sha256, '0'.repeat(64)),
    }),
    /identity mismatch/,
  );
  const compared = await controller.execute('COMPARE_SOURCE_REPORTS', {
    baseline: source,
    candidate: source,
    request: {
      schema_version: 1,
      seeds: [48],
      max_events_per_run: 1000,
      max_total_events: 4000,
      max_receipt_bytes: 1048576,
    },
  });
  assert.equal(compared.comparison.complete, true);
  assert.equal(compared.comparison.mean_dps_delta.numerator, '0');
  let prefix;
  await assert.rejects(
    controller.execute(
      'SIMULATE_NTE',
      { source },
      {
        onUpdate: (r) => {
          prefix = r;
          controller.cancel();
        },
      },
    ),
    /cancelled/,
  );
  assert.equal(prefix.samples.length, 1);
  assert.equal(controller.isBusy, false);
  const resumed = await controller.execute('SIMULATE_NTE', { source, resume: prefix });
  assert.deepEqual(resumed.samples, result.samples);
  assert.deepEqual(resumed.aggregate.metrics, result.aggregate.metrics);
  assert.deepEqual(resumed.report, result.report);
  assert.ok(resumed.aggregate.elapsed_ms >= prefix.aggregate.elapsed_ms);
  const limited = engine.compileSource(source, { maxSteps: 2000, maxActions: 200 }).compiled;
  assert.ok(limited);
  assert.notDeepEqual(limited.input.limits, prefix.compiled.input.limits);
  const limitedResume = await controller.execute('SIMULATE_NTE', {
    source,
    resume: { ...prefix, compiled: limited },
  });
  assert.deepEqual(limitedResume.samples, result.samples);
  assert.deepEqual(
    limitedResume.compiled,
    limited,
    'Resume retains the artifact actually validated and executed by Rust',
  );
  await assert.rejects(
    controller.execute('SIMULATE_NTE', {
      source: edited,
      resume: prefix,
    }),
    /Resume source differs/,
  );
  await assert.rejects(
    controller.execute('SIMULATE_NTE', {
      source,
      resume: { ...prefix, compiled: { ...prefix.compiled, engine_source_sha256: 'stale' } },
    }),
    /identity|fingerprint|differ|mismatch/,
  );
  const retried = await controller.execute('SIMULATE_NTE', { source });
  assert.deepEqual(retried.samples, result.samples);
  const failedSource = source.replace('while .time < 3 { mint attack; wait(1); }', 'mint burst;');
  delivery.reorderFailure = {};
  try {
    await assert.rejects(controller.execute('SIMULATE_NTE', { source: failedSource }), (error) => {
      assert.match(error.message, /Line 4, column 6: Trial 1 halted/);
      assert.equal(error.failure.execution.report.execution_completed, false);
      assert.equal(error.failure.execution.diagnostic.code, 'admission');
      assert.equal(error.failure.execution.request.seed, 42);
      return true;
    });
  } finally {
    delivery.reorderFailure = undefined;
  }
  const failedCompiled = engine.compileSource(failedSource).compiled;
  const failed = await controller.execute('EXPORT_SOURCE_REPORT', {
    compiled: failedCompiled,
    seed: 42,
  });
  assert.equal(failed.report.execution.report.execution_completed, false);
  assert.equal(failed.report.damage.total.dps, null);
  assert.equal(
    (await controller.execute('REPLAY_SOURCE_REPORT', { report_json: failed.report_json }))
      .execution_completed,
    false,
  );

  const supportSource =
    'options iteration=2 duration=10 crit=fixed seed=42; ' +
    'iroi char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp%=1.0; ' +
    'adler char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0; ' +
    'target hp=100000000 def=0 resist=0; ' +
    'active iroi; iroi skill; wait(1); adler swap; adler skill; wait(1);';
  const supportRun = await controller.execute('SIMULATE_NTE', { source: supportSource });
  assert.ok(supportRun.aggregate.metrics.hps.relative.mean > 0);
  assert.ok(supportRun.aggregate.metrics.hps.absolute.mean > 0);
  assert.equal(supportRun.samples[0].hps.relative, supportRun.aggregate.metrics.hps.relative.mean);
  assert.equal(supportRun.samples[0].hps.absolute, supportRun.aggregate.metrics.hps.absolute.mean);
  assert.equal(supportRun.aggregate.metrics.healing.mean, 0);
  assert.equal(
    supportRun.aggregate.metrics.excess_healing.mean,
    supportRun.aggregate.metrics.gross_healing.mean,
  );
  assert.equal(
    supportRun.aggregate.metrics.hps.relative.mean,
    supportRun.aggregate.metrics.gross_healing.mean / 10,
  );
  assert.equal(
    supportRun.aggregate.metrics.hps.absolute.mean,
    supportRun.aggregate.metrics.gross_healing.mean / 10,
  );
  assert.ok(supportRun.aggregate.metrics.shp.relative.mean > 0);
  assert.ok(supportRun.aggregate.metrics.shp.absolute.mean > 0);
  assert.equal(supportRun.samples[0].shp.relative, supportRun.aggregate.metrics.shp.relative.mean);
  assert.equal(supportRun.samples[0].shp.absolute, supportRun.aggregate.metrics.shp.absolute.mean);
  assert.equal(supportRun.aggregate.metrics.shield_absorbed.mean, 0);
  assert.equal('sps' in supportRun.samples[0], false);
  assert.equal('sps' in supportRun.aggregate.metrics, false);

  const compiledSupport = engine.compileSource(supportSource).compiled;
  const directSupportSample42 = engine.runSourceSample({ compiled: compiledSupport, seed: 42 });
  const directSupportSample43 = engine.runSourceSample({ compiled: compiledSupport, seed: 43 });
  const directSupport = engine.invoke('aggregate_trials', {
    request: directSupportSample42.request,
    total: 2,
    reports: [directSupportSample42.report, directSupportSample43.report],
    elapsed_ms: 0,
  }).data;
  assert.deepEqual(supportRun.samples[0].hps, directSupport.samples[0].hps);
  assert.deepEqual(supportRun.samples[0].shp, directSupport.samples[0].shp);
  assert.deepEqual(supportRun.aggregate.metrics.hps, directSupport.aggregate.metrics.hps);
  assert.deepEqual(supportRun.aggregate.metrics.shp, directSupport.aggregate.metrics.shp);

  for (const rotation of [
    'mint burst; zero burst; mint swap;',
    'hotori burst; while .hotori.time_stop.active { hotori attack[window=attack]; }',
  ]) {
    const pausedSource = `options duration=30 iteration=3 crit=fixed seed=42;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_timepiece=120;
      target hp=1000000000 def=0 resist=0; active ${rotation.startsWith('hotori') ? 'hotori' : 'mint'};
      ${rotation}`;
    const paused = await controller.execute('SIMULATE_NTE', { source: pausedSource });
    assert.equal(paused.request.clock_contract, 'relative-combat-v1');
    assert.equal(paused.request.timer_policy, 'paused-world-v1');
    assert.equal(paused.report.relative_elapsed_us, 30_000_000);
    assert.ok(paused.report.absolute_elapsed_us > 30_000_000);
    assert.ok(paused.report.timer_pauses.some((pause) => pause.kind === 'cinematic'));
    assert.equal(
      paused.report.timer_pauses.some((pause) => pause.kind === 'active_stop'),
      rotation.startsWith('hotori'),
    );
    for (const row of paused.samples) {
      const regenerated = await controller.execute('SOURCE_SAMPLE', {
        compiled: paused.compiled,
        seed: row.seed,
      });
      assert.equal(row.relative_elapsed_us, regenerated.report.relative_elapsed_us);
      assert.equal(row.absolute_elapsed_us, regenerated.report.absolute_elapsed_us);
      createReplay(regenerated.request, regenerated.report);
    }
    let pausedPrefix;
    await assert.rejects(
      controller.execute(
        'SIMULATE_NTE',
        { source: pausedSource },
        {
          onUpdate: (value) => {
            pausedPrefix = value;
            controller.cancel();
          },
        },
      ),
      /cancelled/,
    );
    const tampered = structuredClone(pausedPrefix);
    tampered.samples[0].relative_elapsed_us++;
    await assert.rejects(
      controller.execute('SIMULATE_NTE', { source: pausedSource, resume: tampered }),
      /resume|prefix|observation|differ|mismatch/i,
    );
    const tamperedShp = structuredClone(pausedPrefix);
    tamperedShp.samples[0].shp.relative = (tamperedShp.samples[0].shp.relative ?? 0) + 1;
    await assert.rejects(
      controller.execute('SIMULATE_NTE', { source: pausedSource, resume: tamperedShp }),
      /resume|prefix|observation|differ|mismatch/i,
    );
    const tamperedScalar = structuredClone(pausedPrefix);
    tamperedScalar.samples[0].shp = 12.34;
    await assert.rejects(
      controller.execute('SIMULATE_NTE', { source: pausedSource, resume: tamperedScalar }),
      /resume|prefix|observation|differ|mismatch|TimeBasisPair|invalid type/i,
    );
    const continued = await controller.execute('SIMULATE_NTE', {
      source: pausedSource,
      resume: pausedPrefix,
    });
    assert.deepEqual(continued.samples, paused.samples);
    assert.deepEqual(continued.aggregate.metrics, paused.aggregate.metrics);
  }
  const wide = await controller.execute('SIMULATE_NTE', {
    source: `options duration=3 iteration=2 seed=42 crit=fixed;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      mint add stats atk=1000000 dmg%=100000000000;
      target hp=1 def=0 resist=0; target hp=1 def=0 resist=0;
      active mint; wait(1); mint attack;`,
  });
  const settled = wide.report.hit_receipts.reduce(
    (sum, hit) => sum + BigInt(hit.rounded_damage),
    0n,
  );
  assert.ok(settled > BigInt(Number.MAX_SAFE_INTEGER));
  assert.equal(wide.report.total_damage_exact, `${settled}/1`);
  assert.equal(wide.samples[0].damage, Number(settled));
  assert.equal(
    createReplay(wide.request, wide.report).stateAt(wide.report.absolute_elapsed_us).damage,
    Number(settled),
  );

  // Hold the oldest response while the other compute workers finish. A
  // stopped head must bound the results waiting for ordered aggregation.
  const tailReady = new Promise((ready) => {
    delivery.heldFirstChunk = { tail: new Set(), sent: 0, received: 0, ready };
  });
  const blocked = controller.execute('SIMULATE_NTE', {
    source: source
      .replace('iteration=7', 'iteration=401')
      .replace('while .time < 3 { mint attack; wait(1); }', 'wait(1);'),
  });
  const stopped = assert.rejects(blocked, /cancelled/);
  try {
    await Promise.race([tailReady, blocked]);
    await new Promise((done) => setImmediate(done));
    assert.equal(delivery.heldFirstChunk.sent, 6);
  } finally {
    delivery.heldFirstChunk = undefined;
    controller.cancel();
  }
  await stopped;
} finally {
  controller.terminateWorker();
  delete globalThis.Worker;
}
console.log(
  'Live source Worker: Rust summaries, seed samples, executed source, replay, deltas, stop/retry and admission diagnostics passed',
);
