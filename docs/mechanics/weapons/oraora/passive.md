# Oraora! — passive

- Native identity: `weapon:fork_oulaquantao`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 25.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 15.00% |
| Lv 20 | 88.0 | Stage 1 | Max Lv 30 | +18.0 | AtkUp: 18.75% |
| Lv 40 | 154.0 | Stage 2 | Max Lv 40 | +36.0 | AtkUp: 22.50% |
| Lv 60 | 220.0 | Stage 3 | Max Lv 50 | +54.0 | AtkUp: 26.25% |
| Lv 80 | 286.0 | Stage 4 | Max Lv 60 | +72.0 | AtkUp: 30.00% |
| Lv 80 | 286.0 | Stage 5 | Max Lv 70 | +90.0 | AtkUp: 33.75% |
| Lv 80 | 286.0 | Stage 6 | Max Lv 80 | +109.0 | AtkUp: 37.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **395.0** (286.0 + 109.0); Secondary Stat = **AtkUp 37.50%**.

## Refinement scaling

**Passive Name**: Ora Puncher

> Increases the wearer's Basic Attack DMG by <lv>{1}</> for <lv>{0}</>s after each Basic Attack, up to <lv>10</> stacks. Each stack lasts independently.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_wujinjieti_CD | buff_wujinjieti_Up |
|---|---|---|
| T1 | 10.00 | 2.00% |
| T2 | 10.00 | 2.40% |
| T3 | 10.00 | 2.80% |
| T4 | 10.00 | 3.20% |
| T5 | 10.00 | 3.60% |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (395.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Oraora! (Base ATK 395.0):
    - Base ATK = 600 + 395.0 = 995.0.
    - Secondary Stat `AtkUp` contributes +37.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

Continuing native cycles (`examples/oraora-puncher-cycles.nte`, `oraora_cycles.rs`): native level80 Nanally (15998 HP) and Mint (12120 HP) retain three wearer chains starting0/14/35s, two peer chains and two automatic Supports. Wearer inputs are1.5s apart; each native damage marker settles separately, including Normal5's first contact pause before its second marker. The42 wearer and16 peer ordinary packets retain exact ATK1000 coefficients. Every refinement checks the additive stack zone at2.0/2.4/2.8/3.2/3.6 percent per stack. The first chain's first ten connected packets fill the cap at5.217125s; later attempts preserve existing expiries. At14.191054s five old stacks remain; by15.709023s they have all expired, leaving the second visit's first new stack. Native per-packet damage checks independently transcribed live counts, thirty admissions with separate ten-second deadlines and twelve capped rejections. The third visit starts after all earlier expiries. Category-H Support packets at12.666410/33.666410s deal2000 and create no stacks. The unchanged independent ChargeAdd oracle settles77+2979/5000 wearer and61+1381/5000 peer energy with zero spent; every refinement retains serialized native replay.
