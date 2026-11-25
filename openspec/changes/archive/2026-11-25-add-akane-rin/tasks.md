# Tasks

## 1. Data & Hypothesis Contracts (`ntesim-data`, `ntesim-contracts`)

- [x] 1.1 Register Akane Rin (`character:1057`, `slug: "akane"`, `script_alias: Some("Akane Rin")`, `Element::Lakshana`, `WeaponGroup::Two`, Lv1 S0 base stats) in `crates/data/src/constants/character/definitions.rs`, add Lv80 S6 projection assertions in `crates/data/src/constants/character/values.rs`, and update `23 -> 24` released counts in `definitions.rs`, `stat_projection.rs`, and `crates/data/src/recommendations/gear/tests.rs`; verify with `python tools/cargo.py test -p ntesim-data`.
- [x] 1.2 Add `crates/data/src/native_damage/akane.rs` (excluding out-of-combat `Buff_Akane057_BattleState` and UI-only `UpgradeSkill_DisplayGE`), populate `character_actions.rs`, `ability_titles.rs`, `ability_tags.rs` (`GA_Akane_Melee`, `GA_Akane_Skill`, `GA_Akane_UltraSkill`), and `resonance_levels.rs` while removing pre-release `1057`/`GA_Akane_Skill` absence assertions; verify `python tools/cargo.py test -p ntesim-data` passes with `24` released characters.
- [x] 1.3 Add Akane Rin `HypothesisId` variants in `crates/contracts/src/hypotheses.rs`, clean up `GA_Akane_Skill`/`GA_Akane_UltraSkill` from `crates/content/src/characters/action_roles.rs` (retaining `GA_Akane_QTE`), and finalize documentation notes in `docs/mechanics/characters/README.md` and `docs/mechanics/characters/akane/action-damage.md`; verify `python tools/cargo.py test -p ntesim-contracts` compiles and passes.

## 2. Shared `Remora` Extension & Akane Rin Kit (`ntesim-content`)

- [x] 2.1 Extend `crates/content/src/simulation/reactions/remora.rs` and `crates/content/src/registry/{dispatch.rs,profiles.rs}` to sample pre-overwrite `existing_remaining_us` before refreshing active `Remora` timers and provide owner-sourced periodic `apply_from_owner` ticks preserving Esper Cycle reaction counts alongside `Hathor` B2 and `Chaos` B2 listeners; verify `python tools/cargo.py test -p ntesim-content -- remora` passes.
- [x] 2.2 Implement `crates/content/src/characters/akane/` (`Stringblade` `N1`–`N4` / `AppearMelee`, `Melee4` `+30_000 us` trigger delay, `N2`/`N3` hitstop & `Melee3_1`, `Stage Dive`, `Evade` / `Critical Riposte`, `Altered Chord`, `String Energy` `0..=30` deduplication, `Rhythm Wave` Press/Hold mutual exclusivity, multi-dummy `Overtone` `CoefAdd` targeting, Press-only `Synced Resonance` preserving `Rhythm Wave` CD & mode timer, and two-stage `Sonic Boom` with `4_012_833 us` pause, `Solo Cadenza`, and `Starlit Anthem`) and wire `RuntimeState::Akane`, `cinematics.rs`, `player_inputs.rs`, `action_packet_bindings.rs`, `callbacks/{events.rs,handlers.rs}`, and `registry/kits.rs`; verify new Akane kit unit tests in `crates/content` pass.
- [x] 2.3 Implement `Perfect Pitch` permanent unlock, `3.0s` periodic `Remora` ticks on/off field, `1.0s` per-target ICD `GE_Reaction_3_new_1057_Damage` (Cat `E`, `floor(remaining_seconds) * 0.20` capped at `+160%`), and `A1`–`A6`/`R3`/`R6` tiers (including `R3` battle-entry `30` `String Energy` and `R6` post-spend `+10` refund + `+64% CritDMG` coexisting with `A4`), extending `simulation/tests/{ordinary_hit_energy.rs,far_side_moon.rs}` (`akane_hit` + `weapon:fork_prism` pairing) and adding the 2-cycle numeric oracle test; verify `python tools/cargo.py test -p ntesim-content` passes.
- [x] 2.4 Add `start_string_energy` (`0..=30`) actor configuration and validation in `crates/content/src/dsl/{actor.rs,catalog.rs}`, implement `.string`, `.mode`, `.overtone`, `.reverb`, `.cadenza`, `.anthem`, `.perfect_pitch`, `.burst`, `.skill`, `.cd`, and `.energy` queries in `characters/{queries.rs,source_choices.rs}`, and document Akane Rin's slug, option, and query fields in `docs/NTE_DSL.md`; verify `python tools/cargo.py test -p ntesim-content -- dsl` passes.

## 3. Research & WASM Parity (`ntesim-research`, `ntesim-wasm`)

- [x] 3.1 Update `23 -> 24` released-character and `8855 -> 10626` team-composition expectations in `crates/research/src/{search/builds/tests.rs,teams/composition/tests.rs,acceptance/runner/tests.rs,acceptance/specification/tests.rs,profiling/tests.rs}`, preserving `presets/research/recommended-gear.csv` unchanged; verify `python tools/cargo.py test -p ntesim-research` passes.
- [x] 3.2 Update released-roster parity assertions (`23 -> 24`) in `crates/wasm/src/tests.rs`; verify `python tools/cargo.py test -p ntesim-wasm` passes.

## 4. Studio Contracts, Artwork & Generated Bundles (`web/`)

- [x] 4.1 Update `web/src/domain/artwork.ts` mapping key from `'akane-rin'` to `'akane'` (`characters_akane-rin.webp`) and register `akane_hit` in `web/src/domain/contracts/trace.ts` and `web/src/domain/reports/energy-sources.ts`; verify with `node tools/web.mjs check` and `node tools/web.mjs format-check`.
- [x] 4.2 Regenerate `web/src/generated/catalog.json` (via `python tools/cargo.py run -p ntesim-cli -- studio-build-catalog`) and update `web/src/generated/names.json` with `Akane Rin` (`akane`), plus update `23 -> 24` counts in `web/tests/fixtures/resonance_parity.mjs` and `web/tests/localization.test.ts` while leaving `presets/studio/index.json`, `presets/studio/teams.md`, `openspec/specs/studio-application/spec.md`, and `web/tests/team_catalog.test.mjs` untouched; verify `node tools/web.mjs check`, `node tools/web.mjs format-check`, and `node tools/web.mjs test` pass.

## 5. Examples, References & Full Integration Gate

- [x] 5.1 Add continuing Akane Rin + `weapon:fork_prism` script(s) under `examples/` and update `23 -> 24` references in `openspec/README.md` and `openspec/specs/{data-packages,product-scope,script-execution}/reference.md`; verify `python tools/cargo.py run -p ntesim-cli -- run-script examples/<akane-script>.nte` completes without unsupported dependencies.
- [x] 5.2 Run strict OpenSpec validation (`python tools/openspec.py validate add-akane-rin --strict --no-interactive --concurrency 4`), inspect the full diff, and pass `python tools/checkpoint.py run --wasm-smoke`; verify zero skipped assertions or warnings remain.

## Workflow follow-up

- Commit coherent implementation and reference updates on a dedicated branch or commit flow per repository policy.
- Archive the completed `add-akane-rin` OpenSpec change after all checks pass cleanly (`python tools/openspec.py archive add-akane-rin`) and re-validate specs.
