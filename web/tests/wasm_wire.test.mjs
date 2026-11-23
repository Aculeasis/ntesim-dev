import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { NtesimWasm } from '../src/infrastructure/wasm/client.ts';
const { decodeWire } = createRequire(import.meta.url)('../../crates/wasm/js/wire.js');

// Independent v1 frame, shared with the Rust golden assertion.
const golden = Buffer.concat([
  Buffer.from([78, 84, 69, 1, 6, 4, 1, 0, 0, 0, 120, 5, 0, 0, 0, 0, 9, 4, 16, 0, 0, 0]),
  Buffer.from('9007199254740992'),
  Buffer.from([8]),
]);
assert.deepEqual(decodeWire(golden), ['x', 'x', '9007199254740992']);
for (let end = 0; end < golden.length; end++)
  assert.throws(() => decodeWire(golden.subarray(0, end)), /wire/);
assert.throws(() => decodeWire(Buffer.concat([golden, Buffer.from([0])])), /Trailing/);
assert.throws(() => decodeWire(Buffer.from([78, 84, 69, 1, 5, 0, 0, 0, 0])), /reference/);
assert.throws(() => decodeWire(Buffer.from([78, 84, 69, 2, 0])), /header/);

const bytes = readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);
const engine = await NtesimWasm.create(bytes);
const other = await NtesimWasm.create(bytes);
const input = JSON.parse(
  readFileSync(new URL('../../examples/research-crit.json', import.meta.url)),
);
input.actors[0].talent_levels = [1, 2, 3];
input.actors[0].character_max_level = 80;
input.model_assumptions = Object.fromEntries([
  ...Object.entries(input.model_assumptions ?? {}),
  ['__proto__', 'Unicode 😀, escaped \"text\", controls \n\t and 9007199254740993'],
]);
function compare(action, payload) {
  const binary = engine.call(action, payload);
  assert.deepEqual(binary, engine.call(action, payload, true));
  return binary;
}
compare('version');
for (const seed of [0n, 42n, 18446744073709551615n, 43n, 42n]) {
  compare('simulate_game', { ...input, seed });
  // Creating a second binding must never redirect the first instance's cache.
  assert.deepEqual(other.simulateGame({ ...input, seed }), engine.simulateGame({ ...input, seed }));
}
for (const count of [0, 1, 5]) {
  const batch = compare('simulate_game_batch', { scenario: input, start_seed: 100, count });
  assert.equal(batch.length, count);
  batch.forEach((report, i) =>
    assert.deepEqual(report, other.simulateGame({ ...input, seed: 100 + i })),
  );
}
const source = readFileSync(new URL('../../presets/studio/s1.nte', import.meta.url), 'utf8');
compare('invoke', { action: 'compile_script', payload: { source } });
assert.throws(
  () =>
    engine.call('simulate_game_batch', {
      scenario: input,
      start_seed: 18446744073709551615n,
      count: 2,
    }),
  /overflows u64/,
);
assert.throws(() => engine.call('unknown'), /unsupported action/);
compare('simulate_game', input);
for (const integer of [
  '-170141183460469231731687303715884105728',
  '340282366920938463463374607431768211455',
]) {
  const length = Buffer.alloc(4);
  length.writeUInt32LE(integer.length);
  const frame = Buffer.concat([Buffer.from([78, 84, 69, 1, 9, 4]), length, Buffer.from(integer)]);
  assert.equal(decodeWire(frame), integer);
}
console.log(
  'Binary transport: exact JSON parity, owned frames, independent engines and error recovery',
);
