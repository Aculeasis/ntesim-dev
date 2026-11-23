# Voice of the Voyager — passive

- Native identity: `weapon:fork_GoldRecord`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

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

**Passive Name**: Golden Record

> Increases ATK by <lv>{0}</>. Increases the wearer's Support Skill Crit DMG by <lv>{1}</>. Dealing DMG with a triggered Support Skill increases their Ultimate Crit DMG by <lv>{2}</> for <lv>{3}</>s, up to <lv>3</> stacks. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_GoldRecord_AtkUp | buff_GoldRecord_QteCritDmaUp | buff_GoldRecord_QCritDmaUp | buff_GoldRecord_CD |
|---|---|---|---|---|
| T1 | 24.00% | 66.00% | 22.00% | 3.00 |
| T2 | 30.00% | 82.50% | 27.50% | 3.00 |
| T3 | 36.00% | 99.00% | 33.00% | 3.00 |
| T4 | 42.00% | 115.50% | 38.50% | 3.00 |
| T5 | 48.00% | 132.00% | 44.00% | 3.00 |

## Activation and trigger mechanics

The Rust adapter `weapons/gold_record` consumes the typed refinement parameters above. `weapons.gold_record_v2` binds the following executable policy:

- Passive ATK% scales the wearer's base ATK and remains available off field. The weapon's570 base ATK belongs in compiled base ATK; it is not another flat module bonus.
- Support Crit DMG requires an admitted Support input and native category H. The temporary Ultimate Crit DMG requires an Ultimate input and native category Q. Category-A Cat children keep their own category even when their actual parent is Support or Burst. Explicit accepted research packets without a native category retain their declared role boundary.
- Each Support delivery causing positive actual integer HP loss queues one stack. An uncapped three-dummy Support can therefore reach three stacks; no tooltip ICD is invented. This per-recipient trigger is a reasoned model, not a recovered native recipient cap. Rounded-zero damage, immunity and defeated recipients do not grant stacks.
- One source-owned instance caps at three stacks. Each trigger refreshes the full three-second real-time deadline. Switching preserves it; expiry runs before equal-time deliveries. A Cat child cannot add or refresh a Support stack.

## Complete implementation handoff

The source-bound consumer is Nanally's engine-dispatched Support. Its direct packet uses the independently purchased Support level; a surviving Underboss instead uses the summon generation's captured Burst coordinate for Cat QTE. See the [owner binding and timing limits](../../characters/nanally/action-damage.md#independent-executable-checks).

[The two-cycle source](../../../../examples/gold-record-support-cycles.nte) admits Support, Burst and ordinary attacks, refills the second Burst through actual damage, switches out and repeats without resetting state. [The Cat source](../../../../examples/nanally-cat-support.nte) deliberately gives Burst10 and Support1. Rust checks cover T1–T5 damage, three-recipient stack caps, natural expiry, resource fanout/carry and category-A exclusions. Native/WASM/actual Worker compare complete reports, aggregation and replay for both sources.

Independent numeric checks normalize base/final ATK to1000 before the passive, Crit DMG to.50 and resistance/defense tozero. T1 Support1 is `2 *1240 *(1+.50+.66) =5356.8` per eligible dummy; T5 is `2 *1480 *(1+.50+1.32) =8347.2`. Three T1 stacks give Ultimate critical factor2.16; they do not multiply the base1.5 critical factor. Cat QTE at captured Burst10 is `2.999 *1240 *1.5 =5578.14`, without the Support.66 modifier. Raw Support charge7 grants the living source7 and a peer4.2 through exact carry, once per packet rather than three times.

The two-period source uses absolute Q boundaries1.31/61.31s after Supports at
.666410/60.666410s. With the selected conservative Burst clock, first hits at
3.610192/63.610192s retain the temporary Ultimate modifier; second and later Q
packets fall beyond the three-second deadlines3.666410/63.666410s and use the
ordinary1.5 critical factor. Rust checks this per-packet expiry at every refinement.

Native switch eligibility and target caps remain declared models. This integration does not certify Nanally's complete kit or other GroupThree owners' Support dispatch.
