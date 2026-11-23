# Worldrain — passive

- Native identity: `weapon:fork_Worldrain`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 8.80% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | CritBase: 11.00% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | CritBase: 13.20% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | CritBase: 15.40% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | CritBase: 17.60% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | CritBase: 19.80% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | CritBase: 22.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **CritBase 22.00%**.

## Refinement scaling

**Passive Name**: Passive

The consumed `upgradestar_pack_fork_Worldrain` description binds increased
wearer Cosmos Redirect Skill/Ultimate damage and wearer Cycle Intensity after
a Redirect Skill. Retriggering resets the CI duration. The earlier empty card
table did not reflect this stored description.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | Cosmos E/Q damage | Cycle Intensity points | CI duration |
|---|---|---|---|
| T1 | 30% | 36 | 15s |
| T2 | 33% | 45 | 15s |
| T3 | 36% | 54 | 15s |
| T4 | 39% | 63 | 15s |
| T5 | 42% | 72 | 15s |

The typed refinement definition and `buff_worldrain_CosmosUp`,
`buff_worldrain_Mag`, `buff_worldrain_CD` curves supply these consumed values.
CI is points, independently of the ordinary additive damage zone.

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
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Worldrain (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `CritBase` contributes +22.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable CI and continuing reactions

The native projector now binds all three parameters and validates their unit
hints. A successful wearer Redirect Skill admission starts or resets one
wearer-only CI buff before equal-time consequences. It does not stack, follows
world time through switches and expires before equal-time CI reads. Ultimate
and Support admissions do not refresh it. The existing Cosmos E/Q bonus remains
packet-classified and does not multiply reaction damage.

The [shared script](../../../../examples/worldrain-cycle-intensity.nte) equips
legal A0 Hotori and authors three outgoing100-Cycle pools. Skill at0s grants CI;
Blossom switches at2/12s capture it. Skill at12.25s resets expiry to27.25s.
The third switch at20s therefore still selects off-field Hotori as the CI owner,
after the original15s window would have ended. A companion scenario places that
switch exactly at27.25s: CI is gone, the zero-CI tie selects outgoing Iroi.
There are no resource resets; each reaction pays a different outgoing pool.

All five refinements compare real Blossom packets to an otherwise identical
zero-CI control. Independent factors are `1 + CI/600`, namely53/50,43/40,109/100,
221/200 and28/25. Earlier captured packets retain their factor after buff
expiry; a reaction admitted at the exact deadline receives none. Invalid ratios,
zero duration and deadline overflow diagnose before execution. Native/WASM/actual
Worker check complete reports, aggregation and replay for both scripts. Native
cadence and Hotori's other owner branches retain their existing model limits.
