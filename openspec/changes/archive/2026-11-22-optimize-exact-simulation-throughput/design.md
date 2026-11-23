# Design

## Context

See [proposal.md](proposal.md) for motivation and scope. The existing throughput changes already introduced compact timelines, prepared source scenarios, fast hashing, an optional WASM optimizer and parallel Workers. This change must measure the current implementation rather than count those improvements again.

Relevant implementation boundaries:

- `crates/content/src/simulation/timer_queue.rs`: `peek()` projects a cloned complete payload; `pop()` also clones the extracted payload, and execution repeatedly peeks before delivery.
- `crates/content/src/simulation/runtime/source.rs`: prepared action lookup still builds a cloned command key, fingerprints assumptions, acquires `RwLock` and clones a descriptor containing metadata and talent receipts.
- `crates/content/src/simulation/state/simulator.rs`: packet interning constructs normalized metadata and clones provenance fields before searching the catalog.
- `crates/content/src/simulation/hit/calculation.rs` and `gear/source_stats.rs`: packet evaluation repeatedly derives static bonuses, team elements, live buff aggregates and target counts.
- `crates/core/src/combat/exact/fraction.rs`: checked `u128` fractions and Euclidean GCD; `buffs/dispatch/modifiers.rs` has another wide GCD aggregation path.
- `crates/content/src/simulation/runtime/execution.rs`, `runtime/time.rs` and `crates/research/src/trials/studio/observe.rs`: compact runs retain hits, ledgers and transient trace data before observation and validation.
- `crates/wasm/src/source_trials.rs` and `web/src/infrastructure/worker/{parallel,pool,runner}.ts`: preparation, chunk execution, aggregation and ordered commitment. Parallel chunks currently contain up to 50 trials; sequential session steps contain up to 25.
- `tools/native_benchmark.py` already measures native lifecycle boundaries; `tools/wasm_bindings.py` currently writes release glue to a shared output directory. Profiling outputs must not replace ordinary glue.
- `dsl/source.rs::compile_response()` prepares a complete source scenario before exporting it; `PreparedSourceScenario::from_export()` calls `Self::new()` again and compares reconstructed request JSON. Each Worker owns its own prepared-source cache. Initial preparation also resolves every actor's public actions, including actions not necessarily reached by the authored rotation.

### Exploratory observations, not acceptance receipts

The preceding read-only exploration used Node 24.19.0 and matching current source/release WASM. No durable profiling receipt was saved; implementation must reproduce the baseline before engine changes.

| Observation | Result | Boundary / limitation |
| --- | --- | --- |
| Zankou Insight, seeds 42..1041, 120 relative seconds | 78.008 s wall; 77.423 s process CPU | Warm sequential source session, includes Collector and wire generation; excludes initial loading/compilation and UI |
| Same seeds, four WASM Workers | 29.439 s wall; 116.125 s process CPU | Warm chunk execution and delivery; excludes initialization, native aggregate merge and UI; shares four logical CPUs |
| Sequential JS wire decode | 86.709 ms; total wire bytes 8,226,180 | Internal Rust report/value/wire creation remains inside execution timing |
| Event and hit counts | 3,073,000 events; 812,000 hits | All 1,000 sample metric rows match between the two runners |
| Seed 42 | 120,000,000 relative us; 201,347,194 absolute us | Ultimate pauses materially extend the absolute run |
| Seeds 42, 43, 44, 1041 | Identical 812 hit identities, timestamps, pre-critical damage pairs and critical rates; different rolled totals | Evidence for a reuse experiment, not a dependency proof |
| Seed 42 hit fractions | All 2,436 non-critical/critical/chosen damage values fit `u64` pairs; at most 43 numerator bits / 28 denominator bits | Intermediate arithmetic and other runs may exceed `u64` |
| Seed 42 metadata | 61 normalized packet templates; 157 unique pre-critical damage/rate tuples | Counts do not establish the CPU share of interning or arithmetic |

Baseline engine source SHA-256: `570bc6324d4d886320fc6ccc172b5ffc808a92fd99853481665fbaabc93f3303`.
WASM SHA-256: `254bbfe51b5256642fa753d2cd894fed93474dfa3d619a52e41a9226ab5e5184`.
Primary source SHA-256: `a57ad38de2164b43028bca212d9bdaee4e665a6e1299ad1a5562c0d865fdaed4`.
Sample metric rows SHA-256: `b6aebf55b3857effa245839f1dce1b0fa69dc9014ceda44509aabc30f1634e44`.

### Current numeric behavior and authorized statistics revision

Actor and target HP are integers. Direct packet calculations retain exact rational coefficients, modifiers, non-critical damage and critical damage. `apply_damage()` rounds the chosen exact damage to nearest integer, positive halves upward, then settles integer HP loss, capped by available HP. It separately adds the unrounded fraction to `total_damage_exact`; DPS uses that damage total, not the sum of rounded HP losses. Overkill and applied HP loss remain distinct.

For two ordinary hits of exactly 100.4 with sufficient target HP, current HP loss is 200 and the old damage total is 200.8. The user authorized changing primary Damage to 200 in this same plan, without a separate spec. Preserve exact intermediate formulas and existing per-hit rounding; change the statistical accumulation basis to the resulting integer damage. Fractions also represent resource carry, gauges, stat transfers and timing phase/rates, which stay unchanged. Build HP/DEF quantization remains a separate bounded model. This choice is a product statistics policy, not a claim about the game's internal numeric representation.

Apply this bounded policy change before subsequent throughput refactors so later exact parity uses a reference with the same statistics basis. The exploratory hashes and metrics above describe the old fractional-total engine; old damage/DPS rows are not the new statistical oracle.

## Goals / Non-Goals

**Goals:**

- Reduce CPU work per seed while retaining exact formula values, per-hit rounding, RNG consumption, live admission, causal ordering, failures and native/WASM/Worker agreement.
- Make every primary damage summary use checked sums of rounded hit damage; preserve actual HP loss and raw per-hit formula diagnostics separately.
- Measure warm engine throughput separately from cold startup and actual browser completion; establish a valid scaling curve before attributing the four-Worker result.
- Review compilation/preparation first, measuring Run-to-first-result latency and separating source compilation, browser WASM compilation and development builds.
- Make profiling repeatable and opt-in, with no profiler dependency, counters or benchmark driver in ordinary runtime artifacts.
- Address every proposed optimization through an implementation or a bounded, recorded keep/reject experiment. Use measured hotspots to order work and remove unsuccessful experimental code.

**Non-Goals:**

- New approximate-formula or expected-crit modes, rounding DPS itself, or changed content, seed counts, target HP, relative horizon or pause rules to improve a score.
- A universal speed guarantee or a promise of ideal scaling from logical CPU count.
- A mandatory long benchmark in ordinary checkpoint, new `tools/verify_*.py` scripts, public publishing or automatic Team list DPS refresh.

## Decisions

### 0. Review compilation and Run startup before throughput work

Treat the visible `Compiling source scenario`/Run preparation stage as the primary startup concern. Trace time from Run through Worker creation, WASM download/cache lookup, browser module compilation/instantiation, source lex/parse/validation, configuration/build/talent projection, action descriptor resolution, export/value/wire encoding, `from_export()` reconstruction in each Worker, collector setup and first detailed sample. Attribute both wall time and CPU where available; the UI stage label alone does not identify the expensive component.

Measure a cold page/new pool, a repeated unchanged Run, an edited-source Run and a replaced-Worker/resume path. Compare one/two/four Workers with matched CPU allowance and identify whether startup stages serialize or repeat concurrently. Record dependency/cache hits and preparation counts, including how much eager descriptor work is actually used. Preserve an original startup receipt before any fix; do not infer the source compiler is slow from a warm 78-second execution measurement that excluded it.

Use the local gcsim reference as an architectural comparison: its `pkg/gcs/parser` builds an AST with constant folding and `pkg/gcs/eval.NewEvaluator` binds that AST to a simulation core; it also performs character/target setup. Inspect its current batch/Worker caller before concluding how often parsing/preparation is reused. Record the reference revision and relevant source owners when available. An absent visible compile stage is not evidence that no preparation occurs, and different games/scripts are not equivalent timing workloads. Compare reuse, eager/lazy work and setup placement rather than claim a simulator speed ratio from incompatible inputs.

Deliver findings and choose bounded startup fixes from measured duplication: reuse an already validated prepared program in its owning Worker, reduce reconstruction on the trusted internal path, avoid unnecessary full JSON roundtrips/cloning, prepare only required descriptors when diagnostic semantics allow, and evaluate shared compiled `WebAssembly.Module` versus repeated URL loading where supported. Cross-Worker state remains independent. Cache identities cover exact source, compile/execution limits, options and engine/content identity; imported or edited artifacts retain full integrity validation. A matching source string or a warm cache must not bypass changed request/limit validation. Invalid sources retain diagnostic locations/codes, and preparing a program consumes no combat RNG or future live state.

Review development compilation separately: existing Cargo/checkpoint timing receipts, no-op/incremental native builds, full native release, WASM codegen/link, wasm-bindgen, wasm-opt and web bundling. Separate dependency acquisition from compilation and avoid destructive cache cleaning to manufacture a cold build. Reuse existing receipts when valid; collect missing build timings explicitly under the shared CPU limit. Initial review may recommend build changes, but portable/default build settings change only through the measured compiler/optimizer experiment tasks. Development build seconds do not count as Run-time or trial speedups.

Keep the review first in the task order, with findings recorded in this design and existing performance references. Apply startup fixes only after the relevant original and rounded-statistics baselines are saved. Validate startup fixes through the existing source artifact, invalid-source, Worker lifecycle and replay boundaries; report cold/warm time to first progress, first sample and completion, plus preparation memory. If a candidate merely moves work to first use, include that cost in the first-sample/end-to-end result.

### 1. Establish matched baselines and symbolized profiles first

Provide a native benchmark target using the actual `PreparedSourceScenario` and Studio collector, plus WASM/session and actual Worker/browser measurements reusing existing runtime adapters. Keep drivers outside normal CLI/WASM entry points. Add a profile inheriting release optimization, LTO and codegen settings with symbols; preserve names through profiling-only WASM glue/optimizer output under a separate `target/` location. Any detailed allocation/call counters use a non-default build feature confined to development targets. Compare uninstrumented builds for headline speedups.

Receipts under `cache/` bind code/source/artifact hashes, compiler and runtime versions, optimizer settings, numeric mode, exact workload, seed range, successes/failures, processed work, selected CPU affinity, warmup and timed boundaries. Record raw repetitions, median and spread, process-tree CPU, available memory measurements and message/response sizes. Benchmark the primary workload repeatedly, alternating baseline/candidate where possible; keep first sample, compilation and warmup costs explicit. Use Mint/S5 as a simple control and the existing Hotori, shield/incoming-damage and failure fixtures as targeted semantic controls, without changing maintained presets.

Use at most four logical CPUs and one validator at a time. Distinguish a single compute thread from the process CPU allowance for JIT/background tasks. For scaling, keep artifact, warmup, execution API, aggregate work, CPU allowance and metric boundaries identical across one, two and four Workers. Record logical-to-physical topology when available; report unavailable topology explicitly. Measure default browser pool behavior separately if its worker count differs from the experimental pool.

The exploratory 2.65x wall improvement and approximately 1.50x CPU inflation have different boundaries and do not identify a cause. Profile worker occupancy, JIT, allocation, memory growth, structured cloning, aggregator work and idle/tail time. Compare physical-core placements only within the shared CPU cap. Change chunk size, pipelining, aggregation placement or default concurrency only when matched measurements show a gain without lifecycle regressions.

Alternative rejected: infer that four Workers should be 4x faster or optimize JS decoding first from these exploratory results.

### 1a. Sum rounded hit damage for primary statistics

Use each settled outgoing hit's existing `rounded_damage` as the primary damage unit, including direct, follow-up, Break, reaction and finishing packets. Compute it once using the unchanged half-up policy and accumulate it with checked integers wide enough for current exact export limits, normally `u128`. Do not add raw fractions to the primary total. If a retained summary wire field uses a rational representation, encode its integer total as `N/1` at the boundary rather than maintain a fractional accumulator. Preserve exact wide integer export without a `f64` roundtrip.

Team/actor/source/target totals, timeline bins, both DPS bases, per-seed observations, aggregates, comparisons and JSON/CSV/UI summaries all use this definition. DPS remains damage divided by the existing elapsed duration and may be fractional. Update current contracts and consumers together, removing obsolete duplicate aggregate fields where they express the former basis. Keep raw exact `damage`, non-critical and critical formula values in detailed per-hit diagnostics; they are not a second primary DPS basis or a required unrounded running total in batch mode.

Continue to report applied HP loss and overkill independently: a 100-damage hit against 30 remaining HP contributes 100 Damage, 30 HP loss and 70 overkill. For two 100.4 hits against sufficient HP, rounded Damage is 200; two exact 100.5 hits produce 202. Round each hit before addition, never the sum. Formula evaluation, damage-driven gameplay callbacks, HP settlement, RNG and action/clock behavior retain their current inputs; verify each total consumer before changing it so a statistics edit cannot silently change mechanics.

Extend the existing report-analysis and observation tests with these independent expectations, contribution/bin reconciliation, both time bases and wide totals. Preserve exact native/WASM/Worker agreement under the new policy and verify replay/export consistency. Document the new basis in existing capability references and current result descriptions without adding a separate spec or statistics toggle.

Keep artifact-bound measurement points for the original engine, the quality repair, the rounded-statistics-only reference, and subsequent optimized builds. Original parallel aggregates incorrectly retain the collector's initial incomplete quality despite matching samples. The user authorized carrying actual report quality and strict readiness with observations and validating consistency; record that isolated correction rather than silently excluding quality from comparisons. Derive expected rounded totals/contributions/bins independently from original hit receipts; compare unchanged formulas, rounded losses, state, RNG and clocks against the original engine. Then compare all new statistics exactly against the rounded-statistics-only reference. Measure its isolated cost change and combined speedup; retain both comparisons so intentional score changes do not hide execution regressions.

### 2. Inspect queue headers without cloning payloads

Introduce an internal lightweight next-event view containing projected absolute time, causal priority, sequence and the event classification needed for stopping decisions. Peek and next-domain selection operate on that view; extraction moves the owned event and projects its payload once. Reuse a selected domain only while the queue/policy revision is unchanged. Preserve replay context and all domain pause/exception rules.

Profile repeated domain scans, `sync()` calls and zero-delta clock advancement after removing copying. Coalesce only redundant bookkeeping with explicit policy/state invalidation. A zero delta is not permission to skip an equal-time lifecycle transition. Consider compact domain indexing or a cached next header if scans remain hot; reject replacement schedulers that introduce polling or weaken ordering.

Alternative rejected: replace microsecond timestamps with frames or batch DoT ticks across intervening state changes.

### 3. Prepare immutable identities and static inputs

Assign stable internal IDs to prepared actions, packet templates, evidence and hypothesis descriptions. Represent queued immutable data by IDs into the owning prepared catalog rather than repeatedly cloned boxes/strings. Preserve public catalog contents and first-use ordering; dynamic metadata still obtains a complete validated identity. Catalog ownership spans all queued work, failed-seed replay and inspection without unsafe references.

Resolve static action parameters to prepared handles. Keep a bounded cache for genuinely dynamic parameter combinations, keyed by every affecting parameter and state-resolved command; avoid cloning full talent/provenance payloads on cache hits. Preserve native sharing/thread-safety where the existing prepared scenario supports it.

Precompute team element composition and static actor/packet bonus terms. Profile typed stat indexing and small fixed aggregates inspired by gcsim's indexed stat arrays, retaining exact values. Cache live aggregates only with complete invalidation for buff applications/removals/refresh/decay, expiry, source/recipient liveness, active actor, target changes and owner-specific state. Use relevant world/object/action deadlines, not an absolute-time bucket. Cache keys must contain actual values/dependencies across seeds; matching revision numbers alone are not cross-seed equality.

Alternative rejected: pointer identity, incomplete hash-only keys or preset-specific hardcoded coefficients.

### 4. Keep exact fractions; specialize their common cases

Retain the existing public fraction/transport representation and error boundaries. Compare narrow Euclidean and binary GCD on measured operand distributions, including the buff aggregator. Use `u64` computation when operands and checked intermediates fit; otherwise use the exact wide implementation. Add integer/identity/equal-denominator shortcuts only when canonicalization and overflow/error behavior remain equivalent. Test before deleting any apparently redundant normalization.

Measure canonical-denominator-aware addition for the remaining genuinely fractional mechanics as a separate candidate; remove the old raw-damage total accumulator rather than optimize it. Any common-denominator accumulation still needed by resources or other exact mechanics needs checked representability and an exact fallback. Preserve their order and error semantics. Do not globally replace `u128` fields, silently wrap intermediates or change final half-up rounding. Eliminating an overflow that existed only in the removed unrounded statistical accumulator is part of the authorized policy change; formula and resource arithmetic failures remain unchanged.

Alternative rejected: truncate decimal formula intermediates or convert formulas globally to `f64`. Primary damage uses the already-rounded settlement amount, which removes fractional accumulation without approximating the formula or resource mechanics.

### 5. Accumulate batch observations during settlement

Add an internal compact execution outcome consumed directly by the Rust Studio observation/collector boundary. Successful batch runs accumulate actor/source/target contributions, time bins, hit counts, energy grant/spend/overflow, field time, healing and both time-basis shield integrals without constructing discarded full reports.

Classify retained data by concrete consumers before removing it: parent hit identities, callback inputs, energy groups, reached unsupported dependencies, RNG state, source occurrences used for diagnostics, and continuing owner state remain available. Move receipt/clock/parentage checks into online validation with equivalent failure status. Keep only the callback-required history; preserve detailed sample/export/replay behavior. A failed compact run is independently rerun from the initial seed through the detailed path, before a successful observation can be committed.

Preserve checked integer damage totals and the rounded-statistics reference's public rate/distribution values. Accumulate damage contributions and bins as integers before presentation conversion; retain deterministic order for seed commitment and the remaining floating statistics. Adapt the updated `observe()` full-report route as a differential oracle rather than maintaining a separate domain implementation in TypeScript. Current internal WASM consumers migrate together; detailed per-hit formula and causal data remain unchanged.

Alternative rejected: merely remove post-run validators or discard receipts still read by owner callbacks.

### 6. Evaluate guarded reuse across seeds

First evaluate bounded memoization of pure pre-critical calculations keyed by the complete typed effective inputs. This can reuse exact damage pairs while still executing every seed's live scheduler, RNG and HP settlement. Keys include packet semantics, stat/coefficient/modifier zones, target mitigation and reached hypothesis identity; do not retain mutable state references.

Then evaluate a prepared deterministic schedule for supported dependency classes. Admission requires an explicit dependency proof that random outcomes cannot change control flow, packet production, costs, recipient/cap selection, timers or reached support. Preserve actual RNG order and rejection sampling, exact rounded HP loss and every seed's initial state. Include guards for target/actor defeat, health thresholds, random procs, crit-dependent callbacks and live observations. Unsupported dependency classes stay on ordinary execution; eligibility derives from modeled dependencies, not the preset name.

A guard miss discards speculative output and reruns that seed from its original input/seed through ordinary execution. Do not publish a speculative prefix, read future RNG to choose a path or reuse a final mutable state. Four matching sample schedules are not a certificate. Bound preparation and cache memory and include their costs in cold/end-to-end measurements.

Deliver a measured keep/reject decision for both experiments in this design. Ship reuse only after dependency/guard checks and exact comparisons pass; otherwise retain the ordinary path and remove the prototype. This outcome does not weaken the target or authorize a numerical approximation.

### 7. Evaluate build optimization separately

Compare relevant optimization level, LTO and WASM optimizer variants after identifying hot work. Evaluate PGO for supported native targets with explicit training, profile merge and profile-use artifacts; establish WASM toolchain support before promising a PGO result there. Training includes the primary and maintained control workloads. Report unsupported targets or measured regressions as rejected candidates. Never leak host-specific CPU flags or instrumentation into portable defaults.

Keep profile generation, data and variant artifacts outside ordinary builds. Retain a build setting only when uninstrumented measurements and native/WASM correctness checks justify it. Compiler tuning cannot substitute for removing unnecessary work.

### 8. Acceptance and theoretical limits

Reproduce original, rounded-statistics-only and optimized 1,000-trial runs with identical primary source bytes, seeds and limits. For the policy change, derive new damage summaries independently from original rounded hit receipts and compare all unaffected formula/HP/state/RNG/causal/clock data exactly. Subsequent optimization comparisons use the rounded-statistics-only reference and require exact per-seed metrics, aggregates and detailed diagnostic values, excluding only explicitly identified engine-source hashes and host timing fields. New compact outcomes also compare against the same engine's updated detailed route. No numeric epsilon or blanket exclusion of damage-derived fields may substitute for the explicit policy oracle.

Report single-thread speedup, CPU work per seed, one/two/four-Worker scaling and actual browser completion separately. The target is 2x lower warm single-thread time (3x stretch), assessed against fresh matched baselines, with no statistically supported material regression in the controls or unacceptable memory/startup/lifecycle cost. Do not declare the numerical target met using only more Workers or a different timed boundary. If safe experiments miss the target, record the measured result and remaining hotspot limits rather than declare success at 2x or leave unbounded tuning tasks.

For a measured affected fraction `p` accelerated by `k`, estimate `S = 1 / ((1-p) + p/k)` and the elimination ceiling `1 / (1-p)`. A 2x total gain needs at least half the cost eliminated; 3x needs at least two thirds. Use disjoint or correctly nested profile regions, include preparation/fallback overhead and avoid multiplying overlapping claimed gains. This is a bound for a chosen optimization, not an absolute hardware minimum or a promise about parallelism.

## Risks / Trade-offs

- [Risk] Symbolized/instrumented builds alter optimizer or JIT behavior -> Preserve release optimization settings, isolated outputs and separate uninstrumented timing receipts; report profiler overhead.
- [Risk] Startup reuse accepts tampered artifacts or hides deferred preparation -> Complete cache identities and import validation, invalid-artifact/source checks, and measurements through the first sample and actual completion.
- [Risk] Narrow integer operations change overflow/canonicalization -> Checked intermediates, wide fallback and existing arithmetic/rounding error tests with independent expectations.
- [Risk] Rounded team totals coexist with fractional contributions, bins, exports or replay checks -> Migrate all current summary consumers, reconcile integer sums at existing boundaries and check the explicit two-hit/half-up/overkill examples.
- [Risk] Caches miss a paused expiry, source death or packet dependency -> Explicit invalidation/dependency ownership and adversarial existing owner/clock boundaries; fall back where completeness is unproven.
- [Risk] Queue and clock shortcuts reorder equal-time work -> Existing scheduler, pause, replay and horizon tests plus focused coverage for the ownership/header boundary.
- [Risk] Streaming observation hides unsupported/incomplete execution or changes floating summation order -> Online validation, detailed failed-seed rerun, exact per-seed/aggregate differential checks and existing incomplete-run controls.
- [Risk] Parallel throughput gains consume more CPU/memory or delay Stop -> Report process-tree cost and memory alongside wall time, keep bounded work units and existing stale-worker/Stop/resume checks.
- [Trade-off] More preparation and retained immutable caches may hurt small runs -> Bound caches, separate cold/single-sample costs and retain the measured simple control.
- [Risk] Guarded schedule complexity exceeds its gain -> Treat it as a bounded experiment with explicit rejection and removal of unsuccessful code.

## Migration Plan

0. Review compilation and Run startup first, compare preparation/reuse with local gcsim, and retain cold/repeated startup findings and original receipts. Distinguish browser/source compilation from Rust/WASM development builds.
1. Add isolated measurement/profiling tools and save fresh original receipts. Implement the scoped rounded-damage statistics revision, independently verify it against original hit receipts, document the new basis and save a rounded-statistics-only reference before further optimization.
2. Apply measured startup fixes, then queue, immutable preparation/cache and remaining exact-arithmetic changes in measurable increments, reusing focused checks and comparing to the rounded-statistics reference while retaining original mechanics receipts. Report startup and execution gains separately.
3. Migrate compact batch producers, Rust consumers and current WASM/Worker adapters together; validate detailed/compact and sequential/parallel parity.
4. Complete guarded reuse, scaling and build-optimization experiments; keep measured safe candidates and record rejected ones in this design. Replace stale performance claims in existing documentation rather than creating a second workflow log.
5. Run final matched native/WASM/Worker/browser benchmarks, review the stable diff, run `python tools/checkpoint.py run`, and archive only when tasks and affected checks are complete. No publication is part of delivery.

Each optimization remains independently revertible before delivery; runtime contracts are updated coherently rather than adding legacy-input branches.

## Measured implementation decisions

### Verification after synchronization with main

The working branch now starts at main commit `e31e3eed`. The original captures
below predate that commit. Main removed simulation modes and strict readiness,
changed several character mechanics, added Esper Cycles and moved field time
accounting into execution with both clock bases. It also renamed the local
artifact directory to `cache/` and changed checkpoint build reuse. The workload
source remains the same. Refresh runnable measurements against the synchronized
engine and compare the complete current observations, including reaction counts,
compact source contribution groups and field times. Keep the old receipts as
historical evidence. The local gcsim comparison still applies to the preparation
architecture, which keeps the same compile and Worker entry points.

### Original throughput and scaling reference

The rounded-policy migration preserves existing independently authored owner
formula totals as test-only sums of raw hit fractions. The shared existing test
support boundary additionally checks that native primary fields reconcile to
the rounded receipt ledger. Independent report-analysis fixtures establish the
rounding and overkill policy, including totals above u64; the historical owner
oracle values are not rounded as whole totals or replaced from candidate output.
No raw fractional total accumulator is retained in batch execution.

Forcing the existing streaming trial test into a four-Worker pool also exposed
mutation of previously delivered sample prefixes. Parallel updates now copy the
committed array at publication, as the sequential path already does. This is
part of the planned Worker lifecycle verification; it leaves sample values and
ordered commitment unchanged. The quality-reference chunk harness does not
publish controller updates, so its engine timing boundary is unaffected.

The saved original native and release-WASM runs each cover unchanged Zankou
Insight, seeds 42..1041, 120 relative seconds, three process repetitions. All
1,000 sample metrics agree exactly, as do three repetitions of S5 and the
maintained continuing-Hotori and incoming-shield controls. Each primary seed
settles 812 hits and 3,073 events. Original dedicated-aggregator chunk runs and
the actual controller preserve those metrics but expose the quality defect
described above; these receipts are explicitly not full aggregate-parity passes.
The independent original detailed oracle retains every hit's rounded amount,
independently accumulated contributions/bins and a mechanics digest per seed,
excluding only engine identity and the two intentionally revised total fields.
The existing failed-admission fixture fails in native, WASM and the actual
Worker with no committed samples; detailed reports agree after lossless wide
integer transport normalization. Source/artifact identities, raw data and
resource receipts remain under `cache/exact-throughput/original-*`.

Original native process walls are 78.759/73.524/67.319 seconds, with execution
77.134/72.196/66.156 seconds and collection 0.236/0.217/0.178 seconds. Variation
is retained rather than selecting the slowest run as the denominator. Matched
warmed chunk runs use one dedicated aggregator and the stated compute Workers:

| Compute Workers | Process wall, two runs (s) | Process CPU (s) | Aggregate (ms) | Compute linear memory each |
| --- | --- | --- | --- | --- |
| 1 | 85.515 / 81.530 | 86.734 / 83.219 | 600 / 610 | 45.56 MiB |
| 2 | 49.040 / 48.452 | 98.453 / 96.703 | 581 / 620 | 45.56 MiB |
| 4 | 32.142 / 31.804 | 122.563 / 120.266 | 573 / 593 | 45.56 MiB |

The affinity contains two physical SMT cores (logical CPUs 0..3), not four
physical cores. Four Workers provide 2.61x wall speedup over one while consuming
about 1.43x CPU; all compute threads remain occupied for roughly 29 seconds.
The final execution tail is about 0.5--0.8 seconds beyond the busiest thread's
CPU time, which also includes reconstruction and warmup. Wire decode totals
0.18--0.43 seconds; the aggregator retains 104.25 MiB linear memory after
1,000 observations. These figures do not establish pure scheduler overhead:
SMT contention, JIT/background CPU and quantized thread CPU accounting remain
mixed. No ideal fourfold scaling claim follows from this host.

The symbolized 1,000-seed WASM profile contains 52,274 samples. Inclusive
regions are queue 12.89%, static lookups 10.40%, packet metadata/child cache
9.47%, recognized fraction arithmetic 5.65%, and report observation/validation
0.42%. Their respective elimination ceilings are 1.148x, 1.116x, 1.105x,
1.060x and 1.004x; a twofold regional acceleration gives only 1.069x, 1.055x,
1.050x, 1.029x and 1.002x overall. These stack categories are incomplete
attribution, not total algorithm costs: compiler builtins and allocator leaves
can lie outside recognized parents. They are not added to overlapping costs.
Allocator malloc/free, memcmp, child-key hashing and String cloning are leading
self samples. The isolated native diagnostic run counts 1,156,830,409 additional
allocations and about 172.8 GB allocated across 1,000 seeds, with 62.36 MB peak
live allocation. It is instrumented evidence, not headline timing. This makes
payload ownership and repeated immutable construction the first engine targets;
the measured report post-pass alone offers little speedup.

### Startup reference comparison

The local gcsim reference is commit `a3b24946babe26eb5b9e9101d0e0a55c20a2ea59`.
Its `cmd/wasm/main.go::initialize`
parses a source into `simcfg` and `gcsl` once per initialization. `simulate` copies
that configuration and AST for each trial, creates a fresh core and evaluator,
then calls `simulation.New`, which performs character/target setup. Native
`pkg/worker/worker.go` accepts jobs containing an already parsed configuration
and AST and also creates a core/evaluator/simulation per trial. Thus parsing is
reused within a batch; simulation setup is not eliminated.

The actual `ui/packages/executors/src/WasmExecutor.ts` caller retains its helper,
aggregator and simulation Workers across runs, initializes each for the new
source, pipelines requests and drops obsolete run IDs. The helper compiles one
WASM module and shares it to Workers, which instantiate independent memories
(`Workers/worker.ts`). This is different from ntesim's individual Worker
`fromUrl`/`create` loading, although a browser or V8 may itself cache compilation.
The Go evaluator constructor binds the supplied AST/core and creates channels;
it is not evidence that source evaluation or preparation is free.

In ntesim, `compile_source` runs parsing, configuration/build projection,
rotation validation, talent projection, request validation and eager public
action resolution, then exports a source-bound request. `start_source_trials`
calls `from_export`, which repeats that work and compares reconstructed JSON.
The compiling Worker has discarded its original prepared scenario. Each other
Worker repeats reconstruction at its first chunk. Repeated unchanged runs still
compile before accessing the per-Worker reconstruction cache. Current cache
lookup checks source/trials/engine identity but does not cover the complete
artifact or limits; any reuse change must close that validation gap.

These are architectural comparisons based on the actual callers, not
equivalent-work timings or claims that gcsim performs no compilation. No gcsim
code was executed or copied.

### Original startup and build review (2026-11-16)

Original production assets, their inventory, WASM/glue and source identity are
retained under `cache/exact-throughput/original/`; browser receipts are in
`original/browser/measurement.json`. They bind engine `570bc632…3303`, WASM
`254bbfe5…5184`, the unchanged primary source `a57ad38d…daed4`, Chrome
157.0.8103.51 and affinity 0–3. The real controller created eight Workers on a
host reporting twelve hardware threads: seven compute instances and one
aggregator. This is distinct from the matched compute-worker experiment.

| Run | First progress / ms | First received sample / ms | First rendered sample / ms | Completed / ms | Browser CPU / s | Retained JS/WASM / MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Cold page/pool | 3.3 | 1,068.0 | 1,195.0 | 30,449.1 | 119.39 | 233.55 |
| Repeated unchanged source | 5.2 | 531.9 | 627.9 | 29,553.0 | 116.24 | 233.71 |
| Edited source (comment only) | 4.6 | 627.4 | 733.8 | 29,743.2 | 116.55 | 233.97 |
| Reload/replaced pool | 3.2 | 881.0 | 990.3 | 30,186.5 | 117.60 | 232.61 |

These are the explicit long primary benchmark, not ordinary 30-second/100,000-
event browser acceptance. Browser compile-request round trips were
96.3/35.8/33.5/72.5 ms respectively; aggregator initialization was
27.9/6.1/33.0/24.0 ms. Those whole-request boundaries are not inferred parser
times. `useConfig` also compiles the draft on the page before Run is enabled;
its prepared program cannot be shared as mutable state with a simulation Worker.

The diagnostic native one-sample receipt (`startup-native/`, three processes)
binds the original statistics implementation plus opt-in measurement tooling
(engine `ce14fa79…a258`). Median compilation was 54.019 ms, reconstruction
33.213 ms, export JSON 0.071 ms and collector construction 0.013 ms. Across
compilation, reconstruction and the first sample there were two parses
(0.390 ms combined), eight actor projections (31.649 ms), two configurations
(42.561 ms), 54 eager descriptor attempts (40.880 ms), 61 total descriptor
computations and 134 runtime lookups. Seven computations were additional
runtime variants. Rotation validation was 0.022 ms and talent projection
1.553 ms. Regions nest; these numbers must not be added together. The
instrumented first sample was 143.683 ms median, with about 43 MiB process
peak residency; its counter overhead is excluded from headline comparisons.

`startup-worker-phases/` independently measures the production Worker adapter
and generated bindings with four compute instances plus an aggregator. Module
compilation took 14.9–55.8 ms per instance, synchronous instantiation
0.35–0.59 ms; the compiler round trip spent 84.7 ms in Rust/wire production
and 0.87 ms decoding. Reconstruction/collector startup spent 29.0 ms in
Rust/wire production. Concurrent cold detailed chunks spent 530–582 ms in
Rust/wire production and 31–36 ms decoding, including their own reconstruction
and sample. Compute memories reached 45.56 MiB each, versus 9.5 MiB for the
aggregator. This diagnostic warmup is separate from production completion.
Thread CPU is available but quantized at roughly 15.6 ms on this Windows host.

Startup traces in `original/browser-profile/` are diagnostic repetitions, not
headline timings. They show eight JavaScript Worker module compilations on
cold startup and none on repeat. This Chrome category set did not expose
separate WASM compilation events; the explicit Worker compilation measurement
above supplies that boundary. Earlier interrupted exploratory Node timings
are excluded from acceptance after process reaping could not be established.

Development-build receipts live in `build-review/`. The retained pre-change
checkpoint measured native build 31.37 s and WASM build/glue 186.61 s.
The new benchmark's first instrumented release build took 176.7 s: content
101.72 s (18.62 s frontend, 83.10 s codegen), research 82.24 s (5.23 s
frontend, 77.01 s codegen), overlapping; the final binary unit took 61.64 s.
That final unit includes thin-LTO/backend/link work; Cargo does not isolate
linker-only time. A subsequent diagnostic edit rebuilt in 155 s, changing
to the uninstrumented feature set took 147 s, and a matching no-op build
took 0.49 s including the wrapper. These are exact recorded commands, not a
claim about all native incremental edits. Current ordinary WASM compilation
took 177.08 s and wasm-bindgen 0.636 s; wasm-opt was unavailable and did not
execute. Web bundling/staging took 6.044 s. No cache was deleted and build
seconds are not Run-time gains.

Prioritize retaining the fully validated prepared program in its compiling
Worker and validating complete cache identities, then evaluate eager
descriptor removal through first-sample/completion measurements. Standalone
export JSON and collector setup are too small to be leading startup targets.
Keep module sharing as an experiment: V8 cache behavior, background compilation
and independent memories must be measured rather than assuming eight fresh
full compiles. Existing native source preflight/import tests cover diagnostics
and reconstruction; source-trial/resume and streaming Worker tests cover ordered
prefixes and fresh-Worker replay. They do not establish safe warm-cache reuse
of a tampered request or changed compile limits: extend those owning boundaries
in task 1.10. Apply startup candidates only after the rounded-statistics reference.

### Measurement tooling and isolation

The `profiling` Cargo profile inherits release optimization/LTO/codegen/panic
settings and retains debug information. The native benchmark produces a PDB
containing `PreparedSourceScenario` and `run_sample`; the profiling WASM exposes
named Rust functions to V8's CPU sampler, including metadata interning and hit
settlement. Both initial builds and a repeated profiling WASM build preserved
all five ordinary glue/WASM files and the ordinary native benchmark executable
byte-for-byte (`profile-isolation.json`). Profiling WASM/glue has its own target
directory. These large symbol-bearing artifacts are not startup/headline inputs.

`ntesim-throughput` is a required-feature development binary over the actual
source executor and Studio collector. The Python host caps inherited CPU use,
binds artifacts/tools/source and records process CPU/peak residency as well as
explicit inner timings. Running its absolute script path from another working
directory reproduced a bounded measurement. The WASM/session, matching chunk
and actual controller hosts reuse existing generated bindings and Worker
adapters. Their native/JS raw observation and receipt-writing costs are stated
separately; no long measurement is added to checkpoint or Team list generation.

Content spans and the native allocation counter require the non-default
`performance-diagnostics` feature. The allocator lives only in the explicit
benchmark target. Node inspector/sampling support lives only in development
hosts. Portable native/browser validation now rejects those driver, profiler,
counter and symbol artifacts; existing distribution and glue-isolation tests
pass, and ordinary native/WASM/browser bytes pass the artifact guard.

Windows topology reports selected logical CPUs `[0,1]` on one physical core
and `[2,3]` on another. This is four logical CPUs on two SMT cores, not four
physical cores. Worker thread CPU is available, with coarse Windows ticks;
process CPU also includes V8 background/JIT work and receipt output. Use the
matched one/two/four compute-worker receipts before assigning scaling costs.

### Baseline after synchronization

`synced-main-artifacts/manifest.json` binds the fractional-statistics reference
to main commit `e31e3eed1559da585881aaa070aef926ca5b3074` and engine
`48667bc93ce0db0b30489b6c2137bc318dcf87b96ed9f823a04e575cda24d22f`.
Only the development measurement entry point and optional phase spans were
added to that scratch checkout. The unchanged Insight input completed all
seeds 42–1041 in three ordinary native and WASM repetitions and in the
detailed native oracle. Median process wall was 67.46 s native and 74.17 s
sequential WASM. Matched warm one-, two- and four-compute-worker medians were
79.97, 46.80 and 31.15 s. `main-s5-*`, `main-hotori-*`, and `main-shields-*`
contain the corresponding controls, including raw observations, CPU time and
peak working sets. All runs used logical CPUs 0–3 across two physical cores.

Timed chunk round-trip occupancy was 99.3–100% across the compute pool.
The four-worker finish spread was 103–214 ms and the final collector pass
took 605–617 ms. Median transport outside measured Worker handling was
approximately 6–9 ms per chunk. One-to-four compute scaling was 2.57× wall
while process CPU rose from 82.23 to 121.14 s. The shared physical cores and
V8 background work are part of that cost. The measurements quantify combined
CPU inflation without assigning an independent cost to either contributor.
Each compute Worker ended at 45.56 MiB linear memory. The collector reached
106.5 MiB when the matched harness posted all observations at the end. The
harness peak working set includes outstanding JS rows and raw receipt output.

The synchronized 250-seed WASM diagnostic (`main-symbolized/`) has 13,845
CPU samples. Allocator frames account for 18.45% inclusive, queue operations
12.43%, static descriptor queries 10.07%, clock work 8.02%, metadata 7.87%,
and named fraction operations 2.31%. Sample classes overlap. Ideal isolated
Amdahl limits are respectively 1.226×, 1.142×, 1.112×, 1.087×, 1.085× and
1.024×. Allocator samples also occur below these subsystems. The accompanying
native diagnostic counted 25.36 million allocations and 3.70 GB allocated
across 20 trials, with 31.3 MiB peak live allocation. The profile build
produced a 98,185,216-byte PDB with source preparation and sample symbols.
`main-symbolized/isolation.json` confirms both the original archive and the
ordinary current executable and glue retain their prior hashes.

`main-browser/measurement.json` records production Chrome with eight Workers
total and seven compute Workers. Times are relative to Run. The Worker
messages and UI mutations delimit browser phases, including first observation
commit and render. The source stage at the beginning of every run includes
bridge overhead.

| Run | First render | Completion | Completed render | Process CPU | Retained memory |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cold | 1,323 ms | 31.12 s | 31.13 s | 120.18 s | 234.1 MiB |
| Repeated | 600 ms | 30.16 s | 30.16 s | 116.97 s | 234.3 MiB |
| Edited comment | 718 ms | 30.08 s | 30.10 s | 116.49 s | 234.5 MiB |
| Replaced Workers | 879 ms | 30.48 s | 30.48 s | 117.27 s | 232.1 MiB |

Cold compile and aggregator responses arrived about 122 and 160 ms after Run.
Repeated compile and aggregator startup completed by about 44 ms. These
boundaries include communication, while opt-in phase counters measure Rust
preparation and serialization independently. Rebuilding the synchronized WASM
reference took 195.02 s. The synchronized native release Cargo timing report
measured 159.2 s total, with overlapping content (94.05 s) and research
(79.58 s) units. The final binary unit, which includes ThinLTO, took 54.52 s.
The native profile settings are identical across the synchronization, so the
earlier incremental receipts still describe those specific edits. No-op
native and WASM Cargo builds took 0.13 and 0.16 s, bindgen 0.75 s and Vite
bundling with distribution staging 3.40 s. The optional `wasm-opt` executable
was unavailable.

The synchronized native diagnostic reference (`main-diagnostics-primary/`)
measured 46.84 ms for compilation, 0.093 ms for export JSON, 29.10 ms for
reconstruction and 0.018 ms for collector construction. The first detailed
sample took 124.25 ms on that instrumented process. The two passes made 54
eager descriptor attempts (35.13 ms), eight build projections (28.52 ms),
two talent projections (1.20 ms) and two parses (0.41 ms). These nested
spans explain which work can be removed before measuring the Browser path.
The matched Worker receipts measure WASM module compile, instance setup and
bridge handling separately, alongside browser creation, first progress,
sample receipt and render. The validated-program reuse and exact warm-cache
identity checks address the repeated named preparation passes. The unchanged
module lifecycle remains covered by the existing WASM loading checks.

The rounded-only snapshot is bound by `rounded-artifacts/manifest.json` to
engine `c874edd1a09e6341345a7a4fcd31cb631ee025db5516fd52e638cc0c4df10deb`.
The independent oracle reconstructed the rounded contributions and time
bins from each historical hit across all 1,000 seeds. It matched the new
integer ledger and all detailed mechanics digests, including exact formula
fractions, HP settlement, RNG, causality and both clocks. Mean relative DPS
changed from 37,840.72294094617 to 37,840.583050000045. The rounded native,
sequential WASM and four-compute-worker results agreed exactly on samples
and aggregate fields, with elapsed timing excluded. Native and WASM control
results also matched for S5, Hotori and incoming shields.

Rounded-only median native wall time was 65.30 s, CPU 64.30 s and peak
residency 176.7 MiB, compared with 67.46 s, 66.66 s and 183.1 MiB on the
fractional reference. Sequential WASM wall time was 77.90 s, CPU 78.44 s and
peak residency 184.6 MiB, compared with 74.17 s, 75.33 s and 185.5 MiB. The
two release builds therefore show a modest native gain and a WASM regression
for this isolated change. Subsequent exact comparisons use rounded-only
receipts and evaluate engine changes separately.

### Compact trial receipt consumers

The detailed sample and source checkpoint retain authored occurrences, replay
commands, live timing contexts, RNG audits, complete state frames, catalogs
and final state. The compact executor uses the same scheduler and retains
commands and packet coordinates needed by admission and callbacks. Hit
receipts remain available for reaction replacement and parent identity. Each
event validates new hit times and parent links before accumulating checked
integer contribution totals and dyadic absolute-time bins. Bins coarsen before
converting to floating-point rates, and a hit exactly on the final boundary
belongs to the preceding bin.

Energy, player inputs and action admissions are consumed after an event's
rollback paths settle. Cast counts retain command identity across events so a
player control is counted once. Trace grants follow the full observer's
addition order and reconcile with the runtime energy ledger. Healing receipt
identity and balances are checked before summation, and shield absorption
retains redirection order. Shield integrals and field times use the same
clock-advance path for both representations. Compact success still requires
complete bounded execution and verified state. Failed seeds are rerun from
their original seed for a detailed diagnostic. The first report, explicit
sample loading and checkpoint replay use the detailed path.

`reactions/replacement.rs` consults the triggering hit before a replacement
settles. Hit indices also bind follow-ups to their parents. The executor
keeps the hit ledger until trial end. Action metadata identifies admitted
action roles, while packet evidence and used hypotheses still feed the
assessment. The action and trace vectors temporarily accept normal callback
receipts so `support.rs` can roll back a speculative action before any
observation is committed. Source occurrences identify time contexts and
failed inputs, and the source checkpoint hashes occurrences, trace and hit
prefixes with the actual RNG, scheduler clocks and actor/target state. The
compact path has no checkpoint boundary. It checks input-time intervals
online with a contiguous end coordinate and reconciles the result with
absolute elapsed time. RNG streams and their live counters stay
in the simulator, while only detailed outputs serialize the RNG audit and
full final state. Compact completion checks field-time totals, clock bounds,
state invariants and final energy without constructing actor/target/buff
snapshots.

The first combined queue/preparation/compact candidate is frozen under
`candidate-artifacts/` with engine
`2b730c86827f6f0f0d850df068bac3311edd17ffb4a5f8873505c8daaf9720a6`.
All 1,000 compact observations and aggregate fields matched the rounded-only
reference in ordinary native and sequential WASM. The single native run
took 37.87 s wall and 37.58 s CPU, compared with the rounded reference medians
of 65.30 s and 64.30 s. Sequential WASM took 56.91 s wall and 57.89 s CPU,
compared with 77.90 s and 78.44 s. Peak process residency was 175.9 MiB native
and 202.4 MiB WASM. The WASM reference was 184.6 MiB. These figures are a
candidate measurement, with repeat trials and browser memory measured
separately for acceptance.

`candidate-native-oracle/` matched the rounded reference's detailed digests
and observations at all 1,000 seeds. It covers metadata and evidence catalog
first use, action occurrences, causal parents, RNG audits, HP settlement,
and clocks. The existing replay and failed-admission tests also reproduced
the detailed failure generated from an independent seed execution.

The production browser trace (`candidate-browser/`) captured all 1,000
samples and the complete ordered aggregate for each lifecycle. Each matches
the rounded native reference. Browser retention and transport are separate
from sequential Node process residency.

| Run | First response / ms | First render / ms | Completed render / s | CPU / s | Retained / MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cold | 1,058 | 1,202 | 21.165 | 82.67 | 200.6 |
| Repeated | 445 | 542 | 20.493 | 79.48 | 200.8 |
| Edited comment | 536 | 639 | 20.526 | 79.56 | 201.2 |
| Replaced Workers | 707 | 824 | 20.556 | 79.75 | 199.3 |

Cold source compilation and aggregator request took 92.4 and 4.0 ms round
trip, compared with 120.8 and 35.3 ms on synchronized main. In a repeated run
the compile request dropped from 36.3 to 6.2 ms and aggregator startup took
10.5 ms. With a comment edit the compile request took 32.1 ms and aggregator
startup took 7.9 ms. Producer and consumer used 21 ordered observation
commits. Transport recorded 14.68 MB and 50–54 ms of encoding across the
candidate runs, compared with 17.59–17.88 MB on synchronized main. Separate
Worker coverage held the first chunk and verified the maximum of six
uncommitted requests with three compute Workers, then stopped the blocked run.

The combined native diagnostic (`arith-diagnostics/`) made 28 eager
descriptor attempts versus 54 on synchronized main. Total descriptor
computations fell from 61 to 35 on the same compile, reconstruction and 20
trials. Both builds made seven runtime computations after eager preparation.
Compile and reconstruction dropped to 38.87 and 20.83 ms, the first sample
to 104.50 ms, and peak live allocation to 30.5 MiB. The paired diagnostic
counted 16.38 million allocations and 2.41 GB allocated, versus 25.36
million and 3.70 GB on synchronized main. This joint result covers clock,
preparation, compact observation and arithmetic changes.

The three repetitions in `candidate-s5-*`, `candidate-hotori-*` and
`candidate-shields-*` matched the rounded reference's control samples and
statistics. S5 median wall fell from 22.04 to 12.17 s native and from 24.26
to 15.65 s WASM after the queue, preparation and compact changes. Hotori's
one-sample medians were 28.6 ms native and 247 ms WASM, and shields' were
21.0 and 202 ms. The short controls include process startup.

### Dependencies for preparation and reuse

Prepared source caches belong to one exact source artifact, including both
compile limits and the canonical initial request. Each actor's action ID
indexes a single immutable descriptor. Runtime cache entries also compare
the complete selected command and typed parameters, after owner callbacks
choose command coordinates. Static damage bonuses are projected per actor,
packet element and Mental damage flag from the validated initial request.
Even an invalid combination is stored as a result and is only observed when
reached. Team composition reads authored identities and stays fixed through
defeats. Effective live modifiers still read the buff manager at each hit.

The shared prepared action table is capped by the source action budget. The
child packet resolver has a 4,096-entry cap and checks the full coordinate and
assumption map after selecting a hash bucket. Each trial caches packet
normalization for up to 512 catalog identities. Every comparison field is
preserved by the catalog, evidence IDs or validated hypothesis IDs, and the
execution-local energy group is returned independently. Reaching either cache
limit continues through the ordinary resolver or catalog path.

The borrowed buff aggregator supports arbitrary stat keys and reports overflow
in application order. A dense typed representation must retain unknown keys
and this arithmetic order. A reusable live result would also need the exact
set of living sources, active/recipient identity, the relative time through
stack decay and expiry, and the speculative buff copy for a pending hit. The
static table deliberately excludes these dependencies. Keep live evaluation
until a cache demonstrates complete invalidation and a measured benefit.

Cross-seed schedule replay is rejected for the general source route. The
source evaluator reads live HP, energy, and elapsed clocks after causal work
has settled. Crit and proc outcomes can move those values, schedule owner
follow-ups, defeat an actor or target, or prevent an action from being
admitted. A source with a fixed outer sequence can still reach those dynamic
dependencies inside a kit. Timestamp equality at a previous action supplies
no proof for the next admission. The bounded damage arithmetic experiment
starts after live modifiers, mitigation and special multipliers are resolved,
and leaves settlement and all RNG sampling in their existing order.

The source test `source_seed_branches_read_settled_hits_without_consuming_randomness`
selects two real seeds across a settled HP threshold. They take different
100 ms and 200 ms branches and each retains its independent RNG audit.
Existing same-time reaction, seeded proc, critical-window, target-defeat,
and rejected-branch tests passed on the prepared candidate. Buff tests
continued to exercise refresh, independent stack decay, pause expiry,
living-source and recipient rules through current modifier evaluation.

### Pre-critical memoization decision

The experimental table used 256 direct-mapped entries per thread. A lookup
hashed the entire typed `ExactDirectDamageRequest` and compared it for equality
before using the result. The request includes damage kind, attacker level,
resolved stat and scaling, effective bonuses, all mitigation inputs, the
resistance model, special multiplier and critical bonus. Only successful pure
evaluations occupied an entry. Context validation, live modifier callbacks,
critical sampling and settlement still ran for every hit.

`precrit-native-oracle/` matches the rounded reference across 1,000 detailed
reports and RNG audits. The `damage-cache-control.nte` probe alternated target
defense and resistance, changed actor and packet context, and settled held
skill effects through the same cache. Its eight detailed digests matched the
direct evaluation. Primary medians were 37.45 s native and 54.49 s WASM with
the cache, against 37.41 s and 53.48 s for the earlier direct-evaluation runs.
The small difference and calculation result equality favor direct evaluation.
The experiment remains confined to its scratch source and archived receipts.

The cached S5, Hotori and incoming-shield runs matched the existing control
samples, elapsed clocks and statistics. Native compile plus reconstruction
was 43.32 ms cached and 43.20 ms direct in the paired receipts, and first
sample took 73.68 and 71.43 ms. These wall and CPU measurements include cache
initialization, key comparison and ordinary evaluation on misses. Peak
process residency was 176.2 MiB cached and 175.6 MiB direct native. In WASM
the corresponding figures were 190.8 and 200.3 MiB. The direct WASM figure
comes from the single paired baseline. The stable engine uses direct exact
calculation and normal scheduling for all sources.

### Queue decision

The projected head carries the next absolute time, causal priority, sequence
and stop-policy classification. Extraction projects the owned payload. The
queue reuses a selected head until push, pop, cancellation, replacement,
active-stop reclassification, or a clock/policy transition affects eligibility.
The compact progression policy represents the intersection of active stops
and cinematics. Zero elapsed time leaves its mode coordinates in place, while
owner timer handling still settles any state due at the same timestamp.

The `old-queue-native/` experiment restores the synchronized queue, clock
policy and stopping checks while leaving the rest of the candidate intact.
Matched three-by-250-seed runs retained identical observations and statistics.
Current queue median wall and CPU were 9.97 and 9.72 s, compared with 11.64
and 11.47 s with the original queue. Median peak working set was 63.5 versus
64.2 MiB. The remaining domain selection scan runs after invalidation. It
still resolves relative-deadline projection and the same ordered priorities.

The paired 20-seed diagnostic reduced the isolated queue variant's 19.84
million allocations to 16.38 million and 2.75 GB allocated to 2.41 GB.
Peak live allocation was 30.5 MiB in each. The successful queue paths also
passed the existing pause, equal-time, Hotori and replay tests, the full
primary oracle and the rounded-reference control comparisons.

### Exact arithmetic experiments

On the 20-seed synchronized primary diagnostic, 550,164 fraction GCD calls
had a unit operand, 768,735 other pairs fit in 32 bits, 27,179 fit in 64 bits,
and 12,609 needed the wide path. All 443,547 buff-aggregation GCD calls fit
in 32 bits. The same probes sampled S5, Hotori and incoming shields.
`gcd-probe.json` records the weighted operand sample and five repetitions of
each algorithm on both targets. Native median times were 14.93 ms for
binary `u64` GCD, 26.53 ms for Euclidean `u64` and 98.07 ms for Euclidean
`u128`. WASM medians were 26.45, 29.80 and 121.59 ms. Each implementation
produced the same checksum.

The candidate uses unit and zero shortcuts, a checked width test, binary
`u64` reduction and the existing wide Euclidean fallback. Integer fractions
are canonical immediately. Coprime values keep their exact numerator and
denominator, and a checked narrow reduction uses `u64` division after the
width test. Addition keeps the existing equal-denominator fast path and
checked arithmetic order. Public `ExactFraction` values can be noncanonical,
so a canonical-aware addition shortcut would change its handling of some
valid inputs. Existing arithmetic coverage exercises that distinction.

The first arithmetic candidate retained all 1,000 observations and aggregate
fields, with no score difference from the combined candidate on either target.
Native process wall and CPU were 37.41 and 37.06 s, versus 37.87 and 37.58 s.
Sequential WASM was 53.48 s wall and 54.02 s CPU, versus 56.91 and 57.89 s.
These are single matched candidate comparisons. The frozen current snapshot
is `arith-artifacts/`, engine
`a0b488d902765dd54d7ca7756a9466b829616297a823779d1364d59677237dec`.

All three controls retained their exact samples and aggregate metrics in
both arithmetic variants. S5 medians were 12.17 to 11.64 s native and 15.65
to 15.78 s WASM. Hotori was 28.6 to 27.1 ms native and 247 to 226 ms WASM.
Shields was 21.0 to 18.7 ms native and 202 to 208 ms WASM. These short
controls include startup and show mixed small differences. The retained
binary reduction has a clear operand-level benefit and lowered sequential
primary WASM CPU by 6.7% in the matched candidate run.

### Compiler target support

The native FatLTO release and WASM `opt-level=2` variants used the same
arithmetic candidate source, separate target directories, recorded build
flags, and two uninstrumented full primary repetitions. Both reproduced all
1,000 exact observations and aggregate values. Native FatLTO median wall was
39.81 s and CPU 39.44 s, compared with the ThinLTO run at 37.41 and 37.06 s.
The WASM `opt-level=2` output was 10,665,252 bytes versus 11,045,633 at level 3.
Its median wall and CPU were 55.01 and 55.80 s versus 53.48 and 54.02 s. The
smaller WASM had lower measured Node residency, with a throughput tradeoff.
The portable release retains level 3 and ThinLTO. Binaryen was unavailable
in this build environment.

The pinned Rust 1.98.1 `wasm32-unknown-unknown` `-C profile-generate` probe
failed with E0463 for the missing `profiler_builtins` crate. Regular WASM
compilation succeeds with that toolchain. The WASM probe receipt is separate
from native PGO training and the portable release flags.

Native PGO used two isolated target directories over the same frozen source.
The `profile-generate` driver trained on 128 primary seeds, 64 S5 seeds and
the Hotori and incoming-shield controls. Pinned `llvm-profdata` merged the four
runtime profiles into `pgo-data/merged.profdata`; its digest is recorded in
`pgo-data/training.json`, with the training binary, source and output hashes.
The separately built `profile-use` driver matched observations and aggregates
for both 1,000-seed primary runs and the three native controls. Primary median
wall and CPU were 35.46 and 35.34 s, compared with 37.41 and 37.06 s in the
paired portable run. S5 median wall was 10.56 s versus 11.64 s. Process peak
was about 175.6 MiB for either primary artifact. Training builds and the merged
profile are explicit local measurements. Ordinary builds keep portable flags.

### Matched Worker scaling

`final-chunks-{1,2,4}/` contains two full repetitions on the retained release.
Every sample, observation, and aggregate reproduced the rounded reference.

| Compute Workers | Synchronized wall / s | Final wall / s | Final CPU / s | Final process peak / MiB |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 79.97 | 55.19 | 57.20 | 483.5 |
| 2 | 46.80 | 33.05 | 66.98 | 577.4 |
| 4 | 31.15 | 21.01 | 79.79 | 769.4 |

Round-trip occupancy stayed at 99.2–100%. At four compute Workers the finish
spread was 88–151 ms and the final collection pass took 519–528 ms. Median
time outside measured Worker handling was 8.0–9.3 ms per chunk. For the
2.63× wall gain from one to four compute Workers, process CPU increased by
22.59 s. The sampled topology uses two physical cores with SMT. Process CPU
includes V8 background work and concurrent transport. The harness retained
44.7 MiB linear memory per compute Worker and 100.7 MiB for the collector.
Its process peak also includes JS observations held until final submission
and raw receipt output. Production browser retention has its own boundary.

With four compute Workers the 25, 50 and 100 seed chunk experiments took
21.08, 21.01 and 23.47 s median. Chunks of 100 raised the final completion
spread to 5.82–6.49 s, with two compute Workers below 74% round-trip occupancy.
Sharing a compute instance for the harness's final collector submission took
21.10 s and about 749 MiB process peak. The production collector processes
ordered chunks during execution, so the end-of-run shared experiment provides
limited evidence for changing its placement. The controller retains 50-seed
chunks and a dedicated collector. Total capacity is twice the compute Worker
count across executing and returned chunks. One drain processes
ready chunks by offset, acknowledges a commit before releasing credits, and
reports failed seeds in that same order. Delayed-response tests exercise the
bound and earliest-failure reporting. Stop releases both capacity waits and
pending responses. The default pool policy stays tied to the browser's
reported hardware and memory. The four-logical-CPU measurements support the
local scaling comparison under this host's affinity.

### Final release measurements

Three native and three sequential WASM repetitions (`final-native/` and
`final-wasm/`) matched all 1,000 rounded observations and the full aggregate.
The synchronized baseline and rounded-only results separate the change in
score policy from the engine speedup.

| Route | Synchronized median wall / s | Rounded-only median / s | Final median / s | Final wall range / s | Gain over synchronized baseline |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native | 67.46 | 65.30 | 37.28 | 37.14–37.52 | 1.81× |
| Sequential WASM | 74.17 | 77.90 | 54.74 | 53.17–55.31 | 1.35× |

Final native median process CPU was 37.00 s with about 175.9 MiB peak.
Final sequential WASM median CPU was 55.47 s with about 182.8 MiB peak.
The two warm single-threaded improvements were below the 2× target and 3×
stretch goal. Their gains over rounded-only were 1.75× native and 1.42× WASM.

All four production Chrome runs (`final-browser/`) reproduced the 1,000
ordered samples and aggregate. The default machine pool has seven compute
Workers and one aggregator. Browser completion is independently measured
through the app status and the next rendered frames.

| Run | First Worker sample / ms | First render / ms | Completed render / s | CPU / s | Retained / MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cold | 964 | 1,078 | 20.412 | 79.07 | 200.5 |
| Repeated | 448 | 539 | 19.919 | 76.49 | 200.7 |
| Edited comment | 523 | 613 | 19.458 | 75.84 | 201.1 |
| Replaced Workers | 635 | 742 | 19.828 | 76.82 | 199.2 |

The repeat after moving request routing into the dedicated transport module
(`final-browser-transport/`) also matched every sample and aggregate in all
four runs. Completed render times were 20.749, 19.673, 19.686 and 19.818 s
in the same order, a 19.75 s median versus 30.32 s on synchronized main.
The cold first render took 1,095 ms and retained memory remained 199–201 MiB.

Median completion for the first four runs improved 1.53× versus synchronized
main. The cold source and aggregator round trips were 82.7 and 5.7 ms.
The repeated compilation round trip was 6.2 ms. All runs transferred close to
14.68 MB. The sum of the
21 ordered collector round trips was 1.13–1.22 s, overlapping computation.
The final completed-render delay after status was 3–9 ms. The raw browser
receipt includes worker initialization, transport, UI phases and retained
JS/WASM memory, independently of process totals from the Node harness.

The final 250-seed symbolized run retained 9,634 samples, versus 13,845 on
the synchronized reference. Allocator frames account for 21.74% inclusive,
queue work 15.90%, clocks 10.15%, metadata 6.68%, named fraction operations
2.53%, observation 0.28% and static descriptor queries 0.08%. The corresponding
isolated Amdahl limits are 1.278×, 1.189×, 1.113×, 1.072×, 1.026×, 1.003× and
1.001×. These categories overlap, and the percentages use a smaller final
sample total. The final queue and clock shares therefore describe remaining
work under the new workload cost. The isolated timings and allocation pair
measure the queue change directly.
