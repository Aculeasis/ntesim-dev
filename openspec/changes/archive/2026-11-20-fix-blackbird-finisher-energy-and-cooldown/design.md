# Design

## Context

Blackbird's transformation Burst (`GA_BlackBird_UltraSkill`) and finisher (`GA_BlackBird_Evolve_UltraSkill`) were previously constrained by an input-level requirement in `form.rs` demanding 100 energy on both abilities. Additionally, `source.rs` preserved the 100 energy cost and 20s cooldown from the primary action when mapping to `GA_BlackBird_Evolve_UltraSkill`, and `skill.rs` added an artificial 5.0s cooldown upon finisher commit. In NTE, the finisher simply executes to exit the transformation form without consuming energy or having a separate cooldown.

## Goals / Non-Goals

**Goals:**
- Set `energy_cost: 0` and `cooldown_us: 0` for `GA_BlackBird_Evolve_UltraSkill` in `crates/content/src/characters/blackbird/source.rs` and `skill.rs`.
- Update input validation in `crates/content/src/characters/blackbird/form.rs` so that only `GA_BlackBird_UltraSkill` enforces 100 energy cost.
- Preserve 20.0s cooldown on `GA_BlackBird_UltraSkill` from its initial activation commit.
- Update `crates/contracts/src/hypotheses.rs` and mechanics docs (`docs/mechanics/characters/blackbird/action-damage.md`).
- Update affected Rust tests in `crates/content/src/simulation/tests/`.
- Add `presets/studio/zankou-blackbird.nte` using Zankou, Daffodill A3 `[3,4,5]`, Sakiri (with Speedy Hedgehog), and Blackbird (crit build).
- Implement Blackbird's rotation: initial cycle casts skill then activates ult; subsequent cycles switch to BB when ult can be executed (finisher -> skill -> ult -> switch out).
- Update existing presets featuring Blackbird (`blackbird-hathor.nte`, `blackbird-nova.nte`, `blackbird-stain.nte`) to adjust rotation for 0-energy finisher, avoiding unbuffed downtime and enabling smooth finisher -> skill -> ult chains where cooldown permits.
- Regenerate `web/src/generated/team-list.json` and update `presets/studio/teams.md`.

**Non-Goals:**
- Redesigning other characters or set effects.
- Modifying client WASM transport bindings or UI components.

## Decisions

- **Decision 1: Action Command Transformation in `source.rs`**:
  - When `source.rs` maps `GA_BlackBird_UltraSkill` to `GA_BlackBird_Evolve_UltraSkill` in transformed form, set `command.energy_cost = 0` and `command.cooldown_us = 0`.
  - *Rationale*: Keeps authoring simple in `.nte` (`blackbird burst;` works seamlessly whether transformed or not), matching how ordinary vs evolved abilities are handled.
- **Decision 2: Remove Finisher Cooldown in `skill.rs`**:
  - In `cast_evolve_ultra`, remove `ctx.actor.ability_cooldowns.insert(ctx.ability_id.into(), ctx.time_us + 5_000_000)`.
  - *Rationale*: Matches live game behavior where the finisher is immediately available while transformed and does not impose a 5s cooldown.
- **Decision 3: Speedy Hedgehog Placement in the New Team**:
  - Sakiri equips `speedyhedgehog` and `fork_mofeikesi` (consistent with all other teams where Sakiri is the buffer and Iroi is absent), while Blackbird equips `devilsbloodcurse` and `fork_twinbirds` with a crit recipe.
  - *Rationale*: Maximizes team synergy (+15% party ATK on Sakiri burst) and Blackbird's sub-DPS damage.

## Risks / Trade-offs

- **[Existing Test Expectations]** → Tests asserting 100 energy spend on finisher (e.g. `blackbird_evolve_ultraskill_finisher_cost_form_removal_and_a1_spend`) must be updated to expect 0 energy spend on finisher and total 100 energy spend per single cycle.
