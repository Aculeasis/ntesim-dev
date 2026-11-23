# Proposal: Optimize Simulation Throughput

## Why

Simulating long combat scenarios (such as S5 Mint/Sakiri/Zero/Iroi with 120s duration over 1000 trials) currently takes ~115–170 seconds, while comparable simulation tools (such as gcsim) complete 1000 trials significantly faster. Profiling reveals that the current single-threaded simulation loop suffers from major internal overheads:
1. Every combat action in the interpreted rotation re-clones the scenario request and triggers full party validation and JSON serialization inside `admit_descriptor`.
2. Every scheduler event performs per-event state timeline frame observation (`timeline.observe`), cloning buff instances and diffing actor states hundreds of times per trial, even though the timeline is only ever visualized for the initial sample trial.
3. Every trial builds and validates heavy 9 MB report objects that are immediately discarded by the statistical collector after extracting scalar metrics.

Eliminating these redundant operations will dramatically reduce per-trial latency on a single CPU core without altering any deterministic simulation results, RNG sequences, or user-facing analytics.

## What Changes

- **Pre-compile / cache action descriptor preparation**: Avoid cloning the full `GameScenarioRequest`, re-running `PreparedGameScenario::new`, full party validation, and JSON serialization on every single action admission during runtime execution.
- **Support lightweight execution for non-sample trials (Lean/Batch Trial Execution)**:
  - Retain full timeline recording (`timeline.observe`), complete trace collection, and detailed report building for trial 0 (`first_execution`) so the interactive inspector UI retains full fidelity.
  - Disable redundant `timeline.observe` frame snapshots, trace recording, and timeline re-validations for batch trials (trials 1..N) where only aggregate metrics (damage, hits, energy changes, shield absorption) are consumed by `Collector`.
  - Accumulate integral metrics (such as time-weighted shield HP) directly in combat state rather than post-processing hundreds of timeline frames.
- **Tune release code generation**: Enable thin LTO and one codegen unit for release artifacts.
- **Benchmark and audit Web Worker execution in the browser**: Inspect the Web Worker harness and WASM bridge for scheduling stalls, timer clamping (`setTimeout(0)`), or message serialization bottlenecks in browsers (including Firefox).
- **Preserve strict equivalence**: Ensure all damage numbers, DPS distributions, reaction states, and deterministic behavior remain bit-for-bit identical to current baselines for identical seeds.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None (pure performance optimization and internal execution refactoring with identical numerical and causal outputs; `skip_specs: true` declared).

## Impact

- `Cargo.toml`: Release code generation settings.
- `crates/content/src/simulation/`: `runtime/source/actions.rs`, `runtime/execution.rs`, `timeline.rs`, and `runtime/source.rs` to streamline action admission and per-trial observation overhead.
- `crates/research/src/trials/studio/`: `Collector` observation logic to consume integral metrics directly without relying on post-mortem timeline frame diffs for non-sample runs.
- `crates/wasm/src/source_trials.rs`: Ensure trial stepping differentiates sample collection from batch collection.
- `web/src/infrastructure/worker/`: Check worker runner scheduling and message transfer efficiency.
- Current report contracts add the required time-weighted `available_shield_hp` scalar in Rust and TypeScript. Combat and aggregate results remain equivalent; no legacy-input fallback is added.
