import '../../../../crates/wasm/js/lossless-json.js';

export function parseLosslessJson(text: string): unknown {
  const api = globalThis.NtesimLosslessJson;
  if (!api) throw new Error('Shared lossless JSON parser failed to initialize.');
  return api.parseLosslessJson(text);
}
