# Skia action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1008`.

**Responsibility:** Skia outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Current Rust entry-state boundary

When a Normal hit consumes Fang Thrust before the timed Hound attack, the
coordinated Shadow Attack retains that hit's `parent_hit_index`, command, owner,
target and equal-time origin under [ADR 0015](../../../decisions/0015-damage-parent-identity.md).
It is not a separately admitted Skill. The timed Hound path still originates in
the actual Skill command. The existing Rust mark-consumption check independently
retains the 1,350,100us contact and verifies the Normal parent; live source
equipment/Worker parity covers report transport, aggregation and replay.

Public source Skill now selects this native control model. `.skia.mods.gnaw` reads the remaining5s interval from actual grant at cast+1.048984s; `.skia.gnaw.active` reads its half-open permission. Field exit clears the native deadline as well as queued control boundaries, so off-field queries and returning entry selectors cannot observe stale control. Owner defeat clears it through the native kit cleanup. [Source checks](../../../../crates/content/src/simulation/tests/source_forms.rs) cover two continuing Skill/switch cycles without restoring control on return, natural expiry at6.048984s, the last microsecond and owner defeat, with native replay. Querying never grants control or consumes a target mark.

`skia.gnaw_entry_state_v1` binds the six native Skill contacts at.153592/.216117/.296382/.369313/.871421/.913168, the Skill.4 ShadowControl grant at1.048984, ordinary recovery at1.335873 and a12s CD starting at the first notify. These are rounded source coordinates from `Skia_Skill_001`, not the older approximate clock below or measured game seconds. The located `GA_Skia_Skill#/6` Skill.4 container requests ShadowControl/ShadowControl2; the control carrier/base sneak cue supplies native state38 and the five-second duration. A living active owner enters this actual state; no party-only/initial stealth flag is accepted. Teammate entry selectors capture it before outgoing cleanup. Field exit/death invalidates the control generation and removes its queued boundaries without flushing an unadmitted exit attack.

This prerequisite covers the Skill-to-team-entry path, including two continuing native party cycles. It does not silently claim the remaining Hound kit. Reaching the marked-target coordination deadline at state+.35, a natural outro, Normal/Support exit or incoming-HP attack-type dependency diagnoses. A safety horizon before retained required work is incomplete and cannot rank as a successful comparison. The existing packet/awakening policies remain separate; Hound attacks/outro, clones, full Fang Thrust lifecycle and other forms still belong to C-Skia.

The frozen `DT_SkillDamageData` and existing typed rows independently give Skill Charge **1.08*4+.166+.914=5.4**, Cycle **1.8*4+.277+1.523=9** and Break **.22*4+.034+.186=1.1**. This corrects the7.6 resource estimate in the historical walkthrough below; that walkthrough is not a Rust resource oracle. No runtime constants change to match the estimate. Two teammate-entry cycles use the exact5.4 source pool and fractional off-field distribution.

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_SkiaEffectFigure`, `1008AwakenEffect.json`, `DT_CharacterAbilityConfig.json`, permanent owner inputs (optional `skia.json`), and verification cases in [owning Rust checks](../../../../crates/content/src/simulation/tests/skia_actions.rs). All selected outgoing damage is **Lakshana** (`CHARACTER_ELEMENT_TYPE_LAKSHANA`), scales with Skia's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch, E represents Skill, Q represents Burst, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Skia_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| `Melee1_Damage` | 1 stage 1 slash / A | .304 / .521 / .563 | 1.900 / 3.200 / .380 |
| `Melee2_Damage` | 2 stage 2 quick slashes / A | .037 / .063 / .068 each | .230 / .390 / .046 each |
| `Melee4_Damage` | 1 stage 2 follow-up thrust / A | .189 / .324 / .350 | 1.180 / 1.990 / .236 |
| `Melee5_Damage` | 1 stage 3 sweep / A | .321 / .550 / .594 | 2.010 / 3.370 / .402 |
| `Melee6_Damage` | 1 stage 3 uppercut / A | .459 / .787 / .850 | 2.870 / 4.820 / .574 |
| `Melee7_Damage` | 1 stage 4 overhead chop / A | .391 / .670 / .724 | 2.440 / 4.100 / .490 |
| `Melee8_Damage` | 1 stage 5 lead pierce / A | .294 / .504 / .544 | 1.840 / 3.090 / .368 |
| `Melee9_Damage` | 1 stage 5 execution burst / A | .554 / .949 / .1025 | 3.460 / 5.820 / .692 |
| `Skill1_Damage` | 4 Shadow Hound Chase bites / E | .300 / .514 / .555 each | 1.080 / 1.800 / .220 each |
| `Skill2_Damage` | 1 Shadow Hound Chase mid strike / E | .046 / .079 / .085 | .166 / .277 / .034 |
| `Skill3_Damage` | 1 Shadow Hound Chase pounce / E | .254 / .435 / .470 | .914 / 1.523 / .186 |
| `SkillShadowAtk_Damage` | 1 coordinated shadow strike / E | .550 / .943 / 1.018 | 2.500 / 4.200 / .500 |
| `UltraSkill1_Damage` | 1 The Pack spread strike / Q | 2.902 / 4.974 / 5.371 | 0.000 / 0.000 / 3.000 |
| `UltraSkill2_Damage` | 3 The Pack converge bites / Q | .699 / 1.198 / 1.294 each | 0.000 / 0.000 / .700 each |
| `ShadowAtk_Damage` | 1 shadow hound tick / A | .100 / .171 / .185 | .500 / .800 / .100 |
| `ShadowAtk1_Damage` | 1 enhanced shadow bite / A | .200 / .343 / .370 | 1.000 / 1.700 / .200 |
| `ShadowAtk2_Damage` | 1 heavy shadow rend / A | .300 / .514 / .555 | 1.500 / 2.500 / .300 |
| `QTE1_Damage` | 1 Support QTE strike / H | 2.000 / 3.428 / 3.702 | 7.500 / 0.000 / 2.400 |
| `PerfectEvadeAttack_Damage` | 1 Critical Riposte slash / Z | 2.080 / 3.565 / 3.850 | 5.500 / 9.200 / 1.100 |
| `AirAttack_Damage` | 1 air plunge landing / X | 1.000 / 1.714 / 1.851 | 4.000 / 6.700 / .800 |

**Action packet totals:** Normal 5-input chain (9 packets: 1 + 3 + 2 + 1 + 2) delivers **4.431 ATK / ~16.160 raw energy / 27.170 Cycle / 3.234 Break** at Level 8. Modeled ordinary Skill (6 hits: 4x Skill1 + 1x Skill2 + 1x Skill3) delivers **2.570 ATK / 5.400 raw energy / 9.000 Cycle / 1.100 Break**. Modeled direct Burst (4 hits: 1x UltraSkill1 + 3x UltraSkill2) delivers **8.568 ATK / 0 raw energy / 0 Cycle / 5.100 Break**. Critical Riposte delivers **3.565 ATK / 5.500 raw energy**. Support QTE delivers **3.428 ATK / 7.500 raw energy**.

## Selected complete timing specification

**Mixed native coordinates and real-clock execution:** All action and buff timings progress along one continuous real combat time axis. Source montages advance at 1.0 source units per second, modified by hitstop and pause intervals on positive contact. Overlapping pauses on the same caster take the minimum progress rate rather than serially stacking. Cooldowns and the DPS denominator continue uninterrupted during pauses.

- **Skill (`GA_Skia_Skill`, Shadow Hound Chase):** Cooldown notify starts at **0.120s** (`AnimNotify_TriggerCooldown`); 6 packets land from **0.120s** (`Skill1` flurries) to **0.750s** (`Skill3` pounce). Cancel window activates at **0.850s** (`BreakSkill_AnySkill`); recovery is **0.800s**; total montage duration is **1.150s**. Real cooldown is **12.0s** (tooltip 6.0s nominal in fast rotation scenarios), continuing off field. Interruption before notify uses cancel cooldown 2.0s; completed casts maintain 12.0s without surcharge.
- **Burst (`GA_Skia_UltraSkill`, The Pack):** Cooldown **20.0s** (tooltip 15.0s nominal) starts at commitment. Direct gameplay events land at **0.250s** (`UltraSkill1` spread) and **1.100s..1.600s** (`UltraSkill2` converge bites); cancel window activates at **1.900s** (`BreakSkill_AnySkill`); recovery is **1.200s**; total duration is **2.400s**. Commit costs **100 energy**. Direct hits do not generate ordinary energy.
- **Normal Attack chain (`GA_Skia_Melee`):** Five input stages progress with hits at 0.140s (stage 1: Melee1), 0.120s / 0.220s / 0.320s (stage 2: Melee2 x2, Melee4), 0.150s / 0.300s (stage 3: Melee5, Melee6), 0.200s (stage 4: Melee7), and 0.150s / 0.350s (stage 5: Melee8, Melee9). Queued full chain completes in **3.200s** (effective total ~4.000s with recovery). An input gap >2.0s resets the chain; legal queued transitions continue.
- **QTE (`GA_Skia_QTE`):** The selected bounded Support model uses a 0.100-second impact and 0.600-second recovery. The native packet grants 6.700 raw energy and 2.500 Break under their respective delivery rules. A qualified reaction switch admits this Support with no independent cooldown.
- **Critical Riposte (`GA_Skia_Dodge` / `PerfectEvadeAttack`):** Counter slash at **0.160s**, cancel window **0.600s**, total duration **0.750s**.

## Selected complete state and effect specification

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Ultimate energy cap is 100; UltraSkill commits 100 energy. Ordinary energy awards follow per-packet ChargeAdd upon positive target HP loss: 1/1 to active Skia, 3/5 to living off-field allies, with individual Charge Efficiency, integer capacity, fractional carry, and overflow tracking. Cycle gauge has capacity 100, earned on hits; qualifying switch consumes 100.
- **Fang Thrust / Shadow Lock (A3 / Passive):** Locked target gains `skia.fang_thrust` status, granting +8% total damage bonus (`Skia_Leve3DamageUpGeneralBase1` = 0.04 and `Skia_Leve3DamageUpGeneralBase2` = 0.04) against single locked targets even in multi-target scenarios.
- **A1 (Shadow Damage Amplification):** Increases shadow attack damage ratio by +60% (`Skia_Leve1CoefModify` = 0.60).
- **A2 (Shadow Prowess):** Grants +10% Crit Rate (`Skia_Leve2CritBase` = 0.10) for 15.0s (`Skia_Leve2Duaration` = 15.0) upon skill or burst activation.
- **A3 (Resonance 3 & Fang Thrust):** Grants +1 talent level to Skill, UltraSkill, QTE; arms the +8% single-target Fang Thrust damage bonus.
- **A4 (Pack Coordination):** Increases shadow coordination frequency and pounce recovery speed.
- **A5 (Hounds Energy):** Provides energy refund on shadow bites, accelerating cycle transition.
- **A6 / R6 (Lakshana Rend):** Attacks apply Lakshana resistance shred of -2% (`Skia_LevelExtraDamageResistLakshanaBase` = -0.02) per stack up to 5 stacks (-10% total shred).

## Selected complete recipient specification

**All-in-range dummy contract and single-target rules:**
- **Burst Spread AoE (`UltraSkill1_Damage`):** Under the all-in-range dummy contract, all living scenario dummies receive the initial spread slash in ascending target_index order at the primary impact timestamp.
- **Burst Converge bites (`UltraSkill2_Damage`):** Focus on the primary target (dummy index 0).
- **Normal attacks and Skill bites:** Delivered to primary target (dummy index 0). Defeated dummies do not absorb hits.

The bounded runtime now applies this Burst spread scope through the shared native
AoE packet service. Spread deliveries retain their native talent coefficient and
category Q, share one packet group, and precede the primary-only converge train.
Already defeated secondary dummies are omitted; a dead primary cannot absorb
converge bites or prevent the spread from reaching other living dummies. Rust
checks levels1/8/9 independently at ATK1000 and DEF/RES0: each living recipient
takes2902/4974/5371 spread damage and the primary takes three additional
699/1198/1294 converge packets. No Ultimate energy is created by this zero-charge
train. The [Be Happy continuing script](../../../../examples/be-happy-defeat-cycles.nte)
also exercises spread defeats, owner attribution and shared weapon cooldowns in
native/WASM/Worker consumers. Normal/Skill targeting and the unbound Gnaw/clones
remain separate owner branches.

## Complete implementation handoff and two continuing cycles

**Complete build and verification reference:** Built under frozen package `global-2026-10-04` using Kite (`weapon:fork_Kite`, T1, base ATK 445), Lakshana/Cosmos set (`set:Suit11`), `AtkUp` cartridge, and standard console. Verified static stats: Base ATK 969, compiled ATK 2458.2 (rounds to 2458), HP 18230, DEF 760, CR 0.05 (0.15 with A2), CD 0.50, CE 1.0.

**Two continuing cycles walkthrough (60.0s period, 120.0s total):**

- **Cycle 1 (0.0s - 60.0s):**
  - **0.000s:** Enters field via QTE (`GA_Skia_QTE`), dealing 3.428 ATK (7.500 raw energy).
  - **1.000s - 4.200s:** Normal Chain Loop 1 (5 inputs, 9 packets: Melee1..Melee9). Delivers 4.431 total ATK, generating ~16.160 raw energy. Accumulated energy: ~23.660.
  - **5.000s - 6.150s:** Skill cast (`GA_Skia_Skill`, Shadow Hound Chase). 6 packets deliver 2.570 total ATK, generating ~7.600 raw energy. Triggers A2 +10% CR for 15.0s. Nominal cooldown 12.0s triggers. Accumulated energy: ~31.260.
  - **7.000s - 16.600s:** Normal Chain Loops 2, 3, 4, 5 (20 inputs, 36 packets). Generates ~64.640 raw energy. Accumulated energy reaches ~95.900.
  - **17.000s - 18.150s:** Second Skill cast (`GA_Skia_Skill`, off CD). Delivers 2.570 ATK, generates ~7.600 raw energy, capping pool at 100.0 (overflow safely tracked).
  - **19.000s - 21.400s:** Burst cast (`GA_Skia_UltraSkill`, The Pack). Commits 100.0 energy (pool resets to 0.0). Delivers 4 packets for 8.568 total ATK.
  - **22.000s - 60.000s:** Post-burst continuation: 9 Normal chain loops and 3 Skill casts on cooldown. Generates >100 raw energy, refilling energy pool to 100.0 cap by ~45.0s. All cooldowns expire before cycle end.

- **Cycle 2 (60.0s - 120.0s):**
  - **60.000s:** Commences second cycle from live state with 100.0 energy, ready Skill and Burst.
  - **60.000s - 62.400s:** Immediate Burst cast (`GA_Skia_UltraSkill`). Commits 100.0 energy, deals 8.568 ATK.
  - **63.000s - 64.150s:** Skill cast (`GA_Skia_Skill`), generating ~7.600 energy.
  - **65.000s - 78.000s:** 4 Normal Chain Loops (36 packets), generating ~64.640 energy.
  - **80.000s:** Second Skill cast, bringing energy pool to 100.0 cap.
  - **81.000s - 120.000s:** Sustained Normal and Skill rotation maintains uptime and energy readiness, completing 120.0s simulation without resource leakage, state desync, or reset.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted inputs deliver Melee1; Melee2 x2 followed by Melee4; Melee5 followed by Melee6; Melee7; Melee8 followed by Melee9. Native packet names do not follow stage numbering. Published coefficients establish this mapping at all nine levels. The five stages deliver nine hits and loop with effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`skia-normal-chain.json`](../../../../reviews/q10-game-action-binding/skia-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`skia.skill_animation_profile_v1`, `skia.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Skia_Skill_001`), `AnimNotify_TriggerCooldown` is at 0.154s, Quad Strike hits (`GE_Player_Skia_Skill1_Damage` x 4) land at 0.154s, 0.216s, 0.296s, and 0.369s (`Event.Montage.Player.Skill.1`), Dive hit 1 (`GE_Player_Skia_Skill2_Damage`) at 0.871s (`Event.Montage.Player.Skill.2`), and Dive hit 2 (`GE_Player_Skia_Skill3_Damage`) at 0.913s (`Event.Montage.Player.Skill.3`); cancel windows activate at 0.938s (`BreakSkill_Jump`) and 1.336s (`BreakSkill_Move`, `BreakSkill_AnySkill`). For Burst (`Skia_UltraSkill`), direct hits: Spread hit 1 (`GE_Player_Skia_UltraSkill1_Damage`) at 2.482s (`Event.Montage.Player.UltraSkill.1`), and Converge hits 1..3 (`GE_Player_Skia_UltraSkill2_Damage` x 3) at 3.233s, 3.337s, and 3.502s (`Event.Montage.Player.UltraSkill.2`); cancel windows for Move, AnySkill, and Jump activate simultaneously at 4.167s (`BP_CanBreakSkill_C`).

## Action packets and owner effects


- GA_Skia_Melee public primary packet is GE_Player_Skia_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.304/0.521/0.563; category A/LAKSHANA metadata stays source-bound.
- GA_Skia_Skill delivers 6 instances under timing hypothesis skia.skill_burst_packets_v1: primary GE_Player_Skia_Skill1_Damage (Quad Strike hit 1), three child GE_Player_Skia_Skill1_Damage (Quad Strike hits 2..4), one child GE_Player_Skia_Skill2_Damage (Dive hit 1), and one child GE_Player_Skia_Skill3_Damage (Dive hit 2). Native total coefficients at Skill levels 1/8/9 are 1.500/2.570/2.775; category E/LAKSHANA metadata stays source-bound. on_cast refreshes skia.a2_crit_rate (+10% Crit Rate for 15s at A2). before_hit applies skia.fang_thrust status (10s, 12s at A4) and 2% Lakshana RES reduction at A6.
- GA_Skia_UltraSkill delivers 4 instances under timing hypothesis skia.skill_burst_packets_v1: primary GE_Player_Skia_UltraSkill1_Damage (Spread hit 1) and three child GE_Player_Skia_UltraSkill2_Damage (Converge hits 1..3). Native total coefficients at Burst levels 1/8/9 are 4.999/8.568/9.253; category Q/LAKSHANA metadata stays source-bound.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill without manual overrides.
- Existing A1 Fang Thrust auto-attack ratio bonus (+60%), A2 Skill Crit Rate buff (+10% for 15s), A3 locked target DMG bonus (+8%), and A6 Lakshana RES reduction (2%) remain preserved as partial kit models.
- Canonical arbitrary awakening selections: Effect1 Fang Thrust coefficient addition, Effect2 timed crit, Effect3 marked-target bonus and Effect4 mark duration use selected identities. Talent resonance_3 and marked-target resistance resonance_6 remain count-based. Existing unsupported owner mechanics remain pending; selection does not complete the kit.

## Configured Shadow Control ability grant

**Serialized source fact, not native dispatch.** The compact effect table (effect_parameters.jsonl), record `Buff_Skia_ShadowControl`, retains `/4/Properties` `AbilitiesGameplayEffectComponent.GrantAbilityConfigs`: `Ability=GA_Skia_ShadowEvade`, explicit level parameter **1**, InputID **1**, and removal-policy label **`CancelAbilityImmediately`**. The source identifies the granted class and configured fields; it does not establish Shadow Hound/clone packets, successful evade admission or actual cancellation delivery. Native grant/input/cleanup and related action behavior remain in [unknown](#unknown-execution-policies).

## Ordinary-hit Ultimate Energy

The current `skia.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 21.5, Skill 5.4 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`linko-skia-hathor-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/linko-skia-hathor-ordinary-hits.json).

### Zero-start continuing rotation

The checked A0 slice uses five five-input Normal chains, two modeled Skills, one 100-energy Burst, then one more Normal chain after the spend. All 70 damage packets are checked in order; the 66 ordinary Normal/Skill packets use the existing energy hypothesis. With 1000 ATK, zero bonuses, and a 0.5 defense multiplier, the independent total is 20,147 (six Normal chains at 2,215.5 each, two Skills at 1,285 each, Burst at 4,284). ChargeAdd reaches 118.3 before Burst: 118 whole energy is generated, 18 overflows at capacity 100, and the Burst spends 100. The continuation grants 21 more whole energy, leaving 21 energy and 4/5 carry; each other living recipient ends at 83 with 22/25 carry.

Inputs use the tested 800ms Normal recovery, 1.2s Skill recovery/6s cooldown, and 2s Burst recovery. The 40s NTE rotation, Rust request, source/ally packet ledgers, observations, aggregation/replay and native/WASM/Worker output are checked together. This establishes only the named ordinary chain/Skill/Burst sequence and existing energy/packet hypotheses; native eligibility, packet timing, readiness waiting, Fang Thrust mark consumption, alternate forms and shadow-clone/Hound mechanics remain open. Receipt: [`skia-continuing-energy.json`](../../../../reviews/q10-energy-ledger/skia-continuing-energy.json).

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/skia-action-binding-model.json`](../../../../reviews/q10-game-action-binding/skia-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/skia_actions.rs), [`skia-normal-chain.json`](../../../../reviews/q10-game-action-binding/skia-normal-chain.json), [`skia-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/skia-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_skia.json`, `data/extracted/neverness_skia.json`, [consumed typed packet values](../../../../crates/data/src/native_damage/skia.rs) (`DT_SkillDamageData` source remains cache-only), `1008AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed EffectFigure parameters are typed in [`skia/parameters.rs`](../../../../crates/content/src/characters/skia/parameters.rs); unconsumed source rows are cache-only. The existing owner model and its remaining implementation limits are preserved.

- AirAttack/riposte, ShadowAtk/ShadowAtk1–2/SkillShadowAtk, Shadow Hound Gnaw, and Fang Thrust mark consumption remain unimplemented.

## Full-kit source investigation (October 17–18, 2026)

Full extraction across client asset data, ability blueprints, and combat tables establishes Skia's complete native damage packet inventory, passives, and awakening modifiers:

### Serialized damage-row inventory (reachability unverified)

| Ability ID | Gameplay Effect Class | Cat | Base Ratio (Lv 1/8/9) | ChargeAdd | Hetero | Unbalance | Description / Inferred Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee1_Damage` | A | 0.304 / 0.521 / 0.563 | 2.525 | 4.217 | 0.505 | Normal 1 hit |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee2_Damage` | A | 0.037 / 0.063 / 0.068 | 0.306 | 0.510 | 0.061 | Normal 2 hit (x2) |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee4_Damage` | A | 0.189 / 0.324 / 0.350 | 1.572 | 2.625 | 0.314 | Normal 3 finisher |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee5_Damage` | A | 0.321 / 0.550 / 0.594 | 2.667 | 4.454 | 0.533 | Normal 4 hit 1 |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee6_Damage` | A | 0.459 / 0.787 / 0.850 | 3.820 | 6.379 | 0.764 | Normal 4 hit 2 |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee7_Damage` | A | 0.391 / 0.670 / 0.724 | 3.250 | 5.427 | 0.650 | Normal 5 hit 1 |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee8_Damage` | A | 0.294 / 0.504 / 0.544 | 2.448 | 4.087 | 0.490 | Normal 5 hit 2 |
| `GA_Skia_Melee` | `GE_Player_Skia_Melee9_Damage` | A | 0.554 / 0.949 / 1.025 | 4.606 | 7.691 | 0.921 | Normal 5 finisher |
| `GA_Cang_Melee` | `GE_Player_Cang_Skia_Damage` | A | 0.675 / 1.157 / 1.249 | 2.300 | 3.800 | 0.500 | Cang synergy coordinated strike |
| `GA_Skia_Melee` | `GE_Player_Skia_ShadowAtk_Damage` | A | 0.100 / 0.171 / 0.185 | 0.000 | 0.000 | 0.000 | Shadow clone strike basic |
| `GA_Skia_Melee` | `GE_Player_Skia_ShadowAtk1_Damage` | A | 0.200 / 0.343 / 0.370 | 0.000 | 0.000 | 0.000 | Shadow clone strike stage 1 |
| `GA_Skia_Melee` | `GE_Player_Skia_ShadowAtk2_Damage` | A | 0.300 / 0.514 / 0.555 | 0.000 | 0.000 | 0.000 | Shadow clone strike stage 2 |
| `GA_Skia_Melee` | `GE_Player_Skia_AirAttack_Damage` | X | 1.000 / 1.714 / 1.851 | 5.000 | 8.300 | 1.000 | Aerial attack plunge |
| `GA_Skia_ShadowEvadeEnd` | `GE_Player_Skia_Evade1_Damage` | E | 0.500 / 0.857 / 0.925 | 3.300 | 5.600 | 0.700 | Shadow Evade counter hit |
| `GA_Skia_ExtremEvadeAtk` | `GE_Player_Skia_PerfectEvadeAttack_Damage` | Z | 2.080 / 3.565 / 3.850 | 5.200 | 8.700 | 2.500 | Perfect Evade counter |
| `GA_Skia_QTE` | `GE_Player_Skia_QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 6.700 | 0.000 | 2.500 | Switch-in QTE strike |
| `GA_Skia_Skill` | `GE_Player_Skia_Skill1_Damage` | E | 0.300 / 0.514 / 0.555 | 1.080 | 1.800 | 0.220 | Skill: Quad Strike hit |
| `GA_Skia_Skill` | `GE_Player_Skia_Skill2_Damage` | E | 0.046 / 0.079 / 0.085 | 0.166 | 0.277 | 0.034 | Skill: Dive hit 1 |
| `GA_Skia_Skill` | `GE_Player_Skia_Skill3_Damage` | E | 0.254 / 0.435 / 0.470 | 0.914 | 1.523 | 0.186 | Skill: Dive hit 2 |
| `GA_Skia_Skill` | `GE_Player_Skia_SkillShadowAtk_Damage` | E | 0.550 / 0.943 / 1.018 | 1.814 | 3.029 | 0.364 | Skill: Shadow Hound follow-up strike |
| `GA_Skia_UltraSkill` | `GE_Player_Skia_UltraSkill1_Damage` | Q | 2.902 / 4.974 / 5.371 | 0.000 | 0.000 | 0.871 | Burst: Spread initial burst hit |
| `GA_Skia_UltraSkill` | `GE_Player_Skia_UltraSkill2_Damage` | Q | 0.699 / 1.198 / 1.294 | 0.000 | 0.000 | 0.210 | Burst: Converge multi-hit |
| None | `GE_Player_Skia_GuangXiangFive_Damage` | P | 2.000 / 2.000 / 2.000 | 0.000 | 0.000 | 0.000 | Awakening / Passive 5 bonus damage trigger |

### Passives, Awakenings, and Reactions

Source descriptions and selected IDs: `1008AwakenEffect#/rows/1008/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **Passives:**
  - `GA_Skia_Passive_1`: Binds `Buff_Skia_Passivity1_C` (`/Game/Blueprints/Abilities/Player/Ability_008_Skia/Passivity/Buff_Skia_Passivity1`).
  - `GA_Skia_Passive_2`: Binds `Buff_Skia_Passivity2_C` (`/Game/Blueprints/Abilities/Player/Ability_008_Skia/Passivity/Buff_Skia_Passivity2`).
- **Awakenings:**
  - A1 (`Effect1`): Increases auto-attack DMG Ratio for Fang Thrust (`skiamelee1`) by 60%.
  - A2 (`Effect2`): Increases Skia's Crit Chance by 10% and Interruption Resistance by 50% for 15s after casting Shadow Hound Chase.
  - A3 (`Effect3`): Increases damage to targets locked by Fang Thrust by 4%; if only one target is locked onto on field, increases damage by an additional 4% (isolated target bonus).
  - A4 (`Effect4`): Extends duration of Tailed (`skiaskill1`) to 12s. Increases Attachment DMG by 2% per second target is attached, up to 40%.
  - A5 (`Effect5`): Increases target's mass by 50 and reduces its Movement Speed by 20% for each Fang Thrust locked onto it.
  - A6 (`Effect6`): Increases Extra DMG Ratio of Fang Thrust by 20% while in Shadow Hound Gnaw mode (`skiaskill2`) when touching an enemy locked onto by Fang Thrust.
  - Resonance 3 (`resonance_3`): +1 level to Arresting Art, Shadow Hound Chase, and The Pack.
  - Resonance 6 (`resonance_6`): Reduces locked-on target's Lakshana RES by 2% per Fang Thrust, up to 6%.
- **Attribute scaling:**
  - Scaled strictly via `AtkRateBaseArray` under Lakshana attribute. No alternate HP or DEF scaling tables are configured.
- **Reactions:**
  - `ReactionAbilityID`: `GA_Skia_Passive_1` with condition `Buff_Skia_Passivity1_C`.

### A4 attachment-time staircase recovered, October 18

`Cau_Skia_Level4_DamUp:CustomCalculateBaseMagnitude` in the initial bundle (`#/2`, statements 27..634) reads **HitActor ASC GetAttachmentBuffExistTime**. On valid ASC and positive returned duration, it applies **FFloor** before multiplying by scalable `DampUpPerSecond`, then clamps to `[0, MaxDamUp]`. The CDO binds `Skia_Leve4DamageupPersecond` and `Skia_Leve4DamageupMax` (native spelling), and serializes TrustClient=true and ClampValue=(0,10,0). The function's explicit return clamp uses MaxDamUp, not that unrelated three-vector.

The successful expression is `clamp(floor(attachment_duration) * per_second, 0, cap)`, not continuous growth. In `DT_SkiaEffectFigure#/rows`, `Skia_Leve4DamageupPersecond=0.02` and `Skia_Leve4DamageupMax=0.4`. Supplied durations 0.99/1.00/1.99/2.00 therefore give 0/0.02/0.02/0.04; 20 and 21 both give 0.4. `Buff_Skia008_Level4#/1` requests `Buff_Skia008_Level4_DamUp` at BEGIN; the child's `/2/Properties/CustomApplicationModifierInfos` binds this calculator with multiplier 1 to additive **DamageUpGeneralBase**, requiring source tag **State.Damage.Attachment**. Its Infinite duration policy does not mean permanent attachment or unconditional 40% bonus. Native attachment identity, timer reset/refresh and capture clock and actual Hound/clone dispatch remain open; the missing duration consumer must not be replaced by time since arbitrary mark creation. The cited helpers/carriers are warning-free.

`Buff_Skia_ShadowControl2#/2/Properties/DurationMagnitude` selects `Cau_Skia_Level4`. Its warning-free function (`#/2`, statements 79..413) selects `Skia_ShadowControlDuaration_Level4` when the owner has `Buff_Skia008_Level4`, otherwise `Skia_ShadowControlDuaration`: **12 versus 6** in the same figure table. CDO variables named `duaration=10` and `level4 duaration=20` are not read by this function and cannot replace its curve results. The older 10-second base used in the modeled mark remains a model/source conflict; source duration selection is now located, while delivery clock and mark refresh remain open.

### Passive description coverage, reviewed October 18

Saved description source: `guide_skia.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Suppression describes a new Remora application when Fang Thrust locks onto a target already in Remora. Apprehension describes Fang Thrust damage +10% for 15s after The Pack, including its Shadow Hound collision branch. The packet consumers and ordinary bonus zone are recovered below. Natural lock/hit recognition, reaction refresh and collision eligibility remain open; a repeated mark export does not establish these events.

### Apprehension packet consumers recovered, October 18

`Con_Skia_Passivity2:CanApplyGameplayEffectContent` (`#/2`, warning-free) validates SourceActor ASC and requires **both** `Buff_Skia_Passivity2` and `Buff_Player_Skia_UltraSkillPassivity2`. Its constructed tag array is unused by the returned condition. The timed marker (`#/1`) binds `Skia_Passivity2DamageupDuaration=15`, AggregateBySource, limit 1; missing refresh policy is not a second explicit duration rule.

The condition gates additive **DamageUpGeneralBase**, multiplier 1, through `Skia_Passivity2DamageupGeneralBase=0.1` on `GE_Player_Skia_SkillShadowAtk_Damage#/3` and expansion `GE_Player_Skia_ShadowAtk_Damage`, `ShadowAtk1_Damage`, `ShadowAtk2_Damage`, all `#/2/Properties/CustomApplicationModifierInfos`. This locates the selected 10% modifier consumers instead of assigning it to all Normal/Skill rows. Actual Burst marker request, collision/clone dispatch, capture and refresh remain open; known consumers and their stat zone no longer are.

### Short shadow carriers have a separate duration selector, October 18

Initial `Buff_Skia_ShadowBase#/2`, `Buff_Skia008_Level5_1#/1`, and expansion `Buff_Skia_ShadowBase_GuangXiang#/1/Properties/DurationMagnitude` select `Cau_Skia_ShadowBaseDuaration`. Its warning-free `#/2` function checks owner `Buff_Skia008_Level1` and selects `Skia_ShadowAtkDuaration_Level1` versus `Skia_ShadowAtkDuaration`; **both saved rows are 2** at coordinate 0. Its copied CDO literals 3/10 and Mint target class are unused. These short shadow-carrier inputs are distinct from the 6/12-second Tailed control selector above; do not overwrite every Shadow duration with one of those values. Native carrier application, refresh, collision events and clock remain open.

### Shadow exit override retains a null cleanup class, October 18

Initial warning-free **GA_Skia_ShadowEvadeEnd:K2_ActivateAbility** (`#/6` → `/5`) calls **GA_Skia_SneakEndBase** first, then on the valid avatar-player path requires the controller's GetPlayerCharacter to equal that avatar. Its following **K2_RemoveGameEffectsWithClass** request passes **null class, -1 stacks, null source**, not a named Shadow/Tailed carrier. This source cannot establish removal of every buff, a specific Hound carrier or successful sneak cleanup without the native null-class semantics and parent body. The reached override supplies no packet or timer rule that would close Hound dispatch. Rendering: ; do not repeat this export as a missing named cleanup binding.

### Shadow-Evade and swallow custom damage inputs recovered, October 18

The warning-free initial **Execution_Skia_ShadowEvade:CustomCalculateBaseMagnitude#/2** returns **OwnerActor ASC GetAtk() × max(HitActor ASC GetGameplayEffectCount(Buff_Skia_ShadowBase,null,true),0)**. It reads current GetAtk, not GetAtkBase, and passes no source filter to the target's effect-count query. With declared ATK=1500 and count=3 the magnitude is **4500**; base ATK=1000 would incorrectly give 3000. [Native query parameter metadata](../../shared/buffs-and-callbacks.md#effect-request-and-count-query-declarations) identifies the trailing arguments as **OptionalInstigatorFilterComponent=null, bEnforceOnGoingCheck=true**. Do not replace this particular API's effect-count result with a presumed sum of stacks or a source-isolated count. **GE_Player_Skia_EvadeShadow_Damage#/3** supplies it with multiplier 1 as an **Override of target/non-snapshot FixedDamageValue** in FixedDamageExecution; the same execution separately overrides Source/snapshot SkillCoefficientMult=1 and ExecutionDamageType=6. Native fixed-damage settlement, defenses and caller attribution remain distinct from this located magnitude.

**Buff_Skia_ShadowEvadeCheck#/1** is Infinite with configured period **0.3** and source aggregation/cap 1. Its PERIODIC event requests that damage through DamageRange_Skia_EvadeShadow; its separate **SKILL_AFTER_DAMAGE** entry requests self **Buff_Skia_ShadowEvadeDamCD**. That CD carrier (`#/1`) lasts **2**, source-aggregated/cap 1, NeverRefresh/NeverReset. The packet's **Con_Skia_EvadeAtk#/2** returns true on IsHybridMode_Server(SourceActor); otherwise a valid SourceActor player requires absence of that CD class. This identifies both the guard grant request and the mode bypass, without assuming the generic after-damage callback is filtered to this packet or that every 0.3-period request lands.

The warning-free expansion **Execution_Skia_Swallow#/2** validates HitActor and OwnerActor as ability characters and returns **min(OwnerActor base ATK × 1, HitActor.GetHP() × 0.2)**. Samples with owner base ATK 1000 and supplied GetHP values 2000/10000 yield **400/1000**. Both **GE_Player_Skia_Evade2_Damage#/2** and **GE_Player_Skia_Swallow_Damage#/2** bind this helper, multiplier 1, to **additive Source/snapshot Damage** in **MaxHPDamageExecution**. The helper's explicit GetHP read must not be replaced by MaxHP just because the enclosing executor has MaxHP in its name. Final executor semantics, reachability of these two packets, HP-read timing, shadow consumption and native sneak cleanup remain open. Cached source renderings: , `passive-execution-bindings-compact.txt` and `skia-linko-passive-carriers.txt`.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Skia_Melee#/7; Melee.9 | PlayerSkiaMelee9 | (0.0, 0.0, 1.0, 0.0, 1, true) | GE_Player_Skia_Melee9_Damage |
| GA_Skia_Melee#/8; Melee.7 | PlayerSkiaMelee7 | (0.0, 0.0, 1.0, 0.0, 1, true) | GE_Player_Skia_Melee7_Damage |
| GA_Skia_ShadowEvadeEnd#/8; Evade.4 | PlayerSkiaEvade4 | (0.0, 0.0, 1.0, 0.0, 1, true) | GE_Player_Skia_Evade4_Damage |
| GA_Skia_ShadowEvadeEnd#/9; Evade.6 | PlayerSkiaEvadeWall | (0.0, 0.0, 1.0, 3.0, 1, true) | no explicit damage class |
| GA_Skia_Skill#/7; Skill.5 | PlayerSkiaSkill1 | (0.0, 0.0, 0.15, 3.0, 4, true) | GE_Player_Skia_Skill1_Damage |
| GA_Skia_UltraSkill#/9; UltraSkill.1 | PlayerSkiaUltraSkill1 | (0.0, 0.0, 0.0, 0.0, 0, true) | no explicit damage class |

Skill.5 binds the four-count/0.15-period configuration to Skill1_Damage. The Melee7/9 and Evade4 rows instead retain duration=0 with MaxTriggerCount=1; do not discard their damage as zero-lifetime events. EvadeWall has no explicit GE payload, and UltraSkill's zero-count/null container does not supply another Burst packet.

### Sneak exit admission, section reset and control removal recovered, October 18

Warning-free **GA_Skia_SneakEndBase:K2_CanActivateAbility#/3** returns **NOT avatar-player.IsUsingUltraSkill()** with empty RelevantTags on its successful cast; it does not call another Blueprint admission override. **K2_GetCustomStartSection#/4** returns **ShadowEvadeEnd** when CustomStartSectionName is None, otherwise that explicit name. Activation (`#/2 → /1`, 47..515) calls GA_MontageBase, then only when GetSkillStartSectionName equals CustomStartSectionName rotates the valid avatar by a composed yaw-180 transform. EndAbility (`#/5 → /1`) calls GameplayAbility's parent and resets **CustomStartSectionName=None**. This is a located exit selector/reset, not proof of an elapsed stealth duration, resource consumption or guaranteed turn in every exit path.

**GA_Skia_ShadowEvadeEnd1#/2** explicitly calls this base's activation. Its **#/4 GAComponent_AddOrRemoveGE** requests **ActivatedRemoveGEs=[Buff_Skia_ShadowControl1]**. CDO **#/3** binds Evade.1 to **Evade1_Damage/HitBack**, while wall/landing selectors choose their own configured sections. This places a control-marker removal at an admitted exit ability, separate from the previously located Evade4 actor and shadow-damage calculations. Native component ordering, removal success and other shadow/control state remain open.

The separately inspected **GA_Skia_Evade:TriggerJumpSectionImpl#/4 → /2**, 587..666, requests **K2_EndAbility** on exact section **EvadeEnd** and otherwise forwards to the montage parent. Activation's **State.Property.SeakOff** branch only requests a client HUD message; it is not a returned CanActivate predicate or proof that native admission ignores the tag. Sources: `remaining-action-and-target-functions.txt` and `orbit-and-sneak-config.txt`.

### Shadow generation and damage predicates recovered, October 18

Warning-free **Con_Skia_MeleeCheck/SkillCheck:CanApplyGameplayEffectContent#/2** read **TargetActor ASC.LastDamageGEDef** and require **all** of **Ability.Player.Skia, Ability.Melee/Ability.Skill, State.Damage**, respectively. They inspect the last-damage effect, not the currently tested helper carrier or every team Normal input. Their reached consumers **Buff_Skia_ShadowAddMelee/ShadowAddSkill#/2** gate BEGIN requests for **ShadowBase** with **UseSourceObject=true**: Melee's ExecGEs stack argument is **-1**, Skill's is **3**. Both helper carriers have duration0.5; neither duration supplies a damage cadence or native stack-default interpretation.

The actual producer **Buff_Skia_ShadowCheck#/1** names both helpers in a **BUFF_EVENT_UNKNOW**, TargetTrigger=true entry, and self-removes at **SKILL_AFTER_DAMAGE**. Preserve that serialized unknown event label; the native invocation/order is not recovered by relabeling it AFTER_DAMAGE. The destination **ShadowBase#/2** is target-aggregated, cap 3, DeadRemove=true, NeverRefresh/NeverReset with StackExpirationPolicy=RefreshDuration and a custom duration helper. This locates the payload/gates/cap while leaving event delivery, last-damage capture, per-source ownership and expiration semantics open.

**SkillShadowAtk_Damage#/2** separately requires **Con_Skia_SkillshadowCheck#/2**, whose successful predicate is **TargetActor ASC has ShadowBase**. It is actually requested periodically by **Buff_Skia_ShadowAtk#/1** through DamageRange_Skia_ShadowAtk; its duration input9999 and **Skia_SkillSneakAtkPeriod=0.35** are not a complete summon lifetime or real tick clock. The **ShadowControl#/2** carrier requests ShadowAtk on BEGIN and removes it plus ShadowEvadeCheck on FINISH; it self-removes at MELEE_BEGIN, QTE_BEGIN, CHANGE_ROLE_OUT and HITOUT. Its own condition (`Con_Skia_ShadowControl#/2`) requires an Instigator player and either **IsHybridMode_Server(instigator)** or **GetIsControlledCharacter()**, alongside the separate NotIgnoreUseAbilityDetectionConditions requirement. The copied Skia/Skill/Damage array in that condition is unused. The carrier configures SneakDuaration=5 and grants ShadowEvade/InputID1/CancelAbilityImmediately, not a new unconditional input path.

The three ordinary **ShadowAtk/ShadowAtk1/ShadowAtk2_Damage#/2** bind **CoefModify Override=0.6** under **Con_Skia_ShadowAttackCheck**, which checks **SourceActor's Level1 marker**. SkillShadowAtk instead binds **CoefModify Override=0.2** under the separate Level6 condition. Their shared Burst bonus **DamageUpGeneralBase=0.1** is gated by **Con_Skia_Ultraskill3_Buff**, checking **SourceActor ASC has Buff_Player_Skia_UltraSkill3_**. These are located modifier inputs/destinations, not 0.6/0.2 final coefficients or arbitrary independent damage multipliers; native CoefModify composition/capture remains separate.

**Buff_Skia_ShadowPas#/1/ExtraDamageCoefficientForTarget2** declares magnitude1, **bRequireOwnerAsInstigator=true** and **Con_Skia_ExtraShadowDamage#/2**. That warning-free predicate explicitly accepts only gameplay-effect classes **ShadowAtk1_Damage, ShadowAtk2_Damage, ShadowAtk_Damage or EvadeShadow_Damage**, **AND TargetActor ASC has UltraSkill_ToTarget**; invalid effect/ASC returns explicit false. **SkillShadowAtk is absent** from this exact class list. This resolves eligibility without inventing the native extra-damage executor, a second packet or a blanket twofold Shadow multiplier. The separate ShadowBase_GuangXiang predicate requires target **Buff_Reaction_3_new** and Instigator **Buff_Skia_Passivity1**; its BEGIN reaction request is not a recovered new reaction cadence. Sources: `skia-condition-callers.txt`, `skia-shadow-consumers.txt`, `skia-shadow-producer-callers.txt`, `skia-and-linko-trigger-carriers.txt` and `ninth-trigger-config.txt`.

### Shadow passive's target sets and reset callbacks recovered, October 18

Warning-free **Buff_Skia_ShadowPas#/1** names **GameplayCue.Display.Skia.ShadowPas**. Newly read **GC_Skia_ShadowPas:OnWhileActiveEvent#/6 → ExecuteUbergraph#/1** caches MyTarget player, requires **IsAuthorityOrAutonomousHybridMode**, binds the player's ASC **OnAbilityJumpSectionDelegate**, subscribes **AsyncTaskCharacterAttacked(self,player,false,true)**, and binds player **OnAbilityEndEvent / OnActivateAbilityEvent**. The attack callback **#/4** validates DamageGE and checks its asset tags. **Skia + State.Damage + Ability.Melee** takes priority: an unseen DamagedCharacter is added to **ShadowCharacter_Melee**, then receives one **ShadowBase request**. Otherwise **Skia + State.Damage + Ability.Skill** requires an unseen target in **ShadowCharacter_Skill**, adds it, and loops integers 1..3 to request ShadowBase three times. [Native application parameter metadata](../../shared/buffs-and-callbacks.md#effect-request-and-count-query-declarations) identifies cached Skia in **SourceObj**, and the explicit cached-player ASC in **pTriggerAbilityComp**, with level1/stack0/strength1. These are supplied application inputs, not a recovered settled-instigator/source-filter contract. Seen targets do not receive those repeat requests on that callback route. There is no additional visible InstigatorCharacter==cached-player or DamageAmount>0 predicate; the [native callback parameter metadata](../../shared/buffs-and-callbacks.md#native-task-parameter-declarations-recovered-october-18) identifies **bGlobal=false, bAutoEnd=true**, while native event routing/lifetime remains separate.

**ProcessAbilityJumpSection#/7** clears the Melee set when the supplied ability's object-string contains **GA_Skia_Melee**, or the Skill set for **GA_Skia_Skill**, using case-insensitive substring checks. It is called for activation/end of the supplied ability, and by the jump-section delegate **only after the animating ability is valid and its string contains GA_Skia_Melee**. The ChangeSectionName input is stored but unused in that branch. Thus Normal-section changes can rearm its target set; this is not one global once-per-cast/target cap shared with Skill, nor three globally guaranteed native stacks per Skill hit. The separate requirement/carrier routes described above remain distinct; source requests cannot be added together as confirmed deliveries.

OnRemove **#/5** unbinds jump/activation/end delegates when cached player is valid, clears cached player and both target sets. The reached graph stores no attack-task handle or explicit EndTask call for that task. Native teardown, cue duplication/re-entry, zero-stack convention, ShadowBase cap and actual grant ordering remain open. Function streams are complete; the subsequent HaloBuffActor recovery rereads the CDO without its initial tail warning and confirms GameplayCue.Display.Skia.ShadowPas as its configured tag. Sources: `cue-cang-skia-tag-callers.txt` and `roster-cue-extraction.json`.

### Source-filtered Shadow handle and time-change damage request recovered, October 18

The repaired **Buff_Skia_ShadowBase#/2 → GameplayCue.Display.Skia.Shadow → GC_Skia_Shadow** join locates an actual ordinary-Shadow packet selector. **OnWhileActiveEvent#/8 → graph#/1:1251** returns if its saved time-change task is already valid. Otherwise it resolves incoming EffectContext **InstigatorActor** and MyTarget as ability characters. The combat-initialization gate is **IsHybridMode_Client(instigator) OR (!IsHybridMode(instigator) AND IsServerOrStandalone(instigator))**. On that path it caches InstigatorCharacter/OwnerCharacter; an OwnerCharacter.HasAuthority subbranch queries active effects with **EffectDefinition=ShadowBase and EffectSource=InstigatorCharacter**, requiring **exactly one returned handle**. It saves that handle, installs a time-change task with the same query and reads its total duration/start time. A different branch registers OwnerCharacter with **instigator player's active SkiaLogicObj.AddShadowAtkTarget**. Zero/multiple query matches reach a diagnostic rather than an arbitrary first mark.

The cue CDO **#/10** declares unique instance per instigator **and** per SourceObject, bDestroyAtEnd=false and bCheckGamePlayStackChanged=true. ShadowBase itself remains **AggregateByTarget, cap 3, NeverRefresh/NeverReset, StackExpirationPolicy=RefreshDuration and bSuppressStackingCues=true**. The source-filtered query and unique-cue fields do not replace that target-aggregated GE policy or prove native event frequency/instance ownership.

**OnGameplayEffectTimeChange#/5 → graph#/1:8552** stores NewDuration and NewStartTime, then proceeds only when **NearlyEqual(NewStartTime, EffectContainerWorldTime, tolerance 0.0001)**. Valid cached actors and saved-handle stack count >0 lead first to **ShadowAtkDis** on OwnerCharacter attributed to InstigatorCharacter. After instigator's player cast, count **1 selects ShadowAtk_Damage**, **2 selects ShadowAtk1_Damage**, and other positive counts select **ShadowAtk2_Damage** (the branch falls through to that path even beyond 3). The configured cap 3 is not an explicit clamp inside this selector. Each packet obtains **GA_Skia_Melee OutSkillLevel + OutAwakenSkillLevel** for BufferData.nLevel, rather than using that lookup's separate return value; count 3's branch also requests ShadowAtk_Hitback. All routes then conditionally request **Buff_Player_Skia_TimeDilation** when the instigator has UltraSkill3_. The three named packet requests are alternatives, not a three-packet train per callback.

After those requests, if instigator has **ShadowControl2** and actor distance is **strictly <1000**, it retains the mark and, additionally requiring own **Passivity1** and target **Buff_Reaction_3_new**, requests reapplication of that reaction class. Otherwise, OwnerCharacter.HasAuthority requests removal of the saved handle, and the following call requests **all ShadowBase removal with SourceActor=null**, independently of that authority check. This broad class removal must not silently become only one source's mark removal. No new timer is scheduled by this time-change body. Native start-time event meaning, recurrence, stack expiration and damage/reaction/cleanup settlement remain essential; the comparison is not an independently measured hit interval. For supplied times 10 and 10.00005 the tolerance check passes, while 10 and 10.001 fail, without establishing their clock.

**OnStackChanged** only updates particle level/visibility in the reached graph. **GetShadowAtkInterval#/2** attempts a curve-row lookup **Skia_Pas2_Shadow_Atk_Interval with CurveTable=null** and has a 10 fallback on result byte 1; it is not called by this cue's inspected functions. Do not manufacture a valid table binding or adopt 10 as ordinary attack cadence. OnRemove unbinds movement visibility, ends a valid time-change task and clears query/handle/actors. Native SkiaLogicObj scheduling and external calls remain open. Complete functions and scoped properties: , `skia-shadow-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

### Inherited sneak exit cue routing recovered, October 18

Two warning-free data-only cue subclasses inherit **GC_Skia_SneekBase**: **Buff_Skia_ShadowControl#/2 → GC_Skia_ShadowControl#/1** sets **AbilityToActivateOnEnd=GA_Skia_ShadowEvadeEnd** and its own control GE class; **ShadowControl1#/2 → GC_Skia_EvadeSneek#/1** instead sets **GA_Skia_ShadowEvadeEnd1** and ShadowControl1. The base has no configured tag; these child tag/class/parent joins supply its actual consumers. Base ShouldPress=true is explicit, while Specified Input ID is omitted and not filled with an assumed input number.

The base's **OnWhileActiveEvent#/11 → graph#/6:359** calls the Halo parent, skips its own body for **IsHybridMode_Server(MyTarget)**, otherwise caches MyTarget player and requests **GetSneakEffectComponent.StartSneak**. If native IsSneaking is true it enables its Sphere, sets HasActivateAbility=false and installs ability-input/state-change tasks. The input factory metadata identifies **bRequirePlayerPossessed=true and bTriggerOnce=true**, with the configured SpecifiedInputID; these inputs do not replace the callback's separate press-value test. If IsSneaking is false, it requests end handling immediately except for server/standalone's configured Delay 0.2. Matching **bPress==ShouldPress** requests **BreakStateIfInState(StateType 38, RequesterStateType 0, argument0)**; entered-state callback is empty, while broken-state callback acts only for StateType 38. That path calls ConditionalActivateEndAbility and then removes the class loaded from this instance's configured GameplayEffectClass, regardless of attempted activation success.

**ConditionalActivateEndAbility#/5** requires a valid **local-client-role** cached player and valid primary instance of AbilityToActivateOnEnd. State 10 additionally sets CustomStartSectionName=ShadowEvadeEndOnWall after a SneakEndBase cast, ends a valid state monitor and requests BreakStateIfInState(10, requester12,0). State 5 then prevents the normal attempted activation; otherwise **TryActivateAbilityByClass** supplies HasActivateAbility. The helper also requests scene-location correction, whose geometry is outside the combat scenario. OnRemove calls the Halo parent, skips its own hybrid-server body, ends valid tasks, requests EndSneak only while state 38 is active, and may attempt the end ability again if it has not activated and no Ability.QTE is active. If still unsuccessful with **LimitUseSkill>0**, it can play the resolved end montage directly; montage playback is not successful ability admission or a guaranteed end hit. It then clears caches/location state. Native state/input delivery, callback re-entry, actual admission/cost and already queued packets remain open; neither the 0.2 wait nor cosmetic collision/vehicle handling is attack recovery.

### Shadow slow cue request and removal reset recovered, October 18

Warning-free **Buff_Player_Skia_TimeDilation#/2 → GC_Skia__TenacityTime** resolves a separate target-clock request. OnWhile **#/3 → graph#/1:139** requires MyTarget cast to AbilityCharacter, then requests **SetCustomTimeDilation(0.1,3,false,true)**; OnRemove **#/2 → 10** requests **1,3,false,true** on its MyTarget cast. Type 3 is **TimeDilationBuffEffect**, followed by bReplicate=false/bUsedParticle=true under the native parameter declarations. The carrier has HasDuration multiplier 1 against **Skia_ShadowAtkTenacityTimeDuaration=0.75**. Neither function applies a Shadow attack, pauses all GE timers or explicitly tests authority. This binds a class-specific dilation/reset request; it does not establish .75 real seconds of slow, the retained domain's clock, stacking with other dilation, or pending Shadow timing. Sources: `remaining-control-cue-carriers.txt`, `roster-cue-halo-extraction.json`, `cue-supplemental-joins.json` and the named **DT_SkiaEffectFigure** row.

## Unknown execution policies

- **Shadow Hound Gnaw & Shadow clone dispatch:** The cue now locates ordinary Shadow time-change requests, handle/stack selection and native target registration. Resolve native start-time events, SkiaLogicObj scheduling/target cleanup, effective clocks and SkillShadowAtk dispatch; simultaneous versus delayed deliveries remain unproved.
- **Fang Thrust mark stacking and consumption:** Resolve native rules governing maximum concurrent Fang Thrust locks, whether mark refresh extends or overwrites duration, the modeled 10s versus located 6s base-duration conflict above, and whether A6's 20% Extra DMG Ratio triggers on spatial collision or attack hit.
- **A5 mass / movement speed and A6 RES shred stacking:** Model the stacking dynamics of the 2% per mark Lakshana RES shred up to 6%, and whether target mass changes affect juggle physics or interruption windows in the combat engine.
- **Shared native boundary:** Saved asset/Blueprint data supplies selected requests and markers. The existing protected-body and real-time investigation does not recover this owners dispatch/cancel clock; no applicable command/hit/switch trace has been established here. See the shared timing owner and the specific unresolved policies above.


Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The [Shadow Control grant](#configured-shadow-control-ability-grant) has a located class/input/level/removal declaration. Resolve native grant admission, ShadowEvade prerequisites and input arbitration, and active/queued evade cleanup when its carrier ends; do not infer Hound/clone packet behavior from that grant.
- Resolve Normal chain reset/timing, riposte admission, shadow-clone/Hound owner and packet rules, and Fang Thrust mark application/consumption/expiry.
- Ordinary-hit energy caller eligibility/recipient rules remain a tested hypothesis.

## Prior findings

- [`reviews/q10-game-action-binding/skia-action-binding-model.json`](../../../../reviews/q10-game-action-binding/skia-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/skia_actions.rs), [`skia-normal-chain.json`](../../../../reviews/q10-game-action-binding/skia-normal-chain.json), [`skia-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/skia-skill-burst-packets.json).

## Research handoff

October 17 continuation: Selected Skill/Burst source and model slices were updated; full-kit research remains incomplete. Montage notify coordinates for Skill (`Skia_Skill_001`) and Burst (`Skia_UltraSkill`) supply asset-coordinate inputs for bounded timing/cancel profiles (`skia.skill_animation_profile_v1`, `skia.burst_animation_profile_v1`). Note that Skia's single-target lock model (`skia.fang_thrust` isolation under A3) remains preserved as single-target delivery per existing regression tests. Remaining limits: Shadow Hound Gnaw, shadow clones, riposte, and spatial targeting geometry. Resume the material unknown execution policies above, including linked subsystem questions; the brief limits in this handoff are not exhaustive. Coordinates and passing model tests do not establish native playback, cancellation or full-kit behavior. Reuse prior source locations and failed methods, save the next concrete question here, and continue other owners if a branch is exhausted. Historical receipts and unrelated edits remain preserved.


## Research history

- **Current question:** Natural sneak/Hound/clone dispatch, shadow consumption and final custom-damage settlement.
- **Useful source locations:** `Buff_Skia_ShadowEvadeCheck`, `Con_Skia_EvadeAtk`, `Execution_Skia_ShadowEvade`, `Execution_Skia_Swallow`, `GA_Skia_ShadowEvadeEnd` and SkiaLogicObj.
- **Methods already tried:** Shadow-control grant, short/control duration selectors and A4/passive modifiers are located. The new fixed-damage, GetHP/base-ATK swallow expression, periodic request and cooldown/mode predicate narrow the missing paths; the end override still requests a null-class removal.
- **Next concrete step:** Ordinary Shadow packet selection, source-filtered handle/time-change gate, removal/reaction requests and inherited sneak-exit consumers are now located. Resolve native SkiaLogicObj event/clock/target cleanup, shadow stack expiration/re-entry, state/input/ability admission and final damage settlement. Do not adopt the unbound interval helper's null-table fallback as cadence; continue remaining shared/control cue consumers.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
