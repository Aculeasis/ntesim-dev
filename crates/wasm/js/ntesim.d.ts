export interface NtesimVersionInfo {
  version: string;
  content_id: string;
  content_cutoff: string;
  status: string;
  game_dps_computed: boolean;
  adapter: string;
  target_arch: string;
  coverage_sha256: string;
  variants_sha256: string;
}

export interface WasmEnvironmentMetadata {
  adapter: string;
  target_arch: string;
  runtime_model: string;
  content_cutoff: string;
  parity_verified: boolean;
}

export interface WasmInvokeSuccessResponse<T = any> {
  status: "ok";
  action: string;
  environment: WasmEnvironmentMetadata;
  data: T;
}

export declare class NtesimWasm {
  static create(source: BufferSource | WebAssembly.Module): Promise<NtesimWasm>;
  static fromUrl(source: string | URL): Promise<NtesimWasm>;
  call<T = unknown>(fnName: string, payload?: unknown, json?: boolean): T;
  responseBytes(fnName: string, payload?: unknown, json?: boolean): Uint8Array;
  exportJson(fnName: string, payload?: unknown): string;

  version(): NtesimVersionInfo;
  simulateRotation(fixture: any, rotation: string, seed?: number): any;
  simulateGame(scenario: any): any;
  simulateGameBatch(scenario: any, startSeed: number, count: number): any[];
  simulateGameProgram(program: any): any;
  simulateGameProgramBatch(
    program: any,
    startSeed: number,
    count: number,
  ): any[];
  runBatch(scenario: any, request: any): any;
  verifyTabular(cases?: any[]): any;
  enumerateCompositions(variantsCsv?: string): any;
  invoke<T = any>(action: string, payload: any): WasmInvokeSuccessResponse<T>;
}

export declare function stringifyWasmInput(value: unknown): string;
declare global {
  var NtesimWasm: typeof import("./ntesim.js").NtesimWasm;
  var NtesimStringifyWasmInput: typeof stringifyWasmInput;
}
