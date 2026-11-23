# Justice Executioner — passive

- Native identity: `weapon:fork_PoliceRat`.
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

**Passive Name**: Justice Executioner

> Increases ATK by <lv>{0}</>. Increases damage dealt to Bosses by <lv>{1}</>. Unlocks Arc: Officer Whisker. Arc: Officer Whisker — Summons Officer Whisker to assist in combat. Officer Whisker continuously attacks enemies, dealing <lv>{2}</> of the wearer's ATK as damage per hit. Lasts <lv>30</>s. Cooldown: <lv>{3}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_PoliceRat_AtkUp | buff_PoliceRat_Up | buff_PoliceRat_SkillDamage | buff_Rat_CD |
|---|---|---|---|---|
| T1 | 15.00% | 15.00% | 100.00% | 60.00 |
| T2 | 17.50% | 17.50% | 125.00% | 60.00 |
| T3 | 20.00% | 20.00% | 150.00% | 60.00 |
| T4 | 22.50% | 22.50% | 175.00% | 60.00 |
| T5 | 25.00% | 25.00% | 200.00% | 60.00 |

## Activation and trigger mechanics

**Bounded execution, October 24, 2026:** `weapons.police_rat_v3` retains v2's unconditional live base-ATK-scaled ATK percentage and additive Boss damage zone. Boss eligibility comes only from the dummy's authored `boss=true` flag; there is no native enemy identity, catalog or inferred Boss classification.

Explicit `haniel arc` by a living active equipped wearer starts the 30s summon and 60s cooldown. The first attack is 1s later, followed by one Physical ATK hit each second, including the exact endpoint: 30 ticks per activation, with no immediate hit. Hits read live wearer/target stats and continue off field. Source defeat stops the train and pending owner hits cancel. A second admission at exact cooldown readiness retains continuing resources and supplies a new generation and actual Arc command parent, replacing the old fixed command-zero attribution. The player-input receipt also retains the resolved Arc ability under [ADR-0012](../../../decisions/0012-admitted-arc-input-identity.md).

Each tick attacks **one** living in-range dummy, preserving the previous recipient while alive and otherwise selecting the lowest living index. No eligible target ends the train because target revival is unmodeled. Selection occurs at the summon tick; a same-time later defeat can cancel its delivery without a second selection. This deterministic single-recipient policy is an explicit interpretation of the description's per-hit attack, not certified native target acquisition or an AoE claim. No geometry is modeled. The model exposes these recipient/timing assumptions and preserves historical v2 descriptions.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Justice Executioner (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

The shared [two-cycle script](../../../../examples/officer-whisker-cycles.nte) uses a legal A0 Haniel build, explicit Boss/non-Boss dummy assumptions and an ordinary attack before Arc. Activations at 0.887510/60.887510s produce 60 off-field ticks through 30.887510/90.887510s, attributed to their respective commands 1/6. Rust sets compiled ATK and base ATK to 1000 with zero other bonuses: the unconditional ATK and follow-up coefficient give **1150,1468.75,1800,2143.75,2500** noncrit at T1..T5. Retargeting to the explicit Boss dummy gives **1322.5,1725.78125,2160,2626.09375,3125**; its Psyche immunity does not suppress Physical hits. This checks separate live ATK and additive damage zones without duplicating gear in authored stats.

An independent off-field Good Boy team window adds 10% of this recipient's 1000 base ATK to the existing 1150, plus the 6% control upgrade, producing 1310 during 5..25s rather than multiplying 1150 by 1.16. Other Rust checks cover exact-time source defeat, target defeat and cooldown continuity. Native source execution and replay retain complete two-cycle checks across all five refinements, including retargeted Boss recipients. Existing independent checks reject invalid Arc parent links. Browser transport parity is checked at the affected integration boundary.
