# Be Happy — passive

- Native identity: `weapon:fork_vine`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_BLUE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 21.0 | Stage 0 | Max Lv 20 | +0.0 | HPMaxUp: 12.00% |
| Lv 20 | 72.0 | Stage 1 | Max Lv 30 | +14.0 | HPMaxUp: 15.00% |
| Lv 40 | 126.0 | Stage 2 | Max Lv 40 | +27.0 | HPMaxUp: 18.00% |
| Lv 60 | 180.0 | Stage 3 | Max Lv 50 | +41.0 | HPMaxUp: 21.00% |
| Lv 80 | 234.0 | Stage 4 | Max Lv 60 | +55.0 | HPMaxUp: 24.00% |
| Lv 80 | 234.0 | Stage 5 | Max Lv 70 | +68.0 | HPMaxUp: 27.00% |
| Lv 80 | 234.0 | Stage 6 | Max Lv 80 | +82.0 | HPMaxUp: 30.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **316.0** (234.0 + 82.0); Secondary Stat = **HPMaxUp 30.00%**.

## Refinement scaling

**Passive Name**: Huggy Vines

> Restores wearer's HP by <lv>{0}</> upon defeating an enemy. Triggers at most once every <lv>{1}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_vine_Hp | buff_vine_CD |
|---|---|---|
| T1 | 12.00% | 20.00 |
| T2 | 14.00% | 20.00 |
| T3 | 16.00% | 20.00 |
| T4 | 18.00% | 20.00 |
| T5 | 20.00% | 20.00 |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (316.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Be Happy (Base ATK 316.0):
    - Base ATK = 600 + 316.0 = 916.0.
    - Secondary Stat `HPMaxUp` contributes +30.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Continuing spread-defeat cycles

The [shared script](../../../../examples/be-happy-defeat-cycles.nte) equips A0
Skia/Be Happy and declares four anonymous dummies. Skia's reviewed Burst spread
scope is implemented first: one spread packet reaches each living dummy, while
three converge bites remain on the primary. Two simultaneous spread defeats at
0s share one wearer cooldown and create one heal. Real Normal packets refill
Ultimate energy; a later spread defeat at40s creates the second heal. Dead
secondary dummies receive no later packets. No target roster, presets or
resource resets participate.

Independent Rust checks retain the legal native HP projection: base13332 and
the weapon's30% HP give17332 Max HP; quarter-HP admission floors to4333. T1–T5
heals round once to2080/2426/2773/3120/3466 per defeat, with final HP
8493/9185/9879/10573/11265 after two accepted callbacks. A second scenario
replaces one early defeat with a5805-HP dummy. At isolated ATK1000,
two2902-point spread hits leave it at exactly1HP, so it cannot supply
an extra heal. Ascending spread order attributes the first heal to target1 and
the second to target3. Both100-energy Ultimates are paid from continuing state.
The existing owner tests retain exact20s ICD, nonfatal/zero/already-dead hits,
other-killer exclusions, live Max HP capture, healing zones, overflow and dead
wearer behavior. Native/WASM/actual Worker compare complete script reports,
aggregation and replay at all five refinements. Native cadence and Skia's other
unbound branches remain the character card's limits.
