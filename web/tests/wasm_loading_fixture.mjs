import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

export const wasmBytes = readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);

export function installFetchFixture(base) {
  const original = globalThis.fetch;
  const requests = [];
  globalThis.fetch = async (input, init) => {
    const request = new Request(input, init);
    const url = new URL(request.url);
    assert.equal(url.origin, base);
    assert.equal(request.method, 'GET');
    requests.push(url.pathname);
    switch (url.pathname) {
      case '/missing':
        return new Response(null, { status: 404 });
      case '/corrupt':
        return new Response('invalid wasm', { headers: { 'Content-Type': 'application/wasm' } });
      case '/engine.wasm':
      case '/fallback':
      case '/mime-parameters': {
        const mime =
          url.pathname === '/fallback'
            ? 'application/octet-stream'
            : url.pathname === '/mime-parameters'
              ? 'application/wasm; charset=utf-8'
              : 'application/wasm';
        return new Response(wasmBytes, { headers: { 'Content-Type': mime } });
      }
      default:
        throw new Error(`Unexpected WASM request: ${request.url}`);
    }
  };
  return {
    requests,
    restore: () => {
      globalThis.fetch = original;
    },
  };
}
