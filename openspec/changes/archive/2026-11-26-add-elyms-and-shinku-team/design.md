# Design

## Context

See `proposal.md` for motivation and scope. Elyms (`character:1090`, `090_elyms`) is fully specified from client binaries and localization extracts in `docs/mechanics/characters/elyms/action-damage.md`. Her signature weapon `weapon:fork_elyms` (*Echoes on Hold*) and portrait/weapon artwork (`characters_elyms.webp`, `weapons_weaponforkelyms.webp`) are already integrated in `crates/data`, `crates/content/src/weapons/echoes_on_hold/`, and `web/src/domain/artwork.ts`. Meanwhile, `openspec/specs/studio-application/spec.md` and `web/tests/team_catalog.test.mjs` currently freeze the released roster at 24 characters (`10,626` combinations) and exclude Elyms, Akane Rin, and their signature weapons from curated Team list presets.

All technical choices below follow established patterns from `character:1057` (`akane`), `character:1010` (`nanally`), and `presets/studio/teams.md` while keeping raw game data in `cache/` and typed constants in Rust.

## Goals / Non-Goals

**Goals:**
- Deliver exact, deterministic support for Elyms (`character:1090`) across `crates/data`, `crates/contracts`, `crates/content`, `crates/research`, `crates/wasm`, and `web/`.
- Generalize `ModePauseKind::Cinematic` admission in `crates/content/src/simulation/` to cover non-Ultimate skills (`Elyms` `Skill2` / `Skill3`), add an active-actor Ultimate Energy observer hook for `Charge Enhancement`, and add a non-retriggering `Remora` suppression path for `Remora Enhancement`.
- Author, validate, and publish the curated `shinku-elyms` (*Shinku · Charge / Akane / Elyms*) preset under standard Lv80 A0 `teams.md` rules with regenerated `team-list.json` and updated specs/tests.

**Non-Goals:**
- Changing or regenerating unrelated presets in `presets/studio/` (`recommended-gear.csv`, existing curated teams, or the `s5` control).
- Modeling enemy geometry, projectile flight travel time, or non-combat Exploration Passives (`GA_Elyms_Passive3`).
- Implementing undocumented client logic beyond what `action-damage.md` and extracted Blueprints/CurveTables establish.

## Decisions

### 1. Data Layer and Identity Registration (`crates/data`, `crates/contracts`)
- Register `"1090"` (`id: "character:1090"`, slug `"elyms"`, script alias `Some("Elyms")`, element `Element::Cosmos`, `WeaponGroup::One`) in `crates/data/src/constants/character/definitions.rs` and `Lv80 S6` totals (`15392` HP, `660` ATK, `909` DEF) in `values.rs` (`Lv1 S0` `1270` HP, `83` ATK, `75` DEF in `character.rs` / progression tables).
- Add `crates/data/src/native_damage/elyms.rs` populated directly from verified `DT_SkillDamageData` rows (`DamageTypeEX = Cosmos`, exact `DamageSourceCategory` `A`/`X`/`E`/`Q`/`Z`/`H`, 13-level `AtkRateBaseArray`, `ChargeAdd`, `HeterochromeAdd`, `UnbalAdd`). Notice `tables_summary.txt` displayed `Type=None` due to a summary parser mismatch, whereas `tables_extracted.json` preserves exact `DamageTypeEX` and `DamageSourceCategory`.
- Add `character_actions.rs`, `ability_titles.rs`, `ability_tags.rs`, `resonance_levels.rs`, and `native_action_metadata.rs` entries matching `GA_Elyms_Melee`, `GA_Elyms_Skill`, `GA_Elyms_UltraSkill`, `GA_Elyms_UltraSkillB`, `GA_Elyms_QTE`, and `GA_Elyms_PerfectEvade`.
- Register typed `HypothesisId` variants in `crates/contracts/src/hypotheses.rs` for Elyms's clock/cadence, skill redirect, Ultimate/Remembrance off-field coordination, Passive 1 Charge/Remora hooks, Aim cadence, Awakenings, and `ElymsOrdinaryHitEnergyV1`.

### 2. Non-Ultimate `ModePauseKind::Cinematic` Pause Admission
- **Decision:** Replace the `PrimaryActionSlot::Ultimate`-only check in `crates/content/src/simulation/admission.rs` and `pause.rs` with a packet/ability lookup (`cinematics::action_binding(character_id, ability_id, damage_effect_id)`) that returns `Release::Notify { progress_us: 1_719_655, montage: "Elyms_Skill2" | "Elyms_Skill3" }` for `GE_Player_Elyms_Skill2_Damage` and `GE_Player_Elyms_Skill3_Damage`, `None` for `GE_Player_Elyms_Skill1_Damage`, and `5_499_266 us` (`Elyms_UltraSkill` / `Elyms_UltraSkill2`) for `GA_Elyms_UltraSkill` / `GA_Elyms_UltraSkillB`.
- **Rationale / Alternative considered:** Ad-hoc pause timers inside `characters/elyms/` would duplicate `CinematicCast` deadline adjustment under HitPause (`19_800` / `29_700` / `18_000 us`), `ModeClock`, and Field Time Distribution splits. Reusing `ModePauseKind::Cinematic` keeps Relative vs Absolute time accounting unified across Studio and CLI.

### 3. Selective Skill Cooldown Bypass & Redirect (`GA_Elyms_Skill`)
- **Decision:** Mirror the extracted Blueprint state machine (`Con_Elyms_NotHaveAuthorityBuff`, `Buff_Elyms_PowerE1Dur`, `Buff_Elyms_PowerE2Dur`, `K2_ShouldIgnoreCooldown`):
  - Combat start initializes Elyms at `Weak` Connection with `power_e1_until_us = 12_000_000` (on `ClockDomain::World` / relative clock).
  - `Skill2` (*Intermittent Existence*) executes if `power_e1_until_us` is active and Connection is `Weak`, clearing `power_e1_until_us`, ignoring and preserving the `16.0s` Skill cooldown, advancing `Weak -> Medium` (`-> Strong` at `R3`) at `0.605117s`, and granting `Optical Illusion`.
  - `Skill3` (*Goodbye to You*) executes if `power_e2_until_us` (`6.0s` window from `Crosstalk` stage 3 `AddE` at `0.617396s`, or immediately via `R3`) is active at `>= Medium`, ignoring/preserving the `16.0s` Skill cooldown, consuming all active `Optical Illusion` stacks (`+100%` `CoefModify` per stack), accumulating `total_consumed_stacks` (capped at `4`), and advancing `Medium -> Strong`.
  - Otherwise `skill` executes `Skill1` (*The Ones Who Left*), enforcing and starting the `16.0s` Skill cooldown, triggering `Buff_Elyms_QRecoverEnergy` (`+100` Cycle via `actor.grant_cycle` / Cycle ledger), and granting `12.0s` `PowerE1Dur` at `0.818769s` only if Connection is still `Weak`.

### 4. `Do You Get Deja Vu`, `AtkBase` Team Buff, and `On the Line` / `Remembrance`
- **Decision:** On `burst` commitment (`100` Energy, `20.0s` cooldown), snapshot Connection Strength (`+0%` Weak, `+50%` Med, `+100%` Strong DMG Ratio `CoefModify`) and select ST `GA_Elyms_UltraSkillB` if living targets equal `1` and Connection `>= Medium`, else AoE `GA_Elyms_UltraSkill`. Reset Connection to `Weak`. Convert `total_consumed_stacks` into A1 (`+10% * n` team Crit DMG, `20s`) and A6 (`+25% * n` Elyms Follow-up DMG, `20s`) while clearing `total_consumed_stacks` immediately so `Do You Get Deja Vu` itself receives `A1`. Apply `Buff_Elyms_QAtkBuff` (`0.20 * (char_base_atk + weapon_base_atk)` flat `AtkAdd` plus `+10%` Follow-up Attack DMG for `20.0s`) to the whole living team including Elyms.
- **On the Line (`3.0s` x `6`):** Arm `6` triggers bound to the Ultimate cast's AoE/ST mode and Connection snapshot. Trigger `1` fires at `swap_off_field + 0s`, scheduling subsequent triggers every `3.0s` on relative world time while off-field; switching Elyms back on-field suspends the countdown with remaining time preserved. Each trigger schedules 4 hits (`0.12s` apart from `0.254680s` AoE / `0.326896s` ST) tagged with `FOLLOW_UP_DAMAGE_TAG` (`State.Damage.Followup`) in `packet_semantics.rs` under `GA_Elyms_UltraSkill` so `weapon:fork_elyms` (*Echoes on Hold*) counts them automatically.

### 5. Passive 1 (`Relay Node`), `Aim`, Energy Attribution, and Awakenings
- **Charge Enhancement:** Add an active-actor energy-gain hook invoked at combat energy-grant points (`simulation/energy.rs`, `simulation/reactions/remora.rs`, and kit/weapon grants to the currently active actor, excluding `t = 0` initial starting energy) whenever a positive energy grant targets `state.active_actor_index` while Elyms is alive (`hp > 0`). Grants `1` stack of `+3.75%` team Cosmos DMG (`30.0s`, max `8` stacks, refreshed on each trigger).
- **Remora Enhancement:** Extend `simulation/reactions/remora.rs` with `apply_from_owner_with_retrigger_control(..., allow_elyms_retrigger: bool)`. Ally `Remora` creation checks `state.actors` for living Elyms (`hp > 0`); outside her global `7.0s` `ClockDomain::World` ICD across all targets, she schedules a `7.5s` delayed `ContentEvent::ElymsRemoraEnhancement` (`apply_from_owner(..., allow_elyms_retrigger: false)`), skipping cleanly if Elyms or the target dies before firing while cooperating with Akane `Perfect Pitch` and weapon/set hooks.
- **Aim & Energy:** Bind `character:1090 -> "GE_Player_Elyms_Aim_Damage"` in `player_inputs.rs` using the `0.146353s` (`146_353 us`) `BP_CanBreakSkill_C` `AnySkill`/`Jump` cancel cadence (`Branch.7` hit at `48_313 us`), matching Nanally `Kind::Aim`. Register `"character:1090"` with `GameEnergySource::ElymsHit` (`elyms_hit`) in `simulation/energy.rs`, `model/energy.rs`, and `web/src/domain/{contracts/trace.ts,reports/energy-sources.ts}`.
- **Awakenings & Resonances:** Gate `Passive 2` (`CritDamageBase += 0.10` at S4+ / modeled active builds), `A1`–`A6` (`Con_Elyms_Lv1`..`Lv6`), `R3` (`+1` levels and `Weak -> Strong` skip with immediate `PowerE2Dur` unlock), and `R6` (`12%` team All-RES Ignore while living + `GE_Player_Elyms_Skill3Extra_Damage` `CoefModify = (consumed_stacks - 1)` on final Skill3 hit). Note: A3 increases Optical Illusion cap `1 -> 4` (+3 on `Medium -> Strong`), whereas R3 without A3 still caps Optical Illusion at `1`.

### 6. Roster Count Update and `Shinku-Akane-Elyms-Iroi` Curated Team
- Update `24 -> 25` released characters and `10626 -> 12650` four-member combinations across `crates/data/src/stat_projection.rs`, `crates/research/src/**/tests.rs`, `crates/wasm/src/tests.rs`, `web/tests/localization.test.ts`, and `web/tests/fixtures/resonance_parity.mjs`, and regenerate `web/src/generated/{catalog.json,names.json}`.
- **Curated preset (`presets/studio/shinku-elyms.nte`):**
  - Selected Lv80 A0 limited baseline (`teams.md` rules: `lvl=80/80`, `talent=8,8,8`, `qte=8`, `refine=1`, `awakening=[]`, 15 useful lines, max 1 `speedyhedgehog` set):
    - **Shinku:** `fork_lunarphase` (*Blushing Mirage* R1), `lostradiance` 4pc, `crit` + `cd:.6` + `cd=0.64` (`hp=5600 atk=420 cr=0.28 cd=1.80 dmg%=0.1`).
    - **Akane Rin:** `fork_prism` (*Far Side of the Moon* R1), `streetboxer` 4pc, `crit` + `cd:.6` + `atk%=0.4` (`hp=5600 atk=420 cr=0.28 cd=1.16 dmg%=0.1 atk%=0.4`).
    - **Elyms:** `fork_elyms` (*Echoes on Hold* R1), `lostradiance` 4pc, `crit` + `cd:.6` + specialization (`hp=5600 atk=420 cr=0.28 cd=1.16 dmg%=0.1` + verified `EquipmentSlots_elyms` if extractable during apply or generic offensive `atk%=0.4` disclosed in `teams.md`).
    - **Iroi:** `fork_door` (*The Wrong Gate* R1), `speedyhedgehog` 4pc (the single `Suit11` set), `support` + `atk%:.375` + `atk%=0.4` (`hp=5600 atk=420 atk%=1.1250 cr=0.28 dmg%=0.1`).
  - **Element-aware rotation structure:** Align switches with the Cosmos / Lakshana / Nature Esper Cycle reaction triangle: `Elyms <-> Akane` (`Cosmos <-> Lakshana`) generates **Remora** (triggering Elyms *Remora Enhancement* `+7.5s` delayed Remora and Akane *Perfect Pitch*), while `Iroi <-> Shinku` / `Elyms <-> Iroi` (`Nature <-> Cosmos`) generates **Blossom** (detonating **Charge** onto Remora targets and fueling Elyms *Charge Enhancement* + Shinku Burst). Opener builds Elyms Connection/Ult (`Skill2 -> Skill1 (+100 Cycle) -> Akane swap (Remora) / Akane Burst+Skill -> Elyms Crosstalk B1-B3 -> Skill3 -> Burst`), triggers Iroi Skill/Burst for `Speedy Hedgehog` + `The Wrong Gate` + `Blossom`, and delivers Shinku's extended `Surging Crimson` carry window while Elyms `On the Line` and Akane `Overtone`/`Reverb` fire off-field.
  - Register `shinku-elyms` in `presets/studio/index.json` (total `22` curated compositions plus `s5`), document in `presets/studio/teams.md` and `presets/studio/README.md`, remove the temporary Elyms/Akane exclusion in `web/tests/team_catalog.test.mjs`, rebuild WASM, and regenerate `web/src/generated/team-list.json` (`node tools/web.mjs teams`).

## Risks / Trade-offs

- **Skill2/Skill3 `ModePauseKind::Cinematic` refactor touching shared pause path** → Isolate routing cleanly in `cinematics.rs`/`admission.rs` with zero behavioral change to existing Ultimate pauses (`Shinku` Pre-Rage, `Akane` UltraSkill2) and verify full native simulation test suite.
- **`EquipmentSlots_elyms` specialization table absent from initial `elyms-current` export** → Inspect `DT_CharacterEquipmentSlotsData` if present in `cache/` during implementation; otherwise document the standard offensive Lv80 specialization assumption explicitly alongside `shinku-elyms` in `presets/studio/teams.md` (since console stats always enter via `add stats`).
- **Energy hook firing on every active-actor grant (`Charge Enhancement`)** → Route through a single simulation observer called after `grant_energy` / `grant_fractional_energy` / `restore_energy` in combat handlers (never initial `start_energy`) and cover `ChargeAdd`, `Charge` reaction, and `Buff_Elyms_QRecoverEnergy` / kit fills in unit tests.
- **Team rotation timing across 1,000 random trials (`seed=42`, `duration=120`)** → Verify `shinku-elyms` completes all 1,000 trials without `INSUFFICIENT_CYCLE`, stalled cooldowns, or unsupported dependency errors using `ntesim-wasm` and `node tools/web.mjs teams-check` before finalizing.
