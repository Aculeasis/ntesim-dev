# Decomposer — passive

- Native identity: `weapon:fork_dustbin`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_BLUE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 24.0 | Stage 0 | Max Lv 20 | +0.0 | UnbalIntensityBase: 38.0 |
| Lv 20 | 84.0 | Stage 1 | Max Lv 30 | +17.0 | UnbalIntensityBase: 48.0 |
| Lv 40 | 147.0 | Stage 2 | Max Lv 40 | +35.0 | UnbalIntensityBase: 58.0 |
| Lv 60 | 210.0 | Stage 3 | Max Lv 50 | +53.0 | UnbalIntensityBase: 67.0 |
| Lv 80 | 273.0 | Stage 4 | Max Lv 60 | +71.0 | UnbalIntensityBase: 77.0 |
| Lv 80 | 273.0 | Stage 5 | Max Lv 70 | +89.0 | UnbalIntensityBase: 86.0 |
| Lv 80 | 273.0 | Stage 6 | Max Lv 80 | +107.0 | UnbalIntensityBase: 96.0 |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **380.0** (273.0 + 107.0); Secondary Stat = **96 BI points**.

## Refinement scaling

**Passive Name**: "Decomposer"

> Increases wearer's Break Intensity by <lv>{0}</> for <lv>{1}</>s when reducing Break. Triggers at most once every <lv>{2}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_dustbin_Unbal | buff_dustbin_CD | buff_dustbin_CD2 |
|---|---|---|---|
| T1 | 60.00 | 10.00 | 20.00 |
| T2 | 66.00 | 10.00 | 20.00 |
| T3 | 72.00 | 10.00 | 20.00 |
| T4 | 78.00 | 10.00 | 20.00 |
| T5 | 84.00 | 10.00 | 20.00 |

## Activation and trigger mechanics

The executable `weapons.dustbin_v1` model now triggers on actual positive gauge loss by the wearer in the content Break state machine, including partial depletion and delivered off-field packets. A semantic tag alone cannot grant the buff. No bar, zero effective loss, an already Broken/dead target or zero starting gauge cannot trigger it. Per-wearer BI lasts10 seconds without refresh; the shared per-wearer ICD lasts20 seconds, both with half-open deadlines on real combat time. No switch reset is inferred. Accepted pending reductions retain their source attribution even after carrier removal; live Break contributors still require living actors at capture.

The grant follows gauge subtraction and precedes capture of the new Break generation, so the triggering Break receives the new BI. This ordering is a reasoned tooltip-based model, not a recovered native consumer. The secondary stat is **96 BI points**, and the passive adds60/66/72/78/84 points at T1-T5; neither is a percentage multiplier. With level80 base3603, BI300, cap30, DEF1080 and RES.20, ordinary Break is28824, T1 is31706.4 and T5 is32859.36 before one HP rounding. Rust tests check all tiers through six continuing Break generations at0/2/10/20/22/40 seconds, including expiry and ICD boundaries; native/WASM/actual Worker checks cover equipped source scripts, aggregation and replay. These checks establish the model rather than native-game measurements.

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (380.0).
  - Secondary stat `UnbalIntensityBase` adds points to Break Intensity.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Decomposer (Base ATK 380.0):
    - Base ATK = 600 + 380.0 = 980.0.
    - Secondary Stat `UnbalIntensityBase` contributes 96 BI points to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
