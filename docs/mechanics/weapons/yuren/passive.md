# Rainman — passive

- Native identity: `weapon:fork_yuren`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 25.0 | Stage 0 | Max Lv 20 | +0.0 | DefUp: 21.00% |
| Lv 20 | 88.0 | Stage 1 | Max Lv 30 | +18.0 | DefUp: 26.25% |
| Lv 40 | 154.0 | Stage 2 | Max Lv 40 | +36.0 | DefUp: 31.50% |
| Lv 60 | 220.0 | Stage 3 | Max Lv 50 | +54.0 | DefUp: 36.75% |
| Lv 80 | 286.0 | Stage 4 | Max Lv 60 | +72.0 | DefUp: 42.00% |
| Lv 80 | 286.0 | Stage 5 | Max Lv 70 | +90.0 | DefUp: 47.25% |
| Lv 80 | 286.0 | Stage 6 | Max Lv 80 | +109.0 | DefUp: 52.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **395.0** (286.0 + 109.0); Secondary Stat = **DefUp 52.50%**.

## Refinement scaling

**Passive Name**: "Rainman"

> Increases HP by <lv>{0}</>. Strengthens shields by <lv>{1}</> while the wearer's HP is above 50%.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_bowenchanggui_shield | buff_bowenchanggui_HP |
|---|---|---|
| T1 | 10.00% | 10.00% |
| T2 | 12.00% | 12.00% |
| T3 | 14.00% | 14.00% |
| T4 | 16.00% | 16.00% |
| T5 | 18.00% | 18.00% |

## Activation and trigger mechanics

`weapons.yuren_v1` adds 10/12/14/16/18% of native base HP once in
the compiled HP-percent domain, independently of the weapon's 52.5% DEF
secondary stat. At a self shield grant, a living wearer strictly above half
of live effective MaxHP adds the same refinement fraction to the outgoing
ShieldEfficiency sum. Equality and below add none. The granted capacity is
a snapshot; later HP changes or switching do not recalculate it. A wearer
granting a shield to a foreign recipient remains explicitly unsupported.
These grant-time/recipient choices are bounded product hypotheses.

The [continuing script](../../../../examples/rainman-shield-cycles.nte)
uses legal A0 Adler/Rainman and Iroi builds, four ordinary native shield
grants, four fixed named incoming events and ordinary native healing. Each
shield follows the active party carrier after field exit. The wearer remains
active through each tested incoming event; clocks, resources and owner damage trains persist
between casts. The authored console ATK belongs only to Iroi; no weapon HP or
DEF bonus is duplicated in `add stats`.

Independent Rust checks first inspect the actual compiled build: HP is
`round(baseHP*(1+p))` for p=.10/.12/.14/.16/.18, and DEF is
`round(baseDEF*1.525)`, once each with nearest/ties-up settlement. A controlled
HP1000/DEF1000 case then starts at HP501. Native Skill base shield1800 becomes
**1980/2016/2052/2088/2124** at T1..T5. Incoming capacity+1 consumes that
shield and reaches equality500; the next grant is1800. Another1801 reaches
HP499; a third grant stays1800. Iroi's native live-ATK heal restores1000,
so the fourth grant regains the appropriate strengthened capacity. Typed
incoming receipts independently check absorbed totals, HP loss and recipient.

With an additional20% ShieldEfficiency, T1 capacity is
`1800*(1+.20+.10)` = **2340**, not2376 from multiplying zones. At equality
or below only the weapon fraction disappears, leaving **2160**. Native
compilation/WASM/actual Worker compare all refinements through full reports,
aggregation and replay. Existing checks retain odd HP, living-source gates,
exact name/placeholder binding and foreign-recipient
rejection. This establishes the disclosed model, not native-game clock or
shield consumer proof.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (395.0).
  - Secondary stat `DefUp` adds into its corresponding percentage stat zone.
  - Passive HP adds once to base-HP scaling; eligible shield strength adds in the outgoing ShieldEfficiency zone at grant time.
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Rainman (Base ATK 395.0):
    - Base ATK = 600 + 395.0 = 995.0.
    - Secondary Stat `DefUp` contributes +52.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
