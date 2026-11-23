# Native, WASM and Studio Worker calibration

Browser calibration checks agreement with the current shared Rust model. It does not
establish native-game accuracy, replace the Rust mechanics checks, or complete
browser lifecycle/performance and release acceptance.

## Executable boundary

[Source calibration](../web/tests/source_calibration.test.mjs) executes all
23 legal A0 [continuing baselines](NATIVE_BASELINES.md) with two contiguous seeds.
Only iteration/seed options change; builds, targets, rotation text and model
assumptions remain authored source. Native `compile-script` and `batch-script`
are the independent executable/target callers. Direct WASM and the actual
Studio Worker compare their complete compiled programs and source executions:
requests, receipts, ordered source occurrences, locations, state timelines,
quality, diagnostics, resources, RNG and input/engine identities.

The Worker executes its normal `SIMULATE_NTE` trial session and selected-seed
`SOURCE_SAMPLE` path. Every aggregate field and sample is compared with the
shared Rust collector applied to native CLI reports. Only measured wall time
and derived throughput differ by host. An early-victory source checks actual
elapsed-time normalization at this same boundary. Replay consumes each complete
source report and reaches its recorded final hit prefix.

Native/WASM/Worker report exports compare full losslessly decoded JSON and every
named CSV table. Worker-generated exact JSON replays through native CLI, and the
shared verification response retains identical identities, quality/completion
and exact damage analysis. Nonzero paired source comparisons use the CLI's
shared seeds and combined budgets. Admission failures retain identical partial
executions, remain excluded from aggregates/comparison means, and export/replay
without becoming successful runs. Unicode/CRLF syntax diagnostics preserve
typed native locations and their visible Worker error. Invalid hit references,
cyclic autonomous parents and missing enabling casts are rejected by replay.

Existing equipment/source/Worker tests retain the weapon refinements, progression,
sets, reactions, Support, transport and rejection boundaries. The complete
[checkpoint](DEVELOPMENT.md#choosing-checks) builds current native and processed
WASM artifacts, prepares parity inputs and runs the native/WASM battery, actual
Worker tests and browser suite. Its receipt/cache keys bind the checked tree,
engine sources and consuming native/WASM artifact bytes; old browser output
does not certify a changed engine.

```sh
node tools/web.mjs test web/tests/source_calibration.test.mjs
python tools/checkpoint.py run --full
```

## Measurements and limits

`python tools/check.py -- node tools/web.mjs verify-trials --out target/verification/browser-trials.json`
retains the existing 1000-trial fixed/random Worker measurement. It now executes
live source per seed rather than the static scheduled-script adapter, retains
source/compiled/engine/WASM identities, and records actual Rust quality/readiness
instead of hard-coded historical status. Its per-seed direct WASM comparisons
are transport checks, not independent gameplay evidence or browser UI timing.
CPU/memory/retention/search and actual keyboard/mobile/offline acceptance remain
the separate F1-F3/H4-H6 work package.

Independent numerical and causal mechanics checks remain in Rust. Frozen
[tabular calibration](../openspec/specs/calibration/reference.md) retains source inputs,
applicability, tolerances and unresolved interpretations. No applicable captured
game rotation has been acquired. Exact agreement cannot remove a model
assumption, upgrade strict readiness, or turn seed variance into model uncertainty.
The runtime adapter continues to report `parity_verified: false`; only external
checks certify their particular artifacts.
