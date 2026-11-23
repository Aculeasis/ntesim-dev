# A Time Will Come — passive

- Native identity: `weapon:fork_koinobori`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 8.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | CritBase: 10.00% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | CritBase: 12.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | CritBase: 14.00% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | CritBase: 16.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | CritBase: 18.00% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | CritBase: 20.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **CritBase 20.00%**.

## Refinement scaling

**Passive Name**: Tidal Ascension

> Grants the wearer <lv>{0}</> ATK, <lv>{1}</> DEF, and <lv>{2}</> HP when the team has at least <lv>3</> different Esper Types.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_koinobori_AtkUp | buff_koinobori_DefUp | buff_koinobori_HpMaxUp |
|---|---|---|---|
| T1 | 10.00% | 10.00% | 10.00% |
| T2 | 12.00% | 12.00% | 12.00% |
| T3 | 14.00% | 14.00% | 14.00% |
| T4 | 16.00% | 16.00% | 16.00% |
| T5 | 18.00% | 18.00% | 18.00% |

## Activation and trigger mechanics

`weapons.koinobori_v2` grants the wearer the three frozen stat percentages when the **deployed party composition** contains at least three distinct Esper types. Count distinct types, not members; duplicates do not help. Composition remains authored when an ally is defeated. Field switching neither transfers the bonuses nor creates stacks, clocks or resets.

- ATK% scales native base ATK and is available to damage and source healing queries.
- DEF% scales native base DEF and is available to damage, Adler Blessing's existing admission snapshot, and its separate live A4 overflow mitigation. It sums with other percentages and precedes flat DEF. It never applies a damage/crit/target multiplier to a shield.
- MaxHP% settles once in build projection against native base HP. Preserve authored current HP unless the existing full-HP build rule fills the new capacity. Runtime consumers must not add this already compiled HP bonus again.

The new DEF source reader shares the same gear percentage used by damage and the existing pure DEF formula; the prior shield paths had omitted this equipment contribution. Living-source timed buffs remain queried by the owner at its existing snapshot/depletion boundary. Shield state is not resnapshotted on switching. Other Adler party-interception/lifecycle branches retain the [owner card's limits](../../characters/adler/action-damage.md).

## Complete implementation handoff

[The public three-element source](../../../../examples/koinobori-shield-cycles.nte) admits two native Adler Skills and Normal chains separated by Mint field ownership. It preserves continuing HP, energy, periodic damage and cooldowns. Rust checks all five refinements, a legal two-type party, an off-field defeated ally, base-versus-compiled DEF, shield snapshots and actual A4 overflow. Native/WASM/actual Worker compare the source and supplemental explicit incoming-damage API fixtures, complete reports, aggregation and replay. The API fixtures use the existing declared incoming-damage model; no enemy damage or mitigation is inferred.

Independent normalized checks at base/final DEF1000 and Skill1: T1 gives DEF1100, Blessing `1000+.8*1100=1880` and A4 cap550; T5 gives DEF1180, Blessing1944 and A4 cap590. A delivered3000 event at T1 loses1120 HP without A4 or570 with A4. A two-type party retains shield1800 and cap500. Existing DEF+.50 and flat100 produce `1000*(1+.50+.10)+100=1700`, shield2360; multiplying compiled1600 by1.10 incorrectly gives1760. MaxHP remains base-scaled: existing1000/native base800 becomes1080 at T1 or1144 at T5, not1100/1180.

Level80 weapon475 base ATK and secondary Crit Rate20% remain compiled stats; authors do not enter them again as console stats. The deployed-party predicate and existing Adler clocks are explicit models, not full native-kit certification or a statement about Global release availability.
