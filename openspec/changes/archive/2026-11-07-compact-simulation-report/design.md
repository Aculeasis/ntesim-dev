# Design

## Context

See proposal.md for motivation. ResultsPage already consumes Rust-produced TrialAggregate statistics through the Worker. DamagePanels places two fixed-aspect time charts side by side. EnergyPanels pairs a short field-time distribution with a tall per-recipient energy list. Multiple historical CSS layers enlarge report cards. Existing Studio rendering tests and browser acceptance cover controls, failure identity and exports.

The reference is [gcsim ResultsView](https://github.com/genshinsim/gcsim/blob/main/ui/packages/components/src/ResultsView/ResultsView.tsx), inspected in an ignored shallow checkout. Its ordered grid and [damage timeline](https://github.com/genshinsim/gcsim/blob/main/ui/packages/components/src/Cards/ResultCards/Damage/DamageTimelineCard/DamageOverTime.tsx) inform layout and statistical emphasis. Reimplement with existing React/SVG; do not copy source or add chart dependencies.

## Goals / Non-Goals

**Goals:** Improve statistical readability with compact report-scoped styles, wide time plots, pie distributions, consistent colors and dense ranked bars. Preserve all current report controls and access to team builds, samples, comparisons, exports and run details.

**Non-Goals:** Change Rust mechanics, native output, WASM or Worker contracts; infer actual damage elements from character element; aggregate new reaction/aura/healing/shield metrics; introduce positions for stationary anonymous dummies.

## Decisions

- Compose report sections in the reference order using existing panels. Move actions into the two-column row with field time; expand energy sources to full width. Scope compact styles to a report wrapper and load them after current styles so editor/playback stay independent.
- Keep global Run/Settings/Tools controls in document flow outside the editor so they cannot cover report charts or sample controls.
- Use responsive SVG time plots with a wide desktop view box and a narrow mobile view box, retaining physical time in seconds. Show mean/min/max and a clamped mean ± standard-deviation band for damage; show min/max and p25/p50/p75 for cumulative damage. Use existing statistics, no smoothing or fabricated samples. Include a zero origin only for cumulative damage, never invent a damage ramp before the first bin.
- Replace report donuts with pie distributions and explicit legends; keep focusable segments and readable zero-data behavior. Group damage by character element only as explicitly labeled; current native aggregates do not supply per-packet element totals.
- Rank character and source bars by selected values, keep complete source identities and filtering, and use actor slots for chart colors. Group energy rows by native source identity and kind, showing native mean grants as stacked recipient segments. Sum means only; do not manufacture uncertainty bounds for grouped sources. Keep gross/retained/overflow totals and final per-seed resource ledger clearly labeled.
- Preserve current six summaries (DPS, retained EPS, damage, hit count, processed events, duration) rather than borrowing unavailable Genshin rollups. Keep partial/failed status and completed/requested counts visible above them.

## Risks / Trade-offs

- Wide SVGs can shrink labels → use a resize observer to adjust plot coordinates and tick counts; validate desktop and 390 pixel screenshots.
- Dense bars can hide long source names → wrap names on narrow screens and retain full names in titles and keyboard inspection.
- Recipient stacks hide individual energy uncertainty → expose selected per-recipient means and retain gross/retained/overflow totals; never label a sum of independent bounds as a native statistic.
- Pie slices cannot represent zero values → show an explicit empty-data message and retain accessible labels for nonzero observations.

## Migration Plan

This is a local UI change with unchanged serialized contracts. Validate existing focused chart/rendering boundaries, inspect actual browser screenshots and controls, then run checkpoint. Synchronize the delta and archive after checks; commit the coherent change. Rollback is a normal commit revert.
