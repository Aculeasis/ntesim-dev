# Proposal

## Why

The simulation report currently displays redundant static text, duplicated mean axis notes, and verbose explanatory footers under metric cards, charts, and panels. These annotations add visual clutter without providing additional analytical insight beyond what the charts, axes, and interactive tooltips already convey.

## What Changes

- Remove verbose note captions from Healing Per Second (HPS) and Effective Shield HP (SHP) summary cards.
- Remove the redundant execution summary line under the top summary cards.
- Remove the expandable explanation and completed trials note from the Distribution card.
- Remove redundant static chart-note footers from Damage Timeline, Cumulative Damage, DPS by Character Element, Character DPS, and Source DPS panels.
- Remove duplicated `{unit} · mean per completed trial` axis footers from `BarChart` and `StackedBars`.
- Remove redundant accounting note from Energy Per Source, remove the final resource ledger from Character Energy at End of Simulation, and remove the footer sample pointer note.
- Update report tests and specification references to reflect the cleaned, streamlined UI presentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: streamlined statistical report presentation without redundant static card notes or footer captions.

## Impact

Results presentation components (`Summary.tsx`, `Histogram.tsx`, `TimeChart.tsx`, `DamagePanels.tsx`, `EnergyPanels.tsx`, `ResultsPage.tsx`, `BarChart.tsx`, `StackedBars.tsx`), test expectations (`report_controls.test.mjs`), and detailed reference documentation. Native Rust engine, simulation contracts, and Worker aggregation are unchanged.
