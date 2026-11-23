# Real Music — passive

- Native identity: `weapon:fork_appliance`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_BLUE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 24.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 8.00% |
| Lv 20 | 84.0 | Stage 1 | Max Lv 30 | +17.0 | AtkUp: 10.00% |
| Lv 40 | 147.0 | Stage 2 | Max Lv 40 | +35.0 | AtkUp: 12.00% |
| Lv 60 | 210.0 | Stage 3 | Max Lv 50 | +53.0 | AtkUp: 14.00% |
| Lv 80 | 273.0 | Stage 4 | Max Lv 60 | +71.0 | AtkUp: 16.00% |
| Lv 80 | 273.0 | Stage 5 | Max Lv 70 | +89.0 | AtkUp: 18.00% |
| Lv 80 | 273.0 | Stage 6 | Max Lv 80 | +107.0 | AtkUp: 20.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **380.0** (273.0 + 107.0); Secondary Stat = **AtkUp 20.00%**.

## Refinement scaling

**Passive Name**: Haunted Record

> Grants the wearer a <lv>{0}</> Redirect Skill DMG bonus.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_appliance_Up |
|---|---|
| T1 | 12.00% |
| T2 | 13.00% |
| T3 | 14.00% |
| T4 | 15.00% |
| T5 | 16.00% |

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

`weapons.appliance_v1` grants the wearer the native category-E damage bonus.
It uses packet classification rather than the input's display name, adds in
the ordinary damage zone and has no stack, cooldown or field-entry reset.
Existing Rust checks retain unknown/missing category, duplicate configuration
and delayed off-field packet boundaries.

The [shared Mint script](../../../../examples/real-music-redirect-cycles.nte)
keeps legal A0 builds and funds two Bursts through forty continuing Normal
inputs. Two native Skills deliver eight eligible packets across two dummies.
At ATK1000, Skill1's .900 coefficient and an existing .25 general bonus give
**1233/1242/1251/1260/1269** per hit at T1–T5. The 28 native Burst packets
retain `1000*coefficient*1.25`, including their .588 claws and 3.735 slash;
casting an Ultimate does not give it Redirect damage. Native source compile,
WASM and actual Worker compare full reports, aggregation and replay at all
five ranks. Mint's timing and remaining kit branches retain their owner limits.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (380.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Real Music (Base ATK 380.0):
    - Base ATK = 600 + 380.0 = 980.0.
    - Secondary Stat `AtkUp` contributes +20.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
