# Tasks

## 1. Script Options and Stop Policies

- [x] 1.1 Update `crates/content/src/dsl/options.rs` so `crit` defaults to `CriticalPolicy::Random`, keeping `crit=fixed` for explicit deterministic calculations; verify with `python tools/check.py -- cargo test -p ntesim-content -- options`.
- [x] 1.2 Implement stopping semantics in `crates/content/src/dsl/options.rs` and `crates/content/src/simulation/runtime/`: `stop=time` terminates with success when all targets are defeated or time expires; `stop=victory` ignores authored duration and defaults horizon to maximum; verify with targeted runtime stop tests.
- [x] 1.3 Remove redundant `mode=research` and `crit=random` across presets in `presets/studio/`, `examples/`, and documentation; add `crit=fixed` to test fixtures requiring exact deterministic values.
- [x] 1.4 Run focused test validation across affected crates (`crates/content`, `crates/research`, `crates/cli`, `crates/wasm`) with `python tools/check.py -- cargo test`.

## 2. Execution Safety Limits

- [x] 2.1 Update `crates/core/src/execution/limits.rs` to set default `max_horizon_us` to 3,600,000,000 $\mu\text{s}$ (3600 seconds) and default `max_events` to 100,000; update `openspec/specs/execution-safety/reference.md`.
- [x] 2.2 Verify execution limits preflight and enforcement with `python tools/check.py -- cargo test -p ntesim-core -- limits`.

## 3. Studio GUI and Build Optimization

- [x] 3.1 In `web/src/features/results/Summary.tsx`, add `events` to the result metadata row, extracting event count from the first sample or execution aggregate.
- [x] 3.2 In `web/src/features/playback/SamplePage.tsx` and `web/src/features/playback/ReplayStage.tsx`, remove `#sample-note` and `.replay-readouts` to eliminate UI clutter.
- [x] 3.3 In `web/src/components/ui/Dialog.tsx`, add click-outside detection on the `<dialog>` element to dismiss modal dialogs when clicking on the backdrop.
- [x] 3.4 In `web/src/infrastructure/storage/files.ts` and `web/src/app/App.tsx`, implement direct file downloads for `Tools -> Export config` and `Export report` with timestamped expressive filenames (e.g. `YYYYMMDD-HHmmss-<characters>.nte` and `YYYYMMDD-HHmmss-report.json`).
- [x] 3.5 In `web/src/features/results/DamagePanels.tsx` and `web/src/styles/studio-bars.css`, add folding logic and a full-width hoverable chevron footer strip to truncate Source DPS charts $\ge 20$ items to 15 by default.
- [x] 3.6 In `web/vite.config.ts`, configure `@vitejs/plugin-react` to exclude `cache` and apply `compact: true`, preventing Babel de-optimisation warnings during dev server operation.
- [x] 3.7 Run web tests and bundle checks via `python tools/check.py -- node tools/web.mjs test`.

## 4. Verification and Checkpoint

- [x] 4.1 Run full project verification via `python tools/checkpoint.py run` to ensure all native, WASM, and web checks pass cleanly.

## 5. Studio Polish Follow-ups and Settings Fix

- [x] 5.1 Restyle `.chart-expand-bar` and `.chart-expand-pill` in `web/src/styles/studio-bars.css` with transparent background and positioned tab pill matching panel styling.
- [x] 5.2 Update timestamp format in `web/src/infrastructure/storage/files.ts` to `YYYY-MM-DD_HH-mm-ss`.
- [x] 5.3 Rename "Export config" to "Export script" in `web/src/features/import-export/ToolsDialog.tsx`.
- [x] 5.4 Map character identifiers to human-readable names via `english.character` in `generateConfigFilename` and support `character_XXXX` normalization in `web/src/domain/localization.ts`.
- [x] 5.5 Fix option settlement when switching stop policy to `victory`: preserve authored `horizon_us` when `duration_specified` is true in `crates/content/src/dsl/options.rs`, while ensuring runtime requests use maximum horizon; verify with `crates/content/src/dsl/editing_tests.rs`.
- [x] 5.6 Run format, web tests, and `python tools/checkpoint.py run`.

## 6. Stop Victory Duration Handling and Timeline Polish

- [x] 6.1 In `crates/content/src/simulation/runtime/source/observation.rs`, make `.duration` available under `stop=victory` even when duration is not explicitly declared, returning the execution horizon limit; add unit test in `source_targets.rs`.
- [x] 6.2 In `crates/research/src/trials/studio/mod.rs`, bound timeline bins to actual elapsed duration under `stop=victory` instead of padding with zero-DPS bins up to maximum horizon; verify with studio trials tests.
- [x] 6.3 In `web/src/features/results/DamagePanels.tsx`, remove the separate Cumulative Damage panel, rename the selector option to "Cumulative Damage", and compute effective horizon from timeline endpoint under `stop=victory`.
- [x] 6.4 Rebuild WASM and native artifacts, run tests and `python tools/checkpoint.py run`.
