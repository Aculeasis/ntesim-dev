# Silent Garden — passive

- Native identity: `weapon:fork_jingmotingyuan`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 43.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 4.80% |
| Lv 20 | 148.0 | Stage 1 | Max Lv 30 | +30.0 | CritBase: 6.00% |
| Lv 40 | 259.0 | Stage 2 | Max Lv 40 | +61.0 | CritBase: 7.20% |
| Lv 60 | 370.0 | Stage 3 | Max Lv 50 | +92.0 | CritBase: 8.40% |
| Lv 80 | 481.0 | Stage 4 | Max Lv 60 | +123.0 | CritBase: 9.60% |
| Lv 80 | 481.0 | Stage 5 | Max Lv 70 | +154.0 | CritBase: 10.80% |
| Lv 80 | 481.0 | Stage 6 | Max Lv 80 | +185.0 | CritBase: 12.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **666.0** (481.0 + 185.0); Secondary Stat = **CritBase 12.00%**.

## Refinement scaling

**Passive Name**: Silent Garden

> Increases CRIT DMG by <lv>{1}</> for <lv>{0}</>s when the wearer's HP is reduced without taking damage, up to <lv>4</> stacks. Unlocks Arc: Silent Garden. Consumes <lv>{2}</> of current HP and randomly performs one of the following attacks (CD <lv>{3}</>s) in Arc: Silent Garden: Harsh Rebuke: Rains merciless rebukes like leaves in a storm, dealing <lv>5</> instances of Incantation DMG of <lv>{4}</> ATK. Incessant Nagging: Pierces targets with sharp, cutting words like thorns, dealing <lv>6</> instances of Incantation DMG of <lv>{5}</> ATK ; the final instance deals double damage.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_tingyuan_CD | buff_tingyuan_CritDamageUp | buff_tingyuan_HPreduce | buff_tingyuan_CD2 | GE_Fork_serenity_Skill1_Damage1 | GE_Fork_serenity_Skill2_Damage |
|---|---|---|---|---|---|---|
| T1 | 5.00 | 12.00% | 5.00% | 25.00 | 24.00% | 18.00% |
| T2 | 5.00 | 14.00% | 5.00% | 25.00 | 28.00% | 21.00% |
| T3 | 5.00 | 16.00% | 5.00% | 25.00 | 32.00% | 24.00% |
| T4 | 5.00 | 18.00% | 5.00% | 25.00 | 36.00% | 27.00% |
| T5 | 5.00 | 20.00% | 5.00% | 25.00 | 40.00% | 30.00% |

## Activation and trigger mechanics

- **Source:** the refinement tooltip above requires HP reduction without taking damage. Positive settled HP consumption grants one stack to the payer, including off-field wearers. Incoming damage, shield absorption, zero consumption and an action tag alone do not grant stacks. The former `ntesim.trigger.hp_cost` tag shortcut has been removed.
- **Modeled policy:** `weapons.jingmotingyuan_v1` uses one wearer-only additive CRIT DMG buff, capped at four stacks. Every positive consumption refreshes the shared five-second duration, including at cap. Expiry is half-open; a cost at expiry starts one fresh stack. Switching preserves the wearer buff and never transfers it to the active teammate. Native independent-versus-shared stack expiry remains unverified.
- **Executable input:** explicit research `hp_cost_events` settle real integer HP consumption and produce typed `hp_cost_receipts`; see the [runner contract](../../../../openspec/specs/combat-execution/reference.md#explicit-hp-consumption). They require `combat.hp_cost_fixture_v1`, bypass shields and incoming-damage callbacks, and preserve at least one HP. Dead actors pay zero. These are declared scenario costs, not recovered native ability costs or automatic Arc dispatch.
- **Arc input:** `baicang arc;` (or another compatible equipped wearer) admits Arc through the shared engine. It requires the living active wearer outside recovery, consumes 5% of current HP once and starts the source-bound 25-second cooldown. Failed admission consumes neither HP nor RNG. Exact HP costs round to nearest integer, with halves upward, and preserve at least one HP under the explicit bounded model. Positive payment grants the CRIT DMG stack before the Arc packets; shields and incoming-damage callbacks remain separate.
- **Attack choice:** one independent seeded stream per wearer chooses equally between Harsh Rebuke (five packets) and Incessant Nagging (six packets, final coefficient doubled). All five refinements use the committed curves above. One choice/cost covers all eligible dummies; damage and crit settle separately for every packet/target. These are weapon packets, so they do not advance character chains or generate ordinary character-hit energy. Switching preserves the wearer cooldown, random stream and pending packets. Owner defeat before same-time hit delivery cancels those packets, retaining the paid cost, draw and cooldown.
- **Timing and recipient hypothesis:** `weapons.jingmotingyuan_v1` resolves the ordered packet group atomically at activation, with no added recovery. The tooltip supplies a cooldown and multiplicity but no checked packet intervals or cast lock; this bounded instantaneous model introduces no invented measured timestamps. It preserves coefficient totals, per-packet quantization, cost-before-hit ordering and the real 25-second repeat interval, but cannot establish native animation/cancel windows or interactions between individual Arc strikes. With no evidenced recipient cap, all in-range stationary dummies receive the train under the product target contract. Equal attack probabilities, cost rounding/nonlethality, native intervals/recovery and native target selection remain unverified. This adapter does not establish complete weapon readiness.

Implementation cross-check (2026-10-21): the [public Arc catalog](https://neverness.app) and [Camellia Society card](https://www.nte.wiki/arcs/camellia-society/) repeat the 5% cost,25-second cooldown and both packet trains; the latter describes an active Arc ability. Neither supplies timestamps or attack probabilities. A finite YouTube/Bilibili search for this weapon and its Chinese name returned acquisition/build leads, including [this acquisition guide](https://www.bilibili.com/video/BV1sLdSBQE21/), but no inspected timed Arc demonstration. No video measurement is claimed. Equal weights are the symmetric two-outcome prior in the absence of weights; the independent total-coefficient check gives1.20/1.26 ATK at T1 and2.00/2.10 at T5, preserving each alternative rather than substituting their average. Timing remains the explicit atomic model above; further timing polishing is deferred.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (666.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Silent Garden (Base ATK 666.0):
    - Base ATK = 600 + 666.0 = 1266.0.
    - Secondary Stat `CritBase` contributes +12.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

### Executable checks

`crates/content/src/simulation/tests/jingmotingyuan.rs` checks all five refinements against an independent damage oracle: ATK1000, coefficient1, DEF factor1/2 and RES factor4/5 give noncritical400. With initial additional CRIT DMG1, T1 stacks produce critical848/896/944/992; a fifth consumption refreshes at992, and expiry returns800. T5 four-stack damage is1120. HP costs50+100+200+100+100 from1000 leave450, independent of damage dealt. Shielded/off-field payment, zero/dead/clamped payment, incoming damage, action-tag rejection, serialization/replay, event budget and malformed input are separate counterexamples. `web/tests/weapon_worker_parity.test.mjs` compares complete native/WASM/Worker reports, aggregation and replay for source-projected T1–T5 builds; it does not duplicate mechanics arithmetic in JavaScript.

Arc checks cover both sampled outcomes at every refinement: noncritical T1 Harsh deals five times96 (=480); Nagging deals five times72 plus144 (=504). T5 yields five times160 (=800) versus five times120 plus240 (=840). From1000 HP, consecutive costs at0/25seconds leave950/902 HP (the second cost47.5 rounds48). Cost-triggered T1 critical damage multiplies the same packet by2.12, retaining its exact fraction until packet quantization. Additional checks cover all-target choice sharing, no stack at one HP, failed/dead/off-field/recovery admission, exact cooldown boundary, RNG isolation and source compilation. Native/WASM/actual Worker checks compare continuing Arc reports, trial aggregation and replay action labels for T1–T5.

Continuing native cycles (`examples/silent-garden-cycles.nte`, `jingmotingyuan_cycles.rs`) run through live source scheduling with the selected Baicang Word/hitlag model. Native lvl80 HP16604 pays Arc5% at0/25s and Word3% on connected inputs at1.930256/25.184377/28.116386s. Independently rounded payments830/473/765/436/423 leave13677 HP. Each actual cost grants one five-second refreshed CRIT DMG stack; the first generation reaches two stacks, the second three. The generating Normal packet uses the prior stack count, while Arc payment grants its stack before Arc children. All five refinements retain exact coefficients (240/400 Harsh,180/300 Nagging with doubled final packet at ATK1000); the selected B4 adds20% of base ATK to both ordinary and weapon packets, so the controlled ATK1000/base1000 case multiplies these by6/5. Two finite11-packet chains, the continuing three-input Word counter, off-field expiry, the31s wrapped Normal, per-cost critical windows and the independent ChargeAdd owner/peer ledger retain native serialization/rerun agreement. Both sampled Arc outcomes reappear across seeds0–7 without another Arc RNG draw from Word costs.
