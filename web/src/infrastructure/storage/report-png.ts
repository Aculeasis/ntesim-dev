import { checkReportFileSize } from './report-limits';

const signature = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10]);
export const isPng = (bytes: Uint8Array) => signature.every((byte, i) => bytes[i] === byte);
const table = Uint32Array.from({ length: 256 }, (_, n) => {
  for (let i = 0; i < 8; i++) n = n & 1 ? 0xedb88320 ^ (n >>> 1) : n >>> 1;
  return n >>> 0;
});
export function pngCrc(bytes: Uint8Array) {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = table[(crc ^ byte) & 255] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}
export function pngChunk(type: string, data: Uint8Array) {
  const chunk = new Uint8Array(12 + data.length),
    view = new DataView(chunk.buffer);
  view.setUint32(0, data.length);
  chunk.set(new TextEncoder().encode(type), 4);
  chunk.set(data, 8);
  view.setUint32(8 + data.length, pngCrc(chunk.subarray(4, 8 + data.length)));
  return chunk;
}

function scanPng(bytes: Uint8Array) {
  checkReportFileSize(bytes.length);
  const damaged = () => new Error('Damaged PNG: invalid chunks, structure or CRC checksum.');
  if (!isPng(bytes)) throw damaged();
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let offset = 8,
    attachment: Uint8Array | undefined,
    end = -1,
    image = false,
    imageEnded = false;
  while (offset < bytes.length) {
    if (offset + 12 > bytes.length) throw damaged();
    const length = view.getUint32(offset),
      next = offset + 12 + length;
    if (length > 0x7fffffff || next > bytes.length) throw damaged();
    const type = String.fromCharCode(...bytes.subarray(offset + 4, offset + 8));
    if (!/^[A-Za-z]{2}[A-Z][A-Za-z]$/.test(type)) throw damaged();
    if (pngCrc(bytes.subarray(offset + 4, next - 4)) !== view.getUint32(next - 4)) throw damaged();
    if (offset === 8) {
      if (
        type !== 'IHDR' ||
        length !== 13 ||
        !view.getUint32(offset + 8) ||
        !view.getUint32(offset + 12)
      )
        throw damaged();
    } else if (type === 'IHDR') throw damaged();
    if (type === 'IDAT') {
      if (imageEnded) throw damaged();
      image = true;
    } else if (image) imageEnded = true;
    if (type === 'ntEr') {
      if (attachment) throw new Error('Damaged PNG: duplicate embedded simulation report.');
      attachment = bytes.subarray(offset + 8, next - 4);
    } else if (type === 'IEND') {
      if (length || !image || next !== bytes.length) throw damaged();
      end = offset;
    } else if (/^[A-Z]/.test(type) && !['IHDR', 'PLTE', 'IDAT'].includes(type)) throw damaged();
    offset = next;
  }
  if (end < 0) throw damaged();
  return { attachment, end };
}

// Parsing uses bounded byte views only; it never decodes image pixels/metadata.
export function extractPngReport(bytes: Uint8Array) {
  const { attachment } = scanPng(bytes);
  if (!attachment)
    throw new Error(
      'This PNG contains no embedded simulation report. Share the original PNG file.',
    );
  return attachment;
}
export function attachPngReport(png: Uint8Array, container: Uint8Array) {
  const { attachment, end } = scanPng(png);
  if (attachment) throw new Error('PNG already contains an embedded simulation report.');
  checkReportFileSize(png.length + container.length + 12);
  const output = new Uint8Array(png.length + container.length + 12);
  output.set(png.subarray(0, end));
  const chunk = pngChunk('ntEr', container);
  output.set(chunk, end);
  output.set(png.subarray(end), end + chunk.length);
  return output;
}
