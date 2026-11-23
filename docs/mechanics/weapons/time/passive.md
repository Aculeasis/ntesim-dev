# Marching Beyond Time — passive

- Native identity: `weapon:fork_Time`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

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

**Passive Name**: Time Beyond Time

> Increases ATK by <lv>{0}</>. Enters Wastelab and clears current Wastetime when the wearer casts a Redirect Skill. While in Wastelab, allies gain <lv>1</>stack of Wastetime each time they use a Redirect or Support Skill, up to <lv>3</> stacks. When the wearer uses their Ultimate in Wastelab, they exit Wastelab and consume all Wastetime, increasing Ultimate Crit DMG by <lv>{1}</>, plus an additional <lv>{2}</> per Wastetime consumed. Consuming <lv>3</> Wastetime stacks at once grants an extra <lv>{3}</> DEF Ignore for <lv>{4}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Time_AtkUp | buff_Time_stateCritDamageUp | buff_Time_CritDamageUp | buff_Time_DefIgnore | buff_Time_DefIgnore_Dur |
|---|---|---|---|---|---|
| T1 | 16.00% | 24.00% | 8.00% | 12.00% | 70.00 |
| T2 | 20.00% | 30.00% | 10.00% | 15.00% | 70.00 |
| T3 | 24.00% | 36.00% | 12.00% | 18.00% | 70.00 |
| T4 | 28.00% | 42.00% | 14.00% | 21.00% | 70.00 |
| T5 | 32.00% | 48.00% | 16.00% | 24.00% | 70.00 |

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

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Marching Beyond Time (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Continuing native Support cycles

The [shared script](../../../../examples/marching-beyond-time-cycles.nte) equips
legal A0 Hotori and uses Sakiri/Zero as public Support producers. The wearer
enters Wastelab at 0s. Support admissions at 1.2/3.25s plus allied Redirects at
2.05/4.1s cap Wastetime at three; the 5.55s Ultimate consumes it. Real Normal
packets refill the next 100-energy Ultimate. The second Wastelab opens at 45s;
Zero's two-packet Support at 46.2s grants one stack, and the 47.3s Ultimate
consumes one. No caller-tagged Support commands or resource resets are used.

Independent Rust checks cover T1–T5 and every packet of both ten-packet Hotori
Ultimate trains. At isolated static/base ATK1000 and baseline additional CD0.50,
T1 live ATK is1160; the first train has critical factor1.98, the second1.82.
T5 uses ATK1320 and factors2.46/2.14. Ordinary packets keep factor1.50.
Against authored DEF1080 at level80, wearer DEF-ignore12/15/18/21/24% gives
`F_def = 1 / (2 - ignore)`. The second consumption leaves the first window's
original deadline75.55s intact; an ordinary hit at that exact time uses1/2
instead. The final state preserves energy accounting and has no weapon buff.
Native/WASM/actual Worker compare full reports, aggregation and replay for every
refinement using this script.

This validates the existing `weapons.marching_beyond_time_v1` admitted-cast
model. Hotori's complete Time Stop, Katana/rewind, flower and pause behavior
remain owned by his character card; these checks do not certify those branches
or native action cadence.
