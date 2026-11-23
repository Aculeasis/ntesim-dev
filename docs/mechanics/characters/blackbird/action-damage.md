# Blackbird action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1042`.

**Responsibility:** Blackbird outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** selected whole-kit model executes through the Rust/native runner, including transformed secondary actions, held branches, cancel gates, explicit parry and combat transitions. Hypotheses establish the declared implementation model, not observed native behavior. Native source now binds ordinary/evolved holds and supported grounded controls through explicit input windows; parry/combat-state authoring remains Q4-L5.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Native source input windows

The [ordinary/evolved window script](../../../../examples/blackbird-input-windows.nte)
reaches unfinished-montage Roostcall, Bestowed Grace and Tidings through
`window=hold` and ordinary `attack[hold=200ms]`. The native selector retains
its stage/form conditions, hold lead-in and cancellation gates; source copies
no timing constants. Native Jump, Skill and paid Dodge/Riposte use their own
windows. Existing owned frame/cost tests remain authoritative.

Source checks independently retain the ordinary214249us yield and614725us
Roostcall contact, stage3 Jump at1977027us after only prior hitlag, committed
Normal2 extras after replacement, and six spawned Normal5 packets after Jump
even when all targets are immune. Two ordinary and two evolved branch cycles
retain continuing state, source artifacts, completion metadata and full native
replay. A window alone changes no native damage/RNG result; a following tap
still fails at its real recovery boundary. Explicit parry/combat-state inputs
and remaining owner source choices stay in Q4-L3/L5.

## Selected collection handoff (October 18, 2026)

These decisions supersede the historical partial-runtime interpretation below. Native table/asset values are `source`; arithmetic is `derived`; missing dispatch/clock/lifecycle semantics receive explicit checked `inferred` rules. Collection acceptance means a complete selected model, not factual certification of every native body or current executable kit completion. Both the action and [Malice/coordinated subsystem](malice-and-coordinated-damage.md) retain source provenance and earlier failed approaches; do not restart protected-body/extraction campaigns.

### Complete packet specification

**Mixed.** Preserve full arrays from [consumed typed damage rows](../../../../crates/data/src/native_damage/blackbird.rs), effective normal/Skill/Burst investment shared with its evolved counterpart, QTE its own investment; R3 adds1 to Normal/Skill/Burst/evolved counterparts, not QTE. All ordinary and Malice-extra packets are **Psyche/ATK**, ordinary live CR/CD (FixedCritRate0 is the no-override sentinel). Coefficients below are level1/8/9 checks. Bird uses Skill talent. Each evolved extra's coefficient calculator explicitly reads **Normal level**, even on Skill/QTE/Burst; this is independent from the primary packet's talent.

| Suffix after `GE_Player_BlackBird_` | Count | Coefficient1 /8 /9 | Charge / Cycle / Break per primary packet |
| --- | ---: | --- | --- |
| Melee1_Damage |1 |.237 /.406 /.439 |.838 /1.161 /.141 |
| Melee2_Damage |2 |.487 /.835 /.901 |1.726 /2.391 /.291 |
| Melee3_Damage |4 |.301 /.516 /.557 |1.066 /1.476 /.180 |
| Melee4_Damage |5 |.258 /.442 /.478 |.913 /1.265 /.154 |
| Melee5_Damage |5 |.364 /.624 /.674 |1.289 /1.785 /.217 |
| Branch1_Damage |1, Roostcall from any eligible branch montage |1.139 /1.952 /2.108 |4.08 /5.6 /.7 |
| AirAttack_Damage |1 |.476 /.816 /.881 |1.68 /2.3 /.3 |
| PerfectEvade_Damage |1 |1.275 /2.185 /2.360 |3 /4.2 /2.5 |
| QTE_Damage |1 |2 /3.428 /3.702 |7.56 /0 /2.5 |
| Skill_Damage |4 |1 /1.714 /1.851 |1.89 /2.6 /.325 |
| SkillAssist_Damage |1 per coordinated bird |2 /3.428 /3.702 |2 /3.34 /.4 |
| UltraSkill_Damage |9, transformation Burst |.467 /.800 /.864 |0 /0 /.278 |
| Evolve_Melee1_Damage |2 |.325 /.557 /.602 |.974 /1.353 /.162 |
| Evolve_Melee2_Damage |5 |.222 /.380 /.411 |.665 /.924 /.111 |
| Evolve_Melee3_Damage |6 |.210 /.360 /.389 |.630 /.874 /.105 |
| Evolve_Melee4_Damage |1 |1.355 /2.322 /2.508 |4.066 /5.648 /.678 |
| Evolve_Melee5_Damage |6 |.471 /.807 /.872 |1.414 /1.963 /.236 |
| Evolve_Branch1_Damage |5, Bestowed Grace |.432 /.740 /.800 |1.302 /1.798 /.217 |
| Evolve_Branch2_Damage |5, Tidings Unto All |.404 /.692 /.748 |1.218 /1.682 /.203 |
| Evolve_AirAttack_Damage |1 |.560 /.960 /1.037 |1.68 /2.3 /.3 |
| Evolve_PerfectEvade_Damage |1 |1.260 /2.159 /2.332 |2.52 /3.5 /2.5 |
| Evolve_QTE_Damage |1 |2 /3.428 /3.702 |6 /0 /2.5 |
| Evolve_Skill_Damage |4 |1.05 /1.800 /1.943 |1.26 /1.75 /.2 |
| Evolve_UltraSkill_Damage |7, finishing Burst |.900 /1.542 /1.666 |0 /0 /.357 |

Derived sums at8: ordinary **17packets/9.470ATK/19.564charge/27.097Cycle/3.298Break** (source-row arithmetic corrects historical27.135/3.421); evolved **20primary packets/12.338ATK/21.603charge/29.996Cycle/3.603Break**. Four-hit ordinary Skill is6.856, evolved Skill7.2; transformation Burst7.2 and finisher10.794, without later upgrade/Malice modifiers. Ordinary Branch1/2/3 montage names all map the **same single** Roostcall row, not three chained damage skills. Evolved Melee5 uses one spawned actor with period.12/cap6, not six actors or12 primary hits. Plunge main coefficient multiplies by `1+h`, explicit scenario height bonus0..1; no arbitrary height-speed curve or automatic maximum. All source primary packet resource channels remain separate from Malice.

**Separate Malice damage:** each successful evolved main impact that consumes resource produces **one** matching `Evolve_<action>_Extra_Damage` packet with coefficient `k_normal_level * consumed_malice`, or `(k+k3)*consumed_malice` with selected **individual A3**. Source base row is0 and helper supplies **CoefAdd**, not CoefModify/general damage. Native `DT_BlackBirdEffectFigure` gives `k(1/8/9)=.28/.479871/.518260`, `k3=.14/.239935/.259130`; retain all13 source entries. Choose atomic resource capture before spending, bounded at min(liveMalice,action threshold), never a later read of already spent0. Extra uses captured amount/coefficient/causal action/recipients, live ATK/bonuses/DEF/RES and a separate ordinary critical draw at its own delivery. Do not merge it into the parent's critical roll/HP packet or grant raw energy/Cycle/Break: all Extra_Damage rows have zero resource channels and native **category A**, including QTE/Skill/Burst extras. Their primary category remains A/X/Z/H/E/Q as listed in native rows. A stale table name cannot turn an Extra into an E/Q action bonus.

Thresholds per **connected primary packet**, not per dummy, are: Melee1 .541591;2 .369949;3 .349951;4 2.258020;5 .784891; Branch1 .719900;Branch2 .673240; plunge .933204; riposte2.099708; QTE1.666435; Skill3.25. Each has matching direct Extra request except Melee5's located .033s intermediate FINISH route. At full stock an evolved chain spends **11.999999**, leaving all source decimals exact; its Malice coefficient sum is `11.999999*.479871=5.758451520129` at8. A3 raises the gain curve by half, not another +.50 absolute ATK on each main packet. At Malice.1 the first Melee1 spends.1 and adds coefficient.0479871, not.479871 and not a.1% ordinary bonus. At0 the primary still executes but no positive extra damage packet is fabricated.

**Finisher:** its first UltraSkill.1 request is the only mapped extra path; six UltraSkill.2 events are primary-only. With **A1** capture/spend all remaining Malice once (max60) on that first successful packet and produce its one matching extra, `(k+k3)*capturedMalice`; do not reuse the full pool on each of seven hits. Without A1 **do not call the extra helper's unassigned absent-marker branch**; the main7 packets still execute and preserve Malice. `Evolve_UltraSkill_Damage_checkLV1` is a conditional/resource caller with empty coefficient arrays, not an eighth main hit. This reconciles the source A1 gate, first-event mapping and finite resource conservation; native pre/post-spend call order remains an explicitly inferred implementation choice.

Ordinary A2 and R3 coefficient changes bind **CoefModify**, interpreted as relative coefficient multipliers: A2 ordinary Skill ×2, R3 transformation Burst ×2, A1 evolved Skill ×2.5, R3 finishing Burst ×1.5. They do not add1/1.5 directly to the ATK coefficient or multiply the separate Malice CoefAdd a second time. Source calculators return1/1/1.5/.5 with their actual markers; absent marker explicit0 means neutral multiplier1. Individual A3 only changes the Malice-extra gain curve. At base Skill8, A2 gives3.428 per hit (13.712total), not2.714; a bare1000ATK first evolved Skill hit with A1/base1.8 plus Malice-spend3.25 gives main4500 and separate1559.58075, before other zones, not `(1.8+1.5+1.55958075)*1000`. The selected relative interpretation follows CoefModify versus CoefAdd and the repeated percentage descriptions; it corrects historical absolute-add implementations without changing their receipts.

The reaction enhancement's `GE_Reaction_4_new_1042_Damage` is a separate capped **fixed pending-settlement Mental** amount, not another attack coefficient; its complete rule belongs to effects/dependencies. Source review: [ordinary/evolved coefficient helpers](#ordinary-a2-and-resonance-coefficient-consumers-recovered-october-18), [Malice formulas and delayed calls](malice-and-coordinated-damage.md#extra-coefficient-branches-and-delayed-request-recovered-october-18), frozen skill description numbers, [current public full kit](https://neverness.app), `DT_BlackBirdEffectFigure` and named source CDOs in original/supplement bundles. Native and public methods were already tried beyond the two-approach allowance; remaining implementation choices above are finalized, with native certainty retained separately.

### Complete timing specification

**Mixed; selected real-time model.** Native montage coordinates below are source inputs, not measured wall time. Run progress at1 asset second per combat second, adding each connected packet's owner hit pause `duration*(1-rate)` to subsequent montage progress. Ordinary Normal/branch/Skill and evolved Normal1..4/branches use duration.03/rate.01 (**.0297s**); evolved Melee5 actor packets use.03/.5 (**.015s**); plunge/riposte/QTE use.13/.01 (**.1287s**); transformation Burst uses.067/.3 (**.0469s**). Evolved Skill/finisher have no explicit pause entry: selected model adds none. Count pause once per packet, never per dummy or extra packet. Source: corresponding `DT_SkillDamageData.HitPauseData`, `BlackBird_*`/`Blackbird_Evolve_*` montage Notifies, and both GA EffectContainerMaps in original/supplement exports. This reconciles multi-hit pause and usable recovery; it is not the historical fixed800ms-per-input profile.

| Action | Primary hit progress coordinates | AnySkill recovery coordinate |
| --- | --- | ---: |
| Normal1 |.14994475 |.75591975 |
| Normal2 |.10052954,.36498323 |.7339425 |
| Normal3 |.33417627,.38361928,.45063588,.51631886 |1.7302052 |
| Normal4 |.26699644,.3338795,.40135658,.46760905,.5342906 |1.2989099 |
| Normal5 |.23370183,.35172206,.47744727,.60320276,.7178988 |.9164833 |
| Roostcall, all3 branch montages |.20047607 |.75721043 |
| Ordinary plunge, after authored landing |.053784 |.28124905 |
| Ordinary riposte |.28227764 |.9036751 |
| Ordinary QTE |1.0002216 |1.3331809 |
| Ordinary Skill |.067548856,.41632304,.7170708,.9017731 |1.3326912 |
| Transformation Burst |3.2175567,3.458336,3.707061,4.149342,4.250603,4.349944,4.450082,4.54971,4.649931 |5.118154 |
| Evolved Normal1 |.16745183,.35073686 |.9869099 |
| Evolved Normal2 |.093687,.23225366,.3675626,.39968485,.43489504 |.7626522 |
| Evolved Normal3 |.18380167,.365976,.4173092,.46672165,.51726854,.56689256 |1.7506349 |
| Evolved Normal4 |.23295267 |1.0643272 |
| Evolved Normal5 |spawn at.3371184, then6 real-time actor hits at spawn+.12*n, n=0..5 |1.1781635 |
| Bestowed Grace |.61771244,.7339041,.85248506,.96751726,1.08383 |1.345821 |
| Tidings Unto All |.14384507,.28379494,.44197342,.5824538,.7306768 |.97510856 |
| Evolved plunge, after landing |.100642994 |.281249 |
| Evolved riposte |.22595885 |.83927536 |
| Evolved QTE |.8781806 |1.1679292 |
| Evolved Skill |.13333334,.2773192,.44984162,.6839939 |.8941296 |
| Finishing Burst |2.4333334,2.5833333,2.7333333,2.8833334,3.0333333,3.1833334,3.3505173 |4.0833335 |

Derived uninterrupted ordinary chain **5.94036065s**, evolved chain **6.2484877s**, ordinary Skill1.4514912s and transformation Burst5.540254s. Actor periods are real-time, not montage progress: pausing its owner does not spread the six Melee5 hits over.675s. Recovery still accounts for all6 owner pauses, so it finishes after the last actor packet. This is checked against both the six-packet limit and the montage window, rather than treating one actor as six casts.

The executable `blackbird.ordinary_clock_v1` binds ordinary Normal1..5 and Skill to these source coordinates, rounded to microseconds before the shared Rust hit-pause projection. Independent Rust checks establish17 primary contacts and full-chain recovery5.940361s, four Skill contacts67,549/446,023/776,471/990,873us with recovery1,451,491us, no pause on zero applied HP, one pause across all AoE recipients and an independent16s Skill cooldown. Normal continuity uses actual recovery plus2s, including the inclusive boundary. The legal native bird fixture also runs two continuing controllers with this clock, aggregation and seeded replay. Burst/form and transformed-secondary clocks plus early cancel gates remain separate required bindings; the historical caller-clock model remains available when this bounded clock is absent.

Cancellation uses the **earliest applicable** gate, not always AnySkill. In stage order1..5 ordinary Jump gates are `.18454874,.336276,.36836368,.88835806,.7656156`; Skill gates `.3330773,.50201637,.66792464,.6390141,AnySkill`; Move `.7543125,.7520547,1.0017732,1.3012999,1.2868648`. Evolved Jump `.17306578,.10392535,.17535761,.18590695,.52413434`; Skill `.6442127,.57939017,.9664505,AnySkill,AnySkill`; Move `1.0106286,.7696378,1.752073,1.0549064,1.8475844`. AnySkill also admits the next Normal. Roostcall Jump/Skill/Move `.250451,.66748893,1.010329`; Bestowed Grace `.66632885,1.1999046,AnySkill`; Tidings Jump.8402419/Move1.762101/SkillAnySkill. Ordinary/evolved plunge Jump.149206, Move.77753407/.52225524. Ordinary/evolved riposte Jump/Skill0, Move=AnySkill. Ordinary Skill Jump.288606/Move1.526221; evolved Skill Jump.83597463/Move1.0074825. Transformation Burst Jump/Skill/Move5.044123; finisher all gates4.0833335. QTE ordinary Jump=AnySkill/Move1.3354977; evolved Jump1.274009/Move1.2715421. Add only pauses already caused before the gate. Gates permit early cancellation even before all hits: cancel removes remaining montage-bound requests; already spawned actors and captured extras continue.

Switch is admitted at AnySkill recovery except explicit QTE locks: ordinary1.3445089/evolved1.2669404 progress, transformation Burst5.0333333 progress. Require **both** recovery and lock expiry; finisher conservatively locks through4.0833335 (notify width not recovered). Switch has no extra invented universal delay. Field exit cancels unfinished montage packets, retains spawned effects and pauses form duration. AppearMelee uses Normal1; plunge requires an authored nonnegative airborne/landing interval, charged to DPS time, and cannot be used as a free ground input. Ordinary dodge costs15 stamina; choose front recovery1.00476, Skill cancel.333319, Jump.083202; back recovery1.0091343 (evolved1.008023), Skill.531889, Jump0. Dodge itself has no damage. Explicit successful Critical Dodge grants one next-Normal riposte authorization, cleared by another non-Normal action/exit; explicit parry is a distinct scenario event. This finite authorization is inferred, avoiding an unsupported native expiry number.

Cooldowns start at accepted action commit on the real axis: ordinary Skill16s, transformation Burst20s; evolved Skill `.0625*16=1s`, finishing Burst `.25*20=5s`, from `CoolDwonCurveTable` and GA multipliers. Separate handles; entering/leaving form does not reset them. No cooldown on ordinary/alternate Normal, plunge or riposte beyond recovery/admission. Qualified switch owns QTE admission, not a repeatable free QTE input. Time-stop/sequence components affect presentation in this selected timing model, not all party cooldowns, reaction timers or the DPS denominator.

Extras: finalize the unresolved inherited duration policy as one delayed periodic execution at **primary real hit+.067**, from duration.1/period.067/no immediate. Melee5 alone uses its located intermediate FINISH.033, hence **+.100**. Do not also schedule the unused intermediate on direct routes. Rationale: only one tick fits .1, and a single request must conserve the atomic spent resource; instant-plus-period or direct-plus-intermediate would duplicate it. These are inferred delivery times; the serialized missing parent policy remains a factual limit. Pending packets survive normal field exit/form expiry, cancel on target death/scene termination, and never refund consumed Malice.

### Complete resource and admission specification

**Mixed.** Ultimate Energy capacity100, initial0 unless scenario declares another legal start; primary `GA_BlackBird_UltraSkill` requires native 100 Ultimate Energy and incurs a 20s cooldown. In live combat, exiting form via the finishing Burst `GA_BlackBird_Evolve_UltraSkill` requires 0 Ultimate Energy and has 0 cooldown; energy continues accumulating normally from allied/self hits during the transformed state. Primary Burst spends 100 at successful commit, rejecting insufficient energy; finisher requires only live form. Neither form expiry nor Malice depletion refills energy. Other combat attacks have zero Ultimate Energy admission cost. Ordinary dodge15 reads the required [stamina scenario profile](../../shared/energy-and-gauges.md); no unmeasured flight-drain number is substituted for combat stamina.

Malice is an exact nonnegative resource capped60. Battle entry grants20 once via TanFan ENTER_BATTLE; leaving combat schedules one further20 after4s if still out of combat (LeaveBattle duration4.1, first periodic4, no immediate); re-enter cancels that pending out-of-combat callback and grants the distinct new entry20, clamped60. It is not20 every4 combat seconds. Ordinary connected primary packets gain: Normal1 .857919,2 1.762896,3 1.089593,4 .933937,5 1.317647; plunge1.723077; riposte4.615385; Roostcall4.123077. One full ordinary chain gains **20.000003**, once per packet regardless target count. Gains require positive HP application, matching raw resource admission; never per dummy. Ordinary Skill's Skill.2 marker at0 grants40 once even if its later hits miss. QTE.2 at1.0024883 grants20 with B4, not the packet's zero Malice row. Successful parry grants20 at its explicit PARRY_ATTACK event, separate from dodge damage. A6 accepted bird grants4 only outside transformation. Transformation suppresses these ordinary gains; the specific R3 transformation-commit20 is an explicit exception. Both public no-gain-in-form and specific R3 grant remain meaningful.

Evolved packets spend only on successful positive primary HP application, using the thresholds in packets. Reserve/capture/spend once for the causal packet, then enqueue its extra; at0 the primary is still legal. Miss/invulnerable/no living recipient spends nothing. Partial stock spends the remainder, without a negative pool or threshold-level admission restriction. With A1 finisher first connected hit spends all then-live Malice once; it does not prepay60 at cast. Resource conservation checks: full evolved chain11.999999; full evolved Skill13; two of each spend49.999998 from60, leaving10.000002. No integer rounding of Malice. Ordinary Skill activation also clears/rearms its bird controller and **fills Cycle gauge to100 once**, before damage and even on miss; keep source HeterochromeExecution distinct from Ultimate Energy. Its four packets still have their own Cycle/charge rows and cap overflow. Duplicate hybrid callbacks share the same cast ID and cannot repeat the fill or40 grant.

Use [shared exact charge/carry, per-packet positive-HP gate, per-ally .6 share and Cycle rules](../../shared/energy-and-gauges.md). Each actor's efficiency applies to its own received energy; multiple recipients do not multiply source energy. Save charge integer/carry/overflow, Cycle integer/carry/overflow and Malice independently across waits, switches, forms and cycle boundaries. Admission also requires living active actor, required target/airborne/riposte/branch authorization, cooldown and recovery; no implicit waiting or resource reset on failure. Burst mesh replacement is a declared successful zero-delay callback in stationary-dummy baseline; native callback timeout2s is a failure diagnostic/incomplete run, not delayed damage. Inferred transaction: reserve energy/CD until successful callback, release reservation on timeout, never count failed execution as a successful comparison.

### Complete passive and awakening specification

**Mixed;** source markers/curves are `1042AwakenEffect`, `DT_BlackBirdEffectFigure`, owner GA/CDOs; [public kit](https://neverness.app) resolves the native dispatcher gaps. Enable B2/B4 by breakthrough, individual awakenings by selected identity, R3/R6 by selected count. Do not confuse curve legacy suffix Level2 with the actual Level6 gate.

| Selection | Complete damage-relevant rule |
| --- | --- |
| B2 | Stain base1.3 and duration20; Nova cumulative settlement/reapplication in the linked effects specification, enabled while Blackbird is on the deployed team, including off field. |
| B4 | Ordinary QTE marker adds20 Malice once; evolved QTE has its own spend row and no ordinary QTE.2 marker, so does not receive a fabricated extra20. |
| A1 | Evolved Skill main coefficients ×2.5; finisher first successful primary spends all remaining Malice for one extra. |
| A2 | Ordinary Skill main ×2; form DEF adds .20 of base DEF; evolved Skill commit heals own HP by4×baseATK, capped at MaxHP. No heal for each of4 hits; require the A2 marker. |
| A3 | Malice k3 curve adds to k on all eligible extras; not R3 and not an unrelated main coefficient bonus. |
| A4 | Transformation Burst's team ATK+.20 lasts25 instead of15; same recipients also gain general damage+.12 for25. Do not stack both base and replacement ATK carriers. |
| A5 | Blackbird positive HP packets apply Psyche/Lakshana/**Chaos** RES-.08 to each hit recipient for25, one source stack, refresh expiry. Source `CutDef_an` explicitly binds DamageResistChaosBase despite its legacy Incantation curve name. Causal packet uses old RES, following packets see new RES; no per-hit8% stacking. |
| A6 | Transformation Burst UltraSkill.3 grants own Psyche damage+.25 for25; successful coordinated bird adds4 Malice out of form. |
| R3, count>=3 | +1 Normal/Skill/Burst talent, shared with evolved counterparts, not QTE; transformation Burst main ×2, finishing Burst main ×1.5, successful transformation callback adds20 Malice once. |
| R6, count>=6 | Whole deployed party CD+.30 as composition effect while Blackbird remains on the team, including off field; selected inferred lifetime persists death because composition is unchanged, removed on roster removal/scene exit. No extra form-only duplicate. |

The form's native CritDamage curve is **0**; it is not another .30 on top of R6. Team ATK increases use base ATK, not compiled final ATK; e.g. base1000/final2000 becomes2200, not2400. Heal with base1000 is4000, even if buffed finalATK2200. A5 affects ordinary positive main/extra/bird packets without changing fixed Nova settlement; callback guards prevent the settlement from recursively modifying its already fixed amount. All short-lived source-cast buffs are cap1/refresh, not per-target duplicates; independent external-source stacking follows the shared stat zones. Combat unlock/passive markers are not toggleable omissions at baseline BT6.

### Complete alternate-action and form specification

**Mixed.** Ordinary Normal uses stages1..5 then wraps1; a different action or field exit resets stage. Continuity is allowed through the previous stage's full recovery and until2 action-clock seconds **after recovery**; later input starts at1. This inferred recovery-relative rule replaces historical2s-between-starts, which would reset legal multi-hit chains solely because their own recovery exceeded the gap. Holding after ordinary stages1/2/3 selects Roostcall (one identical packet row); native ordinary hold threshold.2 is used after a legal branch/cancel window. Evolved hold after stage2 selects Bestowed Grace; hold during Bestowed Grace or evolved Skill selects Tidings Unto All. Adopt the same .2s hold threshold for evolved branch admission where source threshold is absent, explicitly inferred from the ordinary input family; hold time elapses on the real axis and cannot manufacture a branch before its gate. A branch replaces, not appends free damage to, an unfinished montage. Completion resets the Normal stage. Plunge uses an authored landing interval/height bonus and its ordinary/evolved row; no airborne loop without a fresh airborne event. Explicit critical dodge consumes the one-use riposte authorization for the current form's row. Public “No Quarter” parry is its own event; never equate all ordinary evades with success.

Successful primary Burst switches ordinary abilities to `GA_BlackBird_Evolve_*` and gives hitstun immunity. Selected single-player/non-hybrid form timer is **21.5 on-field World seconds**, from native16+5.5 calculator at successful mesh callback; Hybrid profile16 is separately declared, never silently mixed. The shared cinematic pause freezes the form timer during the primary Burst; the historical15.959746 controllable-second subtraction no longer applies. Separate finisher-permission carrier is26 non-hybrid/16 Hybrid; it does **not** extend the main form. Pause both remaining form/permission clocks off field, resume them on re-entry; cooldowns, team buffs, marks and ordinary reactions use World time; finite pending actors use their declared action contexts. Natural expiry removes replacement abilities and permission and resets Normal stage, preserves Malice, charge, cooldowns, pending packets and live external buffs. Already committed actions retain their captured row/form; expiry does not retroactively change an evolved packet into an ordinary one.

Evolved Normal5 **Melee.5_1 at progress.35128844** (not after all6 actor hits) grants a30s cap1/refresh Skill permission. Actual successful transformed parry grants the same permission. Evolved Skill requires permission plus its1s CD; Skill.2 at progress0 removes it and performs A2 heal once. Cancellation after that cannot refund the permission or duplicate the heal. A pending Melee5 actor survives switching/expiry with its6-hit cap and original Malice-spend behavior. Form-loss clears unspent evolved-Skill permission. Optional finisher requires live form, its permission handle,100energy and5s CD; at successful commit remove main form/permission and retain its7 pending main packets, plus the A1 first-hit extra branch. The UltraSkill.3 event at finisher progress0 has no matching damage/buff map and does not grant the primary Burst's team buff again. Neither the unbound Transform_test carrier nor class names add another automatic form. Source callback order/permission cleanup are finalized inferences checked against finite replacement lifetime and no use of form-only actions after expiry.

The bird is the only mandatory off-field damage mechanism, specified in [selected subsystem rules](malice-and-coordinated-damage.md#selected-complete-subsystem-rules). Its cast generation/marks survive field exit; owner death stops new coordinated triggers but already committed packets retain normal pending-hit treatment. Fly/long-evade/GSkill movement has no outgoing damage in these tables; overworld flight is combat-ineligible. Stationary-dummy movement itself contributes no damage, but authored dodge/parry/proximity scenarios still pay time/stamina and preserve their actual effects.

### Executable ordinary Support

`blackbird.qualified_support_v1` requires the ordinary clock and Malice models.
The existing shared switch qualifier supplies the actual full outgoing Cycle,
supported adjacent-element reaction, living locked target and incoming recovery.
It dispatches one native QTE AoE with invested Support talent, no Ultimate cost
and no added QTE cooldown beyond the disclosed qualification/recovery model.
Unimplemented shared reaction pairs remain a shared interaction acceptance gap.
The real switch owns its100 Cycle spend and reaction; no free QTE input is added.

B4 QTE.2 grants20 Malice at its own progress1.002488s, including on damage miss.
The same Rust caster clock projects this cue alongside the native contact, while
bird requests and world timers keep real time. Positive contact gives actual
marker1.131188s, attack recovery1.461881s and distinct switch lock1.473209s;
miss keeps marker1.002488s with no contact pause. A source death before the cue
cancels its grant, preserving damage/Charge already settled. Native QTE charge7.56,
Cycle0 and Break2.5 remain separate from Malice and fanout never duplicates them.

Independent Rust cases check talent/recipient scope,99 versus100 outgoing Cycle,
B4/miss/death, authored-QTE rejection and both one-microsecond admission boundaries.
The equipped A0 [native fixture](../../../../examples/blackbird-support-cycles.nte)
qualifies two entries using two actual outgoing pools without reset, retains
continuing Malice/Charge and checks aggregation plus seeded full-report replay.
Transformed Support and remaining form policies remain required owner work.

### Executable Back Dodge intent

The optional native `dash_directions` profile binds Back through
`inputs.dash_directions_v1` and the existing secondary owner model. Recovery
captures ordinary1009134us or evolved1008023us at accepted input; Jump is0us,
Skill531889us, and movement uses AnySkill. Early controls still require the
existing `blackbird.cancel_gates_v1` binding. Source form is captured once;
direction has no distance/geometry semantics. Shared15 stamina/.9s cooldown,
explicit Critical Dodge and its finite next-Normal Riposte permission remain.

Rust checks distinguish both full recoveries, zero-time Jump and the strict
Skill boundary. The existing two-cycle native plunge/Riposte consumer now
checks front and back, exact resource conservation, aggregation/replay and
exclusion of an early counter. Front retains its existing full report and
source syntax; source lowering follows the supported native owner profiles.

### Executable ordinary secondary actions

`blackbird.ordinary_secondary_v1` requires the ordinary clock and exact Malice
models. Each native plunge command supplies a fresh explicit airborne-entry
profile (`plunge_inputs`): positive landing time and final height bonus0..1.
The native ATK coefficient multiplies by1+h once, then fans out unchanged to
living dummies. It does not infer geometry, flight or a maximum height. A600ms
profile contacts at653,784us and recovers at1,009,949us on positive HP, or881,249us
on miss. At1000ATK and half DEF, level1/8/9 with h=.5 gives357/612/660.75.

An admitted Critical Dodge pays the shared15 stamina and .9s cooldown, selects
front-dodge full recovery1.004760s, and grants exactly one next-Normal Riposte.
It retains the real Normal input, native PerfectEvade row/GAName, category Z and
invested Normal talent. Its .282278s contact and .903675s recovery gain one
.1287s causal pause on positive HP, irrespective of recipients. Other admitted
inputs/actions or field exit clear unused permission; death cancels the pending
owned counter. Ordinary or rejected dodges never grant it, and no parry20 is
inferred. Early cancellation/back-dodge selection and transformed counterparts
remain separate required owner paths.

Rust independently checks height/coefficient arithmetic, first-living/AoE
resource scopes, exact Malice, positive-HP hit-pause, one-microsecond recovery
boundaries, one-use/cancel/death semantics and failed stamina admission. A native
legal equipped two-cycle fixture checks aggregation, exact seeded replay and
exclusion of an early counter. Only the two consumed native packet rows and
Cycle fields were retained from cached `DT_SkillDamageData`; raw input remains
outside Git. Shared profile validation dispatches actual owner legality, while
primary coefficient scaling is a neutral runtime service also used by Baicang.
Public source lowering uses the selected native Blackbird form and plunge profile.

### Executable primary form and Burst

`blackbird.primary_form_v1` binds primary Blackbird transformation Burst to
the declared successful zero-delay mesh callback, native 100 Ultimate cost and
20s real cooldown. It requires ordinary clock, exact Malice, coordinated bird,
secondary action and qualified Support models.

The non-hybrid 21.5s on-field real form lifetime is captured at accepted commit;
its 9 primary contacts use native progress with .067s/.3 owner pauses and
5.118154s recovery progress (5.540254s total uninterrupted recovery). Form real
time includes those pauses, stops off field, resumes remaining time on entry and
ends before equal-time ordinary admission. Cooldowns, team buffs, reactions,
bird requests and committed packets keep their own real policies. UltraSkill.3
at progress 3.166927s applies the 15s team base-ATK 20% buff, replaced by A4's
25s plus general damage 12%; A6 independently grants own Psyche 25% for 25s.
Already applied buffs persist source death. A2 adds 20% base DEF only while the
form lives. R3 commit grants 20 exact Malice once, while ordinary gains including
A6 birds are suppressed in form.

Natural or death cleanup retains resources/cooldowns and committed packet identity.
Reached ordinary abilities and player inputs diagnose under this bounded model
until evolved action bindings are supplied; qualified transformed Support
diagnoses until transformed QTE is bound.

Rust tests independently check timing coordinates, hit-pause, 100 energy spend,
A2/A4/A6/R3 effects, off-field pausing/resuming, and input/Support guards. The
native [fixture](../../../../examples/blackbird-form-cycles.nte) checks 9-hit
Burst execution, off-field pause, on-field resume and expiration with seeded replay.

### Complete effect specification

**Mixed.** The [complete subsystem rules](malice-and-coordinated-damage.md#selected-complete-subsystem-rules) own every Malice extra, mark refresh/rearm, bird trigger/ICD/count, B2 Stain/Nova settlement/cap/filter/reapplication and cleanup rule. Do not use the historical alternative fixtures in place of those decisions. Roostcall applies one target movement restriction/70% speed reduction for5s, refresh not stack; source curve `Branch_SpeedCut=-.7`, duration5. Treat imprisonment as no target movement during that same window, with no extra damage/vulnerability or invented boss-immunity multiplier. Stationary dummies are already in range, so restriction itself contributes zero extra damage; explicit incoming-action scenarios must respect that state.

Transformation Burst **UltraSkill.3 at progress3.1669266** applies team ATK+.20 for15s (A4 replaces with25 and adds general damage+.12 for25). It is a buff event, **not a tenth primary hit**. Current living deployed teammates receive it, including off-field actors; it follows recipients through field changes, not a zone/proximity aura. Target cap1 per source, refresh on a later primary Burst; real clock, expiry independent of form pause, source death does not clear the already applied team buff (`GlobalCheckSourceDeath=false`). Later roster changes do not retroactively grant a cast buff. A6 own Psyche+.25 for25 is emitted at that same marker only with A6. A2 form DEF+.20 lasts only the form, A2 heal executes once at evolved Skill's progress0. A5/R6 lifetimes, recipients, replacement and after-hit order are in passives. No phantom form CritDamage is added from the zero curve. These source-bound effects and the linked finite subsystem constitute all damage-relevant owner buffs/debuffs; hitback/flight visuals are not damage packets.

### Complete recipient and dependency specification

**Mixed.** Declared scope is stationary all-in-range living dummies. Native DamageRange/Female051/BlackBird target-type bindings select an **AoE full-coefficient** train for Normal, Roostcall, plunge, riposte, QTE, both Skills and both Bursts. No located numeric recipient cap or splitting rule applies; each request hits each eligible dummy once, sorted by stable target index. Re-evaluate live eligibility at the scheduled primary hit; no target reselection creates additional packets. Each Malice extra captures the corresponding primary connected set and never widens it; one death removes just that pending recipient. Evolved Normal5's actor owns6 total pulses, not6 per target. Resource gains/spends, bird count/ICD and hit pause are once per causal packet, while HP, RES debuffs and Nova accumulators remain per recipient. Bird has one chosen trigger target followed by full AoE to all live dummies. Thus doubling dummies doubles uncapped HP contribution without doubling its5-budget, energy or Malice spend. Plunge height is an explicit0..1 bonus scenario; no coordinates/native geometry are required.

Use [damage/stat zones](../../shared/damage-and-stat-zones.md), [equipment build rules](../../shared/equipment-builds.md), [charge/gauges](../../shared/energy-and-gauges.md) and [cycles/Break](../../shared/cycles-and-break.md), with the linked B2 exceptions. QTE requires a qualified adjacent-element switch spending outgoing100Cycle; ordinary switch does not execute free QTE. Blackbird is Psyche, so pairs with Lakshana for Stain and Chaos for Nova; the triggering pair selects shared reaction owner/CI normally. Do not substitute Blackbird's CI for the actual Stain damage actor or ignore another owner's Nova-end/Discord listener. Break packets use listed Unbal once per recipient and the target's declared bar, immunity/acceptance/reset profile; no hidden universal broken-damage multiplier is introduced. A5 Chaos shred does not affect Incant packets or the fixed B2 settlement. Ordinary zero-bar training configuration has no Break event; a breakable profile must retain live gauge across both cycles. Select explicit proximity/critical-dodge/parry/incoming-damage conditions when used, with their real costs/time, rather than auto-proccing them in a solo dummy case.

Implementation owner state is Malice plus charge/Cycle ledgers, Normal stage/recovery-relative continuation deadline, separate ordinary/evolved cooldowns, live form/permission clocks, one30s evolved-Skill permission, one-use riposte authorization, bird generation/marks/count/deadline, per-source temporary effect handles and per-target B2 Nova accumulator/lineage. Queue entries preserve cast/packet/generation IDs, captured recipients/spend/coefficient/form, real delivery time and critical RNG. Source death disables fresh owner action/controller admission; already cast team buffs/pending packets follow their specified retention. Reactions settle HP before owner after-hit listeners; a delayed extra does not consume Malice again or recurse into itself. These rules name code-level state and causal order without requiring native dispatcher recovery.

### Complete implementation handoff

**Mixed; independent continuing-state arithmetic checked October19.** Implement the seven selected specifications above and the linked shared dependencies, retaining the named executable-slice evidence only as a bounded reference. No additional native-body recovery is required to implement this model. Factual certainty remains source values plus explicitly labeled lifecycle/timing/filter assumptions; full executable Blackbird parity is still unimplemented. The handoff is not a claim that the current engine produces the following oracle.

**Legal selected A0 build:** Blackbird80/80 BT6, all four talents8, B2/B4 enabled, no selected awakening; **Crime and Punishment `weapon:fork_twinbirds` T1 level80 BT6**, **Shadow Creed `set:Suit8`**. Declare set count 4 and supply console contributions through `add stats`; the numeric baseline below is preserved without modeling modules.

Static report: character668+weapon570=**baseATK1238**; AtkUp.85, flat420 →2710.3 exact, nearest-integer selected projection **2710**. HP21114, DEF884, CR.59, additional CD1.10, Charge Efficiency1, energy capacity100. [Twinbirds](../../weapons/fork_twinbirds/passive.md) source T1 adds **72 raw CI points** and **.20 Psyche damage**, which this older static compiler omits and the selected model must apply once. Successful **either** Burst commit grants living deployed team CD+.16 and additional wearer+.32 for15 real seconds, refresh not stack, before hits, persists switches; wearer receives+.48 total. The child's explicit15s is selected for the unresolved team custom duration as well; no invented CI-percent duration scaling. Removal/scene cleanup clears the source's passive handles, not retroactively past damage. R6, if selected, adds a distinct.30. [Shadow Creed](../../sets/shadow-creed/passive.md)2pc is already compiled;4pc selects its native **SKILL_REALFINISH** route: successful ordinary/evolved Skill full recovery grants/refreshes one self+.25baseATK for20s, including off-field continuation. Cancellation before successful recovery grants none; never both admission and finish. ATK increment309.5; team Burst increment247.6. These gear rules are active, not neutralized to simplify the oracle.

**Scenario and authored sequence:** one immortal stationary all-in-range dummy, zero RES, declared DEF factor.5, no Break bar, full HP, no incoming attacks/parry/dodge, successful zero-delay mesh callback, no preloaded energy, battle-entry Malice20 and Cycle0. Solo party means no qualified switch/bird trigger/Nova naturally occurs while B2/B4 remain enabled. Repeat the following at offsets **0 and75**, retain every ledger/effect/cooldown at75, simulate through **150s** including waits:

1. Five ordinary five-input chains start at0,5.94036065,11.88072130,17.82108195,23.76144260; within each, next stage starts exactly at its prior AnySkill recovery. All5 finish29.70180325. Wait to32.
2. Ordinary Skill32, full recovery33.4514912 and set buff through53.4514912. Burst34 spends100, form live34..55.5, all9 primary hits/recovery by39.540254; team ATK marker37.1669266 through52.1669266, weapon CD34..49. Wait to40.
3. Evolved chain40..46.2484877, permission at its stage5 marker; evolved Skill47..47.8941296 consumes it and refreshes set through67.8941296. Evolved chain48..54.2484877 grants a new permission; evolved Skill54.4..55.2941296 consumes it, refreshes set through75.2941296. Both use all4 main+4 Malice-extra packets. Last delayed extra is before55.5. No permission is conjured by the start of a chain.
4. Natural form expiry55.5; optional100-cost finisher is unaffordable after the primary Burst and is not admitted. Ordinary chain56..61.94036065; wait through75. At75 the set still has **.2941296s**, so the next period's first Normal hit75.14994475 receives309.5 ATK. Do not discard that effect at the period boundary. Primary Skill/Burst casts107/109 are beyond prior cooldowns, and the same form sequence repeats at115 onward.

Each75s period has **211 positive HP packets**:102 ordinary Normal,4 Skill,9 transformation Burst,40 evolved Normal,8 evolved Skill,48 separate Malice extras. Total422, including96 extras. Raw energy **178.23/period**; first pre-Burst105.38 reaches100 with5 whole overflow and.38 carry; subsequent72.85 ends **73+.23**. Second period ends **73+.46**, cumulative overflow83, total energy356.46=200spent+83overflow+73.46. No regen/reset/finisher-refund. Cycle ends100+.974 with146 overflow, then100+.948 with393 cumulative overflow; Skill's fill100 is distinct from its packet channels. Malice pre-transform60, two evolved chains+two evolved Skills spend49.999998 leaving10.000002, post-form ordinary chain adds20.000003 →**30.000005 at both boundaries**; the next period's ordinary hits legitimately cap it again. At both boundaries form/Skill permission are absent, ordinary cooldowns expired, all pending packet deliveries completed, and set remaining.2941296 is retained.

Independent event/Fraction audit  passed these values, legal recovery order, permission admission, partial-stock extra(.1→.0479871), bird5-count boundary and Nova cap. Reproduce expected ordinary packet damage as `coefficient*(2710 +1238*(.20*team_ATK_active+.25*set_active))*1.20*(1+.59*(1.10+.48*weapon_CD_active))*.5`; extras substitute their captured `spent*.479871` coefficient. No seeded critical result is asserted by this expectation oracle. **Expected total835904.2480269512, DPS5572.694986846342 over150s**; second period exceeds first by124.3250358 solely from the retained set buff on its first hit. Reading only a reset75s rotation would miss this discriminating carry-over. The receipt is ; all portable inputs, equation and schedule are recorded here without requiring that scratch file.

Implementation acceptance must additionally exercise the explicitly specified alternate branches: two-dummy equal HP scaling with single resource spend/pause; Roostcall/cancelled montage; partial/zero Malice; A1 seven-hit finisher with only one all-pool extra and a funded100energy scenario; A2 one4×baseATK heal; A5 post-hit Chaos shred; source-wide bird cap/2s ICD with another active ally; B2 excluded QTE/Nova, cap and one-child lineage; field-exit paused form but running team buff/mark; exact expiry ordering and CLI/browser same scenario. These are concrete rules with values above, not invitations to conduct a new research pass. Required effects cannot be replaced by missing-handler success.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,4,5,5 (seventeen hits). The chain loops, retaining native effective talent projection including resonance_3. Evolve forms and Malice transitions are not inferred from ordinary input execution.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel and animation intervals remain unresolved in this bounded caller-timing adapter; AoE now follows the grouped primary model below. Native animation/cancel evidence is not established by this model. Evidence: [`blackbird-normal-chain.json`](../../../../reviews/q10-game-action-binding/blackbird-normal-chain.json).

`blackbird.primary_packet_groups_v1` now binds each ordinary Normal/Skill/transformation-Burst packet to all living in-range dummies, with recipients ordered within that packet and one causal resource group. The admitted primary's coefficient/metadata are cloned for additional recipients; child packets retain their own native coefficients. Equal-time packets remain distinct groups. A dead target0 cannot suppress the resource opportunity of a surviving recipient. This is the selected no-geometry AoE model, not proof of native geometry or the still-pending action clocks.

Independent Rust checks retain17 Normal groups/19.564 raw energy,4 Skill groups/7.56 energy,9 Burst groups/no charge and the custom-primary counterexamples (Skill3→1500 per recipient, Burst3→1800 with teamATK, without reprojecting either into the native child coefficient). Native [two-train checks](../../../../examples/blackbird-primary-groups.nte) preserve42 primary groups/84 HP packets through aggregation and exact seed replay. These checks preserve existing caller timing; full kit/form timing and Malice remain required work.

## Executable primary Cycle

The existing `combat.packet_cycle_accrual_v1` ledger now consumes Blackbird's reached ordinary Normal/Skill HeterochromeAdd scalars through the owner registry. The five Normal source values are1.161/2.391/1.476/1.265/1.785 per packet; Skill is2.6 per packet. Existing Burst/QTE rows retain explicit0 rather than borrowing Ultimate charge. Only actual positive HP loss from a bound living-owner packet can grant, once per causal group; a dead target0 can fall through to another live recipient. Cycle gain efficiency, integer cap/overflow and exact carry stay separate from Ultimate's efficiency/team matrix.

Independent Rust checks give27.097 Cycle versus19.564 Ultimate per17-packet chain. Four continuing chains end at100Cycle +.388 carry with8 whole overflow, while Ultimate is78+.256; a qualified100Cycle switch and later first Normal preserve.549 carry and1 whole Cycle. At Cycle gain efficiency1.5 one chain gives40+.6455 while Ultimate stays19+.564. Explicit overlays must supply HeterochromeAdd, including0; missing fields diagnose. Evolved/alternate rows remain unbound until their actions execute.

Ordinary Skill activation also executes its independent Cycle fill under `blackbird.skill_cycle_fill_v1`: once after successful admission and before damage, fill the integer pool to100 even on miss, grant only its missing whole amount, preserve exact carry and prior overflow, and ignore packet Cycle gain efficiency. This does not grant Ultimate Energy. Its four connected2.6Cycle packets still accrue into the full pool's overflow. Independent checks retain1.161 from Normal, fill99, then two Skills10.4 each:100 current,20 overflow and.961 carry. Failed cooldown admission after a100Cycle switch spend leaves the pool empty. Native aggregation/replay of [the grouped two-cycle fixture](../../../../examples/blackbird-primary-groups.nte) retains100 current,47 overflow and.994 carry. Ordinary Skill's Malice grant and bird rearm remain required work; this fill does not certify those systems.

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`blackbird.skill_animation_profile_v1`, `blackbird.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Blackbird_Skill`), 4 direct hits land at 0.068s, 0.416s, 0.717s, and 0.902s (`Event.Montage.Player.Skill.1`); cancel windows activate at 0.289s (`BreakSkill_Jump`), 1.333s (`BreakSkill_AnySkill`), and 1.526s (`BreakSkill_Move`). For Burst (`Blackbird_Evolve_UltraSkill`), initial hit lands at 3.167s (`UltraSkill.3`) followed by 9 direct hits at 3.218s, 3.458s, 3.707s, 4.149s, 4.251s, 4.350s, 4.450s, 4.550s, and 4.650s (`UltraSkill.1`); cancel windows activate at 5.044s (`BreakSkill_Jump`, `BreakSkill_Skill`, `BreakSkill_Move`) and 5.118s (`BreakSkill_AnySkill`).
- **Multi-target delivery (`blackbird.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive the full packet train for Skill (4x `GE_Player_BlackBird_Skill_Damage`) and Burst (9x `GE_Player_BlackBird_UltraSkill_Damage`) across all living dummies in ascending target_index order at the primary impact time.

## Action packets and owner effects

- GA_BlackBird_Melee public primary packet is GE_Player_BlackBird_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.237/0.406/0.439; category A/PSYCHE metadata stays source-bound.
- GA_BlackBird_Skill public primary packet is GE_Player_BlackBird_Skill_Damage, delivering a 4-hit packet train (1 primary + 3 children). Native coefficients at Skill levels 1/8/9 are 1.000/1.714/1.851 per hit; category E/PSYCHE metadata stays source-bound. Selected A2 (Wayward Chronicle) doubles each native main coefficient through CoefModify; it does not add1.0 ATK or modify SkillAssist/Malice extras.
- GA_BlackBird_UltraSkill public primary packet is GE_Player_BlackBird_UltraSkill_Damage, delivering a 9-hit packet train (1 primary + 8 children). Native coefficients at Burst levels 1/8/9 are 0.467/0.800/0.864 per hit; category Q/PSYCHE metadata stays source-bound. on_cast activates Requies Animarum team ATK buff (+20% ATK for 15s base, 25s at A4). At resonance count3, doubles each native transformation main coefficient through CoefModify, independently of the individual A3 Malice curve.
- Resonance 3 activates at awakening >= 3 and adds +1 level to GA_BlackBird_Melee, GA_BlackBird_Skill, and GA_BlackBird_UltraSkill without manual overrides.
- A4 grants +12% team damage bonus and extends Requies Animarum to 25s; A5 reduces target Psyche/Lakshana/Chaos RES by 8% on hit; A6 grants +25% Psyche damage on UltraSkill and +30% Crit DMG on recipient.
- Canonical arbitrary awakening selections: Effect1/2 relative main-coefficient upgrades, Effect4 team duration/damage, Effect5 target resistance reduction and Effect6 self Psyche bonus use selected identities. Burst coefficient resonance_3 and team Crit DMG resonance_6 remain count-based. Existing unsupported owner mechanics remain pending; selection does not complete the kit.

Executable coefficient checks: native Skill8 with A2 gives3.428 per packet (6856 noncritical total at1000ATK/DEF factor.5); with R3's Skill9 investment it gives3.702 (7404total). Transformation Burst9 with R3 gives1.728, or9331.2 across9 packets at1200ATK/DEF factor.5. Evolved Skill/finisher modifier selection is packet-scoped and excludes their separate extra packets; this does not certify their still-pending action lifecycle. Rust tests cover the independent fractions and sibling exclusions; native CLI tests retain two Skill casts through seed aggregation and exact report replay.

## Executable B2 Stain

`blackbird.b2_stain_v1` binds the selected Stain rule when a living party Blackbird has breakthrough2. The registry supplies20s and an additive.10 base factor; one shared target status captures both at application. Its1.3 factor uses the eligible damage actor's live CI and the ordinary separate Stain zone. No extra damage packet or resource grant is emitted. Refresh replaces the existing deadline/modifier; expiry is half-open and an accepted generation survives Blackbird's field exit/defeat. A later application after his defeat uses the ordinary policy.

Rust independently checks `1000*1.3*E(108)=1397.5`, other-element/Mental exclusion, expiry at20s, a refreshed deadline30s, source defeat and strict-model/deadline failure before field/Cycle mutation. The [native fixture](../../../../examples/blackbird-stain-cycles.nte) spends two declared Cycle pools without resets; CLI aggregation and exact seed replay consume both applications. This acceptance covers Stain; the separate Nova implementation below owns its accumulator and end effects.

## Executable B2 Nova

`blackbird.b2_nova_v1` binds the selected accumulator/filter/cap model in [the subsystem rules](malice-and-coordinated-damage.md#stain-and-nova-enhancement). A living breakthrough2 Blackbird captures baseATK for each target/generation. Shared direct, reaction and party Break settlement report actual applied HP to the owner. The owner excludes the whole triggering Support/QTE command and children, ordinary Nova ticks and its own fixed settlement. A refreshed generation closes the old accumulation window at admission, preventing same-time new-cast damage from entering both.

After shared pending damage, the ended generation settles its already-scaled/capped value as a separate fixed Mental packet, attributed to Blackbird with the originating switch command. No second DEF/RES/crit/bonus calculation or packet resources apply. Zero accumulation emits no fixed HP packet. The initial generation may create exactly one child; an authored refresh wins over that child. Both generation deadlines and strict-model acceptance are checked before field/Cycle mutation. Target/source defeat suppresses the owner extra/reapplication while shared already-armed Nova follows its own lifecycle.

Independent Rust checks cover10000HP→2000 and5000HP→1000 across continuing generations, a10millionHP accumulator capped at1million for baseATK1000 despite liveATK9000 and90% Mental RES, triggering-QTE/Nova exclusions, per-target isolation, exact expiry, refresh priority and clipped fixed overkill. Separate integration checks retain party Break and unrelated Scorch HP; Aurelia's three500HP post-end Harmonics enter the child only and settle300 at10.2s. The [native two-cycle fixture](../../../../examples/blackbird-nova-cycles.nte) retains two authored generations and their two bounded children through CLI seed aggregation and exact replay. Native getter/writer/filter internals remain inferred as disclosed in the subsystem card; this does not complete Evolve/Malice, the bird controller or alternate actions.

## Recovered Stain and Nova extension parameters

**Source facts for PC_140/logic-566459; no runtime policy change.** The corrected reaction-extension acquisition reads all nested payloads of `DT_ReactionExtendDataTable` without the earlier skipped-struct warnings. Row **1042** compares logically equal to the frozen row. Its first entry binds `DamageUpRatio=0.10` under `Buff_BlackBird042_Passive1`; its second, newly mapped `AnHunReactionExpendSeventhData`, binds **`CumulativeDamageRatio=0.20`**, **`BaseAtkCapRatio=1000`**, `GE_Reaction_4_new_1042_Damage` and reapplication class `Buff_Reaction_4_new`, with the same passive condition and `ReactionAbilityID=GA_BlackBird_Passive_1`.

The frozen Reference guide description separately states Stain base bonus 30%/duration 20 seconds. For Nova it describes additional Mental damage equal to 20% of actual damage accumulated during Nova, excluding the triggering Support Skill and Nova's own damage, capped at **1,000 × Blackbird's base ATK**, unaffected by other damage buffs/debuffs, followed by one Nova reapplication that cannot recursively reapply again. These are described policies, not recovered admission/filter/dispatch code. The configured Stain 0.10 input does not itself prove an absolute 30% result or the 20-second clock. The later [shared Stain cue](../../shared/cycle-source-findings.md#stain-time-change-cue-consumer-recovered-october-18) now locates a start-time request with input 8 behind a native eligibility gate and time-change removal/reapplication; its native predicate, clock and re-entry remain unresolved. Preserve the cap's stated multiplication unit; it is not a 1,000% label.

The named extra GE uses native `FixedDamageExecution` with a source-snapshot additive `SkillCoefficient` custom magnitude `Cau_Reaction4_1042` and a separate `ExecutionDamageType` modifier. Its asset tags include `Ability.Player.ReactionDamage`; class/field names do not independently prove defense, crit, bonus or callback eligibility. Native cumulative-damage filtering/capture, cap settlement, reaction owner, end timing and the one-reapplication guard remain separate from these bindings. The [unknown owner](#unknown-execution-policies) retains those decisions.

`Cau_Reaction4_1042`, `/2/ScriptBytecode`, resolves the ASC of **`HitActor`** at 108, checks it at 138/167, calls native **`GetPendingSettleDamageValue`** at 181, and returns that value via 231/258. It does not multiply by the row ratio or cap inside this graph. The initialized curve-handle locals at 0/54 do not supply a second scaling operation. Continue at the native writer/capture of this pending amount; applying the 20% ratio again to the returned amount is not established by the calculator name.

**Independent description-level counterexample:** explicit accumulated damage 10,000,000, ratio 0.20 and base ATK 1,000 produce `min(2,000,000, 1,000 × 1,000) = 1,000,000`. Reinterpreting the cap as 1,000% would produce only 10,000. This checks the stated model's units; it does not certify native damage settlement or grant the extra packet ordinary buffs.

## Ordinary-hit Ultimate Energy

The current `blackbird.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 19.564, Skill 7.56 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`blackbird-aurelia-haniel-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/blackbird-aurelia-haniel-ordinary-hits.json).

### A0 zero-start Normal → Skill → Burst → Normal continuation

The focused scenario starts at zero energy, delivers six complete A0 Normal chains and one A0 Skill, then admits Blackbird's 100-energy Burst from the resulting owner pool. After the Burst's 2s recovery, a seventh five-input Normal chain begins at 27.2s and finishes at 30.4s, while the A0 Requies Animarum +20% team ATK effect remains active through 40.2s. It reuses the existing `blackbird.ordinary_hit_energy_v1` adapter and its positive applied-HP-loss, recipient, carry, capacity, and overflow rules; Normal and Skill hits grant ChargeAdd, while Burst spends energy and grants none. The timeline follows the shared `actions.caller_timing_v1` research profile: 800ms per Normal input, Skill at 24s, Burst at 25.2s after the 1.2s Skill recovery, and continuation inputs at 27.2–30.4s. The shared timing gate admits the Burst at the exact Skill recovery deadline. At level 8 and 1000 ATK against DEF factor 0.5/RES 0, the independent damage oracle is 6 × 4735 Normal + 3428 Skill + 4320 Burst + 5682 buffed Normal = 41840 across 132 hits, or 10460/9 damage per second over the 36s horizon. Frozen ChargeAdd gives 7 × 19564m + 7560m = 144508m: the owner gets 144 whole energy, carries 127/250, overflows 24 at capacity 100, spends 100, and ends at 20; a living ally gets 86 and carries 881/1250. See [`blackbird-continuing-rotation.json`](../../../../reviews/q10-game-action-binding/blackbird-continuing-rotation.json) for the exact scenario and independent packet list.

This slice checks a zero-energy A0 rotation using the existing adapter policy; it does not establish native caller eligibility or cadence. Awakening-specific damage and resources, Evolve/Malice behavior, alternate forms, other Charge Efficiency values, and other energy sources remain outside this scenario.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/blackbird-action-binding-model.json`](../../../../reviews/q10-game-action-binding/blackbird-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/blackbird_actions.rs), [`blackbird-normal-chain.json`](../../../../reviews/q10-game-action-binding/blackbird-normal-chain.json), [`blackbird-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/blackbird-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_blackbird.json`, `data/extracted/neverness_blackbird.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/blackbird.rs) (original `DT_SkillDamageData` export is cache-only), `1042AwakenEffect` (cache-only awakening source), [`blackbird/parameters.rs`](../../../../crates/content/src/characters/blackbird/parameters.rs) (consumed typed parameters; original `DT_BlackBirdEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Executable transformed secondary actions and control events

All Malice-extra curves now use the shared effective Normal coordinate, including
count-based R3, independently of the primary action's invested talent. Individual
A3 adds only its separate curve. Independent rate checks use Normal8 and a
.541591 spend: A3 alone gives.389840451346; count3 with A3 gives.42102742749;
count3 without A3 gives.28068495166. Native transformed QTE keeps Support8 while
R3 advances purchased Normal1 to2 for its extra, yielding coefficient.755894916.
Aggregation/replay retain both independent talent identities.

Malice extras retain the entire positively connected primary recipient set.
One atomic spend captures one coefficient and extra packet identity; each
connected recipient receives that coefficient at the original delayed deadline.
A same-time settlement callback releases the temporary capture after all parent
recipients settle. Later target death cancels only that recipient without refund
or retargeting; zero-HP recipients never join the captured set. Three-dummy
checks retain69 extra deliveries from20 paid Normal and3 paid Skill groups,
including the last1.500001 Malice shared across all3 recipients. Those deliveries
leave total spend20 and charge26+.643, without multiplied resource grants.

`blackbird.evolved_secondary_v1` binds transformed plunge and the paid next-Normal
Critical Riposte to their native X/Z rows, exact charge/Cycle/Malice and owner
hit-pause. Plunge retains the explicit landing interval and height multiplier.
`blackbird.evolved_support_v1` selects transformed QTE only through a qualified
switch while form remains live; its H primary spends1.666435 Malice and creates
one category-A extra. It has no ordinary B4 marker. Support uses its own talent,
while the extra reads Normal talent. Separate recovery and switch locks remain.

`blackbird.branches_v1` selects Roostcall after ordinary Normal1/2/3, Bestowed
Grace after transformed Normal2, and Tidings during Bestowed Grace or transformed
Skill. The canonical Normal command carries `hold_inputs[index]` with
`duration_us >= 200000` and `moving=false`; direct branch packet inputs reject.
The authored command must reach the previous montage's earliest cancel gate.
It replaces that montage immediately, pays the full hold interval on the real
axis, and starts branch progress after release. This release policy is inferred;
the native evolved hold threshold remains unknown. Primary damage uses invested
Normal talent, full AoE and source packet multiplicities, with exact Malice gains
or spends. Captured extras survive replacement.

`blackbird.cancel_gates_v1` resolves the listed Jump/Skill/Move gates through the
owner's hit-pause tracker, admitting the earlier of that gate and full recovery.
An admitted cancel removes pending montage-bound requests. Melee5's six-hit actor
continues once its spawn coordinate has elapsed, including when its first hit
applies zero HP. Field exit follows the same ownership policy. Dodge uses the
selected front profile, paying stamina and preserving its separate Jump/Skill
gates; an ordinary dodge supplies no riposte authorization.

Typed request `parry_events` (`At` or `AfterAction`) declare successful owner parries under
`blackbird.explicit_parry_v1`. A living active Blackbird gains20 Malice outside
form, or refreshes cap1 thirty-second Skill permission in form. Neither damage
nor dodge success is inferred. Transformed Skill consumes permission once and
routes the A2 base-ATK heal through shared healing bonuses, MaxHP, HP callbacks
and a causal healing receipt; canceling into Tidings cannot repeat the heal.

Typed request `combat_transition_events` declare alternating leave/re-entry,
starting in combat, under `blackbird.combat_transitions_v1`. Leave schedules one
20-Malice grant after4 seconds; re-entry invalidates it and grants a distinct20.
Both gains respect cap60 and form suppression. Dead owners receive no pending
grant. These events do not infer enemy state or actions. Lists are monotonic,
owner/horizon validated, count against work limits and retain event indices in
typed report receipts. Equal-time actions run before these ProcCallback events;
the leave deadline's BuffTickOrExpiry runs before a same-time re-entry.

Independent Rust checks establish: stage3 Jump admits at1,977,027us after the
first contact's29,700us pause and cancels its other three contacts; Roostcall
released after a200,000us hold hits at614,725us and leaves24.980996 Malice across
one or three dummies; chained evolved branches retain the canceled parent's
captured extra and leave14.460769 Malice. A2 healing with baseATK1000 and50%
healing bonus supplies6000 once. Native aggregation/replay retains two continuing
transformed branch/plunge/riposte cycles, absolute parry/combat events, seeded
streams and conserved resources on a legal A0 build. Shared source lowering,
periodic parry sampling and final browser parity remain their native/browser owners.

The earlier packet, primary-form, bird, Stain and Nova sections retain their
separate source rationale and independent checks. Historical source-certainty
gaps below do not mark these executable product models unsupported.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/blackbird). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Blackbird (`GA_BlackBird_*`):

| Action / damage row (`GE_Player_BlackBird_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.237–0.487 / 0.406–0.835 / 0.439–0.901 | 19.564 total / 27.135 total / 3.421 total |
| Plunge `AirAttack_Damage` | X | 0.476 / 0.816 / 0.881 | 1.680 / 2.300 / 0.300 |
| Hold branch `Branch1_Damage` | A | 1.139 / 1.952 / 2.108 | 4.080 / 5.600 / 0.700 |
| Critical Riposte `PerfectEvade_Damage` | Z | 1.275 / 2.185 / 2.360 | 3.000 / 4.200 / 2.500 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 7.560 / 0.000 / 2.500 |
| Skill `Skill_Damage` (4 hits) | E | 1.000 / 1.714 / 1.851 | 1.890 / 2.600 / 0.325 per hit |
| Skill assist `SkillAssist_Damage` | E | 2.000 / 3.428 / 3.702 | 2.000 / 3.340 / 0.400 |
| Burst `UltraSkill_Damage` (9 hits) | Q | 0.467 / 0.800 / 0.864 | 0.000 / 0.000 / 0.278 per hit |
| Evolve Burst `Evolve_UltraSkill_Damage` | Q | 0.900 / 1.542 / 1.666 | 0.000 / 0.000 / 0.357 |

Critical Riposte uses GAName `GA_BlackBird_PerfectEvade`, while Support QTE uses `GA_BlackBird_QTE`. All damage rows deal PSYCHE damage. In Evolve state (`GA_BlackBird_Evolve_*`), distinct attack rows (`Evolve_Melee1..5`, `Evolve_Branch1..2`, `Evolve_AirAttack`, `Evolve_Skill`, `Evolve_UltraSkill`) apply separate coefficients and resource channels.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_BlackBird_Passive_1` binds `Buff_BlackBird042_Passive1`. `DT_ReactionExtendDataTable` row 1042 supplies:
  1. `DamageUpRatio=0.10` under `Buff_BlackBird042_Passive1` (Stain bonus).
  2. `CumulativeDamageRatio=0.20`, `BaseAtkCapRatio=1000`, `GE_Reaction_4_new_1042_Damage` and reapplication class `Buff_Reaction_4_new` under `Buff_BlackBird042_Passive1`. This supports Blackbird's Nova reaction damage: 20% of accumulated damage during Nova capped at 1000 × base ATK.
- **Passive 2 (Breakthrough 4):** `GA_BlackBird_Passive_2` binds `Buff_BlackBird042_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1042AwakenEffect#/rows/1042/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Consumes all Malice after Witch of Deliverance casts Flammae in Unum; increases Flames of Deliverance's DMG Ratio by 150%.
- **A2:** Netherplume Rite DMG Ratio +100%; Witch of Deliverance DEF +20%; successfully casting Flames of Deliverance restores HP equal to 400% Base ATK.
- **A3:** Increases DMG Ratio of attacks consuming Malice by +50%.
- **A4:** Extends team ATK bonus (Requies Animarum) to 25s and grants +12% team damage bonus.
- **A5:** Reduces target Psyche, Lakshana and Chaos RES by 8% for 25s after Blackbird hits and deals damage.
- **A6:** Casting Requies Animarum grants the caster +25% Psyche DMG for 25s; Forsaken Bird coordinated attacks grant four Malice while Blackbird is outside Witch of Deliverance.
- **Resonance 3:** The modification list grants +1 to `GA_BlackBird_Melee`, `GA_BlackBird_Skill` and `GA_BlackBird_UltraSkill`, plus `Buff_BlackBird042_Level3_extra`. Its description additionally specifies Requies Animarum DMG Ratio +100%, Flammae in Unum +50%, and 20 Malice after Requies Animarum. Effective evolved-action level sharing remains a native binding question.
- **Resonance 6:** +30% entire team Crit DMG while Blackbird is on the team; the `1042AwakenEffect` description does not require being on field.

### Evolved-Skill healing basis and native resource boundary, October 18

The expansion's warning-free `Calc_BlackBird_Evolve_Skill_Treat:CustomCalculateBaseMagnitude` (`#/2`, statements 122..522) evaluates `DT_BlackBirdEffectFigure:Buff_BlackBird042_Evolve_Skill_Treat_Level2` at coordinate zero, requires **owner player `Buff_BlackBird042_Level2`**, and multiplies the row's **4.0** by that owner's **GetAtkBase**. It does not use final GetAtk, target MaxHP or the extra-damage coefficient. With base ATK 1000 and final ATK 1500 its successful magnitude is **4000**, not 6000. Native caller, recipient, healing modifiers/clamp and timing remain separate; unsupported failure paths are not successful zero heals.

The expansion's `Player_042_BlackBird#/0` is a `BlackbirdLogicObj` instance with **no serialized Properties**; the player export has five warnings. It supplies no newly recovered native Malice mutation, clamp or coordination body. Reuse the [existing Malice/source findings](malice-and-coordinated-damage.md) and their reached getters instead of interpreting that empty object or another export pass as a resource resolution. Remaining owner-specific capture/order and bird dispatch questions stay open.

### Passive description coverage, reviewed October 18

Saved description source: `guide_blackbird.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Thrushsong describes Stain base bonus 30% for 20s, and a Nova-end extra packet equal to 20% of damage accumulated during Nova, excluding the triggering Support and Nova itself, capped at 1000 times base ATK. It also describes one nonrecursive Nova reapplication and no further buff/debuff scaling of that extra packet. Ever Onward describes 20 Malice on Judgment Upon Me. These are the stated eligibility/cap rules; the linked native pending-damage accumulator, exclusions, delivered reapplication and Malice ledger remain separate unresolved consumers.

### Ordinary A2 and resonance coefficient consumers recovered, October 18

Initial `GE_Player_BlackBird_Skill_Damage#/2/Properties/CustomApplicationModifierInfos` binds **CoefModify** to `Calc_BlackBird_Level2_SkillCofeAdd`; ordinary `GE_Player_BlackBird_UltraSkill_Damage#/2` binds the same stat to `Calc_BlackBird_Level3Extra_UltraSkillCofeAdd`, both multiplier 1. Their warning-free functions (`#/2`) cast OwnerActor to player, require **Buff_BlackBird042_Level2** or **Buff_BlackBird042_Level3_extra**, respectively, and return the matching coordinate-zero row (**1.0** for both), explicit zero without the marker. The latter is resonance, not individual A3. This locates the selected ordinary coefficient-modification inputs; the native CoefModify composition remains distinct from an additive general damage bonus or a direct +1 ATK coefficient. Existing evolved Malice helpers are already covered by the linked subsystem and were not replaced by these ordinary consumers.

### Evolved coefficients, DEF and mode-dependent duration recovered, October 18

Expansion `GE_Player_BlackBird_Evolve_Skill_Damage#/2` binds **CoefModify** to `Calc_BlackBird_Level1_SkillEvolveCofe`, returning **1.5** with owner `Buff_BlackBird042_Level1`; evolved UltraSkill's `/2` binds it to `Calc_BlackBird_Level3Extra_UltraSkillEvolveCofeAdd`, returning **0.5** with `Buff_BlackBird042_Level3_extra`. Both helpers (`#/2`, warning-free) explicitly return zero without their selected marker. Ordinary A2/ordinary resonance inputs above remain distinct.

`Buff_BlackBird042_UltraSkill_Transform#/2/Properties/Modifiers` binds additive **DefUp** to `Calc_BlackBird_Level2_DefUP`: successful A2-marker path returns **0.2**, absent marker zero. The separate `...Transform_UltraSkill#/2` binds **CritDamageBase** to `Calc_BlackBird_Evolve_CritDamUP`, requiring the owner's passive2 and reading **Buff_BlackBird042_UltraSkill_Transform_CritDamUP=0**. That named calculator is not a hidden 30% critical-damage grant; the team resonance carrier is separate.

The main Transform carrier and its UI carrier use `Calc_BlackBird_Transform`; the separate Transform_UltraSkill carrier uses `Calc_BlackBird_Transform_UltraSkill`. Both warning-free `#/2` helpers read base **16** from `Buff_BlackBird042_UltraSkill_Transform` at coordinate 0. **IsHybridMode(OwnerActor)** selects 16 in both. The other branch adds **5.5** for main/UI (**21.5**) but **10** for the separate UltraSkill carrier (**26**). This locates three configured durations and their mode branch; UI duration is not interchangeable with every transformation timer. Native mode selection, state exit, Malice settlement and actual clocks remain open.

### Skill reset and Burst transformation callback recovered, October 18

Initial warning-free **GA_BlackBird_Skill:K2_ActivateAbility** (`#/3` → `/2`, 10..281) obtains the owning player's active BlackbirdLogicObj, requests **ClearAssistPotentialCount()**, then calls the montage parent. Failed player/logic casts return without the reached parent call. This is a caller for the assist-potential counter, not evidence that every Skill clears all Malice. The named native method owns its ledger semantics.

Ordinary **GA_BlackBird_UltraSkill** (`#/2..4`; ) obtains/caches that logic object, sets **bWaitingForTransformMeshApplied=true**, binds **OnTransformMeshApplied**, requests **Transform(player_042_Blackbird_evolve_skin)**, then registers a **2.0 Delay** fallback. On the mesh-applied callback it acts only while waiting, clears that flag, and calls the montage parent only if **IsTransforming() is true**; otherwise it ends the ability. If the delayed callback instead finds the flag still true, it clears it, requests **RestoreTransform()** and ends. A later delayed callback after a successful mesh callback does nothing on this branch. This establishes a completion-gated start and timeout recovery request, not a universal two-second transform animation or immediate Burst damage. Native callback success, deadline clock, interruption cleanup and resource admission remain open.

After the successful callback reaches the parent, **Buff_BlackBird042_Level3_extra** gates coordinate-zero row **Buff_BlackBird042_Level3_Extra_Add=20** and request **AddOrRemovePotential(20,true)**. This matches the separately selected resonance_3 description's 20 Malice, not individual A3's 50% consuming-attack coefficient. Native grant/cap and current transform-state interaction remain with the method; this source does not move the grant to initial activation before the mesh callback.

### GSkill flight admission is separate from combat transformation, October 18

Initial warning-free **GA_BlackBird_GSkill:K2_CanActivateAbility** (`#/3`) uses ActorInfo.OwnerActor and on its valid player path requires **NOT GetPlayerFightState()**, absence of **State.Property.CantTransformFly**, and movement-mode predicate **mode==3 OR (mode==6 AND valid custom-submode==20)**. A failed gameplay-tag interface cast treats the forbidden-tag test as false; a failed player cast leaves the separate unassigned-return path. These are the reached predicate inputs, not a recovered implementation of fight-state classification or a combat Burst/form entry. Its activation/end graph requests LongEvade_SpeedUP addition/removal and interaction-UI tags; those movement/visual helpers do not establish Malice spending or combat damage. Native stamina/admission and live transition timing remain open. 
### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_BlackBird_Melee#/5; Branch.1 | BlackBirdBranch1 | (0.0, 0.0, 0.0, 2.0, 1, true) | GE_Player_BlackBird_Branch1_Damage |
| GA_BlackBird_Evolve_Melee#/5; Melee.5 | PlayerBlackBirdeEvolveMelee5 | (0.0, 0.0, 0.12, 1.0, 6, true) | GE_Player_BlackBird_Evolve_Melee5_Extra, GE_Player_BlackBird_Evolve_Melee5_Damage |

Evolved Melee.5 binds period 0.12, lifetime 1 and cap 6 to a container containing both Evolve_Melee5_Extra and Evolve_Melee5_Damage, plus control. Preserve the two effect classes and their native conditions; do not infer twelve hits from six triggers or an additional spawn hit. Ordinary Branch.1 separately carries its SpeedCut debuff.

### Evolved Skill permission, consumption event and entry selector recovered, October 18

Warning-free **GA_BlackBird_Evolve_Skill:K2_CanActivateAbility#/2** checks avatar player's ASC for **Buff_BlackBird042_Evolve_Melee5_Skill**, returning that predicate with empty RelevantTags on a successful cast. The permission carrier (`#/2`) is HasDuration **30**, source-cap1. Its actual producers are **Evolve_Melee#/4/EffectContainerMap/5**, **Melee.5_1 → UseOwner**, and **Buff_BlackBird042_UltraSkill_Transform#/2**, **PARRY_ATTACK → add**; the transform carrier's FINISH removes it. Thus completing the reached Normal event or a delivered parry callback requests permission; this is not an unconditional free evolved Skill.

The evolved Skill's **#/3/EffectContainerMap** requests separate Extra_Damage/direct Damage plus HitBack on **Skill.1**, then owner **Melee5_Skill_delete and Evolve_Skill_Treat** on **Skill.2**. The warning-free delete carrier (`#/2`) requests removal of all permission stacks on **BEGIN**, not on its configured 0.1 expiry. The Treat carrier (`#/1`) binds **CustomReplyHealthExecution** with additive Source/snapshot SkillCoefficient through **Calc_BlackBird_Evolve_Skill_Treat**, duration 0.1/period0.25. Native notify delivery/cancel ordering, heal settlement and subsequent admission remain open; an interrupted cast before Skill.2 is not proven to consume the marker at activation. The separate CooldownDuration multiplier **0.0625** against **CoolDwonCurveTable:GA_Female_051_Skill** is not a second permission count or a universal zero-cost rule.

**GA_BlackBird_Appear:K2_GetTransformMontageSectionName#/1** queries the owning player's active **BlackbirdLogicObj.IsTransforming()**. True appends **_tran** to the supplied SectionName; false or failed player/logic casts returns the unchanged name. The separate ordinary **GA_BlackBird_Evade:K2_TriggerJumpSection#/5 → /2** first returns early while its avatar ASC has **Buff_BlackBird042_UltraSkill_Transform**. Otherwise its valid-player path requests **LongEvade_SpeedUP**, then long-evade section handling; exact LongFrontEvadeEndHasMoveInput starts sprint and ends, exact LongFrontEvadeEnd stops movement and ends. Its activation's CantTransformFly path only displays a HUD message. **Evolve_Evade** instead only forwards activation/end to the relevant parents in its reached overrides. These distinguish actual form/section routes without recovering the native Malice ledger or input arbitration. All cited assets are warning-free. Sources: `additional-evade-and-pan-functions.txt`, `additional-form-and-appear-config.txt`, `additional-form-marker-effects.txt`, `additional-form-and-appear-callers.txt` and `additional-appear-marker-callers.txt`.

### Burst buff alternatives and held-input requirement recovered, October 18

Warning-free **GA_BlackBird_UltraSkill#/5/EffectContainerMap/2**, **UltraSkill.3 → UseOwner**, requests ordinary **Evolve_UltraSkill_AtkUP**, **AtkUP_Level4**, **Level6_DamUP**, the CD carrier and **DamUP_Level4**. Ordinary ATK's actual condition **Con_BlackBird1042_Level4_1:CanApplyGameplayEffectContent#/2** requires **SourceActor player lacks both Level4 and AtkUP_Level4**; it is not a positive A4 test. The Level4 ATK and damage carriers instead require **SourceActor player has Level4**. Ordinary ATK is additive **AtkUp 0.2 for configured 15**; A4 ATK is **AtkUp 0.2 for configured 25**, and its BEGIN removes all ordinary ATK stacks. A4 damage separately gives additive **DamageUpGeneralBase 0.12 for configured 25**. All three aggregate by target, cap 1, and serialize an GlobalGameplayEffectComponent with CheckSourceDeath=false. Their source/recipient conditions, global component and removal request support alternative ordinary/A4 ATK carriers, not an unconditional combined 40% ATK bonus; actual global recipients, clock and native grant order remain open. Values: `DT_BlackBirdEffectFigure` matching AtkUP/Time and DamUP_Level4 rows.

Other warning-free **Con_BlackBird1042_*#/2** predicates preserve distinct actors and selected markers: **Level1** tests TargetActor player (bound to **Evolve_UltraSkill_Cut#/3**), whereas **Level1_1** tests SourceActor player (bound to **GE_Player_BlackBird_Evolve_UltraSkill_Damage_checkLV1#/2**). **Level3Extra** tests SourceActor **Level3_extra**, and gates both misleadingly named **Level1_Add#/3** and **Level3_Extra_Add#/3**. Individual Level3 in the Malice coefficient helper remains different. **Level2**, **Level6** and **Passive2** test their SourceActor markers and gate Skill_Bati, Level6_DamUP and QTE_Add, respectively. These admissions do not themselves implement the named Malice grants/cuts.

**NeedPressRequirement_BlackBird:CanApplyEffect#/1** gets the mesh owner's ASC, obtains its primary **GA_BlackBird_Evolve_Melee** instance, and returns **IsInputPressed()**. A failed owner-to-player cast explicitly returns true; there is no separate valid-ability-instance branch. Its actual caller is **Blackbird_Evolve_Skill#/20**, a BP_JumpSection application requirement; particle-notify read errors are scoped to other exports. This is a held evolved-Normal input query during a Skill montage, not a measured hold duration or unconditional input consumption. Sources: `blackbird-coefficient-condition-callers.txt`, `blackbird-selected-conditional-carriers.txt`, `blackbird-a4-carrier.txt`, `jiuyuan-and-blackbird-requirement-callers.txt` and `tenth-selected-carrier-callers.txt`.

### Transform-cue exit and Branch source-context paths recovered, October 18

Newly read **GC_BlackBird_UltraSkill_Transform:OnRemoveEvent#/3 → ExecuteUbergraph#/1** uses MyTarget player; on its **IsAuthorityRole + active BlackbirdLogicObj** path it requests **RestoreTransform only if NOT IsPendingTransformMeshSwap()**. Separate branches restore weapon visibility/movement settings and remove all Transform_UI, without reusing that authority predicate. OnWhileActiveEvent#/4 instead removes all **Evolve_UltraSkill2_CD** and requests Transform_UI unless IsCloneScene(player state). These are concrete form cleanup/UI requests, not newly recovered Malice spending, restoration completion or one common timer for all form carriers. The main Transform carrier's warning-free #/2 names GameplayCue.Display.BlackBird.UltraSkillTransform; the subsequent HaloBuffActor recovery rereads that cue without its initial tail warning and confirms the matching configured class tag. Source:  and `cue-blackbird-haniel-tag-callers.txt`.

The matching **GC_BlackBird_Branch_SpeedCut:OnWhileActiveEvent#/2 → ExecuteUbergraph#/1** casts MyTarget to AICharacter, reads **EffectCauser from its instance Parameters variable**, casts that causer to player and requests **Branch_SpeedCut1 on the AI attributed to that player**. OnWhile stores the incoming arguments in **K2Node_Event_Parameters**, but the reached BreakGameplayCueParameters reads **Parameters**, not that event local; its recovered derived CDO #/3 has an empty EffectContext/default parameters. No explicit assignment from the incoming parameters to that instance variable appears in the function. Preserve this distinction and native initialization dependency instead of silently using SourceObject or InstigatorActor. **Buff_BlackBird042_Branch_SpeedCut#/2** supplies the BranchSpeedCut cue tag; the repaired cue now confirms that same tag without the initial inherited-tail warning. Source: `cue-blackbird-branch-functions.txt`, with the later `roster-cue-halo-extraction.json`. Bird count/Malice and OnCoopMechanismCleared cleanup are owned by [the updated subsystem](malice-and-coordinated-damage.md#bird-count-malice-requests-and-coordination-cleanup-recovered-october-18).

### Separate transformation-transition cue checked, October 18

The repaired **Buff_BlackBird042_UltraSkill_Transform_test#/1 → GameplayCue.Display.BlackBird.UltraTransformtest → GC_BlackBird_UltraSkill_Transform_test** join is present despite its test suffix. The carrier has HasDuration 10000, target cap1; that number is not a proved natural form duration. OnWhile **#/4 → graph#/2:254** requires MyTarget player and IsAuthorityRole, then binds **OnBuildCharacterMeshFinish → Transform** and **OnActivateAbilityEvent → activeability**. It does not directly call Transform at entry. The mesh-finish callback obtains that event player's BlackbirdLogicObj, returns while IsTransforming, otherwise requests **Transform(player_042_Blackbird_evolve_skin)** and **Delay 0.1(true)** before pausing **ordinary Transform**, requesting **Transform_UltraSkill**, then pausing the latter; both timer-pause calls pass **true, selector 2**. The [latent-delay parameter metadata](../../shared/action-timing.md#release-task-and-latent-delay-declarations) identifies the preceding delay as **Duration=0.1, AcceptTimeDilation=true**, not a proven fixed real-time transition wait.

The first delivered activation callback resolves the ability's **ActorInfo.OwnerActor as player**, requires authority, unbinds that player's activation delegate and requests removal of Transform_test, without a Skill/Burst-class predicate. OnRemove **#/3 → 777** unbinds the mesh-finish delegate and requests both timers unpaused (**false,2**), but does not explicitly unbind the activation delegate there. The separate Transform_UltraSkill carrier grants **GA_BlackBird_Evolve_UltraSkill**, InputID 3, CancelAbilityImmediately and BuffCanBePausedByClient; its custom duration/crit modifier are separate consumers. The selected Ability_042 reference scan finds no additional serialized producer of Transform_test, so this is a bound transition-carrier route with unresolved admission/current applicability, not permission to add another automatic transformation to the kit. [Native parameter metadata](../../shared/action-timing.md#time-function-parameter-metadata) names the third input **PausedType=2**; its enum identity and arbitration remain open alongside mesh/event ordering, removal/re-entry and successful grants. Sources: `shared-transform-control-carriers.txt`, `blackbird-transform-test-producer-refs.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`; the cited reads are warning-free.

## Unknown execution policies

Historical native-certainty notes only. Implementation decisions for these questions are finalized in the selected handoff above; they do not reopen collection or block this owner's8/8 model readiness.

- **Real timing/admission:** Montage notify coordinates for Skill (`Blackbird_Skill`) and Burst (`Blackbird_Evolve_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Evolve & Malice transition ordering:** Cycle-gauge dispatch, Malice consumption rates, and bird summon recipient selection remain open (see [Malice and coordinated damage](malice-and-coordinated-damage.md)).
- **Nova accumulation & cap settlement:** Calculation graph `Cau_Reaction4_1042` calls `GetPendingSettleDamageValue`; native accumulation filter and cap evaluation details remain an open native consumer question.
- **Spatial geometry & targeting:** Skill AoE and multi-target Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Normal chain timing/reset and Evolve/Malice transition ordering remain unknown; see the linked Malice card for cycle-gauge dispatch, captured Malice, extra-GE calls, and bird recipient-cap/selection/distribution questions.
- The [reaction-extension source owner](#recovered-stain-and-nova-extension-parameters) now binds both serialized entries, the extra GE and its HitActor pending-settlement getter. Resolve native Stain composition/duration and Nova accumulation filters, writer/read capture, exclusions, cap/rounding, end timing, reapplication guard and packet eligibility. The helper only returns the native pending value; adding another ratio operation requires its own evidence, including what that value contains. Source values are available after the skipped-struct repair.
- Ordinary-hit energy callers remain a tested hypothesis; frozen ChargeAdd rows establish constants only.

## Responsibility cards

- [malice-and-coordinated-damage](malice-and-coordinated-damage.md#unknown-execution-policies) contains the findings and questions for that distinct subsystem.

## Prior findings

- [`reviews/q10-game-action-binding/blackbird-action-binding-model.json`](../../../../reviews/q10-game-action-binding/blackbird-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/blackbird_actions.rs), [`blackbird-normal-chain.json`](../../../../reviews/q10-game-action-binding/blackbird-normal-chain.json), [`blackbird-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/blackbird-skill-burst-packets.json).

## Research history

- Blackbird8/8 selected handoff complete. Next campaign owner: **Chaos, first open packets block**. Reuse its existing dossiers before acquisitions.
- Blackbird methods already exhausted: original/supplement client GAs/notifies/bytecode, reaction-extension decode, public guide/frozen numbers and prior video leads. No more native-writer/mesh polishing during collection; the explicit checked models above are ready for later implementation.


Source `dash[parry=true]` pays the existing Dodge cost and dispatches the explicit
parry after that actual input, including after an implicit switch. The flag is
separate from Critical Dodge. Continuing ordinary-source checks retain the
Malice path; transformed source reaches one Goner through its30s permission.
Consumed or expired permission is an ordinary typed resource-admission failure,
with the same readiness predicate and failed native replay, rather than an
internal execution error. Defensive cast validation still guards the invariant.

## Relative combat timer binding

`Blackbird_Evolve_UltraSkill` Common.EndAbilitySkill at 5.021767 montage seconds. This evidence binds only the evolved Ultimate route. The ordinary transformation Ultimate consumes its actual owner action-completion fallback (native profile base recovery5.118154 seconds plus connected local hitlag), rather than borrowing the evolved montage marker. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
