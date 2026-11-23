# Design

## Context

See `proposal.md` for motivation. Currently, `CompiledOptions` sets `crit: CriticalPolicy::Fixed` by default, forcing authors to supply `crit=random` on nearly every scenario. Many scenarios and presets additionally specify `mode=research`, which is redundant since `Mode::Research` is already the engine default.

During runtime, `stop=time` continues running rotations until the horizon even if all targets are defeated, leading to errors when abilities attempt to hit dead dummies. Conversely, `stop=victory` enforces the authored duration if present, failing with `VICTORY_NOT_REACHED` if targets survive that horizon.

Execution limits are bounded by `ExecutionLimits::default()`, capped at 600 seconds and 10,000 events. In Studio, the Result view lacks event count in its metadata strip, the Sample playback view renders repetitive seed and concatenated readout strings, and modal dialogs require explicit button or ESC triggers. Exporting config or reports pops up a modal textarea dialog with generic names (`nte-config.nte`, `nte-research-report.json`) requiring extra clicks. Furthermore, the Source DPS chart grows unbounded when many abilities or procs deal damage, cluttering the Results overview. During development, Vite's React plugin processes pre-bundled vendor files in `cache/vite/deps`, triggering Babel code generator size warnings.

## Goals / Non-Goals

**Goals:**
- Default `CompiledOptions.crit` to `CriticalPolicy::Random`, with `crit=fixed` available for deterministic tests.
- Strip redundant `mode=research` and `crit=random` from preset scripts, examples, and documentation.
- Implement unified stop semantics:
  - `stop=time`: Ends cleanly with success when duration expires OR when all targets are defeated.
  - `stop=victory`: Ignores authored duration and executes until all targets are defeated (or limits/overflow halt the run).
- Raise default execution limits to 3,600 seconds and 100,000 events in `ExecutionLimits`.
- Render `events <count>` in the Studio Results metadata strip.
- Remove `#sample-note` and `.replay-readouts` from the Studio Sample tab.
- Support outside-click (backdrop click) dismissal on `<dialog>` in `Dialog.tsx`.
- Direct file download for `Tools -> Export config` and `Export report` with timestamped and character-descriptive names.
- Truncate Source DPS charts to 15 items when items $\ge 20$, with an interactive full-width expandable footer bar matching the provided design.
- Configure `web/vite.config.ts` to exclude `cache` from Babel transformations and enforce compact code generation.

**Non-Goals:**
- Removing `mode=strict` from the parser or scenario contract validation.
- Adding backward-compatibility shims for obsolete options.
- Altering target HP models or damage formulas.

## Decisions

### Decision 1: Stochastic crit by default (`crit=random`)
- In `crates/content/src/dsl/options.rs`, initialize `crit: CriticalPolicy::Random`.
- In `apply_option`, keep `("crit", "fixed") => CriticalPolicy::Fixed` and `("crit", "random") => CriticalPolicy::Random`.
- In preset scripts, remove `crit=random`. In deterministic unit and integration tests that expect exact un-rolled damage, add `crit=fixed`.

### Decision 2: Stopping logic under `stop=time` and `stop=victory`
- In `crates/content/src/dsl/options.rs`, when `stop == StopPolicy::Victory`, set `horizon_us = ExecutionLimits::default().max_horizon_us` regardless of whether `duration` was specified.
- In `crates/content/src/simulation/runtime/source.rs` and `execution.rs`:
  - When all targets are defeated (`all_targets_defeated() == true`), the execution halts immediately as completed:
    - `execution.success = true`
    - `execution.execution_completed = true`
    - `execution.halt_diagnostic = None`
  - Under `stop=time`, this prevents the rotation loop from trying to issue actions against dead dummies.
  - If targets remain alive at the duration boundary, `stop=time` halts at the horizon as usual.

### Decision 3: Extended execution safety limits
- In `crates/core/src/execution/limits.rs`:
  - `default_max_horizon_us()` returns `3_600_000_000` $\mu\text{s}$ (3600s).
  - `default_max_events()` returns `100_000` events.
- Update reference documentation in `openspec/specs/execution-safety/reference.md`.

### Decision 4: Studio metadata and cleanup
- In `web/src/features/results/Summary.tsx`, read events from `result.samples[0]?.events ?? result.report?.total_events_processed ?? Math.round(a.metrics.events?.mean ?? 0)` and include `['events', events]` in the metadata list.
- In `web/src/features/playback/SamplePage.tsx`, remove `<p className="chart-note" id="sample-note">...`.
- In `web/src/features/playback/ReplayStage.tsx`, remove `<div className="replay-readouts">...`.

### Decision 5: Modal backdrop click dismissal
- In `web/src/components/ui/Dialog.tsx`, add an `onClick` handler on `<dialog>`:
  ```tsx
  onClick={(e) => {
    const rect = ref.current?.getBoundingClientRect();
    if (
      rect &&
      (e.clientX < rect.left ||
        e.clientX > rect.right ||
        e.clientY < rect.top ||
        e.clientY > rect.bottom)
    ) {
      onClose();
    }
  }}
  ```
  This reliably detects clicks on the modal backdrop (`::backdrop`) while ignoring clicks inside the dialog content.

### Decision 6: Vite dev server Babel filter
- In `web/vite.config.ts`, configure `react({ exclude: [/[\\/]node_modules[\\/]/, /[\\/]\cache[\\/]/], babel: { compact: true } })`.
- This ensures Babel skips Vite's pre-bundled dependencies in `cache/vite/deps`, eliminating the styling de-optimisation warning.

### Decision 7: Direct exports with expressive filenames
- Add utility functions in `web/src/infrastructure/storage/files.ts`:
  - `fileTimestamp()` generating `YYYYMMDD-HHmmss`.
  - `configFileName(party)` producing `<timestamp>-<char1>_<char2>.nte` or `<timestamp>-config.nte`.
  - `reportFileName()` producing `<timestamp>-report.json`.
- In `web/src/app/App.tsx`:
  - `ToolsDropdown`'s `onExport` invokes `downloadFile(configFileName(config.party), config.source, 'text/plain')`.
  - `exportReport` invokes `downloadFile(reportFileName(), serializeReportBundle(run), 'application/json')`.
- This eliminates the intermediate preview modal and directly initiates file saving.

### Decision 8: Source DPS list truncation and interactive footer strip
- In `web/src/features/results/DamagePanels.tsx`, maintain state `[sourceExpanded, setSourceExpanded] = useState(false)`.
- When source rows count $\ge 20$:
  - Render `sourceExpanded ? rows : rows.slice(0, 15)`.
  - Render a clickable `.chart-expand-bar` at the bottom of `#source-chart` with a centered chevron toggle button (`⌄` / `⌃`).
  - Style `.chart-expand-bar` in `web/src/styles/studio-bars.css` matching the design reference: full-width bar with centered pill tab, subtle border, cursor pointer, and hover highlight.

## Risks / Trade-offs

- [Risk]: Existing tests without explicit `crit=` expecting fixed numbers fail under random crits.
  - Mitigation: Run cargo tests across crates; add `crit=fixed` to deterministic test cases.
- [Risk]: Backwards incompatibility for scripts expecting `stop=victory` to fail if targets survive duration.
  - Mitigation: Documented change: `stop=victory` now explicitly ignores duration in favor of dummy defeat.
- [Risk]: Direct download bypassing text copy option.
  - Mitigation: Users wishing to copy text can select and copy directly from the code editor on the Simulator tab.
