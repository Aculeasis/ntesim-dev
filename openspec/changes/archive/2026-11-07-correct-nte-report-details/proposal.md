# Proposal

## Why

The report hides element-specific target resistances, leaves a large blank area below target information, omits Source DPS ownership legends and assigns incorrect element colors. Its summaries also omit healing and shields even though the engine records these NTE mechanics.

## What Changes

- Show a compact target header and six element-icon resistance entries, resolving explicit overrides over base resistance; keep the target panel aligned with its histogram neighbor.
- Correct all six element colors to match the supplied NTE reference and existing artwork; keep character ownership colors separate and identify them in Source DPS.
- Give native Esper Cycle damage sources readable names (including Scorch DoT and Blossom), preserving owner attribution and summon/other source identities.
- Extend Rust-owned completed-trial statistics with effective HPS and actual shield absorption per second, with gross/effective/excess healing and absorbed totals available in the report.
- Replace hit/event summary cards with healing/shield summaries; preserve hit/event counts and total damage in the execution summary and all original filters/exports/status boundaries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: target resistance presentation, element and source legends, native healing/shield statistics.

## Impact

Rust Studio trial collection, native/WASM/Worker trial payloads, TypeScript contracts, result components, chart colors and focused native/UI/browser checks. Combat mechanics and source execution semantics remain unchanged. Support rates use actual elapsed time and completed trials only; shield absorption is not generated shield capacity.
