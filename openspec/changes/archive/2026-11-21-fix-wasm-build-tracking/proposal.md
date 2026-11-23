# Proposal: Fix WASM Build Freshness Tracking

## Why

In `crates/wasm/build.rs`, the only tracking directive emitted was `cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT`. In Cargo, emitting any `rerun-if` directive stops automatic tracking of crate source files. Consequently, subsequent local `cargo build` or `tools/cargo.py` compilations without explicit `NTESIM_BUILD_STARTED_AT` never re-executed `build.rs`, causing the embedded `WASM_BUILD_STARTED_AT` timestamp to remain permanently stuck on an old cached instant even when engine sources changed and `ntesim_wasm.wasm` was rebuilt on disk.

## What Changes

- Update `crates/wasm/build.rs` to track engine source modifications:
  - Track `NTESIM_ENGINE_SOURCE_SHA256` via `cargo:rerun-if-env-changed=NTESIM_ENGINE_SOURCE_SHA256` (populated by `tools/cargo.py` whenever any workspace engine sources change).
  - Track local crate files via `cargo:rerun-if-changed=src` and `cargo:rerun-if-changed=Cargo.toml`.
  - Retain `cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT` for explicit CI and distribution timestamp overrides.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None (`skip_specs: true` declared; this is an internal build freshness tracking fix that correctly realizes existing `studio-application` requirement: `When loaded, that instant SHALL reflect the compiled WebAssembly engine's embedded build instant extracted by the client`).

## Impact

- `crates/wasm/build.rs`: Add source and engine fingerprint rerun-if directives.
- Local rebuilds now update the embedded WASM build timestamp whenever engine sources are changed.
