# Chiz action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1073`.

**Responsibility:** Chiz outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** the complete selected Rust owner model is accepted through [full-kit acceptance](#executable-full-kit-acceptance). Explicit checked hypotheses establish the implementation model, not observed native bodies. Earlier prefix receipts retain their bounded historical scope.

**Native-certainty limits:** [unrecovered native execution policies](#unknown-execution-policies). Additional authored control lowering belongs to native workflow; final shared interactions belong to Q3/C-Interactions.

**Executable Normal acquisition:** `chiz.normal_grain_v1` now grants once per admitted Normal input at its first positive actual HP-loss packet. All child pulses/recipients retain the parent command identity; field exit or a later input cannot rearm it. Zero rounded HP loss grants nothing, while a fatal positive hit qualifies. At level 80 five inputs emit twelve damage packets but grant 2250 Grain, or 2700 with selected A4. Native chain/continuation and fractional/fatal checks own these numbers. The continuing Contemplative Cat source also checks two chains, independent party Fons, native/WASM/actual Worker reports and replay. This receipt covers Normal acquisition cadence; the accepted complete owner model is documented below.


The [Grain and market source findings](grain-and-market-source-findings.md) identify the character logic object's configuration, native field-layout comparison, captured loop-cost argument and reached settlement guard. Those facts and the declared finite policies underpin the executable Skill/Burst financial model.

Public source Burst declares the existing market, exact-finance, pressed-Skill
and ordinary-clock models before native initialization. Their owner-defined
initial Grain/Loan state and financial schedules remain separate from party Fons.

### Native source continuation and Barren inputs

Typed source hold parameters now select the existing Barren owner model before
native initialization. Explicit `attack[hold=0]` is tap Barren; positive hold
requires a live Normal continuation and uses the existing combo route. Omission
continues to select an ordinary Normal. The [two-period Barren source](../../../../examples/chiz-barren-inputs.nte)
retains an authored `t1` lock, the secondary request's exclusion, final AoE,
one450-Grain acquisition per branch/input and independent party Fons. Jump
after the cue cancels the final request while preserving the2.5s tap limiter.
The cue at500015us settles before same-time source observation, so a source
Jump window then follows the admitted Barren's own868260us gate.

The [buffered held-Full source](../../../../examples/chiz-buffered-held-full.nte)
yields its two-stock hold at954108us. A separate third Skill input pays one
stock then buffers to the native2581175us loop gate, with its first Full
packet at2833348us. Two continuing15s periods retain40 packets on two dummies,
two actual third-command identities and no terminal remainder spend. At level80,
zero savings/debt gives first cost15000/16 and second cost
`(15000-15000/16)/16`, ending at58125/32 outstanding loans; the second period
does not borrow from a reset account. Native3s refills fund the later period.

`simulation/tests/source_stocks.rs` also executes two nine-second tap-combo
periods with954108us/1661079us yield points, inherited live pause and cancellation
of Half's unreached tail. Saved source exports and direct native replay agree
for these routes and the failed tap-limiter input. These checks bind the selected
owner model to source, rather than establishing new observed game timing.

## Selected collection handoff (October 19, 2026)

The selected model here supersedes historical runtime omissions below. `source` means the named frozen native table/CDO/graph, `derived` independent arithmetic, and `inferred` the declared finite dispatch/financial/clock rule where native methods remain unobserved. No claim of recovered protected C++ bodies is made. Three previous approaches already include client exports/AST/resource layout and public descriptions; do not repeat them before closing implementation decisions. Preserve [Grain/market source findings](grain-and-market-source-findings.md), [native multipliers](../../../../crates/data/src/native_damage/chiichan073.rs) and frozen public stat arrays.

### Complete packet specification

**Mixed.** Native direct packets are Cosmos/ATK with ordinary live crit; categories Normal/Branch A, plungeX, riposteZ, QTEH, SkillE, BurstQ. Normal talent owns Normal/Branch/plunge/riposte, Skill owns all Skill variants, Burst owns its actor train, Support owns QTE. **R3 adds no talent levels** for Chiz. Keep all13 table entries; level1/8/9 checks and channels per actual packet are:

| Suffix after GE_Player_Chiichan073_ | Count in its reached action | Coefficient1 /8 /9 | Charge / Cycle / Break |
| --- | ---: | --- | --- |
| Melee1_Damage |1 |.345 /.591 /.639 |1.727 /2.872 /.350 |
| Melee2_Damage |3 |.171 /.293 /.317 |.855 /1.421 /.173 |
| Melee3_Damage |1 |.560 /.960 /1.037 |2.804 /4.662 /.568 |
| Melee4_Damage |5 from one actor |.209 /.358 /.387 |1.046 /1.739 /.212 |
| Melee5_Damage |2 |.397 /.680 /.735 |1.987 /3.303 /.402 |
| AirAttack1_Damage |1 |.900 /1.542 /1.666 |4.5 /7.5 /.9 |
| Branch2_Damage |1 to locked target, first phase |.960 /1.645 /1.777 |4.8 /8 /.95 |
| Branch2_1_Damage |1 secondary request excluding lock, then1 final AoE |.960 /1.645 /1.777 |4.8 /8 /.95 |
| PerfectEvadeAttack1_Damage |1 |.300 /.514 /.555 |.767 /1.267 /.833 |
| PerfectEvadeAttack2_Damage |1 |.600 /1.028 /1.111 |1.533 /2.533 /1.667 |
| QTE_Damage |1 |2 /3.428 /3.702 |8.6 /0 /2.5 |
| Skill1_1Damage |1 in No Grain |.190 /.326 /.352 |.966 /1.578 /.193 |
| Skill1_Damage |1 in No Grain |.400 /.686 /.740 |2.034 /3.322 /.407 |
| Skill2_2Damage |2 in Half Grain initial section |.532 /.912 /.985 |2.65 /4.45 /.55 |
| Skill2_Damage |1 per held-loop pulse, at most16 source pulses |.250 /.428 /.463 |0 /0 /0 |
| Skill3_Damage |1 in Full Grain |.283 /.485 /.524 |1.419 /2.359 /.276 |
| Skill4_Damage |3 in Full Grain |.418 /.716 /.774 |2.094 /3.48 /.408 |
| UltraSkill1_Damage |4 from actor1 |.446 /.764 /.826 |0 /0 /.134 |
| UltraSkill2_Damage |1 from actor2 |.447 /.766 /.827 |0 /0 /.134 |
| UltraSkill3_Damage |1 from actor3 |2.771 /4.749 /5.129 |0 /0 /.831 |

Ordinary chain **12packets/5.580ATK/16.300charge/27.098Cycle/3.301Break**, recalculated from individual rows, correcting historical27.135/3.298. Melee4 is one period.15/cap5 actor, not5 simultaneous casts. No Grain2 primary packets=1.012ATK/3charge/4.9Cycle/.6Break; Half Grain initial2=1.824/5.3/8.9/1.1; Full Grain4=2.633/7.701/12.799/1.5. Three pressed stages together **8 primary packets/5.469ATK/16.001charge/26.599Cycle/3.2Break**, before eligible separate money effects. Held Full Grain uses Skill.3_1 instead of.3, same primary coefficient but **no extra settlement request**. Never add all16 loop notifies to an ordinary tap Half Grain that never enters its Loop section.

Burst creates **6** primary packets,4*.764+.766+4.749=**8.571ATK/1.501Break**, no energy/Cycle. UltraSkill.3 creates3 actors whose source delay/counts are.01+four.1-period pulses, .49+one and.98+one. UltraSkill.1/2 attach market/display effects, not extra seventh/eighth HP hits. Riposte is source two-hit .514+1.028=1.542 at8 even though public prose calls one instance. Plunge multiplies by1+h for declared height bonus0..1, paying authored airborne time.

**Barren Branch:** the reached Branch.2 request is locked-target only; Branch.3 selects **other targets**, explicitly excluding the current lock; Branch.4 is a final AoE. One dummy receives2 packets (first locked+final), not both first-phase containers plus final. Two all-in-range dummies can produce3 causal requests: locked, secondary, final; each dummy still receives2 full coefficients. Raw channels once per nonempty request, not per target. Choose the same reached two-phase packet route for tap/held Barren; tap's2.5s Cheat carrier is its admission limiter, combo hold reaches the branch directly. The otherwise unbound `Branch_Damage` row .775/1.328/1.434,3.9/6.45/.8 remains retained metadata, **not another automatically delivered hit**: no reached EffectContainerMap emits it. This inferred route selection fulfills the described two-hit branch and the actual secondary exclusion without fabricating an unlocated simultaneous attack.

**Grain bonus packets are separate Cosmos/E effects, not additional ATK:** `Skill1_CoinDamage` and `Skill2_CoinDamage` retain fallback table ratios .590/1.011/1.092 but their located custom executors override the damage base. Select **replacement**: do not add the fallback coefficient*ATK or multiply the flat override by that coefficient a second time. Settlement Coin1 base is `max(0,floor(LastSettleStockResult*1))`; loop/terminal Coin2 base is `max(0,floor(floor(capturedActualSpend)*3))`. Both have zero charge/Cycle/Break. Sample live Cosmos/general/Skill/final zones and target DEF/RES at delivery, **no ATK scaling of this overridden flat base**. Source helper reads CurCoinDamValue, not CurrentCoin, and discrepancy diagnostics do not reject the override. At captured2.9, loop base6, not8; at settled2.9 Coin1 base2, independent of live wallet or ATK. This replaces the full base, with the source's two distinct rounding rules preserved.

Coin1 is requested once alongside No Grain Skill.1, pressed Half Grain's first Skill.2_2, and pressed Full Grain's first Skill.3, when that event has a positive eligible settlement. Held Half Grain's initial two Skill.2_1 hits and held Full Grain Skill.3_1 do **not** duplicate that settlement. Each held-loop Skill.2 emits one primary .428ATK and one Coin2 from actual spent Grain; if finite loop/release consumes a final positive remainder, emit one terminal Coin2 with **no fabricated extra ATK primary**. Early third-charge finish preserves the remaining Grain and has no terminal spend/coin packet. The terminal request is an explicitly inferred cash-out of the described end-consumption, once by hold-generation ID; ordinary loop16 pulses often exhaust the captured budget so there is no terminal packet then.

Coin damage normally **cannot crit**: source FixedCritRate.000001 is selected as the no-crit encoding consistent with its description, not a meaningful one-in-a-million crit roll. During Grain Market one first **positive Coin1 settlement packet** is forced critical, once per market, then later Coin1/Coin2 remain noncritical. That exception never grants ordinary ATK packets or every target a fresh first-crit token. On an AoE settlement capture the token once and apply the same forced-crit status to all its captured recipients; consuming it per first target would violate equal recipient damage. The first settlement flag remains until a positive settlement damage packet occurs; zero amount cannot create HP damage/crit or spend it. Source flat-base replacement/timing/native cash-out remain explicit inferences, checked against both executor-return dimensions, independent public distinction between fixed Grain and ATK damage, and absence of double application of the fallback row.

### Complete timing specification

**Mixed; selected timing model, not measured wall time.** Run montage progress at1/s; connected direct hit pause adds `duration*(1-rate)` once per causal packet, not per dummy. Reached CDOs give Normal1/3 .06/.01→.0594s, Normal5 .1/.01→.099 each, plunge.15/.01→.1485, Skill1/Skill1_1 .2/.1→.18 each, Skill4 .1/.2→.08 each, second riposte.2/.08→.184. Other rows' absent duration adds0 in the selected profile. HitPauseData.DelayTime.0167 delays **pause onset**, not damage delivery; apply only pauses whose onset precedes a gate. Already spawned actor pulse clocks and all financial/market/CD/buff timers run on real time, independently of owner montage pause. These explicit defaults avoid inventing another global animation speed.

QTE uses its actual rate curve over.3s, onset hit+.0167: cubic Hermite endpoints `(0,.1)` and `(.3,.65000004)`, endpoint tangents **3.5355434** and **-.056667246**, no tangent weights. Selected real-time rate integration gives lost montage time `.3-[.3*(.1+.65000004)/2 + .3²*(3.5355434+.056667246)/12]` = **.160558414155s**. Do not guess a constant.01/.1 from another action or use the curve as a damage multiplier. Source asset `Chiichan_QTETimeDuration_Curve`, source CDO HitPauseData.

| Action / phase | Damage/request progress coordinates | AnySkill recovery coordinate |
| --- | --- | ---: |
| Normal1 |.20071144 |.53120935 |
| Normal2 |.19874471,.28370366,.39820552 |.78391576 |
| Normal3 |.51755345 |.98278195 |
| Normal4 |actor spawn.23610586; pulses spawn+.15*n, n=0..4 |1.0994582 |
| Normal5 |.3330858,.66653675 |1.1978028 |
| Barren primary/secondary first phase |.7493102 locked, .7505238 secondary |1.9124784 |
| Barren final phase |1.4645377 |same action |
| Tap Barren admission cue |.24971676, no HP packet itself |.50001514 cue montage, then branch train |
| Plunge, after landing |.19945621 |.50001514 |
| Riposte |.15095001,.333387 |1.026147 |
| QTE |.80015975 |1.7367196 |
| No Grain |Skill.1_1 .18329817, Skill.1 .5995581 |.919881 |
| Pressed Half Grain initial |Skill.2_2 .59989333, Skill.2_1 .96686053 |1.0504841 |
| Held Half Grain initial |Skill.2_1 .59989333,.96686053 |1.0499097 |
| Pressed Full Grain |Skill.3 .2521733; Skill.4 .75125605,.8652367,.9496794 |1.5002682 |
| Held Full Grain |Skill.3_1 .252173; Skill.4 .751256,.865237,.94953763 |1.4998609 |
| Burst |actor spawn2.346949; actor1 +.01,.11,.21,.31; actor2 +.49; actor3 +.98 |3.7542133 |

Derived ordinary full chain **4.91196806s**, No Grain1.279881, pressed Half1.0504841, pressed Full1.7402682; support1.897278014155. The burst's six actor hits occur2.356949,2.456949,2.556949,2.656949,2.836949,3.326949. Display2.4201596 and market entry **UltraSkill.1 at3.420264** are separate source events. Market lasts7 World seconds from that entry, not actor8s lifetime or montage9.133333. Primary Burst damage precedes the first-settlement critical token, so that token cannot turn all six Burst HP packets into crits.

**Held loop sections:** initial Half segment uses its first two packets; at completed initial End boundary progress **1.0332553**, a live held route jumps to Loop/LoopHold, whose source origin is **5.983333**. Subtract that origin from its absolute coordinates; there is no invented6s idle wait. The16 source Skill.2 coordinates are **6.1833334,6.4166665,6.65,6.883333,7.116667,7.35,7.5833335,7.8166666,8.05,8.283334,8.516666,8.75,8.983334,9.216666,9.45,9.667134** for Loop; LoopHold differs only in final**9.666817**. They are one finite ~.233333s train, first entry+.2000004. Capture LoopCostVal at entry once. Natural loop section ends at relative3.7333333; its terminal remainder is settled then once. Source following exit segment begins9.7166663; AnySkill10.132525/10.166059 yields additional exit recovery **.4158587/.4493927** for Loop/LoopHold. Selected early release jumps to this exit segment, paying the same recovery after its last settlement. Pressing a third available charge after the reached loop Jump gate6.25213/6.2513723 (relative.268797/.2680393) exits to Full Grain without consuming the remainder; before that it is a pending authored input, not a free early damage packet. No polling or reset of loop cost at each pulse. Source14.7 montage concatenation is not14.7s attack occupancy.

Other cancellation gates (add only elapsed owner pause): ordinary Normal Jump stages1..5 `.33381662,.24887228,.6699663,.84781486,.8425632`; Move `.61601406,.77015793,1.8492458,2.115799,1.1978028`. Absent separate Skill gate uses AnySkill. Barren Jump.86826044/Move1.9173348; cue montage Jump=AnySkill/Move.7651598. Plunge Jump=AnySkill/Move.831415; riposte Jump/Skill0 and later Skill/Jump/Move.45029625. QTE Jump/Move=AnySkill. No Grain Skill.63204306/Jump.71445745/Move1.2318158. Half pressed Skill.66903627/Jump.65993357/Move2.4182384; held Skill.669036/Jump.6328175/sameMove. Full pressed Jump1.1648549/Move2.6164336; held Jump1.3272481/Move2.5833526. Burst Jump=AnySkill/Move4.255869. Front dodge Jump.082544/Skill.333716/AnySkill.998284; back Jump0/Skill.350074/Move.797249/AnySkill1.0125786, no damage. An explicit successful Critical Dodge authorizes one next-Normal riposte, cleared by use/another non-Normal/exit; no invented native duration number.

Ordinary switch needs recovery and lock expiry: QTE explicit lock1.6507982, Burst lock4.25. No universal extra switch delay. A committed Skill segment/held train may continue as Chiz's afterimage off field under the source-described dual-DPS behavior; uncommitted next pressed stages still need an actual legal owner input, not automatic free continuation. Normal/branch future montage requests cancel on exit, already spawned actors retain their finite pulse cap. Grain Market/TimeSlomo ends immediately on field exit/death, with financial settlement/repayment in its own transaction; it does not pause the whole party time axis. No damage coefficient is inferred from portal/visual actor callbacks. Source death cancels future owner actor admissions, target death cancels that recipient, and already consumed resources are not refunded.

Skill is a **three-stock** action, not three casts locked behind three ordinary3s cooldowns: MaxSkillBean3, GA-bound recharge3s. Selected sequential refill begins when below3, one charge at first-consumption+3 then every3 while below cap, stops at cap; another consumption does not reset an already running deadline. Each pressed stage costs1; held entry reserves2 or3 as declared and cannot exceed current stock. Spent charges remain spent on interruption, retained through switch/market/death, normal scene restart only initializes3. No additional normal Skill CD blocks a funded combo. Burst costs100 and CD15 World seconds from accepted commit. Tap Barren has its2.5s source Cheat admission limiter from accepted tap (marker deduplicated); combo-held branch is recovery/hold-gated rather than a second independent tap reset. Every authored hold, recovery and wait remains in the real DPS denominator.

### Complete resource specification

**Mixed.** The [complete financial model](grain-and-market-source-findings.md#selected-complete-financial-model) supplies level-selected own/loan caps, per-input acquisition, efficiency/deposit/borrowing, exact loop cost/spend, market mode/proceeds/overflow/repayment, finite seeded price generation/explicit trace and interest. These are ready implementation choices; historical missing native-writer notes do not replace them. Keep **Grain, drawn loan, Fons, Ultimate Energy, Cycle and Skill stocks separate**. No amount of Grain directly grants Ultimate Energy, a Skill charge or Fons unless a separately stated event does so.

Ultimate Energy cap100, Burst cost100 from DT_GameplayAbilityCostData/DT_PlayerPackData; initial0 unless declared legal preload. Skill/ordinary attacks do not charge Ultimate Energy or combat stamina; ordinary dodge costs15 stamina from the native cost table and reads the required scenario capacity/initial/consumption-efficiency/recovery profile. Qualified adjacent switch spends outgoing100Cycle for one QTE; the incoming actor's energy is retained. Every listed primary positive-HP packet grants its row's energy/Cycle once per nonempty request, including spawned Normal4 and delayed committed Skill; Grain bonus rows grant0. All per-ally.6 sharing, own received efficiency, exact fractional carry/caps/overflow follow the [shared ledger](../../shared/energy-and-gauges.md), retained across forms/switches/cycles.

B4 Hoarding adds **+.20 Charge Efficiency points while Chiz is active**, removed off field, not a permanent×1.2 on already increased efficiency. Thus with no other bonus she receives own raw16.3 Normal energy as19.56; an active receiver at efficiency1.33 becomes1.53, not1.596. Off-field afterimage energy uses her live off-field efficiency and each ally's own received efficiency; no duplicate grant because the animation remains visually present. B2 Hanahaki adds4 to the **active actor's real Charge-reaction grant**, so a qualifying Blossom pistil hitting Remora yields14 raw energy before that recipient's efficiency, once per pistil regardless targets. It does not grant4 on every Cosmos hit, Grain acquisition, loan withdrawal or Skill settlement. No Charge reaction means no B2 production while the passive remains enabled.

Skill starts with3 stored charges, selected sequential3s refill as timing specifies. Press costs1 and advances No→Half→Full only through legal combo admission; different action/field exit or2s after completed prior segment clears the pending pressed-stage pointer toNo without resetting charges. Held entry requires>=2, pays2 for No+held Half; explicit held3 reserves/pays3 and includes held Full after natural completion. With two paid charges a later legal third press spends1, ends Half early **without spending remaining Grain**, then held Full. A cancelled reserved charge is not refunded. No grains/loans are spent merely by stock deduction; actual loop Coin2 calls perform the source-bound financial cost. Resource0 still permits plain ATK Skill/loop packets; missing charges, energy, airborne/riposte authorization or recovery fail with a diagnostic, not implicit wait/refill.

Fons is account currency, never a proxy for Grain. The selected DPS scenario supplies its account Fons for equipment eligibility explicitly; combat Barren's ordinary Fons gain/kill bonus has no outgoing coefficient/charge conversion in the reached kit. Persistent account/city earning caps remain outside combat simulation; do not credit invented Fons per Grain spend or change that account input during an immortal-dummy comparison. If a finite-kill scenario explicitly changes a damage-relevant account-Fons threshold it must supply those currency events; the chosen baseline does not silently farm the account to satisfy a weapon. Required passives and financial transitions are never switched off to complete a run.

### Complete effect specification

**Mixed.** The [financial owner](grain-and-market-source-findings.md#selected-complete-financial-model) owns every Grain grant, A2 interest, market price capture/revaluation, overflow, loop spend, conversion and exit repayment; their exact timing/rounding is damage-relevant and mandatory. Effects are attributed to their cast/held/market IDs, not to a later global LastSettleStockResult read. Coin1/Coin2 use the separate flat override/base/crit rules in packets, with A6 only on their actual coin tags. No ATK scaling, extra fallback ratio or automatic raw energy is applied to that base. Multiple target deliveries share one financial transaction and one first-settlement critical capture.

`Buff_Chiichan073_UltraSkill_BigA` begins at UltraSkill.1 **3.420264**, lasts7s, adds one first-Coin1 critical token, removes that token and TimeSlomo at finish. TimeSlomo also self-removes on death/field exit; the spawned time-dilation actor's remove listener schedules destruction+.01, not another .01s market or damage extension. Selected stationary-dummy model treats this volume as enemy movement/action presentation suppression; no enemy AI/movement clock is part of target DPS, and no unobserved global party/Grain/reaction/CD pause is imported from the unread native dilation parent. Authored incoming-event schedules already specify final real times and are not converted into free outgoing combat time. Actual seven-second market, switch/death removal, repayment, afterimage hits and DPS denominator remain operative. This scope simplification is consequential only outside the declared stationary target/explicit incoming schedule; it does not disable a required resource or damage proc.

Barren Cheat is a cap1 **2.5s** admission handle, deduplicate cast marker; no damage coefficient follows from the tag itself. Traction/launch effects only maintain the declared all-in-range recipient scenario, no hidden vulnerability/extra HP packet. The branch's financial gain is once per actual connected action. Hoarding live field-state efficiency and Hanahaki actual Charge grant are in resources; both are enabled at BT6, even when a particular solo schedule has no Charge. R6's very short actual modifier and A6's coin-only general addition are in passives. Buff expiry precedes same-time ordinary HP requests; source actor cap/count prevents a surviving visual Boba from spawning another packet train.

### Complete passive and awakening specification

**Mixed.** Ordinary unlocks B2/B4 follow breakthrough; individual effects follow selected identities, R3/R6 selected count. Sources:1073AwakenEffect, DT_ChiichanEffectFigure/global cap/grant curves, and located custom application helpers. All relevant choices are finalized; awakenings are not manually disabled to allow an A0 baseline or combined merely by number.

| Selection | Concrete complete rule |
| --- | --- |
| B2 | Hanahaki: add4 raw energy to the active actor's actual shared Charge reaction (10→14), before recipient efficiency/carry, once per Blossom pistil not per dummy; owner can be off field. |
| B4 | Hoarding: own live Charge Efficiency+.20 points while active, removed on exit, reapplied once on entry, not a multiplicative1.2 on other bonuses. |
| A1 | C andL each×1.1, level80 each16500. Market temporary capacityC+L follows those caps; no repeated10% cap growth on refresh. |
| A2 | Surplus-only exact10% interest once/sec under the financial owner, no loan interest, no10% every.5. |
| A3 | Native rise weight50→75; normalize against lose50, so selected finite generator up probability.6 rather than.5, not1.0. Explicit recorded price traces are scenario overrides, not an A3 claim. |
| A4 | Battle acquisition efficiency adds.20 to the common multiplier; never changes a settlement return, loan limit or raw Ultimate Energy. |
| A5 | ENTER_BATTLE grants C Grain once through the current financial deposit route: outside market repay-then-credit, during market capped invested credit; no repeat on every field entry. |
| A6 | Actual Coin1/Coin2 tagged packets get **general damage+.15**, within their summed ordinary bonus zone. No.15 ATK coefficient, no bonus to every Skill primary, no modification of spent balances or floor stages. |
| R3 count>=3 | Acquisition adds.50 **while Debt**; no talent-level addition for any action. With A4 in Debt multiplier1.70, not1.8. |
| R6 count>=6 | Before **Chiz damage calculation**, apply target Cosmos RES-.10 for **.05s** from Buff_Chiichan073_LevelExtra2_1; source parent explicitly uses SKILL_HIT_BEFORE_CALC, so the causal hit already benefits. One source handle refreshes/replaces, no accumulating-.10 per hit. It is not an invented15/20s persistent shred. |

R6 applies to every eligible Chiz primary/coin damage request, including committed off-field packets; matching Cosmos damage from allies during its actual.05s target window can benefit as well. Source target child HasDuration.05 and additive DamageResistCosmosBase from Chiichan_LevelExtra2=-.1 supersede the old description's unspecified duration. Expire before an equal-time later packet unless that packet refreshes it before its own calculation. Target death/scene cleanup removes it; owner field exit does not retroactively erase its remaining.05. A6 helper explicitly tests State.Damage.Skill.ChiichanCoin1/2 on current GE; ordinary Skill1_Damage fails that test. At flat bonus1000/DEF.5/RES0, A6 alone gives575HP, not650 from multiplying the amount by1.15 then adding.15 again. R6 at target RES.20 changes the causal Cosmos factor.8→.9; it is not10% DEF ignore. Every passive's event/cast deduplication remains independent of target enumeration.

### Complete forms and alternate-action specification

**Mixed.** Ordinary Normal stages1..5 wrap; different action or field exit resets stage, continuation survives full recovery plus2 action-clock seconds. Preserve already spawned five-pulse Melee4 after field exit, source living/target eligibility, one Grain-grant token for that stage. Appear/Pre routes use this owner's Normal1/plunge packets; inherited Female051 prototype entries are **not** an additional Nanally attack or free talent override. Plunge requires an authored airborne/landing interval and0..1 height bonus. Critical Dodge creates the one-use riposte authorization; its two distinct source damage requests/grants remain two even if public prose calls one strike. Ordinary dodge always pays15 stamina, no automatic successful dodge/parry in a dummy scenario.

Tap Barren requires no live Cheat limiter; selected route plays the cue montage to its.50001514 recovery then the reached two-phase Branch1 train. Its2.5s limiter starts accepted tap time, one marker cannot refresh it twice. This finalized inference connects the source HasGE/LogicNot hold-start selector, cue and actual Branch1 damage schedule; the unbound Branch_Damage row is not used as a fabricated extra packet. Combo-held Barren is admitted after a Normal's AnySkill gate with held input true (source HoldTrigger omits a numeric duration, so no invented.2s tax); cancels future Normal montage requests and goes directly to the two-phase train. It does not reset the independent tap limiter or multiply Grain per swallowed target. No target/loot city simulation changes damage.

Three **pressed** Skill segments have independent paid1-charge admission and No→Half→Full pointer. Each full segment delivers its known packets/optional single Coin1; holding Half instead enters its finite Loop. **Held2** pays2 upfront, automatically commits No Grain then held Half initial/loop; normal release cashes the remaining available Grain once and exits, without a free Full Grain segment. A legal later third press pays1 and exits to held Full, preserving the remaining wallet/loan availability. **Held3** reserves3 upfront and performs No Grain→held Half→held Full after its loop/end cash-out; the reserved Full cannot be charged again on entry. Full held has no additional Coin1, unlike pressed Full. Source AutoCostSkillBean section names determine the segments, not three extra cast costs on top of the paid hold. Both loop sections use one capturedA/16, at most16 source pulses and one positive terminal remainder, not endlessly spawning periodic damage at0 resources. Releasing before Loop pays the committed initial segments/recovery but no unrequested loop cost; interrupting after a Coin2 spend cannot refund it. A failed early-third admission does not consume another stock or terminate a valid hold.

Committed Skill segments/held trains can continue as an afterimage after field exit, preserving owner/level/amount/paid-charge data and live damage stats; **new uncommitted owner inputs** are unavailable while off field. The market still exits/repays according to its own switch rule, and Hoarding efficiency is removed. A queued fixed coin packet does not recompute from the repaid live wallet; later actual loop spends use then-live availability and the original captured request, so no additional credit line appears. Source death cancels future segments/loop admissions, already captured actor requests follow their pending-hit cleanup. Third-press continuation cannot be authored by another actor's input slot. These rules implement the described dual-DPS continuation while retaining ordinary action legality.

Grain Market is the only combat economy form: starts at its actual Burst marker, seven World seconds, converts unused loans and invests Grain, permits one settlement per pressed stage/one per held sequence, then repays. New price input/stocks are never reset merely by the form. Recast replaces via a real old-market exit before the new marker, no overlapping investments/listeners. No ordinary Normal damage coefficients change in market; its explicit first Coin1 crit token is not a universal Burst crit buff. Grain ownership persists after market and after loans exhaust. The complete price/cap/transaction/interest model is linked in resources; no leftover “native body required” branch is used as an execution escape hatch.

### Complete recipient and dependency specification

**Mixed.** Under stationary all-in-range scope Normal, plunge, riposte, QTE, Skill primary/coin effects and Burst actors hit all eligible living dummies for full coefficients, sorted stable index; no located recipient cap/falloff or damage division. Re-evaluate eligibility at each request, never duplicate an actor because several dummies were hit. **Barren** deliberately differs: capture the live locked recipient at first phase (fallback lowest living if lock lost); Branch.2 hits it, Branch.3 hits every other eligible dummy excluding that captured recipient, Branch.4 final range hits all. Preserve that causal first-phase lock even if its first hit kills the target; changing it between two near-coincident requests would create a double first-phase hit on a new lock. Dead recipients are removed; a later final AoE can still hit remaining targets. One dummy gets two requests, two dummies produce three nonempty causal request sets but each gets two full coefficients. HP/Break/resource rows are per request, Grain acquisition once per branch action, financial costs once per cast/coin event, not per victim.

Coin amount/forced-crit capture is shared across that packet's recipients; ordinary critical RNG follows shared seeded stable packet/target order. A3 price RNG uses its separately declared continuing substream/draw count so changing AoE target count cannot change the market chart. Fons is an authored account/equipment scenario input, separate from O/D/W/Charge. Target death cannot repay a Chiz loan or rearm her first-settlement token. R6 target modifier lasts.05s and can affect adjacent causal same-time packets once, not a global permanent Cosmos shred. All spawned Normal4/Burst actor count/period/lifetime clocks are retained with their owning cast, not reset on reporting period boundaries.

Use [damage/stat zones](../../shared/damage-and-stat-zones.md), [equipment](../../shared/equipment-builds.md), [energy/gauges/stamina profile](../../shared/energy-and-gauges.md) and [cycles/Break](../../shared/cycles-and-break.md). Cosmos+Anima qualifies Blossom, Cosmos+Lakshana Remora, and real Blossom-on-Remora admits Charge; B2 replaces its10 grant with14 for the active receiver only. Do not trigger that passive on a Cosmos primary alone. Target Break uses the stated per-row values and each actual recipient's declared gauge/acceptance/reset; no universal vulnerability/fake Broken event. A target's R6 Cosmos RES applies live to direct/coin damage, ordinary stats never scale Grain principal. Equal-time financial events execute before their captured money damage, with native.2 gate; ordinary expiry precedes new input/hit, source actor final cap admits no duplicates. These owner/shared rules preserve indirect damage and resource dependencies without requiring native geometry or enemy AI.

### Complete implementation handoff

**Mixed; October19 independent continuing-state audit passed.** Implement the seven selected specifications and [financial model](grain-and-market-source-findings.md#selected-complete-financial-model), including its declared finite price policy/recorded-trace override. Source facts, chosen inferences and current executable omissions remain distinct; no native C++ recovery is required to begin implementing this model after the roster campaign. This8/8 is not a claim of current engine capability or observed market statistics.

**Legal A0 baseline:** Chiz `character:1073`, level80/80 BT6, four talents8, B2/B4 unlocked, no selected awakening; **Contemplative Cat `weapon:fork_mamen` T1 level80 BT6**, **Lost Radiance `set:Suit6`**. Declare set count 4 and supply console contributions through `add stats`; the numeric baseline below is preserved without modeling modules.

Static compiler: character660+weapon512=**baseATK1172**, staticAtkUp.75/flat420→**ATK2471 exact**. HP21114/DEF909/CR.35/additional CD1.54, CI0, energycap100. Static CE1 becomes **1.20 on field** with mandatory B4. Static Cosmos+.30 includes Lost Radiance2pc.10 and authored console bonus .20; do not add those again. [Contemplative Cat](../../weapons/mamen/passive.md) reads declared account **start_fons=1000000**,10stacks*.025=**+.25 Cosmos**, so ordinary bonus factor**1.55**. Fons stays distinct from Grain/loans; the immortal fixture earns/spends none. [Lost Radiance](../../sets/lost-radiance/passive.md) successful Burst commit grants self **.25 DEF ignore for20s**, one refreshable real-time handle, before Burst hits; at DEF1080/attacker80 the factor is4/7 instead of.5, not+25% ordinary damage. Applies to direct and flat Grain Cosmos packets because both take ordinary DEF. Switch preserves its remaining window, source removal clears the equipped passive normally.

**Scenario:** one immortal stationary all-in-range dummy, source level80/targetDEF1080/RES0, no Break bar, full HP, no incoming attacks/dodge/parry; initial Ultimate0, Cycle0, own GrainO0, drawn loanD0, Skill stocks3. Solo has no Blossom+Remora Charge, while B2 remains enabled; no fake4 energy per Cosmos hit. Use an explicitly authored recorded market trace equal to the14 native support values in their saved order for **each** market, index0 at entry and+.5 increments. This is a deterministic scenario override, not a claim about native RNG or the selected A3 generator. To preserve later replay state, trace mode still advances/ignores the generator's28 draws per market. Two continuing periods at offsets0/75 run through150s without resetting any financial/charge/effect/stock state.

1. Six ordinary chains start0,4.91196806,9.82393612,14.73590418,19.64787224,24.55984030, finish29.47180836. They yield13500 own Grain in the first period and117.36 received energy, funding Burst31. Burst actor six-packet train runs33.356949..34.326949; market enters**34.420264**, ends41.420264. Burst recovers34.7542133; explicit switch lock through35.25 does not prohibit own Skill after recovery. Lost Radiance window31..51.
2. Press No Grain**35.5**, Half Grain**37**, Full Grain**38.2**. Source input-price indexes are2,5,7, i.e.**+.38,-.29,+.06**, captured before the chart changes during hit windup. Financial settlements35.7/37.2/38.4 pass the.2 guard; their Coin1 damage occurs36.2795581/37.59989333/38.4521733. All full recoveries finish36.779881/38.0504841/39.9402682; no illegal overlap. First positive Coin1 is forced crit, later Coin1 noncrit; direct primary Skill packets keep ordinary row crit rules. Sequential charge refill38.5,41.5,44.5 restores3, without resetting timers after later pressed casts.
3. Market exit41.420264 repays actual loans. At45 start **held2** (pay2stocks), No Grain45..46.279881 followed by held Half initial start46.279881, its two initial hits46.87977433/47.24674153. No Grain's outside-Surplus money snapshot is captured45.2 and delivered45.7795581 without spending savings. Loop entry**47.3131363** captures available/16, first pulse47.5131367,16th50.9966203. Natural section end51.0464696 exhausts exactly its captured budget here, so no extra terminal remainder packet; exit recovery through**51.4958623**. Stock refill48/51 returns3; no free Full Grain is appended to held2.
4. Ordinary chain53..57.91196806 repays2250 loan from its five acquired stage grants, then wait through75. Repeat from75 using carried debt, not a new O0/D0 initialization. No market/form, pending packet, temporary set buff or recharge timer survives the75/150 boundaries in this particular authored sequence; those states expired/completed normally.

**Financial reconciliation:** first market entry O13500+unused loan15000 givesW28500,D15000. Settlements produce **30000** (39330 before the30000 cap, overflow9330), **21300**, **22578**. Exit repays15000, leavesO7578/D0. Held No Grain bonus base7578; loop captures22578/16=**1411.125**, each of16 spends that exact amount and has flat base3*floor1411.125=**4233**, total67728 before zones; endsO0/D15000. Post Normal repays2250, boundary**O0/D12750**.

Next period's six Normal chains earn13500, first repay12750 and leaveO750/D0. New market startsW15750, then settlement values **21735,15431.85,16357.761**. Exit O1357.761/D0; held No Grain flat base**1357**, loop captures**1022.3600625**, each flat base**3066**, total49056. End held Debt15000, post Normal again leaves**O0/D12750**. No fractions are discarded from the wallet: the packet floor is distinct from actual spend. Across both periods acquisitions31500, realized market net changes-5922/+607.761 and loop spends22578/16357.761 reconcile final net assets O-D=-12750; borrowing/conversion/repayment themselves create no net asset. Resetting the loan at75 would wrongly restore the first period's larger money damage.

Each period has **138 positive HP packets**:84 Normal,6 Burst,8 pressed Skill primaries,4 held initial primaries,16 loop primaries,4 Coin1 (three market+one held No Grain),16 Coin2. Two periods**276**. Raw primary energy138.401/period at activeCE1.2 gives166.0812. First pre-Burst117.36 caps100/overflows17/carry.36, post-Burst48.7212 ends **49+.0812**. Second ends **49+.1624**, cumulative overflow83:332.1624=200spent+83overflow+49.1624. Cycle230.085/period ends100+.085 with130overflow, then100+.170 with360cumulative overflow. Both boundaries stocks3, no pending refill; Grain and energy are independent ledgers.

Independent Fraction/event audit  passed these counts, paid stocks, loan conservation, price capture, floor stages, cap overflow and continuations. Portable damage equation: ordinary direct base=`coef*2471`, Coin1=`floor(capturedProceedsOrOwn)`, Coin2=`3*floor(actualSpent)`; multiply each by**1.55**, DEFfactor4/7 while t in[31,51) or[106,126), otherwise.5, and critical factor**1+.35*1.54** for ordinary primary expectation, **2.54** for first market Coin1, **1** for other money packets. At this A0 noA6/R6 term applies. **Expected total684812.6200862, DPS4565.417467241334 over150s**; first period368081.4400431, second316731.1800431. This is an independent selected-model expectation, not a seeded engine or observed game receipt. Scratch build inputs/reports and audit receipt are , `chiz-build-report.json`, `chiz-two-cycle-receipt.json`; all portable values/schedules are above.

Full implementation acceptance additionally exercises actual specified branches: one/two dummy Barren primary exclusion and split request resources; plunge/riposte grants and paid stamina; held3, early third preserving remainder, release terminal Coin2, partial affordability and afterimage delivery with market exit/deferred financial repayment; price update versus captured input/one-held-settlement guard; A1 capacities, A2 100→110→121 not double-half-second compounding, A3 normalized.6 plus seeded28-draw replay, A4/R3 additive1.7 with repay-first deposits, A5 battle-event dedup, A6 coin-tag-only general bonus, **R6 before-calc.05s shred**, live Hoarding field efficiency and actual14 Charge grant. Runtime must diagnose any missing handler rather than omit financial/passive behavior and report successful DPS. Native-body certainty may be improved later without reopening these accepted collection decisions.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

### Executable unlocked Charge passives

`chiz.charge_passives_v1` binds Hoarding at breakthrough4 and Hanahaki at
breakthrough2, independently of selected awakenings. Actual initial field
entry/re-entry adds one+.20 to Chiz's own received-energy multiplier; exit
removes that contribution. At authored/static1.33 the active value is1.53,
and an already admitted off-field hit uses1.33. No multiplier is applied to
explicit fills. The bounded current No Grain `Skill1_1Damage` primary also
uses its source.966 ChargeAdd through the shared positive-HP source/ally
ledger; remaining Skill packets still require their owner bindings.

A living unlocked Hanahaki provider, including off-field Chiz, changes the
active living recipient's real Blossom-on-Remora Charge basis10→14. This is
once per pistil, including multiple eligible dummies, before the recipient's
own live efficiency/carry/cap. Ordinary Cosmos hits and Grain events grant
no Hanahaki energy. The effect checks source life and unlock at the actual
Charge event; no synthetic reaction or extra packet is created.

Independent checks retain two14*1.33 grants=37.24, active Chiz's two14*1.2
grants=33.6, source defeat between emissions, locked progression and status
expiry. Two caller-timed No Grain afterimage/Normal re-entry cycles give
`2*(.966*1.33+1.727*1.53)=7.85418` own energy and3.2316 ally energy; exit
does not remove the committed primary. Native legal A0 builds also run two
funded Blossom/Remora periods without resetting any Cycle pool: ten real
Charge events across two dummies, first70 energy to Hathor with Chiz off
field, then84 to active Chiz. Their four actual100-Cycle spends, causal
source/recipient receipts, native aggregation and seeded replay are checked.
Hathor80/80 supplies the12s Remora generations; the second Anima provider is
legal Iroi20/20, whose own B2 is not yet unlocked in this prerequisite fixture.
Financial Skill, interest, debt and market remain separate pending work.

### Executable ordinary clock and native Cycle

`chiz.ordinary_clock_v1` binds canonical Normal inputs through the shared exact
actor-progress integrator. Caller hit/recovery delays are replaced by the
reached coordinates. Positive Normal1/3 contacts schedule one pause onset
16700us later, duration60000us at rate.01; Normal5 uses100000us at the same
rate. Misses add no pause, and more dummies do not create more windows. The
full connected recoveries are590609/783916/1042182/1099458/1395803us, totaling
4911968us for twelve primary packets. Admission one microsecond early rejects.

Normal4's producer starts at progress236106 and emits five real-time pulses
150000us apart. Producer delay is separate from the montage anchor, so later
owner pauses cannot stretch its intervals. A living source retains an already
released producer on field exit/replacement; source defeat cancels future
Normal packets and pending pause callbacks. The Normal pointer survives full
recovery plus2 action-clock seconds (inclusive), resets beyond that boundary or on a
different action/field exit, and survives same-actor swaps.

Only the five consumed native Normal HeterochromeAdd fields are added to the
typed Chiz data. The shared per-request Cycle ledger yields27.098 per chain,
independent of Ultimate Charge Efficiency and recipient count. Group identity
survives an empty/dead primary recipient; the first living positive recipient
can settle each request once. All-miss/empty trains retain their unpaused
4595168us recovery and grant no resources. There is no readiness polling.

Independent Rust checks bind all twelve contact timestamps, the four delayed
pause onsets, exact recovery/continuity boundaries, death during Normal4 and
one/three-dummy resource conservation. Legal native A0 Chiz/Mamen builds run
two continuing chains:24 causal packets per dummy, Grain4500, received energy
39.12 under live Hoarding and Cycle54.196, with independent native coefficient
alternatives, aggregation and seeded replay. The earlier caller-timed profile
remains available without this assumption; non-Normal timing, Skill/financial
state and secondary controls remain their separate implementation work.

### Executable pressed Skill and Surplus Coin1 prerequisite

`chiz.pressed_skill_v1` requires the ordinary clock and exact-finance models.
Its canonical Skill input pays one real stock and advances No/Half/Full with
2/2/4 native ATK requests. Three stocks initialize once; real sequential3s
callbacks refill one at a time, retaining the current deadline after another
spend and continuing off field or after defeat. The historical ordinary3s
Skill cooldown cannot block a funded stage. Exhaustion rejects the actual
input before admission mutation, with command/time and typed owner-state
reason; no automatic wait, refund or reset is inserted.

Each cast owns its finite montage progress, causal recipient groups and
connected hit-pause windows. No Grain uses two200ms/rate.1 pauses, starting
16700us after each positive request, yielding1279881us recovery. Pressed Full
uses three100ms/rate.2 pauses and1740268us recovery; Half recovers1050484us.
Extra dummies do not multiply pauses or raw channels. A new real press retains
the pointer through recovery+2s inclusive; another input or field exit clears
only that pointer. Committed Skill requests retain their cast clocks off
field; source defeat cancels unresolved owner damage/financial callbacks,
while the real refill deadline and already spent charges remain.

In this currently admitted outside-market Surplus slice, each input reserves
one financial callback at input+200000us. It runs in ProcCallback after due
ModePause interest and same-time HP, reads the exact live own wallet and
captures `floor(O)` once without consuming Grain. The captured Coin1 shares
No's second, Half's first or Full's first section deadline; zero proceeds
remove that request. Coin1 replaces the entire ATK/coefficient base, retains
native fallback provenance, uses live zones and grants no Charge/Cycle/Break.
Its exact native.000001 fixed-crit sentinel is interpreted as no crit under
this accepted owner model. A6 adds.15 only to actual coin effects. R6 refreshes
one target Cosmos RES-.10 handle for50000us before each causal Chiz hit,
including its first hit; expiry and refresh do not stack the reduction.

Independent Rust checks cover two8-packet pressed cycles, exact contact and
recovery boundaries, empty-wallet suppression, stock exhaustion/refill,
combo reset/expiry, defeat, same-time interest capture and coin-only A6/R6.
At ATK1000 or9000, O15000/DEF.5 with A6 still produces Coin1 base15000 and
8625 noncritical damage. At O450, an exact1s A2 capture yields495; later598.95
captures598 while preserving the wallet fraction. Legal A0 native builds
fund2250 Grain with one Normal chain and run two continuing three-press
cycles against one/three dummies:34 requests per dummy, received energy57.9624,
Cycle80.296, unchanged2250 Grain and three restored stocks, checked through
native aggregation/replay. Held reservation/loop/terminal Coin2, borrowing,
Debt and market remain unadmitted owner work; this prerequisite does not close
the full Chiz task.

### Executable two-stock hold, loans and terminal Coin2 prerequisite

`chiz.held_skill_v1` extends the pressed/ordinary/exact-finance models with a
canonical stationary Skill hold of at least2313136us. It atomically reserves
two stocks for No Grain and held Half. No's actual montage completion starts
Half; Half's reached1033255us boundary starts the loop. These are queued source
boundaries reprojected by actual No hit pause, never a readiness poll or an
invented5.983333s idle. The two held Half primaries produce no second Coin1.

Loop entry captures exact available `O+L-D` divided by16 once. Each of its16
real source pulses spends up to that captured budget before its primary and
Coin2 delivery, own Grain first, then newly drawn principal. Coin2 replaces
the full base with `3*floor(actualSpend)`, has no ordinary channels and does
not crit. Native fallback coefficient/provenance remains visible. Zero credit
still permits the plain.428ATK pulse. Release or natural loop end3.733333s
after entry consumes any remaining credit once as terminal Coin2, without an
extra ATK primary, then pays449393us exit recovery. An excessive requested
hold duration cannot extend the finite16-pulse source train.

The admitted loan principal uses a separate exact capped ledger exposed as
`fractional_resources["chiz.loan_debt"]`: initial/current principal, total
drawn (`granted`), repaid (`spent`), capacity and conservation. Own-wallet
spending and principal draws remain separate. Normal acquisition repays debt
first, then deposits the remainder. A4+.20 and selected-count R3+.50 add in
the acquisition multiplier, reading Debt before the grant. A2's queued credit
stops when loans are drawn and restarts a fresh real Surplus phase when debt
is repaid, even if the own wallet is still empty. Source defeat cancels future
hold costs/damage without refunding prior draws or stocks; real refill remains.

Independent checks retain the source-field O20/D40/L100 budget5, own-first
spending and O0/D98 partial spend2. Two450 grants at A4/R3 multiplier1.7 repay
1000 principal and deposit530, then Surplus uses only1.2. A fresh A0 full hold
spends937.5 sixteen times, yielding sixteen Coin2 bases2811 and final debt15000.
Release at2.55s instead spends937.5 then14062.5, giving2811/42186 bases with
one loop primary and no extra terminal primary. Recovery rejects one
microsecond early. Empty credit, two-stock rejection and death immediately
before/after the first draw have separate causal checks.

Legal native A0 Normal/hold periods at0/15s preserve debt across both cycles:
first available17250 captures1078.125 and sixteen3234 bases; the next Normal
chain repays2250, leaving only2250 available for sixteen140.625 spends and420
bases. Final own Grain0, drawn principal15000, total draws17250 and repayment
2250 conserve both ledgers. There are97 packets per dummy, received energy
59.04, Cycle81.796 and three restored stocks at30s, verified against one/three
dummies through native aggregation/replay. Explicit three-stock holds, early
third-charge Full controls, secondary Grain producers and market conversion/
prices/settlement remain separate pending owner work.

### Executable reserved and buffered held Full

`chiz.held_full_v1` extends the two-stock hold with explicit
`hold_inputs.reserved_stocks=3` or a separate real later Skill tap. Three stocks
are paid at the initial input. Natural loop completion settles any remaining
credit then directly starts held Full, without the ordinary exit segment or
another Coin1. Release before natural completion cancels the reserved Full
without refund, preserving the ordinary terminal spend and exit recovery.
Invalid reservation counts, unsupported owners and insufficient current stock
diagnose; there is no implicit recharge wait.

This extension also admits positive short holds. Release at or before reached
Loop entry pays both committed No/Half segments without entering a loop,
drawing principal or terminal cash-out. Their full connected recovery is
2329791us under the selected clocks; one
microsecond early rejects. A separately paid buffered third input retains its
Full gate after such a release, without fabricating an unrequested loop pulse
or cost. A reserved third stock otherwise cancels without refund. Release
after Loop entry retains its actual loop/terminal spending and exit recovery.

For a two-stock hold, a later tap may reserve its third stock after the live
No Skill progress632043 or Half Skill progress669036 gate, including actual
owner pause, or during the open loop. It buffers until Loop entry+268039us if
that Jump gate has not been reached. Existing loop pulses through that gate
retain their actual spend. At the transition, cancel future loop admissions
without terminal cash-out and preserve remaining own/loan availability.
Duplicate pending finish requests reject without another reservation. The
actual later command owns Full; the initial hold is not silently rewritten.

Held Full emits four ordinary primary requests at progress
252173/751256/865237/949538us, recovery1499861us plus actual three delayed
Skill4 pauses. Its full connected recovery is1739861us. It grants the native
7.701 raw Charge/12.799 Cycle once per nonempty request and no ordinary Grain.
The hold retains only its initial financial snapshot. Same-command terminal
Coin2 captures survive the natural Full transition; source defeat cancels
pending Full damage while preserving paid stocks, principal and refill.

Independent checks cover one/three-dummy three-stock trains, exact Full
contacts/recovery, No's pause-integrated954108us Skill gate (one microsecond
early rejects), Half/Loop buffering, duplicate inputs, cancelled reservations,
source death and cost paid despite immune recipients. A2/A5's same-time
terminal1218.75 spend retains Coin2 base3654 when Full replaces its clock.
Natural full-three recovery is7786330us from the initial input. A buffered
third tap at954108,2000000 or2400000 starts Full at2581175, after the first
loop spend; a later2600000 tap starts then and never consumes the remainder.

The legal A0 native two-period check uses a three-stock train after its first
Normal chain and an actual later third tap after the next Normal chain. It
retains75 packets per dummy, energy77.5224, Cycle100 plus.394 carry/7 overflow,
final drawn debt12890.625 and the actual initial/later Full command IDs5/12.
Stock refills remain8/11/14 then23/26/29s; the later tap does not restart the
deadline. Native aggregation/replay checks one/three dummies. Secondary
actions, market conversion/price/settlement and final full-kit acceptance
remain pending; additional source controls require their explicit owner admission and lowering support.

### Executable ordinary montage cancel gates

`chiz.action_cancels_v1` extends the ordinary/pressed clocks with actual next
Skill and Jump gates. No/Half next-Skill progress632043/669036, Normal's five
Jump coordinates and ordinary/held Full Jump gates read the source cast's
live tracker. Actor pause is not replaced by a fixed maximum delay, and
Jump does not grant an airborne/height assumption or a plunge.

A new pressed Skill inherits any still-live owner pause window. No's legal
954108us Skill gate occurs inside the second pause, which ends996258us.
The remaining42150us at rate.1 loses37935us of the new Half's montage progress.
Its first HP request therefore reaches1591936us, and its Skill gate1661079us.
Financial captures, interest and recharge remain real-time callbacks.

Ordinary montage replacement/Jump cancels unreached Skill notifications:
Half cancelled at its Skill gate loses its still-future966861us primary.
These are two direct montage requests, not a previously released actor;
there is no fabricated late extra Half hit. Field exit retains the distinct
selected committed-Skill afterimage policy. Normal Jump removes unreleased
future requests and retains any already released finite Normal4 producer.
New inputs reset the corresponding combo pointer without resource refunds.
Held third-charge buffering retains its own gates and takes precedence.

Independent Rust checks retain exact inherited-pause contacts, one/three
dummy channels, cancelled Normal2 pulses, one Grain grant per stage, Jump
reset and per-input money ownership. A fast three-press sequence at
0/954108/1661079us produces seven primary requests and3401347us full recovery;
one microsecond early rejects. Legal native two-period checks preserve32
packets per dummy after one funding Normal chain, energy51.6024, Cycle71.396
and unchanged2250 own Grain, with native aggregation and seeded replay.

### Executable full-kit acceptance

The selected owner model binds ordinary/Barren/plunge/Riposte/qualified QTE,
paid Dodge, pressed and held2/held3/buffered Full Skill, Burst actors, exact
Grain/loans/interest/market, awakenings, Hoarding and actual Hanahaki Charge.
Earlier prerequisite receipts remain partial receipts; this acceptance uses
their independent Rust/native checks together with the continuing legal A0
market baseline and the final invested-afterimage integration below.

The initial held No Grain avatar segment owns its actual pause-aware switch
recovery (1.279881s with both connected source pauses). The committed Half/Loop
afterimage has a separate clock and cannot extend that switch gate. A new
explicit Full input still pays its own recovery. This is the checked selected
dual-DPS dispatch policy; no native switch notify is inferred. One microsecond
early still rejects. The original paid held train retains its owner/source
identity after exit; reserved Full neither pays a fourth stock nor repeats the
initial financial settlement. Field exit repays invested principal immediately
unless a transaction is already reserved; later loop spends use live remaining
own/loan availability without creating a second credit line.

Independent checks cover a switch at5.779881s after Burst0/held3 at4.5s:
15000 invested becomes20700 at the captured+.38; repayment leaves5700 savings,
then16 pulses spend1293.75 each, emit3*floor(1293.75)=3879 Coin2 and finish
with15000 debt. The same train still emits its four paid Full requests off
field. Switching later at8s instead repays14231.25 after five loop costs;
only that much loan capacity reopens for the eleven remaining pulses.

The legal native two-period scenario keeps this carried debt, repays13500
through six actual Normal chains, then invests/settles18630 in its second
period. Across75 real seconds it has166 HP packets, own0/debt15000 and exact
loan granted58500/repaid43500; sixteen later Coin2 bases are3492 each. Native
aggregation/replay preserves both exits, fixed amounts and continuing state.
Removing any required clock/financial/Skill model diagnoses rather than
reporting successful incomplete DPS. Bounded current WASM/Worker smoke and
lossless report transport cover the shared boundary; final full browser
acceptance remains browser integration, and no observed-game calibration is claimed.

### Executable A5 battle entry and exact deposits

`chiz.battle_entry_v1` binds explicitly authored, alternating combat leave and
re-entry events. Initial entry has ID0; every later callback uses its stable
event ID. A5 grants the level/A1-adjusted capacity once per entry, without
A4/R3 acquisition efficiency. Repeating an already claimed callback grants
nothing; ordinary field switches do not create battle entries. Leaving combat
does not reset loans, stocks, Energy, Fons, buffs or financial reservations.

The selected deposit routing follows the current financial mode. Outside the
market, it repays debt before crediting capped own Grain. During investment,
market-forced Surplus credits the invested wallet up to C+L, preserving the
captured transaction and deferring principal repayment to actual market exit.
This mode-aware interaction is simulator policy, not a recovered native body.
Each receipt exposes exact decimal-string rational `input`, `repaid`,
`credited` and `overflow`, with input=repaid+credited+overflow. Empty deposit
maps remain omitted for existing consumers.

Independent Rust/native checks drain30000, repay3825 through a live Normal
chain, then verify an unmultiplied15000 entry repays11175 and credits3825.
The next entry credits11175 and discards3825. Investment checks credit2400
and discard12600 without repaying early or changing the captured27600 Coin1.
Native batch/replay and JSON/binary adapter checks retain the full receipts.

### Executable market transaction and lifecycle edges

The market Burst now uses its actual owner montage clock. Only the finite
actor train released at2.346949s survives a living replacement; defeat cancels
unreached actor packets/market entry. Jump respects3.754213s recovery and the
independent4.25s switch lock remains operative. Market/chart deadlines cancel
on exit, rather than leaving ignored callbacks past a short scenario horizon.

Expiry closes new financial admission immediately. A reserved10.3s Skill input
in a market starting3.420264s retains its+.23 price, settles18450 at10.5s and
then repays15000, leaving3450 own Grain. Its captured critical Coin1 still
delivers after exit. An observation ending10.45s keeps the unpaid reservation
and invested wallet, reports `SCENARIO_HORIZON_PENDING` and cannot aggregate as
a completed run. Defeat instead cancels the reservation and repays current
principal without a postmortem revaluation/coin or resource refund.

The first-critical token is consumed by actual positive Coin1 HP loss. An
all-miss request retains it; an immune first recipient cannot steal it from
the other recipients of the same captured packet. Zero proceeds retain the
token. The zero-wallet/A1/A4/R3 check later acquires540 while market-forced
Surplus, settles572.4 at+.06 (Coin1 base572), and exits with15927.6 debt;
applying the Debt1.7 acquisition multiplier there would fail this check.

Rust native replay checks deferred repayment and two generated A3 markets
with continuing seeds42/43 and recipient-count-invariant charts. Each market
uses28 draws. Normalized sign weights are exactly.5/.6; the finite32-bit
uniform-word comparison has less than one-word probability quantization,
and conditional floor selection retains repeated source support entries.
This is explicit simulator RNG policy, not a recovered native statistical law.

### Executable Barren recipient split and cue/limiter

`chiz.barren_v1` binds the reached Branch2/Branch2_1 route. An explicit
zero-duration Normal hold selects tap: its .500015s non-damaging cue precedes
the train and starts one independent2.5s limiter. Positive Normal hold intent
selects combo Barren through the preceding Normal's live AnySkill gate and
two-second continuation; it adds no invented numeric hold delay. Actual held
Skill duration/reserved stocks remain separate inputs.

First phase captures the live lock or lowest living fallback at.749310s,
before HP delivery. Branch2 hits that one recipient; Branch2_1 at.750524s
excludes the captured lock even when the first hit killed it. Final AoE at
1.464538s hits all remaining living dummies. The train recovers1.912478s;
Jump opens.868260s. An input at cue completion can cancel before automatic
train entry. All unreached requests/cue transitions cancel on replacement or
exit, retaining the paid tap limiter and any actual grants.

One dummy gets two nonempty requests; two or more get three, independently
of recipient count. Rust tests check their exact native Charge/Cycle values,
one450-Grain action grant, fatal-lock retention, dead-lock fallback and
cue/held cancellation. Two legal native tap periods over two dummies emit
eight HP packets, receive34.56 Energy, accrue48 Cycle and acquire900 Grain
through aggregation/replay. Both new typed source rows retain only consumed
coefficients/channels. The unlocated Branch_Damage packet and account-Fons
farming remain excluded by the declared combat scope.

### Executable secondary actions and qualified Support

`chiz.secondary_actions_v1` now binds positive authored airborne landing/height,
paid front/back Dodge and one next-Normal Critical Riposte. Plunge contact is
landing+.199456s with .500015s base recovery; connected pause adds.1485s.
Height changes its coefficient alone. Riposte's two independent native requests
land at.150950/.333387, grant900 total level80 Grain once across recipients,
and finish after1.210147s when the second request connects. The real15-stamina
Dodge cost and.9s cooldown remain ordinary shared admission constraints.

Qualified Support consumes the outgoing100 Cycle through the real switch and
hits at.800160s. Base recovery1.736720s plus the independently integrated
Hermite loss160558us gives1.897278s; admitted gates are after the actual curve,
so no within-curve interpolation gate is fabricated. Support grants8.6 raw
Charge, no Grain/Cycle. A positive-HP causal request pauses once over all
recipients. Already connected delayed pauses can affect the same owner's next
avatar action after a legal cancel; detached Skill afterimages retain local
clocks and do not freeze an incoming teammate. Exit/defeat clears authorization
and unreleased owner requests.

Rust checks include exact gate rejection/acceptance, height/resource separation,
missing stamina, one-use Riposte, real Support qualification and a Riposte pause
whose onset occurs after replacement Skill admission. A legal native build
executes two continuing Dodge/Riposte/plunge periods:30 stamina spent,2700
Grain and16.32 received Energy, preserving seeded aggregation/replay. Barren,
combat-entry A5 deduplication and final market-edge acceptance have their separate checks below.

### Executable finite market and continuing native finance

`chiz.market_v1` extends the exact-finance/pressed models with the selected
seven-second investment, six native Burst packets and one input-price
transaction per actual Skill activation. Burst recovery is3.754213s, market
entry3.420264s, switch lock4.25s and cooldown15 World seconds. The two newly
consumed typed Burst rows retain all13 coefficients, zero Charge/Cycle and
their own Break values; no source payload is stored.

Market principal has its own conserved `chiz.market` ledger, capacityC+L,
beside `chiz.grain` and `chiz.loan_debt`. Captured signed return settles after
.2 World seconds, floors only the damage override and retains fractional
principal. Chart updates do not compound funds. First positive Coin1 captures
one critical flag for its whole recipient group; subsequent money requests
retain the fixed no-crit encoding. A2 credits market principal once per real
second, with market expiry preceding the seventh coincident tick. Repayment
keeps residual debt; investment never creates another credit line.

The finite generator uses28 draws from a continuing owner-local PCG stream
per entry, separate from critical rolls and recipient enumeration. Explicit
`chiz.recorded_market_trace_v1` selects the14 recorded source returns as an
authored scenario override while advancing those same28 draws. It makes no
native-RNG or A3-statistics claim. Generation/cursor, switch/death transaction
edges and remaining secondary actions retain their separate acceptance checks.

Rust native acceptance executes the legal A0 build and two75s periods specified
above:276 packets, final own0/debt12750/stocks3, Energy49+.1624 and
Cycle100+.170. Independent expected damage is3424063100431/5000000
(684812.6200862), reconciled from packet bases/live gear zones and preserved
through seeded native replay. This is executable selected-model coverage,
not observed game calibration or full-kit acceptance.

## Ordinary Normal chain (retained caller-timed model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,3,1,5,2 (twelve hits: Melee1, 3x Melee2, Melee3, 5x Melee4, 2x Melee5). The chain loops, retaining native effective talent projection with Normal attack excluded from resonance_3 level bonus.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`chiz-normal-chain.json`](../../../../reviews/q10-game-action-binding/chiz-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`chiz.skill_animation_profile_v1`, `chiz.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Chiichan073_Skill_1_Short`), gameplay events land at 0.183s (`Skill.1_1`) and 0.600s (`Skill.1`); cancel windows activate at 0.632s (`BreakSkill_Skill`), 0.714s (`BreakSkill_Jump`), 0.920s (`BreakSkill_AnySkill`), and 1.232s (`BreakSkill_Move`) with total duration 3.817s. For Burst (`char_rabbit_ultraskill_1_Montage`), gameplay events land at 2.347s (`UltraSkill.3`), 2.420s (`UltraSkill.2`), and 3.420s (`UltraSkill.1`); cancel windows activate at 3.754s (`BreakSkill_AnySkill`, `BreakSkill_Jump`) and 4.256s (`BreakSkill_Move`) with total duration 9.133s.
- **Multi-target delivery (`chiz.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Chiz's primary `GE_Player_Chiichan073_Skill1_1Damage` packet across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

- GA_Chiichan073_Melee admits GE_Player_Chiichan073_Melee1_Damage Native coefficients at Normal levels 1 and 8 are 3.257 and 5.580 across the chain; category A/COSMOS metadata stays source-bound. Normal attack is excluded from resonance_3 level bonus. See chiz-normal-chain.json.

## Action packets and owner effects

- GA_Chiichan073_Skill public primary packet is GE_Player_Chiichan073_Skill1_1Damage. Category E/COSMOS metadata stays source-bound.
- GA_Chiichan073_UltraSkill public primary packet is GE_Player_Chiichan073_UltraSkill1_Damage. Native price/settlement model is pending and rejects at boundary.
- Resonance 3 activates at awakening >= 3 and grants Grain acquisition efficiency buff; no SkillLevel modifications exist in 1073AwakenEffect.json.
- Canonical arbitrary awakening selections: Effect1 capacity, Effect4 acquisition and Effect5 initial Grain use selected identities. Normal resonance_3 has no level bonus; target Cosmos resistance remains count-six resonance_6. Existing unsupported owner mechanics remain pending.

## Ordinary-hit Ultimate Energy

The current `chiz.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee16.3 raw units for the listed modeled packet trains. Hoarding makes this19.56 received while active at baseline efficiency1. The modeled distribution is1/1 to the source and3/5 to each living ally, with each recipient's own live efficiency, carry, capacity and overflow. The selected Charge-passive model above additionally binds the current No Grain primary's.966 raw Charge. Caller eligibility, positive-HP-loss gating, delivery-time recipients and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`chiz-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/chiz-ordinary-hits.json).

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/chiz-action-binding-model.json`](../../../../reviews/q10-game-action-binding/chiz-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/chiz_acceptance.rs), [`chiz-normal-chain.json`](../../../../reviews/q10-game-action-binding/chiz-normal-chain.json).
- Frozen inputs: `data/extracted/guide_chiz.json`, `data/extracted/neverness_chiichan.json`, [typed native damage rows](../../../../crates/data/src/native_damage/chiichan073.rs), `1073AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- The Skill has separate charge-consuming No/Half/Full Grain inputs, hold behavior, and Surplus/Debt spending; the current model does not execute those states.
- Grain market updates, loan conversion, price changes, and repayment remain outside the supported action model.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/chiz). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Chiz (`GA_Chiichan073_*`):

| Action / damage row (`GE_Player_Chiichan073_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.171–0.560 / 0.293–0.960 / 0.317–1.037 | 16.300 total / 27.135 total / 3.298 total |
| Plunge `AirAttack1_Damage` | X | 0.900 / 1.542 / 1.666 | 4.500 / 7.500 / 0.900 |
| Hold branch 1 `Branch_Damage` | A | 0.775 / 1.328 / 1.434 | 3.900 / 6.450 / 0.800 |
| Hold branch 2 `Branch2_Damage` & `Branch2_1_Damage` | A | 0.960 / 1.645 / 1.777 | 4.800 / 8.000 / 0.950 per packet |
| Critical Riposte hit 1 `PerfectEvadeAttack1_Damage` | Z | 0.300 / 0.514 / 0.555 | 0.767 / 1.267 / 0.833 |
| Critical Riposte hit 2 `PerfectEvadeAttack2_Damage` | Z | 0.600 / 1.028 / 1.111 | 1.533 / 2.533 / 1.667 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 8.600 / 0.000 / 2.500 |
| Skill `Skill1_Damage` & `Skill1_1Damage` | E | 0.400 / 0.686 / 0.740 & 0.190 / 0.326 / 0.352 | 2.034 / 3.322 / 0.407 & 0.966 / 1.578 / 0.193 |
| Skill coin bonus `Skill1_CoinDamage` & `Skill2_CoinDamage` | E | 0.590 / 1.011 / 1.092 | 0.000 / 0.000 / 0.000 |
| Burst `UltraSkill1_Damage` & `UltraSkill2_Damage` | Q | 0.446 / 0.764 / 0.826 & 0.447 / 0.766 / 0.827 | 0.000 / 0.000 / 0.134 per hit |
| Burst finisher `UltraSkill3_Damage` | Q | 2.771 / 4.749 / 5.129 | 0.000 / 0.000 / 0.831 |

Critical Riposte uses GAName `GA_Chiichan073_PerfectEvade` (2-hit train), while Support QTE uses `GA_Chiichan073_QTE`. All damage rows deal COSMOS damage. Grain coin damage rows (`Skill1_CoinDamage`, `Skill2_CoinDamage`) supply bonus damage scaling with spent Grain.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Chiichan073_Passive_1` binds `Buff_Chiichan073_Passive1`. `DT_ReactionExtendDataTable` row 1073 binds `ReactionAbilityID=GA_Chiichan073_Passive_1`, providing Chiz's reaction enhancement.
- **Passive 2 (Breakthrough 4):** `GA_Chiichan073_Passive_2` binds `Buff_Chiichan073_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1073AwakenEffect#/rows/1073/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Increases storage capacity of Grain and Grain Loan by +10%.
- **A2:** Grants 10% of current Grain as interest per second while Chiz is in Surplus state.
- **A3:** Increases the chance of a Grain Price increase by 50%.
- **A4:** Increases Grain acquisition efficiency by +20%.
- **A5:** Grants Chiz Grain equal to 100% of Grain storage limit upon entering combat.
- **A6:** Increases Bonus Cosmos DMG of Grain by +15%.
- **Resonance 3:** Increases Grain acquisition efficiency by an extra +50% while in Debt state. (Notice: resonance_3 grants no skill level additions).
- **Resonance 6:** Reduces enemy Cosmos resistance by 10%.

### Passive description coverage, reviewed October 18

Saved description source: `guide_chiz.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Hanahaki Disease describes four additional Charge for the active character; Hoarding describes self Charge Efficiency +20% while on field. The first is a Charge enhancement, not the generic Cosmos reaction previously inferred from legacy asset names. Native activation recipient/frequency, field-entry/exit efficiency and interaction with Grain/Loan events remain with their separate consumers.

### Hoarding efficiency source recovered, October 18

The initial `Buff_Chiichan073_Passive2#/1/Properties/Modifiers` adds **ChargeGetEfficiencyBase** through multiplier 1 and `DT_ChiichanEffectFigure:Chiichan_Pas2=0.2`. The carrier is Infinite, AggregateByTarget, limit 1, NeverRefresh/NeverReset. The source stat and 20-percentage-point contribution are located; it is not a flat four-energy grant, Grain acquisition or an independent 1.2 multiplier on an already increased efficiency. The description's on-field restriction must follow actual carrier/stat eligibility: this object alone declares no field-state predicate.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Chiichan073_Melee#/6; Melee.4 | DamageActorChiichanMelee4 | (0.0, 0.0, 0.15, 0.8, 5, true) | GE_Player_Chiichan073_Melee4_Damage |
| GA_Chiichan073_UltraSkill#/10; UltraSkill.3 | DamageActorChiichanUltraskill1 | (0.0, 0.01, 0.1, 8.0, 4, true) | GE_Player_Chiichan073_UltraSkill1_Damage |
| GA_Chiichan073_UltraSkill#/10; UltraSkill.3 | DamageActorChiichanUltraskill2 | (0.0, 0.49, 1.0, 1.0, 1, true) | GE_Player_Chiichan073_UltraSkill2_Damage |
| GA_Chiichan073_UltraSkill#/10; UltraSkill.3 | DamageActorChiichanUltraskill3 | (0.0, 0.98, 1.0, 1.0, 1, true) | GE_Player_Chiichan073_UltraSkill3_Damage |

UltraSkill.3 requests three actor entries with different packet classes: the first has period 0.1/cap 4 after delay 0.01, and the two single-count follow-ups have delays 0.49 and 0.98. The serialized actor lifetimes 8/1/1 do not define the Burst recovery or real Time Slomo duration. Ordinary Melee.4's period 0.15/cap 5 is a separate train.

### Branch secondary-target exclusion recovered, October 18

Warning-free **DamageRange_Chiichan073_Branch2:K2_CheckCanSelectTarget#/2** casts OwningActor to player and returns **owner.GetAttackTarget() != TargetingActor** on that path. It is actually selected by **GA_Chiichan073_Melee#/5/EffectContainerMap/8**, **Branch.3**, carrying **GE_Player_Chiichan073_Branch2_1_Damage** and Branch_Traction. The preceding **Branch.2** instead uses native TargetType_AttackTarget with **Branch2_Damage**. Thus the located secondary Branch container explicitly excludes the current lock target; these two containers cannot automatically be assigned together to one dummy. A failed owner cast leaves an unassigned return; native selection order, target changes, no-lock fallback, recipient cap and Branch.4's separate range remain open. Sources: `remaining-action-target-callers.txt` and `remaining-action-target-config.txt`.

### Branch weapon actor's montage callbacks checked, October 18

Warning-free **Chiichan_Branch_Actor:ReceiveBeginPlay#/8 → ExecuteUbergraph#/2** calls Actor's parent then creates a play-montage proxy for **Weapon_char_rabbit_skill_branch_Montage**. Completed, BlendOut, Interrupted and NotifyBegin/End callbacks (`#/3..7`) only store their received NotifyName and return. They request no effect application, packet cancellation, resource mutation or actor destruction. Thus the spawned weapon actor does not itself recover another Branch damage event or a completed/interrupt cleanup policy; the owning ability's separate Branch.2/.3/.4 containers and native montage notifies retain their own callers. 
## Unknown execution policies

Historical native-certainty limits only. Their selected implementation decisions are complete above and in the financial owner; they do not block or reopen collection.

- **Real timing/admission:** Montage notify coordinates for Skill (`Chiichan073_Skill_1_Short`) and Burst (`char_rabbit_ultraskill_1_Montage`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Grain economy state machine:** Native cap/acquisition curve reads, price cursor generation, conversion/repayment order, and Surplus/Debt state mutation policies remain unrecovered native logic (see [Grain and market source findings](grain-and-market-source-findings.md)).
- **Spatial geometry & targeting:** Skill AoE and multi-target Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Resolve native cap/acquisition lookup coordinates and application, Grain/loan mutation, price generation/cursor/weight normalization, conversion/repayment order, Skill charge refill/admission and Surplus/Debt transitions. The [source owner](grain-and-market-source-findings.md#global-cap-and-acquisition-curves) now binds global cap/acquisition curves in addition to configured charges, market sequences, loop reads and the settlement guard. Those values and source locations are resolved; per-input/per-packet grant eligibility, effective cap updates and native mutations are not. `ChiiChanLogicObj:SettleStock`, `SetRecStockPriceEnable` and `CostAndRecordValueForCurCoinDam` still own unproved state mutations and rounding; the last function's second argument must not be assigned invented semantics.
- Reconcile the described interest cadence with the configured curve/default/native timer path. Establish the first market price/index, the repeated curve's actual use, first critical settlement, held price capture and switch/end/death behavior. Omitted multipliers/initial fields stay unknown. Source names or the 14/28-point configuration do not prove a 50/50 independent RNG or a fixed replay sequence.
- Ordinary-hit energy applies only to the modeled Normal chain; no Skill train is inferred from successive input variants. Native caller eligibility remains a tested hypothesis.

## Prior findings

- [`reviews/q10-game-action-binding/chiz-action-binding-model.json`](../../../../reviews/q10-game-action-binding/chiz-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/chiz_acceptance.rs), [`chiz-normal-chain.json`](../../../../reviews/q10-game-action-binding/chiz-normal-chain.json).
- The October 16 resource investigation preserves player-object/graph findings and 48-field native schema comparison.
- Literal write scan in  confirmed that no explicit `EX_Let` writes to `CoinMax` exist in serialized JSON, establishing that cap mutation is managed by native C++ runtime setters.

## Research history

- Chiz8/8 selected handoff complete. Next campaign owner: **Daffodil, packets block**, reusing existing dossiers before acquisition.
- Already tried native exports/schema/AST/global curves, cue/custom-executor/actor/montage reads, public/frozen kit and prior source investigations. Native writer/market-cursor/interest-body questions have explicit checked decisions now; do not restart them during collection.

## Relative combat timer binding

`char_rabbit_ultraskill_1_Montage` Common.EndAbilitySkill at 4.250937 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
