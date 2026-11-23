# Tasks

## 1. Clean report annotations

- [x] 1.1 Remove verbose note props from HPS and SHP metric cards and remove `#execution-summary` in `Summary.tsx`.
- [x] 1.2 Remove `<details className="distribution-explanation">` in `Histogram.tsx` and `#distribution-note` in `ResultsPage.tsx`.
- [x] 1.3 Remove `<p className="chart-note">` in `TimeChart.tsx`.
- [x] 1.4 Remove chart notes in `DamagePanels.tsx` from element pie, character DPS, and source DPS.
- [x] 1.5 Remove `{unit} · mean per completed trial` axis footers from `BarChart.tsx` and `StackedBars.tsx`.
- [x] 1.6 Remove chart note and final resource ledger in `EnergyPanels.tsx`, and remove `#sample-pointer` in `ResultsPage.tsx`.

## 2. Verification and delivery

- [x] 2.1 Update `web/tests/report_controls.test.mjs` assertions and verify web tests pass.
- [x] 2.2 Synchronize specification reference, validate OpenSpec, run checkpoint, and archive change.
