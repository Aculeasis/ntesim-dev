// CommonJS entry; browser builds import the same runtime through side effects.
require('../../../target/wasm-bindgen/bindings.js');
require('./lossless-json.js');
require('./wire.js');
require('./runtime.js');
module.exports = {
  NtesimWasm: globalThis.NtesimWasm,
  stringifyWasmInput: globalThis.NtesimStringifyWasmInput,
};
