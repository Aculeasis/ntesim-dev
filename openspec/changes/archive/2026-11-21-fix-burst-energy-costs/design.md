# Design

## Context

See `proposal.md` for background and motivation.

In `ntesim`, character base stats and energy capacities are statically defined in `crates/data/src/constants/character/definitions.rs` (`Base::new(hp, atk, def, energy)`). The nine characters belonging to the Lower Three Rings (Lacrimosa, Sakiri, Zankou, Haniel, Baicang, Adler, Fadia, Daffodil, Aurelia) are configured with `base.energy = 120`, matching native client tables (`DT_PlayerPackData: ChargeMax = 120.0` and `DT_GameplayAbilityCostData: GACost = 120.0`).

However, `crates/content/src/build/public_actions.rs` hardcoded the primary input role `"burst"` energy cost to 100 by default, with only explicit overrides for Fadia, Daffodil, Haniel, and Baicang. As a result, Lacrimosa (`character:1004`), Sakiri (`character:1003`), Zankou (`character:1036`), Adler (`character:1033`), and Aurelia (`character:1070`) defaulted to a 100 Burst cost.

## Goals / Non-Goals

**Goals:**
- Dynamically resolve Burst energy cost in `crates/content/src/build/public_actions.rs` from `ntesim_data::character::definition(character)`, so that any character with a 120-energy base capacity automatically costs 120 energy for their primary Burst action.
- Update `openspec/specs/characters/lacrimosa/spec.md` to require 120 energy for Mode A Burst.
- Clean up outdated references in mechanics dossiers (`docs/mechanics/characters/zankou/action-damage.md`).
- Update affected simulation and contract test assertions where `initial_energy` or `spent_energy` asserted 100 instead of 120 for these characters.

**Non-Goals:**
- Modifying energy gain, Charge Efficiency, or off-field sharing ratios (0.6 factor remains untouched).
- Modifying secondary forms, finishers, or free-cast procs (e.g., Blackbird transformation finisher costing 0, Zankou's free enhanced Inferno Flamenco proc costing 0).
- Changing characters with a 100-energy capacity.

## Decisions

### Decision 1: Derive Burst Energy Cost from Character Definition in `public_actions.rs`
Instead of extending the manual `if character == ...` chain in `public_actions.rs`, the Burst action builder will query `ntesim_data::character::definition(character)`:
```rust
let default_burst_cost = ntesim_data::character::definition(character)
    .map(|d| d.base.energy as u64)
    .unwrap_or(100);
```
- *Rationale*: Eliminates manual per-character branches. Guarantees that every character's primary Burst cost matches their native base energy pool (`ChargeMax` = `GACost` in `DT_GameplayAbilityCostData`).
- *Alternatives considered*: Adding `character:1004`, `character:1036`, `character:1003`, `character:1033`, `character:1070` to the manual `match`/`if` chain. Rejected as repetitive and brittle.

### Decision 2: Update Spec and Documentation Contracts
Update `openspec/specs/characters/lacrimosa/spec.md` (`Requirement: Burst Execution by Stance`) to specify 120 energy. Repair `docs/mechanics/characters/zankou/action-damage.md` to state that paid Inferno/Bloodfeast costs 120 energy (aligning with `DT_PlayerPackData.zankou_base.ChargeMax=120` and `GA_Zankou_UltraSkill.GACost=120`).

### Decision 3: Update Test Fixtures and Scenarios
Any tests that specifically author `start_energy=100` to execute an immediate Burst on 120-energy characters (such as Aurelia or Lacrimosa) must provide `start_energy=120` to satisfy the 120-energy commitment, and tests checking `spent_energy` must assert 120 instead of 100.

## Risks / Trade-offs

- **[Risk] Existing simulation tests failing on energy deficit** → When Burst cost increases from 100 to 120 for Lacrimosa, Zankou, Sakiri, Aurelia, or Adler, scenarios that only accumulated 100 energy before casting Burst will fail action admission.
  *Mitigation*: Review all simulation tests for these characters in `crates/content/src/simulation/tests/` and update scenarios to ensure proper 120-energy funding.
