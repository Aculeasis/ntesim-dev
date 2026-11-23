# Proposal

## Why

Blackbird's finishing Burst (`GA_BlackBird_Evolve_UltraSkill`) was previously modeled with a 100-energy cost and a 5.0s cooldown based on static raw data table defaults (`DT_GameplayAbilityCostData`), incorrectly treating form exit as requiring a second full Ultimate energy pool. In live combat in *Neverness to Everness*, exiting the transformation via the finisher requires zero energy, has no separate cooldown, and Ultimate energy continues to accumulate normally during the transformed state. This prevented legal finisher-skill-ult rotation sequences (such as casting the finisher to exit form, skill for hits and bird marks, and immediately recasting the primary Burst once its 20s cooldown and 100 energy are ready). Furthermore, a team pairing Zankou with Blackbird instead of Iroi is missing from the Studio preset catalog.

## What Changes

- **Blackbird Finisher Energy Cost and Cooldown**:
  - Update `GA_BlackBird_Evolve_UltraSkill` in `crates/content/src/characters/blackbird/` and `crates/contracts/` to require 0 Ultimate Energy (instead of 100).
  - Remove the 5.0s cooldown on `GA_BlackBird_Evolve_UltraSkill`, allowing immediate execution upon transformation entry or exit.
  - Update input validation in `form.rs` and command replacement in `source.rs` to validate that only primary `GA_BlackBird_UltraSkill` demands 100 energy, while evolved `GA_BlackBird_Evolve_UltraSkill` requires 0.
  - Retain the primary Burst's native 20.0s cooldown starting at its initial activation commit.
- **Documentation and Hypothesis Updates**:
  - Update `HypothesisId::BlackbirdEvolveActionsV1` description in `crates/contracts/src/hypotheses.rs` and mechanics references in `docs/mechanics/characters/blackbird/action-damage.md` to reflect 0 energy cost and 0 cooldown on the finisher.
  - Update affected simulator tests in `crates/content/src/simulation/tests/`.
- **Add Studio Preset and Team Catalog Entry**:
  - Add new selectable preset `zankou-blackbird.nte` (`Zankou · Insight / Daffodill A3 / Blackbird`) to `presets/studio/` and `presets/studio/index.json`.
  - Implement Blackbird's rotation: cycle 1 casts skill and activates ult; subsequent cycles switch to Blackbird when energy and cooldown permit to cast finisher (exit ult) -> skill -> primary ult (re-enter ult) -> switch out.
  - Update existing Blackbird presets (`blackbird-hathor.nte`, `blackbird-nova.nte`, `blackbird-stain.nte`) to adjust rotation logic for zero-cost finisher, cleaner form exit, and skill/ult re-entry.
  - Recalculate and update `web/src/generated/team-list.json` and `presets/studio/teams.md`.

## Capabilities

### New Capabilities
- `characters/blackbird`: Black-box observable behavioral contract for Blackbird (`character:1042`), including transformation Burst, zero-cost finisher, ordinary and evolved Skill, and Malice interactions.

### Modified Capabilities
<!-- None -->

## Impact

- `crates/content/src/characters/blackbird/`: `form.rs`, `skill.rs`, `source.rs` updated for zero-cost finisher.
- `crates/contracts/src/hypotheses.rs`: `BlackbirdEvolveActionsV1` description updated.
- `crates/content/src/simulation/tests/`: `blackbird_evolve.rs` and `source_forms.rs` updated to verify 0-energy finisher execution.
- `docs/mechanics/characters/blackbird/action-damage.md`: Updated mechanics dossier.
- `presets/studio/`: New preset `zankou-blackbird.nte`, updated `index.json` and `teams.md`.
- `web/src/generated/team-list.json`: Regenerated with the new team entry and updated DPS.
