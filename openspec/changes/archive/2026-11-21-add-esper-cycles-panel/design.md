# Design

## Context

The Studio simulation report overview currently displays two panels in a two-column grid: `Actions` (a stacked bar chart of actor ability casts) and `Field Time Distribution` (a donut/pie chart of actor field time).

In the Rust engine, batch simulations (`run_batch` where `record_detail = false`) deliberately prune non-essential trace receipts at every step to optimize throughput and memory usage. Therefore, reaction tracking cannot rely on querying `report.trace` post-execution; it must be tracked directly with lightweight counters in the simulation state.

See `proposal.md` for motivation and scope boundaries.

## Goals / Non-Goals

**Goals:**
- Provide zero-overhead reaction tracking during combat simulation via $O(1)$ integer increments in `ReactionRuntime`.
- Accurately determine available reactions based on the distinct elements present in the party (up to 4 reactions maximum out of the 8 game reactions).
- Capture reaction occurrences from both qualified switch admissions and character kit passives/refreshes (e.g. Baicang B2 Scorch refresh on Word, Blackbird A6 Nova reapplication).
- Compute empirical distribution statistics (`mean`, `min`, `max`, `std`, and quantiles) across all completed simulation seeds.
- Present available reactions in descending order of mean triggers in the `Esper Cycles` panel.
- Retain available reactions with 0 triggers visible with a 0 value.
- Dynamically switch between `.three-column` (when reactions are available) and `.two-column` (when 0 reactions are available, completely hiding `Esper Cycles`).

**Non-Goals:**
- Splitting the primary chart bar into sub-series (e.g., separating switch counts from passive counts). The user explicitly requested a unified single counter per reaction.
- Redesigning combat reaction formulas or ICD logic.

## Decisions

### 1. Zero-Allocation In-Engine Reaction Counters

We define an enum or constant indexing for the 8 reactions in `ntesim-content`:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReactionKind {
    Blossom = 0,
    Hexed = 1,
    Scorch = 2,
    Nova = 3,
    Stain = 4,
    Remora = 5,
    Charge = 6,
    Discord = 7,
}
```
`ReactionRuntime` maintains a fixed-size array `reaction_counts: [u32; 8]`.
Whenever a reaction is triggered:
- Switch admission: Blossom, Hexed, Scorch, Nova, Stain, Remora increment their respective index.
- Passive refresh/reapplication: Baicang B2 Scorch refresh increments `Scorch`; Blackbird A6 Nova reapplication increments `Nova`.
- Ternary events: `apply_charge` increments `Charge`; `try_discord_pair` increments `Discord`.

In `GameScenarioReport`, we expose `pub reaction_counts: BTreeMap<String, u32>` (or a fixed vector/map), populated from `reaction_counts`.

*Alternative considered:* Scanning `report.trace` in `observe.rs`. Rejected because `report.trace` is truncated in batch mode to preserve simulation speed.

### 2. Available Reaction Discovery & Aggregation in Rust

In `crates/research/src/trials/studio/`:
- `observe.rs`: `row.reactions` captures the per-trial counts.
- `Collector`: Inspects `request.actors` to determine the set of available reactions:
  - Binary pairs (Cosmos+Nature -> Blossom, Nature+Incantation -> Hexed, Incantation+Chaos -> Scorch, Chaos+Psyche -> Nova, Psyche+Lakshana -> Stain, Lakshana+Cosmos -> Remora).
  - Ternary trios (Lakshana+Cosmos+Nature -> Charge, Incantation+Chaos+Psyche -> Discord).
- For each available reaction, `Collector` computes `Statistics` across all trial observations. If a reaction never triggered, it summarizes a vector of zeros (`mean: 0, min: 0, max: 0, std: 0`).
- The resulting list is sorted in descending order of `mean` (with alphabetical/wheel tie-breaker) and stored in `Aggregate.reactions: Vec<ReactionAggregate>`:
```rust
#[derive(Debug, Serialize)]
pub struct ReactionAggregate {
    pub name: String,
    #[serde(flatten)]
    pub stats: Statistics,
}
```

If the party has no available reactions, `Aggregate.reactions` is empty.

### 3. UI Presentation and Adaptive Layout

In `web/src/features/results/EnergyPanels.tsx`:
- Inspect `a.reactions`:
  - If `a.reactions.length === 0`: Render `<div className="two-column">` with `Actions` and `Field Time Distribution`.
  - If `a.reactions.length > 0`: Render `<div className="three-column">` with `Actions`, `Field Time Distribution`, and `Esper Cycles`.
- The `Esper Cycles` panel renders `<BarChart rows={rows} unit="reactions" />`.
- Each reaction row binds:
  - `name`: Reaction name (e.g., "Remora", "Blossom", "Nova", "Discord").
  - `value`: `r.mean`.
  - `min`: `r.min`, `max`: `r.max`, `std`: `r.std`.
  - `color`: Distinctive theme color per reaction.
- CSS: Ensure `.three-column` has appropriate responsive behavior across screen sizes (collapsing to single column on mobile, identical to `.two-column`).

## Risks / Trade-offs

- **[Risk] Simulation throughput degradation** → **Mitigation**: Counters use a fixed `[u32; 8]` array in memory; incrementing is a single CPU instruction without allocations or hash lookups during combat steps.
- **[Risk] Responsive overflow with 3 columns on tablet/laptop screens** → **Mitigation**: Grid uses `repeat(3, minmax(0, 1fr))` on desktop and collapses to `grid-template-columns: minmax(0, 1fr)` at `<= 900px` screen widths.
