# Tasks

## 1. Engine and Mechanics Updates

- [x] 1.1 Update `crates/content/src/characters/blackbird/form.rs` input validation to require 100 energy exclusively on `GA_BlackBird_UltraSkill`, allowing `GA_BlackBird_Evolve_UltraSkill` at 0 energy.
- [x] 1.2 Update `crates/content/src/characters/blackbird/source.rs` to set `energy_cost = 0` and `cooldown_us = 0` when mapping to `GA_BlackBird_Evolve_UltraSkill`.
- [x] 1.3 Update `crates/content/src/characters/blackbird/skill.rs` in `cast_evolve_ultra` to remove the 5.0s cooldown insertion on the finisher.
- [x] 1.4 Update `crates/contracts/src/hypotheses.rs` description for `BlackbirdEvolveActionsV1` and `docs/mechanics/characters/blackbird/action-damage.md` to document zero cost and zero cooldown for the finisher.
- [x] 1.5 Update existing unit and integration tests in `crates/content/src/simulation/tests/blackbird_evolve.rs` and `source_forms.rs`, and verify with `python tools/cargo.py test -p ntesim-content`.

## 2. Studio Preset and Team Catalog Integration

- [x] 2.1 Create `presets/studio/zankou-blackbird.nte` configuring Zankou, Daffodill A3 `[3,4,5]`, Sakiri (with Speedy Hedgehog), and Blackbird (crit build), implementing the finisher -> skill -> ult rotation cycle.
- [x] 2.2 Add `zankou-blackbird` to `presets/studio/index.json` and document the roster in `presets/studio/teams.md`.
- [x] 2.3 Update existing Blackbird team presets (`presets/studio/blackbird-hathor.nte`, `presets/studio/blackbird-nova.nte`, `presets/studio/blackbird-stain.nte`) to adjust rotations for zero-cost finisher and clean form exit/re-entry.
- [x] 2.4 Rebuild WASM and recalculate team catalog with `node tools/web.mjs teams` and verify with `node tools/web.mjs teams --check`.

## 3. Verification and Checkpoint

- [x] 3.1 Run `python tools/checkpoint.py run` to verify all test suites, type checking, and format consistency pass cleanly.
