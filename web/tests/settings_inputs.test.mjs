import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { optionInput } from '../src/features/settings/option-input.ts';

const originalFetch = globalThis.fetch;
const bytes = readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);
let loads = 0;
globalThis.fetch = async () => {
  loads++;
  return new Response(bytes, { headers: { 'Content-Type': 'application/wasm' } });
};
try {
  await assert.rejects(optionInput('mode', 'research'), /Unsupported option 'mode'/);
  assert.equal(await optionInput('crit', 'random'), 'random');
  assert.equal(await optionInput('start_fons', ''), null);
  await assert.rejects(optionInput('seed', '1.5'), /integer/);
  assert.equal(loads, 1, 'Settings reuse the shared WASM loader');
} finally {
  globalThis.fetch = originalFetch;
}
