# The Good, The Bad, The Bitter — passive

- Native identity: `weapon:fork_BitterCake`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | HPMaxUp: 10.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | HPMaxUp: 12.50% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | HPMaxUp: 15.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | HPMaxUp: 17.50% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | HPMaxUp: 20.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | HPMaxUp: 22.50% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | HPMaxUp: 25.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **HPMaxUp 25.00%**.

## Refinement scaling

**Passive Name**: "Bittercake"

> Increases wearer's DEF by <lv>{0}</> for <lv>{1}</>s when taking damage. Triggers at most once every <lv>{2}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_BitterCake_DefUp | buff_BitterCake_CD | buff_BitterCake_CD2 |
|---|---|---|---|
| T1 | 26.00% | 10.00 | 20.00 |
| T2 | 31.00% | 10.00 | 20.00 |
| T3 | 36.00% | 10.00 | 20.00 |
| T4 | 41.00% | 10.00 | 20.00 |
| T5 | 46.00% | 10.00 | 20.00 |

## Activation and trigger mechanics

`weapons.bittercake_damage_event_v1` consumes actual positive wearer HP loss
after shared shield/effect absorption. It grants wearer-only DEF for10s with
an independent20s ICD. Zero damage, fully absorbed damage and another actor's
HP loss do not qualify. Switching preserves both clocks. DEF adds the
refinement percentage of base DEF once, alongside authored percentages and
flat DEF; it does not multiply already compiled DEF. Incoming amounts are
explicit delivered-damage assumptions, so this buff does not retroactively
mitigate the triggering amount.

The [continuing source](../../../../examples/bittercake-defense-cycles.nte)
uses legal A0 Adler/Iroi builds and three native Skills, with no state resets.
Rust independently fixes base DEF1000 plus flat240. At T1..T5, active-window
DEF is1500/1550/1600/1650/1700; native level-one Skill damage is
1050/1085/1120/1155/1190, while Blessing snapshots
`1000 + .8*DEF` =2200/2240/2280/2320/2360. After expiry the same inputs give
DEF1240, damage868 and shield1992. Actual absorption checks those capacities;
HP losses at0 and20s grant exactly two windows, the19s loss remains inside
the original ICD, and absorbed hits do not refresh it. Adler remains active
through the tested absorptions; subsequent field exit transfers any surviving
party shield instead of leaving recipient-bound protection on Adler. Off-field native
ticks, wearer HP and incoming callbacks continue through the swaps.
Native/WASM/actual Worker compare all five refinements, full reports,
aggregation and replay. Existing Rust cases retain exact expiry, failed
inputs, source units, recipient/death and declared-damage limits. The trigger
and clock interpretation is a checked product model, not recovered native
execution.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 The Good, The Bad, The Bitter (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `HPMaxUp` contributes +25.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
