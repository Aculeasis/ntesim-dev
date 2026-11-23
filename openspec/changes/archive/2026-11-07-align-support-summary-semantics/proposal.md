# Proposal

## Why

The support cards currently measure applied healing and shield absorption per second, so a fully healthy party fighting stationary dummies can show zeros despite producing healing and shields. The requested gcsim-style report instead needs healing including overheal and time-averaged available shield HP, adapted to NTE's native recipient and stacking rules.

## What Changes

- **BREAKING**: Define `hps` as gross healing, including excess healing, divided by actual completed-trial duration; retain gross, applied and excess healing totals.
- **BREAKING**: Replace `sps` with `shp`, the time average of remaining shield capacity protecting the active actor, in HP. Respect NTE's additive shield settlement, actor switches, replacement, depletion, defeat and exact expiry times.
- Retain actual absorbed shield damage as a separate total, excluding immunity and other interception; it does not determine SHP.
- Update the two cards, histogram choices, exports and native/WASM/Worker contracts together. Explain units and include the same per-trial statistics as the other cards.
- Use existing native healing receipts and recorded state boundaries; add no enemy attacks, geometry or Genshin elemental shield multipliers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: Correct native support summary definitions, available-shield accounting and presentation.

## Impact

Rust trial observation and aggregation under `crates/research/src/trials/studio/`; current TypeScript trial contracts, result cards and histogram consumers; existing native, WASM/Worker and report-control coverage; `openspec/specs/studio-application/reference.md` during implementation. Existing combat settlement and the source language stay unchanged. Old serialized `sps` contracts receive no compatibility shim.
