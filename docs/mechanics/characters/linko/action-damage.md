# Linko action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1072`.

Hexed follow-up reports retain an autonomous `settled_hit` origin pointing to
the actual teammate packet, including Support or autonomous teammates. They do
not inherit that teammate's command or switch ownership. The native passive
GAName directly binds `GA_Radio072_Passive_1` to its damage row; action, packet
and `hexed_reaction` semantic receipts retain the same native row evidence.
The existing owner test checks the independent `1000 * .13 * .5 = 65` noncritical
damage, exact parents and rejected missing/future settlements. This corrects
report identity without changing the selected trigger, coefficient or recipients.
Xiaozhen pulses retain their actual latest deployment command and effective
talent coordinate across field exit and repeat pulses. Recasting replaces both
the generation and source command; defeat clears them. Native Skill/packet and
`xiaozhen_pulse` semantic evidence is preserved instead of assigning command zero.

**Responsibility:** Linko outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_LinkoEffectFigure`, `1072AwakenEffect.json`, `DT_CharacterAbilityConfig.json`, permanent owner inputs (optional `linko.json`), and verification cases in [owning Rust checks](../../../../crates/content/src/simulation/tests/linko_actions.rs). All selected outgoing damage is **Anima / Nature** (`CHARACTER_ELEMENT_TYPE_NATURE`), scales with Linko's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch, E represents Skill/bombardment, Q represents Burst/Oversync, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Radio072_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| `Melee1_Damage` | 1 stage 1 shot / A | .496 / .850 / .918 | 3.100 / 5.200 / .620 |
| `Melee2_Damage` | 2 stage 2 shots / A | .170 / .291 / .315 each | 1.060 / 1.780 / .212 each |
| `Melee3_Damage` | 1 stage 3 shot / A | .404 / .692 / .748 | 2.520 / 4.230 / .504 |
| `Melee4_1_Damage` | 1 stage 4 beam start / A | .133 / .228 / .246 | .830 / 1.390 / .166 |
| `Melee4_2_Damage` | 1 stage 4 beam burst / A | .298 / .511 / .552 | 1.860 / 3.120 / .372 |
| `Melee4_3_Damage` | 5 stage 4 beam flurries / A | .139 / .238 / .257 each | .870 / 1.460 / .174 each |
| `Melee4_Damage` | 1 stage 4 beam finish / A | .344 / .590 / .637 | 2.150 / 3.610 / .430 |
| `Melee5_Damage` | 1 stage 5 heavy finish / A | .740 / 1.268 / 1.370 | 4.620 / 7.760 / .924 |
| `Skill1_Damage` | 1 bombardment rocket 1 / E | .178 / .305 / .329 | 1.490 / 2.500 / .300 |
| `Skill2_Damage` | 1 bombardment rocket 2 / E | .138 / .237 / .255 | 1.160 / 1.950 / .230 |
| `Skill3_Damage` | 1 bombardment rocket 3 / E | .238 / .408 / .441 | 1.990 / 3.350 / .400 |
| `Skill4_Damage` | 1 bombardment rocket 4 / E | .446 / .764 / .826 | 3.760 / 6.300 / .750 |
| `Skill1_ThrowXiaozhen_Damage` | 1 Xiaozhen throw strike / E | .333 / .571 / .616 | 2.000 / 3.350 / .400 |
| `Skill1_ThrowXiaozhen_Damage2` | 1 Xiaozhen blast / E | 1.000 / 1.714 / 1.851 | 6.000 / 10.000 / 1.200 |
| `UltralSkillLTE_AOE_Level4_Damage` | 1 Oversync primary burst / Q | 1.000 / 1.714 / 1.851 | 0.000 / 0.000 / 1.200 |
| `UltraSkill2_Damage` | 1 secondary burst pulse / Q | .576 / .987 / 1.066 | 0.000 / 0.000 / .690 |
| `UltraSkill3_Damage` | 1 heavy resonance explosion / Q | 7.353 / 12.602 / 13.610 | 0.000 / 0.000 / 8.820 |
| `UltraSkill3_Damage_level2` | 1 enhanced resonance detonation / Q | 9.559 / 16.382 / 17.693 | 0.000 / 0.000 / 11.470 |
| `UltralSkillLTE_AOE_Damage` | 1 field pulse / Q | .735 / 1.260 / 1.360 | 0.000 / 0.000 / .882 |
| `LTE_Damage` | 1 Synced Strike / Q | 5.000 / 8.569 / 9.255 | 0.000 / 0.000 / 6.000 |
| `LTE_Xiaozhen_Damage` | 1 Xiaozhen Synced Strike / Q | 5.000 / 8.569 / 9.255 | 0.000 / 0.000 / 6.000 |
| `QTE_Damage` | 1 Support QTE strike / H | 2.000 / 3.428 / 3.702 | 7.500 / 0.000 / 2.400 |
| `Branch_Damage` | 1 branch attack strike / A | 1.000 / 1.714 / 1.851 | 4.000 / 6.700 / .800 |
| `PerfectEvadeAttack_Damage` | 1 riposte initial strike / Z | .255 / .437 / .472 | 1.000 / 1.700 / .200 |
| `PerfectEvadeAttack_1_Damage` | 1 riposte counter-blast / Z | 1.215 / 2.082 / 2.249 | 4.800 / 8.100 / .970 |
| `AirAttack_Damage` | 1 falling plunge / X | .550 / .943 / 1.018 | 2.500 / 4.200 / .500 |

**Action packet totals:** Normal 5-input loop (13 hits total: 1 + 2 + 1 + 8 + 1) delivers **5.911 ATK / ~21.602 raw energy / 36.310 Cycle / 4.318 Break** at Level 8. Modeled ordinary Skill (4 rockets: Skill1 + Skill2 + Skill3 + Skill4) delivers **1.714 ATK / ~8.400 raw energy / 14.100 Cycle / 1.680 Break**. Primary Burst (`UltralSkillLTE_AOE_Level4_Damage`) delivers **1.714 ATK** (enhanced by +30% ratio at A2 to 2.228); Synced Strike (`LTE_Damage`) delivers **8.569 ATK**. Critical Riposte (2 hits) delivers **2.519 ATK / 5.800 raw energy / 9.800 Cycle / 1.170 Break**. Support QTE delivers **3.428 ATK / 7.500 raw energy / 2.400 Break**.

## Selected complete timing specification

**Mixed native coordinates and real-clock execution:** All action and buff timings progress along one continuous real combat time axis. Source montages advance at 1.0 source units per second, modified by hitstop and pause intervals on positive contact. Overlapping pauses on the same caster take the minimum progress rate rather than serially stacking. Cooldowns and the DPS denominator continue uninterrupted during pauses.

- **Skill (`GA_Radio072_Skill`):** Cooldown notify starts at **0.131s** (`AnimNotify_TriggerCooldown`); 4 bombardment packets land at **0.131s** (`Skill1`), **0.315s** (`Skill2`), **0.528s** (`Skill3`), and **0.783s** (`Skill4`). Cancel windows activate at **0.800s** (`BreakSkill_AnySkill`) and **0.955s** (`BreakSkill_Move`); total montage duration is **1.200s**. Real cooldown is **12.0s** (tooltip 6.0s nominal in fast rotation scenarios), continuing off field. Interruption before notify uses cancel cooldown 2.0s; completed casts maintain full nominal cooldown without surcharge.
- **Burst (`GA_Radio072_UltraSkill`):** Cooldown **20.0s** (tooltip 15.0s nominal) starts at commitment. Direct gameplay events land at **0.243s** (`UltraSkill.Primary`) through **2.354s** (`UltraSkill.FieldPulse`); cancel windows activate at **2.602s** (`BreakSkill_AnySkill`) and **2.850s** (`BreakSkill_Move`); total montage duration is **3.200s**. Commit costs **100 energy**. Direct hits do not generate ordinary energy.
- **Normal Attack chain (`GA_Radio072_Melee`):** Five admitted input stages progress with hits at 0.160s (stage 1), 0.120s / 0.240s (stage 2), 0.180s (stage 3), 0.100s..0.700s (stage 4 multi-hit flurry: 8 hits), and 0.220s (stage 5 finisher). Queued full chain completes in **3.200s** (effective total ~4.000s with recovery). An input gap >2.0s resets the chain; legal queued transitions continue.
- **QTE (`GA_Radio072_QTE`):** The selected bounded Support model uses a 0.100-second impact and 0.600-second recovery. The native packet grants 5.200 raw energy and 2.500 Break once under their respective delivery rules. Resonance 3 increases the purchased Support talent coordinate by one. A qualified reaction switch admits this Support with no independent cooldown.
- **Critical Riposte (`GA_Radio072_Dodge` / `PerfectEvadeAttack`):** Evade strike at **0.120s**, counter-blast at **0.380s**, cancel window **0.650s**, total duration **0.800s**.

## Selected complete state and effect specification

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Ultimate energy cap is 100; UltraSkill commits 100 energy. Ordinary energy awards follow per-packet ChargeAdd upon positive target HP loss: 1/1 to active Linko, 3/5 to living off-field allies, with individual Charge Efficiency, integer capacity, fractional carry, and overflow tracking. Cycle gauge has capacity 100, earned on hits; qualifying switch consumes 100.
- **Hexed extension (Passive 1 / `LingZhouReactionExpendOne`):** Linko provides `ExtraDamageRatio = 0.13` and additive `DamageUpGeneralBase = 0.10` via `Buff_Radio072_Passive1` gated by `Con_1072_isXieTongDamage` (target has `Ability.Player.Buff.Reaction.LingZhou` AND [effect has XieTongDamage OR (effect has QTE and caster has DomainServer)]).
- **Spectral Rush (Passive 2):** Periodically every 8.0s (reduced to 6.0s at A1 via `Buff_Radio072_Passive2`), Xiaozhen performs a coordinated attack dealing `GE_Xiaozhen_Skill1_Damage` (0.857 ATK at Lv8).
- **Precision Tuning (Passive 3):** Hits apply `LinkoPas3_Effect = -0.08` (-8% elemental RES) across all 6 damage types (Chaos, Cosmos, Incantation, Lakshana, Nature, Psyche) for `LinkoPas3_Duration = 12.0s`.
- **A1 (Spectral Rush Acceleration):** Reduces Passive 2 interval from 8.0s to 6.0s.
- **A2 (UltraSkill Amplification):** Adds +30% ratio bonus (`LinkoLevel_Extra2_Ling` / `LinkoLevel_Extra2_Zhou` = 0.30) to UltraSkill.
- **A3 (Resonance 3):** Grants +1 talent level to Skill, UltraSkill, and QTE; grants +20% ATK (`Linko_Extra1_AtkUp` = 0.20) to Linko.
- **A4 (Precision Tuning Spread):** Spreads Passive 3 elemental RES reduction to all nearby enemies in range.
- **A5 (Bombardment DMG Bonus):** Grants +50% Skill DMG (`LinkoLevel3_DamageUpGeneralBase` = 0.50) to all 4 bombardment rockets of `GA_Radio072_Skill`.
- **A6 (Resonance Field & Skill Crit):** Grants +25% Crit Rate (`LinkoLevel6_CritBase` = 0.25) to Skill rockets. Extends Burst Resonance Field duration from 8.0s to 13.0s (`LinkoLevel_Extra2_Duration` = 13.0), granting +30% Nature/Anima DMG and +30% Incantation DMG to all team members (`TeamAll`).

## Selected complete recipient specification

**All-in-range dummy contract and single-target rules:**
- **Burst AoE (`GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage`):** Under the all-in-range dummy contract, every declared scenario dummy receives the primary Burst hit in ascending target_index order at the primary impact timestamp.
- **Resonance Field Team Buff:** Deployed by UltraSkill, applies to all living team members (`TeamAll`), including active and off-field characters.
- **Normal attacks and Skill bombardment rockets:** Single-target projectiles delivered to primary target (dummy index 0). Defeated dummies do not absorb hits.
- **Passive 3 Resistance Shred:** Applied directly to target hit; under A4 spreads to all living scenario dummies.

## Complete implementation handoff and two continuing cycles

**Complete build and verification reference:** Built under frozen package `global-2026-10-04` using Gold Record (`weapon:fork_GoldRecord`, T1, base ATK 570), Anima set (`set:Suit2`), `AtkUp` cartridge, and radio console (`EquipmentSlots_radio`, 20 cells full occupancy). Verified static stats: Base ATK 1222, live ATK 2100.25 (rounds to 2100), HP 21114, DEF 921, CR 0.45, CD 0.50, CE 1.0.

**Two continuing cycles walkthrough (60.0s period, 120.0s total):**

- **Cycle 1 (0.0s - 60.0s):**
  - **0.000s:** Enters field via QTE (`GA_Radio072_QTE`), dealing 3.428 ATK (7.500 raw energy).
  - **1.000s - 4.200s:** Normal Chain Loop 1 (5 inputs, 13 hits: Melee1..Melee5). Delivers 5.911 total ATK, generating ~21.602 raw energy. Accumulated energy: ~29.102.
  - **5.000s - 6.200s:** Skill cast (`GA_Radio072_Skill`). 4 bombardment rockets deliver 1.714 total ATK (enhanced by +50% DMG at A5 and +25% CR at A6). Generates ~8.400 raw energy. Nominal cooldown 12.0s triggers. Accumulated energy: ~37.502.
  - **7.000s - 16.600s:** Normal Chain Loops 2, 3, 4 (15 inputs, 39 hits). Generates ~64.806 raw energy. Energy pool reaches 100.0 cap at ~16.0s (overflow safely tracked).
  - **17.000s - 18.200s:** Second Skill cast (`GA_Radio072_Skill`, off CD). Delivers 1.714 ATK.
  - **19.000s - 22.200s:** Burst cast (`GA_Radio072_UltraSkill`, Oversync). Commits 100.0 energy (pool resets to 0.0). Deals primary AoE hit (1.714 ATK, boosted by +30% ratio at A2 to 2.228 ATK). Deploys Resonance Field (+30% Nature/Incantation DMG) for 13.0s (until 32.0s).
  - **23.000s - 60.000s:** Post-burst continuation: 8 Normal chain loops and 3 Skill casts on cooldown. Generates >100 raw energy, refilling energy pool to 100.0 cap by ~45.0s. All cooldowns expire before cycle end.

- **Cycle 2 (60.0s - 120.0s):**
  - **60.000s:** Commences second cycle from live state with 100.0 energy, ready Skill and Burst.
  - **60.000s - 63.200s:** Immediate Burst cast (`GA_Radio072_UltraSkill`). Commits 100.0 energy, re-establishes Resonance Field for 13.0s.
  - **64.000s - 65.200s:** Skill cast (`GA_Radio072_Skill`), generating ~8.400 energy.
  - **66.000s - 78.800s:** 4 Normal Chain Loops (52 hits), generating ~86.408 energy.
  - **80.000s:** Second Skill cast, bringing energy back to 100.0 cap.
  - **81.000s - 120.000s:** Sustained Normal and Skill rotation maintains uptime and energy readiness, completing 120.0s simulation without resource leakage, state desync, or reset.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,1,8,1 (thirteen hits). The chain loops, retaining native effective talent projection with Normal attack excluded from resonance_3 level bonus.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`linko-normal-chain.json`](../../../../reviews/q10-game-action-binding/linko-normal-chain.json).

- GA_Radio072_Melee admits GE_Player_Radio072_Melee1_Damage Native coefficients at Normal levels 1 and 8 are 3.450 and 5.911 across the chain; category A/NATURE metadata stays source-bound. Normal attack is excluded from resonance_3 level bonus. See linko-normal-chain.json.
- GA_Radio072_Skill admits GE_Player_Radio072_Skill1_Damage and schedules Skill2, Skill3, and Skill4 once each in order under linko.skill_four_hits_v1. Native total coefficients at Skill levels 1/8/9 are 1.000/1.714/1.851; each packet retains category E/NATURE metadata and native effective talent projection. See linko-skill-four-hits.json. At A5, +50% Skill DMG applies to all four packets; at A6, Skill crit rate remains packet-local.

## Live Oversync and Xiaozhen bindings

The selected executable model exposes `.linko.mods.oversync`, `.linko.oversync.active` and `.linko.xiaozhen.active`. Absent permission reads zero/false. Oversync starts at Burst commitment and lasts 15 World seconds, with `now < until` required by both the query and LTE admission. An LTE admitted one microsecond before expiry can deliver afterwards; an input at expiry is rejected. Oversync is separate from the 8/13s Resonance Field. Switching preserves permission; defeat clears it and the owner's Hexed tracking.

Source Skill declares the typed `linko.xiaozhen_pet_v1` model. Canonical Skill delivers its four bombardment rockets and deploys Xiaozhen once; it does not also append the two Throw packets. An explicitly selected `Skill1_ThrowXiaozhen_Damage` primary retains its two-packet train and deploys the same companion. This selection follows the distinct ordinary and alternate packet rows rather than counting both routes as one cast. It revises the earlier pet-enabled double counting and is a model choice, not proof of native route eligibility.

The first autonomous pulse is 8s after the admitted primary delivery, reduced to 6s at A1, with the same continuing interval. A field exit preserves the companion. Recasting replaces its generation and cadence; defeat clears deployment and invalidates the old generation, including pending pulses. No separately consumed Xiaozhen energy pool is selected. Pulse coefficients inherit the deploying Skill's effective native talent coordinate, including resonance, instead of a fixed L8 coefficient; other damage zones and the existing resource policy remain unchanged. Source execution uses existing bounded caller timing; montage, travel and native summon-route applicability remain unresolved.

Independent Rust checks distinguish the four L1 rockets (1.000 total ATK) from the separate .500 ATK pulse: at 1000 ATK and a .5 DEF multiplier, these deal 500 and 250 noncritical damage. The L8 pulse deals 428.5; A3 promotes it to L9 and 1200 ATK, giving `.925 × 1200 × .5 = 555`. With a 100us caller primary delay, the first A1 pulse occurs at 6,000,100us; the A0 Throw route's first pulse occurs at 8,000,100us. Recasting at 6s replaces the old 8s pulse with 14s/22s deliveries. Source tests exercise two continuing paid Bursts, natural energy refill, off-field pulses, exact Oversync expiry and owner death, then reconcile direct native replay. These are executable model checks, not frame-measured gameplay confirmation.

## Action packets and owner effects

- GA_Radio072_UltraSkill public primary packet is GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage. Native coefficients at Burst levels 1/8/9 are 1.000/1.714/1.851; category Q/NATURE metadata stays source-bound. At A2, adds +30% ratio bonus to UltraSkill. At A6, deploys Resonance Field team buff (+30% Nature/Incantation DMG) for 13s.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Skill, UltraSkill, and QTE, and +20% ATK to Linko.
- Existing A2 ratio (+30%), A5 Skill DMG (+50%), A6 Skill Crit Rate (+25%), and A6 Resonance Field team buff remain preserved as partial kit models.
- Canonical arbitrary awakening selections: Effect2 modeled Burst ratio, Effect5 modeled Skill damage and Effect6 modeled Skill crit use selected identities. ATK/talent resonance_3 and field/team resonance_6 remain count-based. Existing unresolved coordination and packet-role applicability limits are not promoted by selection. Existing unsupported owner mechanics remain pending.

## Recovered Hexed extension inputs

**Source facts for PC_140/logic-566459; no executable passive change.** The corrected reaction-extension acquisition restores row **1072** and its `LingZhouReactionExpendOne` payload. It compares logically equal to the frozen row: `ExtraDamageRatio=0.13`, an explicitly null **`SouceGEClass`** (native spelling), and explicitly zero `ExtraFuWenDamageRatio`, under condition `Buff_Radio072_Passive1`, with `ReactionAbilityID=GA_Radio072_Passive_1`. Zero/null are serialized values; they are kept distinct from absent fields.

The frozen Reference guide description says that Linko raises Hexed's additional follow-up base damage to 30% and that Hexed targets take 10% more follow-up damage. Preserve the 0.13-versus-0.30 coefficient question rather than replacing the saved value. The separate 10% has a reached carrier: `Buff_Radio072_Passive1` declares conditional **additive `DamageUpGeneralBase`**, explicit scalar 1 and constant curve `DT_LinkoEffectFigure:LinkoPas_DamageUpGeneralBase=0.10`, gated by `Con_1072_isXieTongDamage`. Thus the zero `ExtraFuWenDamageRatio` is not proof that every related bonus is zero, and this carrier is not a second independent multiplier or a blanket bonus to all Linko damage.

That predicate's `/2/ScriptBytecode` resolves a player from **SourceActor**, or from SourceActor's owner if the direct cast fails (115/180, 912/954/1019/1029), and requires a valid player state/main character (214–416). It reads **TargetActor's ASC** at 80 and the tag `Ability.Player.Buff.Reaction.LingZhou` at 472. Against the supplied effect's asset tags, CDO `Tag Container` is `Ability.Player.Nanally.XieTongDamage`, and `Tag Container_0` is `State.Damage.QTE`. The main character's `Buff_PlayerRadio072_DomainServer` presence is checked at 583. Comparisons 634/685 and boolean operations 736/774/812/850 produce:

```text
target_has_LingZhou AND (effect_has_XieTongDamage OR
                        (effect_has_QTE AND main_character_has_DomainServer))
```

The Nanally-named tag is a literal source label, not evidence restricting the predicate to Nanally. Target status, effect tags and the source-resolved main-character buff are distinct reads. Native buff admission, asset-tag inheritance, getter capture, conditional-modifier application and death/field-change behavior remain unresolved.

**Independent explicit-input distinction:** for a declared 1,000-unit coefficient base, ratios 0.13 and 0.30 give 130 and 300 before other zones. A relative 30% increase to 0.10 would instead yield 0.13; that arithmetic is a possible explanation, not a selected native formula or a repair of the published absolute wording. With the target tag present, XieTong-tagged effects pass without DomainServer, whereas QTE-only effects require DomainServer; without the target tag both fail. An admitted 0.10 carrier with existing general bonus 0.20 gives **1.30**, not `1.20 × 1.10 = 1.32`. Native trigger, owner/target binding, coefficient composition and conditional-bonus capture remain in [unknown](#unknown-execution-policies).

## Configured Burst ability grants

**Serialized source facts, not LTE admission.** The compact effect table (effect_parameters.jsonl), record `Buff_PlayerRadio072_UltralSkill_ChangeSkill`, retains `/3/Properties` `AbilitiesGameplayEffectComponent`. In source-array order, it grants `GA_Radio072_UltraSkillChild` with InputID **2** and **`CancelAbilityImmediately`**, then `GA_Radio072_SubUltraSkill` with InputID **3** and **`RemoveAbilityOnEnd`**. Both explicitly serialize level parameter **1**. The different removal labels must not be flattened into one cleanup rule. Native grant/input replacement, LTE state/resources, effective level and removal timing remain in [unknown](#unknown-execution-policies); array order is not proof of native callback order.

## Ordinary-hit Ultimate Energy

The current `linko.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 21.602, Skill 8.4 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`linko-skia-hathor-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/linko-skia-hathor-ordinary-hits.json).

### Zero-start continuing rotation

The checked A0 slice uses four five-input Normal chains, two Skills, one 100-energy Burst, then a fifth Normal chain after the spend. All 74 damage packets are checked in order; the 73 ordinary Normal/Skill packets use the existing energy hypothesis. With 1000 ATK, zero bonuses, and a 0.5 defense multiplier, the independent damage total is 17,348.5 (five Normal chains 14,777.5; two Skills 1,714; Burst 857). ChargeAdd reaches 103.208 before Burst: 103 whole energy is generated, 3 overflows at capacity 100, and the Burst spends 100. The continuation ends at 21 energy with 81/100 carry; each other living recipient ends at 74 with 443/500 carry.

Inputs use the tested 800ms Normal recovery, 1.2s Skill recovery/6s cooldown, and 2s Burst recovery. The 36s NTE rotation, Rust request, source/ally packet ledgers, observations, aggregation/replay and native/WASM/Worker output are checked together. This establishes only the named ordinary chain/Skill/Burst sequence and existing energy hypotheses; native eligibility, packet timing, readiness waiting and the pending alternate forms/summons/QTE remain open. Receipt: [`linko-continuing-energy.json`](../../../../reviews/q10-energy-ledger/linko-continuing-energy.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`linko.skill_animation_profile_v1`, `linko.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`char_radio_skill_skill1_1_Montage_New`), gameplay events land at 0.131s (`Skill.1`), 0.282s (`Skill.2`), 0.489s (`Skill.3`), and 0.783s (`Skill.4`); cancel windows activate at 0.178s (`BreakSkill_Jump`), 0.955s (`BreakSkill_AnySkill`), and 1.099s (`BreakSkill_Move`) with total duration 5.450s. For Burst (`char_radio_skill_ultraskill_Montage` / `char_radio_skill_ultraskill_1_Montage`), `UltraSkill.1` lands at 0.243s, six `UltraSkill.2` hits land at 1.528s–2.107s, and `UltraSkill.3` lands at 2.354s; cancel windows activate at 2.602s (`BreakSkill_Move`, `BreakSkill_AnySkill`, `BreakSkill_Jump`) with total duration 6.350s.
- **Multi-target delivery (`linko.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Linko's direct Burst packet (1x `GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/linko-action-binding-model.json`](../../../../reviews/q10-game-action-binding/linko-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/linko_actions.rs), [`linko-normal-chain.json`](../../../../reviews/q10-game-action-binding/linko-normal-chain.json), [`linko-skill-four-hits.json`](../../../../reviews/q10-game-action-binding/linko-skill-four-hits.json).
- Frozen inputs: `data/extracted/guide_linko.json`, `data/extracted/neverness_linko.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/radio072.rs) (original `DT_SkillDamageData` export is cache-only), `1072AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed EffectFigure parameters are typed in [`linko/parameters.rs`](../../../../crates/content/src/characters/linko/parameters.rs); unconsumed source rows are cache-only. The existing owner model and its remaining implementation limits are preserved.

- Branch/Air attacks, SubUltraSkill, BackToLTE/FrontToLTE QTE modes, Xiaozhen summon actions, coordination and the Hexed extension remain unimplemented.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/linko). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Linko (`GA_Radio072_*`):

| Action / damage row (`GE_Player_Radio072_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.133–0.740 / 0.228–1.268 / 0.246–1.370 | 0.835–4.635 / 1.388–7.703 / 0.166–0.923 per hit |
| Branch `Branch_Damage` | A | 1.000 / 1.714 / 1.851 | 5.000 / 8.300 / 1.000 |
| Plunge `AirAttack_Damage` | X | 0.550 / 0.943 / 1.018 | 2.800 / 4.600 / 0.600 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 0.255 / 0.437 / 0.472 | 0.850 / 1.417 / 0.322 |
| Critical Riposte `PerfectEvadeAttack_1_Damage` | Z | 1.215 / 2.082 / 2.249 | 4.050 / 6.750 / 1.534 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 5.200 / 0.000 / 2.500 |
| Synced Strike LTE `LTE_Damage`, `LTE_Xiaozhen_Damage` | H | 5.000 / 8.569 / 9.255 | 0.000 / 15.000 / 2.500 |
| Skill 4-stage `Skill1_Damage` .. `Skill4_Damage` | E | 0.138–0.446 / 0.237–0.764 / 0.255–0.826 | 1.162–3.749 / 1.951–6.292 / 0.235–0.759 per hit |
| Throw Xiaozhen `Skill1_ThrowXiaozhen_Damage` | H | 0.333 / 0.571 / 0.616 | 0.000 / 0.000 / 0.417 |
| Throw Xiaozhen `Skill1_ThrowXiaozhen_Damage2` | H | 1.000 / 1.714 / 1.851 | 0.000 / 0.000 / 1.250 |
| Xiaozhen Assist `GE_Xiaozhen_Skill1_Damage` | A | 0.500 / 0.857 / 0.925 | 0.000 / 0.000 / 0.000 |
| Burst `UltraSkill2_Damage` | Q | 0.576 / 0.987 / 1.066 | 0.000 / 0.000 / 0.115 |
| Burst `UltraSkill3_Damage` | Q | 7.353 / 12.602 / 13.610 | 0.000 / 0.000 / 0.735 |
| Burst A2 `UltraSkill3_Damage_level2` | Q | 9.559 / 16.382 / 17.693 | 0.000 / 0.000 / 0.735 |
| Burst LTE AoE `UltralSkillLTE_AOE_Damage` | Q | 0.735 / 1.260 / 1.360 | 0.000 / 0.000 / 0.074 |
| Burst LTE AoE A4 `UltralSkillLTE_AOE_Level4_Damage` | Q | 1.000 / 1.714 / 1.851 | 0.000 / 0.000 / 1.500 |

Critical Riposte uses GAName `GA_Radio072_PerfectEvade`, Support QTE uses `GA_Radio072_QTE`. The cited direct damage rows use native NATURE (Anima), not COSMOS. Skill delivers 4 hits (`Skill1` through `Skill4`). Burst delivers Oversync (`UltraSkill2/3`) and sets up Resonance Field, enabling Synced Strikes (`LTE_Damage`).

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Radio072_Passive_1` binds `Buff_Radio072_Passive1`. `DT_ReactionExtendDataTable` row 1072 binds `ReactionAbilityID=GA_Radio072_Passive_1` with `ExtraDamageRatio = 0.13` (Hexed extension; native result classification belongs to the shared reaction owner) and conditional 10% follow-up bonus (`Con_1072_isXieTongDamage`).
- **Passive 2 (Breakthrough 4):** `GA_Radio072_Passive_2` binds `Buff_Radio072_Passive3`.

### Awakenings

Source descriptions and selected IDs: `1072AwakenEffect#/rows/1072/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Reduces Spectral Rush's appearance interval by 25% while Linko is off-field.
- **A2:** Increases the DMG Ratio of Oversync's Anima Burst DMG by +30% (`UltraSkill3_Damage_level2`).
- **A3:** Resets Full-Frequency Pulse's cooldown upon casting Oversync.
- **A4:** Deals 1 additional instance of Anima DMG in an area each time a Synced Strike is triggered within Resonance Field (`UltralSkillLTE_AOE_Level4_Damage`). Allows this attack to consume a "Hit 'Em Here!" mark and trigger a Synced Strike.
- **A5:** Allows Synced Strikes triggered by Full-Frequency Pulse to trigger Cycle effects. Increases this Synced Strike's DMG by +50% if the selected ally cannot trigger a Cycle with Linko.
- **A6:** Increases allies' Synced Strike Crit Rate by +25% while Linko is on the team; grants allies' Synced Strikes the Charge effect.
- **Resonance 3:** Native `1072AwakenEffect/1072/AwakenEffectStructList` modifies **GA_Radio072_QTE, GA_Radio072_Skill and GA_Radio072_UltraSkill** by +1, with the separate +20% ATK carrier. Normal is excluded. The English description names Spectral Cross, while the Chinese description and actual modification list identify Support; preserve this localization conflict instead of granting an extra Normal level.
- **Resonance 6:** Extends Resonance Field's duration by +5s and increases all allies' Anima and Incantation DMG by +30% while it is active.

### Burst target-count getter and LTE return admission recovered, October 18

The initial bundle's warning-free `LinkoUltraSkill3_DamageUp:CustomCalculateBaseMagnitude` (`#/1`) uses **owner player GetAttackTarget()**, then `GetBuffCountByType(that target, 3)`, multiplied by `DT_LinkoEffectFigure:LinkoUltraSkill3_DamageUpGeneralBase` at coordinate zero (saved **0.04**). It does not directly read HitActor or count every debuff on all dummies. Supplied counts 0/3/10 yield 0/0.12/0.40; the function shows no explicit count cap. `GE_Player_Radio072_UltraSkill3_Damage` and `_Damage_level2`, both `#/3/Properties/CustomApplicationModifierInfos`, bind this helper with multiplier 1 to additive **DamageUpGeneralBase**. The destination is located; native meaning of type 3 and attack-target capture remain open. Do not assume an always-on 4% per arbitrary debuff multiplier from the helper name.

`GA_Radio072_QTE_BackToLTE:K2_CanActivateAbility` (`#/4`) obtains the owning player controller and returns **NOT IsGamePausedByType(4)** on that valid path. Activation calls the shared QTE parent. This locates one return-to-LTE admission condition, without recovering the pause-type enum, all parent costs, natural dispatch or cleanup. The expansion's player `Player_072_radio#/18` contains an empty `LinkoLogicObj` instance in a warning-bearing player export, not a Blueprint LTE state machine. Remaining coordination/resource/mutation behavior cannot be inferred from that empty instance.

### Passive description coverage, reviewed October 18

Saved description source: `guide_linko.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Precision Tuning describes the Synced Strike initiators element selecting an 8% Esper RES reduction for 12s. Different elements can coexist; same-element applications do not stack and refresh duration. Native initiator/source attribution, exact tag admission and refresh/removal remain open; this is separate from the 0.13-versus-30% Hexed extension conflict.

### Precision Tuning elemental carriers and owner lookup recovered, October 18

The expansion's `Buff_Radio072_Passive3_{an,guang,hun,ling,xiang,zhou}`, each `#/1/Properties`, bind additive **DamageResistChaosBase / CosmosBase / PsycheBase / NatureBase / LakshanaBase / IncantationBase**, respectively, to `DT_LinkoEffectFigure:LinkoPas3_Effect=-0.08`, multiplier 1, and duration `LinkoPas3_Duration=12`. Each is AggregateByTarget with limit 1; no explicit refresh policy is serialized. Six distinct classes support different-element coexistence, without proving every class is delivered on every hit or unlimited stacking of one element.

`Con_1072_Pas3:CanApplyGameplayEffectContent` (`#/2`, warning-free) uses SourceActor if it casts to a player, otherwise its owner if that casts, then gets the player's party member by **character ID 1072** and tests that member's `Buff_Radio072_Passive3`. It does not require the strike initiator to be Linko herself. Initial `GA_Radio072_QTE_BackToLTE#/5/Properties/EffectContainerMap` requests the **Nature** child with its QTE.1 LTE damage; `GA_Radio072_QTE_FrontToLTE#/6` requests that same child in its spawned LTE actor's effect container. These locate specific paths, not the native selector for all allied elements. Other-element admission, Synced Strike attribution and refresh clocks remain open.

### Sync Frequency damage expressions recovered, October 18

The expansion's warning-free `LinkoSkill_DamageUp:CustomCalculateBaseMagnitude` (`#/1`) casts OwnerActor to player, gets player-state **GetMainCharacter()**, obtains that character's LinkoLogicObj, and on successful curve reads returns **1.5 + 0.004 * GetLinkoSyncFrequencyValue()**. Its CDO ClampValue=(0,5,0) is not used as a return clamp. `Buff_Player_Radio072_SkillEffect` and `Buff_Player_Radio072_Skill_DamageUp`, both `#/2/Properties/CustomApplicationModifierInfos`, bind it with multiplier 1 to additive **DamageUpGeneralBase**, gated by `Con_1072_isQTEDamage`. This is a located frequency-dependent bonus, not a new interpretation of Sync Frequency as Ultimate Energy.

`LinkoSkill_xiaozhen_DamageUp#/1` separately looks up party character **1072** for its LinkoLogicObj, counts living equipped players through CharacterIsAlive(), and returns **1.5 + 0.004*frequency + 0.25*max(4-equipped_count,4-living_count)** on its successful path. `GE_Player_Radio072_Skill1_ThrowXiaozhen_Damage` and `...Damage2`, both `#/2`, bind this variant to additive DamageUpGeneralBase. At supplied frequency 100, four equipped/four living gives **1.9**; four equipped/three living or three equipped/three living gives **2.15** for Xiaozhen. These are helper magnitudes, not final damage factors. Native frequency mutation/capture, effective main-character lookup and QTE-tag admission remain open; the two lookup paths must not be silently unified.

### Copied stock-cost graph is conditional source, October 18

Initial warning-free **GA_Radio072_Skill_Throw_XiaoZhen:OnReceiveEvent** (`#/5`) contains a copied Chiz branch: it requires the selected **EffectContainerMap** entry to contain **GE_Player_Chiichan073_Skill2_CoinDamage**, then a valid avatar with active **ChiiChanLogicObj**, before requesting CostAndRecordValueForCurCoinDam(LoopCostVal,1). It always proceeds to the montage parent afterward. The reached CDO (`#/6`) instead names Radio072 ThrowXiaozhen damage/hit-back/passive classes and no Chiichan coin packet in either event container. This source therefore does not establish a Grain cost for Linko's ordinary throw or reinterpret Sync Frequency as Chiz's wallet.

Its separately named **ConditionalSettleStock** (`#/2`) is gated by bHasRecordedStockPrice, non-None SettleStockSectionName, SettleInterval and a SectionCostCoinMap lookup before obtaining ChiiChanLogicObj. The reached CDO omits that map; no caller to this helper is recovered in the same Blueprint function streams. Native/inherited mutation or dynamic invocation is not excluded, but this copied helper is not a supported Linko resource mechanic. The located frequency-dependent modifier consumers above remain the relevant source; actual frequency mutation/admission still requires LinkoLogicObj. 
### Spectral Rush periodic and role-out source routes recovered, October 18

The expansion's internal **Buff_Radio072_Passive2#/2** declares ENTER_BATTLE and RELIVE grants for two separate child carriers, and LEAVE_BATTLE removal of both. Its warning-free predicates **Con_Radio072_Passive2_1#/2** and **Con_Radio072_Passive2_2_Awake_A#/2** look up party character **1072** from SourceActor's player state and respectively require absence/presence of **Buff_Radio072_Level1**. The two Relive variants add **SourceActor.GetPlayerFightState()**. This is the A1 cadence branch; internal Passive2 naming must not overwrite public Passive 2, which binds **Buff_Radio072_Passive3** for elemental RES reduction.

The children **Buff_Radio072_Passive2_1#/2** and **...Passive2_2_Awake_A#/2** are Infinite, DeadRemove, cap 1, with configured period multiplier **1** and rows **LinkoPas2_Xiaozhen_AtkPeriod=8** / **...AtkPeriod_Awake=6**. Both explicitly disable periodic execution on application. Their PERIODIC entries select either **GA_Radio072_Pas2_BuffVersion** under **Con_Radio_Pas2**, or self **Buff_Radio072_ExtraXiaozenMark** under **Con_Radio_Pas2_false**. Those predicates (`#/2`) require GetMainCharacterID=1072 and distinguish **SourceActor != GetMainCharacter()** from equality. Thus the selected path differs for a Linko actor in reserve versus the main actor; it is not a blanket pause/reset of the same timer whenever Linko is on field. Native initial carrier admission and party-wide event/source routing remain unrecovered.

**Buff_Radio072_ExtraXiaozenMark#/1** is Infinite/cap 1. On CHANGE_ROLE_OUT it requests the same GA under **Con_IsServerOrStandalone**, and separately removes itself. Its global component (`#/2`) has NonSourceRemoveAll=true. This locates a delayed role-out route alongside periodic requests, without proving immediate execution, double application or repeat counts. The GA's reached CDO (`#/4`) maps Event.NoMontage.Skill.01 to **Buff_Xiaozhen_Display** through **DamageRange_Radio072_Pas2_BuffVersion**; its Blueprint activation delegates to GA_TriggerGEBase and has no manual extra damage call. One GA-component export retains a warning, so this is a scoped CDO/function reading.

The warning-free range's **K2_SelectOutHitResults#/2** selects the **nearest actor position to OwningActor** among supplied InHitResults, with strict `<` replacement and first-input tie retention. It does not sort by HP or marked-target priority. It returns a one-entry OutHitResults array, empty OutActors and boolean false; with no candidates it still packages its unassigned OutResult. Native interpretation of that false result, empty/default-hit handling, collision filtering and eligible target set remain open. Its CDO has HalfSize=(3000,3000,3000), MaxCount=25; those are query configuration, not a guarantee of 25 marked recipients. **Buff_Xiaozhen_Display#/1** lasts 1.5 and disables periodic execution on application, but contains no explicit damage/mark event in its reached Properties. Actual mark placement/consumption and Synced Strike dispatch still require the native consumer. Sources: `additional-packet-and-charge-helpers.txt`, `passive-execution-bindings-compact.txt` and `skia-linko-passive-carriers.txt`.

### Current A3 cooldown caller versus separate saved mark chain, October 18

Initial warning-free **GA_Radio072_UltraSkill#/6 GAComponent_AddOrRemoveGE** names **Buff_Radio072_Level3_1** in ActivatedAddGEs. That carrier (`#/2`) lasts **.5**, has State.Common.IgnoreTimePause and requires **Con_1072_Level3**, which checks SourceActor's Level3 marker. Its **TriggerCoolDownOrExtendDurationComp#/4** selects **ExtraCooldownTags=[CoolDown.Player.Skill.Linko]**, **ModifyDuration=-20**. This locates the current description's A3 Burst-to-Skill cooldown request; native start/remaining-duration/clamp and the trigger component's execution remain separate.

A different saved source chain must not silently become an additional A3 benefit. Warning-free **GA_xiaozhen_Skill1#/3**, under `Abilities/Monster/mon_Radio`, maps **Event.Montage.Mon.act01.1** to GE_Xiaozhen_Skill1_Damage, a monster hit-back and **Buff_Xiaozhen_Mark_Trigger**. The trigger (`#/1`) declares BEGIN requests for **three Buff_Xiaozhen_Mark stacks** under Con_Radio072_CanAddMark_awakened, or **one Buff_Xiaozhen_Mark_Unawakened** under its companion condition. The Mark carrier (`#/1`) has target cap3; Unawakened (`#/2`) has cap1, a self-overflow reference and a BEGIN request for the same Xiaozhen damage/hit-back using SourceObject. Its follow-up component (`#/3`) names **CheckAbility=GA_Radio072_Melee**, not the currently described allied Redirect trigger. All Mark/Child/Unawakened duration curves are **10**, multiplier 1. These configure a separate saved monster/summon route; the source's test-montage reference, A3 dependency and Melee check differ from current descriptions. Native LinkoLogicObj reachability or an attributable observation is needed before admitting it as the current Spectral Rush ledger.

The two warning-free mark conditions (`#/2`) are **side-effecting**, not read-only readiness queries. Both enumerate **GetAllActorsInRange** centered on TargetActor with vector (20000,20000,20000) and native query byte **8**, without source-filtering existing marks. Unawakened returns false after finding any actor with its mark; if that actor is the proposed target, it first requests another copy there. With no existing marked actor, its valid source-character branch looks up party 1072 and requires absence of Level3. Awakened scans **Mark OR MarkChild**; an existing marked actor other than the proposed target rejects. If the proposed target already has Mark, a local 0..2 loop requests that same carrier **three times**, then returns false. If no marked actor is encountered, its valid SourceActor-as-ability-character branch tests that source's own Level3. The asymmetry in owner lookup and mutation means these cannot be reduced to a generic three-target mark cap. Native re-entry/overflow suppression, context identity and current applicability remain open.

Mark's BEKILL event separately requests **Buff_Xiaozhen_Mark_DeadSpreadTrigger** under **Con_Xiaozhen_Mark**, which casts SourceActor to MonsterCharacterBase and checks its Mark. The spread trigger lasts .5 and allows addition to dead actors; it contains no explicit spread recipients or damage in reached Properties. **BP_xiaozhen:ReceiveTick** checks Buff_Xiaozhen_KillSelfTrigger, then writes LimitCanbeDamage=0 and LimitReduceHP=0 before requesting Buff_mon_KillSelf; its empty Begin/EndPlay paths do not recover mark dispatch. Five BP_xiaozhen exports have read warnings and its CDO includes incoherent inherited values, so those values and its copied sheep-QTE helper are not accepted summon defaults. This is a scoped function/caller finding and an applicability conflict, not current mark completion. Sources: `linko-mark-producer-refs.txt`, `linko-summon-config.txt`, `mark-and-delayed-actor-callbacks.txt` and `xiaozhen-actor-functions.txt`.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Radio072_QTE_FrontToLTE#/6; NoMontage.Skill.01 | DamageActor_Radio072_LTE | (0.0, 0.7, 0.0, 1.0, 1, true) | GE_Player_Radio072_LTE_Xiaozhen_Damage |

The NoMontage.Skill.01 LTE actor carries LTE_Xiaozhen_Damage, HitBack, Passive3_ling and TriggerSkillLTE_AOE in one container after configured DelayTriggerTime=0.7. Its LinkoCanSpawnDamageActor predicate is independently bound at this same spawn entry; see the role guard below. These support/trigger carriers are not three additional direct damage packets.

### LTE actor role guard recovered, October 18

Warning-free **LinkoCanSpawnDamageActor:BPGetCanSpawnActor#/1**, 10..413, returns false when **(!IsHybridMode(owner) && owner.GetLocalRole()==2)** or **(IsHybridMode(owner) && owner.GetRemoteRole()==2)**, true otherwise. It is actually bound at **GA_Radio072_QTE_FrontToLTE#/6/Properties/SpawnDamageTriggerActorsInfo/SpawnArray/0/SpawnConditionObjs/0**, the same LTE entry joined above. Thus this is a reached source gate on actor creation, not a global rule that every Linko damage packet uses a particular role. The saved byte **2** and the two role getters are preserved explicitly; no unverified local/server ownership label or predicted-packet deduplication is substituted. Native owner assignment, successful spawn and delivery still determine applicability. Source: `linko-spawn-guard-callers.txt`.

### A5/A6 conditional damage and Charge blockers recovered, October 18

Warning-free **Con_Radio072_isAwake_CritUp:CanApplyGameplayEffectContent#/2** tests **SourceActor as player → Has(Buff_Radio072_Level6)**. It gates additive **CritBase=0.25** in each reached **GE_Player_Radio072_LTE_Damage, ...LTE_Xiaozhen_Damage, ...Skill1_ThrowXiaozhen_Damage and ...Damage2**, all `#/2/CustomApplicationModifierInfos`, as well as admission of **Buff_Radio072_CritUp#/3**. This is a concrete source-marker gate, not a test that every trigger recipient has A6.

The CritUp carrier (`#/2`) lasts configured **30**, target-cap1, and conditionally adds the same 0.25 CritBase under **Con_1072_isQTEDamage#/2**. That helper's successful expression is **(effect has State.Damage.QTE OR Ability.Player.Linko.XiaozhenDamage) AND NOT Ability.Player.Linko.XiaozhenFront**; CDO `#/1` supplies the exact three tag containers and ServerCheck=false. It is not a universal bonus on every coordinated attack. **Buff_PlayerRadio072_DomainDisplay#/1** requests CritUp, LevelExtra2_1 and DomainServer on BEGIN and removes them on FINISH, locating a grant/removal route despite its Display name. Direct LTE_Xiaozhen_Damage carries XiaozhenFront and is excluded from this carrier predicate while retaining its own direct modifier. LTE_Damage instead carries XiaozhenDamage without Front; if both paths are admitted, their apparent overlap needs native carrier-recipient/application verification. Do not silently count two 25-point bonuses or assert blanket deduplication.

The two ThrowXiaozhen classes also bind additive **DamageUpGeneralBase=0.5** via misleadingly named **LinkoLevel3_DamageUpGeneralBase**, under **Con_1072_isLevel5#/2**. The condition obtains SourceActor's player state, selects party character **1072**, then checks that character's **Level5** marker. This is an A5 source route; the curve's Level3 name does not make it A3. It shows no ally-element compatibility test, preserving the conflict with A5's description-level incompatible-Cycle restriction. The separate frequency/party-size calculator above remains additive in the same damage-bonus zone.

**Con_Radio072_isNotAwake_CantAddpower#/2** performs the same party-1072 lookup and returns **NOT Has(Level6)**; it admits **Buff_Radio072_CantAddPower_E_Trigger/Q_Trigger**, each source-cap1, HasDuration **1**. These trigger carriers do not serialize a further event payload or supply native Charge suppression by themselves. The separate **CantAddPower_E/Q** carriers (`#/2`) have HasDuration **2**, target-cap1 and grant **State.Property.ForbidOthersCharge / ForbidSelfCharge**, with NonSourceRemoveAll=true. Q additionally requires **Con_Radio072_isQDomainDuration#/2**, whose actual predicate is absence of **both UltraDelayDelete and UltraDelayDelete_Extra2** on SourceActor, despite its name. Those two domain markers request removal of CantAddPower_Q at QTE_BEGIN_ALL_PLAYER. This recovers blocker tags/gates and a removal event, not the native trigger producer, successful energy delivery, a universal two-second resource ICD or proof that all A6 Charge restrictions disappear.

All cited condition/effect exports are warning-free. Source projections: , `remaining-condition-callers.txt`, `remaining-condition-effects.txt`, `remaining-linked-conditions.txt`, `remaining-linked-condition-config.txt` and `remaining-buff-producer-callers.txt`; numeric rows are `DT_LinkoEffectFigure:LinkoLevel6_CritBase=0.25`, `LinkoLevel3_DamageUpGeneralBase=0.5`.

### LTE AoE guard and domain-owner predicates recovered, October 18

Warning-free **Con_1072_UltraAOE_SkillTrigger:CanApplyGameplayEffectContent#/2** uses SourceActor if it casts to player, otherwise its owner when that cast succeeds. It returns the OR of **UltraDelayDelete / UltraDelayDelete_Extra2 on that player** and the same two markers on **that player's main character**. This is a source-or-main-character domain gate, not a party-1072 lookup or a check of current target HP. Failed owner/cast paths do not supply another explicit validated player. It admits **Buff_Player_Radio072_TriggerSkillLTE_AOE#/3**; the already located FrontToLTE actor includes that trigger carrier.

Skill/Ultra LTE trigger carriers (`#/2`) both request **Buff_Player_Radio072_AoeDisplay** on BEGIN under **Con_1072_UltraAOEDisplay#/2**, which requires a TargetActor player **without Buff_PlayerRadio072_AOE_CD** on its successful cast. The trigger duration input is **0.3**; the separate guard (`#/2`) has duration **0.6**, target-cap1, NeverRefresh/NeverReset. All three declare **State.Common.IgnoreTimePause**. The separately inspected AoeDisplay (`#/2`) declares HasDuration **3** and IgnoreTimePause, with **no explicit event/damage/guard-add payload** in its saved Properties. This recovers specific admission/request/guard inputs, not a proved 0.6-second AoE attack cadence, final recipient list or universal pause behavior. Native display-to-guard/damage dispatch and ordering remain required; a Display-named carrier is not discarded merely by its name.

The two domain-duration predicates are **not full logical complements**: **Con_1072_Extra2#/2** requires the SourceActor player's **main character** to have **both LevelExtra2 and Buff_PlayerRadio072_UltralSkill**; **Extra2_Not#/2** tests only **NOT main-character LevelExtra2**. They gate **UltraDelayDelete_Extra2** and ordinary **UltraDelayDelete** respectively, with the positive condition also on LevelExtra2_1. Their copied Mint curve literals are unused. **Con_1072_UltralDomain#/2** instead checks SourceActor itself for absence of both domain-delete markers and gates the two corresponding SkillEffect event entries. These lookup differences must survive switching; do not merge them into a generic party field flag.

The separate **Con_1072_Level2/Level2_Not#/2** check SourceActor's own Level2 marker and gate the ordinary/A2 **UltraSkill3_Damage/_level2** classes. **Con_1072_Level5/Level5_Not** likewise check SourceActor, while **Level5_2** checks SourceActor's **main character** and the previously located **Con_1072_isLevel5** checks party member1072. The three reached A5 Skill_Player carrier variants bind these distinct conditions; they are not interchangeable aliases. Native effective source/main-character capture, modifier application, AoE recursion and resource settlement remain open. Sources: `linko-domain-condition-callers.txt`, `skia-and-linko-trigger-carriers.txt`, `linko-aoe-display-carrier.txt` and the already retained domain/Skill carriers.

### Domain cue requests and mark-break callback recovered, October 18

The repaired cue tags join the previously located carriers to their actual class functions. **GC_Linko_TriggerLTEAOE:OnWhileActiveEvent#/2** reads incoming EffectContext **EffectCauser** as player and looks up that player's party **1072**. If EffectCauser lacks **Buff_PlayerRadio072_AOE_CD**, it requests **Buff_PlayerRadio072_UltraLTE_AOE on party Linko**, then **AOE_CD on EffectCauser attributed to EffectCauser**, then **Buff_Radio072_Level4_1 on party Linko** if Linko has Level4. Both TriggerSkillLTE_AOE and TriggerUltraLTE_AOE (`#/2`) name its `GameplayCue.Display.Radio.TriggerLTEAOE` tag. This is an actual extra request route, not evidence of successful AoE damage, all-party guard coverage or a universal .6-second cadence.

The separate **GC_Linko_TriggerAOECD:OnWhileActiveEvent#/2**, joined from **Buff_PlayerRadio072_AOE_triggerCD#/2**, instead requests **AOE_CD on EffectCauser with party1072 supplied as instigator**. Preserve this context difference. **GC_Linko_AoeDisplay**'s reached graph supplies a global sound request and no direct damage payload. **GC_Linko_TriggerSubUltra:OnExecuteEvent#/1**, joined from **UltralSkill_PlayerTriggerSubSkill#/2**, requires MyTarget player and **IsLocalClientRole**, then calls **TryActivateAbilityByClass(GA_Radio072_SubUltraSkill)**; its explicit true return on that path does not depend on the native call's returned success. **GC_Linko_SkillDam:OnRemoveEvent#/2**, joined from **Skill_DamageUp#/2**, gets MyTarget player's **main character's LinkoLogicObj** and calls **ModifyLogicCustomChargeValue(0, "Clean")**. [Native charge-parameter metadata](../../shared/energy-and-gauges.md#custom-charge-mutation-parameter-boundary) names these inputs **ModifyChargeValue=0, GEClassName=Clean**, with no separate return. The string does not by itself establish an absolute-set/reset operation or a party-1072 recipient.

**GC_Xiaozhen_CanAddMark**, joined from **Buff_Xiaozhen_CanMarkAdd#/1**, caches MyTarget as ability character and subscribes to **AsyncTaskAbilityCharacterAttacked(self, Xiaozhen, true)**. The [native callback parameter metadata](../../shared/buffs-and-callbacks.md#native-task-parameter-declarations-recovered-october-18) identifies that final input as **bAutoEnd=true**; this factory has no bGlobal argument and must not be parsed as the four-input player factory. Its callback scans **GetAllActorsInRange** around Xiaozhen with configured vector **(20000,20000,20000)** and query byte8. The recovered control flow is unusual: an entry with **MarkChild OR Mark_Unawakened OR Mark** pops to the loop increment; failed validity/cast likewise skips an entry. A valid unmarked entry instead jumps **961 → 311**, resetting the loop counter to zero. Only loop exhaustion (**454 → 976**) requests **Buff_Xiaozhen_Mark_Trigger on the callback's DamagedCharacter**. An empty result therefore reaches the request, whereas a stable valid unmarked entry appears able to restart the scan indefinitely. Preserve this source anomaly rather than rewriting it into an intended “no existing marks” gate. It does not check positive damage, filter marks by source, or use the cached leader in this reached predicate. Native query/dispatch and current applicability remain open; no explicit task cleanup is present in this class.

**GC_Xiaozhen_MarkBreaktoLTE**, joined from **Buff_Xiaozhen_Mark_Unawakened#/2**, subscribes MyTarget's **CharacterOnDamaged** on OnWhile and removes that delegate on OnRemove. The callback resolves AttackCharacter from DamageCauser as player, else DamageCauser owner as player, else ability-character DamageCauser's leader as player; InstigatorCharacter is cached separately. It requires damage-GE asset tags **State.Damage.Skill OR State.Damage.Skill.LinkoBreakMark**, its **isCooldown=true** flag and Unawakened on the cached DamagedCharacter. It enumerates AttackCharacter's equipped players and requires the entry equal that same player's party1072. The admitted branch removes **one Unawakened** first, then resolves party1072 through the separately cached **InstigatorCharacter** player state and calls **LinkoSelfTriggerQTE** on its LinkoLogicObj. Only after a successful logic cast does it set isCooldown=false and schedule a nonlooping **1.0** timer to restore true. The CDO starts true. Failure after marker removal can therefore occur before that explicit cooldown request; this cannot be flattened into a guaranteed QTE or an unconditional one-second ICD. No callback checks positive DamageAmount. Context reuse, timer lifetime/clock and current summon-route applicability remain unresolved.

All these cue reads are warning-free under the accepted Halo repair. Exact carrier/tag/class edges and complete functions are , `cue-linko-damage-trigger-functions.txt` and `roster-cue-halo-extraction.json`. The earlier monster/summon versus current-description conflicts remain visible; finding a cue body alone does not settle native reachability.

### Remaining mark and team cue consumers recovered, October 18

**Buff_Xiaozhen_Display#/1 → GC_Xiaozhen_MarkNew** now supplies the missing Display-to-mark request. OnWhile **#/2 → graph#/1:949** first attempts incoming EffectContext EffectCauser as player and queries **GA_Radio072_Melee level** through that player's net ID to replace its Buffer Data; failed cast returns to the target-processing branch with the existing Buffer Data (CDO level1/stack0/strength1). After MyTarget's ability-character cast, an already **Unawakened** target receives a further **Unawakened request then GE_Xiaozhen_Skill1_Damage**, both attributed to EffectCauser using that Buffer Data. The requests are sequential and have no explicit Level3 test in this cue; the carriers' side-effecting admission conditions remain separate.

If MyTarget has no Unawakened, the graph sets canAdd=true, queries GetAllActorsInRange around **MyTarget** with vector(20000,20000,20000)/byte8, and scans all valid ability characters. Any existing **Unawakened**, without a source filter, sets canAdd=false; the loop still continues. Only if canAdd stays true does it request Unawakened on MyTarget attributed to EffectCauser. It does not request the awakened Mark/three-stack variant on that path. This is distinct from the previously recovered CanAddMark callback's anomalous loop restart. Display's duration 1.5/period 1/no-immediate fields do not establish repeated OnWhile calls or an extra periodic damage packet. Native current applicability, overlap/overflow re-entry, source attribution and successful settlement remain open.

**Mark_DeadSpreadTrigger#/1 → GC_Xiaozhen_MarkSpread** supplies the separate death-spread route. OnWhile queries a box vector(5000,5000,5000)/byte8 centered on **the cue actor's GetActorLocation**, using MyTarget as query context, then calls **FilterTargetsByBuffStack(result, Mark, false, 1)**. For each returned valid **MonsterCharacterBase**, it requires absence of **both Unawakened and Mark**, then requests **MarkChild** on that actor with cue SourceObject and default BufferData. It does not exclude existing MarkChild, explicitly pass the original Linko/EffectCauser as instigator, or impose another local count limit. The native filter's boolean/count meaning and effective source must be preserved as dependencies. This locates requested spread recipients/filters, not proof that every nearby target gets a current A3 mark or direct damage.

**CantAddPower_E_Trigger / Q_Trigger#/2 → GC_Linko_Level6 / Level6Q** now locate the actual blocker producers despite their Level6 names. Each OnWhile reads EffectCauser as player, gets that player's party **1072**, then requests the corresponding **CantAddPower_E / Q on party Linko attributed to Linko**. No explicit returned-party validity or role check is present. The trigger carriers' already recovered **NOT Level6** admission and Q carrier's separate domain predicate remain in force; a cue name alone is not an A6 requirement or proof of energy loss. Their native two-second blocker enforcement and event timing remain open.

**Skill_Player_level5_2#/1 → GC_Linko_Level5** instead checks EffectCauser's **GetCharacterElementType** result against exactly **0/2/4/5/6**; other values return. It looks up party1072 and requests **Skill_Player_level5_3 on Linko attributed to Linko**. The requested carrier (`#/2`) is OnlyServer, HasDuration 20 and adds **DamageUpGeneralBase=0.5** under the previously recovered QTE/Xiaozhen-without-Front predicate. Its omitted stacking policy/cap is not filled from the other A5 carriers. Preserve this numeric element selector and distinct source/recipient gate alongside the earlier description/ThrowXiaozhen conflict; it is not a generic test of every packet's Cycle compatibility.

**Passive3_1#/1 → GC_Linko_Pas3** reads EffectCauser as player, gets that player's **current attack target as MonsterCharacter**, then selects a resistance carrier by the player's element getter: **1→guang, 2→ling, 3→zhou, 4→an, 5→hun, 6→xiang**, each with prefix **Buff_Radio072_Passive3_**, applied to that attack target attributed to EffectCauser. No branch for 0 is reached. It does not use cue MyTarget as the damage recipient, select the element of the damaged target, or directly add a new hit. Native listener/lock validity, carrier admission and resistance settlement remain separate.

The separate **GC_Linko_Level3** has no saved carrier tag join and passes **GEClass=null** after its 0/2/4/5/6 element selector. Do not manufacture the missing class or use it as an admitted A3 route. **GC_Linko_SkillQTEHit** adds camera/light requests without a direct damage/mark call; its spawned LinkoLTE_Light lifetime 3 is not an attack recovery time. The Daffodil-specific cue consumer belongs to the [Daffodil owner](../daffodil/action-damage.md#linko-linked-extra-parry-marker-request-recovered-october-18). Exact sources: , `linko-team-mark-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`. All cited cue/carrier reads are warning-free; the earlier current-description versus saved summon-route applicability conflict remains unresolved.

### Shared Hexed display time-modification caller recovered, October 18

The shared supplement joins **buff_reaction_display2_new#/2 → GameplayCue.Display.Reaction.2.new_display → GC_CommonDisplay_Reaction2_new_display**. Its warning-free OnWhile **#/2 → graph#/1:590** requires **MyTarget.HasAuthority**, incoming EffectCauser as player, that player's party **1072**, and **Buff_Radio072_Passive1 on that Linko**. It evaluates CDO **DT_LinkoEffectFigure:Linko_Pas1_TimeAdd** at coordinate0 and requests **K2_ModifyActiveEffectStartTime** on MyTarget ASC for **buff_reaction_display2_new**, passing **curve output, null, -1, false**. The constant row is **0**. Result byte1's explicit fallback also calls the same method with 0; other unhandled results return. [Native parameter declarations](../../shared/action-timing.md#time-function-parameter-metadata) now identify **StartTimeDiff=0, SourceAbilityComponent=null, LimitFinalDuration=-1, bLimitToEffectDuration=false**. Native settlement remains necessary: zero StartTimeDiff is not proof of no mutation, zero buff duration or an absent request. The graph does not read Linko's separate 0.13 damage-ratio input.

CDO declares bSupportOnlyOwnGE=true, but the call itself supplies no explicit source filter; native cue ownership/actor context and refresh stacking remain separate. The target carrier (`#/2`) has HasDuration via **Reaction_LingZhou_BuffTime**, target cap1 and an **ExtraDamageCoefficientForTarget2** entry bound to the previously recovered **Cau_Reaction2_new**, requiring target **Ability.Player.Buff.Reaction.LingZhou**, bRequireOwnerAsInstigator=false and Con_Reaction2_new. This locates that calculator's serialized coefficient consumer, not the native copied-damage base, bonus/deduplication or duration settlement. Sources: `shared-transform-control-carriers.txt`, `shared-cue-extraction.json` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`char_radio_skill_skill1_1_Montage_New`) and Burst (`char_radio_skill_ultraskill_Montage`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Burst grant declarations & LTE admission:** `Buff_PlayerRadio072_UltralSkill_ChangeSkill` grants `GA_Radio072_UltraSkillChild` and `GA_Radio072_SubUltraSkill`. Resolve native grant/input replacement, LTE state/resources, effective level and removal timing.
- **Hexed extension ratio conflict:** 0.13 in `DT_ReactionExtendDataTable` vs 0.30 in published description.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.


Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The [Burst grant declarations](#configured-burst-ability-grants) are portable. Resolve native grant/replace timing, effective level, LTE admission/costs and the different active-cast/removal behavior of the child and sub-Ultimate abilities. Source-array order does not establish callback order.
- Resolve LTE state transitions, summon ownership/continuation, coordination eligibility and timing, and target caps/selection for reached packets.
- The [Hexed extension source owner](#recovered-hexed-extension-inputs) now binds the restored 0.13/zero/null inputs and separate additive bonus/predicate. Resolve coefficient composition against the published absolute 30% wording, native extension admission/recipient capture and conditional-modifier delivery. The source separates target status, effect tags and the source player's main-character DomainServer buff; do not replace that condition with a generic follow-up flag or silently multiply a second 10% zone.
- Ordinary-hit energy caller eligibility/cadence/recipient policy remains a tested hypothesis.

## Prior findings

- [`reviews/q10-game-action-binding/linko-action-binding-model.json`](../../../../reviews/q10-game-action-binding/linko-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/linko_actions.rs), [`linko-normal-chain.json`](../../../../reviews/q10-game-action-binding/linko-normal-chain.json), [`linko-skill-four-hits.json`](../../../../reviews/q10-game-action-binding/linko-skill-four-hits.json).

## Research history

- **Current question:** Current native Spectral Rush/LTE mark dispatch and legacy monster/summon-route applicability, plus Hexed coefficient conflict.
- **Useful source locations:** `LinkoLogicObj`, periodic/role-out carriers, `Buff_Xiaozhen_Display`, saved `GA_xiaozhen_Skill1`/Mark_Trigger chain and `TriggerCoolDownOrExtendDurationComp`.
- **Methods already tried:** Periodic 8/6/A1 and role-out routes, nearest selection, elemental RES and Sync expressions are located. Current A3 activation carrier names a -20 Skill-CD adjustment. The separate monster/summon chain has side-effecting mark predicates, A3-gated triple marks and a Melee follow-up check; current descriptions differ, and BP_xiaozhen inherited fields are incoherent. This is an applicability conflict, not a complete current mark model.
- **Next concrete step:** Display-to-Unawakened and death-spread requests, E/Q blocker producers, A5/Pas3 target routing and the Daffodil marker consumer are now located. Level3 remains an unbound null-class candidate. Resolve current saved-summon applicability, native LinkoLogicObj/task/context, side-effecting mark admission/overflow and delivered source/energy settlement. Continue Skia Shadow/Sneek and remaining shared cue consumers.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
