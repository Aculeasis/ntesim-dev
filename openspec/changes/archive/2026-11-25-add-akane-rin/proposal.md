## Why

Akane Rin (`character:1057`, `057_Akane`, S-Class `Lakshana` / `Liquid` WeaponGroup `Two`) already has collected combat data in [action-damage.md](../../../../docs/mechanics/characters/akane/action-damage.md), her signature weapon *Far Side of the Moon* (`weapon:fork_prism`) implemented in runtime content, and her web portrait asset already committed, while the character herself remains stubbed out of executable simulation, data catalogs, and tests. Completing Akane Rin makes her full combat kit, awakening progression, signature-weapon synergy, and `Remora`-driven passive scaling runnable and verifiable across native CLI, WASM Worker, and Studio.

## What Changes

- Register Akane Rin (`character:1057`, DSL slug `akane`, display script alias `Akane Rin`, base stats at Lv1 S0, exact Lv1–Lv13 action damage arrays, resonance skill levels, ability titles, and native ability tags) in `ntesim-data`, expanding released characters from `23` to `24`.
- Implement Akane Rin's full runtime kit in `ntesim-content`:
  - `Stringblade` (`N1`–`N4`, `AppearMelee` / `MeleePre`, `N3` multi-hit branches, `+30 ms` `Melee4` spawn delay), `Stage Dive` plunge, `Critical Riposte` / `Dodge` (`ComboSectionMap`), and `Altered Chord` QTE with `String Energy` (`0..=30`, per-stage deduplication).
  - `Rhythm Wave` Press (`Overtone` single-target mark, 25s, `+40%` / A5 `+64%` `CoefAdd`) and Hold (`Reverb Field` AoE aura, `hold >= 0.2s`, 25s, `+60%` / A5 `+90%` Skill/Hold DMG bonus, A3 pull field & `+1000%` `Skill_Damage`), mutually exclusive (`AkaneMode_RemoveOpposite`) on a shared `16.0s` cooldown.
  - `Synced Resonance` (`GA_Akane_Skill_instead`), unlocked when `String Energy == 30` and either mode is active; Press-only, `0.0s` independent cooldown (preserving `Rhythm Wave` cooldown and active mode timer), spending `30` `String Energy` only on hit, emitting `UpgradeSkill_Damage` (without the UI-only `UpgradeSkill_DisplayGE`), and carrying `RedirectSkill` role compatibility with `weapon:fork_prism`.
  - `Sonic Boom` two-stage Ultimate (`GA_Akane_UltraSkill` Stage 1 costing `100` Ultimate Energy on a `20.0s` cooldown with `4_012_833 us` `CINEMATIC_RELEASE` world pause, `10x UltraSkill1_Damage`, `20.0s` `Solo Cadenza`, and `20.0s` `Starlit Anthem` `+20% ATK`; and free Stage 2 `GA_Akane_UltraSkill_End` unlocked during `[4.0s, 20.0s)` of `Solo Cadenza`, preserving Stage 1's remaining cooldown, ending `Solo Cadenza` without auto-casting on expiry, and dealing `UltraSkill2_Damage`).
  - `Perfect Pitch` (`GA_Akane_Passive_1`) and `Starlit Anthem` (`GA_Akane_Passive_2`): permanent combat unlock on first team `Remora` application, `3.0s` periodic `Remora` tick onto the marked/AoE targets while Akane is alive with active `Overtone` or `Reverb Field` (including off-field), `1.0s` per-target ICD `GE_Reaction_3_new_1057_Damage` (Category E) triggered by any team or own periodic `Remora` reapplication onto an active-`Remora` target, and discrete `floor(remaining_seconds) * 0.20` (capped at `+160%` for `>= 8.0s`) ratio scaling sampled from the pre-existing `Remora` remainder.
  - Full `A1`–`A6` identity-gated awakenings (healing `10.0 * BaseATK` + `+50% DEF` + `60% Poise`, `-12%` `DamageResistLakshanaBase`, A3 `+10.0` `Skill_Damage` ratio & `8.0s` pull, A4 `+30% CritDMG` for `15.0s`, A5 mode upgrades, A6 `16% DefIgnore` under `Starlit Anthem`) and `R3` / `R6` count-gated tiers (`R3` battle-entry `30` `String Energy` + `+800%` `UpgradeSkill_Damage` + `+1000%` `UltraSkill2_Damage` + skill rank boosts; `R6` `+10` `String Energy` refund settled strictly after `30 -> 0` on hit and `+64% CritDMG` for `25.0s` coexisting with `A4`).
- Extend shared `Remora` reaction handling (`crates/content/src/simulation/reactions/remora.rs` and character registry hooks) so `Perfect Pitch` periodic ticks and pre-overwrite existing-duration inspection integrate cleanly alongside existing `Hathor` B2 and `Chaos` B2 listeners.
- Extend `crates/contracts` (`HypothesisId`), `crates/content` DSL build options (`start_string_energy: 0..=30`), live queries (`.energy`, `.string`, `.mode`, `.overtone`, `.reverb`, `.cadenza`, `.anthem`, `.perfect_pitch`, `.burst`, `.skill`, `.cd`), hitstop/cancel/switch-lock windows, and off-field state cleanup.
- Update `crates/research` and `crates/wasm` released-roster counts (`23 -> 24`, team compositions `8855 -> 10626`), trace/energy-source TypeScript contracts, `web/src/domain/artwork.ts` (`'akane': 'characters_akane-rin.webp'`), generated Studio catalog/name bundles, and localization/parity tests, plus new continuing examples in `examples/`.
- **Explicitly preserve unchanged:** the curated 21-team Studio comparison collection (`presets/studio/index.json`, `presets/studio/teams.md`, `openspec/specs/studio-application/spec.md`, `web/tests/team_catalog.test.mjs`) and `presets/research/recommended-gear.csv`.

## Capabilities

### New Capabilities
- `characters/akane`: Complete combat kit, state machine, awakening tiers, `String Energy`, `Overtone` / `Reverb Field` / `Synced Resonance`, two-stage `Sonic Boom`, `Perfect Pitch` `Remora` reapplication passive, montage/clock timing, off-field persistence, and live DSL queries for Akane Rin (`character:1057`).

### Modified Capabilities
- `runtime-content`: Expands released executable character definitions from `23` to `24` with Akane Rin (`akane` / `Akane Rin`), `start_string_energy` actor option, `weapon:fork_prism` character pairing, Studio catalog/artwork/trace/name synchronization (preserving the 21-team Studio comparison list), and shared `Remora` reapplication and pre-overwrite duration hooks.

## Impact

- **Rust crates:** `crates/data`, `crates/contracts`, `crates/content`, `crates/research`, `crates/wasm` (plus native `crates/cli` / unit & determinism checks).
- **Web / Studio:** `web/src/domain/artwork.ts`, `web/src/domain/contracts/trace.ts`, `web/src/domain/reports/energy-sources.ts`, generated `web/src/generated/catalog.json` & `web/src/generated/names.json`, `web/tests/fixtures/resonance_parity.mjs`, `web/tests/localization.test.ts` (`web/tests/team_catalog.test.mjs` untouched).
- **Documentation & Examples:** `docs/mechanics/characters/README.md`, `docs/mechanics/characters/akane/action-damage.md`, `docs/NTE_DSL.md`, `openspec/README.md`, `openspec/specs/{data-packages,product-scope,script-execution}/reference.md`, and `examples/`.
