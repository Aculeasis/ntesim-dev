# Tasks

## 1. Discord Deduplication on Scorch Replacement

- [x] 1.1 In `crates/content/src/simulation/reactions/replacement.rs`, update `on_dot_delivery` to pass `new_discord_application = false` so that ally DoT-triggered Scorch stack additions and refreshes preserve the existing `discord_generation` ID instead of minting a new one. Verify by checking that `on_dot_delivery` compiles cleanly.
- [x] 1.2 Add regression tests in `crates/content/src/simulation/tests/zankou_scorch.rs` verifying that repeated DoT ticks against an enemy with active Scorch and Nova emit exactly one Discord reaction callback and do not recursively schedule multiple `AnHunZhouTwo_Damage` hits. Verify with `python tools/cargo.py test --package ntesim-content --test simulation_tests zankou_scorch`.

## 2. Zankou Blackbird Preset Streamlining

- [x] 2.1 In `presets/studio/zankou-blackbird.nte`, remove the redundant immediate switch to Daffodill directly preceding Zankou's field entry, eliminating idle switch recovery while preserving lawful Phantom Step usage.
- [x] 2.2 Verify single-trial execution and rotation throughput of `presets/studio/zankou-blackbird.nte` using `python tools/cargo.py run --bin ntesim -- run-script presets/studio/zankou-blackbird.nte`.

## 3. Catalog Regeneration and Checkpoint

- [x] 3.1 Regenerate `web/src/generated/team-list.json` to reflect corrected Discord deduplication and the streamlined Zankou-Blackbird rotation using `tools/web.mjs` / `tools/studio_teams.mts`.
- [x] 3.2 Run full project validation using `python tools/checkpoint.py run` and verify all native, WASM, and web acceptance checks pass cleanly.
