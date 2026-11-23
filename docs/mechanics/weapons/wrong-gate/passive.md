# The Wrong Gate — passive

- Native identity: `weapon:fork_Door`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 12.00% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | AtkUp: 15.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | AtkUp: 18.00% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | AtkUp: 21.00% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | AtkUp: 24.00% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | AtkUp: 27.00% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | AtkUp: 30.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **AtkUp 30.00%**.

## Refinement scaling

**Passive Name**: Reverie Gate

> Increases ATK by <lv>{0}</>. When the wearer performs healing, increases the wearer's Anima DMG dealt by <lv>{2}</> and all allies' DMG dealt by <lv>{1}</> for <lv>{3}</>s. Resets the duration when triggered again. Effects with the same name do not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Door_Atk | buff_Door_OtherUp | buff_Door_Up | buff_Door_CD |
|---|---|---|---|---|
| T1 | 16.00% | 15.00% | 30.00% | 20.00 |
| T2 | 20.00% | 18.75% | 37.50% | 20.00 |
| T3 | 24.00% | 22.50% | 45.00% | 20.00 |
| T4 | 28.00% | 26.25% | 52.50% | 20.00 |
| T5 | 32.00% | 30.00% | 60.00% | 20.00 |

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
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 The Wrong Gate (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `AtkUp` contributes +30.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable integration boundary

`weapons.the_wrong_gate_healing_v1` retains its living-source/recipient,
positive-gross (including overheal), shared-name refresh and20s snapshot model.
Iroi Skill and the bound one-time Burst heal now use live source ATK through
`iroi.live_atk_healing_v1`, correcting the historical MaxHP basis. The weapon's
own ATK percentage contributes to healing as well as damage. Damage bonuses,
crit and target modifiers never increase the heal itself.

The [A0 continuing source](../../../../examples/wrong-gate-healing-cycles.nte)
casts twice, carries normal resources/cooldowns and switches to Mint during
each window. Near-full initial HP permits applied healing followed by overheal.
Both recipients trigger the same source identity; they do not stack another
copy of the aura. Attacks after each deadline return to their unbuffed damage.
Rust checks all five ranks with compiled/base ATK fixed at1000. The admitted
zero-delay Skill hits before its healing callback and deals1740/1800/1860/1920/1980
on its1.5-ATK Gun packet. Subsequent .760 Normals receive wearer-only Anima,
dealing1146.08/1254/1366.48/1483.52/1605.12. Mint's separate Iroi flat200 and
the weapon's allied bonus yield143.52/148.2/152.88/157.56/162.24 on its .104
packet, versus104 after both buffs expire. The source also runs in native,
WASM and actual Worker comparisons with aggregation/replay, including a
selected-A3 variant without unrelated awakening effects.

Native Skill montage/heal timing, Iroi's Dream/Moon, QTE/A2 and application-level
B4/A5 dispatch remain owner work. This integration uses the disclosed admitted
callback clock and does not claim that those missing branches are complete.
