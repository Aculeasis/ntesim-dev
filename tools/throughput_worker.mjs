/** Development-only instrumentation around the existing Node Worker adapter. */
import { parentPort, workerData } from "node:worker_threads";
import { registerHooks } from "node:module";
import { Session } from "node:inspector/promises";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

const measurements = [];
let memory;
const threadCpu = () => process.threadCpuUsage?.() ?? null;
const cpuDifference = (before) => {
  const after = threadCpu();
  return before && after
    ? (after.user + after.system - before.user - before.system) / 1000
    : null;
};
const originalCompile = WebAssembly.compile;
WebAssembly.compile = async function (...args) {
  const start = performance.now();
  const result = await originalCompile.apply(this, args);
  measurements.push({ phase: "module_compile", ms: performance.now() - start });
  return result;
};
const OriginalInstance = WebAssembly.Instance;
WebAssembly.Instance = new Proxy(OriginalInstance, {
  construct(target, args) {
    const start = performance.now();
    const result = Reflect.construct(target, args);
    memory = Object.values(result.exports).find(
      (value) => value instanceof WebAssembly.Memory,
    );
    measurements.push({ phase: "instance", ms: performance.now() - start });
    return result;
  },
});
const inspector = workerData.profilePath ? new Session() : null;
if (inspector) {
  inspector.connect();
  await inspector.post("Profiler.enable");
  await inspector.post("Profiler.start");
  await inspector.post("HeapProfiler.startSampling", {
    samplingInterval: 32768,
  });
}
const bindingUrl = pathToFileURL(workerData.bindings).href;
registerHooks({
  resolve(specifier, context, nextResolve) {
    // Keep archived/profiled WASM paired with the bindings generated for it.
    return nextResolve(
      specifier.endsWith("/target/wasm-bindgen/bindings.js")
        ? bindingUrl
        : specifier,
      context,
    );
  },
});
await import("../web/tests/node_worker_bridge.mjs");
const originalResponse = globalThis.NtesimWasm.prototype.responseBytes;
globalThis.NtesimWasm.prototype.responseBytes = function (
  name,
  payload,
  ...rest
) {
  const start = performance.now(),
    cpu = threadCpu();
  const result = originalResponse.call(this, name, payload, ...rest);
  measurements.push({
    phase: "rust_and_wire",
    action: payload?.action,
    ms: performance.now() - start,
    thread_cpu_ms: cpuDifference(cpu),
    bytes: result.length,
  });
  return result;
};
const originalDecode = globalThis.NtesimWire.decodeWire;
globalThis.NtesimWire.decodeWire = function (...args) {
  const start = performance.now();
  const result = originalDecode(...args);
  measurements.push({ phase: "wire_decode", ms: performance.now() - start });
  return result;
};
const handle = self.onmessage;
self.onmessage = async (event) => {
  if (event.data.type === "DEVELOPMENT_FINISH") {
    if (inspector) {
      const { profile } = await inspector.post("Profiler.stop");
      fs.writeFileSync(
        workerData.profilePath + ".cpuprofile",
        JSON.stringify(profile),
      );
      const allocation = await inspector.post("HeapProfiler.stopSampling");
      fs.writeFileSync(
        workerData.profilePath + ".heapprofile",
        JSON.stringify(allocation.profile),
      );
      inspector.disconnect();
    }
    parentPort.postMessage({
      type: "DEVELOPMENT_FINISHED",
      id: event.data.id,
      linear_bytes: memory?.buffer.byteLength,
      profiles: workerData.profilePath
        ? [
            path.basename(workerData.profilePath) + ".cpuprofile",
            path.basename(workerData.profilePath) + ".heapprofile",
          ]
        : [],
    });
    return;
  }
  const start = performance.now(),
    cpu = threadCpu();
  await handle(event);
  parentPort.postMessage({
    type: "DEVELOPMENT_MEASURE",
    id: event.data.id,
    ms: performance.now() - start,
    thread_cpu_ms: cpuDifference(cpu),
    phases: measurements.splice(0),
    memory: {
      ...process.memoryUsage(),
      linear_bytes: memory?.buffer.byteLength,
    },
  });
};
