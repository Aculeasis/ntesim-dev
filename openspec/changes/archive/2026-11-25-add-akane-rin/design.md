# Design

## Context

See [proposal.md](proposal.md) and [action-damage.md](../../../../docs/mechanics/characters/akane/action-damage.md) for motivation and source evidence, and the capability specs in [characters/akane/spec.md](specs/characters/akane/spec.md) and [runtime-content/spec.md](specs/runtime-content/spec.md) for observable requirements.

In the current repository baseline:
- *Far Side of the Moon* (`weapon:fork_prism`) and Akane Rin's portrait asset (`web/public/assets/artwork/characters_akane-rin.webp`) are already committed on `main`, and `web/src/domain/artwork.ts` currently maps the hyphenated key `'akane-rin'`.
- `crates/data` exposes `23` released characters and explicitly asserts the absence of `character:1057` (`character_actions.rs`, `ability_titles.rs`, `ability_tags.rs`, `dsl/catalog.rs`), while `crates/content/src/characters/action_roles.rs` and `crates/contracts/src/hypotheses.rs` carry placeholder `1057` role hypotheses.
- `crates/content/src/simulation/reactions/remora.rs` currently writes the refreshed `Remora` expiration into `state.reactions.remora` before invoking `registry::on_remora_created`, leaving listeners unable to inspect the pre-existing `Remora` remaining duration.

## Goals / Non-Goals

**Goals:**
- Deliver deterministic, real-time execution of Akane Rin (`character:1057`) across native CLI, WASM Worker, and Studio using shared Rust core/content logic.
- Wire full cross-character interoperability (`Hathor` B2 extended `Remora`, `Chaos` B2 listeners, Esper Cycle switches) and `weapon:fork_prism` synergy without duplicating reaction or modifier pipelines.
- Synchronize `ntesim-data`, `ntesim-contracts`, `ntesim-research`, `ntesim-wasm`, and Studio TypeScript metadata (`24` released characters, `10626` 4-member team combinations).

**Non-Goals:**
- Modifying the curated 21-team Studio comparison collection (`presets/studio/index.json`, `presets/studio/teams.md`, `web/tests/team_catalog.test.mjs`, `openspec/specs/studio-application/spec.md`) or `presets/research/recommended-gear.csv`.
- Implementing non-combat overworld guitar input (`GA_Akane_Skill_Gitar`), out-of-combat `+3/s` passive `String Energy` regeneration (`Buff_Akane057_BattleState`), unused `Branch_Damage` rows, or UI-only display packet `UpgradeSkill_DisplayGE`.

## Decisions

### 1. Canonical Slug, Naming, and Artwork Alignment
- **Choice:** Register Akane Rin in `crates/data/src/constants/character/definitions.rs` with `id: 1057`, `slug: "akane"`, `script_alias: Some("Akane Rin")`, `Element::Lakshana`, `WeaponGroup::Two`, and `Base::new(1320, 80, 76, 100)`. Update `web/src/domain/artwork.ts` key from `'akane-rin'` to `'akane'` pointing to existing `characters_akane-rin.webp`.
- **Rationale:** Matches NTE DSL `identifier` rules (which disallow hyphens) and follows the existing `script_alias` pattern (`"Daffodill"`, `"Crimson"`) so `akane`, `akanerin`, and `"Akane Rin"` resolve uniformly across scripts and generated Studio bundles.
- **Alternative rejected:** Renaming the already-committed `.webp` file or weakening DSL identifier parsing to accept hyphens.

### 2. Shared `Remora` Reapplication and `Perfect Pitch` Architecture
- **Choice:** Extend `crates/content/src/simulation/reactions/remora.rs` and character registry callbacks so:
  1. `remora::apply` captures `existing_remaining_us = existing.expires_us.saturating_sub(now)` BEFORE overwriting `state.reactions.remora[target]` whenever `remora::is_active` holds for that target, and passes `existing_remaining_us` to `Remora` creation/reapplication listeners alongside existing `Hathor` B2 and `Chaos` B2 callbacks.
  2. A new `remora::apply_from_owner` helper lets Akane's active `Overtone` or `Reverb Field` emit `3.0s` periodic `Remora` ticks onto its target(s), respecting `Hathor` B2 `RemoraEnhancement`, notifying `Chaos` B2 and `Perfect Pitch`, and leaving Esper Cycle switch reaction counters intact.
  3. Once any active or cycled team member triggers `Remora` in combat, Akane's `Perfect Pitch` passive unlocks permanently (`unlocked = true`) for the rest of combat. Any subsequent `Remora` application (Cycle switch or Akane's own `3.0s` periodic tick) onto a target already carrying active `Remora` outside a `1.0s` per-target ICD fires `GE_Reaction_3_new_1057_Damage` (Cat `E`, `1.500` ATK) via `ProcCallback`, scaling `CoefAdd = ((existing_remaining_us / 1_000_000) as f64 * 0.20).min(1.60)` (`floor(rem) * 0.20`, cap `+160%` for `>= 8.0s`).
  4. Bind `GE_Reaction_3_new_1057_Damage` under `GA_Akane_Skill` in `crates/content/src/characters/action_packet_bindings.rs` so `weapon:fork_prism` Category `E` / `RedirectSkill` DMG bonuses apply cleanly without falsely invoking `on_cast`.
- **Rationale:** Capturing `remaining_us` pre-overwrite guarantees exact source fidelity (`Hathor` B2 `12s -> +160%`, non-`Hathor` `5s -> floor * 20%`), avoids false initial-application procs, and cleanly decouples Esper Cycle reaction counts from passive periodic ticks.
- **Alternative rejected:** Continuous linear remainder scaling, post-refresh sampling, or restricting `Perfect Pitch` to Esper Cycle switches only.

### 3. `String Energy`, Mode Mutuality, and `Synced Resonance`
- **Choice:** Implement `crates/content/src/characters/akane/` with dedicated state inside `RuntimeState::Akane(...)` (`crates/content/src/characters/state.rs`):
  - Track `string_energy: u8` (`0..=30`, initialized from actor config `start_string_energy` and raised to `30` on battle entry when `R3` is active), awarding per-stage gains once per multi-hit cast (`Melee1` `+1`, `Melee2` `+1`, `Melee3` `+2`, `Melee4` `+6`, `Airattack` `+1`, `PerfectEvadeAttack` `+2`, `QTE` `+4`, `Skill` `+6`, `Skill2` `+6`).
  - Enforce mutual exclusivity (`AkaneMode_RemoveOpposite`) between single-target `Overtone` (`25.0s`, `+40%`/`A5 +64%` `CoefAdd` on `N1`–`N4` and `Critical Riposte`) and `Reverb Field` AoE aura (`25.0s`, `+60%`/`A5 +90%` Skill/Hold DMG bonus) on a shared `16.0s` `Rhythm Wave` cooldown (`hold >= 0.2s` selects Hold / `Reverb Field`).
  - In multi-dummy combat with `Overtone` active, execute `N2`/`N3`/`N4` AoE hitboxes against all living dummies while restricting `Overtone`'s `CoefAdd` strictly to the `Overtone`-marked dummy (and keep `N1` targeted to the marked dummy).
  - Redirect Press Skill into `Synced Resonance` (`GA_Akane_Skill_instead`) when `String Energy == 30` and either mode is active: keep `InputActionRole::RedirectSkill` for `weapon:fork_prism`, enforce Press-only (reject `hold >= 0.2s`), use a separate `0.0s` cooldown so `Rhythm Wave`'s `16.0s` cooldown and active mode timer remain untouched, settle `30 -> 0` `String Energy` strictly on hit (`Skill.1_1` / `Display.2`) before `R6` `+10` refund, and suppress `UpgradeSkill_DisplayGE`.
- **Rationale:** Separating `Synced Resonance` execution from `Rhythm Wave` cooldown/mode timers preserves authentic rotation cadence and prevents energy waste on whiff or cancel.
- **Alternative rejected:** Sharing `Rhythm Wave`'s cooldown with `Synced Resonance` or spending `String Energy` at cast start.

### 4. Two-Stage `Sonic Boom` and Real-Time Montage Windows
- **Choice:** Model `Sonic Boom` Stage 1 (`GA_Akane_UltraSkill`, `100` Ultimate Energy, `20.0s` CD, `CINEMATIC_RELEASE = 4_012_833 us` world pause subsuming `UltraSkill1`'s `0.02s @ 0.01` `HitPause`, `10x UltraSkill1_Damage`) entering `20.0s` `Solo Cadenza` plus `20.0s` `Starlit Anthem` (`+20% ATK`, `A6` `16% DefIgnore`). Allow continued Ultimate Energy gain during `Solo Cadenza`, unlock manual Stage 2 (`GA_Akane_UltraSkill_End`, `0` cost, `0.1s` CD) in `[4.0s, 20.0s)` elapsed without overwriting Stage 1's remaining `20.0s` cooldown, and expire `Solo Cadenza` cleanly at `20.0s` without auto-casting Stage 2. Model `N2` (`19_800 us` x2) and `N3` (`19_800 us` x3 + `18_000 us` x4) hitstops plus explicit `+30_000 us` (`0.03s`) `DelayTriggerTime` on `DamageActorAkane_Melee4` and zero-flight `SoundMark` / projectile spawns.
- **Rationale:** Matches UE montage notifies, `HTGAComponent_AbilityGamePaused`, and `DamageTriggerActorDataTable` timing without inventing auto-finishers.
- **Alternative rejected:** Modeling Stage 2 as a separate ability outside `akane burst` or auto-casting Stage 2 upon `Solo Cadenza` expiration.

### 5. Affected Boundaries across Rust, WASM, Studio, and Docs
- **`crates/data`:** Add Akane definition, Lv1–Lv13 damage table (`native_damage/akane.rs`), ability titles/tags/resonance mappings, and remove pre-release absence assertions (`23 -> 24` released characters).
- **`crates/contracts`:** Add Akane `HypothesisId` variants (grounded in `docs/mechanics/characters/akane/action-damage.md`) and clean up obsolete temporary `1057` role hypotheses alongside `action_roles.rs` native tag migration (`GA_Akane_Skill`, `GA_Akane_UltraSkill` moving to `ability_tags.rs`; `GA_Akane_QTE` retained in `action_roles.rs`).
- **`crates/content`:** Add `characters/akane/`, wire `registry/{dispatch,kits,profiles,resources}.rs`, extend `RuntimeState`, `cinematics.rs`, `player_inputs.rs`, `queries.rs` (`.string`, `.mode`, `.overtone`, `.reverb`, `.cadenza`, `.anthem`, `.perfect_pitch`, `.burst`, `.skill`, `.cd`, `.energy`), `dsl/{actor,catalog}.rs` (`start_string_energy`), `callbacks/{events,handlers}.rs`, `simulation/{energy,model/energy,timer_state}.rs` (`akane_hit`), and extend `ordinary_hit_energy.rs`, `remora.rs`, `far_side_moon.rs`, and Akane kit/oracle tests.
- **`crates/research` & `crates/wasm`:** Update released-roster and 4-team composition checks (`23 -> 24`, `8855 -> 10626`), keeping `recommended-gear.csv` untouched.
- **`web/`:** Update `artwork.ts` (`'akane'`), `contracts/trace.ts`, `reports/energy-sources.ts` (`akane_hit`), regenerate `web/src/generated/{catalog.json,names.json}`, update `web/tests/{fixtures/resonance_parity.mjs,localization.test.ts}`, and keep `presets/studio/` + `web/tests/team_catalog.test.mjs` untouched.
- **`docs/` & `examples/`:** Update `docs/mechanics/characters/README.md`, `docs/mechanics/characters/akane/action-damage.md`, `docs/NTE_DSL.md`, `openspec/README.md`, `openspec/specs/{data-packages,product-scope,script-execution}/reference.md`, and add continuing Akane + `weapon:fork_prism` script(s) in `examples/`.

## Risks / Trade-offs

- **Cross-crate count and hypothesis ripple** -> Updating `23 -> 24`, `8855 -> 10626`, `action_roles.rs`, `hypotheses.rs`, and generated `web/src/generated/` bundles touches Rust and TypeScript contracts together; verify atomically with `cargo test`, `tools/web.mjs`, and `python tools/checkpoint.py run --wasm-smoke`.
- **Non-combat ability/UI table contamination** -> Raw UE exports include out-of-combat `+3/s` passive energy regeneration and UI-only `UpgradeSkill_DisplayGE`; omit both explicitly from combat simulation constants and assert their absence in unit tests.
- **Cross-character `Remora` timing drift** -> `Perfect Pitch` depends on both `Hathor` B2 duration extension and `Chaos` B2 listeners under paused/resumed world clocks; cover multi-owner `Cosmos + Lakshana + Akane` two-cycle oracle assertions directly in `crates/content`.

## Migration Plan

1. Implement constants, native damage data, and hypotheses in `crates/data` and `crates/contracts` first without shims for pre-release assertions.
2. Extend `remora.rs`, registry hooks, `characters/akane/`, energy attribution, and DSL actor/query handling in `crates/content`, verifying with focused Rust mechanics and 2-cycle oracle tests.
3. Update `crates/research`, `crates/wasm`, TypeScript contracts, and regenerate Studio bundles (`catalog.json`, `names.json`) while keeping `presets/studio/` and `presets/research/recommended-gear.csv` untouched.
4. Update documentation/examples and run `python tools/checkpoint.py run --wasm-smoke` and `python tools/openspec.py validate` before archiving on completion.
