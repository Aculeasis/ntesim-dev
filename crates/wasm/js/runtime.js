// Shared browser/Node runtime. Load generated bindings, lossless-json and wire first.
const losslessJson = globalThis.NtesimLosslessJson;
const createBindings = globalThis.NtesimCreateBindings;
const wire = globalThis.NtesimWire;

function stringifyWasmInput(value) {
  if (typeof value === "bigint") return value.toString();
  if (value === null) return "null";
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "number")
    return Number.isFinite(value) ? JSON.stringify(value) : "null";
  if (typeof value === "boolean") return value ? "true" : "false";
  if (Array.isArray(value))
    return `[${value.map(stringifyWasmInput).join(",")}]`;
  if (typeof value === "object") {
    return `{${Object.entries(value)
      .filter(([, entry]) => entry !== undefined)
      .map(
        ([key, entry]) => `${JSON.stringify(key)}:${stringifyWasmInput(entry)}`,
      )
      .join(",")}}`;
  }
  return "null";
}

class NtesimWasm {
  constructor(bindings) {
    this.bindings = bindings;
  }

  static async create(source) {
    const bindings = createBindings();
    const module =
      source instanceof WebAssembly.Module
        ? source
        : await WebAssembly.compile(source);
    bindings.initSync({ module });
    return new NtesimWasm(bindings);
  }

  static async fromUrl(source) {
    const response = await fetch(source);
    if (!response.ok)
      throw new Error(
        `Failed to fetch WebAssembly binary (${response.status})`,
      );
    const mime = response.headers.get("Content-Type")?.trim().toLowerCase();
    if (
      typeof WebAssembly.instantiateStreaming === "function" &&
      mime === "application/wasm"
    ) {
      const bindings = createBindings();
      await bindings({ module_or_path: response });
      return new NtesimWasm(bindings);
    }
    return NtesimWasm.create(await response.arrayBuffer());
  }

  responseBytes(fnName, payload = null, json = false) {
    const action = fnName.replace(/^ntesim_wasm_/, "");
    const input =
      payload == null
        ? ""
        : typeof payload === "string"
          ? payload
          : stringifyWasmInput(payload);
    return this.bindings.transport(action, input, json);
  }

  exportJson(fnName, payload = null) {
    return new TextDecoder().decode(this.responseBytes(fnName, payload, true));
  }

  /** Diagnostic JSON mode is for exports and paired transport benchmarks. */
  call(fnName, payload = null, json = false) {
    const bytes = this.responseBytes(fnName, payload, json);
    const parsed = json
      ? losslessJson.parseLosslessJson(new TextDecoder().decode(bytes))
      : wire.decodeWire(bytes);
    if (parsed && parsed.status === "error")
      throw new Error(
        `ntesim wasm error (${parsed.action || fnName}): ${parsed.error}`,
      );
    return parsed;
  }

  version() {
    return this.call("ntesim_wasm_version");
  }

  buildStartedAt() {
    return this.version().build_started_at;
  }

  /**
   * @param {any} fixture
   * @param {string} rotation
   * @param {number} [seed]
   */
  simulateRotation(fixture, rotation, seed) {
    return this.call("ntesim_wasm_simulate_rotation", {
      fixture,
      rotation,
      seed,
    });
  }

  /**
   * @param {any} scenario
   */
  simulateGame(scenario) {
    return this.call("ntesim_wasm_simulate_game", scenario);
  }

  /**
   * @param {any} scenario
   * @param {number} startSeed
   * @param {number} count
   */
  simulateGameBatch(scenario, startSeed, count) {
    return this.call("ntesim_wasm_simulate_game_batch", {
      scenario,
      start_seed: startSeed,
      count,
    });
  }

  simulateGameProgram(program) {
    return this.call("ntesim_wasm_simulate_game_program", program);
  }

  /**
   * @param {any} program
   * @param {number} startSeed
   * @param {number} count
   */
  simulateGameProgramBatch(program, startSeed, count) {
    return this.call("ntesim_wasm_simulate_game_program_batch", {
      program,
      start_seed: startSeed,
      count,
    });
  }

  /**
   * @param {any} scenario
   * @param {any} request
   */
  runBatch(scenario, request) {
    return this.call("ntesim_wasm_run_batch", { scenario, request });
  }

  /**
   * @param {any[]} [cases]
   */
  verifyTabular(cases) {
    return this.call("ntesim_wasm_verify_tabular", { cases });
  }

  /**
   * @param {string} [variantsCsv]
   */
  enumerateCompositions(variantsCsv) {
    return this.call("ntesim_wasm_enumerate_compositions", {
      variants_csv: variantsCsv,
    });
  }

  /**
   * @param {string} action
   * @param {any} payload
   */
  invoke(action, payload) {
    return this.call("ntesim_wasm_invoke", { action, payload });
  }
}

globalThis.NtesimWasm = NtesimWasm;
globalThis.NtesimStringifyWasmInput = stringifyWasmInput;
