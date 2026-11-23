# Fluff of Fearlessness — passive

- Native identity: `weapon:fork_BlastCandy`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 8.80% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | CritBase: 11.00% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | CritBase: 13.20% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | CritBase: 15.40% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | CritBase: 17.60% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | CritBase: 19.80% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | CritBase: 22.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **CritBase 22.00%**.

## Refinement scaling

**Passive Name**: Popping Candy

> Increases ATK by <lv>{0}</> for <lv>{1}</>s after the wearer casts an Ultimate. Effect does not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_BlastCandy_AtkUp | buff_BlastCandy_CD |
|---|---|---|
| T1 | 25.00% | 10.00 |
| T2 | 30.00% | 10.00 |
| T3 | 35.00% | 10.00 |
| T4 | 40.00% | 10.00 |
| T5 | 45.00% | 10.00 |

## Activation and trigger mechanics

The checked `weapons.blast_candy_v1` cast model grants one wearer-only ATK%
window for10s on an admitted Ultimate, before its damage. Recasting refreshes
the window without stacking; field exit preserves its deadline. Ordinary
attacks, Skills and Support do not trigger it; failed admission grants nothing.
The bonus uses base ATK, alongside existing ATK%, rather than multiplying final
compiled ATK.

The [continuing source](../../../../examples/blast-candy-burst-cycles.nte)
uses legal A0 Zero/Adler builds. Two paid Bursts spend200 total energy, with
ordinary chains funding the second and explicit waits respecting Zero's
automatic Support recovery. Rust compares every native packet against the
same continuing source with only this proc removed. Base ATK1000 and compiled
ATK2000 become2250/2300/2350/2400/2450 at T1..T5 inside each window; damage
outside the windows agrees exactly. Resources and packet clocks agree with
the control. Native/WASM/actual Worker compare complete reports, aggregation
and replay at all refinements. Existing Rust cases retain refresh, exact
expiry, self recipients, rejected casts and native source/unit checks.
Native cast timing remains the disclosed owner model.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Fluff of Fearlessness (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `CritBase` contributes +22.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
