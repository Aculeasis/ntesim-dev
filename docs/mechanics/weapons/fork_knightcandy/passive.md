# Fluff of Ferocity — passive

- Native identity: `weapon:fork_KnightCandy`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

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

**Passive Name**: Knight's Candy

> Increases wearer's CRIT DMG by <lv>{0}</> for <lv>{1}</>s after landing a critical hit, up to <lv>10</> stacks. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_KnightCandy_CritDamageUp | buff_KnightCandy_CD |
|---|---|---|
| T1 | 4.00% | 10.00 |
| T2 | 4.80% | 10.00 |
| T3 | 5.60% | 10.00 |
| T4 | 6.40% | 10.00 |
| T5 | 7.20% | 10.00 |

## Activation and trigger mechanics

`weapons.knight_candy_v1` adds one wearer-only CRIT DMG stack after each
positive connected critical hit, including owned off-field packets. Each
stack grants4/4.8/5.6/6.4/7.2 percentage points at T1..T5, capped at10.
A qualifying hit refreshes the shared10s deadline even at cap. The entire
stack group expires together; switching preserves it. Noncritical/zero/dead
recipient hits cannot grant or refresh. Same-time hits all settle before
their proc callbacks, so none benefits from stacks earned at that timestamp.

The [continuing source](../../../../examples/knight-candy-critical-cycles.nte)
uses legal A0 Adler/Iroi builds, two native Skills and two dummies. Each
train has three initial packets plus ten periodic packets per dummy, giving
52 receipts across both periods. Ordinary critical chance is100%, while
Adler's native periodic row replaces it with50%; the declared seed retains
both outcomes. Rust independently accounts for prior critical receipts,
cap and reset, checking every packet's critical multiplier as
`1.5 + prior_stacks * refinement_ratio` and actual rounded damage for its
sampled outcome. The first train builds to cap off field; the next Skill
starts after complete expiry. Native/WASM/actual Worker compare all ranks,
seeded streams, full reports, aggregation and replay. Existing Rust cases
retain exact expiry, refresh at cap, zero/noncritical exclusions and source
binding. Native callback and owner timing fidelity remain disclosed models.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Fluff of Ferocity (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
