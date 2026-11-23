import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
const { parseLosslessJson } = createRequire(import.meta.url)(
  '../../crates/wasm/js/lossless-json.js',
);
const max = BigInt(Number.MAX_SAFE_INTEGER);
for (let delta = -100n; delta <= 100n; delta++) {
  for (const sign of [1n, -1n]) {
    const integer = sign * (max + delta),
      raw = integer.toString();
    const parsed = parseLosslessJson(`{"value":${raw},"quoted":"${raw}"}`);
    assert.equal(parsed.value, delta <= 0n ? Number(integer) : raw);
    assert.equal(parsed.quoted, raw);
  }
}
for (const value of ['0', '-0', '1.5', '1e30', '-1.25E-3', '12345678901234567890.0'])
  assert.deepEqual(parseLosslessJson(value), JSON.parse(value));
const strings = [
  '"9007199254740993"',
  'slash\\9007199254740993',
  '\n9007199254740993',
  '😀 18446744073709551615',
  '\\\\\\"9007199254740993\\\\',
  '\t\r\n\u2028\u2029\0',
  'ordinary text '.repeat(10000) + '9007199254740993',
];
assert.deepEqual(parseLosslessJson(JSON.stringify(strings)), strings);
for (const malformed of [
  '01',
  '1.',
  '1e',
  '{"a":1,}',
  '[1,,2]',
  '"bad\\x"',
  '"newline\n"',
  '"unterminated9007199254740993',
  '"unfinished\\"',
  '{"ok":9007199254740993, 9007199254740993 \t: 0}',
])
  assert.throws(() => parseLosslessJson(malformed), SyntaxError);
console.log(
  'Lossless JSON: integer boundaries, quoted/escaped content and malformed input preserved',
);
