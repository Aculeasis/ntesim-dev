# Eclipse — passive

- Native identity: `weapon:fork_rishi`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | ChargeGetEfficiencyBase: 13.20% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | ChargeGetEfficiencyBase: 16.50% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | ChargeGetEfficiencyBase: 19.80% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | ChargeGetEfficiencyBase: 23.10% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | ChargeGetEfficiencyBase: 26.40% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | ChargeGetEfficiencyBase: 29.70% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | ChargeGetEfficiencyBase: 33.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **ChargeGetEfficiencyBase 33.00%**.

## Refinement scaling

**Passive Name**: Eclipse

> Increases wearer's ATK by <lv>{0}</>. Unlocks Arc: Eclipse. Arc: Eclipse — The wearer creates a <lv>{1}</>s Eclipse. During the Eclipse, each enemy defeated restores <lv>{2}</> Ultimate Energy, up to <lv>{3}</> times. (cooldown: <lv>{4}</>s).

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_rishi_AtkUp | buff_rishi_CD2 | buff_rishi_energy | buff_rishi_stack | buff_rishi_CD |
|---|---|---|---|---|---|
| T1 | 30.00% | 40.00 | 6.00 | 5.00 | 300.00 |
| T2 | 35.00% | 40.00 | 7.00 | 5.00 | 300.00 |
| T3 | 40.00% | 40.00 | 8.00 | 5.00 | 300.00 |
| T4 | 45.00% | 40.00 | 9.00 | 5.00 | 300.00 |
| T5 | 50.00% | 40.00 | 10.00 | 5.00 | 300.00 |

## Activation and trigger mechanics

- **Modeled execution:** `haniel arc;` (or another compatible wearer) explicitly activates the 40-second field through `weapons.rishi_v2`, reserves the 300-second Arc cooldown and credits at most five wearer-attributed defeats in the half-open field window. Ultimate casts alone no longer activate the field. Team kills do not qualify under this existing hypothesis. ATK% joins the live base-ATK percentage zone; per-defeat energy is6/7/8/9/10 across T1–T5 and follows the shared charge-efficiency/carry/overflow settlement.
- **Admission:** the shared Rust action service checks living/active/recovery/cooldown state atomically. Arc itself spends no energy or HP, and adds no recovery under the bounded instantaneous field-creation model; the tooltip supplies lifetime/cooldown but no measured cast lock or animation time. An input just before the 300-second deadline fails with the ordinary cooldown diagnostic and does not refresh the field or reset its defeat count. Exact-deadline input succeeds. Switching retains the original wearer's field and shared action cooldown. No separate equipment cooldown copy remains.
- **Recipient lifecycle (inferred):** the wearer must remain alive at defeat dispatch and callback delivery. A death between the two cancels the pending grant without changing energy, overflow or fractional carry; the reserved defeat slot/cooldown is retained. A pending field callback also requires a living wearer. Switching alone preserves eligibility and does not redirect energy to the active teammate.
- **Rationale and check:** living-recipient ordinary generation and absence of a revival rule support checking life at delivery; this is a bounded model policy rather than native dispatch evidence. `simulation/tests/rishi.rs` checks T1/T5 pending callbacks after death against living off-field counterexamples, unchanged fractional carry, no teammate energy and no additional defeat-slot consumption by a dead source. Existing independent damage, field expiry, cooldown, cap and energy-oracle cases remain in the same owner suite. Worker/native/WASM comparisons cover live and same-time-fatal field admission through source-projected Haniel builds.
- **Remaining:** native cast/recovery timing, exact kill attribution, field-lifetime interpretation and charge scaling remain unverified. Explicit owner admission is executable; the bounded activation/kill/energy policies do not establish native execution or full gameplay readiness.

The independent native-build energy oracle uses the max-level weapon's +33% charge efficiency once:6/7/8/9/10 flat energy becomes7/9/10/11/13 whole units with carry98/31/64/97/30 hundredths. Rust source scenarios combine explicit Arc with a defeating Haniel Burst; removing Arc preserves Burst damage and yields no Eclipse grant. The live source adapter now binds Haniel's native120-cost Burst and ordinary clock; the source report and direct native replay retain the same independent grants. Continuing source Arc inputs at0/300seconds preserve HP/energy, while an input at299.999999seconds fails without refreshing. Native checks cover all five refinements, defeat grants, zero-start field reactivation, aggregation and replay; current source-consumer browser/Worker parity remains a browser integration gate. Existing field expiry, five-defeat cap, off-field ownership and pending-death checks remain in the owner suite.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `ChargeGetEfficiencyBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Eclipse (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `ChargeGetEfficiencyBase` contributes +33.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
