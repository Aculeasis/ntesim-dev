# Design: Optimize Simulation Pipeline

## Context

ntesim simulates combat scenarios across a shared Rust engine powering the native CLI and React Studio (via WebAssembly and Web Workers). Following initial single-core throughput improvements in `2026-11-07-optimize-simulation-throughput`, comprehensive profiling, reference analysis of gcsim, and Rust/WASM ecosystem research identified several key performance bottlenecks across build profiles, engine memory management, and browser execution:
1. **Toolchain Defaults**: Release profiles omit `panic = "abort"` (retaining stack unwinding landing pads that hinder function inlining), retain debug symbols in WASM, and do not leverage Binaryen's `wasm-opt -O3` or W3C standard WASM SIMD128.
2. **Single-Worker Execution in Studio**: The browser runner executes all 1000 trials sequentially in a single Web Worker in 25-trial chunks with `setTimeout(..., 0)` yields, leaving multi-core CPUs largely idle.
3. **Dead-Weight Validation in Batch Trials**: In `crates/research/src/trials/studio/observe.rs`, every batch trial executes and immediately discards a complete `report.damage_analysis()?` pass (allocating BTreeMaps and calculating overkill) and validates the state timeline, even though these are diagnostic checks needed only for sample trial 0 or failed runs.
4. **Observation Heap Allocations**: Accumulating hit sources in `observe.rs` allocates and clones heap strings (`hit.ability_id.clone()`, `hit.damage_effect_id.clone()`) on every hit (~100,000 string allocations per 1000 trials).
5. **Action Resolution Overhead**: `resolve_action` acquires a `Mutex` and clones nested `BTreeMap` assumptions and `GameActionCommand` keys on every dynamic action (117,000 times per 1000 trials).
6. **Buff Stat Aggregation Allocations**: On every hit calculation, `BuffManager::aggregate_modifiers` allocates and populates a new `BTreeMap<String, Ratio>`.
7. **Hasher and Allocator Overhead**: Internal caches use default SipHash, and WASM memory allocation relies on general-purpose `dlmalloc`.

See [proposal.md](proposal.md) for motivation and goals.

## Goals / Non-Goals

**Goals:**
- Eliminate panic unwinding tables and debug symbols in release builds, enable WASM SIMD128 auto-vectorization, and add `wasm-opt -O3` binary post-processing.
- Adopt `talc` as a high-performance Wasm allocator and `ahash` / `FxHashMap` for internal caches.
- Eliminate redundant diagnostic computations (`damage_analysis` validation, timeline frame checks) and sample-only data collection (`replay_actions`, `locations`, `talents`) from batch runs.
- Eliminate heap string cloning in trial observation accumulation by using borrowed keys or pre-indexed IDs.
- Pre-resolve static action descriptors at scenario compile time to eliminate `Mutex` locking and key cloning during action admission.
- Optimize buff modifier aggregation to eliminate `BTreeMap` allocations in the hot hit calculation path.
- Implement a pipelined multi-worker pool with an aggregator worker in Studio, scaling simulation throughput across available CPU cores.
- Preserve 100% bit-for-bit numeric parity with baseline outputs across all 1000 trial seeds.

**Non-Goals:**
- Changing any combat mechanics, damage formulas, frame timing, or deterministic RNG sequences.
- Altering the user-facing report schemas or visual presentation of Studio.
- Supporting multi-threaded simulation inside a single WASM instance (shared memory / pthread WASM); worker parallelism is achieved via separate Web Worker instances.

## Decisions

### Decision 1: Build Profile, Binaryen, and WASM SIMD Optimizations
- **Choice**:
  - In `Cargo.toml`, set `panic = "abort"` and `strip = true` in `[profile.release]`.
  - In `tools/wasm_bindings.py`, post-process the generated `.wasm` binary using `wasm-opt -O3` (or `-O4`).
  - Pass `target-feature=+simd128` during browser WASM compilation to enable LLVM loop auto-vectorization into 128-bit Wasm instructions.
  - In `crates/wasm`, configure `talc` as the global memory allocator (`#[global_allocator] static ALLOC: talc::wasm::WasmDynamicTalc`).
- **Rationale**:
  - `panic = "abort"` eliminates landing pads and allows LLVM to inline functions that may call `unwrap()` or `assert!`.
  - `wasm-opt` provides WebAssembly-specific optimizations that LLVM cannot perform at IR level (local variable compaction, redundant code elimination).
  - SIMD128 is supported across all modern browsers (Chrome, Firefox, Safari 16.4+) and executes vector math with hardware acceleration.
  - `talc` provides superior throughput and lower fragmentation for WebAssembly workloads compared to `dlmalloc`.

### Decision 2: Fast Hashing and Flat Stack Structures in the Engine
- **Choice**:
  - Replace SipHash with `ahash` or `rustc-hash::FxHashMap` for internal caches (`ChildPacketCacheKey`, `ActionCacheKey`).
  - Provide an optimized query path `aggregate_modifiers_flat` in `BuffManager` that aggregates active stat modifiers into a fixed stack array (or small vector) indexed by known stat identifiers, rather than allocating a `BTreeMap<String, Ratio>`.
  - In `live_buff_stats`, return the pre-aggregated stack buffer or a small slice for hit calculation.
- **Rationale**:
  - Internal caches are trusted and do not require HashDoS protection; fast hashing yields a 2x–3x speedup on cache lookups.
  - A flat stack array matches gcsim's `[attributes.EndStatType]float64` pattern, eliminating all heap allocations during hit stat evaluation.

### Decision 3: Strip Unused Diagnostic Work in Batch Runs
- **Choice**:
  - Differentiate `observe()`: in batch trials, skip `report.damage_analysis()?` and intermediate `report.validate_state_timeline()?`. If a trial fails or halts, rerun it as a sample trial with full diagnostic validation.
  - In `crates/research/src/trials/studio/observe.rs`, replace `BTreeMap<(usize, String, String), (f64, f64)>` with an index-based or borrowed key lookup structure (analogous to gcsim's `Sums`), eliminating `hit.ability_id.clone()` on every hit.
  - In `Trial`: when `record_timeline` is false (batch trial), bypass pushing into `replay_actions`, `locations`, and `talents`.
- **Rationale**:
  - `damage_analysis()` builds nested `BTreeMap` structures to calculate overkill and validate parentage; its return value in `observe.rs` was discarded. Skipping it removes substantial CPU work from every batch iteration.
  - Eliminates ~100,000 heap string allocations per 1000 trials in the observation collection phase.

### Decision 4: Pre-Resolve Static Action Descriptors at Scenario Compile Time
- **Choice**:
  - During `PreparedSourceScenario::new`, inspect authored rotation statements and pre-resolve all unique public action descriptors into a flat `Vec<ResolvedActionDescriptor>`.
  - Store action descriptor indices directly on rotation AST nodes or in a fast direct array.
  - At runtime, `resolve_action` performs an `O(1)` index lookup without acquiring `action_cache: Mutex<...>` or cloning keys.
- **Rationale**:
  - Eliminates 117,000 `Mutex` locks and expensive `BTreeMap` clones per 1000 trials.

### Decision 5: Multi-Worker Pool and Aggregator Architecture in Studio
- **Choice**:
  - Adopt gcsim's `WasmExecutor` pattern in `web/src/infrastructure/worker/`:
    1. **Worker Pool**: Spawn `N` worker instances (`clamp(navigator.hardwareConcurrency - 2, 2..8)`).
    2. **Pipelining (`PIPELINE_DEPTH = 2`)**: Keep up to 2 trial requests queued per worker so round-trip latency never starves execution.
    3. **Aggregator Worker**: Simulation workers stream compact trial samples (or binary buffers) to a dedicated aggregator worker that updates `Collector` state.
    4. **Throttled UI Flushes**: The aggregator worker posts progress snapshots to the main thread at adaptive intervals (100–1000 ms), consuming at most 10% of aggregator CPU.
    5. **Stale Run Cancellation**: Every request and response carries an incrementing `run_id`; outdated messages from replaced or cancelled runs are dropped instantly.
- **Rationale**:
  - Provides near-linear scaling across multi-core CPUs in desktop browsers (Firefox, Chrome, Safari).

## Risks / Trade-offs

- **[Risk]** Omitting `damage_analysis()` in batch trials might hide a regression if a bug only manifested in that validation.
  - **Mitigation**: Dedicated parity tests run both full-validation and lean batch modes on identical seeds, verifying exact match. Trial 0 always runs full validation, and any failed batch trial is re-executed under full validation for error diagnosis.
- **[Risk]** WASM SIMD128 browser compatibility on outdated environments.
  - **Mitigation**: SIMD128 has been universally supported across all major desktop and mobile browser engines since 2022. Node and modern browsers run SIMD128 without polyfills or flags.
- **[Risk]** Multi-worker memory footprint in the browser.
  - **Mitigation**: Cap worker count to 4 on devices with less than 8 GB RAM, and maximum 8 on high-memory systems. Workers share immutable compiled source inputs.
- **[Risk]** Non-deterministic trial completion order across parallel workers.
  - **Mitigation**: Seeds remain strictly deterministic (`seed = base_seed + offset`). The aggregator inserts samples by seed offset into the collector array, ensuring identical statistical rollup regardless of worker arrival order.

## Migration Plan

1. **Toolchain & Profile**: Configure `panic = "abort"`, `strip = true`, `simd128`, `wasm-opt`, and `talc`.
2. **Engine Batch & Structs**: Implement fast hashing, flat buff modifiers, compile-time action resolution, and batch observation cleanup.
3. **Parity Check**: Verify bit-for-bit parity across all 1000 seeds against baseline.
4. **Studio Worker Pool**: Implement the multi-worker pool and aggregator worker in TypeScript.
5. **Full Validation**: Run browser acceptance, native benchmarks, and `checkpoint.py`.
