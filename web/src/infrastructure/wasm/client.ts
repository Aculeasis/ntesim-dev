import type { ScenarioProgramRequest, ScenarioRequest } from '../../domain/contracts/scenario';
import type { ScenarioReport } from '../../domain/contracts/report';
import type { TrialAggregate, TrialSample } from '../../domain/contracts/trials';
import type {
  SourceExecution,
  SourceSampleRequest,
  SourceTrialUpdate,
} from '../../domain/contracts/source';
import type { CompiledSourceProgram } from '../../domain/contracts/script';
import type { ActorProjectionRequest, ActorProjection } from '../../domain/contracts/build';
import type { NtesimWasm as Runtime } from '../../../../crates/wasm/js/ntesim.js';
import '../../../../target/wasm-bindgen/bindings.js';
import '../../../../crates/wasm/js/lossless-json.js';
import '../../../../crates/wasm/js/wire.js';
import '../../../../crates/wasm/js/runtime.js';
import type { CompiledConfig, OptionValue } from '../../domain/contracts/display';
import { throwDiagnostics, type SourceResponse } from './script';
import type {
  InspectionResponse,
  ScriptDiagnostic,
  SourceCompileResponse,
} from '../../domain/contracts/script';

export interface WasmVersionInfo {
  version: string;
  engine_source_sha256: string;
  content_id: string;
  content_cutoff: string;
  status: string;
  game_dps_computed: boolean;
  adapter: string;
  target_arch: string;
  coverage_sha256: string;
  variants_sha256: string;
  build_started_at: string;
}

export function stringifyWasmInput(value: unknown): string {
  return globalThis.NtesimStringifyWasmInput(value);
}

/** Presentation adapter over the shared generated-bindings client. */
export class NtesimWasm {
  private constructor(private runtime: Runtime) {}
  static async create(source: BufferSource | WebAssembly.Module) {
    return new NtesimWasm(await globalThis.NtesimWasm.create(source));
  }
  static async fromUrl(source: string | URL) {
    return new NtesimWasm(await globalThis.NtesimWasm.fromUrl(source));
  }
  call(name: string, payload: unknown = null, json = false): unknown {
    return this.runtime.call(name, payload, json);
  }
  simulateGame(request: ScenarioRequest): ScenarioReport {
    return this.call('ntesim_wasm_simulate_game', request) as ScenarioReport;
  }
  simulateGameBatch(scenario: ScenarioRequest, startSeed: number, count: number): ScenarioReport[] {
    return this.call('ntesim_wasm_simulate_game_batch', {
      scenario,
      start_seed: startSeed,
      count,
    }) as ScenarioReport[];
  }
  simulateGameProgram(request: ScenarioProgramRequest): ScenarioReport {
    return this.call('ntesim_wasm_simulate_game_program', request) as ScenarioReport;
  }
  simulateGameProgramBatch(
    program: ScenarioProgramRequest,
    startSeed: number,
    count: number,
  ): ScenarioReport[] {
    return this.call('ntesim_wasm_simulate_game_program_batch', {
      program,
      start_seed: startSeed,
      count,
    }) as ScenarioReport[];
  }
  version(): WasmVersionInfo {
    return this.call('ntesim_wasm_version') as WasmVersionInfo;
  }
  buildStartedAt(): string {
    return this.version().build_started_at;
  }
  projectActor(request: ActorProjectionRequest): ActorProjection {
    return (this.invoke('actor_projection', request) as { data: ActorProjection }).data;
  }
  compileSource(
    source: string,
    limits: { maxSteps?: number; maxActions?: number } = {},
  ): SourceCompileResponse {
    return (this.invoke('compile_source', { source, limits }) as { data: SourceCompileResponse })
      .data;
  }
  inspectScript(source: string): InspectionResponse {
    return (this.invoke('inspect_script', { source }) as { data: InspectionResponse }).data;
  }
  startSourceTrials(compiled: CompiledSourceProgram): void {
    this.invoke('start_source_trials', compiled);
  }
  resumeSourceTrials(
    compiled: CompiledSourceProgram,
    samples: TrialSample[],
    elapsedMs: number,
  ): SourceTrialUpdate {
    return (
      this.invoke('resume_source_trials', { compiled, samples, elapsed_ms: elapsedMs }) as {
        data: SourceTrialUpdate;
      }
    ).data;
  }
  snapshotSourceTrials(elapsedMs: number): TrialAggregate {
    return (this.invoke('snapshot_source_trials', elapsedMs) as { data: TrialAggregate }).data;
  }
  stepSourceTrials(offset: number, count: number, elapsedMs: number): SourceTrialUpdate {
    return (
      this.invoke('step_source_trials', { offset, count, elapsed_ms: elapsedMs }) as {
        data: SourceTrialUpdate;
      }
    ).data;
  }
  runSourceSample(payload: SourceSampleRequest): SourceExecution {
    return (this.invoke('run_source_sample', payload) as { data: SourceExecution }).data;
  }
  compileScript(
    source: string,
    limits: { maxSteps?: number; maxActions?: number } = {},
  ): CompiledConfig {
    const response = (this.invoke('compile_script', { source, limits }) as { data: SourceResponse })
      .data;
    throwDiagnostics(response.diagnostics);
    if (!response.compiled) throw new Error('Missing compiled source response.');
    return response.compiled;
  }
  validateScript(source: string): ScriptDiagnostic[] {
    return (this.invoke('validate_script', { source }) as { data: ScriptDiagnostic[] }).data;
  }
  editOptions(source: string, changes: Record<string, OptionValue>): string {
    const response = (
      this.invoke('edit_script_options', { source, changes }) as {
        data: { source: string | null; diagnostics: ScriptDiagnostic[] };
      }
    ).data;
    throwDiagnostics(response.diagnostics);
    if (response.source === null) throw new Error('Missing edited source response.');
    return response.source;
  }
  parseOptionInput(name: string, text: string): OptionValue {
    return (this.invoke('parse_option_input', { name, text }) as { data: OptionValue }).data;
  }
  simulateRotation(fixture: unknown, rotation: string, seed: number) {
    return this.call('ntesim_wasm_simulate_rotation', { fixture, rotation, seed });
  }
  runBatch(scenario: unknown, request: unknown) {
    return this.call('ntesim_wasm_run_batch', { scenario, request });
  }
  verifyTabular(cases: unknown) {
    return this.call('ntesim_wasm_verify_tabular', { cases });
  }
  enumerateCompositions(variantsCsv?: string) {
    return this.call('ntesim_wasm_enumerate_compositions', { variants_csv: variantsCsv });
  }
  invoke(action: string, payload: unknown) {
    return this.call('ntesim_wasm_invoke', { action, payload });
  }
}
