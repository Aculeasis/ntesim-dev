# Stellar Veil — passive

- Native identity: `weapon:fork_moon`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

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

**Passive Name**: Aqua Astra

> Increases the wearer's Psyche DMG by <lv>{0}</>. Increases Crit DMG dealt by <lv>{1}</> for <lv>{2}</>s when dealing Psyche DMG, up to <lv>10</> stacks. Triggers at most once every <lv>0.1</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_moon_PsycheUp | buff_moon_CritDamageUp | buff_moon_CD |
|---|---|---|---|
| T1 | 12.00% | 2.00% | 5.00 |
| T2 | 14.50% | 2.40% | 5.00 |
| T3 | 17.00% | 2.80% | 5.00 |
| T4 | 19.50% | 3.20% | 5.00 |
| T5 | 21.00% | 3.60% | 5.00 |

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

`weapons.moon_v1` retains the unconditional wearer Psyche bonus and one shared
.1s stack ICD. Connected Psyche packets grant a critical-damage stack after
their damage settles, up to10; an accepted attempt at cap refreshes the whole
5s lifetime. Same-time sibling packets share that attempt and use the pre-proc
critical damage. Field exit preserves the buff and deadline; it grants no
bonus to teammates. Expiry removes all stacks before a hit at the endpoint.

The [continuing Aurelia script](../../../../examples/stellar-veil-critical-cycles.nte)
uses legal A0 equipment and two eleven-input native chains, with a teammate
chain and natural off-field expiry between them. Rust independently checks
native .347/.630/.300/.099 coefficients, the authored .25 general damage zone
plus T1–T5 Psyche .12/.145/.17/.195/.21, and additional critical damage
`.50 + stacks*[.020,.024,.028,.032,.036]`. Both periods reach the10-stack
cap, all simultaneous siblings retain their prior critical multiplier, and
the second starts from zero. Peer packets agree with the unequipped control;
an independent prefix report retains ten stacks after field exit, and no
weapon buff survives the final6s wait. Native source compilation, WASM and
actual Worker compare full reports, aggregation and replay at every rank.
Earlier synthetic tests retain exact ICD/endpoint cases; native Aurelia chain
cadence and the reset-all lifetime are disclosed checked models.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Stellar Veil (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
