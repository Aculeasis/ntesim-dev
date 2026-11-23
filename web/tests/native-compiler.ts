import { readFileSync } from 'node:fs';
import { NtesimWasm } from '../src/infrastructure/wasm/client';
import type { OptionValue } from '../src/domain/contracts/display';
const wasm = await NtesimWasm.create(
  readFileSync(new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url)),
);
export const projectActor = wasm.projectActor.bind(wasm);
export { wasm as engine };
export const compileNte = (
  source: string,
  options: { maxSteps?: number; maxActions?: number } = {},
) => wasm.compileScript(source, options);
export const editOptions = (source: string, changes: Record<string, OptionValue>) =>
  wasm.editOptions(source, changes);
export { getPresetSource } from '../src/domain/presets';
