# Proposal

## Why

Elyms (`character:1090`, `090_elyms`) is the 25th released NTE playable character (S-Class `Cosmos`, Solid Arc). Her signature weapon `weapon:fork_elyms` (*Echoes on Hold*), portrait/weapon artwork, and raw mechanics/frame-timing acquisition dossier (`docs/mechanics/characters/elyms/action-damage.md`) are already in the repository, but she is not yet registered or playable in the shared engine and Studio. Adding Elyms completes 25-character parity (`12,650` four-member combinations) and unlocks Studio Team list evaluation for the meta *Shinku · Charge* lineup with Akane Rin, Elyms, and Iroi (`Shinku-Akane-Elyms-Iroi`).

## What Changes

- **Add Elyms (`character:1090`, slug `elyms`, script alias `Elyms`) across `ntesim-data`, `ntesim-contracts`, `ntesim-content`, `ntesim-research`, and `web/`**:
  - Register base Lv1 S0 / Lv80 S6 stats (`1270` HP, `83` ATK, `75` DEF -> `15392` HP, `660` ATK, `909` DEF), native packet scalars/metadata (`DamageTypeEX = Cosmos`, `DamageSourceCategory`, `ChargeAdd`, `HeterochromeAdd`, `UnbalAdd`) for all 13 levels, ability titles/tags, `WeaponGroup::One`, `Aim` mode packet binding, and `R1`–`R6` scaling in `crates/data`.
  - Implement Elyms's combat state machine in `crates/content`: Connection Strength (`Weak` / `Medium` / `Strong`), combat-start `Intermittent Existence` (`Skill2`) readiness, `GA_Elyms_Skill` 3-way redirect (`The Ones Who Left`, `Intermittent Existence`, `Goodbye to You` with selective cooldown bypass), `Optical Illusion` stacking, `Instant Glitch` / `Crosstalk` / `Interference` / `Memory Anchor` (`Aim`) / `Falling Dream` / *Okay! Shake on It!* / `I'm Here` (`QTE`), AoE & single-target `Do You Get Deja Vu` (`GA_Elyms_UltraSkill` / `GA_Elyms_UltraSkillB`) with team `AtkBase`-based ATK & Follow-up DMG buff, off-field `On the Line` / `Remembrance` `3.0s x 6` coordinated follow-up packets (`FOLLOW_UP_DAMAGE_TAG` compatible with `weapon:fork_elyms`), Passive 1 (`Relay Node`: `Charge Enhancement` & `Remora Enhancement`), Passive 2 (`Pity System`), `A1`–`A6`, `R3`, `R6`, DSL queries, and `ElymsHit` energy receipts.
  - Introduce reusable engine mechanisms in `crates/content`: non-Ultimate `ModePauseKind::Cinematic` pause admission (`Skill2`/`Skill3` `1_719_655 us` vs unpaused `Skill1`), active-actor Ultimate Energy observation for `Charge Enhancement`, live Esper Cycle accumulation efficiency contribution in `cycle.rs`, and `Remora` re-trigger suppression on `Remora Enhancement`'s `7.5s` delayed reapplication.
  - Update released character counts (`24 -> 25`, `10626 -> 12650` four-character compositions) across `crates/data`, `crates/research`, `crates/wasm`, and `web/tests/`, and regenerate `web/src/generated/{catalog.json,names.json}` and trace energy labels (`elyms_hit`).
- **Add curated Team list preset `Shinku · Charge / Akane / Elyms` (`shinku-elyms`)**:
  - Author `presets/studio/shinku-elyms.nte` (`Shinku / Akane Rin / Elyms / Iroi`) using the standard `teams.md` Lv80 A0 limited comparison rules (Shinku `Lost Radiance` + `Blushing Mirage`, Akane Rin `Street Boxer` + `Far Side of the Moon`, Elyms `Lost Radiance` + `Echoes on Hold`, Iroi `Speedy Hedgehog` + `The Wrong Gate`) and a validated continuing rotation coordinating `Remora`, `Blossom`, `Charge`, `Relay Node`, `On the Line`, and `Surging Crimson`.
  - Register the preset in `presets/studio/index.json` and document it in `presets/studio/teams.md` and `presets/studio/README.md`, regenerate `web/src/generated/team-list.json` from a rebuilt WASM engine (`node tools/web.mjs teams`), and lift the `studio-application` / `web/tests/team_catalog.test.mjs` exclusion on Elyms, Akane Rin, and their signature weapons while expanding the curated collection count to `22`.

## Capabilities

### New Capabilities
- `characters/elyms`: Observable combat mechanics, progression stats, Connection Strength & Optical Illusion state transitions, `GA_Elyms_Skill` redirect & selective cooldown bypass, non-Ultimate/Ultimate timer-pause lifecycles, off-field `On the Line` / `Remembrance` coordinated follow-ups, Passives 1–2, `A1`–`A6` / `R3` / `R6`, and DSL queries for Elyms (`character:1090`).

### Modified Capabilities
- `runtime-content`: Expand released character coverage to `25` (`12,650` compositions) with Elyms (`character:1090`, `elyms`, `ElymsHit`), non-Ultimate `ModePauseKind::Cinematic` admission, active-actor energy & dynamic Esper Cycle efficiency hooks, `Remora` anti-recursion flag, `Aim` & `FOLLOW_UP_DAMAGE_TAG` bindings, and regenerated Studio catalog/name assets.
- `studio-application`: Expand the curated Team list collection from `21` to `22` complete teams by adding `shinku-elyms` (`Shinku / Akane Rin / Elyms / Iroi`), remove the Elyms / Akane Rin / `fork_prism` / `fork_elyms` exclusion from curated teams, and refresh `team-list.json` and catalog tests.

## Impact

- Crates: `crates/data`, `crates/contracts`, `crates/content`, `crates/research`, `crates/wasm`.
- Studio & Presets: `presets/studio/{shinku-elyms.nte,index.json,teams.md,README.md}`, `web/src/domain/{contracts/trace.ts,reports/energy-sources.ts}`, `web/src/generated/{catalog.json,names.json,team-list.json}`, `web/tests/{localization.test.ts,fixtures/resonance_parity.mjs,team_catalog.test.mjs}`.
- Docs & Specs: `openspec/specs/{characters/elyms/spec.md,runtime-content/spec.md,studio-application/spec.md,studio-application/reference.md,product-scope/reference.md,script-execution/reference.md}`, `openspec/README.md`, `docs/{DEVELOPMENT.md,NTE_DSL.md,mechanics/characters/elyms/action-damage.md}`.
