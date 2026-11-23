# The Forgotten — passive

- Native identity: `weapon:fork_wuhuakuang`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

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

**Passive Name**: Picture Frame

> Increases ATK by <lv>{0}</> when the wearer’s HP is above 50%; increases DEF by <lv>{1}</> when HP is below 50%.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_kudangao_Atk | buff_kudangao_Def |
|---|---|---|
| T1 | 20.00% | 20.00% |
| T2 | 24.00% | 24.00% |
| T3 | 28.00% | 28.00% |
| T4 | 32.00% | 32.00% |
| T5 | 36.00% | 36.00% |

## Activation and trigger mechanics

`weapons.forgotten_hp_threshold_v1` samples the wearer's current HP
at damage delivery: strictly above half grants the frozen ATK%, strictly below
half grants DEF%, and exact equality grants neither. The comparison uses exact
integer cross-products, including odd MaxHP. There is no proc clock, stack,
refresh or switch reset. Live ATK% adds `base_atk*bonus` to compiled ATK;
supported DEF-scaling packets add the percentage to the DEF base layer.
Explicit incoming damage is already externally mitigated, so this model does
not recalculate that supplied amount from the wearer's DEF.

The [continuing script](../../../../examples/forgotten-hp-cycles.nte) uses
legal A0 Haniel/The Forgotten and Iroi builds, six native Normal attacks
(including the two automatic entries),
three authored incoming events and two ordinary native Skill heals. HP,
energy, buffs and cooldowns continue between the two healing cycles.
Independent Rust numbers normalize MaxHP1000 and start HP501: incoming1
reaches equality500, the next1 reaches499, a native heal fills1000, then501
damage returns499 before the second heal restores1000 again. All five
refinements retain the same strict gate.

With compiled/base ATK1000 and native Normal1 ratio.419, initial T1..T5
hits are **502.8/519.56/536.32/553.08/569.84**, versus419 at equality/below.
Iroi's Skill also supplies flat200 team ATK from her base1000 for20s; the
later high-HP T1 hit is `.419*(1000+200+200)` = **586.6**, not
`.419*(1000+200)*1.20` =603.36. The low-HP hit within that team window is
502.8, independent of refinement. The second Skill replaces its old grant;
it does not stack another200. Its live-ATK heal is `.943*1200` =1131.6 before
rounding/clamping, enough to fill this controlled MaxHP.

Native source execution and replay compare all ranks through full reports.
Browser transport parity is checked at the affected integration boundary. Existing independent Rust checks retain DEF-layer
numbers, missing base-DEF provenance rejection, odd/large HP thresholds,
delayed off-field sampling and strict hypothesis acceptance. The threshold
read and owner action/heal policies remain disclosed models, not native-game
measurements or an incoming mitigation implementation.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - The active threshold contributes ATK% or DEF% to its base-stat layer, never a general damage multiplier.
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 The Forgotten (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `HPMaxUp` contributes +25.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
