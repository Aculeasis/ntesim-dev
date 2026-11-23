# Proposal

## Why

In the simulation results view, the `Target Information` and `Distribution` overview panels touch the bottom of the rollup metric cards due to missing vertical margin. Additionally, target cards display target names and stats on separate lines, causing large vertical height expansion when multiple targets exist, and the `Target Information` panel heading takes up unnecessary space when multiple targets are present.

## What Changes

- Add explicit vertical spacing between the rollup summary cards and the overview panels (`Target Information` and `Distribution`) to prevent visual overlap.
- Render target name and primary stats (Level, HP, DEF) on a single horizontal line within each target card.
- Omit the `Target Information` panel heading when three or more targets are present in the simulation scenario.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `studio-application`: Update target card layout and overview panel presentation rules to ensure compact single-line target headers, adaptive title omission for scenarios with 3+ targets, and proper vertical separation below rollup summary cards.

## Impact

- `web/src/features/results/Targets.tsx`: Update target card markup to place target name and stats on a single header line.
- `web/src/features/results/ResultsPage.tsx`: Conditionally omit panel title when `request.targets.length >= 3`.
- `web/src/components/ui/Panel.tsx`: Allow optional `title` and skip rendering `panel-heading` when both title and controls are omitted.
- `web/src/styles/`: Add bottom margin to `.rollup-grid` and single-line styles for `.target-header-line`.
- `openspec/specs/studio-application/`: Update spec delta for target overview layout.
