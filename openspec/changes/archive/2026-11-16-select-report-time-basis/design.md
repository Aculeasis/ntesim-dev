# Design

## Context

See [proposal.md](proposal.md) for motivation. Current `studio::observe` divides damage by relative duration, energy/healing by absolute duration, and builds one bounded absolute damage histogram. `Collector::rate_stats` and `snapshot` summarize per-trial values in Rust. `Sample` already retains both elapsed totals. `Summary.tsx` owns a duration-only selector, while `ResultsPage.tsx` independently renders histograms and contribution panels.

Shield capacity is already integrated during event-driven clock advancement in `crates/content/src/simulation/runtime/time.rs` using a checked `u128` HP-microsecond accumulator; `runtime/execution.rs` finalizes the absolute `available_shield_hp` scalar. Lean batches rely on this value rather than replaying state timelines. Existing native shield tests retain a timeline-derived independent oracle. Exact sample analysis and comparison endpoints use canonical relative DPS independently of Studio presentation.

The accepted Studio requirements explicitly limit selection to duration and retain absolute support definitions. This change replaces those requirements rather than treating the current mixed bases as an implementation bug.

## Goals / Non-Goals

**Goals:** Select native statistics for either basis immediately, retain a compact single execution payload, and calculate relative shield coverage correctly even when shields change inside stopped time. Keep arithmetic, availability and statistics in Rust, with TypeScript selecting and rendering delivered values.

**Non-Goals:** A second damage timeline, relative playback geometry, revised gameplay timers or action scheduling, relative field-time accounting, changing exact sample/comparison scoring, refreshing curated rankings, publishing or deployment.

## Decisions

### 1. Pair only values whose meaning depends on the selected basis

Use a shared typed relative/absolute pair for the existing per-seed DPS, retained EPS, granted EPS, HPS and SHP fields and their aggregate statistics. Each side is explicitly nullable where unavailable. Use the same pair for actor/source/target DPS statistics; retain each identity and its damage/hits/action/energy totals only once. Keep existing relative and absolute elapsed totals and duration distributions rather than adding another copy.

The detailed report's shield-summary field becomes the corresponding pair of native averages. Update Rust models, current native consumers, WASM serialization, TypeScript contracts, Worker retained-prefix checks and storage validators together. Remove the superseded single-basis field shapes rather than maintaining fallback aliases. The compact wire encoder continues interning repeated keys; do not introduce a new encoding or generic metric catalog for this small fixed set.

This adds at most five extra scalar summary values per successful seed, plus a second statistics object for each basis-dependent aggregate. It does not add actor/source breakdowns per seed, an entire second aggregate object, or copies of static metadata and damage arrays. Absolute rates must be retained independently because a zero-relative-duration trial cannot yield them by scaling an unavailable relative rate. A ratio of aggregate means also cannot recover their distribution.

### 2. Calculate rates per trial, then summarize each basis

Reuse settled damage totals, reconciled gross/retained energy ledgers and canonical gross healing receipts in `crates/research/src/trials/studio/observe.rs`. For positive elapsed seconds `T_b` in basis `b`:

```text
DPS_b         = settled damage / T_b
EPS_b         = retained energy / T_b
granted EPS_b = granted energy / T_b
HPS_b         = gross healing / T_b
```

Reuse the current statistics implementation for every basis, including mean, extrema, population standard deviation and interpolated quartiles. Contribution DPS divides each seed's contribution by that seed's duration before statistics; relative and absolute contribution shares can therefore differ across seeds. Missing sources still contribute zero for that seed, as today. Chunk merge and resume must yield the same values as uninterrupted aggregation.

Availability is per metric and basis. A zero denominator produces `null`, including zero activity; any missing required per-seed value makes that metric's full-set statistics/histogram unavailable. Do not omit undefined seeds. Preserve existing unsupported-model and incomplete-run exclusions: an absolute display must not make unsupported damage valid. Where canonical DPS is unavailable solely because relative duration is zero, supported absolute DPS can still be displayed for positive absolute duration. EPS/HPS/SHP availability is not inferred solely from the DPS flag.

Canonical exact damage analysis, research/comparison contracts, Team list rankings and `game_dps_computed` retain their relative meanings. The report selector is UI state, never a request option or a comparison-admission override.

### 3. Add one relative shield integral at the existing advancement boundary

Let `C(a)` be remaining absorbable shield HP on the active living actor at absolute time `a`, and `R(a)` the native relative clock. Finalize:

```text
SHP_absolute = integral C(a) da / absolute_elapsed
SHP_relative = integral C(a) dR(a) / relative_elapsed
```

Extend the existing checked accumulator with a second `u128` integral. Reuse its active-recipient, living-state, additive shield and exact-expiry clipping. Clock advancement already distinguishes paused World segments and timer transition boundaries: only advancing relative segments add to the second integral. Use the actual native relative delta and assert the segment invariant instead of inferring pauses from an ability name or sampling time periodically. At equal-time transitions, retain existing causal settlement and zero-duration behavior. Overlapping pauses exclude their union once.

Grant, depletion, removal or a switch during a pause still changes the capacity used after release. Such intervals add to absolute weighting while adding no relative weight. Capacity available only during ten paused seconds of a twenty-absolute/ten-relative-second trial gives 5000 absolute SHP and zero relative SHP for a 10000 HP shield. Dividing the old absolute integral by relative duration would incorrectly give 10000.

Finalize each average only for its positive denominator; retain unavailable values for zero elapsed time. Preserve sample/lean-batch parity and extend the existing native shield oracle to verify both clock weightings. No additional event, frame, timer, scheduler pass or per-hit shield record is necessary. This is the only new calculation inside combat execution; the remaining work belongs to result aggregation and consumers.

### 4. One UI selection drives applicable Results consumers

Lift the current selector state out of `Summary` to a run-scoped presentation owner shared by summary and charts. Use `runId` to reset on a new execution/import while preserving the value on incremental delivery, stop/resume and view navigation. Pass the selected basis to Summary, Histogram and damage contribution panels (or a small pure presentation selector they share); avoid cloning `TrialResult` or modifying its native aggregate in place.

Place the same Relative / Absolute control at the upper right of Damage Per Second (DPS); Sim Duration (Dur) loses its control and follows that selection. Keep a suitable accessible name and current option text, with no new visible note, tooltip, legend or per-card basis badge. Relative remains the initial selection, even when unavailable. Histogram duration selects the matching existing elapsed distribution; DPS/EPS/HPS/SHP histograms select native per-seed pairs. All statistic tooltips and unavailable states use the chosen values. Damage, hit counts, actions, energy-source totals, final energy and absolute field time retain their existing meanings.

PNG captures the active selected view through the existing capture path. JSON, compressed reports and embedded PNG data always contain both bases and restore without simulation; imports start at Relative, matching new-run behavior. No extra persisted view-state payload is needed.

### 5. Keep the absolute timeline and Sample useful

Rename the panel exactly to `Damage Timeline (absolute time)` and the rate option to `Damage per second`; keep `Cumulative Damage`. Match chart inspection/accessibility wording to the selection. Its one existing absolute histogram, endpoint, rates and cumulative totals are independent of the DPS-card selection. Cumulative damage is a total plotted against absolute time, not a total divided by absolute duration. Retain the adaptive maximum of 240 aligned buckets and no relative bin array.

Sample keeps absolute seek, speed, event stepping, pause spans and both time readouts. Its exact sample report and CSV remain canonical relative analysis. Transport fixtures may change with the shield-summary contract, but playback geometry needs no feature work.

### 6. Update living documentation and validate the bounded overhead

Update `openspec/specs/studio-application/reference.md` (summary formulas, selector and timeline sections), `openspec/specs/combat-execution/reference.md`, `openspec/specs/wasm-transport/reference.md`, `docs/NTE_DSL.md` and `docs/mechanics/shared/clocks-and-action-timing.md`. Remove the obsolete duration-only and permanently absolute support explanations. Preserve historical acquisition/performance receipts as historical evidence; add the new bounded size/timing result to `docs/BROWSER_PERFORMANCE.md` with its scope and limitations.

Before implementation, retain a baseline using the existing production acceptance measurement boundary and its no-Ultimate, ordinary-Ultimate and continuing-Hotori sources. Afterward compare identical seeds/trial counts under the same four-CPU limit: event/hit/frame counts, combat totals/endpoints, execution and aggregation time, raw bundle bytes, compact transport bytes and compressed export bytes. Reuse existing harness/measurement helpers with a focused extension if required; scratch and receipts belong under `cache/`. Report measured differences, without promising zero or universal percentage overhead. The expected growth is bounded summary scalars/statistics, not data proportional to another detailed event stream.

## Risks / Trade-offs

- **SHP weights a stopped interval incorrectly** -> Test a shield existing only during a pause, mutation inside a pause, release, overlapping pauses, expiry, active switches and zero duration at the existing native shield boundary.
- **Absolute view conceals unavailable canonical DPS** -> Keep canonical flags and comparison eligibility independent; validate both basis-specific values and the original failure reasons across import/resume.
- **Independent panels retain mixed selection or reset during updates** -> One run-scoped state owner; test navigation, incremental delivery, selected histograms/contributions and current-view PNG capture.
- **Statistics and serialized data grow** -> Pair only varying values, reuse source identities and the single timeline, and compare bounded before/after receipts. Additional statistics require work; they are not a free rename.
- **Current contracts have many consumers** -> Update them atomically and run existing native, WASM/Worker, replay and report restoration checks; reject superseded shapes rather than adding shims.

## Migration Plan

Implement and validate the paired native/report/Studio contract as one coherent change, with documentation updated before completion. Existing same-clock but obsolete single-basis result bundles require regeneration; missing shield weighting cannot be reconstructed from aggregate scalars. Keep the current gameplay clock and timer-policy identities because their mechanics are unchanged, while updating any relevant report-format identity and strict field validation. No data migration or compatibility adapter is added. A rollback restores the coherent previous source/contracts together, not selected consumers. Archive only after checks pass and synchronize the three capability deltas then. Publication and deployment are outside this change.
