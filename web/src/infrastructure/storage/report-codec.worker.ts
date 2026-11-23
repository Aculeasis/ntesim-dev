import { decodeContainer, encodeContainer } from './report-container';
import type { CodecRequest, CodecResponse } from './report-codec';

const loadCodec = async () => {
  const [codec, { default: wasmUrl }] = await Promise.all([
    import('@bokuweb/zstd-wasm'),
    import('@report-codec-wasm?url'),
  ]);
  await codec.init(wasmUrl);
  return codec;
};
self.onmessage = async ({ data }: MessageEvent<CodecRequest>) => {
  try {
    const output = await (data.operation === 'encode' ? encodeContainer : decodeContainer)(
      new Uint8Array(data.buffer),
      loadCodec,
    );
    self.postMessage({ id: data.id, buffer: output.buffer } satisfies CodecResponse, {
      transfer: [output.buffer],
    });
  } catch (error) {
    self.postMessage({
      id: data.id,
      error: error instanceof Error ? error.message : 'Report codec failed.',
    } satisfies CodecResponse);
  }
};
