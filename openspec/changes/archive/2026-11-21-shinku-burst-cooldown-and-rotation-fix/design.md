# Design: Shinku Burst Cooldown and Charge Rotation Fix

## Context

Shinku's combat profile in *Neverness to Everness* relies on entering Surging Crimson via her Ultimate ability (Crimson Fury, 60 Defiant Spirit, nominal 20.0s cooldown), fighting in stance for up to 13 seconds (generating Fading Reason, casting Scarlet Descent, and building Crimson Judgment stacks), and finishing after >= 9.0s with Crimson Judgment dashes followed by the Dragonflame Verdict finisher nuke.

In the previous implementation (`openspec/changes/archive/2026-11-20-shinku-surging-crimson-stance/`), `ShinkuKit::select_source_command` dynamically mapped the Ultimate input at stance conclusion to `GE_Player_Shinku_UltraSkill_Rage_Damage` with `*cooldown_us = 20_000_000`. This assigned a new 20-second cooldown at the *exit* of Surging Crimson (~13.5s into combat), stretching the total rotation cycle to ~33.5s. In reality, Shinku's 20.0s Ultimate cooldown begins when Crimson Fury is initially activated.

Additionally, the curated preset `presets/studio/shinku-charge.nte` contains an idle wait on Zero (`while .zero.skill.cd>0 {wait(.zero.skill.cd);}`, wasting 1.51s) and leaves Shinku on-field spamming unbuffed basic attacks waiting for the inflated burst cooldown. This leads to massive energy overflow (>530 energy over 120s) and depressed team DPS.

## Goals / Non-Goals

**Goals:**
- Track Shinku's initial 20.0s Burst deadline in `State` via `paid_burst_until_us`.
- In `ShinkuKit::select_source_command`, set Dragonflame Verdict's cooldown to `state.paid_burst_until_us.saturating_sub(*timestamp_us)` rather than a fixed 20s penalty.
- In `cast_ultra` in `skill.rs`, set `shinku.paid_burst_until_us = ctx.time_us + 20_000_000` on initial activation, and restore any remaining duration to `ctx.actor.ability_cooldowns` upon executing Dragonflame Verdict.
- Optimize `presets/studio/shinku-charge.nte` to utilize Hathor's held skill (`skill[hold=1.5s]`) to generate unbalance, extra hits, and synchronize Zero's skill cooldown, eliminating Zero's idle wait.
- Verify that Shinku's rotation interval shortens from ~33.5s to ~20–23s, party energy overflow is substantially reduced, and overall team DPS increases.

**Non-Goals:**
- Modifying other characters, weapons, or set bonus mechanics.
- Altering the 13.0s stance duration or the 9.0s gating before dashes become available.
- Altering client WASM schemas or UI components.

## Decisions

### 1. Track `paid_burst_until_us` in `shinku::awakenings::State`
- **Decision**: Add `pub paid_burst_until_us: u64` to `State` (defaulting to 0 in `Default`).
- **Rationale**: Persists the real combat timestamp by which the 20-second cooldown commitment expires, surviving stance transitions and allowing continuing cycle persistence.

### 2. Dynamically Compute Remaining Cooldown in `ShinkuKit::select_source_command`
- **Decision**: In `crates/content/src/registry/kits.rs`, when replacing `GA_Shinku_UltraSkill` with `GE_Player_Shinku_UltraSkill_Rage_Damage`:
  ```rust
  *cooldown_us = state.paid_burst_until_us.saturating_sub(*timestamp_us);
  ```
- **Rationale**: When Dragonflame Verdict is cast at e.g. $t = 13.5\text{s}$, the remaining cooldown until $t = 20.0\text{s}$ is only $6.5\text{s}$, not $20.0\text{s}$. If more than 20 seconds have elapsed, cooldown is 0.

### 3. Cooldown Synchronization in `shinku::skill::cast_ultra`
- **Decision**:
  - On initial Burst cast (`"GA_Shinku_UltraSkill"` or `"GE_Player_Shinku_UltraSkill1"`): record `shinku.paid_burst_until_us = ctx.time_us + 20_000_000`. The engine temporarily sets the cooldown map to `ctx.time_us + 9_000_000` to allow the 9s dash gating.
  - On Dragonflame Verdict cast (`"GE_Player_Shinku_UltraSkill_Rage_Damage"`): if `shinku.paid_burst_until_us > ctx.time_us`, insert `shinku.paid_burst_until_us` into `ctx.actor.ability_cooldowns` under `"GA_Shinku_UltraSkill"`.
- **Rationale**: Ensures the engine's internal cooldown map faithfully reflects the remaining cooldown commitment when exiting Surging Crimson.

### 4. Rotation Tuning in `presets/studio/shinku-charge.nte`
- **Decision**:
  - Update Hathor's action from `hathor skill;` to `hathor skill[hold=1.5s];`.
  - Replace `while .zero.skill.cd>0 {wait(.zero.skill.cd);}` with direct action flow.
  - Exit Shinku back to supports once Dragonflame Verdict finishes and support buffs are refreshed.
- **Rationale**: Hathor's held skill channels physical damage ticks, advances unbalance, and naturally covers Zero's remaining skill cooldown without dead idle time, producing a fluid, synchronized 23-second team cycle.

## Risks / Trade-offs

- **[Risk] Test expectations for Shinku action cooldowns**: Existing unit tests checking `ability_cooldowns` after Dragonflame Verdict might expect specific timestamps.
  - *Mitigation*: Existing unit tests in `shinku_actions.rs` were verified and pass cleanly (43/43 tests pass).
- **[Risk] Continuing cycle persistence**: Preserving state across continuous cycles must not leave stale deadlines in the past.
  - *Mitigation*: Using `.saturating_sub(*timestamp_us)` cleanly yields 0 if the timestamp is already past the deadline.
