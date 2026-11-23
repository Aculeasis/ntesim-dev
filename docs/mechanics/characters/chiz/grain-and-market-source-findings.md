# Chiz Grain, hold and market source findings

Native identity: `character:1073`, `Player_073_Chiichan`, `ChiiChanLogicObj`. Responsibility: configured Grain/market inputs and reached Skill resource/settlement graphs. **Source facts** below describe the inspected PC_140/logic-566459 client; frozen-cutoff applicability and complete native resource execution remain open.

The acquisition receipt identifies a portable cache bundle and native metadata comparison. The [action card](action-damage.md#executable-full-kit-acceptance) owns the accepted selected executable kit. This finding records evidence and model limits, rather than replacing its executable checks.

## Selected complete financial model

**Mixed, October19 collection decision.** The native configurations/graphs below remain source facts and the missing mutation bodies remain unobserved. The following explicit model closes those implementation choices after the prior client/schema/AST and public-description approaches. It does not claim the native price generator was recovered. The [action handoff](action-damage.md#selected-collection-handoff-october-19-2026) owns packets and real-time scheduling; do not restart protected-body searches or use the old “blocked” continuation as a reason to omit this mechanic.

### Balances, acquisition and spending

Public source `start_grains` and `start_loans` declare exact own savings O and drawn principal D before combat-entry grants, each within its level/A1 capacity (default 0). Either declaration binds the existing financial/pressed/clock/market models before initialization; the same dependencies are used by public Burst. Explicit zero remains present in inspection and saved inputs. Native requests with these overrides must declare the same model dependencies. Starting loans are principal already drawn, not remaining borrowing capacity and not a credit to the authored own savings.

`.chiz.resources.grains` reads the current native own/market balance; `.chiz.resources.loans` reads drawn principal. `.chiz.debt.active`, `.chiz.surplus.active`, `.chiz.market.active` and remaining `.chiz.mods.market` seconds use native predicates/session deadlines. Reads never draw prices, perform a transaction or consume RNG. Loan/state queries require the financial model, declared by an ordinary Burst or explicit starting balance. Wide exact interest remains rational through the DSL's 4096-bit bound; it never passes through `f64` or an intermediate `u128` conversion. Shared Fons is unchanged by these resource accounts.

Initialization seeds both accounts before applying the same entry ID 0 deposit writer used for subsequent battle entries. A5's entry grant repays drawn principal before crediting capped own savings, without A4 acquisition scaling: at level 80 with only A5, O=1 and D=100 become O=14901/D=0; O=1/D=0 ends at 15000 with 1 overflow. Numeric source checks also retain forty exact 10% credits from 0.000001, debt-suppressed interest, 7s market expiry, repay-first exit from O=500/D=100 to O=400/D=0 without trades, full native replay and unchanged query/RNG output. The executable [finance transport fixture](../../../../presets/native/finance-control.nte) is shared by native, WASM and actual Worker checks.

Select global cap/grant curve coordinate by **character level**, constant-left-key interpolation and endpoints, not talent/awakening level. At80 ordinary own-Grain capacity **C=15000**, loan capacity **L=15000**; at1 both500. A1 multiplies each by1.1 (80→16500 each). Keep exact rational balances: own Grain **O>=0**, already drawn/outstanding loans **D in[0,L]**, and market wallet **W** only while invested. Normal outside-market capacity is O<=C. Available Grain is **A=O+L-D**, exactly the reached loop expression; borrowed Grain is not Ultimate Energy, Fons or a source of interest debt. Fresh combat without A5 starts O=D=0, hence A=L remains borrowable. Preserve D across rotations; leaving combat does not silently repay/reset it. Never read LoanedCoin as remaining capacity.

Chosen acquisition dispatch is **once per connected Normal stage input**, at its first positive HP packet, from the corresponding CoinMelee1..5 curve. Actor Melee4's five pulses still earn one stage grant. Each level80 stage grants450, full five-input chain **2250**, not12*450. Barren earns450 once per whole branch despite its split recipient containers; plunge450 uses the common same-valued level grant (inferred from its described Grain generation and the nine equal acquisition curves). Riposte's two distinct CoinEvadeAttack1/2 rows grant450 each on their respective connected hits, total900. QTE, Skill damage, Burst actors and Grain bonus damage grant no ordinary Grain. No per-dummy multiplication. Normal/Branch/plunge/riposte action IDs persist through delayed delivery so a second actor pulse cannot rearm the same grant.

Battle acquisition multiplier is **1 +(.20 if A4)+(.50 if R3 and Debt before this grant)**, additive. Debt means D>0 outside market; the entire grant snapshots that state before deposit, even if it finishes repayment. Deposit pays `p=min(D,grant)` first, then credits O with the remainder up toC, records discarded overflow. At D100/grant450/R3 only, receive675, repay100, own575; do not use1.5 on only the first100 and change multiplier mid-grant. A4/R3 do not multiply loan conversion, market revaluation, A2 interest or A5's capacity-sized entry grant. Loan interest rate is0. A5 ENTER_BATTLE grants C once through the current financial mode's deposit, without another acquisition multiplier; repeated cue callbacks deduplicate the battle entry ID. Outside investment this repays debt before crediting own Grain; market-forced Surplus instead credits the invested wallet and retains exit-time repayment. This interaction is the selected mode-aware routing policy; see [executable battle entry](action-damage.md#executable-a5-battle-entry-and-exact-deposits).

`spend(x)` captures **s=min(max(x,0),A)**, consumes `min(O,s)` own Grain first, then increases D by the rest, never aboveL; records exact s as CurCoinDamValue. Spend is once per Skill coin event **before** parent damage, regardless of whether the target later evades/is immune; no positive-HP refund or per-target charge. Example O20,D40,L100 gives A80, captured loop cost5, first spend leavesO15,D40; using only O/16 would wrongly pay1.25. O0,D98,L100 with requested5 spends2 and endsD100; flat loop base is6, not15. Outside deposit and post-market repayment normalize the usual own-versus-debt ledger; explicit intermediate source-field fixtures can still read both O/D without mutating the reached expression.

Pressed Skill while outside market is **non-consuming**: in Surplus (D=0) snapshot current O into LastSettleStockResult and emit the eligible Coin1 bonus; in Debt no Coin1 bonus. This selects the explicit zero PressCostCoin and the independent description that consumption belongs to **held** Skill, rather than inventing an all-wallet press tax. The ordinary Skill ATK packets always execute. Held No Grain can emit that same Surplus snapshot once without consuming it; its Half Grain loop then performs the actual own/loan spend. No ordinary Skill packet automatically draws loans merely because a zero-valued section settlement entry exists.

At Loop/LoopHold entry capture **LoopCostVal=A/16 once**, with A replaced by W in an active market because all loan capacity has already been converted. Each of the16 source loop events spends min(capturedCost,currentAvailable), then its Coin2 uses `3*floor(actualSpend)` (outer floor retained). Do not recompute a geometrically shrinking A/16 each tick: initialA16000 must pay1000 sixteen times, not1000,937.5,... leaving a phantom remainder. All16 pulses may still deliver their ordinary .428ATK packet at zero Grain; zero actual spend produces no positive money packet. Normal release/natural loop completion consumes the remaining available Grain once as terminal Coin2; **early third-charge finish consumes no remainder** and goes to held Full Grain without a new Coin1. Exact rational cost carries through fractional balances; floor only at the located damage-base override, not by discarding resource fractions after every spend. The native second argument1 is treated as a call discriminator, not an invented second one-Grain fee or another multiplier.

### Market entry, price capture, settlement and exit

At Burst's UltraSkill.1 progress3.420264, draw **R=L-D** remaining loans, setD=L, move `W=O+R`,O=0 into the market, and enter Surplus/Market for7 World seconds. Temporary invested capacity is **C+L**: converted loans may coexist with own savings, unlike the ordinary own wallet capC. This finite inferred capacity preserves both caps and the described conversion without discarding the entire newly drawn loan. During market treat CurrentCoin as W, so `(L-D+CurrentCoin)/16 = W/16`; no second invisible credit line. A market acquisition/interest credits W up toC+L, records overflow, and is not an immediate loan repayment. Loan balance remains explicit. Surplus is forced by the market mode even though D=L, so its A2 eligibility and R3 Debt efficiency cannot both be applied to the same grant.

Every pressed stage records its **input-time** price r and one pending settlement; held Skill records only its first activation price/one settlement for the whole hold. The native guard interval.2 becomes a scheduled financial event at capture+.2 (inclusive), provided the section is one of the reached section-map keys; clear its recorded flag/section exactly once after settlement. Do not poll or repeatedly settle LoopHold at every damage pulse. Financial capture is independent of damage delivery, so an input at r=.25 still uses.25 if the chart changes before its delayed hit. This also explains why the described held route refers to its first activation. Coin1 delivery uses that cast's recorded result, never a global value overwritten by a later cast.

Selected `SettleStock` semantics: r is a **signed return fraction**, not an absolute Fons price. At settlement revalue the invested wallet to `P=max(0,W*(1+r))`, credit **W'=min(C+L,P)** and record `P-W'` as market overflow; **LastSettleStockResult=W'** (actual settled proceeds, not profit-only and not a separate ATK coefficient). The section's source argument0 means no extra withdrawal, not disabling settlement or loop costs. Exact balances may be fractional, while Coin1 base floors only this captured W'. This is inferred from “settlement result”/remaining invested Grain and the flat-base reader; the native writer is not recovered. Check: W100,r-.08 gives proceeds92 and bonus base92, never negative damage or a92-Grain loan charge; W100,r+.25 gives125, not25 profit-only. At limit200, W180/r+.5 gives200 recorded proceeds plus70 overflow, not270 damage from uncredited overflow. Repeated settlement compounds only when **a real new Skill input** admits it, not at every chart update.

On expiry, switch, valid recast replacement or explicit market end, repay `p=min(W,D)`, then credit outside own `O=min(C,W-p)` and keep `D'=D-p`; record own overflow. Stop market/chart/interest mode, retain charge/Skill stocks/loan debt and queued attack packets. Already admitted price transactions due within.2s are completed once at their reserved financial deadline **before deferred repayment**, then the market closes; no new market input is admitted after exit. The TimeSlomo carrier still removes immediately on switch; a finite reserved financial transaction does not freeze the incoming actor or whole time axis. Source death instead cancels unresolved price transactions and repays the current wallet without creating postmortem damage. Scene end records balances; a reporting horizon alone does not invent market expiry. Coin packets already captured use their own stored amount/first-crit flag even if the later live wallet is0. Example W180,D150,C150 exitsO30,D0; W100,D150 exitsO0,D50. Erasing D at exit would fabricate50 Grain and invalidate the next cycle.

### Finite price model and interest

Use the source14 values as a finite return support, preserving multiplicity: **.25,-.08,.38,.50,.14,-.29,.19,.06,-.28,.32,.25,-.08,-.32,.23**. MaxRise/MaxLose50 bound these fractions by±.5; do not interpret.25 as25 Grain or compound the *price* by1.25 on each chart tick. Source weights50/50 and A3 additive-to-rise-weight ratio.5 select sign probability `p_up=50*(1+.5*A3)/(50*(1+.5*A3)+50)` = **.5 normally/.6 with A3**, not guaranteed rise or+50 percentage points. Conditional on sign select uniformly among its retained positive9/negative5 entries with the shared seeded RNG; duplicate.25/-.08 remain duplicate support entries. Use one sign draw and one within-sign-index draw per price, index=floor(u*n). This checked **inferred finite generator** reconciles the named weights, bounded configured returns and absence of serialized minimums without inventing a continuous range or treating a14-entry sample's9/14 positive fraction as the universal chance. It is not a claim of the native pseudorandom cursor algorithm.

Precompute14 prices at market entry from the continuing RNG stream (`AmountOfStockPriceGenerateOnInit=14`), index0 immediately then1..13 at+.5..6.5. `AmountOfStockPriceGenerateOnUpdate=0` means do not regenerate a new array every.5; expiry7 precedes an index14 update. A new Burst advances the continuing RNG, not reset to its previous seed. For portable deterministic scenario comparison an **explicit recorded14-price trace** may replace these draws; the source order above is one declared trace, not evidence that every native market begins+.25. Mark that override as authored and do not use it to assert A3 statistical behavior. Probability check is exact .5 versus.6, all possible outputs are native support values within±.5, and replay consumes a fixed28 draws/market. This choice completes the model after the prior two-plus approaches; no missing native cursor blocks implementation.

A2 adds **10% interest once per real second** while Surplus with positive O (or market W). Interpret the configured.5 update as two half-second bookkeeping checks; only the full1s boundary credits `.1*liveBalance` once, not.1 twice or compounded5% at both halves. First interest is Surplus-entry+1, subsequent+1 while state persists; leaving Surplus clears its fractional timer, not saved balances. No interest without A2 or in ordinary Debt; market forced-Surplus may earn it without charging loan interest. Apply normal own/market cap and overflow, no acquisition multiplier. Retain exact interest until a damage override's floor. Example100 gives110 after1s, not121 or110.25, and121 after2s absent other changes. Natural market expiry at a shared interest boundary closes mode before a new interest credit, avoiding an eighth interest tick. Half-second bookkeeping/chart updates are not independent Grant Grain events.

## Source identity and decoder scope

The consumed A1 capacity, A4 acquisition and R6 resistance parameters and the metadata-only7s market duration are typed in [`chiz/parameters.rs`](../../../../crates/content/src/characters/chiz/parameters.rs). The old normalized EffectFigure export and its unconsumed rows are cache-only. The exact-Grain prerequisite below preserves fractional capacity/acquisition without inventing rounding; financial Burst remains unbound.

| Source | Export SHA-256 | Relevant pointer |
| --- | --- | --- |
| `/Game/Blueprints/Character/Player/Player_073_Chiichan` | `7f3fc7650ef760448c36d057b604a2b091800c607002c13533d1ff5d5c8ba1cf` | `/3/Properties`, `ChiiChanLogicObj_0` |
| `/Game/Blueprints/Abilities/Player/Ability_073_Chiichan/GA_Chiichan073_Skill` | `53b0323fb5ae9c16b5a6293284700098eb1b1a343274283a8a738e4e6209ff3f` | `/2/ScriptBytecode`, `/3/ScriptBytecode`, `/8/ScriptBytecode`, `/10/Properties`, `/14/Properties` |
| `/Game/DataTable/Skill/GlobalCharacterData/DT_ChiichanEffectFigure` | `44cb9a28ac744b0a05f1fd8130a67c28a3050c2de2758691ecb3effe53eb7b37` | `/0/Rows/<name>/Keys` |
| `/Game/DataTable/Skill/GlobalCharacterData/DT_ChiichanEffectFigurePseudorandom` | `d17c298ebcfe2222baebbca78c44ddba786cc395e2276f1b69188ab147bb9c93` | `/0/Rows/Para/Keys` |
| English `game.locres` | `e2fcf52ad95ae25f7cf1432800c54ad2b3f1fab18a78f5e4a3f67e49ee1869e1` | `/0/Properties/Chiichan_SkillDes/<key>` |

The `ChiiChanLogicObj` schema contains 48 outer fields plus three map/array child descriptors. All 48 outer names/order match the public schema; the comparison has no added or missing fields. This validates this class's field layout independently of a plausible JSON decode. It does not validate inherited defaults or native function bodies.

The player export retains three errors, on `CapsuleComponent`, `PlayerCharacterMovementComponent`, and the player CDO's `VaultParams`; none names `ChiiChanLogicObj`. Skill and both curve exports have no per-export warnings. Keep the original warnings attached instead of promoting the whole player export to a complete decode.

## Skill charges and captured loop argument

The live source name `.chiz.resources.skill_stocks` reads the native pressed
Skill's whole pool (default/cap3), requiring its selected model rather than
fabricating stocks when it is unavailable. Explicit financial starts and ordinary
Skill inputs declare that model before initialization. Serial3s refill events continue off
field; equal-time queries read the settled refill. `.chiz.hoarding.active` reads
the actual field-entry/exit Charge Efficiency passive, and
`.chiz.out_of_combat.active` reads the distinct combat flag. These queries never
spend, refill, change passive state or consume RNG. [Rust source checks](../../../../crates/content/src/simulation/tests/source_chiz_resources.rs)
cover two continuing real spends and off-field refills at the last microsecond
and exact deadline, field transitions, unavailable-model diagnostics and native
replay. Matching stock-start authoring remains Q4-L4; no Fons/Grain/Loan units
are reused for stocks.

### Source held-stock reservation

`chiz skill[hold=10s,stocks=2]` and `stocks=3` bind the existing native
reservation, separately from the initial stock pool. The Skill descriptor
declares finance/clock/pressed/held/held-Full dependencies before initialization;
ordinary zero-duration taps use the same selected stock model. The source layer
converts a whole count and passes it through; native admission owns the2/3
constraint, available-stock payment, finite afterimage/loop and automatic held
Full. Missing `hold`, invalid numeric types and native capacity failures remain
diagnosable rather than becoming an inferred reservation or resource refill.

[Rust source checks](../../../../crates/content/src/simulation/tests/source_stocks.rs)
reserve two then three stocks across two continuing cycles and two dummies.
The first route has72 packet receipts and no held Full; the second delivers
eight native Full packets. Actual3s refills restore the pool while the15000
loan debt persists; explicit shared Fons1234 remains unchanged. Native replay
and scheduled typed export preserve both reservations. Owner tests in
`simulation/tests/chiz_held.rs` and `chiz_held_full.rs` retain independent
packet/time/resource checks. This does not add an initial-stock authoring field
or infer a shared Fons payment.

The Skill's `/14/Properties/MaxSkillBean` is **3**. `/12/Properties/AutoCostSectionNames` contains `Skill2`, `Skill3`, `Skill2Hold`, `Skill3Hold`. The localized `GA_Chiichan_Skill_des` describes three stored attempts, ordered No/Half/Full Grain inputs, two-stack hold entry, and a further stack for an early Full Grain finish. These establish configured capacity and described input choices, not the native refill/atomic admission/cancel policy.

`TriggerJumpSectionImpl` enters Ubergraph offset **996**. The graph resets `LoopCostVal` to zero, and for `Loop` or `LoopHold` obtains the active `ChiiChanLogicObj`. Offsets **1590–1881** read three distinct live fields and assign:

```text
LoopCostVal = (LoanCoinMax − LoanedCoin + CurrentCoin) × CostCoef
```

`CostCoef` explicitly serializes `Value=1` and binds `Chiichan_Loopdmg_Per`, whose constant coordinate-1 value is **0.0625 = 1/16**. The assignment captures an argument when entering the loop section. It is not a demonstrated continuous recalculation on every damage event. The CDO's separate `LoopCostVal=0.1` must not override the reached reset/assignment graph.

`OnReceiveEvent` checks whether the event's configured effect container includes `GE_Player_Chiichan073_Skill2_CoinDamage`. After successful actor/logic-object casts, offset **506** calls native `ChiiChanLogicObj:CostAndRecordValueForCurCoinDam(float(LoopCostVal), 1)`, before the parent `GA_MontageBase_C:OnReceiveEvent` call at **553**. This binds a concrete callback/order and argument; the native consumer's cap, borrow/repay mutation, rounding, second-argument semantics and final damage remain unresolved.

**Independent counterexample:** explicit source fields `LoanCoinMax=100`, `LoanedCoin=40`, `CurrentCoin=20` yield an argument of **5**. Using only remaining own Grain would yield **1.25**. This checks the read expression, not actual native Grain deducted or damage dealt.

## Reached settlement guard

`ConditionalSettleStock` (`/2/ScriptBytecode`) has this source-bound control flow:

1. Without `bHasRecordedStockPrice`, return. Also return when `SettleStockSectionName` is `None`.
2. Convert the serialized `SettleInterval` and compare `GetGameTimeInSeconds() − LastSettleTime >= interval`. The Skill CDO explicitly supplies **0.2**, with no curve.
3. Require the section in `SectionCostCoinMap` and successful actor/logic-object casts. Call native `SettleStock` with that section's scalable value.
4. After that call, set `LastSettleTime` from `GetGameTimeInSeconds`, clear the section to `None`, and clear `bHasRecordedStockPrice`.

The exact source offsets are **0, 14, 75, 117, 146, 192, 230, 244, 667, 737, 1077, 1108**. The map serializes zero-valued entries for `Skill1`, `Skill2`, `Init`, `Loop`, `Skill2Hold`, `LoopHold`; that does not establish free Skill inputs or a zero settlement result. Native `SettleStock` interprets the argument.

At the abstract guard, a recorded valid section with elapsed **0.199** fails and **0.2** passes. After a successful call clears the recorded flag, a repeated invocation cannot pass until the state is recorded again. These are independent control-flow counterexamples, not a measured gameplay timeline. Native event arrival, missing initial `LastSettleTime`/flag defaults and the game-time clock's relation to real combat time remain open. The Ubergraph's reached `SetRecStockPriceEnable(true)` then `(false)` calls do not expose that native function's capture algorithm.

## Market and interest configuration

The player logic object binds the following fields to constant coordinate-1 rows of `DT_ChiichanEffectFigure`:

| Logic field | Curve row | Curve value | Scalar multiplier serialized on the field |
| --- | --- | ---: | --- |
| `UpdateCoinInterestInterval` | `Chiichan_UpdataCoinInterestInterval` | 0.5 | Absent |
| `InterestRateOfLoan` | `Chiichan_InterestRateOfLoan` | 0 | 1 |
| `InterestRateOfDeposit` | `Chiichan_InterestRateOfDeposit` | 0.1 | 1 |
| `PressCostCoin` | `Chiichan_PressCostCoin` | 0 | 1 |
| `HoldCostCoin` | `Chiichan_HoldCostCoin` | 0 | 1 |
| `UpdateStockMarketInterval` | `Chiichan_UpdateStockMarketInterval` | 0.5 | Absent |
| `StockRiseWeight`, `StockLoseWeight` | `Chiichan_StockRiseWeight`, `Chiichan_StockLoseWeight` | 50 each | 1 each |
| `MaxRiseRange`, `MaxLoseRange` | `Chiichan_MaxRiseRange`, `Chiichan_MaxLoseRange` | 50 each | 1 each |
| `AddStockRiseWeightLevel3` | `Chiichan_Level3` | 0.5 | 1 |

An absent scalable multiplier stays absent. Do not promote a raw curve sample to a final timer interval, probability or percent unit without the native consumer/default. Zero press/hold curve values do not bypass the separate loop-cost graph.

The object explicitly serializes `AmountOfStockPriceGenerateOnInit=14` and `AmountOfStockPriceGenerateOnUpdate=0`, and references `DT_ChiichanEffectFigurePseudorandom`. Its `Para` row has 28 points; coordinates 1–14 and 15–28 repeat the same sequence:

```text
0.25, −0.08, 0.38, 0.50, 0.14, −0.29, 0.19,
0.06, −0.28, 0.32, 0.25, −0.08, −0.32, 0.23
```

This locates a concrete sequence/configuration; it does not establish the initial index, random first-price rule, cycling, percentage conversion, clamp, or whether this path supersedes weight-based generation. `bRandomFirstStockPrice`, `MinRiseRange`, `MinLoseRange`, `CoinMax`, and `LoanCoinMax` are mapped fields but are not serialized on this object. No default is invented for them.

The English `GA_Chiichan_UltraSkill_des` describes a seven-second market, conversion of remaining loans into Grain on entry, first settlement guaranteed critical, one held settlement using the first activation price, and repayment priority on exit. `GA_Chiichan_star2_des` describes 10% interest **per second** in Surplus. The raw interest interval curve instead contains **0.5**; native activation, timer/default scaling and settlement are needed to reconcile these sources. Do not implement 10% every half-second, compound twice per second, or a 50/50 independent RNG from names/numbers alone.

## Extra-damage override rounding recovered, October 18

The initial bundle's warning-free `BP_ChiiChanCostCoinDamExecution:K2_GetChiiChanExtraDamBaseOverride` (`#/2`, statements 109..782) reads **InLogicObj.CurCoinDamValue**, not a fresh recomputation of Grain/loan balance. On valid logic/source components it compares that field to `SourceAttributeComponent.GetLogicCustomCoefficient()` with **NearlyEqual tolerance 1.0**. Crucially, a mismatch only prints a diagnostic and jumps back to the same calculation at statement 300; it does not reject the override or select the getter as its value.

The function then sets `CoinToCost=floor(CurCoinDamValue)`. On CoinToCost > 0 it returns **floor(CoinToCost * LoopDamCoef(coordinate 0))**. The two floor stages must stay separate: supplied value 2.9 and coefficient 1.5 give **3**, whereas one final floor on 2.9*1.5 gives **4**. These example inputs discriminate the expression; they do not assert the configured LoopDamCoef or settled resource cost. The function itself does not mutate CurrentCoin/LoanedCoin or establish how CurCoinDamValue was recorded. Native `CostAndRecordValueForCurCoinDam`, `SettleStock`, capture clocks and the positive-value caller remain the resource boundary.

## Distinct settlement and loop inputs recovered, October 18

Initial `GE_Player_Chiichan073_Skill1_CoinDamage#/2/Properties/Executions` selects **BP_ChiiChanExtraDamExecution**. Its warning-free `K2_GetChiiChanExtraDamBaseOverride` (`#/2`) validates InLogicObj and returns **floor(LastSettleStockResult * Chiichan_Exdmg_Coef)**, with the coordinate-zero curve equal to **1**. It does not read current Grain or repay loans. A supplied last settlement 2.9 yields 2, independently of the two-stage loop-cost expression above. Native settlement recording and interpretation of a negative result remain with the execution body.

By contrast, `Calc_ChiiChan_CurrentCoin:CustomCalculateBaseMagnitude` (`#/2`) obtains the instigator's ChiiChanLogicObj and returns **max(0, CurCoinDamValue)**, not the field CurrentCoin. Its copied ClampValue=(0,2700,0) is unused. `GE_Player_Chiichan073_Skill2_CoinDamage#/2/Properties/CustomApplicationModifierInfos` binds this to additive **LogicCustomCoefficient**, alongside the separately selected BP_ChiiChanCostCoinDamExecution. The name CurrentCoin must not redirect this read to an affordability or wallet balance.

`Buff_Chiichan073_Level6#/1/Properties/CustomApplicationModifierInfos` supplies additive **DamageUpGeneralBase** through `Calc_ChiiChan_SkillDamUp`, multiplier 1. That warning-free `#/2` helper reads instigator ASC CurrentDamageGameplayEffect, requires **State.Damage.Skill.ChiichanCoin1 OR State.Damage.Skill.ChiichanCoin2**, and returns **Chiichan_Coin_DmgUP=0.15** at coordinate 0 on that path. It does not modify Grain values or all Skill packets. Native current-effect capture and application remain open; A6's zone/tag consumer is now located.

## Market time-dilation actor's removal listener recovered, October 18

The initial warning-free **Buff_Chiichan073_UltraSkill_TimeSlomo#/1** has HasDuration **7**, target aggregation/cap1, with self-removal requests on **BEKILL** and **CHANGE_ROLE_OUT**. Expansion **Actor_ChiichanUltraSkill_TimeDilation:ReceiveTick#/7 → ExecuteUbergraph#/2**, 130..601, calls the Actor tick parent, then registers **AsyncTaskGameplayEffectGEAddOrRemove** against its owner ASC and that exact carrier class path. The Remove callback (`#/4 → /2`, 34..76) requests **SetLifeSpan(.01)**; Add only stores its callback arguments. BeginPlay and BeginOverlap overrides return without adding a manual damage or pause call.

The task is registered from **Tick**, not once in BeginPlay; no task deduplication or explicit listener teardown is recovered from this Blueprint. Its actor inherits **BP_TimeDilationVolumeActor → native TimeDilationVolumeActor**. The shared Blueprint parent only prints a diagnostic on BeginPlay and supplies no dilation scheduler. Both actor CDOs retain inherited read warnings, so no effective dilation factor, complete lifetime or frozen-resource clock is promoted from those tails. The source now locates a market-carrier removal listener and its .01 lifespan request, not proof of restored world time, one listener per cast, native elapsed seven-second duration or the interest/market timer pause policy. Sources: `projectile-guard-and-slomo-config.txt`, `time-dilation-parent-functions.txt` and `time-dilation-parent-header.txt`.

## Global cap and acquisition curves

The separate `DT_GlobalCommonData` export, SHA-256 `d15fecf599b4a51567972c3c759b63ce5ae1ec4d54c60cc5f8a9a5cef19c4386`, supplies two cap-named rows and nine acquisition-named rows not bound by the serialized logic object above. The same 13-row table is already a frozen implementation input (optional cached source `DT_GlobalCommonData`); no new raw table needs committing. Source pointers are `/0/Rows/<name>/Keys` in the original export, or `/rows/<name>/Keys` in that normalized table.

| Curve family | Coordinate 1 | 20 | 40 | 60 | 80 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `Chiichan_CoinMax`, `Chiichan_LoanCoinMax` | 500 | 1,000 | 5,000 | 12,000 | 15,000 |
| `Chiichan_CoinMelee1..5`, `Chiichan_CoinBranch`, `Chiichan_CoinEvadeAttack1`, `Chiichan_CoinEvadeAttack2`, `Chiichan_CoinEvadeAttackshow` | 20 | 40 | 170 | 360 | 450 |

Both cap rows are identical; all nine acquisition rows are identical. Each has 79 keys, omitting coordinate 2, constant interpolation and constant endpoints. Thus the declared curve selection at 2 remains the coordinate-1 value, not an interpolated step toward coordinate 3. Coordinates resemble progression levels, but the native lookup coordinate, initialization/updates and effective cap/grant consumer remain unproved. The native executable contains both cap row names as UTF-16 strings; that does not establish their call path.

These curve values are **source facts**, not automatic grants or defaults for the omitted `CoinMax`/`LoanCoinMax` fields. At an explicitly selected coordinate 80, one grant per five Normal inputs would total 2,250, whereas one grant per the twelve packets in the current chain would total 5,400. Neither is selected as native behavior. The existing loop expression with an explicitly supplied loan cap 15,000, no used loan and no own Grain yields an argument of 937.5; native deduction/capping remains separate. The new source receipt retains the table and those independent counterexamples. This resolves the curve locations/values, not the native resource policy.

## Handoff and limits

Configured charge capacity, global cap/acquisition curve locations and values, loop argument reads and the Blueprint settlement guard are now identified. Remaining questions are native charge refill/admission; cap/grant lookup coordinates, application and Grain/loan mutation; market generation/cursor/weight normalization; first critical settlement; hold/end/repayment ordering; timer/switch/death behavior; and cutoff applicability. Continue at the named `ChiiChanLogicObj` consumers rather than searching again for missing scalar rows.

The [unknown owner](action-damage.md#unknown-execution-policies) retains these policies. No runtime policy, simulator code, frozen evidence or historical receipt was changed.

## Typed consumed level constants

The existing Grain pool now reads [typed Rust level records](../../../../crates/content/src/characters/chiz/values.rs), containing only capacity and the common grant used by all five admitted Normal stages. The five source grant curves are equal at every supported level and are stored once. These are the consumed frozen `DT_GlobalCommonData.Chiichan_CoinMax/CoinMelee1..5` values, not a new market/resource model or a copy of the full source table. Other unused common curves and raw curve metadata stay outside Git.

Independent frozen checkpoints are `(level, capacity, Normal grant)`1/500/20,20/1000/40,40/5000/170,60/12000/360 and80/15000/450. Level2 retains level1 under the selected Constant-left-key policy. Grants are independent of capacity: a universal3% calculation would give15 rather than20 at level1, and150 rather than170 at level40. Tests preserve these counterexamples and levels0/81 diagnostics. Storage migration does not bind loans, market spending or additional acquisition inputs.

## Executable exact-Grain prerequisite

`chiz.exact_grain_v1` replaces the integer own-Grain pool with the shared exact
bounded ledger. Character-level capacity and each positive first connected
Normal-stage grant retain rational A1/A4 results; there is no whole-unit floor
before a future money-damage override. A5 credits one unmultiplied capacity at
the initial battle entry, starting from zero savings. Field entry never repeats
that grant. The report now uses `fractional_resources.chiz.grain` with exact
initial/current/granted/spent/overflow and conservation, alongside other exact
owner gauges. The retired integer `resources.chiz.grain` entry is omitted.

Independent Rust checks retain level3 capacity525*1.1=577.5 and grant21*1.2=25.2.
Ten inputs/two continuing chains produce252 despite24 HP packets. After23
inputs, grant579.6 splits into capped577.5 and overflow2.1; the last deposit is
23.1. Native legal level3 builds also check selected A1/A4/A5 independently,
the unmultiplied577.5 initial battle grant, exact resource summaries and seeded
aggregation/replay. The existing caller-timed action profile is retained for
this prerequisite. Loans/interest/market/Skill charges and non-Normal grants
remain the explicit remaining owner work; this does not certify a full kit.

## Executable Surplus interest prerequisite

`chiz.exact_finance_v1` promotes own-Grain deposits/A5 accounting to the shared
bounded wide rational pool. The currently admitted paths have no borrowing or
investment, so their financial mode is ordinary Surplus. Selected A2 grants
10% of the live positive own balance once per real second, phase0 at initial
living entry. The.5s source bookkeeping does not grant or compound separately.
Interest ignores acquisition multipliers, retains cap overflow, runs off field
and never produces Ultimate Energy, Cycle or Fons. Empty wallets schedule no
polling; subsequent deposits retain the existing Surplus phase. Source death
cancels credits and actual HP revival starts a fresh phase, even with an empty
wallet. Equal-time ModePause interest precedes ordinary HP/acquisition.

Long exact compounding cannot use the old u128 resource storage:450*(11/10)^37
has132 numerator bits before reaching the A1 cap16500. The kernel therefore
retains components up to4096 bits and rejects an oversized operation atomically.
Small summaries remain numeric; wide summaries use decimal-string components
under the [runner contract](../../../../openspec/specs/combat-execution/reference.md). No per-interest
rounding, float conversion or cap-sized fabricated grant hides overflow.

Independent checks include100→110→121, fractional gain/cap accounting, A4
exclusion, off-field A5, death/revival/stale callbacks and an empty500s horizon
with no polling. Two continuing native Normal chains at0/5s through10s leave
exact Grain8488.73633454 while energy/Cycle remain39.12/54.196. One Normal then
150s of A2 reaches cap16500, with total credited
`450*(11/10)^38 +112*1650`; the exact granted/overflow fractions survive native
aggregation/replay and WASM JSON/binary codecs. The future phase beyond the
observation horizon remains state without an invented queued event or expiry.
Skill stocks/money overrides, loans/Debt transitions and market settlement are
still pending and are not certified by this prerequisite.
