# Crimson Hexblade — passive

- Native identity: `weapon:fork_yaodao`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 25.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 15.00% |
| Lv 20 | 88.0 | Stage 1 | Max Lv 30 | +18.0 | AtkUp: 18.75% |
| Lv 40 | 154.0 | Stage 2 | Max Lv 40 | +36.0 | AtkUp: 22.50% |
| Lv 60 | 220.0 | Stage 3 | Max Lv 50 | +54.0 | AtkUp: 26.25% |
| Lv 80 | 286.0 | Stage 4 | Max Lv 60 | +72.0 | AtkUp: 30.00% |
| Lv 80 | 286.0 | Stage 5 | Max Lv 70 | +90.0 | AtkUp: 33.75% |
| Lv 80 | 286.0 | Stage 6 | Max Lv 80 | +109.0 | AtkUp: 37.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **395.0** (286.0 + 109.0); Secondary Stat = **AtkUp 37.50%**.

## Refinement scaling

**Passive Name**: Crimson Hexblade

> Deals additional Incantation DMG equal to <lv>{0}</> of ATK when the wearer triggers a Parry Attack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | GE_Fork_yaodao_Skill_Damage |
|---|---|
| T1 | 200.00% |
| T2 | 230.00% |
| T3 | 260.00% |
| T4 | 290.00% |
| T5 | 320.00% |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (395.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Crimson Hexblade (Base ATK 395.0):
    - Base ATK = 600 + 395.0 = 995.0.
    - Secondary Stat `AtkUp` contributes +37.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable Parry packet model

`weapons.yaodao_v1` and `weapons.yaodao.separate_packet_v1` emit a separate connected Incantation ATK child packet (`weapon:fork_yaodao:parry_incantation`) whenever the wearer connects a hit carrying the `ntesim.trigger.parry_attack` gameplay tag or `InputActionRole::ParryAttack`. The child packet inherits the parent's timing and target, and deals Incantation damage equal to the refinement scaling curve (200%/230%/260%/290%/320% of ATK) across T1 to T5. Ordinary hits without parry tags emit no child packets.

Continuing cycles in [Rust weapon tests](../../../../crates/content/src/simulation/tests/yaodao.rs) verify all five refinements T1–T5 under `run_game_scenario`, testing parent hit delivery, exact child scaling (2000, 2300, 2600, 2900, 3200 for 1000 ATK), Incantation damage kind, receipt isolation, and continuing multi-cycle state without resetting resources or procs.
