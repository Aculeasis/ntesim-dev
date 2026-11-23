# Blushing Mirage — passive

- Native identity: `weapon:fork_LunarPhase`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

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

**Passive Name**: Moonphase Shift

> Increases ATK by <lv>{0}</>.  After casting Ultimate, increases the wearer's Cosmos DMG dealt by <lv>{1}</> and allows the wearer to ignore <lv>{2}</> of enemy DEF for <lv>{3}</>s. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_LunarPhase_Atk | buff_LunarPhase_Up | buff_LunarPhase_DefIgnore | buff_LunarPhase_CD |
|---|---|---|---|---|
| T1 | 20.00% | 32.00% | 12.00% | 20.00 |
| T2 | 23.00% | 36.80% | 13.80% | 20.00 |
| T3 | 26.00% | 41.60% | 15.60% | 20.00 |
| T4 | 29.00% | 46.40% | 17.40% | 20.00 |
| T5 | 32.00% | 51.20% | 19.20% | 20.00 |

## Activation and trigger mechanics

`weapons.lunarphase_v1` supplies permanent wearer ATK% and a twenty-second
wearer Cosmos-damage/DEF-ignore window after an admitted Ultimate. Recasting
resets the single window; switching preserves it. Exact-time expiry precedes
damage. ATK% adds `base_atk*p` to compiled ATK; Cosmos adds to the ordinary
damage zone, while DEF ignore reduces effective defense separately. Other
elements retain DEF ignore without receiving the Cosmos bonus. Failed
admission grants no window, and the effect does not buff teammates.

The [continuing legal Shinku script](../../../../examples/blushing-mirage-cosmos-cycles.nte)
funds two initial Bursts from zero under her disclosed public cost/timing model,
preserves 200 spent energy and lets both weapon windows expire. Rust checks
all five refinements packet by packet, including Shinku's separate thirteen-second
Surging Crimson damage zone and Mint's unchanged damage. With base ATK1000,
compiled ATK1500, authored general damage25%, DEF1080 and zero resistance,
T1 adds200 ATK: the first level-one Normal packet is **193.375** noncrit;
the initial Burst primary is exactly **117623/376**, from
`0.185*1700*(1+0.25+0.30+0.32)/(2-0.12)`.
CLI compilation, native/WASM and actual Worker compare complete reports,
aggregation and replay at T1..T5. Existing boundary tests retain refresh,
exact expiry, non-Cosmos packets and failed admission. These checks establish
the stated product model; native buff callback/cancel timing and Rage follow-up
dispatch remain outside this execution slice.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Blushing Mirage (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
