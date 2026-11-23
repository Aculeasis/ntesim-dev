# Proposal

## Why

Ten weapons present in the simulation script catalog (`thegoodthebadthebitter`, `tearsbeneaththemask`, `us`, `realmusic`, `dangerousgame`, `firststeptosuccess`, `farsideofthemoon`, `thefoolsspring`, `behappy`, and `drawnblade`) lack bundled artwork in Studio, causing the UI to render empty fallback placeholders instead of weapon icons. Providing bundled identity artwork for these weapons completes catalog coverage across team inspection and weapon selection views.

## What Changes

- Add 10 bundled WebP artwork assets to `web/public/assets/artwork/` corresponding to the missing weapons.
- Register all 10 weapons in `web/src/domain/artwork.ts` under `ARTWORK.weapons`.
- Verify 1-to-1 parity between `ARTWORK.weapons`, filesystem artwork assets, and `CATALOG.weapons` in studio tests and distribution builds.

## Capabilities

### Modified Capabilities
- `studio-application`: Ensure bundled weapon artwork covers all 51 catalog weapons and renders their icons in party and equipment views without fallback placeholders.

## Impact

- `web/public/assets/artwork/`: 10 new WebP images added.
- `web/src/domain/artwork.ts`: 10 new entries mapped in `ARTWORK.weapons`.
- `web/tests/studio.test.tsx`: Validates 51 weapon images and exact filesystem parity.
- `tools/browser_distribution.py`: Bundles the complete set of 51 weapon assets into distribution packages.
