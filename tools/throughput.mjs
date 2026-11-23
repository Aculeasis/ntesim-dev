/** Development-only host over production WASM/session and Worker adapters. */
import fs from "node:fs";
import { createRequire } from "node:module";
import { Worker } from "node:worker_threads";
import { performance } from "node:perf_hooks";
import { Session } from "node:inspector/promises";
import path from "node:path";
import { SimulationController } from "../web/src/infrastructure/worker/controller.ts";
import { sendWorkerRequest } from "../web/src/infrastructure/worker/parallel.ts";
const [
  mode,
  sourceFile,
  countText,
  workerText,
  gluePath,
  profilePath,
  chunkText = "50",
  placement,
] = process.argv.slice(2);
const count = Number(countText),
  workers = Number(workerText);
const source = fs.readFileSync(sourceFile, "utf8");
const bytes = fs.readFileSync(path.join(gluePath, "ntesim_wasm_bg.wasm"));
const cacheRoot = path.resolve(import.meta.dirname, "../cache");
if (!path.resolve(profilePath).startsWith(cacheRoot + path.sep))
  throw new Error("profile output must stay in cache/");
const inspector = gluePath.endsWith("profiling") ? new Session() : null;
if (inspector) {
  inspector.connect();
  await inspector.post("Profiler.enable");
  await inspector.post("Profiler.start");
  await inspector.post("HeapProfiler.startSampling", {
    samplingInterval: 32768,
  });
}
const started = performance.now();
const finishWorker = (adapter) =>
  new Promise((resolve) => {
    adapter.finish = resolve;
    adapter.worker.postMessage({ type: "DEVELOPMENT_FINISH", id: -1 });
  });
const developmentMessage = (adapter, data) => {
  if (data.type === "DEVELOPMENT_MEASURE") {
    adapter.measurements.push(data);
    return true;
  }
  if (data.type === "DEVELOPMENT_FINISHED") {
    adapter.finish?.(data);
    return true;
  }
  return false;
};
let result;
if (mode === "chunks") {
  const chunkSize = Number(chunkText);
  const records = [],
    adapters = [];
  let nextId = 0;
  const createWorker = (workerId) => {
    const worker = new Worker(
      new URL("./throughput_worker.mjs", import.meta.url),
      {
        workerData: {
          bindings: path.join(gluePath, "bindings.js"),
          profilePath: inspector ? profilePath + "-" + workerId : null,
          script: new URL(
            "../web/src/infrastructure/worker/worker.ts",
            import.meta.url,
          ).href,
        },
      },
    );
    const adapter = {
      onmessage: null,
      onerror: null,
      worker,
      measurements: [],
      pending: new Map(),
      postMessage(message) {
        const start = performance.now();
        worker.postMessage(message);
        adapter.pending.set(message.id, start);
        records.push({
          worker: workerId,
          phase: "post",
          id: message.id,
          type: message.type,
          action: message.payload?.action,
          offset: message.payload?.start_offset,
          count: message.payload?.count,
          at_ms: start - started,
          post_ms: performance.now() - start,
        });
      },
    };
    worker.on("message", (data) => {
      if (developmentMessage(adapter, data)) return;
      if (data.type === "SUCCESS" || data.type === "ERROR") {
        const requestStart = adapter.pending.get(data.id);
        records.push({
          worker: workerId,
          phase: "receive",
          id: data.id,
          type: data.type,
          at_ms: performance.now() - started,
          roundtrip_ms: performance.now() - requestStart,
        });
        adapter.pending.delete(data.id);
      }
      adapter.onmessage?.({ data });
    });
    worker.on("error", (error) => adapter.onerror?.(error));
    adapters.push(adapter);
    return adapter;
  };
  const dedicatedAggregator =
    placement === "shared" ? null : createWorker("aggregator");
  const computes = Array.from({ length: workers }, (_, index) =>
    createWorker(index),
  );
  const aggregator = dedicatedAggregator ?? computes[0];
  const request = (worker, command) =>
    sendWorkerRequest(worker, command, bytes, ++nextId);
  const invoke = async (worker, action, payload) =>
    (await request(worker, { type: "INVOKE", payload: { action, payload } }))
      .data;
  try {
    await Promise.all(
      adapters.map((worker) => request(worker, { type: "INIT" })),
    );
    const compiledResponse = await invoke(aggregator, "compile_source", {
      source,
    });
    const compiled = compiledResponse.compiled;
    if (
      !compiled ||
      compiledResponse.diagnostics.length ||
      count > compiled.trials
    )
      throw new Error("invalid source/trial count");
    await invoke(aggregator, "start_source_trials", compiled);
    // Identical explicit warmup/preparation for every compute instance. Each
    // timed trial still starts with fresh combat state and its authored seed.
    const warmupStart = performance.now();
    await Promise.all(
      computes.map((worker) =>
        request(worker, {
          type: "RUN_SOURCE_CHUNK",
          payload: { compiled, start_offset: 0, count: 1 },
        }),
      ),
    );
    const warmup_ms = performance.now() - warmupStart;
    const executionStart = performance.now();
    const samples = [],
      observations = [];
    let offset = 0;
    await Promise.all(
      computes.map(async (worker) => {
        while (offset < count) {
          const current = offset;
          const size = current === 0 ? 1 : Math.min(chunkSize, count - current);
          offset += size;
          const chunk = await request(worker, {
            type: "RUN_SOURCE_CHUNK",
            payload: { compiled, start_offset: current, count: size },
          });
          if (chunk.failure || chunk.samples.length !== size)
            throw new Error("failed or incomplete chunk");
          samples.push(...chunk.samples);
          observations.push(...chunk.observations);
        }
      }),
    );
    const execution_ms = performance.now() - executionStart;
    // Scheduling may finish out of order; statistics commit in authored order.
    samples.sort((a, b) => a.seed - b.seed);
    observations.sort((a, b) => a.sample.seed - b.sample.seed);
    const aggregateStart = performance.now();
    await invoke(aggregator, "add_source_observations", {
      observations,
      elapsed_ms: 0,
      snapshot: false,
    });
    const aggregate = await invoke(aggregator, "snapshot_source_trials", 0);
    result = {
      engine_source_sha256: compiled.engine_source_sha256,
      compute_workers: workers,
      pool_size: adapters.length,
      chunk_size: chunkSize,
      shared_aggregator: placement === "shared",
      warmup_ms,
      execution_ms,
      aggregate_ms: performance.now() - aggregateStart,
      wall_ms: performance.now() - started,
      samples,
      observations,
      aggregate,
      records,
      memory: process.memoryUsage(),
    };
    result.worker_finishes = await Promise.all(adapters.map(finishWorker));
    result.worker_measurements = adapters.map(
      (adapter) => adapter.measurements,
    );
  } finally {
    await Promise.all(adapters.map((adapter) => adapter.worker.terminate()));
  }
} else if (mode === "workers") {
  const records = [],
    adapters = [];
  let id = 0;
  const controller = new SimulationController({
    wasmSource: bytes,
    workerCount: workers,
    workerFactory(script) {
      const workerId = id++;
      const worker = new Worker(
        new URL("./throughput_worker.mjs", import.meta.url),
        {
          workerData: {
            bindings: path.join(gluePath, "bindings.js"),
            script: String(script),
            profilePath: inspector ? profilePath + "-" + workerId : null,
          },
        },
      );
      const adapter = {
        onmessage: null,
        onerror: null,
        worker,
        measurements: [],
        terminate() {
          return worker.terminate();
        },
        postMessage(message) {
          const start = performance.now();
          worker.postMessage(message);
          records.push({
            worker: workerId,
            phase: "post",
            type: message.type,
            action: message.payload?.action,
            offset: message.payload?.start_offset,
            count: message.payload?.count,
            at_ms: start - started,
            ms: performance.now() - start,
          });
        },
      };
      worker.on("message", (data) => {
        if (developmentMessage(adapter, data)) return;
        records.push({
          worker: workerId,
          phase: "receive",
          type: data.type,
          at_ms: performance.now() - started,
        });
        adapter.onmessage?.({ data });
      });
      worker.on("error", (error) => adapter.onerror?.(error));
      adapters.push(adapter);
      return adapter;
    },
  });
  const updates = [],
    progress = [];
  try {
    result = await controller.execute(
      "SIMULATE_NTE",
      { source },
      {
        onProgress(value, stage) {
          progress.push({ value, stage, at_ms: performance.now() - started });
        },
        onUpdate(value) {
          updates.push({
            completed: value.samples.length,
            at_ms: performance.now() - started,
          });
        },
      },
    );
    if (count !== result.samples.length)
      throw new Error(
        "Worker mode requires the unchanged authored trial count",
      );
    result = {
      engine_source_sha256: result.report.engine_source_sha256,
      wall_ms: performance.now() - started,
      samples: result.samples,
      aggregate: result.aggregate,
      records,
      updates,
      progress,
      memory: process.memoryUsage(),
    };
    result.worker_finishes = await Promise.all(adapters.map(finishWorker));
    result.worker_measurements = adapters.map(
      (adapter) => adapter.measurements,
    );
  } finally {
    controller.terminateWorker();
  }
} else {
  // Resolve only the selected generated factory, then use the ordinary runtime.
  const require = createRequire(import.meta.url);
  globalThis.NtesimCreateBindings = require(
    path.join(gluePath, "bindings.js"),
  ).createBindings;
  require("../crates/wasm/js/lossless-json.js");
  require("../crates/wasm/js/wire.js");
  require("../crates/wasm/js/runtime.js");
  const bindings = globalThis.NtesimCreateBindings();
  bindings.initSync({ module: await WebAssembly.compile(bytes) });
  const runtime = new globalThis.NtesimWasm(bindings);
  const load_ms = performance.now() - started;
  const timings = [];
  const invoke = (action, payload) => {
    const begin = performance.now();
    const wire = runtime.responseBytes("ntesim_wasm_invoke", {
      action,
      payload,
    });
    const execution_ms = performance.now() - begin;
    const decode = performance.now();
    const response = globalThis.NtesimWire.decodeWire(wire);
    if (response.status === "error") throw new Error(response.error);
    timings.push({
      action,
      execution_ms,
      decode_ms: performance.now() - decode,
      bytes: wire.length,
    });
    return response.data;
  };
  const compiled = invoke("compile_source", { source });
  if (compiled.diagnostics.length)
    throw new Error(JSON.stringify(compiled.diagnostics));
  if (count > compiled.compiled.trials)
    throw new Error("count exceeds authored trials");
  invoke("start_source_trials", compiled.compiled);
  const samples = [];
  for (let offset = 0; offset < count;) {
    const update = invoke("step_source_trials", {
      offset,
      count: offset === 0 ? 1 : Math.min(25, count - offset),
      elapsed_ms: 0,
    });
    if (update.failure || !update.samples_delta.length)
      throw new Error("failed or incomplete trial");
    samples.push(...update.samples_delta);
    offset += update.samples_delta.length;
  }
  const aggregate = invoke("snapshot_source_trials", 0);
  result = {
    engine_source_sha256: compiled.compiled.engine_source_sha256,
    load_ms,
    wall_ms: performance.now() - started,
    timings,
    samples,
    aggregate,
    memory: process.memoryUsage(),
  };
}
if (inspector) {
  const { profile } = await inspector.post("Profiler.stop");
  fs.writeFileSync(profilePath + ".cpuprofile", JSON.stringify(profile));
  const allocation = await inspector.post("HeapProfiler.stopSampling");
  fs.writeFileSync(
    profilePath + ".heapprofile",
    JSON.stringify(allocation.profile),
  );
  inspector.disconnect();
}
console.log(JSON.stringify(result));
