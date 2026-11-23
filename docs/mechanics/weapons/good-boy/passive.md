# Good Boy's Grand Adventure — passive

- Native identity: `weapon:fork_mofeikesi`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 18.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | AtkUp: 22.50% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | AtkUp: 27.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | AtkUp: 31.50% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | AtkUp: 36.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | AtkUp: 40.50% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +123.0 | AtkUp: 45.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **474.0** (351.0 + 123.0); Secondary Stat = **AtkUp 45.00%**.

## Refinement scaling

**Passive Name**: Morphix

> Increases Charge Efficiency by <lv>{0}</>. Increases Team ATK by <lv>{2}</> for <lv>{1}</>s after the wearer casts an Ultimate. Increases Team ATK by an additional <lv>{3}</> if enemies are controlled by the Ultimate. Effects with the same name do not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_mofeikesi_ChargeGetEfficiency | buff_mofeikesi_CD | buff_mofeikesi_Atk | buff_mofeikesi_Up |
|---|---|---|---|---|
| T1 | 18.00% | 20.00 | 10.00% | 6.00% |
| T2 | 21.00% | 20.00 | 11.50% | 7.00% |
| T3 | 24.00% | 20.00 | 13.00% | 8.00% |
| T4 | 27.00% | 20.00 | 14.50% | 9.00% |
| T5 | 30.00% | 20.00 | 16.00% | 10.00% |

## Activation and trigger mechanics

The executable unconditional slice uses `weapons.good_boy_v3`. Both native build compilation and Studio/DSL stat projection add the frozen 18/21/24/27/30 percentage points to the wearer's existing Charge Efficiency once. The resulting generic received-energy multiplier is visible to the build UI and shared by ordinary hits, off-field team receipts and weapon callbacks. Projection preserves fractional carry, capacity, overflow and spending; switching does not remove this permanent stat. Other recipients retain their own efficiency.

The existing admitted-Ultimate model grants one shared, nonstacking team ATK-percent window for 20 seconds. ATK percentages sum against each recipient's own base ATK, separate from compiled flat ATK. Recasting refreshes the same named effect; the world-time window continues after switching and expires before equal-time hits.

The additional 6/7/8/9/10% team ATK ratio now consumes the typed successful-control receipt from the wearer's same admitted Ultimate. The currently bound producer is Sakiri Suppress. That first qualifying receipt upgrades the single named team window to base plus additional ratio, once across all recipients. It retains the original cast's 20-second deadline and remains latched when the shorter control status ends. This selects one conditional upgrade of the existing window rather than a separate new-duration buff; native capture/application order remains unverified and is disclosed by the model.

A receipt from another actor/command, ordinary Ultimate damage, a preexisting airborne deadline or Boss classification cannot substitute for successful control. A later wearer replacing the shared named snapshot prevents the older wearer's pending control from upgrading it. Repeated recipient controls do not stack or refresh the bonus. Control at or after the original deadline cannot restart the window. Legacy explicit TeamAtk-only API procs omit the additional ratio and retain zero; native parameter projection binds it for every refinement.

Projection now marks execution validated for this model under the explicit control/dummy assumptions, preserving the remaining native control, timing and cross-wearer priority limits. This does not certify complete control dispatch for every character.

## Complete implementation handoff

- Weapon constants remain 474 base ATK and 45% ATK at level 80/stage 6. They enter build stats once; the conditional team ATK bonus is a separate live base-ATK percentage.
- Independent efficiency example: an authored +.50 and T1 +.18 produce 1.68, not 1.50*1.18=1.77. T2–T5 produce 1.71/1.74/1.77/1.80. With the 12% two-piece set stat instead, T1 gives 1.30: two 9-energy grants deliver 11 then 12 whole energy and retain 2/5 fractional carry.
- `examples/good-boy-energy-cycles.nte` runs A0 Sakiri's two continuing Skill/Burst visits, normal chains and real off-field Mint energy generation at all five refinements. The same pool spends 200 energy without resets. `simulation/tests/good_boy_cycles.rs` checks the additive multiplier, each recipient's grant accounting, off-field receipts and deterministic replay. The five public sources cross CLI/WASM/actual Worker comparison, aggregation and replay.
- Native build tests retain replacement of authored efficiency, reject duplicate/invalid multipliers and exercise the set-plus-weapon carry example. Parameter projection rejects changed unit hints and additional/missing dependencies. Legacy explicit TeamAtk-only API procs do not invent an unevaluated static build bonus.

`examples/good-boy-control-cycles.nte` repeats the two continuing visits with an immune first dummy and a controllable second dummy. Independent normalized stats use 2000 compiled ATK and 1000 base ATK: the upgrade changes 2100→2160, 2115→2185, 2130→2210, 2145→2235 and 2160→2260 at T1–T5. These distinguish addition to base ATK from scaling compiled ATK or multiplying percentage zones. The first control-producing hit precedes the new bonus; later packets and living allies consume it. `simulation/tests/good_boy_control.rs` checks these values, multi-recipient nonduplication, persistence after control expiry, original-deadline expiry, stale same-/cross-wearer control, successful control with zero rounded HP loss and replay. CLI/WASM/actual Worker compare all five public controlled sources and the separate immune sources, including aggregation/replay.
