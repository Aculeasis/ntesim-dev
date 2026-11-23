# Bohemian Rose — passive

- Native identity: `weapon:fork_Rose`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

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

**Passive Name**: Bohemian Rose

> Increases ATK by <lv>{0}</>. Grants 1 stack of Chaos Thorn each time the wearer deals DoT. Increases CRIT DMG by <lv>{1}</> per stack for <lv>{2}</>s. Triggers at most <lv>once</> every <lv>0.3</>s, up to <lv>10</> stacks and refreshes the duration when retriggered. Grants <lv>10</> stacks of Chaos Thorn immediately when the wearer casts a Redirect Skill. Extends the Broken state of a Broken enemy by <lv>{3}</>s when the wearer deals damage to them (triggers at most once per Break Effect).

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Rose_AtkUp | buff_Rose_CritDamageUp | buff_Rose_CD | buff_Rose_UnbalTime |
|---|---|---|---|---|
| T1 | 14.00% | 6.00% | 3.00 | 3.00 |
| T2 | 17.50% | 7.50% | 3.00 | 3.00 |
| T3 | 21.00% | 9.00% | 3.00 | 3.00 |
| T4 | 24.50% | 10.50% | 3.00 | 3.00 |
| T5 | 28.00% | 12.00% | 3.00 | 3.00 |

## Activation and trigger mechanics

`weapons.rose_v1` supplies live base-ATK-scaled ATK% and wearer-only
Chaos Thorn critical damage. An admitted Redirect grants ten stacks before
its impact. Positive wearer DoT hits grant one stack after settlement, at
most once per 300ms across all recipients, refreshing the shared 3s expiry
and capping at ten. The first hit cannot benefit from its own later grant.
Switching preserves stacks, clocks and pending Nightmare packets.

`weapons.rose_break_extension_v1` adds 3s to an already Broken living dummy's
recovery once per Break generation. Its status deadline moves together with
the recovery event; the previous recovery becomes stale. A fresh generation
can extend again. The explicit dummy Break profile owns capacity and recovery;
no enemy catalog, geometry or automatic target stats are involved.

The [continuing script](../../../../examples/bohemian-rose-break-cycles.nte)
uses legal A0 Lacrimosa and Mint, two native Skill applications and two
anonymous dummies. Both 14-tick Nightmare trains continue off field. Each
recipient's two-second Broken window extends once to five seconds, then
recovers, independently in both cycles. Nothing resets between casts.

Independent Rust checks set compiled and base ATK1000 with no other bonuses:
live T1..T5 ATK becomes **1140/1175/1210/1245/1280**. Five Nightmare stacks
at Normal talent1 give `.06*5*ATK` = **342/352.5/363/373.5/384** noncrit;
ten Thorn stacks multiply critical damage by **1.6/1.75/1.9/2.05/2.2**.
The complete decay train checks counts `5,5,4,4,4,3,3,3,2,2,2,1,1,1`
separately for both targets and both applications, then final Thorn expiry.
Other owner checks cover first-hit grant order, shared ICD, caps, failed
admission and strict hypotheses. Native/WASM/actual Worker compare all
refinements through complete reports, aggregation and replay. The Nightmare
and Break clocks remain disclosed product hypotheses, not game measurements.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Bohemian Rose (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
