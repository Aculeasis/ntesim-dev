# Cosmos Daze, Wild Reverie — passive

- Native identity: `weapon:fork_bopu`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 10.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | AtkUp: 12.50% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | AtkUp: 15.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | AtkUp: 17.50% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | AtkUp: 20.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | AtkUp: 22.50% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | AtkUp: 25.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **AtkUp 25.00%**.

## Refinement scaling

**Passive Name**: "Bopp"

> Increases wearer's damage dealt by <lv>{1}</> for <lv>{0}s</> after casting Support Skill. Triggers at most once every <lv>{2}s</>.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_haokewu_CD | buff_haokewu_Up | buff_haokewu_CD2 |
|---|---|---|---|
| T1 | 10.00 | 18.00% | 20.00 |
| T2 | 10.00 | 21.00% | 20.00 |
| T3 | 10.00 | 24.00% | 20.00 |
| T4 | 10.00 | 27.00% | 20.00 |
| T5 | 10.00 | 30.00% | 20.00 |

## Activation and trigger mechanics

`weapons.bopu_v1` grants one self-only general damage bonus on admitted Support
QTE casts, for 10s with an independent 20s ICD. The five native refinement
ratios are 18/21/24/27/30%. The buff adds in the ordinary damage-bonus zone,
applies before the admitted Support impact, does not stack, and remains tied
to the wearer while off field. Switching does not reset its clock or ICD.
Ordinary Skill, Burst, Normal and Critical Riposte do not trigger it. Damage
or a positive recipient is not required: an immune-target cast still reserves
the ICD and grants the buff. A rejected admission grants nothing.

The [shared continuing script](../../../../examples/bopu-support-cycles.nte)
uses legal A0 Sakiri equipment and three reaction-qualified Supports at 0, 15 and 30 seconds.
The independent 20-second weapon cooldown permits grants at 0 and 30 seconds. Rust fixes compiled ATK to 1000 and removes other
bonuses: level-1 QTE hits deal 2360/2420/2480/2540/2600 during the equipped
window at T1..T5, and 2000 at the unbuffed 15-second entry. Each uncapped
Incantation packet hits both authored dummies. Three packets grant Sakiri
32 energy with carry 7/10, and each teammate 19 with carry 31/50, independently of
two-target fanout. The prerequisite Support model is owned by
[`sakiri.support_packets_v1`](../../characters/sakiri/action-damage.md#automatic-support-bounded-executable-model).
Native/WASM/actual Worker compare all refinements, unequal purchased talents
and R3 through full reports, aggregation and replay. Existing native cases
retain exact 10s expiry/20s ICD, off-field self recipients, failed admission
and group/unit/source-binding checks. Native trigger/clock fidelity remains
a disclosed checked model rather than recovered native execution.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Cosmos Daze, Wild Reverie (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `AtkUp` contributes +25.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
