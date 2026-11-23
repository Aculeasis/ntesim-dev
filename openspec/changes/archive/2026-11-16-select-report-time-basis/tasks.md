# Tasks

## 1. Retain the bounded performance baseline

- [x] 1.1 Before changing implementation, capture the current no-Ultimate, ordinary-Ultimate and continuing-Hotori workloads using existing acceptance measurement helpers under the four-CPU runner. Retain source/engine identities, trial counts, event/hit/frame counts, elapsed endpoints, execution/aggregation timings and raw/compact/compressed result sizes in `cache/`; verify the paired post-change measurement can reuse the same inputs without recalculating Team list rankings.

## 2. Native shield weighting and summary statistics

- [x] 2.1 Add the relative shield-capacity integral beside the existing absolute accumulator and finalize both native SHP averages, with zero-denominator availability. Extend the existing shield accounting/oracle boundary for pause-only capacity, capacity mutation during a pause, overlapping pauses, release, expiry, switches and early/zero-relative completion; verify sample and lean batch agree without added events or frames.
- [x] 2.2 Update `crates/research/src/trials/studio/` models, observation and aggregation to pair only DPS, retained/granted EPS, HPS and SHP values/statistics and contribution DPS. Extend existing Rust tests for independent numeric expectations, unequal seed durations, all descriptive statistics, no-pause equality, undefined/mixed seeds and chunk/resume equivalence; verify canonical relative exact/comparison DPS and unsupported-result exclusions remain intact using focused crate checks through `tools/cargo.py`.
- [x] 2.3 Update `openspec/specs/combat-execution/reference.md` and `docs/mechanics/shared/clocks-and-action-timing.md` with both rate/SHP formulas and canonical comparison semantics; verify the explanations distinguish weighting from denominator scaling and retain the selected pause-model assumptions.

## 3. Current transport and portable-report contracts

- [x] 3.1 Update native consumers, WASM/Worker types, retained-prefix and selected-sample verification, TypeScript contracts and storage validators atomically for the paired fields. Extend existing native/WASM and Worker checks to verify both summary bases, basis-specific availability and rejection of altered values or superseded shapes; verify no fallback shim or duplicate report/timeline is introduced.
- [x] 3.2 Extend existing report round-trip and resume coverage for JSON, compressed reports and PNG metadata, including partial status and zero-relative results; verify both bases restore without execution, comparison eligibility remains canonical, and native retained-prefix verification rejects changed SHP/rate values.
- [x] 3.3 Update `openspec/specs/wasm-transport/reference.md` with the paired current contract, validation and resume rules; verify documented payload shapes agree with the delivered native and TypeScript types.

## 4. Results selection, charts and documentation

- [x] 4.1 Move the single Relative / Absolute selector into Damage Per Second (DPS), hoist selection to run-scoped presentation state and apply it to Dur, DPS, EPS, HPS, SHP, applicable histograms and actor/source/target DPS contributions. Extend existing report controls/UI coverage to verify default Relative, run reset, stop/resume, incremental delivery, navigation persistence and selection without engine requests, while adding no visible explanatory caption or tooltip.
- [x] 4.2 Rename the timeline panel to `Damage Timeline (absolute time)` and its rate option to `Damage per second`, retaining `Cumulative Damage` and the sole absolute series. Extend the existing chart/UI boundary to verify both options, tail damage and absolute endpoints stay unchanged when the DPS selector changes; verify current-selection PNG capture and unchanged Sample stepping/readouts at desktop and 390-pixel width.
- [x] 4.3 Update `openspec/specs/studio-application/reference.md` and `docs/NTE_DSL.md` to replace duration-only/permanently-absolute-support explanations with the chosen scope, defaults, units, availability and export behavior. Verify wording matches the UI and search maintained documentation for conflicting current statements, preserving historical receipts and unrelated mechanics references.

## 5. Integrated acceptance

- [x] 5.1 Repeat the retained performance workloads with identical seeds/trial counts and compare combat totals, clock endpoints, event/hit/frame counts, execution/aggregation timing and raw/compact/compressed bytes. Verify growth is limited to paired summary scalars/statistics, with no additional detailed stream or timeline; record the measured scope and material limits in `docs/BROWSER_PERFORMANCE.md` and investigate unexplained growth before completion.
- [x] 5.2 Run affected existing native checks, `node tools/web.mjs check`, `node tools/web.mjs test` and the required WASM/Worker/browser acceptance boundaries one validator at a time. Review the stable diff, run `python tools/openspec.py validate select-report-time-basis --strict`, then `python tools/checkpoint.py run`; verify affected checks pass and report actual performance/size limits before delivery.

## Workflow follow-up

- Archive the completed change after its tasks and applicable checks pass, synchronizing all three capability deltas and verifying the archived documentation. Do not publish, deploy or refresh curated rankings as part of this change.
