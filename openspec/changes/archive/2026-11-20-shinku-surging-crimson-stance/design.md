# Design: Shinku Surging Crimson Stance & Rotation Overhaul

## Context

Shinku's combat system in *Neverness to Everness* is characterized by a dual-stance burst transformation:
1. Building Defiant Spirit in base stance to trigger Ultimate (Crimson Fury).
2. Entering the 13-second Surging Crimson stance where basic attacks become Tidal Assault and generate Fading Reason (cap 12).
3. Consuming 12 Fading Reason to cast Scarlet Descent (up to 5 times per window), which grants 1 stack of Crimson Judgment per cast (up to 3 max).
4. In the final phase of Surging Crimson (at >= 9.0s), the Ultimate input executes Crimson Judgment dashes (consuming stacks) and concludes with the Dragonflame Verdict finisher nuke.

Currently, `crates/content/src/characters/shinku/` only models the base 5-hit attack, base 3-hit skill, and initial 9-hit burst. The Rage packets (`Skill1_Rage_Damage`, `Skill2_Rage_Damage`, `UltraSkill2_Rage_Damage`) are missing from `crates/data/src/native_damage/shinku.rs`.

## Goals / Non-Goals

**Goals:**
- Register native damage packets for Scarlet Descent (`Skill1_Rage_Damage`, `Skill2_Rage_Damage`) and the Dragonflame Verdict finisher nuke (`UltraSkill2_Rage_Damage`) in `crates/data/src/native_damage/shinku.rs` using exact client coefficients.
- Model the complete Surging Crimson state machine in `crates/content/src/characters/shinku/`:
  - Fading Reason generation and capacity (12).
  - Scarlet Descent cast limits (5) and zero-cooldown execution.
  - Crimson Judgment stack accumulation (cap 3) and dash execution.
  - Dragonflame Verdict finisher execution and stance exit.
- Integrate dynamic action selection into `ShinkuKit::select_source_command` so standard NTE DSL commands (`shinku skill`, `shinku burst`) dynamically resolve to the appropriate stance action.
- Update `presets/studio/shinku-charge.nte` and test suites to execute and verify the complete rotation.

**Non-Goals:**
- Spatial movement or target tracking physics (all-in-range stationary dummy contract is preserved).
- Enemy AI or parry reaction mechanics.
- Unrelated character or equipment changes.

## Decisions

### 1. Dynamic Stance Command Selection in `ShinkuKit::select_source_command`
- **Decision**: In `crates/content/src/registry/kits.rs`, implement `CharacterKit::select_source_command` for `ShinkuKit` (matching patterns established by Zankou and Chiz).
- **Behavior**:
  - When in Surging Crimson:
    - If `ability_id == "GA_Shinku_Skill"` and `state.fading_reason >= 12` and `state.scarlet_descent_casts < 5`:
      - Replaces damage effect ID with `GE_Player_Shinku_Skill1_Rage_Damage`.
      - Sets `cooldown_us = 0` and `energy_cost = 0`.
    - If `ability_id == "GA_Shinku_UltraSkill"` and `now >= state.surging_crimson_until_us - 4_000_000` (at or after 9.0s into the 13s stance):
      - If `state.crimson_judgment_stacks > 0`:
        - Replaces damage effect ID with `GE_Player_Shinku_UltraSkillPre_Rage_Damage`.
        - Sets `cooldown_us = 0` and `energy_cost = 0`.
      - Else (all dash stacks spent or 0):
        - Replaces damage effect ID with `GE_Player_Shinku_UltraSkill_Rage_Damage`.
        - Sets `cooldown_us = 20_000_000` and `energy_cost = 0`.
  - Otherwise (outside Surging Crimson or when prerequisites are not met): retains ordinary ability descriptors.
- **Alternatives Considered**: Creating separate DSL keywords (e.g. `shinku descent;`, `shinku judgment;`). Rejected because in-game controls map these dynamically to the Skill (E) and Ultimate (Q) buttons during the stance, and preserving standard DSL action bindings keeps scripts intuitive and compatible.

### 2. Packet Scheduling and State Transitions in `on_cast`
- **Decision**:
  - For `GE_Player_Shinku_Skill1_Rage_Damage`: schedules child packet `GE_Player_Shinku_Skill2_Rage_Damage`, consumes 12 Fading Reason, increments `scarlet_descent_casts`, and increments `crimson_judgment_stacks` (clamped to 3).
  - For `GE_Player_Shinku_UltraSkillPre_Rage_Damage`: schedules 3 child packets of `GE_Player_Shinku_UltraSkillPre_Rage_Damage` (4 total hits per dash, matching native `ShinkuUltraSkillPreRage`), and decrements `crimson_judgment_stacks`.
  - For `GE_Player_Shinku_UltraSkill_Rage_Damage`: schedules 9 child packets of `GE_Player_Shinku_UltraSkill_Rage_Damage` (10 flurry hits) plus 1 child packet of `GE_Player_Shinku_UltraSkill2_Rage_Damage` (the 400% nuke), and terminates Surging Crimson (`surging_crimson_until_us = now`).
  - For `GA_Shinku_Melee` in Surging Crimson: advances Tidal Assault stages and increments `fading_reason` by 3 per stage (reaching 12 in 4 inputs or 1 full combo).

### 3. Client-Matched Multipliers in `crates/data/src/native_damage/shinku.rs`
- **Decision**: Add packet definitions directly from `DT_SkillDamageData.json`:
  - `GE_Player_Shinku_Skill1_Rage_Damage`: Atk array `[182000..]` from native row, Category E, Cosmos.
  - `GE_Player_Shinku_Skill2_Rage_Damage`: Atk array `[2176000..]` (217.6% Lv1, 372.9% Lv8), Category E, Cosmos.
  - `GE_Player_Shinku_UltraSkill2_Rage_Damage`: Atk array `[4000000..]` (400% Lv1, 685.5% Lv8), Category Q, Cosmos.

## Risks / Trade-offs

- [Risk] Legacy tests that manually constructed raw `GameActionCommand::InvokeAbility` with `GE_Player_Shinku_UltraSkillPre_Rage_Damage` might expect single packets.
  -> Mitigation: Preserve compatibility for direct packet hypotheses in tests while enabling full dynamic packet trains when invoked through public actions.
- [Risk] Curated team DPS benchmark changes in `shinku-charge.nte`.
  -> Mitigation: The change improves realism and damage fidelity to match actual character performance; benchmark changes will be documented and verified with deterministic replays.
