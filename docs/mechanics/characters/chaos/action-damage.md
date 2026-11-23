# Chaos action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1071`.

**Responsibility:** Chaos outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected research specification below; current executable capability remains the historical partial slice. Explicit checked hypotheses establish model readiness, not observed native behavior.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected collection handoff (October 19, 2026)

Source constants are distinguished from derived arithmetic and finalized inferred clock/lifecycle decisions. This section supersedes historical partial-runtime descriptions below; the current executable slice remains incomplete. Prior original/supplement client extraction, public comparisons, QTE receipts and cue/consumer approaches already exceed the two-method allowance. Reuse their reached facts and finish explicit checked rules, without restarting native-body or visual-portal probes. Source tables are [consumed typed damage rows](../../../../crates/data/src/native_damage/chaos071.rs), [consumed typed owner parameters](../../../../crates/content/src/characters/chaos/parameters.rs), `1071AwakenEffect` (cache-only awakening source), native GA_Chaos071/CDO/notify maps in original/supplement exports, and frozen full multiplier lists.

### Complete packet specification

**Mixed.** All packets use ATK/Lakshana, live ordinary critical chance/damage; FixedCritRate0 is the no-override sentinel. Normal investment owns all Melee/Branch/plunge/riposte rows, Skill owns bites, Burst owns suppression; Support owns QTE. Count-based R3 adds1 to Normal/Skill/Burst, not Support. Preserve13-entry native coefficient arrays; level1/8/9 checks and per-packet channels are:

| Suffix after GE_Player_Chaos_ | Count in its action | Coefficient1 /8 /9 | Charge / Cycle / Break |
| --- | ---: | --- | --- |
| Melee1_Damage |1 |.182 /.312 /.337 |.913 /1.519 /.181 |
| Melee2_Damage |1 |.151 /.259 /.279 |.754 /1.256 /.149 |
| Melee2_1_Damage |1 |.390 /.668 /.722 |1.951 /3.247 /.386 |
| Melee3_Damage |2 |.144 /.247 /.267 |.719 /1.197 /.142 |
| Melee4_Damage |1 |.268 /.459 /.496 |1.340 /2.231 /.265 |
| Melee4_1_Damage |2 |.102 /.175 /.189 |.510 /.848 /.101 |
| Melee4_2_Damage |1 |.860 /1.474 /1.592 |4.300 /7.159 /.851 |
| Melee5_Damage |2 |.249 /.427 /.461 |1.246 /2.074 /.246 |
| Melee5_1_Damage |1 |.798 /1.368 /1.477 |3.994 /6.649 /.790 |
| Branch1_Damage |2, first Verdict stage |.575 /.985 /1.064 |2.9 /4.8 /.6 |
| Branch2_Damage |2, optional full-Crime Settlement stage |4 /6.855 /7.404 |2.35 /3.9 /.45 |
| Airattack_Damage |1, after landing |.450 /.771 /.833 |2.3 /3.8 /.5 |
| PerfectEvadeAttack1_Damage |1 |1.060 /1.817 /1.962 |2.7 /4.4 /2.5 |
| QTE1_Damage |1 |2 /3.428 /3.702 |6.4 /0 /2.5 |
| Skill1_Damage |1 |.825 /1.414 /1.527 |2.006 /3.353 /.412 |
| Skill2_Damage |1 |2.175 /3.728 /4.026 |5.294 /8.847 /1.088 |
| UltraSkill1_Damage |4 |1.273 /2.182 /2.356 |0 /0 /.239 |
| UltraSkill2_Damage |1 |2.909 /4.986 /5.384 |0 /0 /.545 |

Normal input stages are1,2,2,4,3 packets, total**12**. Stage2 ordinary sequence is Melee2 then Melee2_1; stage4 is Melee4, twiceMelee4_1, Melee4_2; stage5 twiceMelee5 then Melee5_1. **Transformed `_Ultra` stage2 swaps the two coefficient/resource rows:** Melee2_Damage_Ultra is.390/.668/.722 and1.951/3.247/.386, Melee2_1_Damage_Ultra is.151/.259/.279 and.754/1.256/.149. Other transformed counterparts match the table. Their ordered individual hits differ despite the same aggregate. Both full chains at8 total **6.238ATK,18.202charge,30.299Cycle,3.600Break**, independently summed from packet rows; this corrects historical19.340/32.228/3.834. Do not copy the first row across all hits in a stage.

Skill sums5.142ATK/7.3charge/12.2Cycle/1.5Break. Suppression is4*2.182+4.986=**13.714ATK**,5packets/1.501Break: UltraSkill.1_1 only marks, UltraSkill.5 only enters form, neither is another damage hit. Plain Verdict first stage2*.985=1.97ATK; full Verdict additionally2*6.855=13.71, total**4packets/15.68ATK/10.5charge/17.4Cycle/2.1Break**. Branch2 category remains native **A**, not Q, true damage, a second copy of already applied damage or DEF bypass solely because its display says Settlement. Normal/Branch/plunge categories A/A/X; riposteZ, QTEH, SkillE, suppressionQ. Plunge multiplies its primary coefficient by1+h with authored height bonus0..1; landing/airborne time counts in DPS.

`GE_Reaction_3_new_1071_Damage` is one **8ATK, Lakshana/Q** Remora-end packet with fixed coefficient8 at every talent level and zero charge/Cycle/Break. It receives ordinary live ATK/critical/DEF/RES and eligible Warrant/owner/gear bonuses. Its duration-dependent separate final-damage factor and dispatch are specified by effects; it is not shared Mental Nova or an eight-hit attack. Source row/GE_Chaos_DamageBase application modifier binds Warrant's live target check; no global30% damage buff is granted to unrelated allies. Each packet hits eligible recipients once with the full coefficient; resource and Crime grants do not multiply by target count. Source hit schedules/branch state below determine the real count, not visual hound passes.

### Complete timing specification

**Mixed, selected real-time profile.** Advance montage progress at1/s plus connected packet pause `duration*(1-rate)` from native HitPauseData: Melee1 .0198; ordinary Melee2 .005, Melee2_1 .016; Melee3 .0198 each; Melee4 .0495; Skill bites .0495 each; riposte.297; Settlement Branch2 .1852532 each. Same explicit pauses on transformed counterparts. Rows without explicit pause in the reached CDO add0 in this model, not an invented universal hitstop; do not swap pause metadata merely because transformed stage2 swaps coefficients. Pause once per packet regardless dummies. All real resource/buff/reaction/CD clocks continue during owner hitstop. These are inferred conversion/default rules with concrete source inputs, not observed timestamps.

| Action / stage | Hit progress coordinates in source montage | AnySkill recovery |
| --- | --- | ---: |
| Normal1 |.21457261 |.49906215 |
| Normal2 |.2373203,.3944437 |1.1539758 |
| Normal3 |.3420448,.47862947 |.506253 |
| Normal4 |.19421315,.3049187,.41848084,.61842537 |1.9688814 |
| Normal5 |.093479164,.25291294,.7293985 |1.0728416 |
| Transformed Normal1 |.21457261 |.49906215 |
| Transformed Normal2 |.23732,.394444 |1.1539758 |
| Transformed Normal3 |.3420448,.478629 |.506253 |
| Transformed Normal4 |.19421315,.3049187,.41848084,.61842537 |1.9688814 |
| Transformed Normal5 |.093479164,.25291294,.752119 |1.090035 |
| Plain Verdict stage1 |.1666913,.8836576 |.898334 |
| Transformed Verdict stage1 |.16719586,.883187 |.899451 |
| Full Verdict stage2 |.06584698,.46685714 |.633193 |
| Transformed Full Verdict stage2 |.06584698,.43118155 |.633193 |
| Plunge, both forms, after landing |.047594465 |.15928943 |
| Critical Riposte |.25015664 |.366207 |
| QTE |.7046429 |1.3763515 |
| Skill |.29827952,.6363952 |.9559941 |
| Burst |2.8074656,3.1802945,3.501765,3.9472904,4.792259 |4.9150305 |

Derived full ordinary chain **5.33091395s**, transformed **5.34810735s**, Skill1.0549941, full Verdict stage2 **1.0036994**. Full Verdict combines stage1 then stage2, not4 packets all at the first hit. A neutral held input pays **1s** held lead-in from BP_ChaosLogicObj.HOLD_DURATION before stage1; this selected inference distinguishes native input thresholds from the readable idle hold model. Combo holding can instead replace a running Normal only when its serialized HoldTrigger and montage-position predicates are satisfied; never append the cancelled Normal's future hits. The selected implementation may expose the neutral held input as the full supported action while mapping combo holds to their existing source predicates; no free instant charged attack is admitted.

Cancellation: earliest Jump gates for ordinary stages1..5 are `.26372725,.46491987,.50241214,1.2096211,1.0618224`; transformed stage3 Jump `.48324442`, others same. Move gates `.47345543,1.1489141,.58726496,1.2273152,1.2382003` (transformed stage3.587265). Ordinary/transformed stage1 Verdict Jump.18388283/.18285966, Move1.1088872/1.1161712. Full stage2 ordinary Jump.67441165/Move.6512871, transformed Jump.6324019/Move.6267849; AnySkill as table. Plunge Jump=AnySkill/Move.8251076; riposte Jump/Skill.0001, Move.366207; QTE Jump=AnySkill/Move1.2911102. Skill Jump1.3246958/Move1.2098653; Burst Jump/Move5.412379. Absent separate Skill gate uses AnySkill. Add only already caused pauses. Non-damaging front dodge has Jump.082544/Skill.333716/AnySkill.998284; back Jump.0001/Skill.5901485/Move.5999394/AnySkill.7951953. Riposte requires explicit successful Critical Dodge; one next-Normal authorization is consumed by use, another non-Normal action or field exit, an inferred finite guard without an invented expiry number.

Switch requires AnySkill recovery and explicit lock expiry: QTE lock starts.0001 for.87563384, Burst lock0..4.9445777. Other actions use recovery, no added universal switch fee. At cancel/exit discard future montage requests; already committed packets/statuses remain, no resource refund. AppearMelee/Pre use Normal1 hit.21782039 instead of.21457261, same recovery; no extra attack before stage1. Plunge requires authored positive airborne/landing time and height input; cannot repeat from ground for free. Use real total time including all holds/waits/recovery and presentation sequence; a Time Stop component does not erase party time or reset reaction/CD clocks.

Skill native cooldown marker is at progress **.24363968**, before first damage. Choose long/short cancellation branch from its reached GC_Chaos_SkillCD rule: cancellation at elapsed<.2s from cast starts a3s CD at cancellation, elapsed>=.2s uses15s at cancellation; successful uninterrupted Skill starts15s at the notify marker. Do not also start a second15s at recovery; the source unsuffixed fixed2 cooldown is not this branch. Exactly.2 selects15. Early forced cancellation before a cancel gate requires an authored interruption event; ordinary invalid input does not cancel or wait. Burst CD15 starts at accepted commit; source owner display curve20 does not override its GA-bound CoolDwonCurveTable15. No ordinary/Branch/riposte CD beyond resource/recovery; qualified-switch owns QTE admission. Boundary checks: Skill at0 is next available15.24363968, not15.0 or16.0549941; cancellation at.199 yields deadline3.199, at.2 deadline15.2.

### Complete resource and admission specification

**Mixed.** Ultimate Energy cap100/ordinary efficiency1 from DT_PlayerPackData:chaos_base; Burst cost100 from DT_GameplayAbilityCostData and explicit AbilityChargeCost. Initial0 unless legal preload declared; spend100 at accepted Burst commit, retain fractional carry. Skill/Normal/held/plunge/riposte have no additional energy/stamina tax in reached inputs; dodge costs **15 stamina**, requiring the [declared stamina profile](../../shared/energy-and-gauges.md). QTE follows qualified adjacent switch spending outgoing100Cycle; incoming100energy is not required. All positive applied-HP primary packets grant their listed channels once irrespective of targets; [shared per-ally .6/efficiency/carry/cap](../../shared/energy-and-gauges.md) applies. Fixed Remora-end8ATK has zero channels. No automatic resource reset between forms, waits, switches or rotation periods.

Crime is separate, exact nonnegative pool **0..1000**, starts0. BP_ChaosLogicObj maps connected packet GE names to DT_ChaosEffectFigure values. Chosen positive-HP once-per-packet dispatch: Normal1 **22**; ordinary Normal2 first33/second15; transformed first15/second33 (map names, not coefficient size); Normal3 **28 each**; Normal4 **35,13,13,19**; Normal5 **41,41,94**; plunge30; riposte100; Skill50+50; QTE/Branch/Burst0. Ordinary and transformed full chains each **382**, Skill100. This derived382 follows all12 mapped packets, correcting tooltip rounded/display300 rather than silently scaling every native row to match it. No per-dummy duplication; a three-dummy Melee1 grants22, not66. A1 multiplies the affected mapped grant by **1.4** when at least one connected recipient has Warrant, once per packet, retaining decimals. Native new scalar.4/frozen description take precedence over current Reference guide's conflicting.3 (Neverness also reports.4); do not sum the legacy zero-scaled generation fields or choose an arbitrary midpoint.

During live Dread Echo **while Chaos is active/living**, grant **30 every.2 World seconds**, explicit first at entry+.2, final at entry+12 before form cleanup (60ticks/1800 potential total), clamped1000. This final endpoint is a selected periodic exception to ordinary expiry-first ordering. Source AutoIntervalTime.2, UltraAutoAddSinsCore30 and independent display150/s agree; omitted native scalar is inferred1, not0. Off-field time still expires the12s form; scheduled ticks while inactive grant nothing, are not banked, and the original phase resumes on re-entry. No automatic generation outside form. A6 successful Burst commit fills1000 once, distinct from the later form timer; otherwise entry preserves current Crime.

Held Verdict does **not** spend on input/start/first plain hit. At completion of stage1 (its AnySkill boundary), after same-time auto ticks and positive-hit Crime grants, read current pool: at1000 capture and spend all1000 once and launch stage2; below1000 finish plain stage1 and preserve Crime. Full marker is derived from pool==1000, cleared on spend; infinite BranchChargeComplete's unrelated duration field5 is not a5s resource decay. At850 just before a1s/first-stage windup, six30 ticks can reach cap, so starting before1000 can still enhance; paying at hold admission would contradict that independent rotation observation. Spend occurs even if stage2 later misses or is interrupted; it cannot be repeated per hit or target. Partial pool is never confiscated for an unenhanced Verdict. At cap1000 the first stage's0 generation prevents self-refill loops; two consecutive full Verdicts need new Crime between them. Native mutation/ordering bodies remain unobserved; this precise decision is checked against the finite cap, zero Branch gain and documented in-animation completion.

All actions require living active actor, appropriate target/airborne/riposte state, recovery and CD; a zero pool still permits plain Verdict, but never a fabricated stage2. Selected form/row captures at admitted action start and retains that committed action through natural form expiry; later inputs use live form. A failed ordinary input emits a diagnostic without hidden wait/refill. Save Crime, energy/Cycle integer/carry/overflow, Normal stage and post-recovery2s continuation deadline, form/tick phase, cooldowns, Warrant handles and pending effects across cycle boundaries.

Public source `start_crime=X` declares exact Crime in 0..1000 (default 0), binding the existing ordinary Crime model before initialization. `.chaos.resources.crime` reads that pool and `.chaos.dread_echo.active` reads the settled native form flag. Two continuing ordinary chains from 0 grant 382 each and leave 764 on one or three stationary dummies; a 600 preload reaches 1000 with 364 overflow after the same two chains. The source fixtures retain direct native replay. The public party projection now binds the native form/clock before initialization. Its 60 final-inclusive 30-unit ticks grant 1800 from a 100 preload, leave 1000/overflow 900 and clear the flag at 14.718888s without internal fixture overrides. Full-form ordinary inputs, neutral Verdict and explicit `window=hold` combo/continuation inputs have source/native replay; remaining owner input choices stay in Q4-L3; the query preserves the native form/tick schedule, authored dummy assumptions and RNG. Conventional energy and shared Fons retain their separate accounts.

### Complete effect specification

**Mixed. Warrant:** Skill.3 at progress **.28373313** attaches the.5s DddHuntMark bridge whose BEGIN immediately applies the actual **15s** Warrant; .5 is not the Warrant lifetime or delayed bite. At that marker choose locked living target then lowest other eligible index, **up to2 per Skill cast**; it can precede bite damage. Missing all eligible targets creates no mark. Burst UltraSkill.1_1 at **2.7** marks **all** eligible living targets, before its first suppression damage; not an extra hit. Existing target marks refresh to marker+15, cap1 per Chaos source; Skill's2-target admission does not remove other still-live marks or impose a global2 cap on Burst. Status continues through source field exit/death until its own expiry, target death/dispel/scene cleanup. Stable IDs distinguish real reapplication from duplicated cue delivery.

Only **Chaos-owned** damage to that marked target gets general damage+.20, replaced by **+.30 with B4**, through Cau_Chaos_Passive2_DamUp. This is DamageUpGeneralBase added within the ordinary bonus zone, not another multiplicative1.3 on top of other general/element bonuses. Absent mark gives0; B4 does not make it.50. Apply live at each hit, including Branch, Skill, transformed Normal and his Remora-end packet. A2/A5 live conditions are checked separately before damage; positive HP then grants Crime. R6 is a real target RES modifier, so other actors' Lakshana packets can benefit even though their ordinary damage does not inherit Chaos's personal Warrant bonus.

**Justice in Time, B2:** while Chaos is deployed, attach one owner listener to each real target Remora, including applications by another party member. On natural expiry or explicit non-death removal of a living target's Remora, deliver **one**8ATK Lakshana/Q packet to **that target only**, using live Chaos stats/normal crit/DEF/RES/Warrant and one duration factor `1+min(3,.45*max(0,elapsed_since_last_application-5))`. Reapplication before end **resets** its elapsed start/deadline and produces **no premature explosion**. Hathor's duration extension to12 is retained; it does not add another duplicate Chaos listener. Dispel before5 uses factor1. Target death/scene teardown/source-owner death cancels the pending owner explosion; field exit preserves it. A normal expired5s Remora yields8ATK;10s yields26ATK,12s yields32ATK, upper cap32ATK. No trigger-QTE exclusion applies here beyond ordinary live event order; this is not Blackbird's accumulated-HP subsystem. It grants no raw energy/Cycle/Break, but gear/owner damage listeners may observe it once under their own eligible-category guards.

This final-duration rule is **inferred**, from both saved/current passive descriptions and the native constant FinalDamageUp3 gated by **Hathor's** passive. Do not silently substitute a Chaos marker in that helper or apply a universal3 at5s. Check: Hathor's described12-second Remora gives `.45*(12-5)=3.15`, capped3, exactly matching that helper's reached3.0 path. At5s the no-Hathor path's0 final bonus also agrees. The description supplies intermediate durations not read in that graph; selected model computes them once, never adds helper3 **again** to duration bonus3. Source curve/consumer disagreement is retained as a factual limit. Source native8ATK row and Warrant modifier remain unchanged. Example baseATK1000/liveATK2000, no crit/bonus, DEF.5/RES0, duration12 gives32000HP, not64000 after duplicate×2 or128000 after duplicate×4.

**Defensive/presentation effects:** Branch.5/6 grant EagleEye self/target tags for4s, cap1/NeverRefresh; no serialized outgoing-stat modifier, so no invented damage bonus. Full Verdict Branch.7 grants actual damage immunity for1.1 real seconds, removed earlier by montage skill finish/cancellation; incoming scenarios must respect it. Burst immunity begins at commit and removes at EndAbilitySkill progress4.8024926/cancellation, separate from4.9150305 recovery and12s Dread Echo. Display.1 at1.6333333/3.59231 only controls presentation, does not begin/refresh the Crime timer. Source curves' UltraSkillDamageUp0 contributes no phantom damage. Team A3 CD, A6 refill and R6 are governed below; temporary effects expire before equal-time ordinary damage unless the resource final tick exception is specified.

### Complete passive and awakening specification

**Mixed.** B2/B4 require their ordinary breakthrough unlocks; individual identities and count-based R3/R6 remain independent. Full native values in1071AwakenEffect/DT_ChaosEffectFigure; no unselected effect is inferred merely from awakening count.

| Selection | Complete modeled rule |
| --- | --- |
| B2 | Justice in Time Remora-end listener/formula/lifetime above, available off field. |
| B4 | Prime Target replaces personal Warrant general bonus.20 with.30; no additional.30 stack. |
| A1 | Mark-eligible Crime grant×1.4, exact, once per causal packet; native/frozen.4 chosen over Reference guide.3 conflict. No extra to0 Branch/QTE/Burst generation or passive auto30. |
| A2 | Chaos ignores .20 of target DEF on his packet if its recipient has Warrant. Apply to DEF before normal defense formula; not .20 resistance shred or final×1.2. |
| A3 | Accepted Retribution commit gives own CD+.30 for20 World seconds, cap1/refresh, persists field exit, before its hit train; not a team buff. |
| A4 | A marked recipient killed by a party-owned positive HP packet transfers this source's Warrant once to the lowest-index living **unmarked** eligible dummy (all-in-range implies proximity condition true), fresh15s. Exclude dying target/existing marked targets; no candidate means removal. Party kill attribution is the selected inference from the mark/death lifecycle; not an arbitrary nearest coordinate. No transfer on ordinary expiry/dispel, no recursive transfer without another real death. |
| A5 | Only Chaos damage **to the sole marked living target** gains general damage+.20 when exactly1 living recipient has this source's Warrant. Recount live marks after prior deaths/expiry; unmarked AoE recipients gain none. In the ordinary bonus zone alongside Warrant, not a separate unconditional final factor. |
| A6 | Successful Burst commit fills Crime1000 once, before its later Dread Echo marker; no refill on entering combat or on each suppression hit. |
| R3 count>=3 | Normal/Skill/Burst talent+1, including their alternate/transformed rows; no QTE+1 and no scaling of fixed8ATK passive. |
| R6 count>=6 | Marked targets have Lakshana RES-.10 for the live Warrant generation, not all enemies and not only Chaos damage. Refresh/remove/reapply synchronize with Warrant; one source modifier, no stacking on every hit. |

R6 selected synchronization reconciles the target cue's cap1/NeverRefresh15s child with the **actual Huntmark FINISH removal request** recovered in its parent. Reapplying the mark starts a replacement15s generation of its linked shred; removal clears it even if a cue's OnRemove body is empty. This is inferred lifecycle completion, not an unbounded stale shred or an additional -.10 after each hit. A4 transfer likewise creates a new generation once and checks selected R6. At DEF1080/attacker80, ordinary defense factor.5 becomes1080/(1080+864)=**5/9** with A2, not.6; sole-mark B4+A5 contributes+.50 general (before other zones), not1.3*1.2. Three targets with one marked give A5 only to that one; if two are marked neither receives it until one mark ends. Killing an unmarked target cannot transfer another target's Warrant.

### Complete form and alternate-action specification

**Mixed.** Normal stages1..5 wrap; different admitted action/field exit resets stage, same-actor switch does not. Keep chain through recovery and2 action-clock seconds after it; beyond this start1. Recovery-relative rule avoids resetting a legal slow montage under the old2s-between-starts convention. Dread Echo begins **UltraSkill.5 at2.7188878** after Burst admission, duration **12 real seconds**. Recast admission removes prior form and later marker starts a fresh timer/phase, not overlapping auto-resource loops. It uses transformed Normal/Branch/plunge variants, not a second independent weapon/ultimate form. It has no extra coefficient multiplier; matching source rows plus their second-stage swap are authoritative. Form expires off field in real time; auto Crime grants require on field. Form cleanup removes display/auto phase/replacement state, retains Crime, energy/Cycle, cooldowns, Warrant, pending committed action and actor effects. No form-only action is admitted after expiry, but its already captured full Verdict can finish.

Idle held Verdict uses the modeled1s lead-in, then ordinary/transformed stage1 and the live-Crime decision from resources. For **combo hold**, source GA_Chaos071_Melee JumpSectionMap resolves the full set: ordinary Normal stages1..5 require held duration.2 and progress strictly greater than `.3,.6,.4,1.09,.85`; transformed `.3,.64,.4,1.09,.9`. AppearMelee requires.25 held/progress>.3; Pre.2/>.3 in both forms. Plunge landing.2/>.12; ordinary stage2 continuation.2/>1.06, transformed stage2 continuation.1/>1.06. Once the authored hold input satisfies both predicates, replace remaining montage with stage1 using live form; no additional neutral1s lead-in on that combo route. Exact lower bounds are exclusive; equal-coordinate input is not admitted until an authored later input, no frame polling or arbitrary epsilon. Stage1's completed live-Crime check still controls stage2. A released early/insufficient-Crime hold produces plain stage1 only; holding does not generate Crime by itself. Repeat requires new held admission and real time. Full-stage immunity/EagleEye/pending hits retain their own clocks.

The same ordinary Skill, QTE and Burst remain available within Dread Echo under their ordinary cooldown/resource gates, not duplicated free variants. QTE has one hit and may auto-route a later **authored** Normal to AppearMelee, without a free hidden extra damage packet on QTE completion. Critical Dodge enables the one riposte; parry/back dodge is explicit scenario state and does not grant a phantom flat BlockDodge Crime because that native field is0. Plunge falls until its scenario landing and height bonus, then one attack. Hound Suppression/Skill visuals correspond to the listed montage requests, not autonomous pets that continue generating new hits off field. Already committed hits retain delivery on ordinary field exit only if no future montage request is required; an interrupted montage loses not-yet-requested bites/suppression hits. Native portal/WorldTransport/Fishing/mask effects are outside combat DPS and are not imported as hold durations, damage cooldowns or resource grants.

### Complete recipient and dependency specification

**Mixed.** Native DamageRange_Chaos_* requests are uncapped full-coefficient AoE in the stationary all-in-range model for Normal, both Verdict stages, plunge/riposte/QTE, Skill bites and suppression. Apply each hit to every currently eligible living dummy in stable index order; **Warrant's Skill limit2 is separate**, while Burst marks all. Mark selection locks the authored target then lowest-index second; transfer chooses lowest unmarked eligible, not a guessed nearest geometry. No target count divides one coefficient. Resource/Crime/pause once per packet; marks, R6, A2/A5/Warrant checks and HP per recipient. One Remora status on target0 explodes once on target0 even with five nearby unmarked dummies; two actual statuses produce two separate end packets. Do not promote this target-specific passive into the direct attack's AoE.

Use [damage zones](../../shared/damage-and-stat-zones.md), [equipment](../../shared/equipment-builds.md), [resource ledger](../../shared/energy-and-gauges.md), [qualified switch/cycles/Break](../../shared/cycles-and-break.md). Cosmos+Lakshana qualifies Remora, Psyche+Lakshana Stain; Charge's active-actor10 energy needs a real Blossom pistil hitting a Remora target and never comes from merely enabling Chaos B2. Remora default5 is a real status even in a stationary scenario; Hathor12 extension changes this owner's duration factor and follows that owner's unlock, no second final3 bonus. Reapplying Remora resets this owner's elapsed window without prematurely exploding; a separate real status expiry can occur off field. Stain uses the actual damage actor's live CI; B4 personal Warrant is in ordinary general zone while Stain's ratio remains the shared separate factor. Recipient R6 shred is live Lakshana RES for all eligible sources. Break uses each target's declared gauge/acceptance/recovery without fabricated universal vulnerability. Respect other owners' damage/kill/reaction callbacks once, retaining source/causal IDs so an indirect packet cannot recursively recount itself.

State required by implementation: exact Crime and auto-tick phase/form expiry, charge/Cycle ledgers, Normal stage/continuation deadline, cast recoveries/CD/locks, held-input/form snapshot/stage2 spend transaction, riposte authorization, per-source target Warrant generations/expiry/R6 handles, A3 self-CD handle, per-target Remora last-application/end/listener and pending packets. At equal time expire ordinary buffs/marks, then scheduled resource tick (including final Dread Echo endpoint), then due HP/resources, death/transfer and nonrecursive follow-ups, then authored action admission; stage1 enhancement checks after the causal resource transitions. No implementation flag may omit Crime, transformation, mandatory B2/B4 or linked gear effects and still label the configuration complete.

### Complete implementation handoff

**Mixed; independently checked continuing-state model.** The preceding seven specifications and linked shared dependencies provide the selected actions, conditional effects and implementation state. Executable bindings and acceptance are recorded below. Native uncertainty remains visible; additional source control lowering belongs to native workflow and final browser integration to browser integration.

**Legal A0 recipe:** Chaos `character:1071`,80/80 BT6, Normal/Skill/Burst/Support8, ordinary B2/B4 unlocked, no selected awakenings. **What's Desired `weapon:fork_GoldWool` T1 level80 BT6**, Street Boxer `set:Suit5`. Declare set count 4 and supply console contributions through `add stats`; the numeric baseline below is preserved without modeling modules.

Static compiler: character636+weapon570=**baseATK1206**, authored `atk%=.75 atk=420 cr=.30 cd=.92` gives **2530.5 exact ATK**, CR.59 and additional CD1.42, Charge Efficiency1/energycap100, CI0. Ordinary base CD is.50; no hidden console/specialization.60 is added. Defensive stats are unused by this no-incoming fixture. Street Boxer2pc Lakshana+.10 is already compiled. [Gold Wool](../../weapons/fork_goldwool/passive.md) T1 adds permanent Lakshana+.20, one self CD+.40 for20s. Select its recovered callback distinction: **ordinary Skill successful REALFINISH/full recovery**, and **Burst Q_SKILL_BEGIN/commit**; refresh one instance, not Skill admission plus finish duplicates. Failed/cancelled-before-finish Skill does not grant it; existing granted modifier persists field exit and expires on the real clock. This explicitly replaces that weapon card's old both-at-admission hypothesis for the selected profile.

[Street Boxer](../../sets/street-boxer/passive.md)4pc follows its later accepted `sets.street_boxer_v2` owner model: baseline self **CR+.14**, replaced by the powered curve for20s after a qualified team Remora/Stain admission. Both frozen curves are.14, so CR remains**.73**; the older October19 Chaos recipe's additional child+.14/.87 interpretation is superseded. An unrelated equipped off-field teammate also listens, refresh never stacks, and expiry restores baseline. Distinct-value Rust tests discriminate replacement from additive behavior despite equal frozen values. Solo has no reaction window, while parent and listener remain enabled. All-Lakshana packets use ordinary bonus zone1+.10+.20+(.30 if target Warrant), plus any selected A5.

**Scenario:** one immortal stationary all-in-range target, DEF factor.5, RES0, explicitly no Break bar, full HP, no incoming hits, parry or dodge, no energy/Crime preload. Solo keeps required B2/B4 enabled but has no Cosmos/Psyche switch to manufacture Remora/Stain or a weapon/set reaction proc. All abilities are admitted on their real recovery gates. Execute the same75s authored schedule at offsets0 and75, retain state at75 and evaluate through150:

1. Six complete ordinary chains consecutively from0,5.33091395,10.66182790,15.99274185,21.32365580,26.65456975; finish31.98548370. Wait to33; Skill33 marks at33.28373313, bites33.29827952/33.6858952, recovers34.0549941 and grants weapon CD until54.0549941.
2. Burst35 spends100 energy and refreshes weapon CD35..55. Marks all at37.7 (Warrant through52.7), enters Dread Echo37.7188878..49.7188878, recovers39.9150305; the distinct switch lock lasts39.9445777. Auto Crime30 at37.9188878 and every.2 through49.7188878,60ticks while active, expiry after final tick.
3. Neutral held Verdict40: hold to41, transformed stage1 hits41.16719586/41.883187, checks/spends1000 at**41.899451**, stage2 hits41.96529798/42.51588575, recovers42.9031504. Transformed Normal chain43..48.34810735 contributes12 primary packets and382Crime, with correct swapped stage2 coefficient/resource order.
4. Held Verdict48.5: capture transformed action while form is live, hold to49.5, stage1 hits49.66719586/50.383187, checks/spends1000 at**50.399451**; its captured stage2 continues after natural form expiry, hits50.46529798/51.01588575, recovers51.4031504. Expiry does not reset Crime or cancel this already committed action. Ordinary chain52..57.33091395; wait through75 and repeat. Later period uses the retained resources/ordinary expired effects; it does not repeat initial Crime0 assignment.

Each period has **111 positive HP packets**:84 ordinary Normal,12 transformed Normal,2 Skill,5 Burst,4 plain Verdict-stage packets and4 Settlement packets; total**222**. Raw energy **173.916/period**. Pre-Burst116.512 reaches100 with16whole overflow/.512carry, after-Burst57.404 ends **57+.916**; second ends**57+.832**, cumulative overflow90:347.832=200spent+90overflow+57.832. Cycle raw289.392/period ends100+.392 with189overflow, then100+.784 with478cumulative overflow. Crime grants include8 full chains*382+Skill100+auto1800 per period; two1000 spends and cap losses leave **382Crime at both boundaries**, first cumulative Crime overflow2574, then5530. These values retain the382 first-period remainder in second-period cap accounting; replacing it with0 would incorrectly lower overflow by382. No form, mark, gear temporary window or pending packet survives either boundary in this specific schedule, but their clocks were allowed to expire rather than reset.

Independent event/Fraction audit  checks exact resources, two funded1000 Crime spends per period, recovery, final periodic endpoint and form-crossing action. Portable expectation per packet is `coef*2530.5*(1.3+.3*Warrant_active)*(1+.73*(1.42+.40*weapon_CD_active))*.5`, with live deadlines. The old nearest-integer-ATK oracle823292.21321588 is corrected by **2530.5/2531**; the engine never rounds compiled ATK to2531. The native acceptance test independently checks each exact noncritical/critical alternative and this corrected expectation sum, plus seeded full-report aggregation/replay. Expected raw damage is distinct from sampled rounded damage and gameplay measurement. All necessary schedule inputs/equations are written here.

Full implementation acceptance additionally exercises explicit conditional branches above: three dummies with only2 Skill marks/all3 Burst marks; partial Crime plain Verdict preserves pool; late auto tick upgrades an850-start hold; cancelled full stage consumes1000 only once; A1 native1.4 versus conflicting1.3, A2 DEF5/9 counterexample, A3 selected-only effect, A4 stable unmarked transfer, A5 sole-mark per-recipient checks, A6 commit fill/R6 shared target shred; Remora5/10/12s gives8/26/32ATK and refresh delays rather than prematurely explodes; off-field form expires without banking Crime; QTE qualified switch, stamina-paid explicit riposte, correct early Skill3/15 cancellation branch; active weapon/Street Boxer reaction windows and CLI/browser parity. These values/rules are ready for implementation; tests of them do not establish unobserved native dispatch.

## Source facts and executable model

### Executable linked gear and stationary entry scope

Gold Wool's selected successful-finish model is bound through the owner clock:
connected Skill finishes1054994us after admission, grants one20s self window,
and Burst refreshes at commit. Cancelling before completion removes only the
unearned callback; a prior live window survives. Reached and inherited pauses
retime completion. Zero-damage successful Skill finishes955994us, with no
invented pause. All-rank Rust/native checks retain the permanent bonus,
separate critical multiplier, exact expiry, death and seeded replay.

The cached `GA_Chaos071_Melee` selector distinguishes default `Melee1` from
`MontageSelect_PreSection` (`PlayMontageName=MeleePre`,
`DistancePreSection=350`). Its unobserved selector body does not establish a
geometry-free permission. Under the mandatory stationary, all-in-range dummy
scope, ordinary entry uses default Melee1 and qualified QTE completion enables
the separately bound AppearMelee. Distance-selected approach Pre is outside
this profile: no coordinates, range checks, automatic approach or free manual
Pre override are added. Its source hold predicate remains evidence, not a
reachable extra DPS action. Native acceptance checks the ordinary first
contact214573us; QTE checks the distinct Appear217820us contact.

The legal A0 [native source](../../../../examples/chaos-native-cycles.nte)
runs the authored schedule above twice through150s with no reset:222 positive packets, four funded1000Crime spends,
57+.832 remaining energy/90overflow,100+.784Cycle/478overflow and382Crime/
5530overflow. A separate75s prefix verifies first-period carry and cap losses.
Every hit checks exact2530.5 ATK, the live Warrant/general zone and live weapon
critical window. The compiled configuration declares console contributions
and set count; ordinary source inputs, explicit waits and `hold=1` preserve
the authored schedule and direct native replay. The first Normal independently
checks `.312*2530.5*1.3*.5=513.1854` noncritical damage; captured transformed
Settlement contacts50.465298/51.015886s outlive the49.718888s form endpoint. Existing conditional owner tests retain awakenings,
three-recipient limits, Remora, off-field lifecycle and secondary controls.

### Executable Back Dodge intent

`inputs.dash_directions_v1` binds an explicit action-indexed Back intent through
the accepted secondary model; absent/Front profiles retain the existing front
behavior and report. Back pays the same15 stamina and shared.9s dodge cooldown,
with795195us recovery,100us Jump,590149us Skill and599939us movement gates.
Direction is authored movement intent, with no coordinates or inferred range.
The committed input retains its actual command identity and one explicitly
successful Critical Dodge permission. Back itself grants no packet/Crime or
immunity. Jump and other admissions consume the opportunity normally.

Rust checks separate all minus-one/exact gates and two continuing paid Back
Ripostes: inputs795195/2795195 contact1045352/3045352, totaling200Crime while
stamina falls100→70. Native front/back plus plunge cycles retain260Crime,
10charge and16.4Cycle, native aggregation and seeded replay. Unsupported owners,
non-Dash profile indices, missing stamina and unknown directions diagnose.
The Back hypothesis in each input receipt identifies the actual direction;
Front's complete report remains unchanged. Public source lowering remains
native workflow, alongside existing hold/jump lowering.

### Executable B2 Remora end

`chaos.b2_remora_end_v1` now arms one living breakthrough2 Chaos listener for
each real shared Remora application, including another party member's reaction.
The shared generation owns the last-application time/deadline and replacement;
the Chaos owner consumes removal and computes the selected separate final bonus.
Refresh retires its old callback without damage. Target/source death cancels
the owner request, field exit preserves it, and duplicate removal cannot emit
another packet. Pure statuses without an owner end listener add no pending
damage callback beyond the scenario horizon.

Natural expiry or the shared explicit non-death removal boundary emits one
`GE_Reaction_3_new_1071_Damage` to that living target only. It retains native
8ATK/Lakshana/Q and zero charge/Cycle/Break. The cached row's actual GAName is
`GA_Chaos071_UltraSkill`; the exact owner binding preserves that identity under
the autonomous `GA_Chaos_Passive_1` producer, with Other input role and the real
qualified-switch cause. It neither fabricates a Burst input nor triggers Burst
cost/cast effects. Live ordinary ATK/crit/mitigation/Warrant/gear zones apply.
The final duration bonus remains separate from both coefficient and general
damage bonus; the source's Hathor-gated3 is not added again.

Independent Rust checks: at live2000ATK/DEF.5/RES0, expiry5s gives8000HP,
removal10s gives26000HP, enhanced expiry12s caps at32000HP. The same packet
retains coefficient8 in receipts. A post-application+50% live ATK buff on
base1000 changes compiled2000 to2500 and12s damage to40000. A live B4 Warrant
makes the5s packet10400, not12800. Tests also cover misses, B2 gating, explicit
early removal, refresh, target death before a short horizon, source versus
applier death, off-field ownership and zero Remora channel gains. Qualified
entry retains its native recovery and once-per-packet6.4 charge: wearer6+.4,
peers3+.84. Explicit removal advances real queued events before the callback;
it never skips an earlier QTE or writes the clock backwards. Native legal parties
retain two ordinary/enhanced expiries, actual switch causes and seeded
aggregation/replay; only the Remora carrier is hit among multiple dummies.
Scene destruction drops this continuing runtime; authored Chaos combat-transition
controls remain a separate binding.

### Executable front Dodge, Critical Riposte and authored plunge

`chaos.secondary_actions_v1` extends the ordinary Crime/clock profile with paid
front Dodge, one next-Normal Critical Riposte permission, authored airborne
plunge, and reached movement/Skill cancel gates. Back Dodge is bound above;
distance-selected Pre is outside the stationary entry scope. This is the selected timing/lifecycle
model above, not an observation of native timestamps.

Front Dodge costs15 from an explicit shared stamina profile and recovers at
998284us, with Jump82544us and Skill333716us. Only an explicitly successful
Critical Dodge grants Riposte; another admitted action/input, field exit or
death consumes that permission. No arbitrary timeout is added. The later
canonical Normal dispatches the common native `GA_Chaos071_PerfectEvade` Z row
in both forms, with effective Normal talent. Contact250157us adds297000us of
pause only after positive HP settlement; recovery366207us therefore becomes
663207us on connection. Jump/Skill gates are100us. Three recipients still
grant only100Crime,2.7charge and4.4Cycle. Raw Riposte rows cannot bypass entry.

Plunge requires positive declared flight and height bonus0..1. Its single
contact occurs landing+47594us, recovery landing+159289us, coefficient scales
by1+h, and live admission selects ordinary/transformed row while a committed
packet survives natural form expiry. One connected group grants30Crime,
2.3charge and3.8Cycle. The Normal-combo model accepts a.2s hold overlapping
flight only after strictly.12s landing-montage progress. Movement gates are
projected through reached pauses and bounded by their actual source lifetime;
full Verdict keeps its later EndAbilitySkill boundary without extending its
earlier immunity.

Skill cooldown starts at its projected243640us marker, including inherited
Riposte hitstop. Independent check: Riposte at998284 hits1248441; Skill admitted
1248442 during that pause hits1843721 and becomes ready16789081, rather than
starting its cooldown against the unpaused input timestamp. Arc commits now
close the old owner montage/continuation source before weapon activation,
using the actual input command index.

Rust checks cover strict minus-one/exact cancel and combo boundaries, stamina
failure, misses, permission consumption, A1/form selection, height coefficients,
flight across form expiry, and Arc closure of a post-recovery full Verdict.
Two continuing native paid Riposte/plunge cycles on two dummies settle8 hits,
260Crime,10charge and16.4Cycle with seeded aggregation/replay. The shared script
also supports `chaos plunge[airtime=600ms,height_bonus=1/2]:2`, retaining759289us
per cycle and exact source-to-command profiles. No jump velocity or geometry
is inferred.

### Executable ordinary Crime prerequisite

`chaos.ordinary_crime_v1` binds the ordinary positive-HP GE-key grants to one
exact pool starting0, capped1000. The five Normal stages grant22;33+15;28+28;
35+13+13+19;41+41+94, totaling382 per twelve-packet chain. Skill grants50 per bite.
The pool persists across waits, switches and cycle boundaries. No owner preload,
automatic form tick, Verdict spend or A6 fill is inferred by this bounded model.

Ordinary Normal/Skill/Burst packet trains carry one causal identity per primary
across their full eligible AoE. Ultimate charge, Cycle and Crime grant once on
positive applied HP, including when target0 is already dead and another dummy
connects. Zero HP and dead owners grant nothing. Normal stage continuity lasts
through caller-authored recovery plus2 action-clock seconds. Without the clock model
below, caller packet timing and Warrant application remain
historical execution policies, separate from the selected full-kit specification.

Selected A1 uses the native1.4 multiplier when any connected recipient has this
Chaos source's Warrant. The first eligible recipient grants the mapped base;
the first marked connected recipient also grants the exact missing.4 increment.
Two per-packet guards prevent duplicate base or A1 gains and preserve the same
cap/overflow for either recipient order. A marked recipient with zero applied HP
cannot upgrade an unmarked positive hit. This uses actual source-owned Warrant,
not an unrelated owner's mark or an assumed globally marked target list.

The same ordinary packets consume typed source `HeterochromeAdd` through
`combat.packet_cycle_accrual_v1`. Per full chain the independent channels are
382 Crime,18.202 Ultimate charge and30.299 Cycle; Cycle efficiency scales only
the latter. Four continuing chains retain1000 Crime/528 overflow,72 Ultimate
with.808 carry, and100 Cycle/21 overflow/.196 carry. A1 Skill grants140 total
across three recipients; an ordinary22-Crime packet grants30.8 with a connected
mark, including exact20.8 overflow from a990-start boundary fixture. Native
legal builds, two continuing authored-recovery chains, aggregation and seeded
replay retain the new ledger. These checks certify this resource prerequisite,
not form, natural QTE, complete Warrant dispatch or whole-kit acceptance.

### Executable ordinary clock and Skill Warrant

`chaos.ordinary_clock_v1` requires the resource prerequisite and replaces caller
Normal/Skill timing with the selected source notify coordinates, rounded once
to microseconds. It uses the shared exact hitlag integrator, representing the
documented effective progress losses as full-stop intervals after connected
contact. This is an explicit selected conversion, not recovered slow-motion
window shape. Ordinary packet schedules and full recovery gates exercise the
complete losses; early cancel gates and transformed partial-window behavior
remain separate bindings. Dummies share one pause per causal primary packet.
Misses add none. Cooldowns, status durations, other actors and DPS time remain
on the real axis. Only pending owner montage requests are reprojected.

One full ordinary chain has twelve contacts and recovery5,330,914us. Skill
contacts are298,280/685,895us, recovery1,054,994us. Skill cooldown is15 seconds
from its243,640us marker, so a cast at0 is next available15,243,640us. These
values override caller delays/recoveries. Normal continuity includes actual
recovery plus2 action-clock seconds. No polling or hidden waits are added.

Skill.3 applies Warrant at283,733us, before the first bite, to at most two
currently living dummies in stable index order. Warrant expires15 World seconds
after that marker; later bites do not refresh it. B4 replaces the general bonus
.20 with.30 on each marked recipient. A2 retains its separate marked-recipient
DEF ignore; A5 counts living source-owned marks rather than all declared dummies.
R6's target resistance modifier follows actual Warrant recipients. Field exit
preserves already applied marks and cancels pending montage requests, including
an unreached marker. Dead sources apply no later marker.

Independent Rust checks cover one/three-dummy clock agreement, rejection one
microsecond before recovery/CD, marker-relative exact expiry, a sole living
mark among three declared dummies, and A2's5/9 DEF factor versus neutral1/2.
An unchanged sparse scenario processes the same number of events at30 and500
seconds. Native builds preserve two continuing Normal chains and two Skill casts
with actual owner timing, resource ledgers, Warrant markers, aggregation and
seeded replay. Burst/form, Crime auto ticks/Verdict spending, early cancellation,
conditional transfer, natural QTE and remaining team interactions stay open;
Burst under this bounded clock diagnoses its missing form binding.

### Executable full-stage Verdict continuation lifetime

`chaos.verdict_continuation_v1` requires neutral Verdict. The reached
`EndAbilitySkill` notify bounds the old full stage at progress1,088,377us
ordinary or1,102,773us transformed; its earlier633,193us AnySkill gate remains
ordinary next-action readiness. Only the bounded continuation source survives
that readiness gate. The raw8.833333s pose tail is not a combat resource/state
window and is not retained as a runtime constant. Source carriers are cached
`char_Chaos_skill_branch_2_Montage{,_Ultra}.json` and the GA's JumpSectionMap.

A new authored hold can release at progress strictly above1,060,000us, with
200,000us ordinary or100,000us transformed held inside that stage, before its
exclusive end. Read the old captured form for that predicate and live form for
the new Branch1, without an idle lead-in or resource reset. The ordinary
stage1 live-Crime decision still owns any later enhancement. Another admitted
action, exit, interruption or defeat consumes the old source. With the Jump
binding, the native full-stage Jump gate also remains available until this end.
Per-input owner deadlines return typed recovery failure for an early hold/Jump
after general readiness; no executor wait or retry is inserted. Existing
earlier bindings retain their own clamped gate policy. Full-stage immunity
keeps its separate existing cap/recovery rule.

Independent boundaries: a full ordinary stage starting18,898,334us recovers
19,902,033us after two pauses, but its continuation admits20,328,841us (strict
1.06 plus370,506us caused pause) through20,357,216us. At20,357,217us its source
has ended. Jump's native674,412us gate admits19,943,252us; minus1 fails despite
general readiness. A post-recovery interruption cancels that source without
refunding its1000 spend. The transformed100,000us hold can still use a captured
transformed predicate after form expiry while selecting an ordinary new row.
Native acceptance funds two full stages and two continuations, retaining4092
Crime granted/2000 spent/2032 overflow/60 stock,16.812 Ultimate charge and exact
Cycle carry through continuing aggregation/replay. Source end and native
dispatch order remain a selected causal binding checked against these markers.

### Executable qualified QTE and authored Appear Normal

`chaos.qualified_support_v1` requires the ordinary clock and selects one
engine-owned QTE after the shared eligible adjacent-element switch spends the
outgoing owner's100 Cycle. Incoming Ultimate energy is neither required nor
spent. Authored direct QTE requests diagnose under this binding. One native H
packet reaches every living dummy at704,643us, with recovery1,376,352us, zero
hit pause/additional CD,6.4 raw Ultimate charge,0 Cycle,2.5 Break and0 Crime.
The existing direct-QTE policy distributes charge once per causal packet,
including the ordinary.6 ally share. Its875,734us switch-lock endpoint is before
full recovery, so it grants no early switch or extra recovery fee.

Successful full QTE recovery authorizes one later authored Appear Normal:
Normal1's row at217,820us contact instead of214,573us, followed by ordinary
Normal stage progression/recovery. There is no hidden attack at QTE completion.
QTE retains its Support talent while the later Normal uses effective Normal
talent; R3 does not upgrade QTE. Another action/exit consumes unused permission;
an interrupted or defeated QTE cannot complete it. Form/tick expiry remains
real, QTE uses its ordinary native row in either form, and the later input
selects live form. The separately accepted Normal combo model extends Appear
to its source250,000us real hold and progress strictly above300,000us, without
a new idle lead-in. Pre/plunge/other continuation routes remain separate.

Independent checks: three QTE recipients yield6.4 charge to Chaos and3.84 to
the ally, with no incoming energy spend or duplicate opportunities. Recovery
at1,376,352us permits one authored Normal at1,594,172us; at R3 its Normal1
coefficient uses level2 while QTE remains level8. Missing Cycle/nonadjacency
cannot grant QTE or Appear. Interruptions before/at the contact retain only
reached charge and no completion permission. QTE across form expiry preserves
the original14,718,888us endpoint and the later Normal's live row.

Native acceptance starts with outgoing Blackbird Cycle100, spends it for QTE,
uses his real Skill fill to fund a second QTE, and retains both100-Cycle spends.
One authored Appear plus the two QTEs and real Blackbird coordinated feedback
produce24 target deliveries over three dummies, with19.449 retained Chaos
charge and17.7878 Blackbird charge, no resets or hidden second Normal. Native
aggregation/seeded replay includes those cross-owner causal descendants.

### Executable source-owned Warrant transfer

`chaos.warrant_transfer_v1` requires the ordinary clock and is reached only for
selected individual A4 on an actual marked party kill. Before target cleanup,
each Chaos source independently chooses the lowest-index living unmarked
dummy, excluding the victim and existing marks, then applies fresh15s Warrant
with the same source/R6 resistance modifier. No candidate removes the victim's
mark without refreshing another recipient. Removal after source decisions
guards duplicate callbacks; application cannot recursively trigger a death.
The shared observer covers ordinary, reaction and finishing kills, including
equipment-owned positive-HP packets. Expiry/dispel/scene cleanup has no kill
callback. Off-field sources retain this effect; existing independently lived
marks also retain their A4 lifecycle after source defeat, consistent with the
declared mark lifetime rather than B2's separate owner-death cancellation.

Independent causal check: Skill marks0/1 at283,733us; recipient0 with500HP dies
to the first bite at298,280us, transferring to2 before that same bite reaches2.
At1000ATK/DEF factor.5, target2 then takes536.25 (.825*1000*.5*1.3), while the
next unmarked target takes412.5. Target1 expires15,283,733us; new target2 expires
15,298,280us. Only100 Crime is generated by both bite groups, regardless of
recipient count. With no unmarked candidate, the surviving sole mark gains
its independently selected A5 before its later same-group contact, without a
lifetime refresh. Checks also retain R6 on transferred marks, skip dead
candidates, distinguish A4 from R3 count, cover teammate/source-defeat and
Scorch kills, and run two continuing native Skills with exact resources,
aggregation and seeded replay. Stable all-in-range candidate choice and party
attribution remain selected rules, not observed native nearest-target dispatch.

### Executable Normal combo Verdict and Jump cancels

`chaos.normal_combo_verdict_v1` extends neutral Verdict to an unfinished
Normal1..5. The authored command is the hold release: at least200,000us of
real held time must fit inside the preceding montage, and actor progress must
be strictly above300,000/600,000/400,000/1,090,000/850,000us ordinary or
300,000/640,000/400,000/1,090,000/900,000us transformed. The old action's captured
form chooses its predicate; the new Branch1 captures live form. Future old
contacts are removed and no extra idle1s lead-in is charged. A recovered montage
cannot authorize a free idle subsecond Verdict. Pre/Appear/plunge/other source
continuation predicates require their separate action bindings.

`chaos.jump_cancel_v1` binds explicit Jump to the reached montage's native gate,
including only already caused pauses and clamping to natural recovery. Invalid
early input fails; no retries or hidden waits occur. Commit removes future
montage contacts/Crime decisions, resets the chain and removes branch immunity,
while preserving stock, delivered effects, committed spend and real form ticks.
Jump adds no damage, flight or recovery duration. Dodge and the other secondary
actions remain separate bindings.

Independent causal checks: ordinary Normal3 starts1,693,838us and its first hit
at2,035,883us adds19,800us pause. Progress400,000 is reached2,113,638us; equality
fails, while authored release2,113,639us launches Branch1 at2,280,330us and
cancels the otherwise due second Normal3 packet2,192,267us. Crime is98, not126.
Transformed Normal2's640,000 boundary admits at6,179,863us in the checked chain,
not the ordinary predicate's6,139,863us. All five Normal Jump gates are checked
at minus1us/exact in both forms. A transformed full-stage Jump at10,521,796us
retains the1000 spend but removes immunity before incoming damage1us later.
Two native continuing combo/Jump cycles retain196 Crime,14.474 Ultimate charge
and24.038 Cycle, with only the emitted contacts in aggregation/seeded replay.

### Executable neutral held Verdict

`chaos.neutral_verdict_v1` requires ordinary Crime and the ordinary clock. A
canonical Normal with a stationary `hold_inputs` duration at least1s pays that
entire duration before Branch1 begins. Zero remains a tap. Movement intent and
caller-selected Branch rows diagnose; without the separately accepted combo
model, an unfinished montage blocks this neutral input and subsecond holds
diagnose. Form and effective Normal talent are captured on admission,
so natural Dread Echo expiry cannot change or cancel an in-progress Verdict.

The two Branch1 contacts use ordinary166,691/883,658us or transformed
167,196/883,187us, with recovery898,334/899,451us. At that boundary, a two-phase
owner callback settles due ticks/HP and immediate death before reading live
Crime, then extends recovery before equal-time ordinary admission. At1000 it
spends1000 once and launches Branch2; otherwise it preserves partial Crime.
No Branch packet generates Crime, and later interruption/miss never refunds
the full-stage spend. Each stage's primary packet identity is shared across all
living dummies, preserving one charge/Cycle opportunity and one pause per hit.

Branch2 contacts use65,847/466,857us ordinary or65,847/431,182us transformed;
each positive packet adds185,253us effective lost progress, independent of
recipient count. Recovery633,193us becomes1,003,699us after both pauses. Full
stage immunity lasts at most1.1 real seconds, capped by actual recovery and
removed immediately on interruption/defeat. Both Branch rows remain category A.
At Normal8, four packets total15.68ATK,10.5 Ultimate charge,17.4 Cycle and2.1
Break; typed native rows retain all13 consumed talent coordinates.

Independent checks: a382-Crime plain stage preserves382. Three ordinary chains
produce1146, allowing exactly one1000 spend plus146 overflow; a second held
action without new Crime is plain. Burst0 then hold7,619,437us begins below cap:
stage1 ends9,518,888us, exactly tick34, which upgrades990 to1000 with20 overflow
before the single spend. All60 ticks occur once, leaving780 after the full
stage. A same-time fatal incoming hit instead prevents spending. A transformed
hold13.5s survives form expiry14.718888s, reads/spends at15.399451s and finishes
16.403150s. Native acceptance funds two continuing ordinary/transformed Verdicts
through six chains and one Burst, retaining2000 total spent Crime,2092 overflow,
exact charge/Cycle carry, aggregation and seeded replay. Sparse duration adds
no polling. These tests establish the selected causal model, not unobserved
native Blueprint dispatch bodies.

### Executable forced interruption prerequisite

`chaos.forced_interrupt_v1` requires the ordinary clock and explicitly authored
`interrupt_events`. It releases an in-progress living owner's recovery, cancels
future montage contacts and unreached Warrant/form markers, resets the Normal
chain and removes montage immunity/switch lock. Existing damage, Crime, marks,
committed costs and an already entered form retain their causal state. The
selected interruption adds no unobserved stun duration. Voluntary cancel gates
remain a separate binding.

The reached Skill cancellation branch uses real elapsed time:199,999us gives
3,199,999us next availability;200,000us gives15,200,000us. An interruption at the
first bite's298,280us retains that bite on all eligible recipients, its50 Crime
and the reached Warrant marker, cancels only the second bite, and sets cooldown
to15,298,280us. This does not refund resources or restart the original notify CD.
At Burst entry minus1us interruption cancels entry; at entry itself the form
already exists and all60 real auto-tick opportunities continue. Tests also
exercise native continuing Skill casts, aggregation and seeded replay. These
are selected causal rules supported by the source branch graph, not proof of
unobserved native cancellation dispatch.

### Executable primary Burst and Dread Echo

`chaos.primary_form_v1` extends the ordinary clock/resource models. Burst costs
100 Ultimate energy with native capacity100 and15-second commit cooldown.
Five primary contacts follow2,807,466/3,180,295/3,501,765/3,947,290/4,792,259us,
with no invented pause. Recovery4,915,031us, switch lock4,944,578us and immunity
end4,802,493us are separate. Burst Warrant marks all currently living dummies
at2,700,000us; Dread Echo enters at2,718,888us. The presentation markers create
no additional damage. A3's self critical-damage buff keeps its existing commit
hook. A6 fills missing Crime to1000 once at commit, including on miss; failed
energy admission grants nothing and schedules no form.

Dread Echo runs12 World seconds, including time off field. A living active owner
receives30 Crime every200,000us, first at entry+200,000, through the final endpoint.
That endpoint grants then removes form before a same-time authored action.
Skipped off-field ticks do not bank; re-entry resumes the original phase.
Generation guards invalidate old callbacks on recast. Defeat stops grants.
Natural cleanup resets the Normal stage and preserves resources, cooldowns,
Warrant and packets already captured by an admitted action.

Live form chooses the native Normal `_Ultra` packet rows. Their ordinary GA
identity and all consumed native channels are retained; stage2 swaps the two
coefficient/charge/Cycle/Break rows and uses15/33 Crime, independently of
coefficient size. Remaining transformed Normal rows share source arrays/channels
with their exact ordinary counterparts. Only those nine reached alternate rows
are retained. Transformed stage5's final hit752,119us and recovery1,090,035us
give full transformed-chain recovery5,348,107us. Other stages retain their
ordinary progress/pause profiles. An action admitted one microsecond before
expiry keeps its captured transformed row after expiry; a boundary input uses
the ordinary row.

Independent Rust checks give60 ticks/1800 potential Crime, with first/final
times2,918,888/14,718,888us. Leaving at5s and returning at10s preserves11+24
actual ticks, grants1050, and still expires14,718,888us; staying off field grants
only330. A6 adds one1000 fill before those ticks. Three-dummy checks retain full
AoE, exact swapped stage2 damage, no multiplied resources, and native switch/
immunity/recovery boundary failures. Native two-cycle acceptance funds the second
Burst with six continuing ordinary chains and executes one transformed chain:
94 positive packets on one dummy,120 auto ticks,200 energy spent,18+.414 energy
remaining,100 Cycle/112 overflow/.093 carry,6274 Crime granted/5274 overflow.
Aggregation and seeded replay retain the complete continuing report. These
checks establish the declared primary-form model, not native body certainty or
the remaining Verdict/secondary/QTE/team-interaction acceptance.

Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,2,4,3 (twelve hits). The chain loops, retaining native effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`chaos-normal-chain.json`](../../../../reviews/q10-game-action-binding/chaos-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`chaos.skill_animation_profile_v1`, `chaos.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`char_Chaos_skill_skill_1_Montage`), `AnimNotify_TriggerCooldown` is at 0.244s, direct bites land at 0.298s (`Skill.1`) and 0.636s (`Skill.2`); cancel windows activate at 0.956s (`BreakSkill_AnySkill`), 1.210s (`BreakSkill_Move`), and 1.325s (`BreakSkill_Jump`). For Burst (`char_Chaos_skill_ultraskill_Montage`), direct hits: 3 hits of `UltraSkill.1` land at 2.807s, 3.180s, and 3.502s, with initial/special hits at 2.700s (`UltraSkill.1_1`), 3.947s (`UltraSkill.3`), and final hit `UltraSkill.4` at 4.792s; cancel windows activate at 4.915s (`BreakSkill_AnySkill`) and 5.412s (`BreakSkill_Move`, `BreakSkill_Jump`).
- **Multi-target delivery (`chaos.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive the full packet train for Skill (2 bites: `GE_Player_Chaos_Skill1_Damage` and `GE_Player_Chaos_Skill2_Damage`) and Burst (5 hits: 4x `GE_Player_Chaos_UltraSkill1_Damage` + 1x `GE_Player_Chaos_UltraSkill2_Damage`) across all living dummies in ascending target_index order at the primary impact time.

## Action packets and owner effects

- GA_Chaos071_Melee public primary packet is GE_Player_Chaos_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.182/0.312/0.337; category A/LAKSHANA metadata stays source-bound.
- GA_Chaos071_Skill public primary packet is GE_Player_Chaos_Skill1_Damage (Doubtmark first bite), delivering 2 native bites with child packet GE_Player_Chaos_Skill2_Damage at impact. Native coefficients at Skill levels 1/8/9 are 0.825/1.414/1.527 (bite 1) and 2.175/3.728/4.026 (bite 2); category E/LAKSHANA metadata stays source-bound. before_hit applies Warrant (+20% damage bonus).
- GA_Chaos071_UltraSkill public primary packet is GE_Player_Chaos_UltraSkill1_Damage. Suppression delivers four native GE_Player_Chaos_UltraSkill1_Damage hits followed by one GE_Player_Chaos_UltraSkill2_Damage hit, using native Burst coefficients at levels 1/8/9 of 1.273/2.182/2.356 and 2.909/4.986/5.384; category Q/LAKSHANA metadata stays source-bound. Ordered coincident delivery at primary impact is the tested chaos.suppression_packets_v1 timing hypothesis. Every hit applies Warrant (+20% damage), A2 DEF ignore when applicable, A5 single-target damage when applicable, and A6 Lakshana resistance reduction; A3 casts grant +30% Crit DMG for 20s before the train.
- Resonance 3 activates at awakening >= 3 and adds +1 level to GA_Chaos071_Melee, GA_Chaos071_Skill, and GA_Chaos071_UltraSkill without manual overrides.
- Awakening 2 ignores 20% of target DEF when attacking enemies with Warrant.
- Awakening 5 increases damage dealt by 20% when only 1 target with Warrant remains.
- Resonance 6 reduces Lakshana resistance by 10% on enemies with Warrant.
- Canonical arbitrary awakening selections gate modeled A2 DEF ignore, A3 Burst Crit DMG and A5 damage by effect identity. R3 talent bonuses and R6 Lakshana RES reduction use selection count. [3] grants the Burst Crit DMG buff without R3 or A2; [1,4,5] grants A5 and R3 without A2 or A3. Crime and Warrant transfer remain pending.

## QTE direct-input energy slice

The scenario runner accepts the exact `GA_Chaos071_QTE` / `GE_Player_Chaos_QTE1_Damage` packet pair as an explicit direct input. After positive actual HP loss against target zero, the separate `chaos.qte_direct_packet_energy_v1` hypothesis grants the frozen 6.4 `ChargeAdd`: 1x to Chaos and 3/5 to other living configured recipients, followed by each recipient's Charge Efficiency, fractional carry and shared cap/overflow settlement.

The combined zero-start scenario in `chaos_zero_start_qte_burst_and_normal_continuation_reconciles_damage_and_energy` joins 28 Normal inputs at the DSL's 800ms caller interval (65 receipts), that exact caller-supplied QTE input at 22.4s, a 100-cost Burst at 23.0s, and one Normal continuation at 25.0s. The source has 96 energy before QTE; its 6.4 ChargeAdd reaches the 100 cap and funds the Burst under the separate direct-input hypothesis. With 1000 ATK, level-one talents, target DEF 1080 and a 25,200,100µs horizon, its independent damage oracle is 75064/5 across 72 receipts. The Web test compiles the authored Normal/Burst rotation, inserts the same explicit QTE pair, and checks browser and Worker parity. These are scenario timings; the assumed 500ms QTE recovery is not native timing evidence.

This does not add a native QTE trigger or establish QTE legality, readiness, cost, timing, multiplicity or native energy eligibility. The direct scenario command is caller-supplied; only this exact packet pair is covered. See the [direct-packet energy receipt](../../../../reviews/q10-energy-ledger/direct-packet-energy-owners.json) and shared [energy accounting model](../../shared/energy-accounting.md).

## Individual awakening selection

`[3]` grants the modeled Burst Crit DMG effect without A2 or R3; `[1,4,5]` reaches R3 and A5 without A2 or A3. Ordinary effects use their own selected index; resonance remains count-based.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/chaos-action-binding-model.json`](../../../../reviews/q10-game-action-binding/chaos-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/chaos_actions.rs), [`chaos-normal-chain.json`](../../../../reviews/q10-game-action-binding/chaos-normal-chain.json), [`chaos-skill-packets.json`](../../../../reviews/q10-game-action-binding/chaos-skill-packets.json).
- Frozen inputs: `data/extracted/guide_chaos.json`, `data/extracted/neverness_chaos.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/chaos071.rs) (original `DT_SkillDamageData` export is cache-only), `1071AwakenEffect` (cache-only awakening source), [`chaos/parameters.rs`](../../../../crates/content/src/characters/chaos/parameters.rs) (consumed typed parameters; original `DT_ChaosEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- The legacy direct-input profile below does not bind natural Branch/QTE dispatch, Crime or transformations. The executable owner slices above bind those reached paths separately; full-kit acceptance still requires the remaining secondary/passive/gear consumers. The legacy QTE direct-input energy slice does not establish its natural trigger or action admission.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/chaos). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Chaos (`GA_Chaos071_*`):

| Action / damage row (`GE_Player_Chaos_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.182–0.860 / 0.312–1.474 / 0.337–1.592 | 19.340 total / 32.228 total / 3.834 total |
| Transformed chain `Melee1..5_Damage_Ultra` | A | Matches ordinary chain coefficients | Same ChargeAdd / Cycle / Unbal |
| Plunge `Airattack_Damage` | X | 0.450 / 0.771 / 0.833 | 2.300 / 3.800 / 0.500 |
| Hold branch 1 `Branch1_Damage` | A | 0.575 / 0.985 / 1.064 | 2.900 / 4.800 / 0.600 |
| Hold branch 2 `Branch2_Damage` | A | 4.000 / 6.855 / 7.404 | 2.350 / 3.900 / 0.450 |
| Critical Riposte `PerfectEvadeAttack1_Damage` | Z | 1.060 / 1.817 / 1.962 | 2.700 / 4.400 / 2.500 |
| Support `QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 6.400 / 0.000 / 2.500 |
| Skill bite 1 `Skill1_Damage` | E | 0.825 / 1.414 / 1.527 | 2.006 / 3.353 / 0.412 |
| Skill bite 2 `Skill2_Damage` | E | 2.175 / 3.728 / 4.026 | 5.294 / 8.847 / 1.088 |
| Burst `UltraSkill1_Damage` (4 hits) | Q | 1.273 / 2.182 / 2.356 | 0.000 / 0.000 / 0.239 per hit |
| Burst `UltraSkill2_Damage` (1 hit) | Q | 2.909 / 4.986 / 5.384 | 0.000 / 0.000 / 0.545 |
| Reaction extension `GE_Reaction_3_new_1071_Damage` | Q | 8.000 / 8.000 / 8.000 (fixed) | 0.000 / 0.000 / 0.000 |

Critical Riposte uses GAName `GA_Chaos071_PerfectEvade`, while Support QTE uses `GA_Chaos071_QTE`. All damage rows deal LAKSHANA damage. In transformed state, matching `_Ultra` damage rows are dispatched with identical coefficients.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Chaos_Passive_1` binds `Buff_Chaos071_Passive1`. `DT_ReactionExtendDataTable` row 1071 binds `ReactionAbilityID=GA_Chaos_Passive_1`, enabling Chaos's Reaction 3 extension damage via `GE_Reaction_3_new_1071_Damage` (flat coefficient 8.000).
- **Passive 2 (Breakthrough 4):** `GA_Chaos_Passive_2` binds `Buff_Chaos_Passvie2`.

### Awakenings

Source descriptions and selected IDs: `1071AwakenEffect#/rows/1071/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Grants an additional +40% Crime when attacking enemies with Warrant.
- **A2:** Ignores 20% of target DEF when attacking enemies marked with Warrant (`TargetDefIgnore = 0.20`).
- **A3:** Casting Retribution (Burst) grants +30% Crit DMG for 20.0s.
- **A4:** Defeating an enemy with Warrant transfers it to a nearby unmarked target and recalculates its duration. The description does not promise the nearest target.
- **A5:** Increases damage dealt by +20% when only 1 target with Warrant remains.
- **A6:** Immediately restores Crime to its cap after casting Retribution (Burst), per `1071AwakenEffect`; entering combat is not this described trigger.
- **Resonance 3:** +1 level to Pursuit (Normal), Doubtmark (Skill), and Retribution (Burst).
- **Resonance 6:** Reduces Lakshana resistance by 10% on enemies marked with Warrant.

### Crime configuration and packet-key lookup recovered, October 18

The expansion's warning-free **`BP_ChaosLogicObj#/1/Properties`** contains the resource configuration missing from the short source inventory. The player object's empty instance properties do not mean the referenced Blueprint CDO is empty. It explicitly configures **SinsCoreMax=1000**, **AutoIntervalTime=0.2**, **HOLD_DURATION=1.0**, HuntMark class `Buff_Chaos_skill_Huntmark`, UltraSkill marker `Buff_Chaos_Ultraskill`, and completion event `Ability.Player.Chaos.BranchChargeComplete` with `Buff_Chaos_GE_BranchChargeComplete`.

`DamageGENameAddSinsCoreKeyMap` maps 30 ordinary/transformed damage identifiers to same-named rows of `DT_ChaosEffectFigure`. For example the ordinary and `_Ultra` Melee1 rows both contain **22**, a Crime input distinct from their Ultimate ChargeAdd. The new A1 scalar binds `Chaos_Star1ExtraSinsCoreCoe` with explicit multiplier **1** and constant row **0.4**. Older `ExtraSinsCoreCoe`, MeleeAddSinsCore, SkillAddSinsCore, AutoSinsCoreRatio, UltraSinsCoreRatio and BlockDodgeAddSinsCore fields explicitly serialize multiplier **0**; `UltraAutoAddSinsCore` omits it and binds row **30**, while the separate display row is **150**. Thirty per configured 0.2 interval would arithmetically give 150 per second, but the omitted effective scalar, scheduler/admission and mutation consumers remain unproved. Do not zero that path or infer universal per-hit generation from the map alone.

The same asset's only inspected Blueprint function, `CanPlaceChaosBeaconActorByBlueprint`, checks world-placement restrictions and character-state admission; it does not implement Crime grant/spend/clamp or Warrant transfer. Native resource mutation, effective per-packet dispatch, transfer recipients and real clock remain open at this precise boundary. Natural QTE trigger/cost/multiplicity remains independently unresolved and must not be removed from the question list.

### Passive description coverage, reviewed October 18

Saved description source: `guide_chaos.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Justice in Time describes a Remora-end packet at 800% ATK, with 45% added total damage per second beyond the base five seconds, capped at 300%, and duration reset on reapplication. Prime Target describes a Warrant damage bonus of 30%. The damage row alone does not supply the elapsed-duration expression, additive-versus-relative interpretation, expiry/reapplication order or captured target. Keep these description-level conditions distinct from the recovered Crime configuration below.

### Warrant bonus selector and Remora consumer conflict recovered, October 18

Initial `Cau_Chaos_Passive2_DamUp:CustomCalculateBaseMagnitude` (`#/2`, warning-free) casts OwnerActor to player and validates HitActor ASC. No target **Buff_Chaos_skill_Huntmark** explicitly gives zero; a marked target selects **Chaos_Passive2DamageUp=0.3** when the owner has `Buff_Chaos_Passvie2`, otherwise **Chaos_HuntMarkDamageUp=0.2**. The 0.3 replaces 0.2, not an additional 0.3 on top. `GE_Chaos_DamageBase#/2` and `GE_Reaction_3_new_1071_Damage#/2/Properties/CustomApplicationModifierInfos` bind it to additive **DamageUpGeneralBase**, multiplier 1. The copied Skia rate/cap/ClampValue fields are not read by this expression. Native source attribution and delivered modifier capture remain open.

The same reaction packet binds **FinalDamageUp** to `Cau_Reaction3_1071_DamUp`. Its warning-free `#/2` checks owner **Buff_Hathor025_Passive1**, despite the Chaos/1071 name, then returns `Reaction_GuangXiang7_DamageUP_1071=3.0` at coordinate 0 on that valid successful path, otherwise explicit zero on the failed gate. There is no elapsed-duration read in this helper. The described Remora per-second growth is therefore not recovered by this constant calculator; its native duration settlement and unexpected Hathor gate remain a concrete consumer/description conflict. Do not replace that marker with Chaos's passive or invent a time curve without evidence.

### Saved transport/player callbacks checked, October 18

The independently inspected **GA_Chaos071_WorldTransport** uses a configured **0.99** WaitInputRelease; its auto-release branch requests **Buff_Chaos_TransformPlayMontage**, then ends. **BP_ChaosBeaconActor** handles portal mesh/material/particle sequences and, through its owner player state/main character, conditionally requests TryActivateAbilitiesByTag while that transform marker is present. Its inherited CDO has a read warning. Neither this reached route nor **player_071_Chaos**'s LevelEX-dependent material/face-mask callbacks supplies a Roulette/Supply/Time Slomo combat ledger or a damage clock. Their transport/visual waits are therefore not imported as combat recovery or cooldowns. Complete function streams and scoped warnings are in . Native combat logic and the concrete owner gaps below remain the return conditions; do not repeat these portal/material graphs as a new combat source.

### Warrant resistance request and Skill cooldown branch recovered, October 18

The warning-free repaired **GC_Chaos_Skill_HuntMarks** is joined from **Buff_Chaos_skill_Huntmark#/2** through `GameplayCue.Display.Chaos.Huntmarks`. **OnWhileActiveEvent#/3 → graph#/1** reads the incoming event's EffectContext **EffectCauser**, requires it cast to ability character and have **Buff_Chaos_LevelEX**, then requests **Buff_Chaos_LevelEX_DmgResistDown** on **MyTarget** after its ability-character cast. SourceObject is the cue actor; the call does not explicitly pass EffectCauser as an instigator argument. The debuff (`#/2`) adds **DamageResistLakshanaBase=-0.1**, from multiplier -1 × **Chaos_StarExtra2ResDown=0.1**, with duration **Chaos_HuntMark_Duration=15**, target cap1 and NeverRefresh/NeverReset. No explicit removal request appears in this cue's empty OnRemove/EndPlay bodies. The configured 15-second debuff is not proved to expire exactly when Warrant ends; effective context, native admission and expiration remain separate.

**GC_Chaos_SkillCD** is joined from **Buff_Chaos_SKillBreakCDTime#/1**, whose HasDuration input is **0.2**, through `GameplayCue.Display.Chaos.SkillCD`. OnWhile caches **StartTime = GetWorldTimeSeconds_HybridMode(playerController, playerController)** after MyTarget's player/controller lookup. OnRemove repeats that clock lookup and requests **GE_CoolDownBase_ChaosB** when elapsed **>= 0.2**, otherwise **...ChaosC**. The exact boundary selects B; there is no explicit role check in this graph. Both cooldowns (`#/2`) are HasDuration, target cap1, BuffSave, with the short-press Skill cooldown tag. B binds **CoolDwonCurveTable:GA_Chaos071_Skill=15**, C binds **CoolDownBreak=3**, each multiplier1 and constant across coordinates 1..7. The separate unsuffixed Chaos cooldown's fixed2 is not either requested branch. For example elapsed 0.199 selects the 3 input, while 0.2 selects 15; neither example proves actual elapsed wall time or delivered cooldown after native cancellation/clock handling.

Complete functions and carriers: , `chaos-mark-cooldown-carriers.txt`, `cooldown-source.json`, `roster-cue-halo-extraction.json` and `cue-carrier-joins.json`. This closes serialized branch/recipient/curve selection, while leaving Crime mutation and Warrant transfer unanswered.

### Burst presentation callbacks distinguished from Crime mutation, October 18

Both warning-free **Buff_Chaos_Ultraskill#/2 and Buff_Chaos_Ultraskill_display#/2** join **GC_Chaos_Skill_UltraskillDisplay**. OnWhile/OnRemove require **IsClientOrStandalone(MyTarget)** and cast that actor to **player_071_Chaos**, then call its **UltraSkillStart/End**. These are Blueprint player methods, not an unexamined ChaosLogicObj resource API. Following **player_071_Chaos#/21 → graph#/8:7434** and **#/20 → 7651** reaches material-array initialization, weapon mesh color parameters and timeline Play/Reverse. Timeline Update/Finished likewise writes/clears presentation state; no Crime/resource/damage/grant write appears in this reached chain. This narrows that misleading name lead without excluding the separate native transformed-state consumer or proving full player defaults. The player asset retains two scoped object-read warnings; these complete functions are preserved. Sources: `chaos-player-full-functions.txt`, `remaining-control-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

## Unknown execution policies

Historical native-certainty limits, not reopened collection requirements. Selected implementation decisions for these questions are complete above.

- **Real timing/admission:** Montage notify coordinates for Skill (`char_Chaos_skill_skill_1_Montage`) and Burst (`char_Chaos_skill_ultraskill_Montage`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Crime & Warrant transition mechanics:** Exact Crime consumption rates per transformed action, natural Warrant application/transfer timing, and transformed state duration remain open native questions.
- **Spatial geometry & targeting:** Warrant target-search radius on transfer (A4) and multi-target Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Resolve Crime generation/spending and Warrant transfer/extension conditions, target caps/selection/distribution, transformation entry/exit, and timing for each reached action.
- Resolve the native QTE trigger, action admission/readiness, costs, timing and multiplicity. The known card's exact direct-packet energy hypothesis does not establish native QTE energy eligibility or dispatch.

## Prior findings

- [`reviews/q10-game-action-binding/chaos-action-binding-model.json`](../../../../reviews/q10-game-action-binding/chaos-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/chaos_actions.rs), [`chaos-normal-chain.json`](../../../../reviews/q10-game-action-binding/chaos-normal-chain.json), [`chaos-skill-packets.json`](../../../../reviews/q10-game-action-binding/chaos-skill-packets.json).

## Research history

- Chaos8/8 selected handoff complete. Next campaign owner: **Chiz, packets block**, reusing her existing action and Grain/Fons evidence first.
- Tried methods: original/supplement GA/notify/cue/logic/config/conditional helper reads, public/frozen multiplier comparison, prior QTE receipts and visual callback exclusions. Duration/Crime/transfer decisions are finalized checked inferences; native-body accuracy polishing is deferred until implementation.


Public source now binds these existing combo/continuation predicates through
ordinary `attack[hold=...]` and `window=hold`. The observer uses the same minimum
hold table as native admission; it copies no progress or hitlag constants into
the compiler. A plain stage advances through actual queued events before a
funded full-stage gate can be observed. Window completion alone cancels nothing.

Independent source checks reproduce the Normal3 gate2,113,639us and retained
contacts214573/756182/918306/2035883/2280330/2997297 on each recipient. A longer
400ms hold after the minimum window319801us rejects until400000us without a
retry; an idle200ms hold also rejects. Two funded continuing Verdict windows
yield first at3,328,841us after ordinary readiness2,902,033us, retain168 packets
on two dummies, and reconcile Crime initial1000 +2292 granted -2000 spent
-292 overflow =1000 final. A parent admitted transformed at14s survives natural
form expiry14.718888s, yields17.329958s with its captured100ms predicate and
starts a new ordinary Branch at17.496649s. Native source artifacts, exact report/
RNG replay and WASM transport retain those selections; these checks establish
the selected model, not observed game timing.

## Relative combat timer binding

`char_Chaos_skill_ultraskill_Montage` Common.EndAbilitySkill at 4.802493 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
