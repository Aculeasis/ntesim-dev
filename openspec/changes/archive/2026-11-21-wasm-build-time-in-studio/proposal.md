# Proposal

## Why

Studio currently displays the build-start instant of the Vite web GUI bundle in its header pill. During development or when serving static artifacts, the web GUI and the WebAssembly simulation engine (`ntesim-wasm`) can be compiled at different times, causing the displayed timestamp to misrepresent the actual engine build instant. Embedding the build timestamp into the WASM binary and extracting it on the client ensures the UI accurately reflects when the simulation engine was built.

## What Changes

- Embed the build-start UTC ISO 8601 instant into `ntesim-wasm` at compile time via a build script (`crates/wasm/build.rs`), honoring `NTESIM_BUILD_STARTED_AT` when provided or computing the current UTC instant when compiling standalone.
- Expose `build_started_at` in `WasmVersionInfo` and via `engine.buildStartedAt()` on `NtesimWasm`.
- Update Studio's header build identity component and state to extract and display the WASM engine's embedded build instant once the engine is loaded, retaining the bundled static build instant as an initial or fallback representation.
- Ensure browser distribution packaging passes `NTESIM_BUILD_STARTED_AT` consistently to both `cargo build -p ntesim-wasm` and Vite staging so the packaged distribution artifacts agree on the build instant.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: The header build timestamp reflects the compiled WebAssembly engine's embedded build instant extracted by the client upon engine loading.

## Impact

- `crates/wasm/build.rs`, `crates/wasm/Cargo.toml`, `crates/wasm/src/model.rs`, `crates/wasm/src/dispatch.rs`, `crates/wasm/src/transport.rs`.
- `crates/wasm/js/runtime.js` and `web/src/infrastructure/wasm/client.ts`.
- `web/src/app/BuildIdentity.tsx`, `web/src/app/App.tsx`, `web/src/features/editor/useConfig.ts`.
- `tools/browser_distribution.py`.
- Unit tests and browser acceptance checks.
