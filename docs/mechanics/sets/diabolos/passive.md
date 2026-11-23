# Diabolos (Diabolos) — set effects

- Native identity: `set:Suit1`.
- Responsibility: this card owns the set's threshold effects and combat execution model.

## 2-piece set bonus

> Epic (2): Chaos DMG +10%.

- **Modifier Pack Binding**: `DT_EquipmentModifyData` -> `Chaos_2`.
- **Stat Contributions**: `DamageUpChaosBase=10.0%`.
- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.

## 4-piece set bonus

> Legendary (4): Ignores 12% of enemies' Chaos RES. Ignores 24% of enemies' Chaos RES for 20s after the wearer participates in Nova or Scorch reactions.

- **Blueprint / Buff Class**: `BlueprintGeneratedClass'Buff_Equipment_Chaos2_4_C'`.
- **Curve Parameters from `CT_Equipmentbuff`**:
  - `Chaos2_4_CD`: values = `[20.0]`
  - `Chaos2_4_IgnoreChaos`: values = `[0.12]`
  - `Chaos2_4_IgnoreChaos2`: values = `[0.24]`

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

`sets.diabolos_v2` binds the reaction window through equipped-effect participation dispatch. A qualified engine Cycle snapshots its CI owner, then treats both living outgoing/incoming actors as participants before spending the outgoing pool and producing the reaction. Each Diabolos wearer among those two receives a self-only twenty-second world-clock window for Nova/Scorch, regardless of reaction ownership. Its 24-point Chaos RES ignore **replaces** the baseline 12; it does not add to 36. Reparticipation refreshes one window, field exit preserves it, and exact expiry restores 12. Same-actor/unqualified switches, Blossom, unrelated equipped teammates and non-Chaos packets receive no powered effect. Stain/Remora production and their other equipment consumers remain separate pending bindings.

The projector consumes only the three typed constants above. The existing gear validation boundary checks horizon plus duration before execution; the dispatcher prepares every participant's buff, assumptions and causal receipt before publishing any of them.

The [Rust owner tests](../../../../crates/content/src/simulation/tests/diabolos.rs) isolate the RES zone with ATK 1000, coefficient 1, level-80 DEF factor 1/2 and original RES .20. Baseline `.20-.12=.08` gives **460**. During the window, `R=-.04` uses the selected `standard_rational` branch: `1-R/(1-R)=27/26`, giving exactly **6750/13**. Incorrectly summing to .36 would give 16500/29. Hits immediately before/at/after 20s distinguish expiry; a second qualified Cycle at 10s extends it to 30s. Other checks cover incoming nonowner participation, both equipped participants, third-party exclusion, other elements/reactions, failed preparation and explicit-overlay missing/invalid parameters. Native/WASM/Worker compare legal four-piece sources over two continuing Nova and Scorch cycles with aggregation/replay. These are bounded execution and negative-RES policies, not proof of native participation dispatch or clock fidelity.

- **Engine Mapping & Zone Placement**:
  - 2-piece effect applies statically in compiled build stats.
  - 4-piece effect registers event listeners in simulation execution (`crates/content/src/simulation/`).
- **Independent Verification Case**:
  - Equipping 2 pieces of Diabolos activates `Chaos_2` (DamageUpChaosBase=10.0%).
  - Equipping 4 pieces unlocks `BlueprintGeneratedClass'Buff_Equipment_Chaos2_4_C'`, transitioning active/powered combat multipliers when activation conditions are satisfied.
  - Replay and multi-trial traces verify determinism and exact stat conservation.
