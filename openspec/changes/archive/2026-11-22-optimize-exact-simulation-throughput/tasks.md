# Tasks

## 0. Compilation and startup review first

- [x] 0.1 Trace the actual Run-to-first-sample path before throughput work and retain original cold/repeated/edited-source/replaced-Worker startup measurements under the shared CPU cap. Attribute Worker/WASM loading and module compilation, source parsing/validation, build/talent projection, all-action descriptor preparation, serialization, per-Worker `from_export()` reconstruction, collector setup and sample execution; deliver phase timings/preparation counts and available memory rather than attributing time from UI labels.
- [x] 0.2 Compare the local gcsim parser/evaluator and actual batch/Worker setup callers with the observed ntesim path; record reference identity and concrete differences in preparation reuse, eager work and initialization in `design.md`, explicitly avoiding equivalent-work or no-compilation claims that the reference does not establish.
- [x] 0.3 Review development compilation separately using valid existing Cargo/checkpoint receipts and bounded missing measurements: no-op/incremental native, release linking/LTO, WASM build, wasm-bindgen/wasm-opt and web bundling. Deliver distinct timings and bottlenecks without changing shared CPU policy, deleting existing build caches or counting build seconds as Run-time gains.
- [x] 0.4 Reconcile the measured review with source compilation/artifact and Worker lifecycle coverage; record prioritized startup fixes or measured rejection in `design.md` and the existing performance reference, including cold/warm first-progress/first-sample latency. Preserve original baselines and keep actual startup fixes after baseline capture in group 1.

- [x] 0.5 Repair observation quality propagation with consistency checks in the existing collector and forced-parallel Worker coverage; retain an isolated quality-repair reference and prove sample metrics unchanged from the original receipts.

## 1. Isolated profiling, baselines and rounded damage statistics

- [x] 1.1 Add a release-equivalent profiling profile and isolated native/WASM symbol outputs, preserving ordinary glue and optimizer behavior; verify profiles expose useful function names and rebuilding them does not overwrite ordinary artifacts.
- [x] 1.2 Add the explicit native benchmark target/driver over `PreparedSourceScenario` and the Studio collector, and reuse existing WASM/Worker adapters for matching measurements; verify the driver runs from any checkout location through shared CPU wrappers and retains raw results only under `cache/` with builds under `target/`.
- [x] 1.3 Isolate any counters/allocation instrumentation behind a non-default development feature; extend the relevant tooling/build boundary to verify ordinary native, WASM and browser packages exclude instrumentation, profiler dependencies and benchmark drivers.
- [x] 1.4 Before engine edits, retain fresh native and WASM baselines for unchanged Zankou Insight, seeds 42..1041, 120 relative seconds, plus documented controls. Deliver artifact/source-bound raw repetitions, per-seed observations, aggregate results, CPU/affinity, cold/warm boundaries and available memory measurements; do not treat the exploratory figures as saved acceptance receipts.
- [x] 1.5 Capture symbolized CPU/allocation profiles and matched one/two/four-Worker baselines with identical work and timing boundaries; record topology availability, per-worker occupancy, JIT, transport, aggregation and tail costs, and derive initial hotspot-specific Amdahl limits.
- [x] 1.6 Document runnable profiling/benchmark commands, CPU constraints, receipt fields and the revised rounded-damage versus applied-HP-loss definition in existing development/performance and affected capability references; verify the commands reproduce a bounded measurement without adding a long benchmark or DPS refresh to ordinary checkpoint.
- [x] 1.7 Replace the unrounded primary damage accumulator with a checked integer sum of each hit's existing rounded damage, across direct/follow-up/Break/reaction/finishing paths; update total/contribution/bin/DPS/statistics, native/WASM/Worker/UI, export and replay consumers coherently. Verify exact formula diagnostics, per-hit rounding and gameplay callback inputs stay unchanged, wide totals retain precision, and batch mode no longer accumulates an unused unrounded total.
- [x] 1.8 Extend the existing report-analysis/observation and transport boundaries with independent rounded-statistics expectations: two 100.4 hits give 200 Damage, two 100.5 hits give 202, and a 100 hit against 30 HP gives 100 Damage/30 HP loss/70 overkill. Verify all contribution/bin sums, both DPS bases, JSON/CSV/replay and compact/detailed parity; DPS itself is not rounded to an integer.
- [x] 1.9 Derive expected rounded summaries from original receipts for the 1,000 primary seeds, prove unchanged formula values, HP state, RNG, causal order and clocks, and retain a rounded-statistics-only reference before further throughput edits. Report its isolated speed change and use that reference for subsequent exact metric comparisons; the original damage/DPS hashes remain historical.
- [x] 1.10 Apply the startup reductions selected by group 0 after saving original and rounded-statistics references: reuse validated preparation, reduce redundant reconstruction/JSON work, and evaluate required-descriptor or module reuse only where measurements justify it. Extend existing source artifact/invalid-source and Worker lifecycle checks for edited source/limits/options, tampered imports, warm-cache identity, worker replacement/resume and diagnostic parity; report cold/repeated first-sample/completion costs and remove candidates that merely defer work or regress startup memory.

## 2. Queue ownership and clock bookkeeping

- [x] 2.1 Replace deep-copy queue peeks with a lightweight projected event header and move payloads during extraction; verify existing timer-queue, equal-time causality, horizon, replay-context and source scheduler tests, extending the owning boundary only for distinct header/ownership regressions.
- [x] 2.2 Profile repeated next-domain scans, queue synchronization and zero-delta clock advancement; retain measured safe reductions with explicit queue/policy invalidation, or record rejection in `design.md`. Verify paused deadlines, ordinary cinematics, Hotori exceptions and equal-time transitions through existing clock tests.
- [x] 2.3 Compare the queue candidate against rounded-statistics reference primary/control results and profile receipts; deliver exact semantic parity and measured CPU/allocation changes, and update the queue decision in `design.md` with the final implementation and remaining cost.

## 3. Immutable action, metadata and stat preparation

- [x] 3.1 Introduce stable prepared packet/evidence/hypothesis handles and remove repeated metadata object construction on hot hits; verify detailed catalog contents and first-use ordering, queued follow-ups, native replay and existing metadata/report tests match the baseline.
- [x] 3.2 Resolve statically known source actions/parameters to prepared handles and reduce lock/key/result cloning on cached dynamic actions; verify current source action, branch, hold, target, talent, unsupported-dependency and failed-admission checks, preserving native sharing and cache bounds.
- [x] 3.3 Precompute team elements and static actor/packet damage terms; evaluate typed indexed modifier aggregates and revision/deadline-based live caches. Verify exact invalidation on refresh, decay, expiry during pauses, owner/recipient death, active-actor/target changes and stateful owner modifiers using existing buff and owner tests; retain only caches with complete dependencies.
- [x] 3.4 Measure primary/control allocation and CPU changes after preparation/cache edits, compare saved per-seed and sample results exactly, and update `design.md` with cache ownership, bounds and accepted/rejected aggregate representations.

## 4. Exact arithmetic specialization

- [x] 4.1 Measure operand distributions and compare narrow Euclidean/binary GCD in exact fractions and buff aggregation; retain the measured algorithm with checked wide fallback and extend existing exact arithmetic tests for zero/one, noncanonical inputs, wide operands and overflow/error equivalence.
- [x] 4.2 Evaluate checked narrow arithmetic, integer/equal-denominator shortcuts and canonical-aware addition for remaining fractional mechanics; verify independent existing damage/healing oracles, half-up boundaries, large exact JSON values and resource carry. Reject any candidate that changes formula/resource results or errors; do not reintroduce the removed unrounded damage-total accumulator.
- [x] 4.3 Compare native and WASM arithmetic candidates on the primary/control workloads and remove unsuccessful prototypes; document measured gains and retained numeric invariants in `design.md`, preserving per-hit settlement and the rounded-statistics reference.

## 5. Streaming compact batch observations

- [x] 5.1 Inventory concrete consumers of hit history, time ledgers, metadata, trace, source occurrences, RNG audits and final state; define the compact internal outcome/accumulator and online validation responsibilities in `design.md`, retaining every mechanic/callback dependency before removing data.
- [x] 5.2 Accumulate rounded damage contributions/bins as checked integers, plus energy, field time, healing and both shield time bases during execution; verify against the updated full-report `observe()` route and rounded-statistics reference with exact public values, extending the existing compact/report-analysis boundary for distinct risks.
- [x] 5.3 Replace discarded batch report/trace creation with compact outcomes and online parentage/clock/completion checks; verify parent-dependent follow-ups, reached unsupported dependencies, budget failures, incomplete runs and independent detailed failed-seed replay using existing source, receipt and eligibility tests.
- [x] 5.4 Migrate the Rust collector, source-trial WASM endpoints and current Worker consumers together; run existing `source_trials_tests.rs`, `streaming_trials.test.mjs` and `source_calibration.test.mjs` boundaries for metrics, ordered seeds, resume, sample loading and replay parity, and measure retained memory/transport changes.
- [x] 5.5 Update existing performance/development references to describe the delivered compact path and remaining retained state; compare all 1,000 primary observations/aggregates against the rounded-statistics reference and document any rejected data-elision candidate in `design.md`.

## 6. Guarded cross-seed reuse experiments

- [x] 6.1 Prototype bounded pure pre-critical result memoization keyed by complete typed effective inputs; compare cached/uncached exact values and RNG consumption, force changes in stat, mitigation, semantic/hypothesis and owner-state inputs, and record the measured keep/reject outcome in `design.md`.
- [x] 6.2 Evaluate deterministic-schedule reuse for explicitly supported dependency classes; deliver a dependency/guard proof and a bounded prototype, or a concrete rejection. Exercise RNG-driven branches/procs, crit-dependent callbacks, HP thresholds, target defeat and unsupported dependencies in existing source/owner boundaries; guard misses must discard speculative output and reproduce the seed from its original state without duplicate statistics.
- [x] 6.3 Compare ordinary and candidate execution across all 1,000 primary seeds and selected existing adversarial controls, including exact RNG audits, rounded HP losses, both clocks and failure statuses; include preparation/cache/fallback costs and memory, remove rejected prototypes and document supported dependency classes and limitations without preset-name special cases.

## 7. Worker scaling and actual browser completion

- [x] 7.1 Repeat matched one/two/four-Worker measurements after engine changes; attribute CPU inflation, memory growth, occupancy and tail work using the isolated harness and record a supported explanation or explicit measurement limit in the existing performance reference.
- [x] 7.2 Evaluate chunk size, pipelining, aggregator placement and default worker-count choices against measured bottlenecks; retain only justified changes or record rejection in `design.md`. Verify ordered seed commitment, bounded in-flight work, Stop latency, stale-response isolation, fresh-Worker resume and sample loading through existing Worker lifecycle tests.
- [x] 7.3 Measure the unchanged primary workload through the actual production browser/Worker path with startup, aggregation, messaging and completed rendering reported separately; retain exact seed/aggregate parity and verify applicable existing browser lifecycle/memory checks without broadening their ordinary budgets to hide regressions.

## 8. Compiler and optimizer experiments

- [x] 8.1 Compare a bounded set of profile/LTO/WASM optimizer variants selected from measured hotspots; retain artifact-bound uninstrumented native/WASM timings and parity results, documenting accepted settings or rejection in `design.md`.
- [x] 8.2 Evaluate native PGO with explicit train/merge/use artifacts and primary/control workloads; establish WASM support before attempting it there, record unsupported targets, remove unsuccessful variants and verify instrumented or host-specific build flags do not enter portable defaults.
- [x] 8.3 Update the existing development/performance build guidance for any accepted variant and verify documented commands and normal build isolation; report measured gains separately from engine and parallelism improvements.

## 9. Integrated acceptance and stable delivery

- [x] 9.1 Run final matched uninstrumented native/WASM and one/two/four-Worker primary benchmarks with repeated measurements; publish exact parity against the rounded-statistics reference, independently verified score changes from the original policy, isolated/combined CPU and wall speedups, separate cold/repeated Run-to-first-sample and development-build measurements, memory/control results and remaining hotspot-specific limits, stating whether 2x or 3x was actually achieved.
- [x] 9.2 Complete affected existing core/content/research tests, native/WASM/Worker replay/transport checks and applicable production browser acceptance through project wrappers; reuse valid receipts and rerun affected checks after fixes without adding overlapping mechanics suites.
- [x] 9.3 Reconcile `proposal.md`, `design.md` and tasks with the retained implementations and bounded experiment outcomes; verify strict OpenSpec validation, review the stable diff for unrelated edits, obsolete documentation, portability and accidental profiler code in ordinary artifacts.
- [x] 9.4 Run `python tools/checkpoint.py run` on the stable final implementation, repair affected failures and rerun as required; retain a passing receipt before any implementation commit or completed-change archive.

## Workflow follow-up

- Archive this change after all implementation tasks and applicable checks are complete; keep the user's requested no-separate-spec scope explicit and reconcile existing capability references with rounded primary damage statistics.
- The rounded damage-total policy is authorized in this change. Changes to per-hit rounding, formula approximation or game-fidelity assumptions remain outside its scope.
- Publication/deployment and Team list DPS refresh remain separate explicit actions.
