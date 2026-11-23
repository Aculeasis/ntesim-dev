# Design

## Context

`DamagePanels` builds chart rows from `aggregate.sources`. Each source identifies an owner slot, ability and effect. The native aggregate computes mean damage instances and both mean DPS bases over the same completed seeds, treating an absent effect as zero for that seed. The source chart already handles unavailable DPS, owner colors, ranking and folding. See [the proposal](proposal.md) for the scope.

## Goals / Non-Goals

**Goals**

- Derive the compact view from the delivered aggregate and render it through the existing bar chart.
- Keep mode selection scoped to the displayed run and independent of the source Type, Character, and DPS basis controls.

**Non-Goals**

- Simulator, native report, WASM, Worker protocol, and serialized report changes.
- New per-ability distributions or export fields for presentation state.

## Decisions

- Group the filtered source aggregates in a single pass keyed by owner slot and `ability_id`. Add the unrounded means for the selected metric, including the selected time basis for DPS, then sort and fold the grouped rows. The mean of per-seed totals equals the sum of the effect means, including runs where seed durations vary. This work scales with the number of source aggregates and grouped rows. Keep the current unavailable DPS branch ahead of row construction.
- Resolve compact labels using `resolveAbilityTitle` and add `Reactions` for the shared `ntesim.reaction` ID. Preserve owner color and owner inspection details. The existing `BarChart` row supports a mean without interval fields, so compact rows can omit those fields and the effect suffix. The existing detailed labels and distribution fields serve `full`.
- Add a title-adjacent control slot to `Panel` and share the small button styling with the DPS basis button. Keep the heading a heading and the toggle a separate button in the same visual group. Keep Type and Character in their existing controls area and allow that area to wrap at narrow widths. Follow `ResultsPage`'s run-scoped time-basis state pattern for the source mode so new or restored runs start in `full` and later chunks preserve the current selection.

## Risks / Trade-offs

- Effect minima, maxima and standard deviations do not describe the distribution of an ability total. Compact rows carry the mean only. The detailed chart retains access to each effect's full delivered statistics.
- Names can match across owners or ability IDs. Group by report identity and localize labels only for display. Grouping before folding ensures effects outside the detailed chart's visible slice contribute to the total.
