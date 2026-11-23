import { checkReportFileSize, checkReportJsonSize, REPORT_JSON_LIMIT } from './report-limits';

export interface ReportCodec {
  compress(bytes: Uint8Array, level: number): Uint8Array;
  decompress(bytes: Uint8Array): Uint8Array;
}
const signature = new TextEncoder().encode('NTESIMRP');
export const isReportContainer = (bytes: Uint8Array) =>
  signature.every((byte, i) => bytes[i] === byte);
const damaged = () => new Error('Damaged compressed report container or Zstandard frame.');

// Parse the complete ordinary frame before calling the codec, which trusts FCS
// when allocating its fixed destination. Keep wide metadata as BigInt throughout.
export function preflightFrame(frame: Uint8Array, expected: number, limit = REPORT_JSON_LIMIT) {
  checkReportJsonSize(expected, limit);
  let offset = 0;
  const read = (count: number): bigint => {
    if (offset + count > frame.length) throw damaged();
    let value = 0n;
    for (let i = 0; i < count; i++) value |= BigInt(frame[offset++]) << BigInt(i * 8);
    return value;
  };
  if (read(4) !== 0xfd2fb528n) throw damaged();
  const descriptor = Number(read(1));
  if (descriptor & 0x18 || descriptor & 3) throw damaged();
  const single = !!(descriptor & 0x20),
    flag = descriptor >>> 6;
  let window = 0n;
  if (!single) {
    const wd = Number(read(1)),
      base = 1n << BigInt(10 + (wd >>> 3));
    window = base + (base / 8n) * BigInt(wd & 7);
  }
  const sizeBytes = flag === 0 ? (single ? 1 : 0) : 1 << flag;
  if (!sizeBytes) throw new Error('Unsupported Zstandard frame: a known decoded size is required.');
  const contentSize = read(sizeBytes) + (flag === 1 ? 256n : 0n);
  if (single) window = contentSize;
  if (contentSize > BigInt(limit) || window > BigInt(limit))
    throw new Error('Report decoded size or Zstandard window exceeds the 100 MB limit.');
  if (contentSize !== BigInt(expected) || !window) throw damaged();
  let last = false;
  while (!last) {
    const block = Number(read(3)),
      type = (block >>> 1) & 3,
      size = block >>> 3;
    last = !!(block & 1);
    if (type === 3 || size > 131072 || BigInt(size) > window) throw damaged();
    offset += type === 1 ? 1 : size;
    if (offset > frame.length) throw damaged();
  }
  if (descriptor & 4) read(4);
  if (offset !== frame.length) throw damaged();
}

export async function encodeContainer(
  json: Uint8Array,
  loadCodec: () => Promise<ReportCodec>,
  limit = REPORT_JSON_LIMIT,
): Promise<Uint8Array<ArrayBuffer>> {
  checkReportJsonSize(json.length, limit);
  const hash = new Uint8Array(
    await crypto.subtle.digest('SHA-256', json as Uint8Array<ArrayBuffer>),
  );
  const codec = await loadCodec(),
    frame = codec.compress(json, 9);
  preflightFrame(frame, json.length, limit);
  checkReportFileSize(48 + frame.length);
  const output = new Uint8Array(48 + frame.length);
  output.set(signature);
  output[8] = 1;
  output[9] = 1;
  new DataView(output.buffer).setUint32(12, json.length, true);
  output.set(hash, 16);
  output.set(frame, 48);
  return output;
}

export async function decodeContainer(
  bytes: Uint8Array,
  loadCodec: () => Promise<ReportCodec>,
  limit = REPORT_JSON_LIMIT,
): Promise<Uint8Array<ArrayBuffer>> {
  checkReportFileSize(bytes.length);
  if (bytes.length < 49 || !isReportContainer(bytes)) throw damaged();
  if (bytes[8] !== 1 || bytes[9] !== 1)
    throw new Error('Unsupported compressed report version or codec.');
  if (bytes[10] || bytes[11]) throw new Error('Unsupported compressed report flags.');
  const size = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(12, true);
  checkReportJsonSize(size, limit);
  const frame = bytes.subarray(48);
  preflightFrame(frame, size, limit);
  const codec = await loadCodec();
  let decoded: Uint8Array;
  try {
    decoded = codec.decompress(frame);
  } catch {
    throw new Error('Damaged Zstandard data or decoded output exceeds its declared capacity.');
  }
  if (decoded.length !== size) throw damaged();
  const hash = new Uint8Array(
    await crypto.subtle.digest('SHA-256', decoded as Uint8Array<ArrayBuffer>),
  );
  if (!hash.every((byte, i) => byte === bytes[16 + i]))
    throw new Error('Damaged report: SHA-256 checksum mismatch.');
  return decoded as Uint8Array<ArrayBuffer>;
}
