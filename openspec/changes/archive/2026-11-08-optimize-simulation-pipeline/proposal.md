# Proposal

## Why

While single-core simulation throughput was substantially improved (~3.8x) by introducing lean batch modes, profiling, reference analysis of gcsim, and Rust/WASM ecosystem best practices reveal substantial untapped throughput opportunities:
1. Studio in the browser runs 1000 trials sequentially in a single Web Worker with synchronous chunk yielding and structured cloning overhead.
2. In the engine's batch trial path, every trial executes and immediately discards a full `damage_analysis()` calculation, validates state timelines, and clones heap strings for each hit during observation accumulation.
3. Dynamic action admission acquires a `Mutex` and clones nested `BTreeMap` keys and commands on every action step.
4. Hit calculation allocates and populates a new `BTreeMap<String, Ratio>` for every hit.
5. Release builds retain panic unwinding landing pads and debug symbols, and omit post-processing with Binaryen (`wasm-opt`) and hardware SIMD128 vectorization.
6. Internal caches use the standard library's SipHash, and WASM memory allocation defaults to `dlmalloc`.

Adopting gcsim's proven architecture (pipelined Web Worker pool, lean batch execution, and zero-allocation stat aggregation) along with safe Rust & WebAssembly toolchain optimizations (`panic = "abort"`, `strip = true`, `wasm-opt -O3`, `simd128`, fast non-cryptographic hashing, and `talc` allocator) will maximize throughput across both native execution and browser Studio while preserving 100% bit-for-bit numeric parity.

## What Changes

- **Toolchain and Build Profile Optimizations**:
  - Configure `panic = "abort"` and `strip = true` in `[profile.release]` to eliminate stack unwinding tables and debug symbols, enabling more aggressive LLVM function inlining.
  - Enable WASM SIMD128 (`target-feature=+simd128`) for browser builds to allow automatic vectorization of numeric calculation loops.
  - Add `wasm-opt -O3` post-processing in the WASM build pipeline to compact local variables, inline WebAssembly-level constructs, and strip redundant bytecode.
  - Configure `talc` as the global allocator for `wasm32-unknown-unknown` to minimize allocation overhead and fragmentation.
- **Fast Hashing and Zero-Allocation Data Structures**:
  - Replace SipHash in internal simulation caches (`ChildPacketCacheKey`, `ActionCacheKey`) with `ahash` or `rustc-hash::FxHashMap`.
  - Optimize `BuffManager::aggregate_modifiers` and `live_buff_stats` to aggregate into a fixed stack array or pre-allocated buffer instead of allocating a fresh `BTreeMap<String, Ratio>` on every hit.
- **Strip Dead-Weight Validation and Clones in Batch Trials**:
  - Remove redundant `report.damage_analysis()?` and intermediate state timeline validation from the batch observation loop in `crates/research/src/trials/studio/observe.rs`.
  - Replace dynamic `String` cloning (`hit.ability_id.clone()`, `hit.damage_effect_id.clone()`) in `row.sources` with borrowed slices or pre-indexed keys.
  - Omit collection of sample-only replay artifacts (`replay_actions`, `locations`, `talents`) during batch trials.
- **Streamline Action Resolution and Admission**:
  - Eliminate `Mutex` locking and deep key cloning in `PreparedSourceScenario::resolve_action` by pre-resolving public actions during scenario preparation or indexing by static descriptor keys.
- **Pipelined Web Worker Pool in Studio**:
  - Replace single-worker execution in `web/src/infrastructure/worker/` with a concurrent worker pool (`clamp(navigator.hardwareConcurrency - 2, 2..8)`) inspired by gcsim's `WasmExecutor`.
  - Introduce request pipelining (`PIPELINE_DEPTH = 2`) so workers never idle during round-trips to the UI thread.
  - Route trial results to an aggregator worker with throttled UI flushes, preventing UI thread serialization contention.
  - Streamline trial sample communication with compact transferable arrays instead of heavy structured cloning.

## Capabilities

### New Capabilities
<!-- None: purely internal throughput and execution pipeline optimizations -->

### Modified Capabilities
<!-- None: all observable requirements, calculation formulas, RNG sequences, and UI report contracts remain bit-for-bit identical. The change declares skip_specs: true. -->

## Impact

- **Build & Packaging (`Cargo.toml`, `tools/` build scripts)**: Release profiles, WASM compilation flags, and `wasm-opt` post-processing.
- **Engine (`crates/content`, `crates/core`, `crates/research`)**: Hot combat hit calculation, buff modifier aggregation, and batch observation paths.
- **WASM Bridge (`crates/wasm`)**: Global allocator selection (`talc`), batch trial execution, and aggregate endpoints supporting concurrent worker sessions.
- **Studio Infrastructure (`web/src/infrastructure/worker`)**: Multi-worker pool, pipelined runner, client controller, and protocol.
- **Performance**: Near-linear scaling across multi-core CPUs in desktop browsers, plus compounding 20%–40% reduction in single-core execution time.
