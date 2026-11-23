# Tasks

## 1. Asset Staging and Mapping

- [x] 1.1 Add 10 bundled WebP artwork assets to `web/public/assets/artwork/` and verify valid RIFF/WEBP headers and dimensions
- [x] 1.2 Map all 10 weapons in `web/src/domain/artwork.ts` under `ARTWORK.weapons` matching catalog IDs

## 2. Verification and Delivery

- [x] 2.1 Run `node tools/web.mjs test tests/studio.test.tsx` to verify complete catalog artwork mapping and exact directory parity
- [x] 2.2 Run `python tools/checkpoint.py run` to verify formatting, typechecks, tests, and distribution builds
