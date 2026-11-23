# Us. — passive

- Native identity: `weapon:fork_Prokaryon`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_BLUE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

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

**Passive Name**: Protocore

> Increases the wearer's Basic Attack damage by <lv>{0}</>.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Prokaryon_Up |
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

`weapons.prokaryon_v1` adds the refinement bonus only to the wearer's native
category-A packets. The element and initiating action do not replace that
packet classification. There is no activation clock, stack or field reset.
Unknown/missing categories and duplicate configured passives diagnose.

The [continuing Nanally script](../../../../examples/prokaryon-normal-cycles.nte)
uses legal A0 equipment and two ordinary five-stage chains separated by a
teammate's chain and a real automatic Support. Rust fixes ATK1000 and other
elemental/critical bonuses to zero: its56 Normal receipts (28 per dummy) use their independent
native coefficients times `1.25 + [.12,.13,.14,.15,.16]` at T1–T5. Both uncapped
Support recipients retain `2000*1.25=2500`; the weapon does not turn QTE into
Basic Attack damage. Resources continue through the field changes. Native
source compilation, WASM and actual Worker compare complete reports,
aggregation and replay across all ranks. Owner timing/selection retain the
disclosed Nanally model; this does not certify the remaining kit branches.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (380.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Us. (Base ATK 380.0):
    - Base ATK = 600 + 380.0 = 980.0.
    - Secondary Stat `AtkUp` contributes +20.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
