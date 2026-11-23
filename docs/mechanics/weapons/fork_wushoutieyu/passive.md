# Raging Flames — passive

- Native identity: `weapon:fork_wushoutieyu`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 43.0 | Stage 0 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 20 | 148.0 | Stage 1 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 40 | 259.0 | Stage 2 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 60 | 370.0 | Stage 3 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 80 | 481.0 | Stage 4 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 80 | 481.0 | Stage 5 | Max Lv 0 | +0.0 | None: 0.00% |
| Lv 80 | 481.0 | Stage 6 | Max Lv 0 | +0.0 | None: 0.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **481.0** (481.0 + 0.0); Secondary Stat = **None 0.00%**.

## Refinement scaling

**Passive Name**: Headless Rider

> Increases Lakshana DMG by <lv>{3}</>. Increases the wearer's Redirect Skill and Ultimate DMG by <lv>{0}</> for <lv>{1}</>s after casting Ultimate. Increases the wearer’s Redirect Skill DMG by <lv>{2}</> per cast while active, up to 2 stacks.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_wushoutieyu_Up | buff_wushoutieyu_CD | buff_wushoutieyu_Up2 | buff_wushoutieyu_UpLakshana |
|---|---|---|---|---|
| T1 | 10.00% | 10.00 | 5.00% | 15.00% |
| T2 | 12.50% | 10.00 | 6.00% | 17.50% |
| T3 | 15.00% | 10.00 | 7.00% | 20.00% |
| T4 | 17.50% | 10.00 | 8.00% | 22.50% |
| T5 | 20.00% | 10.00 | 9.00% | 25.00% |

## Activation and trigger mechanics

`weapons.raging_flames_v1` adds the unconditional Lakshana bonus to the
ordinary additive damage zone. An admitted wearer Ultimate opens a ten-second
E/Q damage window and clears the old Skill stacks. Each admitted Redirect
Skill within the window adds one E-only stack, capped at two; its expiry stays
the window's original deadline. Switching preserves the weapon window. At the
exact deadline both modifiers expire before damage. Packet E/Q categories
determine damage eligibility independently of the input role that triggers a
stack. Failed owner admission grants neither a stack nor a new window.

The [continuing shared script](../../../../examples/raging-flames-redirect-cycles.nte)
funds two Bursts from zero with Hathor's Normal chains and retains 200 spent
energy. Two actual zero-hold taps per period grant Power; its pool carries into
the next period. The selected hybrid Burst binds its post-recovery Emergency
window and snapshots Power4 then6, giving60% then90% of base ATK. Two completed
Cyclone inputs each start their first stage (seven contacts), costing one Power
each and building the first and second weapon stacks. This is a legal completed-
input sequence, not the early-cancel three-stage combo.

Twenty-eight native Skill contacts check all five refinements, wearer scope and
the additive zone. With1000 base/static ATK and authored25% general damage,
each level-one.429ATK Cyclone contact uses1600 ATK in the first period and1900
in the second. Its exact damage is `ATK*.429*(1.25+Lakshana+window+n*stack)`
with n=1/2. A weapon-free control retains the same intrinsic snapshot and native
timestamps, independently checking that the weapon joins the one bonus zone.
Normal packets receive only the permanent Lakshana bonus; Mint receives no
weapon bonus. Power grants8/spends4/retains4, and both weapon windows expire off
field. Native source execution, JSON request roundtrip/replay and the direct
CLI `run-script`/aggregation boundary retain the complete continuing report.
Existing boundary checks retain the third-stack cap, exact expiry, refresh,
delayed delivery and rejected Nanally recast. Cast timing and refresh-stack
reset remain disclosed product assumptions. The early-cancel Cyclone combo
remains separate Q4-L3 integration work; the script uses the same selected
native bindings as ordinary source inputs, with explicit zero-hold taps.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (481.0).
  - Secondary stat `None` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Raging Flames (Base ATK 481.0):
    - Base ATK = 600 + 481.0 = 1081.0.
    - Secondary Stat `None` contributes +0.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
