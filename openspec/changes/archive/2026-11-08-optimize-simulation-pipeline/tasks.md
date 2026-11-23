# Tasks

## 1. Toolchain, Compiler Flags, and WASM Runtime

- [x] 1.1 In `Cargo.toml`, configure `panic = "abort"` and `strip = true` for `[profile.release]`, and verify release build compilation with `python tools/cargo.py build --release`.
- [x] 1.2 Verify WASM memory configuration and allocator compatibility for `wasm32-unknown-unknown` with SIMD128 and panic=abort.
- [x] 1.3 In `tools/wasm_bindings.py` and `.cargo/config.toml`, integrate `target-feature=+simd128` and `wasm-opt -O3` post-processing into the WASM build pipeline, verifying generated `.wasm` passes browser smoke tests.

## 2. Engine Data Structures, Fast Hashing, and Batch Path Cleanup

- [x] 2.1 Replace standard SipHash with `ahash` or `rustc-hash::FxHashMap` in `crates/content/src/runtime_api/context.rs` and `crates/content/src/simulation/runtime/source.rs`, and verify with `python tools/cargo.py test -p ntesim-content`.
- [x] 2.2 In `crates/research/src/trials/studio/observe.rs`, bypass `report.damage_analysis()?` and intermediate state timeline validation on batch trials (retaining them on sample trial 0 and diagnostic halts), and verify with `python tools/cargo.py test -p ntesim_research`.
- [x] 2.3 In `crates/research/src/trials/studio/observe.rs`, eliminate heap `String` cloning in `row.sources` and energy accumulation using borrowed slices or pre-indexed keys, and verify with existing parity tests in `crates/research/src/trials/studio/tests.rs`.
- [x] 2.4 In `crates/content/src/simulation/runtime/source.rs`, bypass accumulation of sample-only replay artifacts (`replay_actions`, `locations`, `talents`) during batch trials, and verify that sample trial 0 retains full replay artifacts while batch trials match exact metric outputs.

## 3. Action Pre-Resolution and Stat Modifier Optimization

- [x] 3.1 In `crates/content/src/simulation/runtime/source.rs` and `actions.rs`, pre-resolve unique public action descriptors during scenario preparation, replacing runtime `Mutex<HashMap<ActionCacheKey, ...>>` with direct index lookups, and verify with `python tools/cargo.py test -p ntesim_content`.
- [x] 3.2 In `crates/core/src/buffs/dispatch/modifiers.rs` and `crates/content/src/simulation/hit/calculation.rs`, optimize `BuffManager::aggregate_modifiers` and `live_buff_stats` to eliminate dynamic `BTreeMap<String, Ratio>` allocations in the hit calculation path, and verify with `python tools/cargo.py test -p ntesim_core -p ntesim_content`.

## 4. Web Worker Pool and Pipelined Studio Runner

- [x] 4.1 In `web/src/infrastructure/worker/`, implement a multi-worker pool (`clamp(navigator.hardwareConcurrency - 2, 2..8)`) with request pipelining (`PIPELINE_DEPTH = 2`) and stale run cancellation, replacing the single-worker runner in `runner.ts` / `controller.ts`.
- [x] 4.2 In `web/src/infrastructure/worker/`, implement an aggregator worker routing and throttled UI flush mechanism (100–1000 ms) with transferable array / compact sample IPC.
- [x] 4.3 Verify Web Worker pool behavior, progress updates, cancellation, and error handling with `node tools/web.mjs test` and Node worker smoke tests.

## 5. End-to-End Verification and Benchmarking

- [x] 5.1 Run single-core native and WASM comparison benchmarks against baseline to verify performance improvements and exact numeric parity across all 1000 seeds.
- [x] 5.2 Run browser Studio smoke check and `python tools/checkpoint.py run` to verify clean delivery.

## Workflow follow-up

- Review stable diff and archive the change after checks pass.
