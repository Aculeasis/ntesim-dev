/** Run through python tools/cpu_limit.py; a real Worker owns the WASM engine. */
import { Worker, isMainThread, parentPort } from "node:worker_threads";
import { createRequire } from "node:module";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";

if (isMainThread) {
  const worker = new Worker(new URL(import.meta.url));
  try {
    const result = await new Promise((resolve, reject) => {
      worker.once("message", resolve);
      worker.once("error", reject);
      worker.once("exit", (code) => {
        if (code) reject(new Error(`Worker exited ${code}`));
      });
    });
    mkdirSync("cache/verification", { recursive: true });
    writeFileSync(
      "cache/verification/wasm-transport.json",
      JSON.stringify(result, null, 2) + "\n",
    );
    console.log(JSON.stringify(result, null, 2));
  } finally {
    await worker.terminate();
  }
} else {
  const { NtesimWasm, stringifyWasmInput } = createRequire(import.meta.url)(
    "../crates/wasm/js/ntesim.js",
  );
  const bytes = readFileSync("target/wasm-bindgen/ntesim_wasm_bg.wasm");
  const source = readFileSync("presets/studio/s5.nte", "utf8");
  const engine = await NtesimWasm.create(bytes);
  const { request } = engine.invoke("compile_script", { source }).data.compiled;
  const payload = { scenario: request, start_seed: 42, count: 25 };
  const binary = engine.call("simulate_game_batch", payload);
  const json = engine.call("simulate_game_batch", payload, true);
  assert.deepEqual(binary, json);
  const measurements = { binary: [], json: [] };
  for (let round = 0; round < 7; round++) {
    for (const mode of round % 2 ? ["json", "binary"] : ["binary", "json"]) {
      const started = performance.now();
      const reports = engine.call(
        "simulate_game_batch",
        payload,
        mode === "json",
      );
      const elapsed = performance.now() - started;
      assert.equal(reports.length, 25);
      measurements[mode].push(elapsed);
    }
  }
  const input = stringifyWasmInput(payload);
  const sizes = Object.fromEntries(
    ["binary", "json"].map((mode) => [
      mode,
      engine.bindings.transport("simulate_game_batch", input, mode === "json")
        .byteLength,
    ]),
  );
  const median = (values) =>
    [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
  const medians = Object.fromEntries(
    Object.entries(measurements).map(([key, value]) => [key, median(value)]),
  );
  parentPort.postMessage({
    schema_version: 1,
    node: process.version,
    workload:
      "S5, 25 full reports per batch, seven alternating warm measurements in Node Worker",
    includes:
      "request encoding, prepared simulation, serialization, generated binding copy, decoding",
    excludes: "startup, WASM compilation, warmup, postMessage, UI aggregation",
    wasm_sha256: createHash("sha256").update(bytes).digest("hex"),
    source_sha256: createHash("sha256").update(source).digest("hex"),
    measurements_ms: measurements,
    median_ms: medians,
    response_bytes: sizes,
    speedup: medians.json / medians.binary,
    exact_full_report_parity: true,
  });
}
