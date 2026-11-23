# Design

## Context

Studio bundles visual artwork in `web/public/assets/artwork/` and maps them in `web/src/domain/artwork.ts`. `studio.test.tsx` strictly validates that every file in `web/public/assets/artwork/` has a corresponding entry in `ARTWORK` and valid WebP headers (`RIFF` and `WEBP`).

Forty-one weapons had bundled assets; ten weapons in `CATALOG.weapons` were previously unmapped, falling back to `<span className="asset-placeholder">`.

## Goals / Non-Goals

**Goals:**
- Provide bundled 100x100 WebP assets for all 10 missing weapons in `web/public/assets/artwork/`.
- Maintain strict 1-to-1 consistency between `ARTWORK.weapons`, catalog definitions, and disk files.
- Pass all unit tests in `studio.test.tsx` and bundle checks in `browser_distribution.py`.

**Non-Goals:**
- Modifying engine weapon combat logic, damage formulas, or stat definitions.
- Adding dynamic online image fetching in browser clients (assets remain bundled and offline-first).

## Decisions

- **Exact identity mapping**: Sourced original 100x100 WebP identity assets matching Reference guide's CDN IDs:
  - `weapon:fork_bittercake` -> `weapons_weaponforkbittercake.webp` (Reference guide Arc 19)
  - `weapon:fork_nestbird` -> `weapons_weaponforknestbird.webp` (Reference guide Arc 42)
  - `weapon:fork_prokaryon` -> `weapons_weaponforkprokaryon.webp` (Reference guide Arc 10)
  - `weapon:fork_appliance` -> `weapons_weaponforkappliance.webp` (Reference guide Arc 3)
  - `weapon:fork_dustbin` -> `weapons_weaponforkdustbin.webp` (Reference guide Arc 18)
  - `weapon:fork_nonos` -> `weapons_weaponforknonos.webp` (Reference guide Arc 27)
  - `weapon:fork_prism` -> `weapons_weaponforkprism.webp` (Reference guide Arc 51)
  - `weapon:fork_snowman` -> `weapons_weaponforksnowman.webp` (Reference guide Arc 23)
  - `weapon:fork_vine` -> `weapons_weaponforkvine.webp` (Reference guide Arc 37)
  - `weapon:fork_yaodao` -> `weapons_weaponforkyaodao.webp` (Reference guide Arc 13)
- **Offline bundling**: Keep images under `web/public/assets/artwork/` so the portable browser app and offline packaging retain full visual parity without network access.

## Risks / Trade-offs

- [Risk] Asset naming mismatch breaks test assertion `readdirSync(artworkDirectory).sort() === mappedFiles`.
  → [Mitigation] Sorted insertion in `ARTWORK.weapons` matching filesystem names exactly; verified via `studio.test.tsx`.
