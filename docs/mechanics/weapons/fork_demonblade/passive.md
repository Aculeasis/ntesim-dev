# Ravenous Blade — passive

- Native identity: `weapon:fork_DemonBlade`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

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

**Passive Name**: Cursed Blade - Soulbound

> Increases Crit Rate by <lv>{0}</>. Grants the wearer a <lv>{1}</> Crit DMG bonus for <lv>{2}</>s when they deal Incantation DMG. Stacks up to <lv>7</> times, with no more than 1 stack gained every <lv>0.3</>s. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_DemonBlade_Crit | buff_DemonBlade_CritDamageUp | buff_DemonBlade_CD |
|---|---|---|---|
| T1 | 16.00% | 9.00% | 15.00 |
| T2 | 20.00% | 11.25% | 15.00 |
| T3 | 24.00% | 13.50% | 15.00 |
| T4 | 28.00% | 15.75% | 15.00 |
| T5 | 32.00% | 18.00% | 15.00 |

## Activation and trigger mechanics

- **Field State & Trigger Conditions**:
  - Evaluated based on the wearer's active vs off-field state and damage delivery.
  - Parameters specify explicit cooldowns and thresholds.
- **Stacking and Refresh Behavior**:
  - Stacks are bounded by the explicit limits declared in the passive description.
  - Reapplication adheres to the identified ICD and expiration/reset rules.
- **Reset / State Transitions**:
  - Switching characters or transitioning field state triggers resets where explicitly stated in the skill description.

## Complete implementation handoff

`weapons.demon_blade_v1` supplies permanent wearer CRIT Rate and post-hit
critical-damage stacks on connected Incantation damage. The .3s ICD is shared
across the wearer's packets/recipients, including Support; simultaneous
siblings settle before their one proc. The seven-stack cap retains a
refresh-all15s lifetime, including accepted attempts at cap. Field exit keeps
the remaining lifetime; the effect does not grant critical damage to allies.

The [continuing Sakiri script](../../../../examples/ravenous-blade-critical-cycles.nte)
uses legal A0 equipment, two eleven-input native chains and a real returning
Support after the first generation expires. Rust checks the independent
.403/.138/.757/.016/.716/.331/.035/1.062 Normal coefficients and2.0 Support
coefficient at ATK1000 with the authored .25 general bonus. Critical damage is
`.50 + stacks*[.09,.1125,.135,.1575,.18]` at T1–T5, preserving exact fractional
ranks and pre-proc sibling causality. The compiled .84 CRIT Rate plus the
weapon's .16/.20/.24/.28/.32 reaches the guaranteed-hit cap; no critical
stat override is needed. An independent prefix report retains seven stacks
after switching, while the full run checks expiry, a fresh second generation,
the cap and unchanged peer packets. Native source compilation, WASM and actual
Worker compare reports, aggregation and replay at every rank. The native
Sakiri chain cadence and reset-all lifetime retain their disclosed model limits.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Ravenous Blade (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
