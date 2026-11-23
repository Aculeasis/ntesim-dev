# Tasks

## 1. Build Freshness Tracking

- [x] 1.1 Update `crates/wasm/build.rs` to emit `cargo:rerun-if-env-changed=NTESIM_ENGINE_SOURCE_SHA256`, `cargo:rerun-if-changed=src`, and `cargo:rerun-if-changed=Cargo.toml`, and verify compilation.
- [x] 1.2 Verify that modifying engine sources triggers `build.rs` rerun and produces an updated `NTESIM_WASM_BUILD_TIME`, while unchanged builds retain the cache.
- [x] 1.3 Run focused checks and `python tools/checkpoint.py run` to verify workspace integrity.
