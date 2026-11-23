# Fluff of Fleetness — passive

- Native identity: `weapon:fork_MotorCandy`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | CritDamageBase: 17.60% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | CritDamageBase: 22.00% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | CritDamageBase: 26.40% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | CritDamageBase: 30.80% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | CritDamageBase: 35.20% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | CritDamageBase: 39.60% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | CritDamageBase: 44.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **CritDamageBase 44.00%**.

## Refinement scaling

**Passive Name**: Motor Candy

> Increases <lv>{1}</> ATK every <lv>{0}</>s while the wearer is the active character, up to <lv>5</> stacks. Resets when the wearer leaves the field.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_MotorCandy_CD | buff_MotorCandy_AtkUp |
|---|---|---|
| T1 | 1.00 | 5.00% |
| T2 | 1.00 | 6.00% |
| T3 | 1.00 | 7.00% |
| T4 | 1.00 | 8.00% |
| T5 | 1.00 | 9.00% |

## Activation and trigger mechanics

`weapons.motor_candy_v1` starts a real-time field clock at initial active
placement or actual field entry. Every1s adds one wearer-only ATK% stack,
capped at5; no further ticks are needed at cap. Field departure removes all
stacks and invalidates the old timer generation. Re-entry starts from zero
and cannot inherit a stale tick. Same-actor swaps and rejected switches do
not reset the clock. The living-source buff adds each refinement ratio
against base ATK, alongside existing ATK%, rather than multiplying compiled
ATK. Idle time advances it without readiness polling.

The [continuing source](../../../../examples/motor-candy-field-cycles.nte)
uses legal A0 Mint/Adler builds and two field periods. Rust fixes base
ATK1000 and compiled ATK2000 and compares every native packet with the same
source without this proc. Each stack adds50/60/70/80/90ATK at T1..T5, giving
cap ATK2250/2300/2350/2400/2450. Both entries emit exactly five one-second
ticks; Normal and Skill packets exercise zero, one and capped stacks. The
second period rebuilds from zero while cooldowns/resources continue, peer
damage stays identical, and the final departure leaves no weapon buff.
Native/WASM/actual Worker compare all refinements, complete reports,
aggregation and replay. Existing Rust cases retain equal-time ticks,
initially off-field carriers, stale-generation cancellation, delayed hits,
same-actor/rejected switches and source binding. Native timing and lifecycle
fidelity remain the disclosed checked model.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `CritDamageBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Fluff of Fleetness (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `CritDamageBase` contributes +44.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
