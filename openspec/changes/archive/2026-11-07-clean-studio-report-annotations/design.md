# Design

## Architecture

This change streamlines the visual presentation of Studio simulation results by removing static, verbose text blocks, redundant per-chart axis notes, and footer pointers across the report panels. The core data models, simulation calculations, native statistics, interactive inspectors, and export capabilities remain fully intact.

## Implementation Details

1. **Top Summary Cards (`web/src/features/results/Summary.tsx`)**:
   - Omit the `note` prop from the HPS card (removing `"Includes overheal · applied ... · excess ... · gross ... HP"`).
   - Omit the `note` prop from the SHP card (removing `"Time-averaged available shield capacity on active actor · ... HP mean absorbed damage"`).
   - Remove `<div id="execution-summary">` below the rollup cards (removing `"... mean total damage · ... mean hits · ... mean processed events · ... ms"`).

2. **Distribution Card (`web/src/components/charts/Histogram.tsx` and `web/src/features/results/ResultsPage.tsx`)**:
   - In `Histogram.tsx`, remove `<details className="distribution-explanation">` (removing `"How to read this distribution"` and its explanatory paragraph).
   - In `ResultsPage.tsx`, remove `<p className="chart-note" id="distribution-note">` (removing `"... completed trials"`).

3. **Time Charts (`web/src/components/charts/TimeChart.tsx`)**:
   - Remove `<p className="chart-note">` containing timeline description and `"hover or focus points for statistics"`.

4. **Damage Panels (`web/src/features/results/DamagePanels.tsx`)**:
   - Remove `"Grouped by character element."` from DPS by Character Element panel.
   - Remove `"Colors group characters by their element, not individual damage packets."` from Character DPS panel.
   - Remove `"Includes settled skill, equipment, Esper Cycle and summon damage. Colors identify the damage owner."` from Source DPS panel.

5. **Bar Charts and Stacked Bars (`web/src/components/charts/BarChart.tsx`, `web/src/components/charts/StackedBars.tsx`)**:
   - Remove `<p className="chart-note chart-unit">{unit} · mean per completed trial</p>` from both components. This removes redundant axis notes from Actions, Character DPS, Source DPS, and Ending Energy.

6. **Energy Panels and Results Page (`web/src/features/results/EnergyPanels.tsx`, `web/src/features/results/ResultsPage.tsx`)**:
   - In `EnergyPanels.tsx`, remove `<p className="chart-note">` under Energy Per Source (`"Gross = retained + overflow. Starting energy is not generation..."`).
   - In `EnergyPanels.tsx`, remove `<details className="resource-details">` containing the final resource ledger from Character Energy at End of Simulation.
   - In `ResultsPage.tsx`, remove `<p className="chart-note" id="sample-pointer">` (`"View individual healing, shield and resource events in Sample..."`).

7. **Tests and References**:
   - Update `web/tests/report_controls.test.mjs` assertions to ensure clean rendering without the removed note phrases.
   - Update `openspec/specs/studio-application/reference.md` to keep reference text coherent with the streamlined UI.
