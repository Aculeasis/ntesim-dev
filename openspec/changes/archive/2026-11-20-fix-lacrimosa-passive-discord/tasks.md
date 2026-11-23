# Tasks

## 1. Passive Mechanic Correction

- [x] 1.1 Remove the periodic DoT `is_broken` trigger from `on_hit` in `crates/content/src/characters/lacrimosa/skill.rs`. Verify via `python tools/cargo.py check`.
- [x] 1.2 Update `lacrimosa_public.rs` to assert that DoT ticks on Broken targets without Discord do not trigger `AnHunZhouTwo_Damage`, add a dedicated test for Discord reactions triggering Tomato Banquet, and verify `daffodil_break_discord.rs` continues to pass.

## 2. Ability Naming & Studio Presentation

- [x] 2.1 Add `AbilityTitle { ability_id: "GA_lacrimosa_Passive_1", title: "Tomato Banquet" }` to `crates/data/src/ability_titles.rs` and update `web/src/generated/names.json`.
- [x] 2.2 Verify `web/src/domain/reports/source-label.ts` formats `GA_lacrimosa_Passive_1` as `"Tomato Banquet"` and confirm with unit/web tests (`localization.test.ts`).

## 3. Preset Verification & Checkpoint

- [x] 3.1 Re-run `presets/studio/blackbird-hathor.nte` simulation and verify damage distribution matches intended mechanics (Lacrimosa ~16%, Blackbird ~56% carry share, 0 phantom reaction hits).
- [x] 3.2 Run `python tools/checkpoint.py run` to verify overall repo health.
