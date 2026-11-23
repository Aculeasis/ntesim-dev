import { NtesimWasm } from './client';
let engine: Promise<NtesimWasm> | undefined;
export function loadEngine() {
  return (engine ??= NtesimWasm.fromUrl(
    new URL('../../../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
  ).catch((error) => {
    engine = undefined;
    throw error;
  }));
}
