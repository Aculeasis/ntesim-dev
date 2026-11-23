# Proposal

## Why

NTE simulation scripts currently require repetitive boilerplate (`mode=research`, explicit `crit=random`), lack flexible stopping rules when targets are defeated before the time horizon, and are restricted by restrictive 600s and 10k event execution limits. Furthermore, the Studio UI presents clutter on the Sample playback tab, lacks an immediate events count in the Results summary, lacks backdrop-click dismissal for modal dialogs, opens redundant intermediate modals when exporting configs or reports instead of directly saving them with expressive names, displays excessively long Source DPS charts without pagination or folding, and logs styling de-optimization warnings during Vite dev server operation.

Streamlining these option defaults, expanding execution safety horizons, and polishing the Studio GUI creates a more productive authoring, export, and analysis workflow.

## What Changes

- **BREAKING**: Change the default critical strike policy from fixed/expected to stochastic (`crit=random`). Scenarios requiring deterministic, non-random calculations must explicitly declare `crit=fixed`.
- Remove redundant `mode=research` from all scenario presets, examples, and documentation while maintaining parser and contract validation support for `mode=strict`.
- Update `stop=` policy behavior:
  - `stop=time`: Simulation completes successfully when the time horizon expires OR when all targets are defeated (target defeat is treated as normal completion rather than an error or continuing into dead targets).
  - `stop=victory`: Ignores authored `duration` and executes until all targets are defeated (or until execution limits/overflow are reached).
- Raise default execution safety limits:
  - Default maximum horizon (`max_horizon_us`) increased from 600 seconds (600,000,000 $\mu\text{s}$) to 3600 seconds (3,600,000,000 $\mu\text{s}$).
  - Default maximum event count (`max_events`) increased from 10,000 to 100,000 events.
- Direct file exports with expressive filenames:
  - `Tools -> Export script` immediately initiates file download without an intermediate modal dialog, naming the file with a timestamp in `YYYY-MM-DD_HH-mm-ss` format and human-readable character names (e.g. `YYYY-MM-DD_HH-mm-ss-zankou-iroi.nte`).
  - `Export report` on Results view immediately initiates file download without an intermediate modal dialog, naming the file with a timestamp in `YYYY-MM-DD_HH-mm-ss` format (e.g. `YYYY-MM-DD_HH-mm-ss-report.json`).
- Source DPS chart truncation and folding:
  - When the Source DPS chart has 20 or more entries, it renders the first 15 entries by default.
  - A subtle full-width clickable footer strip preserving the card panel background with a centered chevron toggle (down chevron to expand, up chevron to collapse) allows expanding to show all entries, highlighting on hover.
- Option editing and stop policy settlement:
  - Switching stop policy to `stop=victory` in Settings preserves authored duration in script options while allowing the simulation runtime to run until target defeat.
- Studio GUI enhancements:
  - Results view metadata row displays the event count (`events <count>`) alongside iterations, duration, seed, and targets.
  - Sample playback view removes the redundant `#sample-note` header ("Seed 42 · 514 hits · 1751 events") and raw inline `.replay-readouts` ("damage0hits0events...").
  - All modal dialogs (using `Dialog.tsx`) now dismiss when clicking on the backdrop outside the dialog box.
- Tooling optimization:
  - Configure Vite React plugin and Babel filters to exclude pre-bundled dependency caches (`cache/vite/deps`), eliminating the 500KB de-optimisation warning during `node tools/web.mjs dev`.

## Capabilities

### Modified Capabilities
- `script-execution`: Default critical policy defaults to `crit=random`; `stop=time` halts on time or target defeat without error; `stop=victory` executes until all targets are defeated without requiring a bounding duration.
- `execution-safety`: Maximum simulation horizon default increases to 3600s; maximum event count default increases to 100,000.
- `studio-application`: Result summary includes event count; Sample view omits redundant readout lines; modal dialogs dismiss on backdrop click; config and report exports initiate immediate download with expressive timestamped names; Source DPS charts with $\ge 20$ entries fold to 15 with an interactive expand strip; dev server excludes pre-bundled caches from Babel transformations.

## Impact

- Rust engine (`crates/core`, `crates/content`, `crates/research`, `crates/wasm`):
  - `ExecutionLimits::default()` reflects 3600s and 100k events.
  - `CompiledOptions` defaults `crit` to `CriticalPolicy::Random`.
  - Stopping and execution runtime handles target death as a successful stopping condition under `stop=time`, and unbounded horizon under `stop=victory`.
- Scenarios and presets (`presets/studio/`, `examples/`):
  - Removed `mode=research` and `crit=random` boilerplate.
  - Added `crit=fixed` to deterministic tests expecting fixed values.
- Studio web app (`web/src/`):
  - `Summary.tsx` displays event count.
  - `SamplePage.tsx` and `ReplayStage.tsx` omit redundant playback readout text.
  - `Dialog.tsx` handles outside clicks.
  - `App.tsx` and `ToolsDialog.tsx` trigger direct downloads with expressive filenames.
  - `DamagePanels.tsx` folds Source DPS charts with $\ge 20$ items to 15 with a full-width expand toggle bar.
  - `web/vite.config.ts` updates plugin options.
