# Proposal

## Why

Results currently combines relative whole-run DPS with absolute EPS, HPS and SHP, while its time selector changes only duration. A single report selection should let users inspect coherent relative or absolute summaries without rerunning combat or multiplying detailed report data.

## What Changes

- Move the existing Relative / Absolute selector from Sim Duration (Dur) into Damage Per Second (DPS), retaining Relative as the default and adding no explanatory captions or tooltips.
- Apply the selected basis to whole-run DPS, retained/granted EPS, gross HPS, time-averaged SHP, duration and their statistical distributions, including actor/source/target DPS contributions. Preserve the selection for the displayed run across navigation and incremental deliveries.
- Keep the one absolute damage/cumulative timeline, rename its panel to `Damage Timeline (absolute time)` and its rate option to `Damage per second`. Keep Sample playback, field-time accounting, raw totals and canonical relative comparison results on their existing definitions.
- Compute relative SHP by weighting shield capacity only over advancing relative time, alongside the existing absolute integral. Derive both sets of rate statistics from each trial's existing settled totals before aggregation.
- **BREAKING**: Update current Rust, native report, WASM, Worker, TypeScript, resume and portable-report contracts together for compact dual-basis summary values. Do not duplicate event streams, detailed samples, static contribution identities or damage timelines, and do not add compatibility shims.
- Update accepted capability references and the maintained clock/report documentation to describe the selector, both formulas, availability rules and the absolute timeline exception.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: Report-wide basis selection from the DPS card, basis-aware support summaries and distributions, absolute timeline wording and portable dual-basis data.
- `combat-execution`: Compact native observations for both summary bases and correctly weighted relative SHP, while retaining canonical relative DPS for exact analysis and comparisons.
- `wasm-transport`: Agreement, validation, replay and resume for both native summary bases without TypeScript domain calculations.

## Impact

The only new combat-execution calculation is the second shield-capacity integral in `crates/content/src/simulation/runtime/time.rs`, finalized through the native report. `crates/research/src/trials/studio/` must calculate and aggregate both bases; current wire types, Worker checks, storage validation, result cards/charts and report exports must consume them consistently. Existing native shield/statistics tests and Worker/report/UI boundaries will be extended. No new dependencies or second simulation path are needed. Bounded before/after size and timing receipts will check the additional scalar/statistics cost; a universal overhead percentage is not assumed.
