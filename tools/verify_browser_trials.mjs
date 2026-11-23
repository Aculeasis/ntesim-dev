import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Worker as NodeWorker } from "node:worker_threads";
import { createHash } from "node:crypto";
import { getPresetSource } from "../web/src/domain/presets.ts";
import { NtesimWasm, stringifyWasmInput } from "../web/src/infrastructure/wasm/client.ts";
import { SimulationController } from "../web/src/infrastructure/worker/controller.ts";
const product = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
if (process.argv.includes('--browser')) {
  await import('./browser_acceptance.mjs');
  process.exit(0);
}
const bytes = fs.readFileSync(path.join(product, "target/wasm-bindgen/ntesim_wasm_bg.wasm"));

const wasm = await NtesimWasm.create(bytes);
globalThis.Worker = class {
  constructor(script) {
    this.worker = new NodeWorker(
      pathToFileURL(path.join(product, "web/tests/node_worker_bridge.mjs")),
      { workerData: { script: String(script) } },
    );
    this.worker.on("message", (data) => this.onmessage?.({ data }));
    this.worker.on("error", (error) => this.onerror?.(error));
  }
  postMessage(message) {
    this.worker.postMessage(message);
  }
  terminate() {
    return this.worker.terminate();
  }
};
const controller = new SimulationController({ wasmSource: bytes }),
  runs = [];
try {
  const savedS5 = wasm.compileSource(getPresetSource("s5")).compiled;
  assert.ok(savedS5);
  const sampleS5 = wasm.runSourceSample({ compiled: savedS5, seed: 42 });
  assert.equal(sampleS5.report.execution_completed, true);
  assert.equal(sampleS5.report.game_dps_computed, true);
  assert.equal(sampleS5.diagnostic, null);
  const failingSource = wasm.compileSource(
    "options duration=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; mint skill, skill;",
  ).compiled;
  const unavailable = wasm.runSourceSample({ compiled: failingSource, seed: 42 });
  assert.equal(unavailable.report.execution_completed, false);
  assert.equal(unavailable.report.game_dps_computed, false);
  assert.ok(unavailable.diagnostic, "Failing source retains its reached failure instead of fabricated resources");
  for (const crit of ["fixed", "random"]) {
    const source = wasm.editOptions(fs.readFileSync(path.join(product, "presets/native/live-control.nte"), "utf8"), { crit, iteration: 1000 }),
      compiled = wasm.compileSource(source).compiled,
      updates = [];
    const result = await controller.execute(
      "SIMULATE_NTE",
      { source },
      {
        onUpdate: (r) =>
          updates.push({
            completed: r.aggregate.completed,
            elapsed_ms: r.aggregate.elapsed_ms,
          }),
      },
    );
    assert.equal(result.samples.length, 1000);
    assert.equal(result.aggregate.completed, 1000);
    for (let i = 0; i < 1000; i++) {
      const direct = wasm.runSourceSample({ compiled, seed: 42 + i });
      assert.equal(result.samples[i].seed, direct.report.seed);
      assert.equal(result.samples[i].damage, direct.report.total_damage_dealt);
      assert.equal(result.samples[i].hits, direct.report.hits_settled);
      assert.equal(result.samples[i].events, direct.report.total_events_processed);
      assert.equal(result.samples[i].relative_elapsed_us, direct.report.relative_elapsed_us);
      assert.equal(result.samples[i].absolute_elapsed_us, direct.report.absolute_elapsed_us);
      if (i === 0) {
        assert.deepEqual(result.request, direct.request);
        assert.deepEqual(result.report, direct.report);
        assert.deepEqual(result.occurrences, direct.occurrences);
      }
    }
    assert.equal(updates[0].completed, 1);
    assert.equal(updates.at(-1).completed, 1000);
    assert.equal(result.aggregate.metrics.dps.relative.std > 0, crit === "random");
    runs.push({
      crit,
      source_sha256: createHash("sha256").update(source).digest("hex"),
      compiled_sha256: createHash("sha256").update(stringifyWasmInput(compiled)).digest("hex"),
      engine_source_sha256: compiled.engine_source_sha256,
      first_seed: 42,
      last_seed: 1041,
      completed: 1000,
      requested: 1000,
      horizon_seconds: compiled.request.relative_duration_us / 1e6,
      metrics: result.aggregate.metrics,
      elapsed_ms: result.aggregate.elapsed_ms,
      trials_per_second: result.aggregate.trials_per_second,
      updates,
      parity: "1000 direct live WASM/Studio Worker sample comparisons passed",
      quality: result.aggregate.quality,
      game_dps_computed: result.aggregate.game_dps_computed,
    });
  }
} finally {
  controller.terminateWorker();
  delete globalThis.Worker;
}
const verification = {
  schema_version: 1,
  verified_utc: new Date().toISOString(),
  node: process.version,
  wasm_sha256: createHash("sha256").update(bytes).digest("hex"),
  runs,
};
if (process.argv[2] === "--out") {
  const target = path.resolve(product, process.argv[3] || "");
  if (!target.startsWith(product + path.sep) || !target.endsWith(".json"))
    throw new Error("Output must be a JSON file inside ntesim.");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, JSON.stringify(verification, null, 2) + "\n");
}
console.log(JSON.stringify(verification, null, 2));
