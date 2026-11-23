# Edgar action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1021`.

**Responsibility:** Edgar outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** the opt-in Rust `edgar.full_kit_v1` model binds the selected channel, secondary actions, healing/keys/field, HP-loss refund and recipient-owned Charge guard. [Full-kit acceptance](#executable-full-kit-acceptance) distinguishes native constants from checked clocks and integer HP settlement. Historical executable slices remain separately disclosed for existing research inputs. Additional source control lowering belongs to native workflow.

**Accuracy notes:** historical [execution gaps](#unknown-execution-policies) retain provenance; the selected sections now settle implementation decisions.

## Selected packet specification

This campaign specification supersedes the historical executable slice below. **Source:** original client bundle `bundle-v1/exports.zip` (canonical hash in its acquisition manifest), `GA_Edgar_Melee/Skill/UltraSkill/QTE`, their montage exports, `DT_SkillDamageData` and `DT_EdgarEffectFigure`; permanent owner inputs (optional `edgar.json`). **Mixed:** repeated Skill delivery follows the actual loop, rather than the old seven coincident children. Public corroboration: [Wild Current](https://neverness.app) describes a held channel, and its saved level-one multiplier is seven instances. Prior graph/description investigations below already exhaust the two approaches; remaining dispatch details use the checked rule here.

All coefficients multiply Edgar's **live ATK**, can crit normally, and deal Cosmos damage. Use the complete thirteen-level arrays in `DT_SkillDamageData.rows.GE_Player_Edgar_*`, indexed by effective talent level minus one; the samples below are levels **1 / 8 / 9**, not interpolated endpoints. Resource columns are energy / Cycle / Break **per positive applied packet**, once irrespective of dummy count.

| Row suffix (`GE_Player_Edgar_`) | Count / category | Coefficient 1 / 8 / 9 | Energy / Cycle / Break |
| --- | --- | --- | --- |
| `Melee1_Damage` | 1 / A | .309 / .530 / .572 | 1.931 / 3.215 / .390 |
| `Melee2_Damage` | 1 / A | .627 / 1.075 / 1.161 | 3.923 / 6.532 / .792 |
| `Melee3_Damage` | 2 distinct notifies / A | .390 / .668 / .722 each | 2.436 / 4.056 / .492 each |
| `Melee4_Damage` | 1 / A | .833 / 1.428 / 1.542 | 5.208 / 8.672 / 1.052 |
| `Melee5_Damage` | 1 / A | .698 / 1.196 / 1.292 | 4.366 / 7.270 / .882 |
| `AirAttack_Damage` | 1 landing / A | .312 / .535 / .577 | 2 / 3.3 / .4 |
| `PerfectEvadeAttack_Damage` | 1 / Z | 1.040 / 1.782 / 1.925 | 2.6 / 4.3 / 2.5 |
| `QTE_Damage` | 1 / H | 2 / 3.428 / 3.702 | 3.6 / 0 / 2.5 |
| `Skill_Damage` | 1 per emitted channel pulse / E | .800 / 1.371 / 1.481 | 3.2 / 5.329 / .643 |
| `UltraSkill_Damage` | 1 / Q | 4 / 6.855 / 7.404 | 0 / 0 / 1.5 |

Normal is **six packets**, level-eight total **5.565 ATK / 20.300 energy / 33.801 Cycle / 4.100 Break**. A full held Skill emits seven separate packets: **9.597 ATK / 22.4 energy / 37.303 Cycle / 4.501 Break**; a tap emits one. There are no six automatic children attached to a tap. `Skill.1` targets the current ability lock (one enemy); `Skill.2` independently requests one allied heal. Repeating the loop repeats those requests; it is not seven hits on every dummy. Consistency: .8 × 7 = 5.6 at level one, matching the saved multiplier; the native 37 Cycle display rounds 7 × 5.329. This resolves the historical coincident/AoE hypothesis without removing channel damage.

**Inferred plunge rule:** scenario supplies normalized fall bonus `h` in [0,1], damage coefficient multiplied by `1+h`, resources unchanged. This retains the published maximum +100% without inventing a height-to-metres curve outside the stationary-dummy scope. Ground-entry plunge is `h=0`; a declared maximum fall is `h=1`.

Healing is separate from outgoing damage: each emitted Skill heal uses `Skill_Treat_MaxHPPer(level) * Edgar live MaxHP + Skill_Treat_Value(level)`; Burst field ticks use the analogous UltraSkill rows. Level 1 / 8 / 9 Skill pairs are **(.014,300) / (.0239,514) / (.0259,555)**, Burst **(.011,350) / (.0188,600) / (.0203,648)**. Apply source/recipient healing modifiers, A3 where eligible, then clamp to missing HP; expose both attempted and effective healing and overheal for equipment listeners. Healing itself grants no packet energy/Cycle/Break. No hidden damaging summon or extra Burst hit is implied by the field.

## Selected timing specification

**Mixed timing model:** use native montage seconds at play rate one, advancing on the single real combat clock. A positive connected hit adds owner action-clock loss `AttackDilationTime * (1-rate)` once, not once per recipient. Delayed pause onset is retained (Melee3 .04, Melee4/5 .033); hit damage lands at its event, not after the delay. No universal time stop or world-clock pause is added. Edgar's missing explicit Skill/Burst hitpause fields contribute zero extra action delay; this is a declared timing inference, not a measured video claim.

| Action | Montage-local hits | Earliest ordinary next ability / jump / move |
| --- | --- | --- |
| Normal 1 | .18333334 | 1.0962062 / .20072556 / 1.0962062 |
| Normal 2 | .28333333 | .9503515 / .31332883 / .83298695 |
| Normal 3 | .20035695, .43373406 | 1.1663235 / .22513139 / 1.3361104 |
| Normal 4 | .56750554 | 1.2764947 / .59162724 / 1.1254174 |
| Normal 5 | .7344072 | 1.3352499 / .7664797 / 1.7060256 |
| Appear Normal 1 | .28328854 | 1.1089821 / .315733 / 1.1089821 |
| Plunge after landing | .049876 | .198548 / .082917 / .581056 |
| Riposte attack section | .5177309 | .90188086 / 0 / .90188086 |
| QTE | .7168937 | 1.3936534 for all three; switch lock ends 1.3361492 |
| Tap Skill | .8660991 damage, .86711484 heal | 1.3501 / .9286244 / 1.6328098 |
| Burst | 2.7509558 damage + field spawn | 3.3981643 all three; switch lock 3.3264744 |

Native per-hit action losses are .0297 for Melee1/2/4/5, **.0603 for each** Melee3, .297 for Riposte, .210 for QTE; plunge/Skill/Burst have no added owner loss in this profile. Thus complete Normal recovery is **6.0640258s**, not the historical two-second input-gap convention. Normal inputs may queue at each stage's next-ability gate; different actions, field exit or explicit move/jump cancel clear the chain. After recovery, a later fresh Normal starts stage one unless the next-stage input was queued; same-owner switch does not reset. All unissued hits of an interrupted action are cancelled; already emitted projectiles/heals resolve. Ordinary switching uses the next-ability gate, subject to explicit switch locks. Earlier jump cancels forfeit later hits and spend the scenario's normal jump cost; they do not secretly complete the chain.

**Held Skill:** source `GA_Edgar_Skill` hold >=.2 and open montage interval (.8,.84) selects `Loop`, whose montage origin is **6.0333333** and length **.6**. Choose transition at .8+epsilon (reported times below use .8); avoid charging the unused end-section coordinates as an idle wait. Loop damage offset **.0219797**, heal **.0217107**. Hold release is honored after at least .3s total input hold; force release at **4.9s total action time**. Seven maximum-hold damage times are **.8219797 + .6*k**, k=0..6, last4.4219797; heals are **.8217107 + .6*k**. No eighth tick at5.0219797. The small heal-before-damage offset is real source ordering. Releasing before .8 takes the one-pulse tap path. Release/interruption inside Loop cancels unissued pulses and starts separate `Edgar_Skill_End`; ordinary next input is release + **.43089145**, jump immediately. Source .6s loop plus `AnimTimeConvertToCDRatio=1.676` gives **1.0056s additional CD per complete loop**, independently explaining the displayed 1s rather than inventing a tick interval.

Tap base CD **16s** starts at .84958535. Held channel marks deferred cooldown on Loop entry; at release, start full `16 + 1.676 * actual_loop_elapsed` at End notify release+.09674061. A2 clamps only the additional term to [0,3]. Maximum hold has loop elapsed4.1: additional **6.8716**, CD22.8716, ready at action start **27.86834061**; A2 ready at **4.9+.09674061+19 = 23.99674061**. Interrupted before any initial/Loop cooldown marker uses **3s from interruption**; interruption after Loop entry settles accrued extra and full base immediately from interruption, without adding an unplayed End notify. These clock choices are inferred from deferred/cumulate/end markers and the recovered A2 clamp. They preserve longer hold => longer CD and prevent repeatedly cancelling a completed channel for the short penalty. Burst cost/CD begin on commitment: **100 / 20s**; later cancellation does not refund them. Field spawn is its actual 2.7509558 event.

## Selected resources and admission

Source `.edgar.resources.keys` reads the whole0..3 native pool, initially0. `.mods.finnegans_wake` and `.finnegans_wake.active` read the actual live field; `.out_of_combat.active` reads the owner combat flag. The selected full-kit model is included by native build projection before initialization, including parties where Edgar never acts but observes another owner's Support. Two-cycle source/native replay tests check Skill's one key, clearing at Burst field spawn, an11s field after the2.750956s spawn marker, exact expiry and continued natural energy. A separate idle A1 observer test checks a teammate's actual Support. These bindings do not add nonzero key starts or certify remaining held/action choices (Q4-L4/L3).

**Mixed:** energy starts0, cap100; Cycle starts0, cap100. Use [shared energy, exact fractional carry and switch admission](../../shared/energy-and-gauges.md) for packet awards, off-field .6 sharing, Charge and ordinary versus qualified switches. Skill/Normal/plunge/riposte/QTE have no additional Ultimate-energy cost; Skill requires its completed cooldown and an on-field living Edgar. Burst requires100 before commitment, subtracts100 atomically; do not count a future R6 refund toward admission. Failed inputs are diagnostics, never implicit waits. Ground Normal requires grounded state, plunge declared airborne/landing time; critical riposte requires an explicit successful Critical Dodge token and consumes it once. Dodge uses the configured stamina profile and native strength-cost row, not energy; no stamina is consumed merely for channel duration. Native evade cost binding is inherited through `AbilityStrengthCost`, with the shared default DodgeCost lookup used by the other owners; explicit scenario stamina must be present before invoking dodge/jump branches.

Key pool starts0, integer cap3, infinite duration, survives field exit; source `Buff_Edgar_TreatKey` explicitly removes on death. B4 is required. First `Skill.2` of an actual tap/channel adds one through **6s nonrefreshing `Buff_Edgar_TreatKey_KongKe`**, so all seven loop heals still grant only one key. This trigger does not require an injured ally or a positive heal. Own successful QTE also grants one; A1 grants one for each other living teammate's actual Support cast, including while Edgar is off field, without double-observing own QTE. Burst **does not consume at commitment**: at field spawn snapshot `n`, set duration10+n, then clear all current keys, per recovered authority ordering. Keys acquired later belong to the next Burst; no retroactive duration extension. Without B4, duration10 and no ordinary key generation. Existing historical key-persistence code must be changed during implementation.

Consistency: Skill once -> one key -> next field11s -> zero keys. Two supports plus one Skill ->13s, not10s after premature clearing. A cancelled Burst before spawn spends energy/CD but retains keys because its removal caller was never reached. A channel cancelled before its first heal/key event grants no key. Pool cap/death cleanup are separate from the 6s Skill wrapper gate.

## Selected effects and healing

**Source + inference:** each Skill heal event rescans equipped living characters (including Edgar/off-field members), excludes HP=0 and abs(HP-MaxHP)<=1e-6, selects lowest **absolute HP**, stable party-index tie. It does not use HP percentage. Example1000/10000 versus500/1000 selects500; after healing the second to full, the next pulse may select the first. No candidate means no heal execution/callback, while Skill key and enemy damage requests remain independent. Background heal differs only by hidden damage float. A3 checks selected recipient's HP/MaxHP **<=.30 before healing**, then multiplies entire base heal by1.15; .30 exactly qualifies. At sourceHP10000 and skill1,440 becomes506. Clamp only after modifiers; overheal cannot create HP or count as R6 damage.

Field: source `BP_EdgarUltraSkillField` spawns on Burst hit, snapshots **duration only**, then clears keys. One field per Edgar; replace an existing field without a terminal extra heal. Its period is **1s**, first tick spawn+1 through spawn+duration inclusive (terminal tick before lifetime cleanup), so zero/three keys yield10/13 heal requests. This endpoint convention is inferred from a whole-number duration/period, preserving ten ticks for the published ten-second field; no extra initial heal is manufactured. At each tick heal the current active living character in declared field range, using Edgar's then-live HP/talent/heal modifiers; no lowest-HP selection for Burst. Field persists on ordinary switches and heals the new active actor; it uses real time at owner gameplay scale1, never pauses because Edgar is off field. Owner death, combat end or replacement removes future ticks and A6 field DEF; recipient death skips that tick, no resurrection or catch-up. All eligible party members are in field range in the default dummy scenario; explicit out-of-field condition disables only that actor's field heal/DEF eligibility.

Damage remains independent of healing success. Emit attempted/effective/overheal receipts for each delivered heal so a healing-triggered weapon can distinguish them. No universal damage proc is granted by Edgar: optional equipment must supply its own trigger and arithmetic. Skill target-link effect is removed on Skill end, field duration is not a persistent attack buff, keys do not multiply damage, and no heal grants energy without an explicit passive/equipment rule.

## Selected passives and awakenings

**Mixed:** selected identities, not awakening count, enable A1..A6; count alone enables R3/R6. Source table1021 and helper functions below determine magnitudes. Native carrier inspection adds concrete `TreatKey_KongKe`6s and recipient Charge guard rules; remaining consumers have already had native/public approaches and use these finite checked inferences.

| Selection | Complete modeled rule |
| --- | --- |
| B2 Gentle Edge | On a qualified Support switch that actually creates **Charge**, replace the ordinary10-energy Charge grant with **120 raw energy to its triggering/incoming character**, apply normal recipient CE/cap/carry once (not120+10; not per dummy or pistil). Guard on that recipient for30s; source `Calc_Edgar_Passive1` checks OwnerActor ASC, native Charge carrier applies `GE_Reaction_GuangLingXiang_ChargeCD` after execution. During that recipient's guard, Slowed-target Vita Pistils grant it no energy; other packet energy is unaffected. `LimitCharge_1021` shares the same duration row/tag and clears on leaving battle. Select recipient-owned suppression/guard rather than a party-global lock because the calculator checks recipient ASC and carriers aggregate by target. Unaffected recipients can trigger their own grant. While guarded a new Charge grants no ordinary10 fallback; otherwise suppression would defeat the described tradeoff. At30s expiry ordinary behavior/next120 admission returns. Edgar must be living with B2; he may be off field. |
| B4 Unchanging Warmth | Key generation/cap/capture/removal as above; no ordinary attack or heal-count key. |
| A1 | Other teammates' actual Support casts add one key once, cap3, living/off-field Edgar eligible; no own-QTE duplicate. |
| A2 | Skill additional CD capped3, base16 retained. |
| A3 | Skill healing multiplier1.15 at recipient preheal ratio<=.30, no Burst bonus. |
| A4 | Skill-category damage bonus+.50, additive with ordinary damage bonuses for every pulse; no normal/QTE/Burst bonus. |
| A5 | Static HP%+.20, summed with gear/other HP% once, not multiply final HP by1.2. |
| A6 | In-field living allies including Edgar receive+.15 DEF% of their own base DEF while field exists. Remove immediately when out of range/field ends; no stacking from overlapping fields of the same owner. |
| R3 | Skill/Burst/QTE talent+1; ordinary Normal unchanged. Clamp only at native13 cap. |
| R6 | Accumulate actual positive HP loss of each equipped teammate, **including Edgar**, as `damage_applied / that_actor_MaxHP_at_loss / .03`, cap20 exact pending units. No shield-only damage, overkill, dead actor loss, overheal or healing reversal; self HP costs count if actually paid. Fractional losses accumulate across actors/events. At each admitted Burst: pay100 first, capture floor(pending), reset pending0, grant that much raw self energy through ChargeExecution/CE; applies once, not every second despite the0.1-duration/1-period GE serialization. Future damage then funds next cast. Source helper floors pending; this chooses the description's direction and reset transaction, rather than multiplying by .03. |

B2 countercheck: CE1/cap100 from0 receives100 with20 overflow, not130; while its guard is active another Slowed-target pistil gives0, a normal melee hit still gives its row award. R6 countercheck: two1.5%-MaxHP losses yield one pending Charge; sixty-percent loss caps20; cast from100 leaves20 at CE1, then pending0. This accumulator cannot autonomously fund its own admission. R6/A6 are different effects despite both native names containing Level6. City/life traits have no combat trigger and add no DPS effect.

## Selected alternate actions and field

Public source `dash[back=true]`/`dash[back=false]` now binds native Back/Front
choice; omission remains Front. `critical=true` (or exact1) separately declares
assumed Critical Dodge success, and the following ordinary `attack` selects
the existing native Riposte. [Rust source checks](../../../../crates/content/src/simulation/tests/source_directions.rs)
change the direction between two continuing paid cycles, retain70 of an
explicit100-stamina account after two15-cost dodges with zero regeneration,
produce two native Riposte packets, and reconcile saved source and native replay.
The choice uses native gates and costs; it introduces no caller timing or
geometry. Remaining owner choice/cancellation integration stays in Q4-L3.

No damaging transformation/recast/summon or enhanced Normal is configured; held Skill and persistent healing field are the two alternate-state mechanisms and remain fully active at A0. Native `Loop` is a hold continuation, not a separate free Skill input or stock. Tap/held share one CD. Release ends channel and removes target link; field exit interrupts channel and all unissued heal/damage requests but keeps already spawned Burst field and key pool. No off-field channel continuation is inferred from the ordinary background-heal class (it describes its **recipient**, not Edgar's casting state).

Critical Dodge front/back evasion portions can transition into `PerfectEvadeAtk` on authored success. Front has native next-Skill gate .32818943, generic1.0049118, jump.082544; back Skill.42820832/generic1.0008111/jump0. The explicit riposte input enters attack section0, emits the one Z packet at.5177309, then adds.297 action loss and recovers1.19888086. Successful riposte may queue Normal5 via source section jump, otherwise fresh Normal1 after recovery. No automatic critical-hit guarantee or second Z hit is granted by Critical Dodge. Airborne plunge waits the authored real landing time then uses landing schedule; an ordinary switch changes field control without starting an attack. Target movement/navigation is outside scope. Scenario-supplied Critical Dodge success does not synthesize enemy HP loss, R6 Charge or Break events.

## Selected targets and shared dependencies

**Mixed:** ordinary melee, plunge, riposte, QTE and Burst use their source damage-range classes, all eligible living dummies in range, full coefficient to each, stable ascending target index, no splitting or invented one-target AoE cap. Skill instead binds `TargetType_AbilityTarget` and has **one current locked enemy per pulse**; do not inherit the historical AoE assumption. Target death retargets the lowest living eligible index at the next pulse, never redirects an already delivered packet; no eligible enemy skips its damage but does not cancel the independent ally-heal/key event. Source/recipient HP and stats are live per event. Healing uses its distinct selectors above and never damages an enemy because a synthetic hit-result structure was returned.

Use [damage/stat zones](../../shared/damage-and-stat-zones.md), [energy and gauges](../../shared/energy-and-gauges.md), [cycles/Break](../../shared/cycles-and-break.md), and [equipment/build legality](../../shared/equipment-builds.md). Cosmos pairs with Anima for Blossom and Lakshana for Remora; Charge additionally needs an existing Vita Pistil + Remora target and qualified Support. Apply Edgar B2's replacement before resolving the ordinary pistil grant, then recipient suppression; a Chiz-enhanced14 ordinary grant is likewise replaced, never added to120. Other reaction owners keep their own damage and generation lineage. No target bar means no Break even though packet UnbalValue exists. Edgar's own Break uses normal shared level/BI/original-cap rules; no extra damage multiplier is inferred from his healing role. R6 consumes applied player HP-loss receipts after HP clamp, not enemy damage or shields. Equal-time expiry/recovery precedes hits; damage then resources/Break then procs, with declared final field tick before its same-time field removal. A heal's equipment callback uses that heal's actual recipient and source, with causal lineage to prevent recursion.

**Common dodge choice (inferred, no owner override):** use the same15-stamina, .9s admission cooldown profile as the other ordinary evades, scaled by configured stamina-use efficiency and the shared two-second recovery gate. `GA_Edgar_Evade` binds the generic strength-cost class with no local scalar, so this is a common-profile inference rather than a recovered15 literal in that subobject. Its non-damage timing uses the front/back gates above. An ordinary jump's cost is an explicit account/action scenario value since no Edgar-specific cost is described; reject a missing profile only when that branch is authored. Two consecutive dodge admissions less than.9s apart fail even with full stamina. Critical success consumes the actual dodge's time/cost rather than awarding a free second action.

## Selected implementation handoff

**Mixed, implementation-ready chosen model:** implement the preceding seven sections together. Content owns per-event packet rows, channel transition and end clock, heal selectors/modifiers, keys +6s gate, B2 recipient guard/suppression, field ticks, A/R selection and R6 HP-loss accumulator. Shared services own stat zones, caps/carries, causal ordering, reaction/Break and seeded crit. Keep ordinary cooldowns, fractional balances, field generations, gear ICD and pending events across authored cycles. Historical coincident Skill, unconditional B4-free duration extension and nonconsuming keys must not be reused. No further native-body recovery is required for this selected model; inference flags belong in CLI/browser results.

**Legal A0 recipe:** Edgar `character:1021`, level80/breakthrough6, purchased Normal/Skill/Burst8 (QTE8), no awakenings; Mind Royale `weapon:fork_lingganzhongjiezhe`80/6/**T1**; Speedy Hedgehog `set:Suit11`. Set count is 4; console contributions use `add stats`, with no module geometry or specialization inference. Source compiler gives baseATK**976=421+555**, AtkUp.75, flat420 => staticATK**2128**; HP**28506.8 ->28507**, DEF824, CR.35, CD1.10, Cosmos bonus0, CE**1.12**, BI60, CI0. Round static HP nearest/ties-up once; preserve exact live additions.

Keep [Mind Royale's checked cast model](../../weapons/mind-royale/passive.md): T1 Skill cast grants **10 raw self energy through CE**,20s ICD, once at commitment (not per pulse). The item-owned contract and independent CE/carry checks supersede the former owner forecast's unscaled10 assumption; CE1.12 yields11.2. Keep [Speedy Hedgehog](../../sets/speedy-hedgehog/passive.md): 2pc+.12 CE already compiled; Burst commitment grants team+.15 ATK% of each baseATK for20s, cap1/refresh, persists field exit/source death per selected carrier model. Edgar therefore uses liveATK**2274.4=2128+.15*976** while active, not2128*1.15. This does not add a second compiled2pc or round live146.4 away. The static compiler's historical `incomplete` status omits these dynamic rules; it does not invalidate the legal recipe or claim implemented full-kit readiness.

**Two continuing75s cycles:** solo active Edgar, initialHP**14000/28507**, energy0/cap100, Cycle0/cap100, carries0, keys0, no cooldowns/buffs, target80/HP24million/DEF1080/CosmosRES.20, explicit **no Break bar**, seed42. No incoming damage; no dodge/jump/stamina-dependent action. B2/B4 remain enabled. Solo Cosmos creates no qualified reaction, rather than disabling Gentle Edge. Expected-damage arithmetic below uses the crit expectation; eventual sampled execution must retain seed/RNG across both periods.

| Relative time, repeat with+75 | Action/state |
| --- | --- |
| 0,6.0640258,12.1280516,18.1920774,24.2561032,30.3201290 | Six complete Normal chains, last recovers36.3841548. Explicit wait to37. |
| 37 | Hold Skill to41.9; Mind Royale grants10 raw once,11.2 through CE. Seven damage pulses37.8219797..41.4219797 and seven independent heals37.8217107..41.4217107. First heal request grants one key;6s wrapper blocks the rest. End recovers42.33089145; cooldown completes64.86834061. |
| 44 | Burst spends100, set ATK buff through64, CD through64. |
| 46.7509558 | One Burst hit and field spawn: capture1key =>11s, clearkeys0. Burst recovery47.3981643. |
| 47.7509558..57.7509558 | Eleven field heal requests,1s apart, terminal then field removal. First cycle's sixth tick reaches fullHP; remaining five are overheal. |
| 58 | Seventh Normal chain, all hits before64 use the ATK buff; recovery64.0640258. Explicit wait75, carrying resources; Skill CD has also completed. |

Each period emits **50 damaging packets**:42 Normal+7Skill+1Burst. First channel heal base is **1195.3173** per request; shared integer-HP settlement rounds each request to1195, leavingHP**22365**. Field heal base **1135.9316** rounds to1136, with six positive ticks then fullHP. In period2 the Skill selector finds no injured member and emits no heal callback, but its independent key request still succeeds; the field still attempts11 heals with effective0. Both fields expire before their period boundaries. Keys0, cooldowns ready, no pending hits/field or gear buffs at75/150; no state reset occurs.

Per period natural raw packet energy is164.5, CE-adjusted**184.24**, plus11.2 gear =>**195.44** received. Ends: energy**22 +.440 /22 +.880**, cumulative overflow**73 /168**, total `390.88 =200 spent+168 overflow+22.88 retained`. Cycle**273.910** each period ends**100+.910 /100+.820**, cumulative overflow**173 /447**. Fractional carry survives capping/spending. Skill CD remains governed by the channel model even though the selected75s cadence never presses it early.

Independent Fraction/event audit:  / `edgar-two-cycle-receipt.json` checks100 packets/150s, admissions, exact carry/overflow, hold CD and HP clamp. No engine/game claim: coefficient sum before ATK buff each period42.987, during buff12.420; expected damage `2*(42.987*2128+12.420*2274.4)*(1+.35*1.10)*.5*.8 =` **132654.617472**, expected DPS**884.36411648**. The denominator includes channel/end, Burst recovery and all waits. Counterexamples for R6, A3, multiple targets and suppressed Charge are specified above; those conditional handlers must still be implemented even though this A0 solo fixture does not trigger them.

## Executable full-kit acceptance

`edgar.full_kit_v1` selects the complete Rust owner clock and event handlers. A tap has one pulse; held Skill has independent, finite single-target damage and lowest-absolute-HP healing events. Dead locked targets retarget only on a later pulse; a missing enemy does not suppress the independent key/heal request. Early interruption settles the three-second penalty or actual elapsed channel cooldown; A2 caps only the additional term. Full-HP requests retain their B4 key gate without manufacturing a heal callback. Actual native Support admission grants own/A1 teammate keys once, including off-field Edgar.

Burst pays100 before R6's floor/reset/CE refund. Its impact starts the field and consumes captured keys; cancellation before impact retains keys and creates no field. Fields use live source/recipient healing modifiers, follow the active living recipient and retain an inclusive last tick, independent of source field position. Defeat, combat leave and replacement invalidate pending field generations. The stationary scenario assumes the equipped party is in field range; no coordinates or automatic navigation are inferred. A6's base-scaled DEF grant starts at field spawn and follows the same real deadline and source life. Normal, explicit plunge, paid Dodge/Critical Riposte, queued Normal5 after a connected Riposte, ordinary appearance and qualified Support retain their own clocks and recipients.

The shared HP observer supplies loss-time MaxHP to R6's exact capped ledger. Applied self costs count; shield absorption, overkill and healing do not. Fractional pending units, reset spending and overflow are reported in `fractional_resources["edgar.pending_charge"]`; integer keys use `resources["edgar.keys_of_truth"]`. B2 replaces the ordinary/Chiz-enhanced Charge by120 raw energy and applies a recipient-owned half-open30s suppression guard. Different recipients retain separate eligibility, including off-field Edgar as granter; combat leave clears the guard.

Independent Rust checks cover source packet/resource sums, delayed hitpause, microsecond cancel/CD boundaries, heals before channel damage, release between heal/damage, lowest absolute HP, A3's inclusive30% boundary, target loss, off-field field healing, source defeat, R6 fractional settlement and paid admission, real Support keys, plunge scaling, stamina/Riposte and B2 replacement/suppression/expiry. Native source-built A0 Mind Royale T1 + Speedy Hedgehog executes two continuing75s periods with100 packets and expected damage132654.617472, exact live ATK windows, HP28507, keys0, energy22+.88/overflow168, Cycle100+.82/overflow447, aggregation and seeded replay. Individual source timestamps round to the engine's microseconds; the five-stage connected recovery is6.064027s versus the unrounded analytical6.0640258s. The cached arithmetic forecast predates the item-owned CE correction; native expectations above retain that correction and shared integer-HP rounding. These are checked model results, not observed game timing. Additional typed source hold/jump/critical/combat-transition lowering remains native workflow; final owner interactions remain Q3/C-Interactions.

## Source facts and executable model

The following executable slices describe retained historical research policies. They do not override `edgar.full_kit_v1`.
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Each admitted Normal input advances one of five stages, looping after the fifth. Stage three delivers two native Melee3 packets; the remaining stages deliver one. Native talent projection and live delivery modifiers remain shared. The admitted input ledger retains the primary input binding; hit receipts identify the actual stage packet.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact 2-second boundary continues. This is a research continuity convention, not measured animation timing. The third-stage projectiles deliver in order at the input's research primary impact time. Native animation/travel intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`edgar-normal-chain.json`](../../../../reviews/q10-game-action-binding/edgar-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`edgar.skill_animation_profile_v1`, `edgar.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Edgar_Skill`), `AnimNotify_TriggerCooldown` is at 0.850s, gameplay events land at 0.866s–0.867s (`Skill.1`, `Skill.2`, `Skill.3`); cancel windows activate at 0.929s (`BreakSkill_Jump`), 1.350s (`BreakSkill_AnySkill`), and 1.633s (`BreakSkill_Move`) with total duration 6.633s (end animation `Edgar_Skill_End` activates `BreakSkill_Jump` at 0.0s, `TriggerCooldown` at 0.097s, and `BreakSkill_Move`/`BreakSkill_AnySkill` at 0.431s). For Burst (`Edgar_UltraSkill`), gameplay event `UltraSkill.1` lands at 2.751s; all cancel windows activate at 3.398s (`BreakSkill_AnySkill`, `BreakSkill_Jump`, `BreakSkill_Skill`, `BreakSkill_Move`) with total duration 6.283s.
- **Multi-target delivery (`edgar.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive the full packet train for Skill (7 hits of `GE_Player_Edgar_Skill_Damage`) and Burst (1 hit of `GE_Player_Edgar_UltraSkill_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Action packets and owner effects

- GA_Edgar_Melee public primary packet is GE_Player_Edgar_Melee1_Damage. Native coefficients at Normal levels 1/8 are 0.309/0.530; category A/COSMOS metadata stays source-bound. Edgar's resonance_3 does not include Melee, so level remains unboosted at A3.
- GA_Edgar_Skill delivers 7 instances of GE_Player_Edgar_Skill_Damage (1 primary + 6 children) under coincident timing hypothesis edgar.skill_burst_packets_v1. Native coefficients at Skill levels 1/8/9 are 0.800/1.371/1.481; category E/COSMOS metadata stays source-bound. A successful admitted Skill adds 1 Key of Truth at every awakening (up to 3); at A4, Skill DMG receives a +50% damage bonus across all hits.
- GA_Edgar_UltraSkill public primary packet is GE_Player_Edgar_UltraSkill_Damage. Native coefficients at Burst levels 1/8/9 are 4.000/6.855/7.404; category Q/COSMOS metadata stays source-bound. Each admitted Ultimate samples current keys and enters a 10s + 1s per key Finnegan's Wake at every awakening; no key damage bonus is supported by the frozen native figure.
- Resonance 3 activates at awakening >= 3 and adds +1 level to QTE, Skill, and UltraSkill without manual overrides.
- At A1+, an admitted QTE by another living teammate adds one key to living Edgar, including off-field Edgar. Edgar is excluded from team QTE observation so own QTE grants once. Keys cap at three and persist through Burst and field switches under the reviewed no-consumption hypothesis. A4 Skill bonus (+50%) remains modeled; A6 allied DEF is implemented under edgar.a6_wake_team_def_v1 in reviews/q2-edgar-periodic-healing.json. The native 0.03 extra-charge ratio and 20.0 limit do not establish an additional Burst damage scalar or charge settlement.
- Canonical arbitrary awakening selections: Effect1 teammate QTE keys, Effect4 Skill damage, Effect5 static Max HP and Effect6 Burst team DEF use selected identities. Native builds, Studio and runtime MaxHP provenance consume the same Effect5 selection; all HP percentages still settle together once. Resonance skill levels use count. Existing unsupported owner mechanics remain pending.
- An integrated runner scenario verifies an A1 allied QTE grants one Key of Truth, an independently selected A4 Skill delivers all seven packets at the exact +50% bonus, and Burst samples both keys for its bounded Wake extension. This is the coincident-packet runner path; continuous Skill channeling, HP-loss charge settlement and lowest-HP healing selection remain outside the scenario.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/edgar-action-binding-model.json`](../../../../reviews/q10-game-action-binding/edgar-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/edgar_actions.rs), [`edgar-normal-chain.json`](../../../../reviews/q10-game-action-binding/edgar-normal-chain.json), [`edgar-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/edgar-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_edgar.json`, `data/extracted/neverness_edgar.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/edgar.rs) (original `DT_SkillDamageData` export is cache-only), `1021AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- AirAttack and continuous channeling behavior remain unimplemented.
- Lowest-HP teammate healing and any damage-relevant HP/overheal interactions need a live recipient policy.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/edgar). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Edgar (`GA_Edgar_*`):

| Action / damage row (`GE_Player_Edgar_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.309–0.833 / 0.530–1.428 / 0.572–1.542 | 1.931–5.208 / 3.215–8.672 / 0.390–1.052 per hit |
| Plunge `AirAttack_Damage` | A | 0.312 / 0.535 / 0.577 | 2.000 / 3.300 / 0.400 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.040 / 1.782 / 1.925 | 2.600 / 4.300 / 2.500 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 3.600 / 0.000 / 2.500 |
| Skill `Skill_Damage` | E | 0.800 / 1.371 / 1.481 | 3.200 / 5.329 / 0.643 per hit |
| Burst `UltraSkill_Damage` | Q | 4.000 / 6.855 / 7.404 | 0.000 / 0.000 / 1.500 |

Critical Riposte uses GAName `GA_Edgar_PerfectEvade`, while Support QTE uses `GA_Edgar_QTE`. All damage rows deal COSMOS damage. Skill delivers 7 instances of `Skill_Damage` (1 primary + 6 children) under coincident timing hypothesis `edgar.skill_burst_packets_v1`. Burst delivers 1 instance of `UltraSkill_Damage` and activates Finnegan's Wake (10s + 1s per Key of Truth).

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Edgar_Passive_1` binds `Buff_Edgar021_Passive1`. `DT_ReactionExtendDataTable` row 1021 binds `ReactionAbilityID=GA_Edgar_Passive_1` with `GE_Reaction_GuangLingXiang_Charge_1021` and `GE_Reaction_GuangLingXiang_LimitCharge_1021`, providing Cosmos reaction (GuangLingXiang / Reaction 7) charge enhancement.
- **Passive 2 (Breakthrough 4):** `GA_Edgar_Passive_2` binds `Buff_Edgar021_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1021AwakenEffect#/rows/1021/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Grants Edgar 1 Key of Truth whenever any teammate casts a Support Skill.
- **A2:** Caps the max cooldown extension from Wild Current at 3s.
- **A3:** Increases Wild Current's healing effectiveness for allies with current HP below 30% by 15%.
- **A4:** Wild Current (Skill) DMG +50%.
- **A5:** Edgar's Max HP +20% (`MaxHP + 0.20`).
- **A6:** Grants 15% DEF to all allies in the range of Finnegan's Wake.
- **Resonance 3:** Increases the skill level of Wild Current (Skill), Finnegan's Wake (Burst), and Weight of Knowledge (QTE) by +1.
- **Resonance 6:** Converts 3% of any teammate's Max HP lost into 1 Charge for Finnegan's Vigil, up to 20 Charges. Resets after casting Finnegan's Vigil.

### Healing magnitude consumers recovered, October 18

The initial bundle's `Edgar_Skill_Treat:CustomCalculateBaseMagnitude` (`#/2`, statements 79..1917) reads the **instigator's** `GA_Edgar_Skill` level and MaxHP. On successful curve reads its base result is `GA_Skill_Treat_MaxHPPer(level) * instigator MaxHP + GA_Skill_Treat_Value(level)`. Its A3 branch additionally requires `Buff_Edgar021_Level3` on that instigator and **HitActor current HP / HitActor MaxHP <= LowHealthPercent**, then multiplies the entire base result by `1 + TreatUpLv3`. `/1/Properties` binds those scalables to `Buff_Edgar_Star3_LowHPPercent=0.3` and `Buff_Edgar_Star3_TreatUP=0.15` in the saved figure table. This locates the inclusive threshold and distinguishes the healing basis from the recipient tested for low HP.

`Edgar_UltraSkill_Treat#/2` similarly uses the instigator's UltraSkill level and MaxHP, multiplying `GA_UltraSkill_Treat_MaxHPPer` and adding `GA_UltraSkill_Treat_Value`. At level 1 and instigator MaxHP 10,000, the helpers yield **440 Skill**, **506 with the eligible A3 branch**, and **460 Burst** before native healing modifiers/clamp/rounding. A recipient with different MaxHP does not replace the instigator's basis. These warning-free helper graphs resolve magnitude expressions; they do not select lowest-HP recipients, determine channel tick count, or establish heal delivery/overheal callbacks.

### Wake duration calculator recovered, October 18

The expansion's `Buff_Edgar_UltraSkill_Field#/1/Properties/DurationMagnitude` selects `Cau_Edgar_FieldDuaration`. The latter's warning-free `CustomCalculateBaseMagnitude` (`#/2`) initializes AdditionalDuration=0, then adds `max(owner TreatKey count,0) * PerKeyAddDuration` only after a valid player-character cast and a positive `Buff_Edgar021_Passive2` check. It always proceeds to the FieldDuration sum. The bound `DT_EdgarEffectFigure#/rows/GA_UltraSkill_Duration` and `GA_UltraSkill_PerKeyAddDuration` are constant 10 and 1 at the requested coordinate 0. Thus the helper gives 10 without the passive and 10/11/13 with the passive and supplied key counts 0/1/3. There is no key deletion or count cap in this function; the modeled three-key cap, capture instant, consumption and callback clock need their own consumers. This narrows the existing unconditional modeled duration formula without silently changing it.

### Passive description coverage, reviewed October 18

Saved description source: `guide_edgar.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Gentle Edge describes an immediate 120 Ultimate Energy grant to the character who triggers Charge through Support, followed by suppression of Vita Pistil Ultimate Energy on Slowed targets and a 30s cooldown. Unchanging Warmth describes one Key of Truth from Skill or Support, cap three, with one second of Wake extension per key. This distinguishes native description-level generation from the modeled allied-QTE A1 and no-key-consumption choices. Recipient, suppression lifetime, guard ownership and delivery/capture clocks remain open.

### Charge-enhancement magnitude and guard recovered, October 18

Initial `GE_Reaction_GuangLingXiang_Charge_1021#/2/Properties/Executions` names **ChargeExecution** with Source/snapshot additive PParameter1 from `Calc_Edgar_Passive1`. Its warning-free `#/2` function validates **OwnerActor ASC**, rejects the positive path if it already has **GE_Reaction_GuangLingXiang_ChargeCD**, and otherwise returns `Reaction_GuangLingXiang2_Charge_1021=120` at coordinate 0. Unlike the mutating Parry calculators, it does not call AddHeterochromeValue or modify resources directly. This locates the magnitude and checked guard class, not the described 30-second CD's actual grant/owner/clock, Support-trigger admission, efficiency bypass or later Vita Pistil suppression.

### Skill healing recipient and A2 cooldown cap recovered, October 18

Initial **GA_Edgar_Skill#/8/Properties/TeatTargetType** selects **TargetType_GetLowestHealthEquippedPlayer**. Its warning-free **GetTargets** (`#/1`; ) clears its prior candidate and scans the owning player's **GetEquippedPlayers()**. It skips invalid entries, **HPCurrent exactly zero**, and HPCurrent nearly equal to HPMax at tolerance **1e-6**. Among the remaining entries it selects the smallest **absolute current HP**, using a strict `previous HP > candidate HP` replacement. Equal HP keeps the earlier selected entry; there is no HP-percent division or positive-HP test in this selector. Supplied members at **1000/10000 HP** and **500/1000 HP** therefore select the latter despite its higher HP percentage. If none qualifies, all output arrays are empty; otherwise only OutHitResults receives one synthetic hit for the selected player, with OutActors/OutTargetData empty. This recovers a concrete selection/cap rule for this source, without choosing native party order or attributing behavior to other heal selectors.

Skill **OnReceiveEvent** (`#/7`, 33..1212) calls the parent, then only for **Event.Montage.Player.Skill.2** requests that target type. For each returned player hit, **GetIsControlledCharacter()** chooses **Buff_Edgar_Skill_Treat** versus **Buff_Edgar_Skill_Treat_Background**, with the Skill's avatar player passed as the explicit actor source in K2_ActivateGameEffectsWithClassByActor. The background class declares the ordinary heal as parent and only overrides bNotShowDamageFloat=true; it is not a separate lower-HP-ratio formula. Native helper-to-hit conversion, repeated callback frequency, healing execution/capture/clipping and source propagation remain separate; the selector itself is no longer unknown.

**K2_GetCustomAdditionalCooldown** (`#/4`) returns AdditionalCooldown unchanged without the avatar's Buff_Edgar021_Level2. With A2 it returns **clamp(AdditionalCooldown,0,3)**, from MaxBranchAdditionalCD → **Buff_Edgar_Star2_MaxSkillAnimTimeCD=3** at coordinate 0. Supplied additional values -2/2/5 yield **0/2/3** with A2, versus -2/2/5 without it. This caps an additional term, not total Skill CD, and does not establish how native animation/channel time produces that input. Skill end (`#/5` → `/2`, 10..156) requests removal of Buff_Edgar_Skill_linkeffect from its current ability target; the reached attack-target-change callback merely stores old/new targets and supplies no separate cleanup request. Earlier target-link cleanup and live cooldown clocks remain open.

### Wake field samples duration before requesting key removal, October 18

Initial **BP_EdgarUltraSkillField** recovers a consumer missing from the duration calculator alone. Its **ReceiveBeginPlay#/4 → ExecuteUbergraph#/3 at 158** pushes the key-removal continuation, then on authority calls **CalcFieldDuration** and **SetLifeSpan(OutDuration)** before executing that continuation. **CalcFieldDuration#/2** uses the same **10 + max(Keys,0) × 1** expression only with owner Buff_Edgar021_Passive2; without that marker it returns base 10. The continuation (`#/3`, 15..157) obtains the owner as an ability character and requests **K2_RemoveGameEffectsWithClass(Buff_Edgar_TreatKey,-1,null)**. It is reached even after the authority test fails, although native authority rules decide whether a removal settles there.

This establishes source ordering **calculate field duration → set lifespan → request all-key removal** on the authority path. The current executable no-consumption policy above remains a historical bounded model and conflicts with this located source request; it must not be promoted to native persistence through Burst. Whether the separate duration GE is instantiated before or after that clear, native spawn/owner propagation, removal success, effective lifespan clock and periodic heal delivery remain open. The named CDO binds TreatGE=Buff_Edgar_UltraSkill_Treat and bAffectedByOwnerGameplayScale=true, but its TreatInterval multiplier is omitted. The SphereComponent export retains one warning; the named actor CDO and Blueprint functions decode fully. Source:  and `field-source-config.txt`.

### R6 pending-Charge settlement input recovered, October 18

The warning-free expansion **Calc_Edgar_Level6Extra_Charge:CustomCalculateBaseMagnitude#/2** casts OwnerActor to player, obtains its active **EdgarLogicObj**, and returns **floor(PendingAddAbilityCharge)**. It does not derive this accumulator from a fresh HP-loss read, clamp it to 20, clear it, or write resources. Supplied pending values **0.9/1.9/20.9** therefore feed **0/1/20**. **GE_Edgar_Level6Extra_AddCharge#/1** binds the helper with multiplier 1 to additive **Source/snapshot PParameter1** in **ChargeExecution**; it has serialized duration magnitude **0.1**, period **1** and cap 1, which alone do not settle actual execution frequency.

The reached **player_021_edgar#/3 EdgarLogicObj** names this carrier as **AddChargeGE** and selects **Buff_Edgar021_Level6_extra**. Its DamageHPPercentConvertToChargeRatio has explicit multiplier **100** against the constant **0.03** row Buff_Edgar_Star6_Extra_ChargeRatio; MaxChargeCanAddPerUltraSkillActivation has explicit multiplier **1** against the constant **20** row Buff_Edgar_Star6_Extra_LimitChargeRatio. Preserve those units/multipliers and the description's 3%-HP-loss/20-Charge rule; these fields do not reveal the native conversion direction, accumulator update, cap-before-rounding or reset point. The player/component exports retain two unrelated parser warnings; the named logic-object and separate helper/effect exports are the scope of this finding. Source rendering:  and `passive-execution-bindings-compact.txt`.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Edgar_Skill`, `Edgar_Skill_End`) and Burst (`Edgar_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Continuous channeling & heal target selection:** Continuous-channel packet cadence/cancellation and lowest-HP heal target selection; effective healing/overheal may affect the Wrong Gate damage proc.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Resolve continuous-channel packet cadence/cancellation, native conversion/delivery of the selected heal hit and effective healing/overheal effects on Wrong Gate. The exact reached lowest-absolute-HP selector, exclusions and single-hit output are now known above; do not reopen that question as unspecified targeting.

## Prior findings

- [`reviews/q10-game-action-binding/edgar-action-binding-model.json`](../../../../reviews/q10-game-action-binding/edgar-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/edgar_actions.rs), [`edgar-normal-chain.json`](../../../../reviews/q10-game-action-binding/edgar-normal-chain.json), [`edgar-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/edgar-skill-burst-packets.json).

## Research history

- **Accepted:**8/8; selected sections supersede historical runtime assumptions. Native/public approaches already used; no third reader/body hunt is needed for this owner.
- **Next:** Fadia's first unchecked block, reusing its existing dossier and source callbacks. Preserve this owner for implementation rather than optional fidelity polishing.

## Relative combat timer binding

`Edgar_UltraSkill` Common.EndAbilitySkill at 3.402224 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
