# Song of the Whale — passive

- Native identity: `weapon:fork_Whale`.
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

**Passive Name**: Deep Blue Sorrow

> Increases ATK by <lv>{0}</>. Increases the wearer's damage to Broken enemies by <lv>{1}</>. Restores <lv>{2}</> HP to the wearer if a Broken enemy is defeated. Triggers at most <lv>once</> every <lv>{3}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Whale_AtkUp | buff_Whale_Up | buff_Whale_Hp | buff_Whale_CD |
|---|---|---|---|---|
| T1 | 12.00% | 12.00% | 30.00% | 30.00 |
| T2 | 14.00% | 14.00% | 30.00% | 30.00 |
| T3 | 16.00% | 16.00% | 30.00% | 30.00 |
| T4 | 18.00% | 18.00% | 30.00% | 30.00 |
| T5 | 20.00% | 20.00% | 30.00% | 30.00 |

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

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Song of the Whale (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Continuing native Broken-defeat cycles

The [shared script](../../../../examples/whale-break-defeat-cycles.nte) equips
A0 Hathor/Song of the Whale. Its four anonymous dummies declare current gauge1/2,
original capacity1 and six-second Broken duration. Connected three-packet Skills
admitted at0/30s deplete real gauges at.516245/30.732420s. Native Break HP
contributions combine with the Skill to defeat two secondary dummies in the
first period and the remaining secondary in the second. The two
simultaneous first defeats share one wearer cooldown; the second heal occurs
exactly at its30s deadline. No caller-supplied Broken tag or resource reset is
required.

Independent T1–T5 Rust checks isolate static ATK1500 and base ATK1000. The weapon
adds120/140/160/180/200 live ATK, rather than multiplying the static pool. With
talent-one Normal coefficient.243, Broken hits at2.216746/32.216746s use the same
additive12/14/16/18/20% damage zone; contacts at exact native recovery
6.516245/36.732420s lose that zone. Explicit source waits account for the native
216746us first-Normal contact without overriding its packet/recovery timing. Each
level80 Break contribution is independently `3603 * (1/3) = 1201` at DEF/RES0,
without ordinary weapon ATK/damage scaling. Four first-generation and two later
contributions retain owner/target identities and cause the expected defeats.

Native Max HP15998 and tenth-HP admission give initial HP1599. Both heals round
`15998*.30` once to4799, leaving11197 HP at every refinement. Defeats/heals occur
at.982202/30.982202s, exactly thirty seconds apart. The authored last dummy HP12500
keeps it alive through the second Break at every tier: even T5 has dealt only
12490.82 exact damage/12491 rounded HP loss by that point; T1 exceeds12500 after
the third Skill packet. A matching scenario
without gauges still defeats ordinary targets but grants no Broken bonus or
healing. Native live source and JSON request replay retain the complete reports
across all refinements. The existing unit boundary
retains live Max HP/healing modifiers, overflow, other-killer and dead-wearer
checks. This validates the disclosed `weapons.whale_v1` model; remaining public
controls/cancellation acceptance stays Q4-L3.
