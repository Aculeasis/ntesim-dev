# Film - MANISH — passive

- Native identity: `weapon:fork_jiaojuan`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

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

**Passive Name**: Film - MANISH

> Increases Break Intensity by <lv>{0}</>. Increases DMG by <lv>{1}</> against Broken units.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_oulaquantao_Unbal | buff_oulaquantao_Up |
|---|---|---|
| T1 | 48.00 | 10.00% |
| T2 | 57.00 | 12.00% |
| T3 | 66.00 | 14.00% |
| T4 | 75.00 | 16.00% |
| T5 | 84.00 | 18.00% |

## Activation and trigger mechanics

`weapons.jiaojuan_v2` supersedes the historical v1 claim that Break Intensity lacked a live content consumer. The typed refinement values above are consumed by the wearer's live Break contribution and ordinary hit modifiers:

- Add48/57/66/75/84 Break Intensity points to that living wearer. These points join `1 + total BI/300 + Break bonus`; they do not increase gauge reduction, ordinary damage, or teammates' Break contributions.
- Add10/12/14/16/18 percentage points to the ordinary damage zone only while the actual recipient carries the profile-produced Broken status. Existing damage bonuses sum with this value. It is not a final multiplier or another Break damage bonus.
- The packet depleting the gauge settles before Broken applies; later packets use the status. Recovery removes it at the half-open deadline and restores the original gauge capacity. No passive stacks, timer, switch reset or ICD are added.
- Each living team member produces its own Break contribution, including an off-field wearer. Dead sources and missing target Break profiles retain the shared eligibility rules.

## Complete implementation handoff

[The public source](../../../../examples/film-break-cycles.nte) runs three native Edgar Normal chains with Mint field transitions. The target is an authored anonymous dummy, initially at.5 of its original1-point gauge; each recovery restores the full1. No resources or state are reset between cycles. Native tests use live source scheduling, retaining two actual Edgar appear packets and the five Blossoms after the final qualified switch, separately from the18 ordinary chain packets and six Break contributions. The source runs at all five refinements through native/WASM/actual Worker, including complete reports, trial aggregation and replay. The projection now reports executable coverage for both passive terms under the disclosed models.

Rust checks normalize ATK to1000, additional general DMG to.50, crit rate tozero, and DEF/RES tozero. After the first two packets produce Break, a level-one Edgar390 packet becomes `390 *(1+.50+.10) =624` at T1 and `390 *(1+.50+.18) =655.2` at T5. The incorrect multiplicative T1 reading gives643.5. For level80 and original capacity1, the T1 wearer's Break contribution is `3603 *(1+48/300) /3 =1393.16`; T5 is1537.28. Mint's independent contribution remains1201. Neither general.50 nor Broken.10 modifies those Break packets. Three five-stage chains supply60.9 raw source energy and36.54 to the living peer; fractional carry persists through switching and recovery.

Weapon475 base ATK and secondary ATK+25% remain compiled build stats. Authors do not add them again. Native Break units/formula, action timing and release availability retain the shared combat-model limits; this source establishes the implemented passive rather than full Edgar kit coverage.
