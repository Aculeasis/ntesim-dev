# Proposal

## Why

Simulation results in Studio currently display character action counts and field time distributions, but do not indicate which Esper Cycle reactions are available to the deployed team composition or how frequently each reaction triggered across simulation trials. Adding an "Esper Cycles" statistics panel allows players and theorycrafters to immediately observe reaction availability, audit kit synergy (such as Scorch or Nova uptime), and quantify stochastic trigger distributions without opening raw execution traces.

## What Changes

- Add native per-trial reaction counters to the simulation runtime for all 8 game reactions:
  - 6 binary reactions (`Blossom`, `Hexed`, `Scorch`, `Nova`, `Stain`, `Remora`), counting both qualified switch admissions and character kit refreshes/reapplications (such as Baicang B2 Scorch refresh on Word and Blackbird A6 Nova reapplication).
  - 2 ternary reactions (`Charge` and `Discord`), counting Charge energy grant events from Blossom pistil hits on Remora targets and Discord break-gauge deductions on Scorch/Nova generation overlap.
- Extend `Observation` and `Aggregate` in `ntesim-research` to collect per-trial reaction counts and compute empirical distribution statistics (`mean`, `min`, `max`, `std`, and percentiles) for each reaction available to the party.
- If a reaction is available in the squad but was never triggered during the run, it is retained in the aggregate with 0 counts (`mean: 0, min: 0, max: 0`).
- If no reactions are available in the party (0 available reactions, e.g. mono-element squad or non-adjacent elements), the aggregate reactions collection is empty.
- Update Studio UI (`EnergyPanels.tsx`):
  - When available reactions exist, render a 3-column layout (`Actions`, `Field Time Distribution`, `Esper Cycles`), displaying an interval `BarChart` sorted in descending order of trigger mean, complete with whiskers and tooltips.
  - When no reactions are available in the party, completely hide the `Esper Cycles` panel and maintain the 2-column layout (`Actions`, `Field Time Distribution`).
- Audit and document character passives associated with reactions, scheduling discrete follow-up implementation tasks for any unmodeled or stub reaction passives identified in the codebase.

## Capabilities

### Modified Capabilities
- `studio-application`: Introduce requirements for the Studio report's Esper Cycles panel, reactive layout adaptation (3-column vs 2-column), and native reaction distribution metrics.

## Impact

- Rust engine (`crates/content`): `ReactionRuntime` / `GameSimulatorState` maintains 8 lightweight integer counters, updated with $O(1)$ operations during combat without trace bloat in batch simulations.
- Rust research & WASM (`crates/research`, `crates/wasm`): `Observation` and `Aggregate` serialize reaction statistics.
- TypeScript web app (`web/src`): `TrialResult`, `TrialAggregate`, `EnergyPanels.tsx`, and responsive styles handle 3-column layout and the Esper Cycles bar chart.
