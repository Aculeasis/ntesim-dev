# Proposal

## Why

In Studio, damage breakdown charts (Source DPS and Source Damage Instances) display many duplicate rows with identical labels (for example, up to 10 rows titled "Wildfire", 6 rows titled "Sanguine Dash", and 5 rows titled "Inferno Flamenco" for Zankou). While a few characters like Oneiroi and Lacrimosa have curated effect labels, most skills and their damage effects (including combo steps, charged branches, stance attacks, and character DoTs like Heartwrench or Vile Ash) lack distinct human-readable labels, causing confusing duplicate bars in simulation reports.

## What Changes

- Add distinct, human-readable labels for character skill damage effects, combo sequences, stance hits, and DoTs in Studio report source labeling.
- Curate explicit known labels for major multi-hit abilities, DoTs, awakenings, weapon procs, and special effects across characters.
- Introduce a clean, consistent humanization fallback for all remaining uncurated effect IDs (stripping internal prefixes/suffixes and formatting hit/branch numbers) so no multi-packet ability ever displays duplicate indistinguishable rows.
- Ensure secondary actions without top-level ability titles (such as Air Attacks and Dodge Counters) render clean names instead of raw internal engine IDs (`GA_...`).

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `studio-application`: Require that damage breakdown charts present distinct, human-readable labels for individual damage effects, multi-hit components, and DoTs without redundant duplicate ability titles.

## Impact

- `web/src/domain/reports/source-label.ts` and associated localization catalogs or effect maps.
- Report tests in `web/tests/report_controls.test.mjs` and related test assertions.
- No engine simulation domain logic, mechanics math, or native contracts are modified; changes are strictly in web presentation.
