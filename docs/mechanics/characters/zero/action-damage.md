# Zero action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1046`.

**Responsibility:** Zero outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Continuing Skill Cycle correction (November 13, 2026)

The selected ordinary Skill now fills Zero's own Cycle pool to its existing
100-point capacity once per admitted semantic cast, including Hotori replay.
This corrects the missing fill exposed by curated Blossom/Charge scripts.
The [current Zero guide](https://neverness.app)
describes the Skill's instant 100 Cycle Energy. Cast-time completion, including
a cast without connected damage, is an explicit implementation hypothesis;
native callback timing is not independently established. It does not grant
ordinary Ultimate energy, reset cooldowns or add 100 on top of stored Cycle.
The existing Cycle ledger retains prior charge and records the actual fill
and switch expenditure. Failed admission never reaches the callback.

Legacy S3/S4/S5 request fixtures consequently contain five additional Zero-owned
Blossom pistils. Their independent CLI oracle checks `72000/19` per pistil:
level-80 base 9000, CI zero, Sakiri-reduced DEF 972 against level constant 1080,
and RES 20%. Short energy-ledger fixtures retain the new pending Blossom tail
through its final emission instead of treating that incomplete horizon as success.

Existing Rust source-query coverage now checks two continuing paid Cycle
switches, initial pools 0/100, exact authored CI 348, ten Blossom packets of
14220 each at level80/DEF0/RES0, and native replay. Historical packet/timing
and other unimplemented owner branches below retain their own limits.

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_Female051EffectFigure`, `1046AwakenEffect.json`, `DT_CharacterAbilityConfig.json`, permanent owner inputs (optional `zero.json`), and verification cases in [owning Rust checks](../../../../crates/content/src/simulation/tests/zero_actions.rs). All selected outgoing damage is **Cosmos** (`CHARACTER_ELEMENT_TYPE_COSMOS`), scales with Zero's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch, E represents Skill, Q represents Burst, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Female046_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| `Melee1_Damage` | 1 stage 1 slash / A | .230 / .394 / .426 | 1.450 / 2.400 / .290 |
| `Melee2_Damage` | 1 stage 2 slash 1 / A | .259 / .444 / .479 | 1.620 / 2.700 / .324 |
| `Melee2_1_Damage` | 1 stage 2 slash 2 / A | .224 / .384 / .415 | 1.400 / 2.330 / .280 |
| `Melee3_Damage` | 1 stage 3 thrust / A | .379 / .650 / .702 | 2.370 / 3.960 / .474 |
| `Melee4_Damage` | 1 stage 3 small flurry / A | .064 / .110 / .118 | .400 / .670 / .080 |
| `Melee5_Damage` | 1 stage 3 heavy sweep / A | .385 / .660 / .713 | 2.410 / 4.020 / .482 |
| `Melee6_Damage` | 1 stage 4 rising cut / A | .154 / .264 / .285 | .960 / 1.610 / .192 |
| `Melee7_Damage` | 1 stage 4 falling cleave / A | .997 / 1.709 / 1.845 | 6.230 / 10.420 / 1.246 |
| `Melee8_Damage` | 2 stage 5 execution slashes / A | .683 / 1.171 / 1.264 each | 4.270 / 7.140 / .854 each |
| `Skill1_Damage` | 1 Esper Thrust flurry 1 / E | .200 / .343 / .370 | 1.500 / 2.500 / .300 |
| `Skill2_Damage` | 1 Esper Thrust flurry 2 / E | .200 / .343 / .370 | 1.500 / 2.500 / .300 |
| `Skill3_Damage` | 1 Esper Thrust mid pierce / E | 1.256 / 2.153 / 2.325 | 4.500 / 7.500 / .900 |
| `Skill4_Damage` | 1 Esper Thrust final detonation / E | 1.344 / 2.303 / 2.488 | 5.000 / 8.300 / 1.000 |
| `Skill_Kill_Damage_lv1` | 1 A1 execution bonus strike / E | 2.000 / 2.000 / 2.000 | 0.000 / 0.000 / 2.000 |
| `Skill_Kill_Damage_lv2` | 1 A1 enhanced execution strike / E | 3.000 / 3.000 / 3.000 | 0.000 / 0.000 / 3.000 |
| `UltraSkill1_Damage` | 1 Burst initial slash / Q | 1.327 / 2.274 / 2.456 | 0.000 / 0.000 / 1.500 |
| `UltraSkill2_Damage` | 4 Burst dimensional flurries / Q | .053 / .091 / .098 each | 0.000 / 0.000 / .100 each |
| `UltraSkill3_Damage` | 1 Burst heavy temporal cut / Q | 1.775 / 3.042 / 3.285 | 0.000 / 0.000 / 2.000 |
| `UltraSkill4_Damage` | 1 Burst reality rupture / Q | 1.686 / 2.890 / 3.121 | 0.000 / 0.000 / 2.500 |
| `QTE1_Damage` | 1 Support QTE opening strike / H | .753 / 1.291 / 1.394 | 2.899 / 0.000 / .941 |
| `QTE2_Damage` | 1 Support QTE finisher / H | 1.247 / 2.137 / 2.308 | 4.801 / 0.000 / 1.559 |
| `PerfectEvadeAttack_Damage` | 1 Critical Riposte slash / Z | 1.500 / 2.571 / 2.776 | 5.500 / 9.200 / 1.100 |
| `AirAttack_Damage` | 1 air plunge landing / X | .650 / 1.114 / 1.203 | 3.000 / 5.000 / .600 |

**Action packet totals:** Normal 5-input chain (10 packets across 5 stages) delivers **6.957 ATK / ~27.080 raw energy / 45.380 Cycle / 5.438 Break** at Level 8. Modeled ordinary Skill (4 packets: Skill1..Skill4) delivers **5.142 ATK / ~12.500 raw energy / 20.800 Cycle / 2.500 Break** (plus 2.000/3.000 ratio execution at A1). Modeled direct Burst (7 packets: 1x 1, 4x 2, 1x 3, 1x 4) delivers **8.570 ATK / 0 raw energy / 0 Cycle / 6.400 Break**. Support QTE (2 packets) delivers **3.428 ATK / 7.700 raw energy**. Critical Riposte delivers **2.571 ATK / 5.500 raw energy**.

## Selected complete timing specification

**Mixed native coordinates and real-clock execution:** All action and buff timings progress along one continuous real combat time axis. Source montages advance at 1.0 source units per second, modified by hitstop and pause intervals on positive contact. Overlapping pauses on the same caster take the minimum progress rate rather than serially stacking. Cooldowns and the DPS denominator continue uninterrupted during pauses.

- **Skill (`GA_Female046_Skill`, Esper Thrust):** Cooldown notify starts at **0.120s** (`AnimNotify_TriggerCooldown`); 4 packets land at **0.120s** (`Skill1`), **0.280s** (`Skill2`), **0.550s** (`Skill3`), and **0.850s** (`Skill4`). Cancel window activates at **0.950s** (`BreakSkill_AnySkill`); recovery is **0.800s**; total montage duration is **1.200s**. Real cooldown is **12.0s** (tooltip 6.0s nominal in fast rotation scenarios), continuing off field. Interruption before notify uses cancel cooldown 2.0s; completed casts maintain 12.0s without surcharge.
- **Burst (`GA_Female046_UltraSkill`):** Cooldown **20.0s** (tooltip 15.0s nominal) starts at commitment. Direct gameplay events land across 7 hits from **0.200s** (`UltraSkill1`) to **1.950s** (`UltraSkill4`); cancel window activates at **2.100s** (`BreakSkill_AnySkill`); recovery is **1.200s**; total duration is **2.600s**. Commit costs **100 energy**. Direct hits do not generate ordinary energy.
- **Normal Attack chain (`GA_Female046_Melee`):** Five input stages progress with hits at 0.140s (stage 1: Melee1), 0.120s / 0.240s (stage 2: Melee2, Melee2_1), 0.100s / 0.200s / 0.320s (stage 3: Melee3, Melee4, Melee5), 0.120s / 0.280s (stage 4: Melee6, Melee7), and 0.150s / 0.350s (stage 5: Melee8 x2). Queued full chain completes in **3.200s** (effective total ~4.000s with recovery). An input gap >2.0s resets the chain; legal queued transitions continue.
- **QTE (`GA_Female046_QTE`):** Support switch-in delivers 2 hits at **0.100s** and **0.350s**, recovery window **0.600s**. The switch must trigger a qualifying reaction.
- **Critical Riposte (`GA_Female046_Dodge` / `PerfectEvadeAttack`):** Counter slash at **0.180s**, cancel window **0.600s**, total duration **0.750s**.

## Selected complete state and effect specification

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Ultimate energy cap is 100; UltraSkill commits 100 energy. Ordinary energy awards follow per-packet ChargeAdd upon positive target HP loss: 1/1 to active Zero, 3/5 to living off-field allies, with individual Charge Efficiency, integer capacity, fractional carry, and overflow tracking. Cycle gauge has capacity 100, earned on hits; qualifying switch consumes 100.
- **A1 (Executioner Strike):** When target level <= character level, Skill triggers an extra execution hit `Skill_Kill_Damage_lv1` (2.000 ratio) or `Skill_Kill_Damage_lv2` (3.000 ratio).
- **A2 (Energy Restoration):** Casting Skill refunds 8.0 base energy (`Female051_LV2` = 8.0), scaled by live Charge Efficiency.
- **A3:** Burst packets gain 50 percentage points of Crit Rate (`Female051_LV3` = 0.50). The separate Resonance 3 milestone raises Normal, Skill and Burst talent levels by one.
- **A4:** Burst packets gain 0.1 percentage points of damage bonus per point of Zero's base ATK, up to 25%.
- **A5:** Admitted Support grants Zero +10% base-ATK-scaled ATK for 20s.
- **A6:** The A1 execution packet uses the 300% coefficient.
- **Resonance 6:** `zero.cosmos_cycle_atk_v1` gives every living party member, including Zero, +12% of their own base ATK for 20 World seconds when Zero pays for a qualified Blossom or Remora switch. The outgoing payer identifies the trigger under this selected model. The existing higher-CI reaction damage owner can differ. A second activation refreshes the carrier without stacking. `1046AwakenEffect.json` supplies the trigger wording and `Female051_Extra1_1`/`Female051_Extra1_2` supply amount and duration.

## Selected complete recipient specification

**All-in-range dummy contract and single-target rules:**
- **Burst AoE (`GA_Female046_UltraSkill`):** Under the all-in-range dummy contract, all living scenario dummies receive the 7 Burst hits in ascending target_index order at each hit's scheduled timestamp.
- **Normal attacks and Skill thrusts:** Single-target strikes delivered to the primary target (dummy index 0). Defeated dummies do not absorb hits.

## Complete implementation handoff and two continuing cycles

**Complete build and verification reference:** Built under frozen package `global-2026-10-04` using World Rain (`weapon:fork_worldrain`, T1, base ATK 570), Psyche/Cosmos set (`set:Suit11`), `AtkUp` cartridge, and standard console. Verified static stats: Base ATK 1188, compiled ATK 2766.3 (rounds to 2766), HP 17920, DEF 840, CR 0.05 (0.30 on execution), CD 0.50, CE 1.0.

**Two continuing cycles walkthrough (60.0s period, 120.0s total):**

- **Cycle 1 (0.0s - 60.0s):**
  - **0.000s:** Enters field via QTE (`GA_Female046_QTE`), dealing 3.428 ATK across 2 hits (7.700 raw energy).
  - **1.000s - 4.200s:** Normal Chain Loop 1 (5 inputs, 10 packets: Melee1..Melee8). Delivers 6.957 total ATK, generating ~27.080 raw energy. Accumulated energy: ~34.580.
  - **5.000s - 6.200s:** Skill cast (`GA_Female046_Skill`, Esper Thrust). 4 packets deliver 5.142 total ATK (plus A1 execution hit). A2 refunds 8.0 energy, generating ~20.500 total energy. Nominal cooldown 12.0s triggers. Accumulated energy: ~55.080.
  - **7.000s - 13.600s:** Normal Chain Loops 2, 3 (10 inputs, 20 packets). Generates ~54.160 raw energy. Energy pool reaches 100.0 cap at ~12.0s (overflow safely tracked).
  - **17.000s - 18.200s:** Second Skill cast (`GA_Female046_Skill`, off CD). Delivers 5.142 ATK, refunds energy.
  - **19.000s - 21.600s:** Burst cast (`GA_Female046_UltraSkill`). Commits 100.0 energy and delivers 7 packets for 8.570 total ATK.
  - **22.000s - 60.000s:** Post-burst continuation: 8 Normal chain loops and 3 Skill casts on cooldown. Generates >100 raw energy, refilling energy pool to 100.0 cap by ~45.0s. All cooldowns expire before cycle end.

- **Cycle 2 (60.0s - 120.0s):**
  - **60.000s:** Commences second cycle from live state with 100.0 energy, ready Skill and Burst.
  - **60.000s - 62.600s:** Immediate Burst cast (`GA_Female046_UltraSkill`). Commits 100.0 energy, deals 8.570 ATK, re-applying A6 shred.
  - **63.000s - 64.200s:** Skill cast (`GA_Female046_Skill`), generating ~20.500 energy with A2 refund.
  - **65.000s - 75.000s:** 3 Normal Chain Loops (30 packets), generating >80 raw energy, refilling pool to 100.0 cap.
  - **77.000s:** Second Skill cast.
  - **78.000s - 120.000s:** Sustained Normal and Skill rotation maintains uptime and energy readiness, completing 120.0s simulation without resource leakage, state desync, or reset.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,3,2,2 (ten hits). The chain loops, retaining native effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`zero-normal-chain.json`](../../../../reviews/q10-game-action-binding/zero-normal-chain.json).

## Action packets and owner effects

- GA_Female046_Melee public primary packet is GE_Player_Female046_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.230/0.394/0.426; category A/Cosmos metadata stays source-bound.
- GA_Female046_Skill public primary packet is GE_Player_Female046_Skill1_Damage. Native coefficients at Skill levels 1/8/9 are 0.200/0.343/0.370; category E/Cosmos metadata stays source-bound.
- The ordinary Skill train delivers Skill1, Skill2, Skill3, and Skill4 once each, using the four ordered ratios in frozen GA_Female_Skill_1_des and matching native DT_SkillDamageData rows. Children use the admitted primary impact time; animation intervals and travel remain unresolved.
- GA_Female046_UltraSkill public primary packet is GE_Player_Female046_UltraSkill1_Damage. DT_GameplayAbilityTipsData lists UltraSkill1, UltraSkill2, UltraSkill3, UltraSkill4; the display expression multiplies UltraSkill2 by four. The executable train preserves the primary then emits native UltraSkill2 four times, UltraSkill3 once, and UltraSkill4 once at the primary impact time on default target 0. Under the all-in-range dummy contract (`zero.burst_all_in_range_v1`), every eligible scenario-declared dummy receives that ordered seven-packet train, in ascending target_index order with all seven packets for one target before the next (target-major order); no position, radius, shape, or placement check gates delivery, and defeated or otherwise ineligible dummies keep their existing skip rules. This is not a native recipient-cap or distribution claim. Native packet spacing remains unresolved.
- Each Burst packet uses its own DT_SkillDamageData coefficient and resonance-projected talent coordinate. Independent raw sums for levels 1/8/9 are 5.000/8.570/9.254; packet order is 1,2,2,2,2,3,4. Q/Cosmos metadata stays source-bound.
- Ordinary awakening effects are gated by their selected IDs (A1/A2/A3/A4/A5/A6 map to Effect1/2/3/4/5/6). A6 Effect6 selects the 300% A1 extra hit. Resonance milestones use the number of selected nodes.
- A3 Effect3 adds 0.50 crit rate to the four native Burst packet IDs only at delivery, using shared clamp and per-packet eligible target rolls; FixedCritRate 0 remains the native no-override sentinel. A4 Effect4 contributes min(base_atk*0.001, 0.25) additively in the ordinary damage-bonus zone for those same native Burst packets. The additive-zone mapping is an explicit checked hypothesis; final ATK does not determine this contribution.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill, selecting native coordinate level + 1 without manual overrides.
- A1 Blooming Gaze child packet (extra DMG ignoring 75% DEF when target level <= actor level), A2 Finding the Calling (+8 Ultimate Energy on skill cast), A5 Theopneustos (+10% ATK for 20s on QTE), and A6 Deceptive Liberation (300% A1 hit) remain partial kit models.

## Ordinary hit energy (current model)

The current `zero.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 20.320 and Skill 8.801 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`zero-continuing-energy.json`](../../../../reviews/q10-energy-ledger/zero-continuing-energy.json).

## Automatic Support (bounded executable model)

Source `.zero.mods.support_atk` returns the remaining World seconds of the native
A5 Support ATK buff on a living Zero; `.zero.support_atk.active` reads the same
half-open interval. Known absent effects, including A0, read zero. Native Support
commitment grants20s; field exit preserves its deadline and queries never grant
or refresh it. The public names bind the existing managed effect explicitly,
without exposing raw buff IDs. [Rust source checks](../../../../crates/content/src/simulation/tests/source_managed_modifiers.rs)
exercise two continuing automatic entries, off-field observation, the last
microsecond, exact expiry, unknown names and native replay.

The incoming reaction-qualified switch admits Zero's two-packet QTE.
Under `zero.support_packets_v1`, QTE1 lands at 0.100s and QTE2 at 0.350s.
Recovery lasts 0.600s. Both uncapped Cosmos/category-H
packets hit every living stationary dummy. Each reads live owner/target stats,
retains the entering switch's command and Support origin, and uses the actual
admitted talent coordinate. Switching preserves launched packets; source defeat
cancels pending hits. These recipient and timing rules remain explicit product
models rather than certified native target caps or montage timing.

The frozen rows give 2.899 + 4.801 = **7.700** raw Ultimate Energy per cast and
zero Cycle. This corrects the earlier summary's 3.000/4.500 approximation and
7.500 total. One energy group per packet grants once from its first positive
recipient, including when target zero is defeated or immune. The shared model
grants the source 1/1 and living teammates 3/5, with separate recipient efficiency,
carry, capacity and overflow. The frozen resonance row upgrades Normal/Skill/Burst,
**not QTE**; the earlier summary's QTE +1 claim is superseded. A5 still grants
10% of Zero's base ATK once on admission before both hits.

The [shareable script](../../../../examples/zero-support-cycles.nte) admits three
continuing entries at 0/15/30s without resets. Rust independently checks purchased
levels 1/8/10, selected awakening boundaries, both recipients and command parents,
same-character no-op, qualified re-entry, exact owner defeat, off-field continuation
and implicit-input recovery rejection. At 1000 ATK with no other bonuses, level
1 hits are 753/1247 and level 8 hits 1291/2137; A5's live 1100 ATK gives
1420.1/2350.7 at level 8. Three casts grant Zero 23 with carry 1/10, and a teammate
13 with carry 43/50, independent of two-target fanout. Strict policy preflights
the owner model and every follow-up before any Support resources, cooldowns,
buffs or packets commit. Native/WASM/actual Worker compare the shared script,
including all five weapon refinements and A5/unequal purchased talents, through
complete reports, aggregation and replay.

## Zero-start continuing rotation

The 28-second A0 scenario in [`zero-continuing-energy.json`](../../../../reviews/q10-energy-ledger/zero-continuing-energy.json) and authored [`zero-continuing-energy.nte`](../../../../examples/zero-continuing-energy.nte) schedules 25 ordinary Normal inputs from 0.0s through 19.2s, Skill at 20.0s, Burst at 22.0s, and five more Normal inputs from 24.0s through 27.2s. With synthetic 1000 ATK, target DEF1080/RES0, fixed noncritical damage, and a 28-second denominator, it settles 71 hit receipts and reconciles 27,727 total damage (exact horizon DPS 27,727/28).

Five Normal chains (5 × 20.320 = 101.600) plus Skill (8.801) yield 110.401 energy before the 100-cost Burst, with 10 overflow and 401/1000 carry. Burst spends 100 energy, leaving 0; one continuing chain adds 20.320 energy, leaving the source at 20 energy with 721/1000 carry. Both allies receive 3/5 of each ChargeAdd packet, ending at 78 energy with 2163/5000 carry.

## Individual awakening selection

`[2]` refunds Skill energy without the A1 child; `[3]` adds Burst Crit Rate without R3. Ordinary Effects use their selected identity; resonance uses selection count. `[6]` enables the A1 child upgrade when A1 is also selected. The Cosmos Cycle team buff requires all six nodes.

## A1 first-hit rule (tested hypothesis)

The modeled A1 child triggers on the first nonzero connected Skill hit per admitted activation and target, only when target level `<=` Zero's level. The child retains the struck target, runs after parent settlement, cannot retrigger, and overlapping newly admitted Skills have independent eligibility. The equal-level boundary follows explicit Chinese wording; English localization implies strict `<`. These lifecycle rules are tested hypotheses, not native gameplay verification. See [`zero-a1-model.json`](../../../../reviews/q10-game-action-binding/zero-a1-model.json) and [verify_zero_first_hit.mjs](../../../../tools/verify_zero_first_hit.mjs).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`zero.skill_animation_profile_v1`, `zero.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Male Skill (`char_m_skill_skill_1_Montage`), `AnimNotify_TriggerCooldown` is at 0.000s, gameplay events land at 0.148s (`Skill.1`), 0.266s (`Skill.2`), 0.382s (`Skill.3`), and 1.132s (`Skill.4`); cancel windows activate at 1.641s (`BreakSkill_AnySkill`, `BreakSkill_Jump`) and 1.834s (`BreakSkill_Move`) with total duration 5.317s. For Female Skill (`char_f_skill_skill_1_Montage`), gameplay events land at 0.148s (`Skill.1`), 0.266s (`Skill.2`), 0.382s (`Skill.3`), and 1.100s (`Skill.4`); cancel windows activate at 1.602s (`BreakSkill_Jump`), 1.636s (`BreakSkill_AnySkill`), and 1.834s (`BreakSkill_Move`) with total duration 4.333s. For Burst (`char_m_skill_utraskill_Montage` / `char_f_skill_utraskill_Montage`), `UltraSkill.1` lands at 2.168s, four `UltraSkill.2` hits land at 2.323s, 2.454s, 2.609s, and 2.746s, `UltraSkill.3` lands at 2.854s, and `UltraSkill.4` lands at 3.749s; cancel windows activate at 4.170s (`BreakSkill_Move`, `BreakSkill_AnySkill`, `BreakSkill_Jump`) with total duration 6.733s (male) / 6.400s (female).
- **Multi-target delivery (`zero.aoe_all_in_range_dummy_v1`, `zero.burst_all_in_range_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Zero's ordered seven-packet Burst train (1x `GE_Player_Female046_UltraSkill1_Damage`, 4x `GE_Player_Female046_UltraSkill2_Damage`, 1x `GE_Player_Female046_UltraSkill3_Damage`, 1x `GE_Player_Female046_UltraSkill4_Damage`) across all living dummies in ascending target_index order (target-major order) at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/zero-action-binding-model.json`](../../../../reviews/q10-game-action-binding/zero-action-binding-model.json).
- Continuing energy: [`reviews/q10-energy-ledger/zero-continuing-energy.json`](../../../../reviews/q10-energy-ledger/zero-continuing-energy.json).
- Owner supplements: [verify_zero_first_hit.mjs](../../../../tools/verify_zero_first_hit.mjs), [`zero-a1-model.json`](../../../../reviews/q10-game-action-binding/zero-a1-model.json), [owning Rust checks](../../../../crates/content/src/simulation/tests/zero_actions.rs), [`zero-normal-chain.json`](../../../../reviews/q10-game-action-binding/zero-normal-chain.json).
- Frozen inputs: `data/extracted/guide_zero.json`, `data/extracted/neverness_zero.json`, [consumed native damage rows](../../../../crates/data/src/native_damage/female046.rs) (original `DT_SkillDamageData` evidence is cache-only), `1046AwakenEffect` (cache-only awakening source), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), [`zero/parameters.rs`](../../../../crates/content/src/characters/zero/parameters.rs) (consumed typed parameters; original `DT_Female051EffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed constant A1-A5 parameters are typed in `zero/parameters.rs`; source-only EffectFigure rows are cache-only. This migration preserves the existing A3 crit-rate and A4 capped additive damage models despite older summaries above. Independent Rust checks cover the 249/250/251 base-ATK boundary, packet applicability and two A2 casts with 1.4 efficiency, fractional carry and capacity overflow. Python retains native/WASM transport and deterministic replay checks.

- Hold/plunge actions and remaining resource/passive/action rules remain unimplemented.
- A2 Skill energy refund and A5 QTE ATK buff are already in the known model; do not reclassify those supported slices as research blockers.

## Full-kit source investigation (October 17–18, 2026)

Full extraction across client asset data, ability blueprints, and combat tables records Zero's serialized damage rows and configured passive/awakening references; reachability and execution remain open:

### Serialized damage-row inventory (reachability unverified)

| Ability ID | Gameplay Effect Class | Cat | Base Ratio (Lv 1/8/9) | ChargeAdd | Hetero | Unbalance | Description / Inferred Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee1_Damage` | A | 0.230 / 0.394 / 0.426 | 1.151 | 1.914 | 0.232 | Normal 1 |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee2_Damage` | A | 0.259 / 0.444 / 0.479 | 1.297 | 2.157 | 0.261 | Normal 2 hit 1 |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee2_1_Damage` | A | 0.224 / 0.384 / 0.415 | 1.122 | 1.865 | 0.226 | Normal 2 hit 2 |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee3_Damage` | A | 0.379 / 0.650 / 0.702 | 1.900 | 3.158 | 0.382 | Normal 3 flurry |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee4_Damage` | A | 0.064 / 0.110 / 0.118 | 0.319 | 0.530 | 0.064 | Normal 4 rapid strike |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee5_Damage` | A | 0.385 / 0.660 / 0.713 | 1.927 | 3.205 | 0.388 | Normal 5 main strike |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee6_Damage` | A | 0.154 / 0.264 / 0.285 | 0.771 | 1.282 | 0.155 | Normal 6 transition strike |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee7_Damage` | A | 0.997 / 1.709 / 1.845 | 4.993 | 8.303 | 1.005 | Normal 7 heavy slash |
| `GA_Female046_Melee` | `GE_Player_Female046_Melee8_Damage` | A | 0.683 / 1.171 / 1.264 | 3.420 | 5.687 | 0.688 | Normal 8 finisher slash |
| `GA_Female046_Melee` | `GE_Player_Female046_AirAttack_Damage` | X | 0.650 / 1.114 / 1.203 | 3.300 | 5.400 | 0.700 | Aerial attack plunge |
| `GA_Female046_ExtremEvadeAtk` | `GE_Player_Female046_PerfectEvadeAttack_Damage` | Z | 1.500 / 2.571 / 2.776 | 3.800 | 6.300 | 2.500 | Perfect Evade counter |
| `GA_Female046_QTE` | `GE_Player_Female046_QTE1_Damage` | H | 0.753 / 1.291 / 1.394 | 2.899 | 0.000 | 0.941 | QTE hit 1 |
| `GA_Female046_QTE` | `GE_Player_Female046_QTE2_Damage` | H | 1.247 / 2.137 / 2.308 | 4.801 | 0.000 | 1.559 | QTE hit 2 |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill1_Damage` | E | 0.200 / 0.343 / 0.370 | 0.587 | 0.973 | 0.120 | Skill: Appraise hit 1 |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill2_Damage` | E | 0.200 / 0.343 / 0.370 | 0.587 | 0.973 | 0.120 | Skill: Appraise hit 2 |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill3_Damage` | E | 1.256 / 2.153 / 2.325 | 3.683 | 6.110 | 0.753 | Skill: Engrave strike 1 |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill4_Damage` | E | 1.344 / 2.303 / 2.488 | 3.944 | 6.543 | 0.807 | Skill: Engrave finisher |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill_Kill_Damage_lv1` | E | 2.000 / 2.000 / 2.000 | 0.000 | 0.000 | 0.000 | Skill extra damage (level/marker predicate, tier 1) |
| `GA_Female046_Skill` | `GE_Player_Female046_Skill_Kill_Damage_lv2` | E | 3.000 / 3.000 / 3.000 | 0.000 | 0.000 | 0.000 | Skill extra damage (level/marker predicate, tier 2 / A6) |
| `GA_Female046_UltraSkill` | `GE_Player_Female046_UltraSkill1_Damage` | Q | 1.327 / 2.274 / 2.456 | 0.000 | 0.000 | 0.398 | Burst: Divide by Zero opening hit |
| `GA_Female046_UltraSkill` | `GE_Player_Female046_UltraSkill2_Damage` | Q | 0.053 / 0.091 / 0.098 | 0.000 | 0.000 | 0.016 | Burst: Divide by Zero multi-ticks (x4) |
| `GA_Female046_UltraSkill` | `GE_Player_Female046_UltraSkill3_Damage` | Q | 1.775 / 3.042 / 3.285 | 0.000 | 0.000 | 0.533 | Burst: Divide by Zero collapse hit |
| `GA_Female046_UltraSkill` | `GE_Player_Female046_UltraSkill4_Damage` | Q | 1.686 / 2.890 / 3.121 | 0.000 | 0.000 | 0.506 | Burst: Divide by Zero finale blast |
| `GA_Female046_UltraSkill_Fantasy` | `GE_Player_Female046_UltraSkill1_boss034_fantasy_Damage` | Q | 12.000 / 12.000 / 12.000 | 0.000 | 0.000 | 1.500 | Fantasy Boss special burst execution |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee1_Damage` | A | 0.226 / 0.226 / 0.226 | 1.130 | 1.883 | 0.228 | Unarmed stance basic 1 |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee2_Damage` | A | 0.272 / 0.272 / 0.272 | 1.362 | 2.270 | 0.274 | Unarmed stance basic 2 |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee3_Damage` | A | 0.323 / 0.323 / 0.323 | 1.614 | 2.691 | 0.325 | Unarmed stance basic 3 |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee4_Damage` | A | 0.159 / 0.159 / 0.159 | 0.794 | 1.323 | 0.160 | Unarmed stance basic 4 hit 1 |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee4_1_Damage` | A | 0.133 / 0.133 / 0.133 | 0.665 | 1.109 | 0.134 | Unarmed stance basic 4 hit 2 |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee4_2_Damage` | A | 0.591 / 0.591 / 0.591 | 2.959 | 4.931 | 0.596 | Unarmed stance basic 4 finisher |
| `GA_Female046_Melee_UnArmed` | `GE_Player_Female046_unarmed_Melee5_Damage` | A | 1.174 / 1.174 / 1.174 | 5.876 | 9.794 | 1.183 | Unarmed stance basic 5 |
| `GA_Female046_UltraSkill_UnArmed` | `GE_Player_Female046_unarmed_UltraSkill1_Damage` | Q | 1.000 / 1.000 / 1.000 | 0.000 | 0.000 | 0.500 | Unarmed burst hit 1 |
| `GA_Female046_UltraSkill_UnArmed` | `GE_Player_Female046_unarmed_UltraSkill2_Damage` | Q | 1.000 / 1.000 / 1.000 | 0.000 | 0.000 | 0.500 | Unarmed burst hit 2 |
| `GA_Female046_UltraSkill_UnArmed` | `GE_Player_Female046_unarmed_UltraSkill3_Damage` | Q | 1.000 / 1.000 / 1.000 | 0.000 | 0.000 | 0.500 | Unarmed burst hit 3 |

### Passives, Awakenings, and Reactions

Source descriptions and selected IDs: `1046AwakenEffect#/rows/1046/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **Passives:**
  - `GA_Female_Passive_1`: Binds `Buff_Female051_Passive1_C` (`/Game/Blueprints/Abilities/Player/Ability_051_Female/PassiveEffect/Buff_Female051_Passive1`).
  - `GA_Female_Passive_2`: Binds `Buff_Female051_Passive2_C` (`/Game/Blueprints/Abilities/Player/Ability_051_Female/PassiveEffect/Buff_Female051_Passive2`).
- **Awakenings:**
  - A1 (`Effect1`): Deals 1 additional hit of 200% ATK to enemies of lower level on first hit. This damage ignores 75% of target's DEF.
  - A2 (`Effect2`): Grants an additional 8 Ultimate Energy after successfully casting Appraise and Engrave.
  - A3 (`Effect3`): Increases Divide by Zero's Crit Chance by 50%.
  - A4 (`Effect4`): Every 1 point of Zero's Base ATK increases Ultimate: Divide by Zero damage by 0.1%, up to a maximum of 25%.
  - A5 (`Effect5`): Increases Zero's ATK by 10% for 20s when triggering Support Skill.
  - A6 (`Effect6`): Increases Extra DMG Ratio of Appraise and Engrave to 300% of ATK.
  - Resonance 3 (`resonance_3`): +1 level to Appraisal, Appraise and Engrave, and Divide by Zero.
  - Resonance 6 (`resonance_6`): Grants the team 12% ATK for 20s when Zero triggers any Cosmos Esper Cycle.
- **Attribute scaling:**
  - Scaled strictly via `AtkRateBaseArray` under Cosmos attribute. No alternate HP or DEF scaling tables are configured.
- **Reactions:**
  - Binds Reaction 7 (Cosmos / GuangLingXiang) via `GE_Reaction_GuangLingXiang_1009_C`.

### Extra-Skill level predicates and A4 consumer recovered, October 18

The initial `GA_Female046_Skill#/4/Properties/EffectContainerMap` requests both `GE_Player_Female046_Skill_Kill_Damage_lv1` and `...lv2` from its Skill.1 and Skill.3 entries. Each GE's `/2/Properties/ApplicationRequirements` selects `Con_FemaleSkill_LV1` / `LV2`; the names **Kill** do not imply an HP threshold or require a target death.

On valid attacker/target ASCs, `Con_FemaleSkill_LV1:CanApplyGameplayEffectContent` (`#/2`, statements 197..1067) requires `Buff_Player_Female051_SkillDamageTrigger`. With Level1 present and Level6 absent it compares **attacker level >= target level**; otherwise it uses **attacker level > target level**. `Con_FemaleSkill_LV2` requires Level6 and the same trigger marker; its Level1 branch uses >= and the other branch uses >. Consequently equal-level A1+A6 can admit only the second predicate, while lower-level targets can make both predicates true at A6. Do not infer two settled extra hits: `Buff_Player_Female051_SkillDamageTrigger#/1` requests `Buff_Female051_Skill_BuffDelete` on SKILL_AFTER_DAMAGE and removes itself on SKILL_FINISH, so native effect-list/callback ordering and marker consumption still govern multiplicity. These warning-free graphs resolve the predicate inputs and expose the overlap; the existing bounded A1 model remains a separate choice.

`Buff_Female051_Level4#/1/Properties/CustomApplicationModifierInfos` places **Female_DamUp_LV4** on additive **DamageUpGeneralBase**. That calculator (`#/2`, statements 739..1094) reads owner GetAtkBase and computes `min(base ATK * Female051_LV4_1 * Female051_LV4_2, Female051_LV4_3)`, with saved rows **1, 0.001, 0.25**. This establishes the source stat/zone and cap for the reached consumer; no final-ATK multiplier or independent bonus zone is justified. At base ATK 100/250/1000 its successful magnitude is 0.10/0.25/0.25. Modifier eligibility/capture and historical 1009-versus-1046 description variants must remain explicit.

### Passive description coverage, reviewed October 18

Saved description source: `guide_zero.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Appraiser describes the active character receiving healing equal to 50% of Zero base ATK when gaining Ultimate Energy through Charge. Anomaly Perception describes Divide by Zero damage +25%. These are distinct from Skill AddHeterochromeValue and A4 base-ATK scaling; native Charge attribution, heal routing/frequency, modifier eligibility and zero/overflow-energy handling remain open.

### Misleading passive-condition name excluded, October 18

The initial `Con_FemalePas2:CanApplyGameplayEffectContent` (`#/2`) does **not** test the second passive marker. It casts GameplayEffect and returns **AssetTagHasAllMatchingGameplayTags([Ability.Player.Female, Ability.UltraSkill, State.Damage])**. It gates the already located `Buff_Female051_Level4` A4 modifier. The similarly named historical `Buff_Female009_Level4` also references it, but uses a different calculator/stat destination. Therefore this condition is not a recovered native grant/consumer for the described +25% Anomaly Perception passive; keep source identity and stat selection separate.

### Unarmed Skill stamina request recovered, October 18

The expansion's `GA_Female046_Skill_UnArmed#/5/Properties/EffectContainerMap` maps **Event.Montage.Player.Common.Display** to owner-targeted `Buff_Female046_SkillEnergyCost`. That effect (`#/2/Properties/Executions`) names **StrengthExecution** with a Source/snapshot additive SkillCoefficient custom magnitude from `Calc_Female_Unarmed_EngergyCost`. The warning-free calculator (`#/2`) returns explicit **-35** on a successful OwnerActor player cast; it does not inspect target level, MaxHP or Ultimate Energy. The source is thus a stamina mutation request, not a 35 Ultimate Energy cost or an energy grant implied by the asset name. Native affordability, efficiency, settlement frequency/refund and natural unarmed form admission remain open. No cost is inferred for other unarmed actions.

### Charge healing basis and avatar lookup recovered, October 18

Initial `GE_Reaction_GuangLingXiang_1009#/2/Properties/Executions` names **CustomReplyHealthExecution**, supplying Source/snapshot additive SkillCoefficient from `Cau_GuangLingXiang1_Female`. Its warning-free `#/2` helper casts **OwnerActor** to player, validates its player state, and reads `Reaction_GuangLingXiang1_Recover_1051=0.5` at coordinate 0. It first looks up the configured **Female="1051"** party character; if valid it returns **that character's GetAtkBase * 0.5**. Only if that lookup is invalid does it attempt **male="1046"** and use that base ATK. It does not select the maximum party ATK, use the recipient's ATK, or read the copied Female_Pas_2 handles/Adler tag container into the result. A valid chosen base ATK 1000 supplies **500**, even if final ATK is 1500.

The carrier declares HasDuration scalar **1**, period **2**, AggregateByTarget, limit 1 and NeverRefresh. Those values do not prove two-second repeated healing or a single heal without the native immediate-execution/caller context. The selected 1046 reaction row legitimately names the 1009 asset; the variant lookup and delivery/recipient/zero-energy callback policies remain explicit.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Female046_Melee#/7; projectile/spawn entry | PlayerFemaleMelee9 | (0.0, 0.0, 0.15, 1.35, 7, true) | GE_Player_Female_Melee9_Damage |
| GA_Female046_Melee#/8; projectile/spawn entry | PlayerFemale051Melee3 | (0.0, 0.0, 0.12, 0.35, 2, true) | GE_Player_Female051_Melee3_Damage |
| GA_Female046_Skill#/5; Skill.4 | PlayerFemaleSkill4New | (0.0, 0.0, 0.0, 2.5, 1, true) | GE_Player_Female046_Skill4_Damage, GE_Player_Female046_Skill_Kill_Damage_lv1, GE_Player_Female046_Skill_Kill_Damage_lv2 |
| GA_Female046_Melee_UnArmed#/7; projectile/spawn entry | PlayerFemaleMelee9 | (0.0, 0.0, 0.15, 1.35, 7, true) | GE_Player_Female_Melee9_Damage |
| GA_Female046_Melee_UnArmed#/8; projectile/spawn entry | PlayerFemale051Melee3 | (0.0, 0.0, 0.12, 0.35, 2, true) | GE_Player_Female051_Melee3_Damage |
| GA_Female046_Skill_UnArmed#/6; projectile/spawn entry | PlayerFemaleSkill4New | (0.0, 0.0, 0.0, 2.5, 1, true) | GE_Player_Female051_Skill4_Damage, GE_Player_Female051_Skill_Kill_Damage_lv1, GE_Player_Female051_Skill_Kill_Damage_lv2 |

The armed/unarmed Melee actors explicitly select shared Female_Melee9_Damage and Female051_Melee3_Damage, not newly invented Female046 versions. Their event-map tag could not be joined through SpawnActorMaps in this projection, so the actor container is located without claiming a specific natural Normal input. Armed Skill.4 uses Female046_Skill4_Damage and Female046 kill classes; the unarmed actor instead selects **Female051_Skill4_Damage** and Female051_Skill_Kill_Damage_lv1/lv2. The unarmed event tag has not been joined through this map. Native kill/level conditions remain necessary; these are not three guaranteed simultaneous packets.

### Female/male cue candidates checked, October 18

The corrected external-cue selection reads **12 female/male-folder candidates**, all warning-free, rather than treating the earlier folder whitelist as full-roster coverage. The actual carrier join is **Buff_Female046_PrisonAbility#/2 → GameplayCue.Display.Male.prison → Male_PrisonAbility#/6**. Its OnWhile **#/3 → graph#/1:498** caches **GetOwner as player** and requests **SwitchAnimationMode(1,0.2,false)**. Tick **#/4 → 197** repeats that request only if the current animation-mode byte differs from 1; OnRemove **#/2 → 661 → 10** resolves GetOwner again and requests mode0 with the same 0.2 argument. The numeric animation-mode values and blending input do not establish armed/unarmed ability arbitration, weapon loss or a damage clock. The Female_PrisonAbility name alone is not a substitute for this Male-tag join.

The other candidates supply material/camera/animation and SetCustomTimeDilation requests, without a located Cosmos Cycle/R6 team-ATK producer. In particular their Female Skill1/SkillBoss/slomo tags are not admitted as this current 1046 kit merely by sharing a folder or name. Continue source-bound ability/carrier dependencies and the R6 event producer; do not convert cosmetic curves into action durations. Sources: `cue-zero-functions.txt` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Skill extra-packet multiplicity:** Level/marker predicates and both requesting event entries are located above. Resolve marker consumption and effect-list/callback order, especially the A6 lower-level-target overlap; do not reopen the disproved HP-threshold/target-death interpretation.
- **Unarmed moveset state transitions:** Resolve the conditions under which Zero drops or sheathes her weapon to transition into `GA_Female046_Melee_UnArmed` and `GA_Female046_UltraSkill_UnArmed`.
- **Boss fantasy burst variant:** Resolve trigger conditions and admission rules for `GE_Player_Female046_UltraSkill1_boss034_fantasy_Damage` (1200% ratio) vs standard 7-packet Divide by Zero.
- **Resonance 6 native callback:** The simulator uses the outgoing Cycle payer. The exact native carrier callback remains unverified.
- **Shared native boundary:** Saved asset/Blueprint data supplies selected requests and markers. The existing protected-body and real-time investigation does not recover this owners dispatch/cancel clock; no applicable command/hit/switch trace has been established here. See the shared timing owner and the specific unresolved policies above.


Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Burst packets: exact spacing remains unknown. Under `zero.burst_all_in_range_v1`, the seven-packet train is delivered to every eligible scenario-declared dummy per the all-in-range dummy contract; the frozen source does not bind native recipient caps/selection/distribution. Resolve those policies; do not investigate native geometry/radius/placement.

## Prior findings

- [`reviews/q10-game-action-binding/zero-action-binding-model.json`](../../../../reviews/q10-game-action-binding/zero-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [verify_zero_first_hit.mjs](../../../../tools/verify_zero_first_hit.mjs), [`zero-a1-model.json`](../../../../reviews/q10-game-action-binding/zero-a1-model.json), [owning Rust checks](../../../../crates/content/src/simulation/tests/zero_actions.rs), [`zero-normal-chain.json`](../../../../reviews/q10-game-action-binding/zero-normal-chain.json).

## Research handoff

Use a Global-cutoff-bound source or a controlled Global combat trace to resolve each question. Preserve conflicting localized and native evidence, identify the specific effect on the damage/action model, and move only resolved mechanics to the known card. Do not write implementation code as part of research.


## Research history

- **Current question:** Extra-Skill marker consumption/order and unarmed action admission; R6 team callback source/recipients.
- **Useful source locations:** `GA_Female046_Skill`, `Con_FemaleSkill_LV1`, `Con_FemaleSkill_LV2`, `Buff_Player_Female051_SkillDamageTrigger`, `Buff_Female051_Level4`, `Female_DamUp_LV4`, 1046AwakenEffect.
- **Methods already tried:** Recovered exact source/target-level inequalities, A1/A6 marker split, deletion requests and A4 base-ATK/zone/cap. The extra rows have no located HP-execute predicate. Kept 1009/1046 description variants distinct. Shared protected-body/timing findings are retained; no game code was executed.
- **Next concrete step:** Recover native marker removal and delivered-effect ordering, unarmed form arbitration and Cosmos Cycle callback routing, or attributable observations distinguishing the alternatives. Existing synthetic execution checks cannot establish those native rules.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. Zero has an empty serialized pause component; that does not imply a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
