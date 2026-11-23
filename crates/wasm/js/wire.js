/** Decoder for NTE wire v1. Transport only: no simulation/domain arithmetic. */
function decodeWire(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const strings = [];
  let offset = 4;
  if (
    bytes.length < 5 ||
    bytes[0] !== 78 ||
    bytes[1] !== 84 ||
    bytes[2] !== 69 ||
    bytes[3] !== 1
  )
    throw new Error("Unsupported NTE wire header.");
  function requireBytes(size) {
    if (offset + size > bytes.length)
      throw new Error("Truncated NTE wire frame.");
  }
  function uint32() {
    requireBytes(4);
    const value = view.getUint32(offset, true);
    offset += 4;
    return value;
  }
  function read(depth = 0) {
    if (depth > 512) throw new Error("NTE wire nesting limit exceeded.");
    requireBytes(1);
    switch (bytes[offset++]) {
      case 0:
        return null;
      case 1:
        return false;
      case 2:
        return true;
      case 3: {
        requireBytes(8);
        const value = view.getFloat64(offset, true);
        offset += 8;
        if (!Number.isFinite(value))
          throw new Error("Invalid NTE wire number.");
        return value;
      }
      case 4: {
        const length = uint32();
        requireBytes(length);
        const value = decoder.decode(bytes.subarray(offset, offset + length));
        offset += length;
        strings.push(value);
        return value;
      }
      case 5: {
        const index = uint32();
        if (index >= strings.length)
          throw new Error("Invalid NTE wire string reference.");
        return strings[index];
      }
      case 6: {
        const value = [];
        while (bytes[offset] !== 8) value.push(read(depth + 1));
        offset++;
        return value;
      }
      case 7: {
        const value = {};
        while (bytes[offset] !== 8) {
          const key = read(depth + 1);
          if (typeof key !== "string")
            throw new Error("Invalid NTE wire object key.");
          const item = read(depth + 1);
          // Match JSON.parse for __proto__ without invoking the legacy setter.
          if (key === "__proto__")
            Object.defineProperty(value, key, {
              value: item,
              writable: true,
              enumerable: true,
              configurable: true,
            });
          else value[key] = item;
        }
        offset++;
        return value;
      }
      case 9: {
        const text = read(depth + 1);
        if (
          typeof text !== "string" ||
          !/^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$/.test(text)
        )
          throw new Error("Invalid NTE wire exact number.");
        const value = Number(text);
        if (!Number.isFinite(value))
          throw new Error("Invalid NTE wire exact number.");
        return !/[.eE]/.test(text) && !Number.isSafeInteger(value)
          ? text
          : value;
      }
      case 10: {
        const text = read(depth + 1);
        if (typeof text !== "string")
          throw new Error("Invalid NTE wire raw JSON.");
        return globalThis.NtesimLosslessJson.parseLosslessJson(text);
      }
      default:
        throw new Error("Invalid NTE wire tag.");
    }
  }
  const result = read();
  if (offset !== bytes.length) throw new Error("Trailing NTE wire bytes.");
  return result;
}

const wireApi = { decodeWire };
if (typeof module !== "undefined" && module.exports) module.exports = wireApi;
globalThis.NtesimWire = wireApi;
