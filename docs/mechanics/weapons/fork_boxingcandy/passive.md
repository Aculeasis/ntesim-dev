# Fluff of Fortitude — passive

- Native identity: `weapon:fork_BoxingCandy`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

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

**Passive Name**: Jawbreaker's Candy

> Increases the wearer's DMG by <lv>{0}</>. Increases this effect to <lv>{1}</> against enemies with HP below 50%.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_BoxingCandy_Up | buff_BoxingCandy_Up2 |
|---|---|---|
| T1 | 22.00% | 28.00% |
| T2 | 26.00% | 34.00% |
| T3 | 30.00% | 40.00% |
| T4 | 34.00% | 46.00% |
| T5 | 38.00% | 52.00% |

## Activation and trigger mechanics

`weapons.boxing_candy_v1` selects the recipient's current HP immediately
before each wearer hit. Exactly50% uses the ordinary ratio; strictly below
half uses the enhanced ratio **instead of** the ordinary ratio. The selected
value adds once to existing general/elemental/category bonuses. There is no
cast snapshot, duration, stack or switch reset.

The [continuing source](../../../../examples/boxing-candy-hp-cycles.nte)
uses legal A0 Nanally/Mint builds and two ordinary chains separated by a
switch. Rust sets the primary to500000/1000000HP and leaves the secondary full.
At ATK1000 and authored general bonus25%, native Normal1's .296ATK gives
435.12/446.96/458.8/470.64/482.48 at T1..T5 on the initial exact-half hit.
Later primary packets use the replacement28/34/40/46/52% zone; the secondary
recipient keeps22/26/30/34/38%. Independent native coefficients check
every receipt, including all seven Normal4 pulses and the second chain.
HP and energy continue without resets. Ordinary melee and automatic Support
both reach the two living dummies, producing29 receipts per dummy while ordinary
energy counts each packet group once. Native/WASM/actual Worker compare full-HP and exact-half fixtures
at all refinements, full reports, aggregation and replay. Existing Rust
checks retain crossing, per-recipient HP, invalid ratios/units and source
bindings. Native damage/action timing remains the disclosed owner model.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Fluff of Fortitude (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
