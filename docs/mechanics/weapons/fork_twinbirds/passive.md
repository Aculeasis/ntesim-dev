# Crime and Punishment — passive

- Native identity: `weapon:fork_twinbirds`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 9.60% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | CritBase: 12.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | CritBase: 14.40% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | CritBase: 16.80% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | CritBase: 19.20% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | CritBase: 21.60% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | CritBase: 24.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **CritBase 24.00%**.

## Refinement scaling

**Passive Name**: Twinflight

> Increases Cycle Intensity by <lv>{0}</>. Increases the wearer's Psyche DMG by <lv>{1}</>. Increases the team's Crit DMG by <lv>{2}</>, and the wearer's by an additional <lv>{3}</>, for <lv>{4}</>s after the wearer casts Ultimate. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | Buff_TwinBirds_MagUp | Buff_TwinBirds_PsycheUp | buff_TwinBirds_CritUp1 | buff_TwinBirds_CritUp3 | buff_TwinBirds_CritUp1_CD |
|---|---|---|---|---|---|
| T1 | 72.00 | 20.00% | 16.00% | 32.00% | 15.00 |
| T2 | 90.00 | 25.00% | 20.00% | 40.00% | 15.00 |
| T3 | 108.00 | 30.00% | 24.00% | 48.00% | 15.00 |
| T4 | 126.00 | 35.00% | 28.00% | 56.00% | 15.00 |
| T5 | 144.00 | 40.00% | 32.00% | 64.00% | 15.00 |

## Activation and trigger mechanics

- **Field State & Trigger Conditions**:
  - Evaluated based on the wearer's active vs off-field state and damage delivery.
  - Parameters specify explicit cooldowns and thresholds.
- **Stacking and Refresh Behavior**:
  - Stacks are bounded by the explicit limits declared in the passive description.
  - Reapplication adheres to the identified ICD and expiration/reset rules.
- **Reset / State Transitions**:
  - Switching characters or transitioning field state triggers resets where explicitly stated in the skill description.

## Complete implementation handoff

`weapons.twinbirds_v2` binds `Buff_TwinBirds_MagUp` as a permanent wearer-only `cycle_intensity` modifier, available before any cast and continuing off field while the wearer lives. Its 72/90/108/126/144 values are **stat points**, not percentages or Cycle Energy. The existing reaction snapshot sums authored CI and eligible live modifiers before comparing participants, so the weapon can change both ownership and reaction magnitude without a separate weapon branch in reaction settlement. The v1 Psyche bonus and Ultimate-triggered shared team/additional self CRIT DMG buffs retain their existing timing and refresh behavior. The native projector consumes all five parameters; it no longer reports the CI branch as unbound.

The [Rust weapon tests](../../../../crates/content/src/simulation/tests/twinbirds.rs) compile legal A0 Blackbird equipment at every refinement and create Nova before any cast. At level 80, zero resistance gives `45000*(1+CI/600)` = **50400 / 51750 / 53100 / 54450 / 55800** damage. The captured owner remains Blackbird after switching off field. T1 plus 528 authored points gives 600 CI and 90000 damage; an outgoing participant at 73 beats an incoming T1 wearer at 72, proving the bonus does not leak to the teammate. Equal 72-point participants retain the outgoing tie winner. A zero starting Cycle pool still creates no reaction. Two continuing qualified switches spend the two initial pools without duplicating the permanent modifier. Native/WASM/Worker compare all refinements with aggregation and replay. These checks establish the selected model, not native dispatch or Global release availability.

The [ordinary continuing script](../../../../examples/twinbirds-critical-cycles.nte)
also funds two Blackbird initial Bursts from zero, preserving 200 spent energy.
All five refinements check eighteen native Burst packets and Normal/Skill
continuation through the live source executor, plus Mint's attacks inside and
after both fifteen-second windows. Blackbird's form pauses off field; an
authored on-field wait reaches ordinary Skill after form expiry.
The wearer receives team plus additional self CRIT DMG; Mint receives only the
team part and no Psyche bonus. Both timed buffs expire, leaving exactly the one
permanent CI modifier. The independent T1 Burst oracle uses base ATK1000,
compiled ATK1500, Blackbird's own live team20% ATK, general damage25%, zero DEF/RES
and additional base CRIT DMG50%:
`0.467*(1500+1000*0.20)*(1+0.25+0.20)*(1+0.50+0.16+0.32)`
= **2279.2869** before rounding for each of nine packets. The general/Psyche
and team/self critical additions are separate zones. Native source execution,
native replay and independent zone assertions cover T1..T5. Complete browser
weapon parity resumes with the browser integration source-consumer migration; the scheduled
Studio adapter retains its form limitations. Existing tests
retain refresh, exact expiry and rejected admission. Native callback timing
and Blackbird's unbound kit branches remain disclosed limits.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Crime and Punishment (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
