# Proposal

## Why

In *Neverness to Everness*, characters belonging to the Lower Three Rings (下三环: Chaos/暗, Incantation/咒, Psyche/魂) have an Ultimate Energy capacity and Burst cost of 120, while characters of the Upper Three Rings (上三环: Cosmos/光, Nature/灵, Lakshana/相) have 100. Native client data (`DT_PlayerPackData` and `DT_GameplayAbilityCostData`) assigns both `ChargeMax: 120` and `GACost: 120` to Lacrimosa, Sakiri, Zankou, Haniel, Baicang, Adler, Fadia, Daffodil, and Aurelia.

While character constants (`crates/data/src/constants/character/definitions.rs`) correctly assign 120 base energy to all nine characters, `crates/content/src/build/public_actions.rs` hardcoded a fallback Burst energy cost of 100. Only Haniel, Fadia, Daffodil, and Baicang had explicit 120 overrides, leaving Lacrimosa, Zankou, Sakiri, Adler, and Aurelia with a 100 energy cost against a 120 capacity. Consequently, in the simulator their Bursts spend only 100 energy and leave 20 residual energy, whereas in-game their Bursts consume the entire 120 energy pool to zero. Furthermore, `openspec/specs/characters/lacrimosa/spec.md` erroneously specified 100 energy.

## What Changes

- **Harmonize Public Action Burst Energy Costs**: Update `crates/content/src/build/public_actions.rs` so that public Burst action energy costs dynamically match the character's native base energy capacity (120 for Chaos, Incantation, and Psyche characters, 100 for Cosmos, Nature, and Lakshana), eliminating manual and missing per-character overrides.
- **Update Lacrimosa Specification**: Align `openspec/specs/characters/lacrimosa/spec.md` (`Requirement: Burst Execution by Stance`) and its Mode A Burst commit scenario to require 120 energy rather than 100.
- **Update Character Documentation and Action-Damage Dossiers**: Fix outdated references in `docs/mechanics/characters/zankou/action-damage.md` that stated paid Inferno/Bloodfeast costs 100 energy instead of 120.
- **Reconcile Simulation Tests and DSL Examples**: Adjust affected simulation tests, benchmarks, or example `.nte` scripts where Lacrimosa, Zankou, Sakiri, Adler, or Aurelia Bursts were asserting or relying on 100 energy spend.

## Capabilities

### Modified Capabilities
- `characters/lacrimosa`: Update Mode A Burst energy requirement and commit scenario from 100 energy to 120 energy.

## Impact

- **Rust crates**:
  - `crates/content/src/build/public_actions.rs`: Resolves Burst energy cost from character definition base energy (`base.energy`), correctly assigning 120 cost to all 120-capacity characters.
  - `crates/content/src/simulation/tests/*`: Reconciles tests asserting energy spend or residual energy for affected characters.
- **OpenSpec**:
  - `openspec/specs/characters/lacrimosa/spec.md`: Delta spec updating Burst requirement and scenario to 120 energy.
- **Documentation**:
  - `docs/mechanics/characters/zankou/action-damage.md`: Replaces obsolete 100 energy references with native 120 cost.
