# Solifugae Vita — passive

- Native identity: `weapon:fork_snowman`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 25.0 | Stage 0 | Max Lv 20 | +0.0 | DefUp: 21.00% |
| Lv 20 | 88.0 | Stage 1 | Max Lv 30 | +18.0 | DefUp: 26.25% |
| Lv 40 | 154.0 | Stage 2 | Max Lv 40 | +36.0 | DefUp: 31.50% |
| Lv 60 | 220.0 | Stage 3 | Max Lv 50 | +54.0 | DefUp: 36.75% |
| Lv 80 | 286.0 | Stage 4 | Max Lv 60 | +72.0 | DefUp: 42.00% |
| Lv 80 | 286.0 | Stage 5 | Max Lv 70 | +90.0 | DefUp: 47.25% |
| Lv 80 | 286.0 | Stage 6 | Max Lv 80 | +109.0 | DefUp: 52.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **395.0** (286.0 + 109.0); Secondary Stat = **DefUp 52.50%**.

## Refinement scaling

**Passive Name**: Solifugae Vita

> Increases ATK by <lv>{0}</> while the wearer has a Shield.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_snowman_AtkUp |
|---|---|
| T1 | 18.00% |
| T2 | 21.00% |
| T3 | 24.00% |
| T4 | 27.00% |
| T5 | 30.00% |

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
  - Secondary stat `DefUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Solifugae Vita (Base ATK 395.0):
    - Base ATK = 600 + 395.0 = 995.0.
    - Secondary Stat `DefUp` contributes +52.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

Continuing native cycles (`examples/snowman-blessing-cycles.nte`, `snowman_cycles.rs`): the group-FIVE wearer is native lvl80 Adler (14302 HP) — the only implemented group member whose own kit can shield the wearer in a shared script, since his Blessing Skill shields the active actor — with a Mint peer (12120 HP) and two 8s Blessing windows from casts at 0s and 8.8s (six-second cooldown) at static ATK 1000 and DEF 1000, so each shield snapshots exactly 1800. Every tier settles all 56 packets: the 26 category-A chain packets per pair of visits carry the shielded live-ATK zone `static_atk + base_atk x` 18/21/24/27/30 percent — Melee1 201x59/50 at T1 through 201x13/10 at T5 — while the two boundary hits landing exactly at 8s and 16.8s settle at bare 201 (the half-open shield interval removes the bonus before HitDelivery, and the `actor_shield_expired` trace receipts carry those instants); the DEF-scaled Skill2/Skill3 (700) and tick-train packets (200) prove the zone never touches DEF scaling, with the second cast's train raised by the consumed 13-stack karma final zone to 882/252. Adler is an ordinary-hit energy owner, and an independent ChargeAdd oracle (owner x1, peers x3/5) settles 53+731/1250 wearer and 35+37/50 peer Ultimate energy with zero spent; every tier replays identically from its serialized request. Shield depletion by incoming damage stays covered by the unit tests (`snowman_bonus_removed_when_incoming_damage_depletes_shield`, `adler_skill_and_snowman_use_settled_shields_through_switch_depletion_and_expiry`) because scenario `incoming` declarations are not implemented by the source compiler.
