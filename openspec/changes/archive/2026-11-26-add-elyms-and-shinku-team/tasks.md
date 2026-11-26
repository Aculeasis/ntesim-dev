# Tasks

## 1. Data Tables, Contracts, and Character Constants

- [x] 1.1 Register Elyms (`"1090"`, slug `"elyms"`, alias `"Elyms"`, `Element::Cosmos`, `WeaponGroup::One`) in `crates/data/src/constants/character/definitions.rs` and `values.rs`, update the released count assertion (`24 -> 25`) in `crates/data/src/stat_projection.rs`, and verify `python tools/cargo.py test -p ntesim-data` passes with Lv1 S0 (`1270` HP, `83` ATK, `75` DEF) and Lv80 S6 (`15392` HP, `660` ATK, `909` DEF) checks.
- [x] 1.2 Add `crates/data/src/native_damage/elyms.rs` (all 29 packets with `DamageTypeEX = Cosmos`, `DamageSourceCategory`, Lv1–13 `AtkRateBaseArray`, `ChargeAdd`, `HeterochromeAdd`, `UnbalAdd` from `tables_extracted.json`), wire `native_damage/mod.rs`, `character_actions.rs`, `ability_titles.rs`, `ability_tags.rs`, `resonance_levels.rs`, and `native_action_metadata.rs`, and verify `python tools/cargo.py test -p ntesim-data` passes.
- [x] 1.3 Add typed Elyms `HypothesisId` variants in `crates/contracts/src/hypotheses.rs` and `GameEnergySource::ElymsHit` (`elyms_hit`) in `crates/content/src/simulation/model/energy.rs`, and verify `python tools/cargo.py test -p ntesim-contracts` passes.

## 2. Shared Engine Hooks

- [x] 2.1 Generalize `crates/content/src/characters/cinematics.rs` and `crates/content/src/simulation/{admission,pause}.rs` so non-Ultimate actions can bind `ModePauseKind::Cinematic` per character/ability/packet (`Skill2`/`Skill3` -> `1_719_655 us`, `Skill1` -> none, `UltraSkill`/`UltraSkillB` -> `5_499_266 us`) without altering existing characters, and verify `python tools/cargo.py test -p ntesim-content pause` passes.
- [x] 2.2 Add a shared active-actor Ultimate Energy grant observer in `crates/content/src/simulation/{energy,reactions/remora}.rs` and combat energy grant paths (excluding `t = 0` initial start energy) dispatched through `registry::on_active_actor_energy_gain`, plus unit assertions confirming active-actor energy detection across hit, Charge reaction, and kit/weapon grants.
- [x] 2.3 Extend `crates/content/src/simulation/reactions/remora.rs` with `apply_from_owner_with_retrigger_control(..., allow_elyms_retrigger: bool)` so Passive 1 `Remora Enhancement` (`7.5s` delay / `7.0s` global ICD) does not re-trigger itself while cooperating with Akane `Perfect Pitch`, Hathor `B2`, and Chaos `B2`, and verify `python tools/cargo.py test -p ntesim-content remora` passes.

## 3. Elyms Combat Kit, Passives, Awakenings, DSL, and Oracle Tests

- [x] 3.1 Implement `crates/content/src/characters/elyms/{mod,parameters,skill,secondary,awakenings,passive,on_the_line,preview,clock}.rs` and register Elyms in `characters/{mod,state,player_inputs,packet_semantics,action_packet_bindings,action_roles,source_choices,queries,team}.rs`, `registry/{dispatch,kits,profiles,resources}.rs`, `simulation/energy.rs` (`ElymsHit`), and `dsl/{actor,catalog,queries}.rs`; verify `Aim` (`146_353 us`), `State.Damage.Followup` on `XieTongSkillA1`/`B1` with live `weapon:fork_elyms`, and `.elyms.*` DSL queries compile and pass unit checks.
- [x] 3.2 Add comprehensive unit and 2-cycle oracle tests (`elyms_actions`) in `crates/content/src/simulation/tests/` covering combat-start `PowerE1Dur`, `Skill1`/`2`/`3` redirect and selective cooldown bypass, AoE vs ST `UltraSkill`, whole-team `AtkBase` + Follow-up DMG buff, `On the Line` `3.0s x 6` pause/resume, Passive 1 (`Charge Enhancement` & `Remora Enhancement`), `Passive 2`, `A1`–`A6`/`R3`/`R6`, and multi-target bounds; finalize `docs/mechanics/characters/elyms/action-damage.md` §6.2 notes and `docs/NTE_DSL.md`, and verify `python tools/cargo.py test -p ntesim-content` passes.

## 4. Research, WASM, and Studio Assets

- [x] 4.1 Update `24 -> 25` released characters and `10626 -> 12650` combinations in `crates/research/src/**/tests.rs` and `crates/wasm/src/tests.rs`, and verify `python tools/cargo.py test -p ntesim-research` and `python tools/cargo.py test -p ntesim-wasm` pass.
- [x] 4.2 Add `elyms_hit` to `web/src/domain/{contracts/trace.ts,reports/energy-sources.ts}`, rebuild WASM and regenerate `web/src/generated/{catalog.json,names.json}`, update `24 -> 25` checks in `web/tests/localization.test.ts` and `web/tests/fixtures/resonance_parity.mjs`, and verify `node --test web/tests/localization.test.ts` and Studio parity checks pass.

## 5. Curated Team List Preset `Shinku · Charge / Akane / Elyms`

- [x] 5.1 Check whether `EquipmentSlots_elyms` can be verified from `cache/` (otherwise apply standard Lv80 offensive specialization documented in `teams.md`), author `presets/studio/shinku-elyms.nte` (`Shinku / Akane Rin / Elyms / Iroi`, Lv80 A0 limited builds: Shinku `lostradiance` + `fork_lunarphase`, Akane Rin `streetboxer` + `fork_prism`, Elyms `lostradiance` + `fork_elyms`, Iroi `speedyhedgehog` + `fork_door`) with a validated continuing rotation, and verify all 1,000 trials (`duration=120`, `seed=42`) complete without failures.
- [x] 5.2 Add `shinku-elyms` to `presets/studio/index.json` (`22` curated compositions plus `s5`), document builds and button sequence in `presets/studio/teams.md` and `presets/studio/README.md`, update `web/tests/team_catalog.test.mjs` (`curated.length === 22`, permit Elyms, Akane Rin, `weapon:fork_elyms`, and `weapon:fork_prism`), rebuild WASM, regenerate `web/src/generated/team-list.json` (`node tools/web.mjs teams`), and verify `node tools/web.mjs teams-check` and `node --test web/tests/team_catalog.test.mjs` pass.

## 6. Final Documentation and Repository Checkpoint

- [x] 6.1 Update `docs/DEVELOPMENT.md`, `openspec/README.md`, and `openspec/specs/{product-scope/reference.md,script-execution/reference.md,studio-application/reference.md}` for `25` released characters, `characters/elyms`, and `22` curated Team list entries, verifying no broken links or stale exclusions remain.
- [x] 6.2 Run `python tools/checkpoint.py run --wasm-smoke` and `python tools/openspec.py validate "add-elyms-and-shinku-team" --strict --no-interactive --concurrency 4`, confirming all checks pass and the diff is clean.

## Workflow follow-up

- Archive the completed change via `python tools/openspec.py archive "add-elyms-and-shinku-team" --yes` once implementation and checks pass.
- Commit coherent changes only after checkpoint and archive consistency verification.
