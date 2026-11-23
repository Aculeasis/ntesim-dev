# Fireflies and the Forest (Fireflies and the Forest) — set effects

- Native identity: `set:Suit2`.
- Responsibility: this card owns the set's threshold effects and combat execution model.

## 2-piece set bonus

> Epic (2): Anima DMG +10%.

- **Modifier Pack Binding**: [typed static modifiers](../../../../crates/data/src/set_modifiers.rs) -> `Nature_2` (original `DT_EquipmentModifyData` evidence is cache-only).
- **Stat Contributions**: `DamageUpNatureBase=10.0%`.
- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.

## 4-piece set bonus

> Legendary (4): Increases wearer's CRIT DMG by 8% when a nearby enemy takes Anima DMG from the Team, up to 7 stacks. Each stack lasts 10s. Effect remains active when the character is off-field.

- **Blueprint / Buff Class**: `BlueprintGeneratedClass'Buff_Equipment_Nature2_4_C'`.
- **Consumed [typed parameters](../../../../crates/data/src/set_parameters.rs)** (original `CT_Equipmentbuff` evidence is cache-only):
  - `Nature2_4_CD`: values = `[10.0]`
  - `Nature2_4_CritDamage`: values = `[0.08]`
  - `Nature2_4_Stack`: values = `[7.0]`

## Combat procs and team interactions

- **Trigger & Activation Window**:
  - Evaluated on qualified combat events (hit delivery, reaction participation, skill activation, shield status).
  - Duration and ICD are enforced at discrete simulation timestamps.
- **Wearer State & Team Scope**:
  - Applied to the wearer or eligible active teammate as declared by the set specification.
  - Off-field listeners and wearer swaps preserve or transition active buff state without double-granting.
- **Target Dummy Interactions**:
  - Stationary benchmarks evaluate debuff/shred directly against target defense/resistance zones.

## Complete implementation handoff

- **Engine Mapping & Zone Placement**:
  - 2-piece effect applies statically in compiled build stats.
  - 4-piece effect registers event listeners in simulation execution (`crates/content/src/simulation/`).
- **Independent Verification Case**:
  - Equipping 2 pieces of Fireflies and the Forest activates `Nature_2` (DamageUpNatureBase=10.0%).
  - Equipping 4 pieces unlocks `BlueprintGeneratedClass'Buff_Equipment_Nature2_4_C'`, transitioning active/powered combat multipliers when activation conditions are satisfied.
  - Replay and multi-trial traces verify determinism and exact stat conservation.
