# First Step to Success — passive

- Native identity: `weapon:fork_nonos`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_BLUE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 24.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 8.00% |
| Lv 20 | 84.0 | Stage 1 | Max Lv 30 | +17.0 | AtkUp: 10.00% |
| Lv 40 | 147.0 | Stage 2 | Max Lv 40 | +35.0 | AtkUp: 12.00% |
| Lv 60 | 210.0 | Stage 3 | Max Lv 50 | +53.0 | AtkUp: 14.00% |
| Lv 80 | 273.0 | Stage 4 | Max Lv 60 | +71.0 | AtkUp: 16.00% |
| Lv 80 | 273.0 | Stage 5 | Max Lv 70 | +89.0 | AtkUp: 18.00% |
| Lv 80 | 273.0 | Stage 6 | Max Lv 80 | +107.0 | AtkUp: 20.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **380.0** (273.0 + 107.0); Secondary Stat = **AtkUp 20.00%**.

## Refinement scaling

**Passive Name**: Noros

> Increases wearer's ATK by <lv>{0}</> for <lv>{1}</>s when casting a Redirect Skill. Triggers up to once every <lv>{2}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_nonos_AtkUp | buff_nonos_CD | buff_nonos_CD2 |
|---|---|---|---|
| T1 | 12.00% | 10.00 | 20.00 |
| T2 | 13.00% | 10.00 | 20.00 |
| T3 | 14.00% | 10.00 | 20.00 |
| T4 | 15.00% | 10.00 | 20.00 |
| T5 | 16.00% | 10.00 | 20.00 |

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

`weapons.noros_v1` admits the wearer-only ATK window on a successful Redirect
cast, before its impact. The window lasts10s and the independent20s ICD starts
at that admission; field changes, unqualified casts and casts inside the ICD
do not reset it. Rejected inputs grant nothing. Buff expiry is half-open.

The [continuing Pact script](../../../../examples/first-step-pact-cycles.nte)
uses legal A0 Jiuyuan equipment and native Skill casts at0/16/32/52s. Only
0/32/52s grant the weapon window:16s is blocked and52s is exactly ready.
Two natural Seed/Bud generations continue on two dummies while Jiuyuan is off
field. Rust independently sets compiled ATK2000 and base ATK1000: T1–T5 live
ATK becomes2120/2130/2140/2150/2160 inside the window. Native .15 periodic
packets therefore deal318/319.5/321/322.5/324, then300 after expiry; all four
natural1.0ATK settlements deal2000, including the62s window endpoint. No
generation refresh or state reset is added by the intervening casts. Native
source compilation, WASM and actual Worker compare reports, aggregation and
replay at every rank. Jiuyuan's clocks/contact policies retain their disclosed
owner assumptions; the weapon check does not certify native callback timing.

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (380.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 First Step to Success (Base ATK 380.0):
    - Base ATK = 600 + 380.0 = 980.0.
    - Secondary Stat `AtkUp` contributes +20.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
