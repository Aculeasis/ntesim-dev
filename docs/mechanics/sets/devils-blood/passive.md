# Devil's Blood: Curse (Devil's Blood: Curse) — set effects

- Native identity: `set:Suit3`.
- Responsibility: this card owns the set's threshold effects and combat execution model.

## 2-piece set bonus

> Epic (2): Psyche DMG +10%.

- **Modifier Pack Binding**: `DT_EquipmentModifyData` -> `Psyche_2`.
- **Stat Contributions**: `DamageUpPsycheBase=10.0%`.
- **Formula Zone**: Evaluated during static build compilation (`ntesim_data::stat_projection`); adds directly into the character's base/damage stat pool.

## 4-piece set bonus

> Legendary (4): Increases wearer's DMG by 18%. The bonus becomes 36% against units affected by Nova or Stain.

- **Blueprint / Buff Class**: `BlueprintGeneratedClass'Buff_Equipment_Psyche_4_C'`.
- **Curve Parameters from `CT_Equipmentbuff`**:
  - `Psyche_2`: values = `[0.1]`
  - `Psyche_4_1`: values = `[0.18]`
  - `Psyche_4_2`: values = `[0.18]`

## Combat procs and team interactions

- **Source:** `Buff_Equipment_Psyche_4` `/1/Properties` has an infinite additive `DamageUpGeneralBase` modifier using `CT_Equipmentbuff:Psyche_4_1`, plus one conditional additive modifier using `Psyche_4_2` and `Con_Equipment_Psyche_4`. The condition's `/1/Properties/Tag Container` contains `Ability.Player.Buff.Reaction.HunXiang` and `Ability.Player.Buff.Reaction.AnHun`. These exact fields are retained in effect parameters (effect_parameters.jsonl). The recovered function also calls native `GetReactionResultType`; [reaction classification](../../shared/cycle-source-findings.md#reaction-result-classification) owns that mapping, without proving its native capture/evaluation timing.
- **Modeled policy (inferred):** `sets.devils_blood_v1` adds the first18% to every eligible wearer packet. At delivery, any live status on that packet's selected target carrying either native tag adds the second18% once. This follows the tooltip's target condition and the serialized tag container. Status source and stacks do not multiply the bonus; simultaneous Nova and Stain still give36% total. Unrelated tags, statuses on another dummy and statuses expiring at the packet timestamp do not qualify. Switching preserves the wearer's pending packets and never lends the set bonus to the new active actor. The passive creates no timer or ICD; it observes the status lifecycle already managed by the engine.
- **Formula and check:** both18% terms join the ordinary damage-bonus zone, alongside static damage/element bonuses. ATK1000, coefficient1, DEF factor1/2, RES factor4/5 and another25% damage bonus give `400*(1+.25+.18)=572`, or `400*(1+.25+.18+.18)=644` while affected. Multiplying `400*1.25*1.18` would incorrectly give590. Native two-piece Psyche10% remains static and is not re-added by this adapter. Existing True-damage bonus bypass remains intact.
- **Executable scope:** native equipped builds and Studio/source declarations bind both curves and preserve parameter receipts; two-piece-only builds do not install the four-piece adapter. Qualified Cycle switches now automatically produce Nova/Stain target tags. Rust reaction tests exercise both automatic branches, including Stain's separate multiplier and Twinflight's live CI, while full native/WASM/Worker source fixtures cover continuing applications and expiry. The native reaction-result getter's capture/evaluation remains uncertified; the accepted target-tag model is the executable boundary.

### Executable checks

`crates/content/src/simulation/tests/devils_blood.rs` runs full packet settlement with the independent572/644 oracles, teammate-owned statuses, either/both tags, target isolation, half-open expiry and delayed off-field delivery. It also checks source curve receipts, parameter-unit failures, invalid/duplicate equipment binding and the ordinary Mint source-build ratio `1.43/1.25=143/125`. The native build suite checks declared set counts and static two-piece preservation. `web/tests/weapon_worker_parity.test.mjs` compares full native/WASM/Worker reports, aggregation and replay for two/four-piece source builds; mechanics arithmetic remains exclusively in Rust.

## Complete implementation handoff

- **Engine Mapping & Zone Placement**:
  - 2-piece effect applies statically in compiled build stats.
  - 4-piece effect registers event listeners in simulation execution (`crates/content/src/simulation/`).
- **Independent Verification Case**:
  - Equipping 2 pieces of Devil's Blood: Curse activates `Psyche_2` (DamageUpPsycheBase=10.0%).
  - Equipping 4 pieces unlocks `BlueprintGeneratedClass'Buff_Equipment_Psyche_4_C'`, transitioning active/powered combat multipliers when activation conditions are satisfied.
  - Replay and multi-trial traces verify determinism and exact stat conservation.
