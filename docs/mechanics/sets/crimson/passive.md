# Crimson: Twin Butterflies (Twin Butterflies (Crimson)) — set effects

- Native identity: `set:Suit4`.
- Responsibility: this card owns the set's threshold effects and combat execution model.

## 2-piece set bonus

> Epic (2): Incantation DMG +10%.

- **Modifier Pack Binding**: `DT_EquipmentModifyData` -> `Incantation_2`.
- **Stat Contributions**: `DamageUpIncantationBase=10.0%`.
- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.

## 4-piece set bonus

> Legendary (4): Increases wearer's ATK by 6% when a nearby enemy takes Incantation DMG from the Team, up to 6 stacks. Each stack lasts 10s. Effect remains active when the character is off-field.

- **Blueprint / Buff Class**: `BlueprintGeneratedClass'Buff_Equipment_Incantation_4_C'`.
- **Curve Parameters from `CT_Equipmentbuff`**:
  - `Incantation_2`: values = `[0.1]`
  - `Incantation_4_1`: values = `[0.06]`
  - `Incantation_4_2`: values = `[6.0]`
  - `Incantation_4_3`: values = `[10.0]`

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
  - Equipping 2 pieces of Crimson: Twin Butterflies activates `Incantation_2` (DamageUpIncantationBase=10.0%).
  - Equipping 4 pieces unlocks `BlueprintGeneratedClass'Buff_Equipment_Incantation_4_C'`, transitioning active/powered combat multipliers when activation conditions are satisfied.
  - Replay and multi-trial traces verify determinism and exact stat conservation.
