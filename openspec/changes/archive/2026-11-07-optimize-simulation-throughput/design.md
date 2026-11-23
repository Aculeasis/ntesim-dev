# Design: Optimize Simulation Throughput

## Context

ntesim simulates combat through a shared Rust engine across native CLI and React Studio (via WebAssembly and Web Worker). Currently, a 120-second combat scenario over 1000 trials takes ~115 seconds in Firefox and ~170 seconds in native single-threaded execution.

Profiling and source analysis show two primary hotspots:
1. **Repeated scenario preparation during action admission**: In `crates/content/src/simulation/runtime/source/actions.rs`, every dynamic action in the rotation executes `admit_descriptor()`, which clones the full scenario request, instantiates a new `PreparedGameScenario`, runs full scenario validation twice, projects talents, and serializes talent receipts to JSON string via `serde_json::value::to_raw_value`. In a 120-second scenario with 117 actions, this causes ~117,000 full scenario validations and JSON encodings per 1000 trials.
2. **Exhaustive state timeline recording on every event**: In `crates/content/src/simulation/runtime/execution.rs`, every scheduler event (697 frames per trial) invokes `timeline.observe()`, which snapshots 4 actors, target summaries, clones the entire active buff list, and computes a diff frame. Over 1000 trials, ~700,000 frames are created, validated with `validate_state_timeline()`, and discarded by `Collector` after reading only scalar metrics.

The batch path retains detailed observations only for the initial sample, while later trials accumulate the metrics consumed by the statistical collector.

## Goals / Non-Goals

**Goals:**
- Eliminate redundant request cloning, party re-validation, and JSON serialization from runtime action admission.
- Introduce a lightweight execution path for batch trials (trials 1..N) that bypasses per-event `timeline.observe()` and redundant trace collection while preserving sample fidelity on trial 0.
- Maintain 100% bit-for-bit equivalence in simulation results: identical damage, hits, energy ledger, DPS distribution, and RNG sequences for identical seeds.
- Validate performance gains on a single CPU core without external workload distortion.
- Audit the browser Web Worker runner for JS event loop / WASM bridge overheads.

**Non-Goals:**
- Multi-core thread pooling / Web Worker farming (explicitly deferred to isolate engine throughput gains on a single core first).
- Any alteration to combat rules, frame timing, damage formulas, or deterministic causality.
- Modifying the visual presentation or contracts of the Studio UI.

## Decisions

### Decision 1: Pre-resolve and cache public action descriptors

- **Choice**: Pre-resolve action packet metadata and command parameters during scenario preparation (or memoize prepared descriptors keyed by `(actor_index, action, params)`).
- **Rationale**: The party composition, talent levels, weapons, cartridges, and public action descriptors are static across trial steps. Validating the entire party and projecting talents repeatedly inside `admit_descriptor` at runtime is unnecessary; resolving them once allows `admit_descriptor` to emit the pre-resolved command and packet directly. Cache keys include the state-resolved command and every packet-affecting parameter, with timestamps normalized before lookup and restored at admission. Cache size is bounded by the source action limit; additional distinct keys resolve without retention.
- **Alternatives considered**:
  - *Optimize `PreparedGameScenario::new` internals without caching*: Rejected because invoking full party validation and JSON serialization 117,000 times per run remains inherently wasteful regardless of micro-optimizations.

### Decision 2: Distinct Sample Run vs Batch Run Modes (following gcsim's `opt.Debug` model)

- **Choice**: Differentiate between:
  1. **Sample trial (trial 0)**: Full `timeline.observe()`, complete `trace` collection, comprehensive receipts, and full timeline validation for the UI inspector.
  2. **Batch trials (trials 1..N)**: Skip actor/buff snapshot creation and diffing at intermediate events. Retain full initial/final frames and compact receipt boundaries, preserving existing timeline and damage reconciliation. Suppress per-hit descriptive trace allocation; retain energy and switch events needed by `Collector`. Other callbacks may still allocate transient trace descriptions, which are discarded after the event. Canonical damage, healing, energy and causal receipts remain available to validation. A failed batch trial is rerun with full observation for inspection.
- **Handling shield HP (`shp`)**:
  - Currently, `calculate_available_shield_hp` in `Collector` reconstructs shield coverage by iterating over 697 timeline frames post-trial.
  - Instead, accumulate time-weighted active shield HP in `PreparedExecution` using checked `u128` arithmetic during clock advancement, clipping each living active actor shield to its expiry. Expose the required `available_shield_hp` report scalar to Rust and TypeScript; zero elapsed time produces zero. Both sample and batch runs use this scalar. The original timeline reconstruction remains a test oracle, with exact equality checks.
- **Alternatives considered**:
  - *Keep full timeline recording for all trials but optimize frame diffing*: Rejected because allocating and diffing 700,000 frames when 99.9% are immediately dropped is unnecessary work.

### Decision 3: Audit Browser Web Worker Runner and Message Transfer

- **Choice**: Inspect `web/src/infrastructure/worker/runner.ts` and `client.ts`:
  - Check chunking interval (`count = Math.min(25, ...)`), `postMessage` serialization overhead, and `await new Promise(r => setTimeout(r, 0))` scheduling.
  - Ensure the browser runner yields cooperatively without introducing timer-clamping latency spikes (particularly in Firefox).

### Decision 4: Single-Core Benchmarking Protocol

- **Choice**: All verification and before/after comparisons must measure wall-clock and CPU time on **1 logical CPU core** (using an affinity-pinned native streaming harness and Node/V8 source-trial runner).
- **Rationale**: Isolates code optimizations from background CPU utilization and multi-threading artifacts.

## Risks / Trade-offs

- **[Trade-off]** Thin LTO and one codegen unit increase release build time. Runtime measurements exclude compilation; development profiles remain unchanged.
- **[Risk]** Disabling `timeline.observe()` in batch runs might inadvertently skip state mutation if any combat logic depended on timeline side effects.
  - **Mitigation**: `timeline.observe()` takes an immutable `&GameSimulatorState` reference; code audit confirms zero combat state mutations. In addition, unit tests will verify that batch runs produce identical damage and metric arrays as full-observation runs for identical seeds.
- **[Risk]** Integral metric accumulation (shield HP, field time) might diverge from post-mortem frame iteration if edge cases (e.g. actor defeat, mid-frame expiration) differ.
  - **Mitigation**: Add a dedicated parity test comparing the state-accumulated shield HP against the frame-reconstructed shield HP across all character kits.

## Migration Plan

Current Rust and TypeScript report contracts gain the required `available_shield_hp` scalar together. No legacy report fallback is retained. Sample reports retain their full observation and replay data. Native source research batches that retain every execution continue to use full reports; the lean API is used by streaming WASM trials and is callable by native consumers. Release builds use thin LTO and one codegen unit; development settings remain unchanged.

## Worker audit

`web/src/infrastructure/worker/runner.ts` keeps the first trial separate, then
steps at most 25 trials per synchronous WASM call. A 1000-trial run makes 41
steps and cooperative timer yields. Timing uses monotonic `performance.now()`;
there is no per-trial JS timer. Progress snapshots are delivered after the first
trial, on completion/failure, or at least 250 ms after the previous delivery.
The main thread receives only new sample rows at a checked offset. The first
execution and compiled source are still cloned into each delivered snapshot.
Each step serializes its aggregate; emitted updates also request a fresh snapshot
with the current elapsed time. These are remaining costs, not removed work.

The existing chunk size and yield preserve cancellation by Worker replacement.
No scheduling or transport change is needed for this engine optimization.
Node Worker integration covers deltas, stop/retry, replay and admission failures;
this audit does not measure Firefox scheduling or browser rendering latency.

## Single-core measurement (2026-11-07)

The unchanged `presets/studio/s5.nte` runs 1000 seeds (42..1041), each with a
120-second horizon. Baseline source is commit `9612a1b7`; the optimized release
includes the descriptor cache, compact batch observation and thin LTO with one
codegen unit. Windows x64 affinity was fixed to one logical CPU for each measured
process and its children. Rust is 1.98.1 and Node/V8 is v24.19.0. Builds and other
validators did not overlap measurements. These are single runs on this host,
not portable performance budgets.

| Engine | Before wall time | After wall time | Speedup | Before / after CPU time |
| --- | ---: | ---: | ---: | ---: |
| Native streaming collector | 111.944 s | 29.330 s | 3.82x | 105.109 / 28.844 s |
| WASM source-trial session in Node/V8 | 104.325 s | 33.022 s | 3.16x | 99.625 / 32.390 s |

Wall timing excludes engine loading and initial source compilation and includes
sample trial 0. WASM timing includes session start and 41 trial steps; it excludes
Worker/UI messaging and timer yields. Native wall timing covers the execution and
collection loop; native process CPU time also includes initialization and output.
WASM CPU time covers the measured session and comes from `process.cpuUsage()`.

All 1000 per-seed metric rows and every aggregate field match exactly across
native-before, native-after, WASM-before and WASM-after. No numeric tolerance or
normalization of metric values was used. All aggregates use elapsed_ms=0 for
comparison, keeping host timing outside simulator statistics. Full/compact Rust
report tests additionally compare every report field except timeline and trace,
including exact damage, final state, RNG audits and causal receipts. The supported
shield owner is Adler; continuing and incoming-damage examples check its integral
against independent timeline reconstruction with exact equality.

Raw reports, timing receipts and the bounded comparison result are in ignored
`cache/throughput/`. The before/after harness uses the same shared Rust
`PreparedSourceScenario` and Studio `Collector`; the native measurement does not
claim to optimize research commands that intentionally retain every full report.
