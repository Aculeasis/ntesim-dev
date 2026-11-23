# Tasks

## 1. Engine and Mechanics Updates

- [x] 1.1 Add `paid_burst_until_us: u64` field to `shinku::awakenings::State` in `crates/content/src/characters/shinku/awakenings.rs`.
- [x] 1.2 Update `crates/content/src/characters/shinku/skill.rs` in `cast_ultra` to record `shinku.paid_burst_until_us = ctx.time_us + 20_000_000` on initial burst cast, and restore remaining cooldown to `ctx.actor.ability_cooldowns` on Dragonflame Verdict finisher.
- [x] 1.3 Update `crates/content/src/registry/kits.rs` in `ShinkuKit::select_source_command` to set `*cooldown_us = state.paid_burst_until_us.saturating_sub(*timestamp_us)` when mapping to `GE_Player_Shinku_UltraSkill_Rage_Damage`.
- [x] 1.4 Verify existing Shinku action tests with `python tools/cargo.py test -- shinku`.

## 2. Studio Preset Refinement

- [x] 2.1 Update `presets/studio/shinku-charge.nte` to use `hathor skill[hold=1.5s]` and eliminate Zero's `while .zero.skill.cd>0 {wait(.zero.skill.cd);}` idle wait.
- [x] 2.2 Verify deterministic export and metrics improvement with `python tools/cargo.py run -- export-script presets/studio/shinku-charge.nte cache/shinku_bench`.

## 3. Verification and Checkpoint

- [x] 3.1 Run `python tools/checkpoint.py run` to ensure all Rust tests, formatting, and invariants pass cleanly.
