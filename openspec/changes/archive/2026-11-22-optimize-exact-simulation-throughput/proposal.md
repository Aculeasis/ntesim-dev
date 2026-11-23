# Proposal

## Why

The synchronized Zankou Insight reference took 74.17 seconds median wall in sequential release WASM for 1,000 trials of 120 relative seconds. Reduce event payload copies, repeated metadata preparation, exact arithmetic and receipt processing. Make Damage and DPS consistently count rounded hit damage while preserving combat execution.

## What Changes

- Begin with a compilation/startup review: trace Run-to-first-result latency through Worker/WASM loading, source parsing/validation, build projection, descriptor preparation and per-Worker reconstruction. Compare cold/repeated runs and the corresponding local gcsim stages; review Rust/WASM build time separately so distinct compilation costs are not conflated.
- Add a reproducible, opt-in native/WASM/Worker performance harness and a release-equivalent profiling build with usable symbols. Keep benchmark drivers and instrumentation outside ordinary native and Studio artifacts.
- Repair the observed quality propagation defect in parallel aggregation by transporting report quality with observations and checking consistency. The original repair also transported strict readiness before main removed that field.
- Establish matching one-, two- and four-Worker measurements, including CPU affinity/topology, startup, warmup, execution, aggregation, transport, memory and actual browser completion. Diagnose sublinear scaling before changing pool sizing or scheduling.
- Remove deep queue payload copying during peeks and extraction while preserving clock domains, equal-time causal ordering and exact stopping boundaries.
- Prepare immutable action and packet identities and static damage inputs. Bound the reusable descriptors and metadata lookup tables. Resolve live stat and buff dependencies at each hit.
- Reduce measured startup duplication and unnecessary eager preparation while preserving invalid-source/import diagnostics, artifact validation and cache invalidation. Report time to first progress/sample alongside throughput so extra precomputation cannot hide a slower Run action.
- Change primary Damage/DPS, contributions, time bins, comparisons and exports to sum each hit's already-rounded damage using checked integer accumulation. Keep actual HP loss/overkill separate and exact per-hit formula values in detailed diagnostics; DPS itself may remain fractional. Measure the gain from removing fractional total accumulation separately.
- Accelerate remaining exact rational arithmetic with checked narrow-integer paths, measured GCD alternatives and wide fallback. Preserve formula values, per-hit rounding, HP settlement and resource carry.
- Introduce a compact batch result with streaming observations and online validation, retaining all state needed by content callbacks and the existing detailed sample/replay path.
- Evaluate bounded pre-critical calculation reuse and establish the dependencies that govern schedule reuse. Compare compiler and WASM optimizer variants, including opt-in PGO where the pinned target supports it.
- Target a 2× warm single-threaded speedup on the unchanged primary workload, with 3× as a stretch goal. Measure actual engine and parallel scaling separately.

## Capabilities

### New Capabilities

None. The profiling and benchmark entry points are opt-in development tooling.

### Modified Capabilities

The existing `combat-execution`, `wasm-transport` and `studio-application` consumers adopt rounded hit damage as their common statistics basis. The proposal, design, tasks and existing capability references record the change under the requested `skip_specs: true` workflow. Detailed receipts retain the exact formula values and existing per-hit settlement.

## Impact

- Rust core: exact fractions, buff modifier aggregation and existing scheduler/arithmetic regression boundaries.
- Rust content: clock-indexed event queue, prepared source/action descriptors, metadata catalogs, damage preparation, runtime clocks and batch observation production.
- Rust research/WASM: Studio observation/collector integration, compact batch transport and source-trial sessions. Internal contracts and current consumers change together where necessary.
- Worker infrastructure: measured chunking, aggregation, scheduling and worker-count choices; preserve cancellation, ordered seed commitment, resume verification and detailed sample loading.
- Development tooling/configuration: an isolated benchmark target/driver, profiling profile, artifact-bound receipts under `cache/`, build outputs under `target/`, and relevant development/performance documentation. Ordinary checks do not acquire a 1,000-trial benchmark or refresh Team list DPS.
- Current native, WASM, Worker, report/replay, UI and JSON/CSV consumers migrate to the same rounded-total definition without obsolete-input shims or an extra user-selectable statistics mode. Existing capability references and performance guidance document the delivered definition.
- No new approximate formula mode, truncation of DPS itself, game-fidelity claim, game-code execution, public deployment or compatibility shim. Benchmark controls retain their maintained builds, actions, target, seed sequence and duration.
