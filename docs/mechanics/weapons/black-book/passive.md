# Black Tome — passive

- Native identity: `weapon:fork_BlackBook`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 12.00% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | AtkUp: 15.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | AtkUp: 18.00% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | AtkUp: 21.00% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | AtkUp: 24.00% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | AtkUp: 27.00% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | AtkUp: 30.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **AtkUp 30.00%**.

## Refinement scaling

**Passive Name**: "Black Tome"

> Increases Break Intensity by <lv>{0}</>. Seals Arc: Black Tome with two chains by default. Unleashes 1 chain each time any character on the team casts a Support Skill. Makes Arc: Black Tome available when all two chains are unleashed. Arc: Black Tome — Summons the Black Tome for <lv>{1}</>s. Designates an enemy every <lv>{2}</>s. Increases the wearer's Chaos DMG dealt to designated enemies by <lv>{3}</>. Deals Chaos DMG equal to <lv>{4}</> of the wearer's ATK to designated enemies when they take Break Damage.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_BlackBook2_Unbal | buff_BlackBook2_CD | buff_BlackBook2_CD2 | buff_BlackBook2_DamageUpChaosBase | buff_BlackBook2_SkillDamage |
|---|---|---|---|---|---|
| T1 | 60.00 | 20.00 | 5.00 | 20.00% | 200.00% |
| T2 | 67.00 | 20.00 | 5.00 | 24.00% | 230.00% |
| T3 | 74.00 | 20.00 | 5.00 | 28.00% | 260.00% |
| T4 | 81.00 | 20.00 | 5.00 | 32.00% | 290.00% |
| T5 | 88.00 | 20.00 | 5.00 | 36.00% | 320.00% |

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
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Black Tome (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `AtkUp` contributes +30.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.


The [live Break contribution model](../../shared/content-break-model.md#live-break-hp-contributions) consumes this item's BI points at actual Break generation. Black Tome applies the wearer bonus once; ordinary damage remains separate. The bounded Arc/designation execution model below retains explicit eligibility/capture limits.

## Equipped Arc execution (October 24, 2026)

`weapons.black_book_v1` binds two initial chains. Each admitted team Support cast removes one; a skipped Support or same-character swap cannot substitute for a cast. Sakiri's default engine Support and Daffodil's qualified Support supply public producers. After both chains release, explicit Arc with equipped Black Tome starts a .2s startup/recovery. Successful completion summons the tome for a half-open20s window, reseals two chains and has zero normal cooldown; a pending startup or live summon prevents overlap. An explicit interruption before completion releases recovery, cancels the pending summon and applies30s cancellation cooldown without resealing. Ordinary inputs cannot shorten startup through character cancel gates; source defeat before completion prevents the summon. Further Support casts build the next unlock without resetting other combat state.

The summon initially designates the lowest-index living dummy. Every5s it retains a living designated target or chooses the next lowest living index; target defeat does not cause an immediate retarget. During the modeled window, including its exact endpoint, only the wearer's Chaos hits against that target receive the refinement Chaos bonus. Each delivered team Break HP contribution on that target emits one immediate owner-attributed Chaos ATK packet. Break Intensity affects the independent Break contribution, while this follow-up uses live wearer ATK and ordinary Chaos damage zones. It continues off field; defeated wearers stop designation, new emissions and pending owner packets. Timing/selection are explicit product hypotheses, not certified native callback or prediction behavior.

Each activation retains its actual command index for subsequent hits. Renewal supplies a new parent; ally-triggered Break cannot misattribute the follow-up to a Support or command zero. The player-input ledger exports the resolved Arc ability, so aggregation/replay can validate this input parent alongside ordinary ability admissions. [ADR-0012](../../../decisions/0012-admitted-arc-input-identity.md) owns this narrow transport addition.

The shared [two-cycle script](../../../../examples/black-tome-cycles.nte) uses Lacrimosa with Black Tome, Sakiri, Haniel and Mint. Four paid reaction switches admit automatic Supports. The script declares one dummy Break profile and Arc admissions at 16.1s and 46.1s, with summon windows **[16.3,36.3)** and **[46.3,66.3)**. Rust independently sets live ATK1000 and zero crit/other bonuses: T1 through T5 follow-ups are `1000*[2,2.3,2.6,2.9,3.2]*[1.20,1.24,1.28,1.32,1.36]` = **2400/2852/3328/3828/4352**, with no Break Intensity factor. Four team contributions at 16.7/30.1/36.1/46.7/66.1s give twenty follow-ups, including off-field delivery; hits at40/67s give none. Separate tests check exact half-open endpoints, startup interruption/retry,5s retargeting and a real later team Break after wearer defeat. Native source compilation and retained WASM/Worker transport checks compare reports, aggregation and replay, including rejected missing/wrong/future input links.
