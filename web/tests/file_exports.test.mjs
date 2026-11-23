import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  init,
  compress,
  decompress,
} from '../../cache/web/node_modules/@bokuweb/zstd-wasm/dist/common/index.node.js';
import {
  encodeContainer,
  decodeContainer,
  preflightFrame,
} from '../src/infrastructure/storage/report-container.ts';
import {
  attachPngReport,
  extractPngReport,
  pngChunk,
} from '../src/infrastructure/storage/report-png.ts';
import {
  checkReportFileSize,
  checkReportJsonSize,
} from '../src/infrastructure/storage/report-limits.ts';
import { serializeReportBundle } from '../src/infrastructure/storage/files.ts';
import { captureDimensions } from '../src/features/import-export/report-capture.ts';
import {
  generateTimestamp,
  generateConfigFilename,
  generateReportFilename,
} from '../src/infrastructure/storage/files.ts';

const d = new Date(2026, 10, 14, 19, 30, 5);
assert.equal(generateTimestamp(d), '2026-11-14_19-30-05');

const reportName = generateReportFilename();
assert.match(reportName, /^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}-report\.json$/);

const configName = generateConfigFilename([
  'character_1036',
  'character_1075',
  'character:1003',
  'daffodill',
]);
assert.match(configName, /^\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}-zankou-iroi-sakiri-daffodill\.nte$/);

for (const extension of ['json', 'nter', 'png'])
  assert.ok(generateReportFilename(extension).endsWith(`-report.${extension}`));
assert.throws(() => serializeReportBundle({ result: null }), /No delivered/);
const state = {
  source: '# executed λ漢字',
  result: {
    aggregate: {},
    report: { exact: '123456789012345678901/7', wide: '18446744073709551615' },
  },
  status: 'stopped',
  failure: null,
  failedTrial: null,
};
const pretty = serializeReportBundle(state),
  compact = serializeReportBundle(state, true);
assert.deepEqual(JSON.parse(compact), JSON.parse(pretty));
assert.ok(pretty.includes('\n') && !compact.includes('\n'));
assert.ok(new TextEncoder().encode(compact).length > compact.length);
assert.match(compact, /123456789012345678901\/7/);
assert.equal(JSON.parse(compact).source, state.source);

await init();
let loads = 0,
  decodes = 0;
const load = async () => {
  loads++;
  return {
    compress,
    decompress: (bytes) => {
      decodes++;
      return decompress(bytes);
    },
  };
};
const bytes = new TextEncoder().encode('{"wide":18446744073709551615,"text":"λ漢字"}');
const container = await encodeContainer(bytes, load);
assert.deepEqual(await decodeContainer(container, load), bytes);
const rejectBeforeCodec = async (input, pattern = /Damaged|Unsupported|limit|size/) => {
  const before = loads;
  await assert.rejects(decodeContainer(input, load), pattern);
  assert.equal(
    loads,
    before,
    'untrusted metadata must fail before codec initialization/allocation',
  );
};
for (const [offset, value] of [
  [0, 0],
  [8, 2],
  [9, 2],
  [10, 1],
  [11, 1],
]) {
  const bad = container.slice();
  bad[offset] = value;
  await rejectBeforeCodec(bad);
}
const badHash = container.slice();
badHash[16] ^= 1;
await assert.rejects(decodeContainer(badHash, load), /SHA-256/);
for (const size of [0, 100_000_001, 0xffffffff]) {
  const bad = container.slice();
  new DataView(bad.buffer).setUint32(12, size, true);
  await rejectBeforeCodec(bad);
}
const framed = (frame, size) => {
  const bad = new Uint8Array(48 + frame.length);
  bad.set(container.subarray(0, 48));
  bad.set(frame, 48);
  new DataView(bad.buffer).setUint32(12, size, true);
  return bad;
};
const magic = [0x28, 0xb5, 0x2f, 0xfd];
for (const frame of [
  [...magic, 0, 0, 1, 0, 0], // unknown content size
  [...magic, 0xe0, 255, 255, 255, 255, 255, 255, 255, 255, 1, 0, 0], // huge 64-bit FCS
  [...magic, 0x80, 255, 1, 0, 0, 0, 1, 0, 0], // huge window
  [...magic, 0x21, 1, 1, 1, 0, 0], // dictionary
  [...magic, 0x28, 1, 1, 0, 0], // reserved descriptor
  [...magic, 0x20, 1, 7, 0, 0], // reserved block
  [...magic, 0x24, 1, 9, 0, 0, 65], // missing checksum
])
  await rejectBeforeCodec(framed(new Uint8Array(frame), 1));
await rejectBeforeCodec(new Uint8Array([...container, ...container.subarray(48)]));
await rejectBeforeCodec(new Uint8Array([...container, 0]));
await rejectBeforeCodec(container.subarray(0, container.length - 1));

// Frame declares 32 bytes, advertises a bounded 1KB window, then stores 64 raw
// bytes. Preflight admits allocation of exactly 32; the real decoder must fail.
const forgedFrame = new Uint8Array([
  ...magic,
  0x80,
  0,
  32,
  0,
  0,
  0,
  1 | ((64 << 3) & 255),
  (64 << 3) >>> 8,
  0,
  ...new Array(64).fill(65),
]);
preflightFrame(forgedFrame, 32);
const beforeDecode = decodes;
await assert.rejects(decodeContainer(framed(forgedFrame, 32), load), /capacity/);
assert.equal(decodes, beforeDecode + 1);
await assert.rejects(decodeContainer(container, load, bytes.length - 1), /limit/);
assert.deepEqual(
  await decodeContainer(container, load),
  bytes,
  'valid operations recover after errors',
);
const decoderSource = readFileSync(
  new URL(
    '../../cache/web/node_modules/@bokuweb/zstd-wasm/dist/common/simple/decompress.js',
    import.meta.url,
  ),
  'utf8',
);
assert.match(decoderSource, /malloc\(size\)/);
assert.match(decoderSource, /_decompress\(heap, size, src, buf.byteLength\)/);
assert.doesNotMatch(decoderSource, /while\s*\(/);

// A complete ordinary 1x1 PNG, including valid pixel data, remains byte intact
// except for the new private chunk; import never needs an Image/canvas API.
const png = new Uint8Array(
  Buffer.from(
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=',
    'base64',
  ),
);
// Build the IDAT CRC explicitly so the fixture is independent of copied metadata.
const idatLength = new DataView(png.buffer).getUint32(33);
const { pngCrc } = await import('../src/infrastructure/storage/report-png.ts');
new DataView(png.buffer).setUint32(41 + idatLength, pngCrc(png.subarray(37, 41 + idatLength)));
const attached = attachPngReport(png, container);
assert.deepEqual(extractPngReport(attached), container);
assert.deepEqual(await decodeContainer(extractPngReport(attached), load), bytes);
assert.throws(() => extractPngReport(png), /no embedded simulation report/);
assert.throws(() => attachPngReport(attached, container), /already/);
const duplicate = new Uint8Array([
  ...attached.subarray(0, -12),
  ...pngChunk('ntEr', container),
  ...attached.subarray(-12),
]);
assert.throws(() => extractPngReport(duplicate), /duplicate/);
const corrupt = attached.slice();
corrupt[corrupt.length - 17] ^= 1;
assert.throws(() => extractPngReport(corrupt), /CRC/);
for (const bad of [
  attached.subarray(0, -1),
  new Uint8Array([...attached, 0]),
  attached.subarray(0, 20),
])
  assert.throws(() => extractPngReport(bad), /Damaged PNG/);
const hugeChunk = png.slice();
new DataView(hugeChunk.buffer).setUint32(33, 0xffffffff);
assert.throws(() => extractPngReport(hugeChunk), /Damaged PNG/);
checkReportFileSize(100_000_000);
checkReportJsonSize(100_000_000);
assert.throws(() => checkReportFileSize(100_000_001), /file limit/);
assert.throws(() => checkReportJsonSize(100_000_001), /decoded size/);
for (const [w, h] of [
  [0, 1],
  [1, 32768],
  [8000, 8000],
  [NaN, 1],
])
  assert.throws(() => captureDimensions(w, h), /too large/);
captureDimensions(390, 12000);
assert.throws(() => attachPngReport(png, new Uint8Array(100_000_000)), /file limit/);
assert.throws(() => extractPngReport(new Uint8Array(100_000_001)), /file limit/);

const { reportCodec } = await import('../src/infrastructure/storage/report-codec.ts');
const originalWorker = globalThis.Worker;
let active = 0,
  terminated = 0;
const messages = [];
globalThis.Worker = class {
  constructor(url, options) {
    assert.match(String(url), /report-codec.worker.ts$/);
    assert.equal(options.type, 'module');
    assert.equal(++active, 1);
  }
  postMessage(message, transfer) {
    assert.equal(message.buffer, transfer[0]);
    messages.push(message);
    queueMicrotask(() => {
      this.onmessage({ data: { id: message.id - 1, error: 'stale' } });
      this.onmessage({
        data:
          message.operation === 'decode'
            ? { id: message.id, error: 'damaged test frame' }
            : { id: message.id, buffer: message.buffer },
      });
    });
  }
  terminate() {
    active--;
    terminated++;
  }
};
try {
  const first = reportCodec('decode', new Uint8Array([1]));
  const second = reportCodec('encode', new Uint8Array([2]));
  await assert.rejects(first, /damaged test/);
  assert.deepEqual(await second, new Uint8Array([2]));
  assert.equal(active, 0);
  assert.equal(terminated, 2);
  const controller = new AbortController();
  controller.abort();
  await assert.rejects(reportCodec('encode', new Uint8Array([3]), controller.signal), /cancelled/);
  assert.equal(messages.length, 2);
} finally {
  globalThis.Worker = originalWorker;
}
