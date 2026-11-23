# Tasks

## 1. Data and Native Damage Packets

- [x] 1.1 Register native damage packet definitions for `GE_Player_Shinku_Skill1_Rage_Damage`, `GE_Player_Shinku_Skill2_Rage_Damage`, and `GE_Player_Shinku_UltraSkill2_Rage_Damage` in `crates/data/src/native_damage/shinku.rs` and verify `python tools/cargo.py check` passes.

## 2. Shinku Stance State and Engine Mechanics

- [x] 2.1 Extend `State` in `crates/content/src/characters/shinku/awakenings.rs` with `fading_reason`, `scarlet_descent_casts`, and `crimson_judgment_stacks`.
- [x] 2.2 Update `crates/content/src/characters/shinku/skill.rs` to handle Surging Crimson Tidal Assault Fading Reason generation, Scarlet Descent 2-hit packet scheduling, Crimson Judgment 4-hit dash packet scheduling, and Dragonflame Verdict 10+1 hit finisher execution.
- [x] 2.3 Implement `select_source_command` in `crates/content/src/registry/kits.rs` (and `ShinkuKit`) to dynamically route `skill` and `burst` commands during Surging Crimson.

## 3. Verification and Scenario Update

- [x] 3.1 Add unit and integration tests in `crates/content/src/simulation/tests/shinku_actions.rs` covering the full Surging Crimson loop (Burst -> Tidal Assault -> Scarlet Descent -> Crimson Judgment x3 -> Dragonflame Verdict) and verify `python tools/cargo.py test shinku_actions`.
- [x] 3.2 Update `presets/studio/shinku-charge.nte` to utilize the empowered Surging Crimson rotation and verify with `python tools/cargo.py run -- run-script presets/studio/shinku-charge.nte`.
- [x] 3.3 Run `python tools/checkpoint.py run` and verify all checks pass.
