import assert from 'node:assert/strict';
import {
  importFile,
  importReportFile,
  serializeReportBundle,
} from '../src/infrastructure/storage/files.ts';
import {
  encodeContainer,
  decodeContainer,
} from '../src/infrastructure/storage/report-container.ts';
import { attachPngReport, pngChunk } from '../src/infrastructure/storage/report-png.ts';
import {
  init,
  compress,
  decompress,
} from '../../cache/web/node_modules/@bokuweb/zstd-wasm/dist/common/index.node.js';
import { createReplay } from '../src/domain/replay/model.ts';
import { sameObservation } from '../src/domain/reports/identity.ts';

// Shared cases stay within the existing report-controls and Studio boundaries.
export async function checkPortableReportImports(reportState, importedReport, engine, execution) {
  const healingSource = reportState.source,
    artifact = reportState.result.compiled;
  await init();
  const loadCodec = async () => ({ compress, decompress });
  const decode = (bytes) => decodeContainer(bytes, loadCodec);
  const compactReport = serializeReportBundle(reportState, true);
  const binary = await encodeContainer(new TextEncoder().encode(compactReport), loadCodec);
  const png = new Uint8Array([
    137,
    80,
    78,
    71,
    13,
    10,
    26,
    10,
    ...pngChunk('IHDR', new Uint8Array([0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0])),
    ...pngChunk('IDAT', new Uint8Array([0])),
    ...pngChunk('IEND', new Uint8Array()),
  ]);
  for (const [name, payload] of [
    ['report.nter', binary],
    ['report.png', attachPngReport(png, binary)],
  ]) {
    const restored = await importReportFile(new File([payload], name), engine, decode);
    assert.deepEqual(restored, importedReport);
    assert.ok(restored.result.aggregate.actors[0].field.relative);
    assert.ok(restored.result.aggregate.actors[0].field.absolute);
  }
  for (const edit of [
    (v) => {
      delete v.report.clock_contract;
    },
    (v) => {
      v.report.timer_policy = 'obsolete';
    },
    (v) => {
      v.report.relative_elapsed_us = v.report.absolute_elapsed_us + 1;
    },
    (v) => {
      v.samples[0].absolute_elapsed_us++;
    },
    (v) => {
      v.report.timer_pauses = [
        {
          owner: { owner: 0, generation: 0 },
          kind: 'cinematic',
          start_absolute_us: 0,
          end_absolute_us: v.report.absolute_elapsed_us + 1,
        },
      ];
    },
    (v) => delete v.report,
    (v) => {
      v.report = {};
    },
    (v) => {
      v.samples = [];
    },
    (v) => {
      v.request = {};
    },
    (v) => {
      v.aggregate.metrics = {};
    },
    (v) => {
      v.aggregate.actors = [{}];
    },
    (v) => {
      v.party.actors = [{}];
    },
    (v) => {
      v.report.hit_receipts = [null];
    },
    (v) => {
      v.failed_trial = {};
    },
    (v) => {
      v.compiled.input.source = '# different execution';
    },
    (v) => {
      v.partial = false;
    },
    (v) => {
      v.samples[0].shp = 12.34;
    },
    (v) => {
      v.report.available_shield_hp = 12.34;
    },
    (v) => {
      v.aggregate.metrics.dps = { mean: 100 };
    },
    (v) => {
      delete v.samples[0].dps.relative;
    },
    (v) => {
      v.aggregate.actors[0].field = { mean: 10, min: 10, max: 10, std: 0 };
    },
    (v) => {
      delete v.aggregate.actors[0].field.relative;
    },
    (v) => {
      delete v.aggregate.actors[0].field.absolute;
    },
  ]) {
    const bad = JSON.parse(serializeReportBundle(reportState));
    edit(bad);
    await assert.rejects(
      importReportFile(new File([JSON.stringify(bad)], 'bad.json'), engine),
      /simulation report data|clock contract|timer policy|native.*clock|native.*timer|relative.*duration/,
    );
  }
  const failed = {
    ...reportState,
    status: 'failed',
    failure: 'source failure',
    failedTrial: {
      compiled: artifact,
      execution: {
        ...execution,
        diagnostic: {
          code: 'admission',
          message: 'source failure',
          loc: { line: 1, column: 1, offset: 0, length: 1 },
        },
        failure_context: { occurrence_index: 0, command_index: null },
      },
    },
  };
  assert.equal(
    (await importReportFile(new File([serializeReportBundle(failed)], 'failed.json'), engine))
      .failure,
    'source failure',
  );
  for (const state of [failed, reportState]) {
    const json = serializeReportBundle(state, true);
    const container = await encodeContainer(new TextEncoder().encode(json), loadCodec);
    for (const [name, payload] of [
      ['status.json', new TextEncoder().encode(json)],
      ['status.nter', container],
      ['status.png', attachPngReport(png, container)],
    ]) {
      const restored = await importReportFile(new File([payload], name), engine, decode);
      assert.equal(restored.failure, state.failure);
      assert.equal(restored.cancelled, state.status === 'stopped');
      assert.deepEqual(restored.failedTrial, state.failedTrial);
      assert.equal(restored.result.partial, true);
      assert.deepEqual(restored.result.aggregate, state.result.aggregate);
    }
  }

  const wideSource =
    'options duration=3 iteration=1 crit=random seed=42; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; mint attack; wait(1);';
  const wideCompiled = engine.compileSource(wideSource).compiled;
  const wideExecution = engine.runSourceSample({
    compiled: wideCompiled,
    seed: wideCompiled.request.seed,
  });
  const wideAggregate = engine.invoke('aggregate_trials', {
    request: wideExecution.request,
    total: 1,
    reports: [wideExecution.report],
    elapsed_ms: 0,
  }).data;
  const wideState = {
    source: wideSource,
    status: 'completed',
    failure: null,
    failedTrial: null,
    result: {
      ...reportState.result,
      ...wideAggregate,
      compiled: wideCompiled,
      request: wideExecution.request,
      report: wideExecution.report,
      party: engine.inspectScript(wideSource).party,
      occurrences: wideExecution.occurrences,
      trials: 1,
      partial: false,
    },
  };
  const wideStateDigits = wideState.result.report.rng.streams['character:1019'].state;
  assert.equal(wideStateDigits, '10030406343644371790');
  const wideJson = serializeReportBundle(wideState, true);
  const wideBinary = await encodeContainer(new TextEncoder().encode(wideJson), loadCodec);
  for (const [name, payload] of [
    ['wide.json', new TextEncoder().encode(wideJson)],
    ['wide.nter', wideBinary],
    ['wide.png', attachPngReport(png, wideBinary)],
  ]) {
    const restored = await importReportFile(new File([payload], name), engine, decode);
    assert.equal(restored.result.report.rng.streams['character:1019'].state, wideStateDigits);
    assert.deepEqual(restored.result.request, wideState.result.request);
    assert.deepEqual(restored.result.samples, wideState.result.samples);
    assert.deepEqual(restored.result, wideState.result);
  }
  await assert.rejects(importReportFile(new File([png], 'plain.png'), engine), /no embedded/);
  await assert.rejects(
    importReportFile(new File([new Uint8Array([0xff])], 'bad.json'), engine),
    /UTF-8/,
  );
  await assert.rejects(
    importReportFile(new File(['{'], 'bad.json'), engine),
    /invalid report JSON/,
  );

  // Actual complete-file and decoded boundaries, with the same valid bundle.
  // Whitespace keeps parsing/validation small while exercising real UTF-8 bytes.
  const boundary = new Uint8Array(100_000_000).fill(32);
  boundary.set(new TextEncoder().encode(compactReport));
  const boundaryFile = new File([boundary], 'boundary.json');
  assert.equal((await importReportFile(boundaryFile, engine)).source, healingSource);
  let readOversize = false;
  await assert.rejects(
    importReportFile(
      {
        size: 100_000_001,
        name: 'oversize.json',
        text: async () => {
          readOversize = true;
          return '';
        },
      },
      engine,
    ),
    /file limit/,
  );
  assert.equal(readOversize, false);
  const oversizedFile = new File([boundary, new Uint8Array(1)], 'oversize.json');
  await assert.rejects(importReportFile(oversizedFile, engine), /file limit/);
  await assert.rejects(importFile(boundaryFile, engine), /50 MB/);
  const compressedBoundary = await encodeContainer(boundary, loadCodec);
  assert.equal(
    (await importReportFile(new File([compressedBoundary], 'boundary.nter'), engine, decode))
      .source,
    healingSource,
  );
  assert.equal(
    (
      await importReportFile(
        new File([attachPngReport(png, compressedBoundary)], 'boundary.png'),
        engine,
        decode,
      )
    ).source,
    healingSource,
  );
  const tooBig = compressedBoundary.slice();
  new DataView(tooBig.buffer).setUint32(12, 100_000_001, true);
  await assert.rejects(
    importReportFile(new File([tooBig], 'too-big.nter'), engine, decode),
    /decoded size/,
  );
  await assert.rejects(
    importReportFile(new File([attachPngReport(png, tooBig)], 'too-big.png'), engine, decode),
    /decoded size/,
  );
}

export async function checkPortablePlayback(state, roundTrip, engine, execution, model) {
  await init();
  const load = async () => ({ compress, decompress });
  const bytes = await encodeContainer(
    new TextEncoder().encode(serializeReportBundle(state, true)),
    load,
  );
  const png = new Uint8Array([
    137,
    80,
    78,
    71,
    13,
    10,
    26,
    10,
    ...pngChunk('IHDR', new Uint8Array([0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0])),
    ...pngChunk('IDAT', new Uint8Array([0])),
    ...pngChunk('IEND', new Uint8Array()),
  ]);
  for (const [name, payload] of [
    ['roundtrip.nter', bytes],
    ['roundtrip.png', attachPngReport(png, bytes)],
  ]) {
    const imported = await importReportFile(new File([payload], name), engine, (input) =>
      decodeContainer(input, load),
    );
    assert.deepEqual(imported, roundTrip);
    assert.ok(sameObservation(imported.result.report, execution.report));
    assert.equal(
      createReplay(imported.result.request, imported.result.report).events.length,
      model.events.length,
    );
  }
}

export async function checkStudioReportRoundtrip(state, engine, execution, model) {
  const exportedJson = serializeReportBundle(state);
  assert.equal('selected_sample' in JSON.parse(exportedJson), false);
  const roundTrip = await importReportFile(
    { name: 'nte-research-report.json', size: exportedJson.length, text: async () => exportedJson },
    engine,
  );
  assert.equal(roundTrip.source.trim(), state.source.trim());
  assert.ok(sameObservation(roundTrip.result.report, execution.report));
  assert.equal(
    createReplay(roundTrip.result.request, roundTrip.result.report).events.length,
    model.events.length,
  );
  await checkPortablePlayback(state, roundTrip, engine, execution, model);
}
