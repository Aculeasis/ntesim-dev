# Street Boxer (Street Boxer) — set effects

- Native identity: `set:Suit5`.
- Responsibility: this card owns the set's threshold effects and combat execution model.

## 2-piece set bonus

> Epic (2): Lakshana DMG +10%.

- **Modifier Pack Binding**: `DT_EquipmentModifyData` -> `Lakshana_2`.
- **Stat Contributions**: `DamageUpLakshanaBase=10.0%`.
- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.

## 4-piece set bonus

> Legendary (4): Increases Crit Chance by 14%. Team triggering Remora or Stain increases the bonus to 14% for 20s.

- **Blueprint / Buff Class**: `BlueprintGeneratedClass'Buff_Equipment_Lakshana2_4_C'`.

## Combat procs and team interactions

- **Consumed source parameters:** `CT_Equipmentbuff:Lakshana_4_1=.14` is the baseline, `Lakshana_4_2=.14` is the powered replacement, and `Lakshana_4_CD=20` is seconds. Only these three consumed curves are retained in the typed set table; projection preserves their exact receipts and rejects missing/invalid ratios or zero/overflowing deadlines.
- **Selected executable model:** `sets.street_boxer_v2` adds the baseline once in the wearer's ordinary crit-rate zone. Four-piece static build/source inspection includes this unconditional fraction in `CritBase`; compiled gear records that inclusion, and runtime applies only a replacement's difference from it. Legacy raw requests omit that flag and retain the runtime baseline addition. Each qualified team Cycle admission producing Stain or Remora starts one self-only twenty-second window for every living equipped wearer. The wearer may be off field and outside the triggering pair; this is a team listener, unlike Diabolos's participant-only rule. A dead wearer receives no new window. All equipped recipients are prepared atomically before publication.
- **Window lifecycle:** the powered curve replaces the baseline rather than adding a second crit bonus. Further qualifying team reactions reset the same window's expiry without stacking. Field exit preserves its world-clock deadline, and exact expiry restores the baseline. Other reactions, ordinary switches, same-actor swaps and missing outgoing Cycle do not create or refresh it. Packet-specific fixed crit overrides continue to take precedence.
- **Independent checks:** at ordinary CR.50, both real frozen states give `.50+.14=.64`, never `.78`. Rust tests also use a distinct research-only powered value.28 to discriminate the replacement/expiry code path: CR.78 before expiry and.64 at expiry. This test value is not a runtime constant. Native source fixtures cover two continuing Stain admissions, an earlier Remora/Blossom Charge interaction, an unrelated equipped teammate, and post-expiry attacks through CLI/WASM/actual Worker, aggregation and replay. Source/callback fidelity remains a bounded model; native dispatch internals are not claimed verified.

## Complete implementation handoff

- **Engine Mapping & Zone Placement**:
  - 2-piece effect applies statically in compiled build stats.
  - 4-piece unconditional crit compiles statically; its reaction replacement registers event listeners in simulation execution (`crates/content/src/simulation/`).
- **Independent Verification Case**:
  - Equipping 2 pieces of Street Boxer activates `Lakshana_2` (DamageUpLakshanaBase=10.0%).
  - Equipping 4 pieces unlocks `BlueprintGeneratedClass'Buff_Equipment_Lakshana2_4_C'`, transitioning active/powered combat multipliers when activation conditions are satisfied.
  - Replay and multi-trial traces verify determinism and exact stat conservation.
