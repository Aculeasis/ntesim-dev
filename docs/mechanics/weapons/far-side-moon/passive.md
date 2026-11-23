# Far Side of the Moon — passive

- Native identity: `weapon:fork_prism`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | CritDamageBase: 19.20% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | CritDamageBase: 24.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | CritDamageBase: 28.80% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | CritDamageBase: 33.60% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | CritDamageBase: 38.40% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | CritDamageBase: 43.20% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | CritDamageBase: 48.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **CritDamageBase 48.00%**.

## Refinement scaling

**Passive Name**: Gravitide

> Increases Crit Rate by <lv>{0}</>. Increases the wearer's Redirect Skill DMG by <lv>{1}</>. Increases the wearer's Basic Attack DMG by <lv>{2}</> and Lakshana DMG by <lv>{3}</> for <lv>{4}</>s after casting a Redirect Skill.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Prism_CritUp | buff_Prism_DamageUp1 | buff_Prism_DamageUp2 | buff_Prism_LakshanaUp | buff_Prism_DamageUp2_CD |
|---|---|---|---|---|---|
| T1 | 16.00% | 28.00% | 28.00% | 16.00% | 15.00 |
| T2 | 20.00% | 35.00% | 35.00% | 20.00% | 15.00 |
| T3 | 24.00% | 42.00% | 42.00% | 24.00% | 15.00 |
| T4 | 28.00% | 49.00% | 49.00% | 28.00% | 15.00 |
| T5 | 32.00% | 56.00% | 56.00% | 32.00% | 15.00 |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritDamageBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Far Side of the Moon (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritDamageBase` contributes +48.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

Continuing native cycles (`examples/prism-gravitide-cycles.nte`, `prism_cycles.rs`): a native lvl80 Daffodil wearer (16180 HP, the group-compatible `CHARACTER_GROUP_TYPE_TWO` owner) and a Mint peer (12120 HP) execute the live source with native clocks: Redirect Skill casts at0s and17s respect the notify-started16s cooldown, with two full chains and a two-input visit at static ATK1000, plus two Mint chains. Every tier settles all 57 wearer and 16 peer chain packets against exact native ATK-1000 coefficients (wearer skills 409/337/498, chain 167/153x2/132x2/62x2/141/263/524/193/127/101x4/101x2/473, peer 104/231/170/262/341/184/255/523): the eight category-E Skill packets carry the equipped-static Redirect Skill zone 28/35/42/49/56 percent on both casts, the category-A chain packets carry the Basic Attack zone 28/35/42/49/56 percent only inside the 15s window, and every owner packet receipt keeps the CritUp rate 16/20/24/28/32 percent with no critical under fixed crit. The first window survives the field switch and its expiry is exact: a Normal starts at14.875380s, its first contact lands at15s after equal-time expiry, and both of that input's packets retain bare coefficients. The second cast refreshes the window to32s before the40s horizon end; both refresh receipts carry the exact `until {expiry}us` detail and no window instance survives to the final state. Every frozen Daffodil packet is Chaos-typed, so the window's Lakshana zone has no native packet to bind and remains covered by the synthetic category/element unit tests. An independent ChargeAdd energy oracle (owner x1, peers x3/5) settles 68+3779/5000 wearer and 55+4861/5000 peer Ultimate energy with zero spent, and every tier reconciles its full source report with direct native execution and serialized replay. The current CLI also executes the source directly; final browser source-consumer parity remains browser integration.
