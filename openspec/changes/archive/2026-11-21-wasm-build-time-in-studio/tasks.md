# Tasks

## 1. WASM Engine Build Timestamp Embedding

- [x] 1.1 Create `crates/wasm/build.rs` to emit `NTESIM_WASM_BUILD_TIME` (reading `NTESIM_BUILD_STARTED_AT` or computing current UTC ISO 8601 timestamp) and configure `crates/wasm/Cargo.toml`.
- [x] 1.2 Update `crates/wasm/src/model.rs`, `crates/wasm/src/dispatch.rs`, and `crates/wasm/src/transport.rs` to include `build_started_at` in `WasmVersionInfo` and dispatch actions, and verify with `python tools/cargo.py test -p ntesim-wasm`.

## 2. WASM Client and Runtime Extraction

- [x] 2.1 Expose `buildStartedAt()` in `crates/wasm/js/runtime.js` and typed `web/src/infrastructure/wasm/client.ts`, and verify with `node tools/web.mjs test tests/wasm_contract.test.mjs`.

## 3. Web UI Build Time Display Logic

- [x] 3.1 Update `web/src/app/BuildIdentity.tsx`, `web/src/features/editor/useConfig.ts`, and `web/src/app/App.tsx` to display the WASM engine's extracted build instant when loaded, falling back to `build.build_started_at`.
- [x] 3.2 Extend `web/tests/build-identity-cases.tsx` to verify custom and extracted WASM build timestamps and run `node tools/web.mjs test tests/studio.test.tsx`.

## 4. Distribution and Integration Checks

- [x] 4.1 Update `tools/browser_distribution.py` to ensure `os.environ[INSTANT]` is passed to `cargo build -p ntesim-wasm` during packaging.
- [x] 4.2 Run focused tests, rebuild WASM, and execute `python tools/checkpoint.py run` to verify end-to-end integration and stability.
