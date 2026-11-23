# Proposal

## Why

When viewing simulation results in Studio, the top control bar continues displaying a `Run` button instead of an export action, and sits on its own row creating a large dead empty area between the header intro and the simulation progress/results. Furthermore, an entire redundant toolbar row below the party cards duplicates iteration counts and export/config/sample buttons that are already directly accessible from top navigation and controls.

## What Changes

- When the `Results` tab is active, replace the `▷ Run` button in the top action bar with a standard `Export report` button.
- Integrate the view title, subtitle intro, and action controls into a shared header layout row to eliminate the empty vertical gap above the simulation progress and report.
- Remove the redundant results toolbar row (`Completed · 1000 / 1000 completed`, `Export report`, `Edit config`, `View sample`) below the party cards in `ResultsPage`.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `studio-application`: Refine the results view controls and header layout to replace the Run button with Export report on Results, eliminate the empty header gap, and remove the redundant toolbar below party cards.

## Impact

- `web/src/app/App.tsx`: Replace Run with Export report when `view === 'results'`; wrap heading/intro and `editor-actions` in `.view-header`.
- `web/src/features/results/ResultsPage.tsx`: Remove the redundant `results-toolbar` row and unused navigation button handlers.
- `web/src/styles/`: Update layout styling for `.view-header` and `.editor-actions` to sit inline with view heading/intro in static views.
- `openspec/specs/studio-application/`: Update spec and reference to reflect the updated layout and export control location.
