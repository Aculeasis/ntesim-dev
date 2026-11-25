# Spec Delta

## Purpose

Defines observable combat mechanics, resource rules, awakenings, passive `Remora` interactions, world-clock timing, and DSL query behavior for Akane Rin (`character:1057`), grounded in [action-damage.md](../../../../../../../docs/mechanics/characters/akane/action-damage.md).

## ADDED Requirements

### Requirement: Akane Rin Character Identity and Stat Progression
The simulator SHALL register Akane Rin (`character:1057`, DSL slug `akane`, script alias `Akane Rin`) as an S-Class `Lakshana` (`Liquid`) character in `WeaponGroup::Two` starting at Lv1 S0 with `1320` HP, `80` ATK, `76` DEF, `100` `ChargeMax`, `5%` Crit, and `50%` CritDMG, projecting to `15998` HP, `636` ATK, and `921` DEF at Lv80 S6.

#### Scenario: Lv1 S0 and Lv80 S6 stat projection
- **WHEN** an actor is built from `character:1057` at Lv1 S0 and Lv80 S6 without external equipment stats
- **THEN** Lv1 S0 resolves to `1320` HP, `80` ATK, `76` DEF, `100` `ChargeMax`, `5%` Crit, and `50%` CritDMG
- **AND** Lv80 S6 projects to `15998` HP, `636` ATK, and `921` DEF

#### Scenario: DSL character slug and alias resolution
- **WHEN** a script declares an actor using `akane`, `akanerin`, or `"Akane Rin"`
- **THEN** the compiler binds the actor to `character:1057` (`Akane Rin`)

### Requirement: Stringblade Normal Chain, Dodge, Plunge, and QTE
Akane Rin SHALL execute the four-stage `Stringblade` ground sequence (`N1`–`N4`), `AppearMelee`, `Stage Dive` plunge (`Airattack_Damage`), `Evade` (`15.0` stamina) and `Critical Riposte` (`PerfectEvadeAttack_Damage`), and `Altered Chord` (`QTE_Damage`), honoring exact montage hitboxes, `+30 ms` `Melee4` delay, hitstop (`19.8 ms` x2 on `N2`; `19.8 ms` x3 + `18.0 ms` x4 on `N3`), and cancel windows.

#### Scenario: Normal chain cadence and Melee4 trigger delay
- **WHEN** Akane Rin chains `N1 -> N2 -> N3 -> N4` uninterrupted on field
- **THEN** `N1` (`2` hits), `N2` (`2` hits + `39.6 ms` hitstop), `N3` (`7` total hits across `Melee3` + `Melee3_1` + `131.4 ms` hitstop), and `N4` (`1` hit at notify time `+ 30 ms` `DamageActorAkane_Melee4` delay) resolve in causal real time
- **AND** `N3` conditional sub-stage `Melee3_1` executes only when advancing through the combo continuation window

#### Scenario: Switch-out resets transient chain state
- **WHEN** Akane Rin switches off field mid-rotation
- **THEN** her active Normal combo stage and active action reset to idle while persistent cooldowns, `String Energy`, mode timers, `Solo Cadenza`, `Starlit Anthem`, and `Perfect Pitch` state persist

### Requirement: String Energy Accumulation and Cap
Akane Rin SHALL maintain `String Energy` in the bounded integer range `0..=30` (default `0`, optional initial value via `start_string_energy`), awarding `+1` on `N1`, `N2`, and `Airattack`, `+2` on `N3` and `PerfectEvadeAttack`, `+4` on `QTE`, `+6` on `N4`, `Skill`, and `Skill2`, and `0` on `UltraSkill` and `UpgradeSkill`, deduplicated per cast stage and clamped at `30`.

#### Scenario: Multi-hit stage deduplication and clamping
- **WHEN** Akane Rin lands all hits of `N1` (`2` hits), `N3` (`7` hits), and `Skill2` (`2` hits) starting from `24` `String Energy`
- **THEN** `N1` adds `+1` once (`25`), `N3` adds `+2` once (`27`), and `Skill2` adds `+6` once clamped to `30`

#### Scenario: Initial String Energy build validation
- **WHEN** an actor specifies `start_string_energy` within `0..=30` for Akane Rin
- **THEN** combat begins with that `String Energy` value unless overridden by higher battle-entry effects (`R3`)
- **AND** values outside `0..=30` fail build validation

### Requirement: Rhythm Wave Press (Overtone) and Hold (Reverb Field) Modes
`Rhythm Wave` (`GA_Akane_Skill`) SHALL share one `16.0s` cooldown across Press (`hold < 0.2s`, dealing `Skill_Damage` and applying single-target `Overtone` `SoundMark` for `25.0s`) and Hold (`hold >= 0.2s`, dealing `Skill2_Damage` x2 and creating `Reverb Field` AoE aura for `25.0s`), with activating either mode immediately removing the opposite mode.

#### Scenario: Mutual exclusivity between Overtone and Reverb Field
- **WHEN** Akane Rin activates `Rhythm Wave` Hold while `Overtone` is active
- **THEN** `Overtone` is removed immediately (`AkaneMode_RemoveOpposite`), `Reverb Field` activates for `25.0s`, and the shared `16.0s` `Rhythm Wave` cooldown starts

#### Scenario: Recasting Press restores Overtone
- **WHEN** Akane Rin activates `Rhythm Wave` Press while `Reverb Field` is active and `String Energy < 30`
- **THEN** `Reverb Field` is removed immediately and `Overtone` activates for a fresh `25.0s` duration on the primary target

### Requirement: Mode Damage Amplification and Multi-Dummy Targeting
While `Overtone` is active, Akane Rin's `N1`–`N4` and `Critical Riposte` SHALL gain `+40%` additive `CoefAdd` (`+64%` at `A5`) against the marked target, with AoE Normal hitboxes (`N2`, `N3`, `N4`) striking all living dummies while restricting `CoefAdd` to the marked dummy. While `Reverb Field` is active, her Skill and Hold attacks SHALL gain `+60%` DMG bonus (`+90%` at `A5`).

#### Scenario: Multi-dummy Overtone targeting and CoefAdd restriction
- **WHEN** Akane Rin attacks with `Overtone` placed on dummy `1` while dummy `2` is also alive
- **THEN** `N1` hits dummy `1`, while `N2`–`N4` hit both dummy `1` and dummy `2` in range
- **AND** only hits against dummy `1` receive the `+40%` (`+64%` at `A5`) `CoefAdd`

#### Scenario: Reverb Field Skill damage amplification
- **WHEN** Akane Rin casts `Rhythm Wave` Hold or `Synced Resonance` while `Reverb Field` is active
- **THEN** those Skill attacks receive `+60%` DMG bonus at base rank (`+90%` with `A5`)

### Requirement: Synced Resonance Redirect Skill and Energy Settlement
When `String Energy == 30` and either `Overtone` or `Reverb Field` is active, a Press Skill cast SHALL redirect into `Synced Resonance` (`GA_Akane_Skill_instead`) with `RedirectSkill` role and `0.0s` cooldown without disturbing `Rhythm Wave`'s `16.0s` cooldown or active mode timer. `Synced Resonance` SHALL reject Hold (`hold >= 0.2s`), emit `UpgradeSkill_Damage` only, and spend `30` `String Energy` on hit.

#### Scenario: Synced Resonance preserves mode timer and Rhythm Wave cooldown
- **WHEN** `Rhythm Wave` has `10.0s` cooldown remaining, `Overtone` has `19.0s` remaining, and `String Energy == 30`, and Akane Rin casts `akane skill`
- **THEN** `Synced Resonance` executes with `0.0s` cooldown, preserves `Rhythm Wave`'s `10.0s` remaining cooldown and `Overtone`'s `19.0s` timer, emits `UpgradeSkill_Damage` (without UI-only `UpgradeSkill_DisplayGE`), and consumes `30 -> 0` `String Energy` on hit

#### Scenario: Redirect gate, Hold rejection, and non-hit preservation
- **WHEN** `String Energy == 30` and an active mode exists, and the script attempts `akane skill hold` or switches away before the impact notify
- **THEN** `hold >= 0.2s` is rejected while redirected (`Synced Resonance` is Press-only)
- **AND** canceling or failing to land the hit leaves `String Energy` at `30`

### Requirement: Sonic Boom Two-Stage Ultimate and Solo Cadenza
`Sonic Boom` Stage 1 (`GA_Akane_UltraSkill`) SHALL cost `100` Ultimate Energy, trigger a `20.0s` cooldown and `4_012_833 us` world pause (subsuming hitpause), deal `10x UltraSkill1_Damage`, and grant `20.0s` `Solo Cadenza` and `20.0s` `Starlit Anthem` (`+20% ATK`). Between `4.0s` and `20.0s` elapsed in `Solo Cadenza`, Stage 2 (`GA_Akane_UltraSkill_End`, `0` cost, `0.1s` CD) SHALL unlock manually without resetting Stage 1's cooldown or auto-casting at expiry.

#### Scenario: Stage 1 activation, world pause, and Stage 2 window
- **WHEN** Akane Rin casts `Sonic Boom` Stage 1 at `t0` with `100` Ultimate Energy
- **THEN** world timers pause for `4_012_833 us`, `Solo Cadenza` and `Starlit Anthem` (`+20% ATK`) run for `20.0s`, and Stage 2 remains locked until `4.0s` have elapsed in `Solo Cadenza`
- **AND** Ultimate Energy continues accumulating normally during `Solo Cadenza`

#### Scenario: Manual Stage 2 cast vs Solo Cadenza natural expiry
- **WHEN** Stage 2 is cast at `6.0s` elapsed in `Solo Cadenza` vs omitted until `20.0s` expires
- **THEN** manual Stage 2 deals `UltraSkill2_Damage`, refreshes `Starlit Anthem` to `20.0s`, ends `Solo Cadenza`, and preserves Stage 1's remaining `20.0s` cooldown
- **AND** letting `Solo Cadenza` reach `20.0s` ends the state cleanly without auto-casting Stage 2

### Requirement: Perfect Pitch Permanent Unlock and Periodic Remora Tick
`Perfect Pitch` (`GA_Akane_Passive_1`) SHALL unlock permanently for the remainder of combat once an active or cycled character triggers `Remora` (`Cosmos + Lakshana`) on any target. While Akane Rin is alive (on-field or off-field) with `Perfect Pitch` unlocked and either `Overtone` or `Reverb Field` active, the active mode SHALL apply `Remora` every `3.0s` to its marked or AoE targets.

#### Scenario: Permanent unlock and 3.0s periodic tick on and off field
- **WHEN** a teammate triggers `Remora` once while Akane Rin has `Overtone` active, and Akane Rin switches off field
- **THEN** `Perfect Pitch` remains permanently unlocked for the combat even after switching out
- **AND** every `3.0s` `Overtone` applies `Remora` to the marked target (and `Reverb Field` applies `Remora` to all living targets) until Akane Rin is defeated or the mode expires

### Requirement: Perfect Pitch Reapplication Damage and Remaining-Time Scaling
When a team `Remora` application or Akane Rin's own `3.0s` periodic tick applies `Remora` to a target already carrying active `Remora` outside a `1.0s` per-target ICD, `Perfect Pitch` SHALL emit `GE_Reaction_3_new_1057_Damage` (Category `E`, base `150%` ATK). Its `CoefAdd` SHALL equal `floor(remaining_seconds) * 0.20` capped at `+160%` for `>= 8.0s`, computed from the pre-existing `Remora` remainder prior to refresh.

#### Scenario: Remaining-time scaling from existing Remora before refresh
- **WHEN** `Perfect Pitch` is unlocked and a target has `8.7s` (Hathor B2 `Remora`) or `4.6s` (standard `Remora`) remaining at the moment `Remora` is reapplied
- **THEN** `GE_Reaction_3_new_1057_Damage` samples the pre-refresh remainder, resolving `+160%` (`floor(8.7)=8 -> min(1.60, 1.60)`) and `+80%` (`floor(4.6)=4 -> 0.80`) `CoefAdd` respectively
- **AND** first application onto a target without active `Remora` deals no `Perfect Pitch` damage

#### Scenario: Per-target 1.0s ICD isolation and Category E weapon synergy
- **WHEN** two dummies receive `Remora` reapplications within `0.5s` of each other via AoE `Reverb Field` and a third reapplication hits dummy `1` at `+0.5s`
- **THEN** dummy `1` and dummy `2` each trigger `Perfect Pitch` once (`1.0s` ICD tracked per target), suppressing only the second hit on dummy `1`
- **AND** the Category `E` damage event receives `weapon:fork_prism` Redirect Skill DMG and Lakshana overlap modifiers

### Requirement: Awakening Progression (A1–A6) and Resonance Tiers (R3, R6)
Awakenings `A1`–`A6` and resonance tiers `R3`/`R6` SHALL apply their verified modifiers: `A1` team heal (`10.0 * BaseATK`) + `+50% DEF` + `60% Poise` (`15.0s`), `A2` `-12%` Lakshana RES, `A3` `+10.0` `Skill_Damage` ratio & `8.0s` pull, `A4` `+30% CritDMG` (`15.0s`), `A5` `+64%`/`+90%` modes, `A6` `16% DefIgnore` under `Starlit Anthem`, `R3` battle-entry `30` String Energy + `UpgradeSkill`/`UltraSkill2` bonuses, and `R6` `+10` refund + `+64% CritDMG` (`25.0s`).

#### Scenario: R6 String Energy settlement order and A4/R6 CritDMG stacking
- **WHEN** an `A6 R6` Akane Rin lands `Synced Resonance` on an enemy
- **THEN** `String Energy` settles `30 -> 0` on hit first and then receives `R6`'s `+10` refund (leaving `10`, or `0` if the attack fails to hit)
- **AND** `A4`'s `+30% CritDMG` (`15.0s`) and `R6`'s `+64% CritDMG` (`25.0s`) stack independently for `+94% CritDMG` while both are active

#### Scenario: A3 Press ratio boost and A6 DefIgnore lifecycle
- **WHEN** `A3` Hold or Press fires and when `A6` `Starlit Anthem` expires at `20.0s`
- **THEN** `A3`'s `+1000%` (`10.0`) fixed multiplier applies only to `Skill_Damage` (Press), while `A3` pull duration runs `8.0s`
- **AND** `A6`'s `16% DefIgnore` applies throughout `Starlit Anthem` and drops cleanly upon expiry

### Requirement: Akane Rin DSL Actions, Queries, and Combat Boundaries
The NTE DSL SHALL support Akane Rin action commands and live state queries (`.energy`, `.string`, `.mode`, `.overtone`, `.reverb`, `.cadenza`, `.anthem`, `.perfect_pitch`, `.burst`, `.skill`, `.cd`) across continuing runs, while omitting out-of-combat `+3/s` passive String Energy regeneration (`Buff_Akane057_BattleState`) and UI-only `UpgradeSkill_DisplayGE` from combat damage output.

#### Scenario: Live DSL queries and combat-only omissions
- **WHEN** a continuing combat script queries Akane Rin's state after entering combat and executing `Rhythm Wave`, `Synced Resonance`, and `Sonic Boom`
- **THEN** `.string`, `.mode`, `.overtone`, `.reverb`, `.cadenza`, `.anthem`, `.perfect_pitch`, `.burst`, `.skill`, and `.cd` return live causal values matching the active timeline
- **AND** neither out-of-combat `+3/s` `String Energy` regeneration nor `UpgradeSkill_DisplayGE` appears in traces or damage totals
