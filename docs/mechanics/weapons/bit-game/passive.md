# Blow up the Crowd — passive

- Native identity: `weapon:fork_BitGame`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 11.00% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | AtkUp: 13.75% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | AtkUp: 16.50% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | AtkUp: 19.25% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | AtkUp: 22.00% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | AtkUp: 24.75% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | AtkUp: 27.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **AtkUp 27.50%**.

## Refinement scaling

**Passive Name**: Beat King

> Increases the active character's ATK by <lv>{0}</> while the wearer is off-field, and increases the active character's ATK by <lv>{1}</> when the wearer deals damage, up to <lv>4</> stacks and triggering at most <lv>once</> every <lv>{2}</>s. Resets the effect when the wearer becomes the active character. Increases Psyche DMG by <lv>{3}</> while the wearer is the active character, and increases Psyche DMG by <lv>{4}</> when dealing Psyche DMG with a Basic Attack, up to <lv>10</> stacks. Triggers at most once every <lv>0.3</>s. Resets the effect when switching characters.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_BitGame2_AtkUp1 | buff_BitGame2_AtkUp2 | buff_BitGame2_CD | buff_BitGame2_DamageUpPsycheBase1 | buff_BitGame2_DamageUpPsycheBase2 |
|---|---|---|---|---|---|
| T1 | 10.00% | 2.00% | 2.00 | 12.00% | 2.00% |
| T2 | 11.00% | 2.30% | 2.00 | 14.00% | 2.30% |
| T3 | 12.00% | 2.60% | 2.00 | 16.00% | 2.60% |
| T4 | 13.00% | 2.90% | 2.00 | 18.00% | 2.90% |
| T5 | 14.00% | 3.20% | 2.00 | 20.00% | 3.20% |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Blow up the Crowd (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable integration boundary

`weapons.bit_game_v1` remains the accepted weapon rule. Native Rust checks all
five ranks through [two continuing Hootie cycles](../../../../examples/bitgame-hootie-cycles.nte):
eight active Mint inputs produce two Ensemble packets per speaker, each packet
hits both dummies but grants only one stack through the shared 2s ICD, and
re-entering Haniel clears the earlier stacks. Energy and cooldowns continue.
With both base and compiled ATKs fixed at1000, Hootie grants flat100: the first
Mint .104 packet deals124.8/125.84/126.88/127.92/128.96 across T1–T5 against
DEF0/RES0, and each off-field .20 Ensemble deals220 per recipient. These are
independent Rust oracles. The fixture executes live source with Haniel's full
native clocks and reconciles the complete report with direct native replay;
current browser source-consumer/Worker parity remains browser integration.

[Haniel Ace](../../../../examples/haniel-ace-ensemble.nte) additionally exercises
on-field Psyche bonuses and Normal stacks, A4 coefficient doubling and owned
Symphony/extra packets. These three summon packets are not Normal packets.
Haniel's source projection now selects her exact montage/Cannon/Nova models;
this weapon fixture keeps its focused Hootie/field-exit scope and uses no
synthetic off-field hit. Remaining source parry/cancellation and browser
integration retain their distinct affected-boundary checks.
