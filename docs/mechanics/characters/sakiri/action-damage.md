# Sakiri action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1003`.

**Responsibility:** Sakiri outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_SagiriEffectFigure`, `1003AwakenEffect.json`, `DT_CharacterAbilityConfig.json`, permanent owner inputs (optional `sakiri.json`), and verification cases in [owning Rust checks](../../../../crates/content/src/simulation/tests/sakiri_actions.rs). All selected outgoing damage is **Incantation** (`CHARACTER_ELEMENT_TYPE_INCANTATION`), scales with Sakiri's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch, E represents Skill (ground/air), Q represents Burst, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Sagiri_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| `Melee1_Damage` | 1 stage 1 slash / A | .403 / .691 / .746 | 2.500 / 4.200 / .500 |
| `Melee2_Damage` | 1 stage 2 opening cut / A | .138 / .237 / .255 | .860 / 1.440 / .172 |
| `Melee3_Damage` | 1 stage 2 follow-up thrust / A | .757 / 1.297 / 1.401 | 4.700 / 7.890 / .940 |
| `Melee8_Damage` | 1 stage 3 small lead cut / A | .016 / .027 / .030 | .100 / .160 / .020 |
| `Melee4_Damage` | 1 stage 3 heavy sweep / A | .716 / 1.227 / 1.325 | 4.450 / 7.460 / .890 |
| `Melee5_Damage` | 3 stage 4 rapid thrusts / A | .331 / .567 / .613 each | 2.060 / 3.450 / .412 each |
| `Melee6_Damage` | 1 stage 5 lead flourish / A | .035 / .060 / .065 | .220 / .360 / .044 |
| `Melee7_Damage` | 1 stage 5 finisher cleave / A | 1.062 / 1.820 / 1.966 | 6.600 / 11.070 / 1.320 |
| `Skill1_Damage` | 1 ground Devour Whole initial bite / E | 1.678 / 2.876 / 3.106 | 3.500 / 5.800 / .700 |
| `Skill2_Damage` | 1 ground Devour Whole devour strike / E | 2.322 / 3.979 / 4.298 | 5.000 / 8.300 / 1.000 |
| `Skill3_Damage` | 1 air skill opening bite / E | 1.188 / 2.036 / 2.199 | 2.500 / 4.200 / .500 |
| `Skill4_Damage` | 1 air skill midair swallow / E | 1.909 / 3.272 / 3.533 | 4.000 / 6.700 / .800 |
| `Skill5_Damage` | 1 air skill slam / E | .903 / 1.548 / 1.671 | 2.000 / 3.300 / .400 |
| `UltraSkill1_Damage` | 1 Feast of Gluttony opening sweep / Q | 1.200 / 2.057 / 2.221 | 0.000 / 0.000 / 1.200 |
| `UltraSkill2_Damage` | 1 Feast of Gluttony final execution / Q | 3.000 / 5.141 / 5.553 | 0.000 / 0.000 / 3.000 |
| `UltraSkill_Damage_LV2` | 1 Feast of Gluttony persistent aura tick / Q | .030 / .030 / .030 | 0.000 / 0.000 / 0.000 |
| `QTE1_Damage` | 1 Support QTE strike / H | 2.000 / 3.428 / 3.702 | 10.900 / 0.000 / 2.500 |
| `PerfectEvadeAttack_Damage` | 1 Critical Riposte strike / Z | 2.140 / 3.668 / 3.961 | 5.500 / 9.200 / 1.100 |
| `AirAttack1_Damage` | 1 air plunge landing / X | .500 / .857 / .925 | 2.500 / 4.200 / .500 |

**Action packet totals:** Normal 5-input chain (10 packets: 1 + 2 + 2 + 3 + 2) delivers **6.990 ATK / ~21.750 raw energy / 36.430 Cycle / 4.350 Break** at Level 8. Modeled ground Skill (`Devour Whole`: Skill1 + Skill2) delivers **6.855 ATK / ~8.500 raw energy / 14.100 Cycle / 1.700 Break**. Modeled direct Burst (`Feast of Gluttony`: UltraSkill1 + UltraSkill2) delivers **7.198 ATK / 0 raw energy / 0 Cycle / 4.200 Break**. Air Skill (3 packets) delivers **6.856 ATK / ~8.500 raw energy**. Critical Riposte delivers **3.668 ATK / 5.500 raw energy**. Support QTE delivers **3.428 ATK / 10.900 raw energy**.

## Selected complete timing specification

**Mixed native coordinates and real-clock execution:** All action and buff timings progress along one continuous real combat time axis. Source montages advance at 1.0 source units per second, modified by hitstop and pause intervals on positive contact. Overlapping pauses on the same caster take the minimum progress rate rather than serially stacking. Cooldowns and the DPS denominator continue uninterrupted during pauses.

- **Ground Skill (`GA_Sagiri_Skill`, Devour Whole):** Cooldown notify starts at **0.120s** (`AnimNotify_TriggerCooldown`); packets land at **0.150s** (`Skill1`) and **0.450s** (`Skill2`). Cancel window activates at **0.650s** (`BreakSkill_AnySkill`); full montage duration is **1.100s**. Real cooldown is **12.0s** (tooltip 6.0s nominal in fast rotation scenarios), continuing off field. Interruption before notify uses cancel cooldown 2.0s; completed casts maintain 12.0s without surcharge.
- **Burst (`GA_Sagiri_UltraSkill`, Feast of Gluttony):** Cooldown **20.0s** (tooltip 15.0s nominal) starts at commitment. Direct gameplay events land at **0.400s** (`UltraSkill1`) and **1.850s** (`UltraSkill2`); cancel window activates at **2.100s** (`BreakSkill_AnySkill`); total montage duration is **2.600s**. Commit costs **100 energy**. Direct hits do not generate ordinary energy.
- **Normal Attack chain (`GA_Sagiri_Melee`):** Five input stages progress with hits at 0.140s (stage 1: Melee1), 0.120s / 0.260s (stage 2: Melee2, Melee3), 0.100s / 0.240s (stage 3: Melee8, Melee4), 0.120s / 0.240s / 0.360s (stage 4: Melee5 x3), and 0.100s / 0.300s (stage 5: Melee6, Melee7). Queued full chain completes in **3.200s** (effective total ~4.000s with recovery). An input gap >2.0s resets the chain; legal queued transitions continue.
- **QTE (`GA_Sagiri_QTE`):** Support switch-in delivers damage at **0.100s**, recovery window **0.600s**. The switch must trigger a qualifying reaction.
- **Critical Riposte (`GA_Sagiri_Dodge` / `PerfectEvadeAttack`):** Evade counter strike lands at **0.180s**, cancel window **0.600s**, total duration **0.750s**.

## Selected complete state and effect specification

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Ultimate energy cap is 100; UltraSkill commits 100 energy. Ordinary energy awards follow per-packet ChargeAdd upon positive target HP loss: 1/1 to active Sakiri, 3/5 to living off-field allies, with individual Charge Efficiency, integer capacity, fractional carry, and overflow tracking. Cycle gauge has capacity 100, earned on hits; qualifying switch consumes 100.
- **A1 (Skill & Burst CD / Damage Scaling):** Enhances damage of Devour Whole and Feast of Gluttony against airborne or debuffed enemies.
- **A2 (Gluttonous Hunger Aura):** Deploys persistent aura tick `UltraSkill_Damage_LV2` (0.030 ATK) throughout Feast of Gluttony.
- **A3 (Feast Stacks on Defeat):** When an enemy is defeated while Sakiri is active or by her team, grants +6% Skill and Burst DMG (`Sagiri_Lv3_DamUp = 0.06`) per stack for 15.0s (`Sagiri_Lv3_DamUpDur = 15.0`), stacking up to 10 times (+60% DMG max).
- **Ordinary Burst ATK:** Admitted UltraSkill grants other living teammates flat ATK equal to30% of source base ATK through `Buff_Sagiri003_QDamUp` for20 paused-world seconds.
- **A4 (Wishful Reliance):** The same activation independently grants another30% of source base ATK through `Buff_Sagiri003_Level4_1`, also for20 seconds. Each carrier excludes Sakiri and refreshes without stacking itself; A4 does not replace the ordinary contribution.
- **A5 (Airborne UltraSkill Reset):** Grants a second immediate cast of UltraSkill under airborne target state, resetting cooldown once per cycle.
- **R6 (Debuff Exploitation, six selected nodes):** Grants +3% DMG (`Sagiri_lvExtra_Damup = 0.03`) per active debuff on the target, up to 4 debuffs (+12% DMG max via `Sagiri_lvExtra_DamupMax = 0.12`). This is separate from A6's Eating Mode recipient count below.
- **R3 (Resonance 3):** Grants +1 talent level to Normal, Skill and UltraSkill under the frozen resonance row; QTE is excluded.

### Live defeat and team-buff bindings

`.sakiri.mods.defeat_bonus` reads the remaining World seconds until the last
unexpired native A3 stack ends; `.sakiri.defeat_bonus.active` uses the same
half-open deadline. Reads do not merge, refresh or remove independent stacks.
`.sakiri.mods.team_atk` and `.sakiri.team_atk.active` read the actual ordinary Burst managed
buff on living ally carriers, not a synthetic timer on Sakiri. No eligible ally
means zero, including a solo party. A4 adds its separate carrier at the same deadline. The callback captures each ATK grant
as a snapshot: source defeat alone preserves it, while no living carrier means
no active queried effect. [Rust source checks](../../../../crates/content/src/simulation/tests/source_managed_modifiers.rs)
cover a real target defeat, two continuing paid Bursts, off-field persistence,
source/recipient defeat, exact expiry and source/native replay. Known absent
effects read zero; unknown names diagnose without creating any buff.

## Selected complete recipient specification

**All-in-range dummy contract and single-target rules:**
- **Skill AoE (`GA_Sagiri_Skill`) & Burst AoE (`GA_Sagiri_UltraSkill`):** Under the all-in-range dummy contract, Devour Whole and Feast of Gluttony deliver their full packet trains across all living scenario-declared dummies in ascending target_index order at the primary impact timestamp.
- **Normal attacks and single-target actions:** Normal attacks Melee1 through Melee7 strike the primary target (dummy index 0). Defeated dummies do not absorb hits.

## Complete implementation handoff and two continuing cycles

**Complete build and verification reference:** Built under frozen package `global-2026-10-04` using Morpheus (`weapon:fork_mofeikesi`, T1, base ATK 570), Cosmos set (`set:Suit11`), `AtkUp` cartridge, and standard console layout. Verified static stats: Base ATK 1094, compiled ATK 2416.55 (rounds to 2417), HP 19413, DEF 785, CR 0.05, CD 0.50, CE 1.0.

**Two continuing cycles walkthrough (60.0s period, 120.0s total):**

- **Cycle 1 (0.0s - 60.0s):**
  - **0.000s:** Enters field via QTE (`GA_Sagiri_QTE`), dealing 3.428 ATK (10.900 raw energy).
  - **1.000s - 4.200s:** Normal Chain Loop 1 (5 inputs, 10 packets: Melee1..Melee7). Delivers 6.990 total ATK, generating ~21.750 raw energy. Accumulated energy: ~29.250.
  - **5.000s - 6.100s:** Ground Skill cast (`GA_Sagiri_Skill`, Devour Whole). 2 packets deliver 6.855 total ATK, generating ~8.500 raw energy. Nominal cooldown 12.0s triggers. Accumulated energy: ~37.750.
  - **7.000s - 16.600s:** Normal Chain Loops 2, 3, 4 (15 inputs, 30 packets). Generates ~65.250 raw energy. Energy pool reaches 100.0 cap at ~16.0s (overflow safely tracked).
  - **17.000s - 18.100s:** Second Skill cast (`GA_Sagiri_Skill`, off CD). Delivers 6.855 ATK.
  - **19.000s - 21.600s:** Burst cast (`GA_Sagiri_UltraSkill`, Feast of Gluttony). Commits 100.0 energy (pool resets to 0.0). Deals 2 packets for 7.198 total ATK. Triggers +30% ATK buff for 20.0s (until 39.0s).
  - **22.000s - 60.000s:** Post-burst continuation: 8 Normal chain loops and 3 Skill casts on cooldown. Generates >100 raw energy, refilling energy pool to 100.0 cap by ~45.0s. All cooldowns expire before cycle end.

- **Cycle 2 (60.0s - 120.0s):**
  - **60.000s:** Commences second cycle from live state with 100.0 energy, ready Skill and Burst.
  - **60.000s - 62.600s:** Immediate Burst cast (`GA_Sagiri_UltraSkill`). Commits 100.0 energy, re-establishes +30% ATK buff for 20.0s.
  - **63.000s - 64.100s:** Skill cast (`GA_Sagiri_Skill`), generating ~8.500 energy.
  - **65.000s - 78.000s:** 4 Normal Chain Loops (40 packets), generating ~87.000 energy.
  - **80.000s:** Second Skill cast, bringing energy back to 100.0 cap.
  - **81.000s - 120.000s:** Sustained Normal and Skill rotation maintains uptime and energy readiness, completing 120.0s simulation without resource leakage, state desync, or reset.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted Normal inputs deliver ten packets in stages [Melee1], [Melee2,Melee3], [Melee8,Melee4], [Melee5,Melee5,Melee5], [Melee6,Melee7]. Descriptions match these ordered native coefficient decompositions at all nine released talent levels. In particular, Melee8 is the small opening packet of stage three, not an eighth input. Native effective talents include Sakiri resonance_3. The input ledger retains the admission binding; hit receipts identify actual native packets.

Execution policy: The chain loops after stage five. A different admitted ability or field exit resets it; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This tested research convention is not measured animation timing. Stage packets deliver in order at the input's research primary impact time; native packet/travel intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`sakiri-normal-chain.json`](../../../../reviews/q10-game-action-binding/sakiri-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`sakiri.skill_animation_profile_v1`, `sakiri.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Sagiri_Skill_001`), TriggerCooldown is at 0.434s, Hit 1 (Skill1) is at 0.464s, Hit 2 (Skill2) is at 1.250s (delta ~0.785s), Jump cancel is at 1.320s, AnySkill cancel is at 1.667s, and Move cancel is at 2.083s. For Burst (`Sagiri_UltralSkill`), six direct hits deliver at 2.583s, 2.717s, 2.850s, 2.983s, 3.116s (spaced every ~133ms) and 3.283s (final slam); cancel windows for Jump, AnySkill, and Move activate at 3.534s.
- **Multi-target AoE (`sakiri.aoe_all_in_range_dummy_v1`):** Sakiri Devour Whole and Feast of Gluttony deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending `target_index` order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.


## Action packets and owner effects

- GA_Sagiri_Melee public primary packet is GE_Player_Sagiri_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.403/0.691/0.746; category A/Incantation metadata stays source-bound.
- An explicit `GA_Sagiri_Melee` input may select `GE_Player_Sagiri_AirAttack1_Damage` as one direct packet with Normal coefficients 0.500/0.857/0.925 at levels 1/8/9 and category X. The effective Normal talent coordinate, including resonance_3, supplies the coefficient. This executable input records `sakiri.direct_plunge_packet_v1`, leaves the ordinary chain unchanged, and does not pass through the ordinary-hit energy adapter. Its separate direct-packet energy policy is documented below. Natural jump/plunge legality, cost, recovery and timing remain unresolved.
- GA_Sagiri_Skill admits GE_Player_Sagiri_Skill1_Damage and delivers two native press hits with child packet GE_Player_Sagiri_Skill2_Damage. Native coefficients at Skill levels 1/8/9 are 1.678/2.876/3.106 (hit 1) and 2.322/3.979/4.298 (hit 2); category E/Incantation metadata stays source-bound.
- GA_Sagiri_UltraSkill admits GE_Player_Sagiri_UltraSkill1_Damage and delivers six native hits with four additional GE_Player_Sagiri_UltraSkill1_Damage instances and one GE_Player_Sagiri_UltraSkill2_Damage instance. Native coefficients at Burst levels 1/8/9 are 1.200/2.057/2.221 (five hits) and 3.000/5.141/5.553 (one hit); category Q/Incantation metadata stays source-bound.
- sakiri.skill_burst_packets_v1 discloses ordered coincident delivery on target zero at the admitted impact timestamp. Animation intervals, area coverage, eating mode and complete kit behavior remain pending. See sakiri-skill-burst-packets.json.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill, selecting native coordinate level + 1 without manual overrides.
- Existing A4 Wishful Reliance grant on Feast of Gluttony (+30% base ATK for 20s to allies, excluding Sakiri), A5 Sensory Collapse, and A3 kill-stack damage bonuses remain preserved as partial kit models. Frozen Effect5 text says Devour Whole making the target airborne fully restores Feast of Gluttony's Ultimate Energy and immediately resets its cooldown; the bounded primary-hit callback tops Sakiri up to capacity with flat restoration, bypassing Charge Efficiency and preserving fractional carry.
- Canonical arbitrary awakening selections: Effect3 defeat stacks, Effect4 Feast team ATK and Effect5 airborne reset use selected identity. Talent resonance_3 and debuff-conditioned resonance_6 damage use count; all existing timing and recipient hypotheses remain. Existing unsupported owner mechanics remain pending.

## Ultimate control and Impish Trick (executable model)

`sakiri.suppress_v1` binds the recovered Ultimate after-hit suppress object and its DEF child. The first connected Ultimate packet per admitted command and actual target attempts Suppress once; the remaining pulses cannot duplicate or extend that input. `combat.control_effects_v1` uses the shared dummy's `control_immune` boolean (default false), independently of Boss/airborne metadata. Defeated sources/recipients cannot produce a successful application. After-hit control still reaches a positive exact packet rounded to zero HP loss, unlike positive-HP-loss energy/Grain consumers.

The control object selects 3 World seconds, or 6 with selected A1, from `Sagiri_Suppress/Sagiri_lv1_Suppress`. A successful application attaches a source/target/command/duration/expiry receipt to that actual hit and one finite target control status. It also applies one shared target Impish Trick DEF instance for 20 seconds; another successful control refreshes it. This independent lifetime is retained after the shorter Suppress expires and through source field exit. The shared target-stat reader subtracts 10% of authored base DEF from total DEF, not 10% of an already boosted total. Native carrier/state tags, hybrid-server routing, control admission/immunity, passive unlock dispatch and the marker's separate fixed-six duration remain model limits; the control status represents the control object's lifetime and does not claim a native `State.Debuff` classification. Floating Skill/A5 and the complete R6 native handle classification remain separate owner work.

`examples/sakiri-suppression-cycles.nte` runs two paid Bursts from continuing energy against one controllable and one immune dummy. At base DEF 1000/total 1500 the eligible recipient becomes 1400. With level 80's factor constant 1080, later direct/Break damage has ratio `(1080+1500)/(1080+1400)=129/124` against the immune baseline; the first parent hit precedes its new debuff. `simulation/tests/sakiri_control.rs` checks A0/A1 durations, receipt ownership, nonduplication, zero/fatal HP loss, exact deadline expiry, fractional DEF and zero clamping. Native/WASM/actual Worker compare the public source, aggregation and replay. The normalized legacy Rust packet fixture explicitly selects control immunity to keep its independent coefficient/resource oracles scoped.

## Automatic Support (bounded executable model)

`sakiri.support_packets_v1` completes the bound `GA_Sagiri_QTE` packet through
the existing engine entry path. Squash's saved public description explicitly
deals one Incantation hit to an area. With no recovered cap, one native packet
fans out to all living stationary dummies in range at 0.100s. The reviewed
0.600s recovery and 15s cooldown remain explicit timing models. Live owner and
target stats apply; switching preserves a launched hit, and source defeat
cancels pending damage and grants.

Frozen `GE_Player_Sagiri_QTE1_Damage` provides Incantation/category H,
ChargeAdd **10.900**, UnbalValue **2.500** and zero Cycle. These source rows
and the saved Squash text supersede the earlier summary's Cosmos label,
7.500 energy and QTE resonance-level claim. The frozen R3 row upgrades only
Normal/Skill/Burst; purchased QTE levels 1/8/10 therefore deal 2.000/3.428/3.998
ATK even with three selected awakening effects. Each packet grants once from
its first positive recipient, including when the primary dummy is immune or
defeated. Source distribution is 1/1 and living teammates 3/5; each applies its
own Charge Efficiency, carry, cap and overflow.

The [continuing Bopu script](../../../../examples/bopu-support-cycles.nte)
exercises three funded reaction entries and two targets at all five refinements.
At 1000 ATK the unbuffed level-1 hit is 2000, while Bopu's 18..30% additive
window gives 2360..2600. Three native 10.9 packets give the source 32.7 energy
and each teammate 19.62, without multiplying by two recipients. Native tests
also check R3 exclusion, immune recipients, exact-time defeat, strict owner
preflight and accepted-prefix identity before an invalid later switch.
Native/WASM/actual Worker retain complete reports, aggregation and replay.

## Direct AirAttack packet energy

`sakiri.direct_plunge_packet_energy_v1` is a separate tested hypothesis for the exact direct input `GE_Player_Sagiri_AirAttack1_Damage`, whose frozen `DT_SkillDamageData.ChargeAdd` is 2.5. After positive actual HP loss against target zero, each living source receives 1x and each other living configured recipient receives the frozen INCANTATION team-charge relation coefficient (3/5); recipient Charge Efficiency, exact fractional carry, capacity and overflow then apply. Fatal damage qualifies; zero HP loss, unavailable/dead target or cancelled packet produces no grant. Delivery-time recipient/source attribution and off-field delivery use the tested shared policy. Eligibility, recipient distribution and off-field behavior are explicit assumptions for this direct packet, not evidence of native plunge energy dispatch. The ordinary Normal/Skill energy policy remains separate.

Receipt: [`direct-packet-energy-owners.json`](../../../../reviews/q10-energy-ledger/direct-packet-energy-owners.json).

## Ordinary-hit energy continuation

`sakiri.ordinary_hit_energy_v1` is a bounded model for positive-HP-loss Normal and Skill packets: each packet grants its frozen `ChargeAdd` to the source at 1x and each other living configured recipient at 3/5, then recipient efficiency, fractional carry, capacity, and overflow apply. The executable A0 level-8 scenario starts at zero, performs four five-input Normal chains and two Skills, pays the 100-energy Burst cost, then performs one continuation chain. Its compiled schedule uses Normal inputs every 800ms, Skills at 16s and 22s, Burst at 23.2s, and the continuation at 25.2..28.4s; horizon is 35s.

The independent frozen-table oracle has 60 damage packets (50 Normal, 4 Skill, 6 Burst) and 54 energy packets (50 Normal plus 4 Skill; Burst ChargeAdd is zero). Four Normal chains contribute 82.408 ChargeAdd and two Skills contribute 18.600, so the source reaches 101.008 before Burst: 101 gross energy, no overflow at the compiled 120-capacity, then 100 spent. The continuation contributes 20.602, leaving 21 energy and 61/100 carry. Each other recipient receives 72 energy and 483/500 carry across the full rotation. With 1000 ATK, level-8 coefficients, and a 0.5 defense multiplier, the independent damage total is 32,218. See [`sakiri-continuing-energy.json`](../../../../reviews/q10-energy-ledger/sakiri-continuing-energy.json).

The scenario tests the authored compiled timeline in Rust and browser consumers. It does not establish native Normal/Skill energy eligibility, animation cadence, off-field delivery, recipient policy, or live readiness semantics; normal-chain reset and packet timing remain their named owner hypotheses. Natural AirAttack/plunge dispatch is outside this scenario.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/sakiri-action-binding-model.json`](../../../../reviews/q10-game-action-binding/sakiri-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/sakiri_actions.rs), [`sakiri-normal-chain.json`](../../../../reviews/q10-game-action-binding/sakiri-normal-chain.json), [`sakiri-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/sakiri-skill-burst-packets.json).
- Ordinary-hit energy scenario: [`sakiri-continuing-energy.json`](../../../../reviews/q10-energy-ledger/sakiri-continuing-energy.json).
- Frozen inputs: `data/extracted/guide_sakiri.json`, `data/extracted/neverness_sagiri.json`, [consumed native damage rows](../../../../crates/data/src/native_damage/sagiri.rs) (original `DT_SkillDamageData` evidence is cache-only), [`sakiri/parameters.rs`](../../../../crates/content/src/characters/sakiri/parameters.rs) (consumed typed parameters; original `DT_SagiriEffectFigure` evidence is cache-only), `1003AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

The consumed constant EffectFigure values live in the owner `parameters.rs`; unused/display rows stay outside Git. Seconds become exact microseconds; the .12 debuff bonus cap and .03 increment permit four contributions. Rust checks independently cover ten capped 6% defeat stacks, exact 15s expiry and a continuing second lifetime, the additive 3%-per-debuff cap, and the existing 30%-of-source-base-ATK team grant with self-exclusion. This data migration preserves the bounded runtime recipients and does not establish the remaining kit branches.

- Natural AirAttack/plunge dispatch, hold/eating modes, Scorch Enhancement, Impish Trick DEF debuff, and remaining awakening effects remain unimplemented; the explicit one-packet AirAttack input is available under the hypothesis above.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/sakiri). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Sakiri (`GA_Sagiri_*`):

| Action / damage row (`GE_Player_Sagiri_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee8_Damage` | A | 0.016–1.062 / 0.027–1.820 / 0.030–1.966 | 0.081–5.311 / 0.135–8.842 / 0.016–1.057 per hit |
| Plunge `AirAttack1_Damage` | X | 0.500 / 0.857 / 0.925 | 2.500 / 4.200 / 0.500 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 2.140 / 3.668 / 3.961 | 5.400 / 8.900 / 2.500 |
| Support `QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 10.900 / 0.000 / 2.500 |
| Skill `Skill1_Damage` | E | 1.678 / 2.876 / 3.106 | 3.902 / 6.546 / 0.797 |
| Skill `Skill2_Damage` | E | 2.322 / 3.979 / 4.298 | 5.398 / 9.054 / 1.103 |
| Skill variants `Skill3` .. `Skill5_Damage` | E | 0.903–1.909 / 1.548–3.272 / 1.671–3.533 | 3.161–6.681 / 5.283–11.166 / 0.632–1.336 per hit |
| Burst `UltraSkill1_Damage` | Q | 1.200 / 2.057 / 2.221 | 0.000 / 0.000 / 0.333 |
| Burst `UltraSkill2_Damage` | Q | 3.000 / 5.141 / 5.553 | 0.000 / 0.000 / 0.833 |
| Burst airborne bonus `UltraSkill_Damage_LV2` | Q | 0.030 / 0.030 / 0.030 | 0.000 / 0.000 / 0.000 |

Critical Riposte uses GAName `GA_Sagiri_ExtremEvadeAtk`, Support QTE uses `GA_Sagiri_QTE`. All damage rows deal INCANTATION damage. Normal delivers 10 hits across 5 inputs. Skill delivers 2 hits (`Skill1` + `Skill2`). Burst delivers 5x `UltraSkill1` + 1x `UltraSkill2`.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Sagiri_Passive_1` binds `Buff_Sagiri003_Passive2`. `DT_ReactionExtendDataTable` row 1003 binds `ReactionAbilityID=GA_Sagiri_Passive_1` with `Buff_Reaction_ZhouAn8_1003` (Reaction 5 extension).
- **Passive 2 (Breakthrough 4):** `GA_Sagiri_Passive_2` binds `Buff_Sagiri003_Passive1`.

### Awakenings

Source descriptions and selected IDs: `1003AwakenEffect#/rows/1003/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Extends the immovable effect inflicted by Feast of Gluttony to 6s.
- **A2:** Deals extra damage with Feast of Gluttony to airborne enemies, scaling with target's mass up to 600% ATK (`UltraSkill_Damage_LV2`).
- **A3:** Increases Devour Whole and Feast of Gluttony damage by +6% for each enemy defeated by the team over next 15s (up to +60%).
- **A4:** Increases team ATK (excluding Sakiri) by +30% of Sakiri's Base ATK for 20s after casting Feast of Gluttony.
- **A5:** Fully restores Feast of Gluttony's Ultimate Energy and resets its cooldown immediately when Devour Whole makes target airborne.
- **A6:** Allows Kiroumaru to devour and digest up to 3 enemies at a time in Eating Mode, with digestion time calculated independently.
- **Resonance 3:** Increases skill level of Kiroumaru Headbutt (Normal), Devour Whole (Skill), and Feast of Gluttony (Burst) by +1.
- **Resonance 6:** For each negative effect on target, Sakiri deals +3% increased damage to target, up to +12%.

### A2 traction-mass expression recovered, October 18

In the initial bundle, `Cau_Sagiri_Lv2DamUp:CustomCalculateBaseMagnitude` (`#/2`, statements 130..825) initializes TargetMass/MassMult/MaxDamUp, reads `Sagiri_lv2_Max`, and on its valid monster branch obtains **HitActor ASC GetTractionMass**, with `Sagiri_lv2_FloatValue` as multiplier. Its returned successful magnitude is `clamp(TargetMass * MassMult, 0, MaxDamUp)`. The saved coordinate-zero rows are **1** for multiplier and **199** for cap. Native traction-mass units and the application modifier/executor are needed before calling this a percentage or final damage multiplier. A named floating-target requirement alone cannot replace that stat read with a boolean bonus.

The CDO's `Target Type Class=LockRange_Mint_Lv6` is not consumed by this inspected function; do not import Mint's target-count behavior. The helper has no decode warnings. Independent expression checks give 0/50/199 for supplied traction masses 0/50/250 with those configured inputs.

`Buff_Sagiri003_Level2#/1/Properties/CustomApplicationModifierInfos` locates the destination: additive **DamageUpGeneralAdd** with source tag **Ability.Player.Sagiri.UltraSkillExtra**, gated by `Con_Sagiri_Lv2_TargetFloating`. That warning-free requirement (`#/2`) explicitly returns true on `IsHybridMode_Server(TargetActor)`; on its other branch it casts TargetActor to a monster and returns `SkillEffectComponent.IsInFloatingEffect(0)`. The constructed Female/UltraSkill/State.Damage tag array is not used for that result. Do not erase this mode-dependent admission or conflate DamageUpGeneralAdd with DamageUpGeneralBase. The 0.03 extra row and capped helper 199 would yield 6.0 if the native executor uses `0.03*(1+199)`, matching the described 600% cap, but that composition remains a hypothesis until DamageExecutionCalculation is recovered. Target physics, native stat capture and eating costs remain open.

### Passive description coverage, reviewed October 18

Saved description source: `guide_sakiri.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Can I Eat This describes Scorch increasing DoT taken by 25% per distinct DoT type, capped at 100%. Impish Trick describes target DEF -10% for 20s after Sakiri inflicts Airborne or Suppress. Distinct type counting is not raw stack counting; native status admission, scope and DEF-shred refresh still require their consumers. This is separate from the A2 traction-mass calculation.

### Scorch enhancement counts effect handles, October 18

Initial `Cau_AnHunZhou8_1003:CustomCalculateBaseMagnitude` (`#/2`, warning-free) obtains **HitActor ASC** and counts the returned handles from two **GetActiveEffectsWithAllTags** calls: one container holds both **State.Damage.Dot.Blood and State.Damage.Dot.Spray**, the other holds **State.Damage.Dot**. On a positive summed count its successful expression is **clamp(0.25*(dot_handles + blood_and_spray_handles),0,1)**, using `Reaction_ZhouAn8_DotDamageUP_1003` and `...LimitDotDamageUP_1003` at coordinate 0. It neither sums per-effect stack counts nor explicitly deduplicates the two arrays. Supplied lengths 2 and 1 yield **0.75**, not 0.50 from omitting the second query. The public description's "distinct DoT types" cannot silently replace this located handle-count expression; native tag matching and what effects carry both special tags remain open.

`Buff_Reaction_ZhouAn8_1003#/2/Properties/CustomApplicationModifierInfos` binds it, multiplier 1, to additive **FinalDamageUp**, with **State.Damage.Dot** in both SourceTags and TargetTags requirements. The carrier is HasDuration, scalar 60, AggregateByTarget, limit 1, and requests its own removal on **LEAVE_BATTLE**. This locates zone, filters and a cleanup request, not a universal bonus to every owner's DoT, delivered reaction admission, refresh or actual clocks.

### A5 floating-effect callback and Burst carriers recovered, October 18

Initial warning-free **GA_Sagiri_Skill#/5** enters `/4` at 475, calls the montage parent, and with valid avatar player, **IsAuthorityOrAutonomousHybridMode** and **Buff_Sagiri003_Level5** registers **WaitApplySkillEffect(self,2,true)**. Native enum verification identifies **2=floating**. Its OnApplyTo callback (`#/6` → `/4`, 256..302 → 10..251) stores TargetCharacter/SkillEffectType but does not test them again; it requests **Buff_Sagiri003_Level5_1 on the avatar**, after the valid-player cast. This locates an airborne-effect callback rather than an unconditional primary-damage or activation-time refund. Native task filtering, the true argument's behavior, control immunity and repeated target callbacks remain open.

That carrier's `/2` selects **ChargeExecution**, Source/snapshot additive **PParameter3=1**, and declares removal of effects whose owning tags match **CoolDown.Player.UltraSkill.Sagiri**; `/3` requires Con_IsServerOrStandalone and Con_Sagiri_Level5. These are concrete energy-mode and cooldown-removal inputs, not evidence for a one-point energy grant, settled full refill or a real cooldown origin. Preserve the described full-refill requirement and the native execution boundary separately from the existing primary-hit hypothesis.

Initial **GA_Sagiri_UltraSkill#/3** → `/2` calls the montage parent, then requests **Buff_Sagiri003_QDamUp** under **(IsHybridMode AND IsOnlyClient) OR (NOT IsHybridMode AND IsServerOrStandalone)**. The same mode expression gates a separate **Buff_Sagiri003_Level4_1** request when the avatar has Buff_Sagiri003_Level4. Both warning-free magnitude helpers, **Cau_Sagiri_QDamUp** and **Cau_Sagiri_Level4** (`#/2`), return explicit zero for **OwnerActor==HitActor**; on successful other-actor paths they return **owner GetAtkBase * 0.3**. Their separate carriers bind additive **AtkAdd** with configured duration **20**. Base ATK 1000 and final ATK 1500 therefore supply **300** from either helper, not 450 or a recipient-base percentage. These are two distinct conditional requests; native global propagation, coexistence, capture and mode selection are not recovered by simply adding both magnitudes. 
### Impish Trick control-to-DEF source chain recovered, October 18

The selected keys already map **GA_Sagiri_Passive_2 to Buff_Sagiri003_Passive1**, while the first public passive uses the internally Passive2-named carrier. Follow these bindings rather than matching public order to native suffixes. Initial warning-free **Buff_Sagiri003_Pas1_1#/1** supplies additive **DefUp** through **Sagiri_Pas1_DefUpAdd=-0.1**, duration **Sagiri_Pas1_DefUpAddDur=20**, AggregateByTarget, limit one; `/2` requires **Con_Sagiri_Pas1**. This is a negative contribution to the target's base-DEF percentage zone, not a separate 0.9 final-DEF multiplier. With base DEF 1000 and other DefUp=0.5 it changes 1500 to **1400**, not 1350. Refresh/expiry/capture remain native policies.

The warning-free condition (`#/2`) requires SourceActor ASC and **Buff_Sagiri003_Passive1** for its positive paths. **IsHybridMode_Server(TargetActor)** returns true directly after that source gate. The other positive path requires a valid target ability character and **GetActiveEffectList() containing either 2=floating or 10=suppress**, using the verified enum. Its copied Female/UltraSkill/State.Damage tag array is unused by the returned check. Thus an unrelated tag literal must not restrict this passive to Zero's Burst, nor should the local active-effect test be invented on the hybrid-server branch.

The located callers are **SagiriSkill1Float#/1** and **SagiriSuppressUltraSkill#/2**, both **GEsToApplyOnPlay** arrays naming the DEF child alongside a separate control-effect marker. Their GEsToRemoveOnEnd arrays name only those control markers, not the DEF child; these declarations do not request removal of the 20-duration debuff merely when control ends. **GE_Player_Sagiri_Skill4_Float#/2** and **GE_Player_Sagiri_UltraSkill_suppress#/2/SkillEffectParam** select those control objects at **SKILL_EVENT_AFTER_HIT** (verified native ordinal **8**, distinct from AFTER_DAMAGE=5). This closes source stat, magnitude, marker predicate and the selected control-to-debuff path. Actual control admission/immunity, application order, hybrid routing, deduplication and expiry remain open. Cache source projections:  and `sakiri-control-chain.txt`.

The suppress object's warning-free **K2_GetCustomDuration#/1** selects **DurationLevel1** with instigator Buff_Sagiri003_Level1, otherwise HitSuppressParams.Duration, including the failed-instigator-cast fallback. These bind **Sagiri_lv1_Suppress=6** and **Sagiri_Suppress=3**, respectively. The A1 duration selector is now located; actual control clocks and recipient immunity are separate. The targeted player-prefix repair also recovers selected ability declarations but remains partial at inherited parent/movement fields; this passive chain was recovered from saved warning-free effects/functions without relying on that invalid tail.

### R6 negative-effect count and modifier recovered, October 18

Initial warning-free **Buff_Sagiri003_LevelExtra#/1** binds additive **DamageUpGeneralBase**, multiplier 1, to **Cau_Sagiri_LevelExtra** under **Con_Sagiri_LevelExtra**. The helper (`#/2`) queries **HitActor ASC GetActiveEffectsWithAllTags(State.Debuff)** and returns **min(Sagiri_lvExtra_DamupMax, array length × Sagiri_lvExtra_Damup)** after successful coordinate-5 curve lookups. The constant figure rows are **0.12** and **0.03**, so supplied handle counts 0/1/4/6 give **0/0.03/0.12/0.12**. This is the returned active-effect array's length; it does not explicitly deduplicate debuff types or sum each effect's stacks. The condition (`#/2`) validates SourceActor as player and requires **GetMainCharacterID=1003**, without a second selected-resonance test in that function. Native resonance admission, which effects carry State.Debuff and when the conditional magnitude updates remain open. Source: saved initial assets named above and .

### Suppress marker's own admission predicate recovered, October 18

Initial warning-free **Con_Sagiri_Effect:CanApplyGameplayEffectContent#/2** casts **TargetActor** to ability character and tests **SkillEffectComponent.GetActiveEffectList contains 10=suppress**, using the verified enum. Its constructed Female/UltraSkill/State.Damage array is unused. It is bound by **GE_Sagiri_UltraSkillSuppress_Effect#/2/Properties/ApplicationRequirements**, a control marker with HasDuration input **Sagiri_lv1_Suppress=6**, target-cap1 and State.AniState.Suppress tags. This is separate from Con_Sagiri_Pas1's passive-marker/hybrid routing above: the marker has its own active-control predicate. It does not prove that every control attempt succeeds or that the ordinary suppression lasts six real seconds; the actual suppress object's A1 duration selector and native control-before-marker ordering remain separate. Sources: `remaining-condition-callers.txt` and `remaining-condition-effects.txt`.

### Skill floating field overlap callbacks recovered, October 18

Initial **GA_Sagiri_Skill#/8/SpawnActorsInfo/SpawnArray/0** requests **Actor_SagiriSkill_Floating**. Its BeginPlay (`#/7 → ExecuteUbergraph#/5`, 993..1007 → 15..263) calls Actor's parent, separately loads local-client visual assets, waits configured **0.2** and requests **PhysicsFloatingEffect.SetCollisionEnabled(1)**. The overlap event (`#/3 → /5`, 1037..1435) requires owner and OtherActor ability-character casts and **owner.CanAttackTarget(other,null)**, then calls **other.K2_ActivateGameEffectsWithClassByActor(self,owner,GE_Player_Sagiri_Skill4_Float,BufferData(...),null)**. This locates a source/target-aware application request feeding the already recovered Float→passive DEF-debuff chain.

EndOverlap (`#/4 → /5`, 1436..1565) only requires OtherActor's ability-character cast before requesting **OtherActor.SkillEffectComponent.BreakFloatingEffect()**; it does not repeat CanAttackTarget or an explicit source-class filter. EndPlay calls the Actor parent and stops local visual/audio effects, with no explicit all-target control-removal loop. Do not infer immunity success, per-source clearing, repeated damage on every overlap, a real 0.2-second delay or automatic DEF-child removal. The actor has two object-read warnings, while its function streams are complete; invalid inherited defaults are not used. Source: `overlap-and-invisibility-callers.txt`.

### A3 slay-listener routing recovered, October 18

The repaired join is **Buff_Sagiri003_Level3#/2 → GameplayCue.Display.Sagiri.Lv3 → GC_Sagiri_Passive2**, despite the cue's Passive2 name. **OnWhileActiveEvent#/5 → graph#/1:31** caches MyTarget as player, refreshes its player-state binding and enables Tick. RefreshPlayerState compares that player's current player state to CachePlayerState, removes **OnSlay_Event** from the old valid state and binds it to the new valid state. Tick repeats that comparison; CDO **#/8** declares TickInterval 1, not a one-second buff producer.

**OnSlay_Event#/4 → 203** ignores its DeadCharacterName argument. It requires a valid cached player and **IsServerOrStandalone**, then requests **Buff_Sagiri003_Level3_1 on that player**, cue actor SourceObject, level1/stack0. No explicit killer-character, enemy-type, controlled-character or digestion check appears in this callback; the native player-state OnSlay event determines which kills reach it. The requested carrier (`#/1`) adds **DamageUpGeneralBase=0.06** only for source tag **Ability.Player.Sagiri.Skill**, with duration **Sagiri_Lv3_DamUpDur=15**, target cap10 and server/standalone admission. A supplied settled count 2 would provide 0.12 in that additive zone if native per-stack evaluation applies; the graph does not prove two requests settle as two stacks or how expiry refreshes.

OnRemove **#/3 → 155** calls OnPlayerStateChange(null), disables Tick and clears player/state, explicitly removing the old slay delegate. This locates an A3 event/recipient/cleanup route, not a digestion-clock rule or complete kill attribution. Sources: `sakiri-shinku-cue-carriers.txt`, `roster-cue-halo-extraction.json`, `cue-supplemental-joins.json` and **DT_SagiriEffectFigure**. All cited reads are warning-free.

### Suppress marker cue removal body checked, October 18

The warning-free **GE_Sagiri_UltraSkillSuppress_Effect#/3 → GC_Sagiri_UltraSkillSuppress** join reaches a presentation-only Blueprint body: OnWhile **#/3 → graph#/1:310** returns, while OnRemove **#/2 → 10** casts MyTarget to ability character and spawns the configured Niagara at its rootSocket transform. It does not write the control ledger, remove the separately admitted DEF child, change native suppress duration or apply another Burst packet. The actual SkillEffect Suppress object's 3/6 duration selector and the marker's own active-suppress requirement remain the combat consumers; the displayed effect ending is not independent control-expiry evidence. Sources: `form-wrapper-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Sagiri_Skill_001`) and Burst (`Sagiri_UltralSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Eating mode & digestion state machine:** Eating mode hold costs, swallow/spit action rules, digestion ticks, and A6 multi-target digestion.
- **Target airborne detection & A5 reset:** The floating-effect task/callback and selected energy/CD-removal carrier are located above. Native control/task admission, repeated-callback handling, energy-mode settlement and actual cooldown removal remain open.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.


Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Resolve natural jump/plunge dispatch, timing and resource/cancel rules, hold/eating costs and reset rules, Scorch Enhancement trigger/refresh, native Impish Trick control/application ordering and expiry, natural AirAttack energy eligibility, and remaining awakening gates. The DEF child, control callers, magnitude and predicate are located above. The direct one-packet AirAttack input uses `sakiri.direct_plunge_packet_energy_v1` as a tested energy assumption; its exact source row and settlement limits are in the known card. This does not establish natural plunge eligibility.

## Prior findings

- [`reviews/q10-game-action-binding/sakiri-action-binding-model.json`](../../../../reviews/q10-game-action-binding/sakiri-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/sakiri_actions.rs), [`sakiri-normal-chain.json`](../../../../reviews/q10-game-action-binding/sakiri-normal-chain.json), [`sakiri-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/sakiri-skill-burst-packets.json).

## Research history

- **Current question:** Eating Mode hold/swallow and resource settlement, native A5 floating-task delivery, Scorch handles, and control/debuff application ordering.
- **Useful source locations:** `GA_Sagiri_Skill`, `GA_Sagiri_UltraSkill`, `SagiriSkill1Float`, `SagiriSuppressUltraSkill`, `Buff_Sagiri003_Pas1_1`, `Cau_Sagiri_LevelExtra`.
- **Methods already tried:** Saved source graphs now locate the Impish Trick DEF child/predicate/control callers, A1 suppress-duration selector, A5 floating callback and R6 debuff-handle expression. The player-prefix reflection repair recovers selected ability declarations but still fails in inherited Character/movement fields; it supplies no new clean logic payload. The protected native-body route remains exhausted.
- **Next concrete step:** A3's player-state OnSlay binding/cleanup and Skill-tagged damage carrier are now located despite the Passive2 cue name. Resolve native kill attribution/stack refresh, eating/airborne-task/ChargeExecution settlement and clocks. Do not repeat the player-prefix lane merely to obtain a clean export; continue independent roster dependencies.

## Current rolling Burst ATK audit, November 14

The warning-free `GA_Sagiri_UltraSkill.ExecuteUbergraph_GA_Sagiri_UltraSkill` object2 contains two independent execution-flow branches. Offset125 checks `Buff_Sagiri003_Level4`;446 requests `Buff_Sagiri003_Level4_1` only on that branch. The separate flow at589 reaches826 and requests `Buff_Sagiri003_QDamUp` without an awakening predicate. Both run from K2_ActivateAbility after the parent. Their server/hybrid gates concern native authority, not a second combat prerequisite. No delivered-hit or damage gate appears.

`Cau_Sagiri_QDamUp` and `Cau_Sagiri_Level4`, each object2, return0 at52 for OwnerActor==HitActor; otherwise they read their respective curve at152, source `GetAtkBase` at288, and multiply at396. Current `DT_SagiriEffectFigure` rows `Sagiri_QDamUp_AtkAdd`/`Sagiri_lv4_AtkAdd` are both.30; both `AtkAddDur` rows are20. The distinct carriers have HasDuration, AggregateByTarget and cap1. The ordinary carrier binds additive AtkAdd, not AtkUp or recipient ATK. Raw exports, hash-bearing inspection receipts and the curve summary stay in .

**Demonstrated defect:** the previous adapter gated the ordinary QDamUp grant on A4 and never produced the separate Level4 carrier. Correct behavior is ordinary30% plus an additional30% with exactly A4. Source base1000 gives+300 without A4 and+600 with A4, an isolated+300 difference. Neither grants Sakiri herself ATK. Captured source base includes weapon base through shared build projection; console additions, live ATK-percent buffs and recipient base do not alter this basis.

The current `sakiri.burst_atk_v1` managed model applies each cap1 snapshot at admitted Burst activation for20 paused-world seconds to other living teammates, on or off field. Refresh replaces each source's carrier; failed admission emits neither. Existing snapshot lifetime survives source defeat, whereas dead recipients contribute nothing and receive no new grant. This is the selected party/lifetime interpretation of the global component, not a claim to reproduce multiplayer authority/routing. `.mods.team_atk` observes the ordinary carrier's actual deadline; A4 shares that deadline and is separately identifiable in native buff/trace receipts.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
