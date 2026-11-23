# Daffodil action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1054`.

**Responsibility:** Daffodil outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** the selected Rust owner model binds the ordinary clock, secondary and held actions, Phantom Step, Resonance/Insight, Break/Discord and linked equipment. Checked hypotheses establish the declared implementation model, not unobserved native bodies. Additional source control lowering and final shared interactions retain their queue boundaries.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected collection handoff (October 19, 2026)

These selected rules supersede the historical partial-runtime slice below. Source constants come from [consumed typed damage rows](../../../../crates/data/src/native_damage/daffodill.rs), [consumed typed owner parameters](../../../../crates/content/src/characters/daffodil/parameters.rs), `1054AwakenEffect` (cache-only awakening source), original/supplement GA/CDO/notify/actor/condition exports and frozen public multipliers. Source/derived/inferred statements remain separate; a complete selected model does not establish all native dispatcher bodies or executable support. Previous three-plus methods already include native/notify extraction, direct-packet receipts and public comparisons, so do not repeat protected-body research for remaining lifecycle decisions.

### Complete packet specification

**Mixed.** Direct packets use Chaos/ATK and live ordinary crit (FixedCritRate0=no override). Normal owns Melee/Branch/plunge/entry/riposte talent, Skill owns ordinary Resonance/Echoes, Burst owns Finale, Support owns QTE. R3 adds1 to Normal/Skill/Burst including corresponding alternate rows, **not Support**. Preserve full13-entry arrays; level1/8/9 checks and per-packet channels below are portable values.

| Suffix after GE_Player_Daffodill_ | Count in its full action | Coefficient1 /8 /9 | Charge / Cycle / Break |
| --- | ---: | --- | --- |
| Melee1_Damage |1 |.167 /.286 /.309 |1.041 /1.737 /.209 |
| Melee2_Damage; Melee2_1_Damage |1 each |.153 /.262 /.283 |.957 /1.596 /.192 |
| Melee3_Damage |2 |.132 /.226 /.244 |.824 /1.374 /.166 |
| Melee3_1_Damage |2 |.062 /.106 /.115 |.389 /.648 /.078 |
| Melee3_2_Damage |1 |.141 /.242 /.261 |.878 /1.465 /.177 |
| Melee3_3_Damage |1 spawned actor impact |.263 /.451 /.487 |1.638 /2.733 /.329 |
| Melee4_Damage |1 |.524 /.898 /.970 |3.271 /5.457 /.658 |
| Melee4_1_Damage |1 |.193 /.331 /.357 |1.205 /2.011 /.242 |
| Melee4_2_Damage |1 |.127 /.218 /.235 |.793 /1.323 /.159 |
| Melee5_Damage; Melee6_Damage |2;4 |.101 /.173 /.187 |.631 /1.052 /.127 |
| Melee5_1_Damage |1 |.473 /.811 /.875 |2.95 /4.922 /.593 |
| AirAttack_Damage |1, landing |.424 /.727 /.785 |2.7 /4.4 /.5 |
| Branch1_Damage |1 |.116 /.199 /.215 |.722 /1.213 /.148 |
| Branch2_Damage |1 |.096 /.165 /.178 |.594 /.998 /.122 |
| Branch3_Damage |1, final |.389 /.667 /.720 |2.42 /4.065 /.496 |
| Branch4_Damage |1 |.113 /.194 /.209 |.702 /1.179 /.144 |
| Branch5_Damage |1 spawned actor impact |.158 /.271 /.292 |.984 /1.653 /.202 |
| Branch6_Damage; Branch7_Damage |2 each |.096 /.165 /.178 |.594 /.998 /.122 |
| EntryAttack1_Damage; EntryAttack1_1_Damage |1, route aliases |.681 /1.167 /1.260 |.579 /.962 /.119 |
| EntryAttack2_Damage; EntryAttack2_1_Damage |1 each |.553 /.948 /1.024 |.470 /.782 /.097 |
| EntryAttack3_Damage |1 |.553 /.948 /1.024 |.470 /.782 /.097 |
| EntryAttack3_1_Damage |1 with A2 notify condition |.553 /.948 /1.024 |.470 /.782 /.097 |
| EntryAttack4_Damage |1 |1.105 /1.894 /2.045 |.940 /1.561 /.193 |
| AppearSuccess_Damage |1 conditional parry/A2 finisher |3 /5.141 /5.553 |0 /0 /2 |
| PerfectEvadeAttack_Damage |1 |1.040 /1.782 /1.925 |2.6 /4.3 /2.5 |
| QTE_Damage |1 |2 /3.428 /3.702 |5.6 /0 /2.5 |
| Skill1_Damage |2 in ordinary Resonance |.409 /.701 /.757 |.831 /1.376 /.164 |
| Skill2_Damage; Skill6..9_Damage |1 each, ordinary Resonance |.337 /.578 /.624 |.685 /1.134 /.135 |
| Skill5_Damage |1, ordinary final |.498 /.853 /.922 |1.012 /1.676 /.199 |
| Skill3_Damage |6 in Echoes |.348 /.596 /.644 |.708 /1.172 /.139 |
| Skill4_Damage |1, Echoes final |.911 /1.561 /1.686 |1.853 /3.068 /.365 |
| UltraSkill1_Damage |1 |.493 /.845 /.913 |0 /0 /.154 |
| UltraSkill3_Damage |2 |.493 /.845 /.913 |0 /0 /.154 |
| UltraSkill2_Damage |1 |.253 /.434 /.468 |0 /0 /.079 |
| UltraSkill6_Damage |2 |.254 /.435 /.470 |0 /0 /.079 |
| UltraSkill4_Damage |7 |.511 /.876 /.946 |0 /0 /.160 |
| UltraSkill5_Damage |1 |2.183 /3.741 /4.041 |0 /0 /.682 |

Ordinary chain stage counts1,2,6,3,7 =**19packets/5.463ATK/19.902charge/33.196Cycle/4.001Break** at8; source-row sums replace the old22/36.8/4.4 summary. Stage3 is twoMelee3, twoMelee3_1, oneMelee3_2 and its one actorMelee3_3; stage5 twoMelee5, fourMelee6 thenMelee5_1. Shared actor configuration's one-count cap is not two seconds of repeated hits. Branch is **9packets/2.156ATK/7.798charge/13.100Cycle/1.600Break**; Branch5 shares actor geometry with Normal3_3 but uses its own coefficient. No source Branch6/7 omission is permitted because the public short inventory skipped them.

Ordinary Resonance is8 packets **5.145ATK/6.099charge/10.098Cycle/1.202Break**; the five equal-ratio intermediate hits are **five distinct mapped rows Skill2/6/7/8/9**, not five calls of a single row. Echoes at2 Resonance stacks is7 packets **5.137ATK/6.101charge/10.100Cycle/1.199Break** before its captured stack modifiers. Normal stack0/1 uses ordinary route;2 replaces it with Echoes, not both trains. Finale is14 packets **13.712ATK/2.501Break** with zero raw energy/Cycle. In particular UltraSkill6 is.435 at8 versus UltraSkill2.434; collapsing them changes damage despite equal hit count. Plunge height multiplier1+h uses declared0..1 scenario, paying airborne/landing time.

Phantom Step ordinary entry has **5 primary packets**: Entry1(or route-alias1_1),2,2_1,3,4; at8 sum**5.905ATK/2.929charge/4.869Cycle/.603Break**. With A2, the explicit Entry3_1 conditional request adds one.948/.470/.782/.097 packet; never unconditionally append it at A0 merely because the notify retained a condition array. The final AppearSuccess5.141 is **one** separate packet when an authored successful parry occurs, or selected A2's failed-counter permission admits it; no success-plus-failure double finisher. These route predicates/one-entry charge and full-energy reward are finalized in forms/resources. B4 adds.80 general damage to Phantom Step primary/conditional finisher packets only, not ordinary Normal, QTE or an extra .8ATK coefficient. Categories of these rows remain native A; Break contribution packets below are separate.

**True Break contribution and A5:** use the [shared Break formula/base-level model](../../shared/cycles-and-break.md#break-base-values-needed-by-daffodils-open-handoff), not another ordinary2ATK hit. Daffodil has intrinsic Light Prism **+.20 Break bonus**; with A4 and n own Insight stacks on that recipient, add **.15*n** in that same Break-bonus zone. Her own contribution is `B(level)*(1+BI/300+.20+.15*n*A4)*(originalCap/3)*Fdef*FchaosRes*eligibleSpecial`. A5 repeats **that Daffodil contribution once per n** after the ordinary team Break event; other party contributions are not repeated. Snapshot transition-time stats/Insight count for the whole group, zero raw energy/Cycle/gauge and no crit/ordinary bonus, no recursive Break.

The retained `ExtraUnbalance_Damage` row has fixed fallback2ATK/category metadata, but its explicit State.Damage.Unbalance/OnceExtraUnbal/display-Unbal route is modeled as **replacement with this Break contribution**, following the independent “additional Break Damage per stack” description. Do not count both that fallback2ATK and the repeated contribution or multiply the whole team result by n. Native executor switch remains inferred; this choice preserves the BI/original-cap dependency rather than disabling A5 or inventing ATK-only Break. Check at80/BI0/cap30/DEF.5/RES0: baseline Light Prism contribution21618; n2+A4 yields27022.5, A5 adds two27022.5 packets, total own81067.5 before any other actors. Changing ATK/CR alone leaves these numbers unchanged; reducing live gauge cap never replaces originalCap30 in the HP formula. R6/A6 target/DEF scope is separately applied through ordinary Break channels.

### Complete timing specification

For a reaction entry with an earned Phantom Step, the observed QTE animation plays through before Step begins automatically. The checked model uses the guarded `GA_Daffodill_QTE` transition to `AppearAttack` at 1.150000 seconds of montage progress. The QTE hit's 0.349650-second connected pause moves this handoff to 1.499650 real seconds after entry. The 0.900691-second ordinary input gate belongs to the standalone QTE. The montage asset has a 5.5-second segment that includes its tail, while the charge-backed route explicitly selects `AppearAttack`. A successful QTE parry finishes first and Step uses a remaining charge. An interruption or defeat before the transition discards the pending Step.

**Mixed; selected real-time conversion.** Montage progress advances1/s plus each connected packet's explicit owner pause `duration*(1-rate)`: Melee2_1 .0099s; Normal3_3 actor.0576; Normal5_1 .0768; branch final.1188; Phantom Step Entry1 .0594/Entry4 .080; plunge.0396; riposte.297; QTE.34965; ordinary Skill final.0475; Echoes final.1188. Omitted duration in reached CDO adds0, not another universal pause. Count once per causal packet, never per dummy. Spawned actor period/delay clocks and all buff/CD/resource timers run on real time. This is an explicit checked timing model from source Notifies/HitPauseData, not measured animation speed or a universal Time Stop of the whole party.

| Action | Primary hit progress / source event | AnySkill recovery |
| --- | --- | ---: |
| Normal1 |.18530756 |1.0030121 |
| Normal2 |.12462029 row2,.34545302 row2_1 |.70544714 |
| Normal3 |.186126,.22842233 row3; .35535678,.39773178 row3_1; .56924194 row3_2; .907714 actor3_3 |1.6077594 |
| Normal4 |.25233728 row4,.4901387 row4_1,.6389153 row4_2 |1.3798112 |
| Normal5 |.2501529/.4835633 row5; .3000073,.42494434,.5344058,.59725183 row6; .8003871 row5_1 |1.1613348 |
| Mirror on Water |.10405359 actorBranch5; .40008837 Branch4; .40078044 Branch1; .78410625 Branch2; .83313024,.8927235 Branch6; .9350618,1.0077134 Branch7;1.2080154 Branch3 |1.4333231 |
| Phantom Step |.2539521 Entry1,.610256 Entry2,.700212 Entry2_1,.789783 Entry3, A2-only.877311 Entry3_1,1.3173304 Entry4, conditional1.3176104 AppearSuccess |1.5056272 |
| Plunge, after authored landing |.05378426 |.281249 |
| Close Call riposte |.22976089 |.874201 |
| QTE |.7450327 |.90069103 |
| Ordinary Resonance |.19667533/.33190313 Skill1; .67002934 Skill2,.7498901 Skill6,.8326246 Skill7,.9165188 Skill8,1.000193 Skill9;1.1055893 Skill5 |1.2104917 |
| Echoes |.1189588,.19501123,.28375718,.39368528,.47179937,.5459915 Skill3; .78439486 Skill4 |.9886282 |
| Finale |2.8493748 Ultra1;2.9060674/2.960202 Ultra3;3.04996 Ultra2;3.100386/3.1496067 Ultra6;3.7759802,3.8360596,3.891205,3.9523182,4.0195265,4.0979834,4.154141 Ultra4;4.657557 Ultra5 |5.0889783 |

Derived full ordinary chain **6.00166464s**, branch1.5521231, Phantom Step1.6450272 at A0 (including the.08 final primary pause), ordinary Skill1.2579917, Echoes1.1074282, riposte1.171201, QTE1.25034103. An unparried charge-backed QTE keeps this same1.250341 real-time lock; only a live eligible parry extends recovery to the shifted final marker at1.667260, including its.34965 pause. A late, dead-target or cancelled parry neither spends a charge nor lengthens ordinary Support recovery. Finale has no explicit positive-duration hitpause in these rows, so5.0889783. ActorNormal3_3/Branch5 each has one immediate finite impact, not an invented1s repeat. Entry conditional finisher comes after Entry4's.08 pause; it does not deliver concurrently merely because the source coordinates differ by.00028. The target mark/self buff/charge reset event **UltraSkill.Daffodill.Appear is at0**, not its first2.849s hit or a fictitious damage event at2.7.

Earliest cancellation gates, before adding only already caused pause: ordinary Normal Jump stages1..5 `.21649006,.16153404,.90169144,.29100367,.89238024`; Skill `.28317827,.4270513,1.1789385,.77037495,.98510677`; Move `.3834363,.6002742,1.8493471,1.5609148,1.4127365`. Branch Jump1.2661986/Skill1.2999074/Move1.7125585; Step Jump1.3669974/Move1.9989663/SkillAnySkill. Plunge Jump.149206/Move.750726/SkillAnySkill; riposte Jump/Skill0, MoveAnySkill. QTE JumpAnySkill/Move1.2911102. Ordinary Skill Jump.2886063/Move1.7130674/SkillAnySkill; Echoes Jump.20848411/Skill.86328506/Move1.5994873. Finale Move5.0848727/Jump5.0862412/SkillAnySkill. Cancellation drops future montage requests but retains already spawned actor/captured proc entries; no refund of committed cost/stack/entry charge. Field exit follows this same packet rule, not an automatic whole-cast packet dump at first impact.

Switch requires recovery plus explicit no-change lock expiry: QTE lock.86302876, Finale5.087548. Ordinary switch has no extra invented time tax; incoming Phantom Step, if admitted, owns its listed recovery. AppearMelee source hit.18592605/recovery.9994027, Pre hit.18592605/recovery.9976763 reuse Normal1, not an extra free stage. Front dodge Jump.082544/Skill.333754/AnySkill1.014541; back Jump0/Skill.494283/Move.836966/AnySkill1.0070679; neither has outgoing damage. During a live Normal, special Displaced Edge may begin at that stage's Jump gate, pays the same dodge15 and front recovery1.014541, suspends/resumes the unfinished Normal progress and preserves its stage. Selected inference uses invincibility only through that dodge action or its earlier cancellation; no re-emission of already landed Normal hits. Its real time is added to DPS, continuity deadline is not lost solely to this legal suspended action. A normal dodge outside the combo does not preserve a nonexistent suspended stage.

Skill16s cooldown begins at its actual notify: ordinary.18229023, Echoes.11032817 from cast progress, before their first hit. Interruption before the notify applies native cancel curve **3s from interruption**; after it, retain the already started16s deadline, no recovery-time duplicate. Ordinary invalid input does not itself interrupt. Burst cooldown20s starts accepted commit; full120-energy parry reward does not clear it. Normal/branch/plunge/riposte/Step have only their stated admission/recovery/cost limits; QTE is one qualified-switch consequence. Successful explicit Critical Dodge authorizes one next-Normal Close Call, consumed by use/another non-Normal action or field exit; no guessed native one-second authorization is imported.

### Complete resource and admission specification

**Mixed.** Native DT_GameplayAbilityCostData gives Finale **120energy**; DT_PlayerPackData gives owner cap120. Initial0 unless a legal preload is declared; spend120 at accepted commit, retain fractional carry/overflow. Ordinary/alternate Normal, ordinary/echo Skill, plunge/riposte/Step have no additional energy tax in reached rows; ordinary/special dodge costs**15 stamina** and requires the [scenario stamina profile](../../shared/energy-and-gauges.md). No free failed dodge/parry is assumed. Use shared positive applied-HP per-packet energy/Cycle grants, own/other.6 relation, each receiving actor's efficiency, exact carry and cap independently; fatal positive HP qualifies, zero/miss/dead target does not. No per-target resource multiplication and no auto reset after waits, field changes or reporting periods. True Break/A5 packets grant0 raw energy/gauge.

**Resonance stacks:** one persistent owner pool0..2, starts0, no timer (native SkillDamUp Infinite/limit2). Each successfully admitted **actual party Support/QTE cast**, including her own, grants1 once regardless of hit/miss/target count; direct fake API packet inputs or ordinary switches do not. Preserve stacks off field. At Skill admission snapshot/consume current n (0/1 ordinary Resonance;2 Echoes), capturing `n*(.50 or A1?1.00)` **general Skill damage addition** and `n*(1.25 or A1?2.50)` **additive Skill Break Efficiency**, only for that cast's Skill-tagged packets. New Support during its animation adds to the next pool, not this captured cast. A failed input consumes none; interruption after accepted commit does not refund. With n2, base extra general+1.0/Break efficiency+2.5 means gauge multiplier3.5 at A0; A1 replaces per-stack values, yielding+2.0/+5.0 and multiplier6.0, not an additional old+new set. Do not apply this modifier to her Normal/Entry/Burst/true Break contributions.

**Phantom Step charges:** successful Finale's progress0 event **resets to2** and refreshes one30s entry-admission handle; native request100 is clamped by source cap2, never100 delivered uses. Each ordinary re-entry from a distinct actor while handle live, a living target available and charge>0 consumes1 at entry admission and performs one Step train, capturing the lowest-index living dummy; later parry eligibility remains bound to that target. Same-actor switch, readiness polling and hybrid-server predicate bypass do not create a new charge. No target or expired handle falls back to ordinary non-damaging entry, preserving charges until expiry if no admission occurred. At expiry remove unused charges/handle; no periodic recharge and no refund after cancellation. Ordinary target Insight can outlast this separate self handle. A qualified adjacent switch starts one QTE. A live charge then admits one regular Step at the QTE animation transition. A successful authored QTE parry spends one charge for its finisher, and the subsequent Step requires its own remaining charge.

Within an admitted Step, authored parry success against its captured living target before the shifted final marker admits one **AppearSuccess** finisher at1.3176104 progress; if no parry succeeded, A2's final-counter fallback admits that same candidate once instead. A2 also controls its additional Entry3_1 notify. A positive finisher connects → **fill energy to120 once**, not120 per dummy plus ordinary share or cooldown reset; retain/record normal carry and overflow conventions. No energy-fill on an ordinary un-parried A0 Step, raw Entry rows alone, mere permission display, or failed final damage. The selected runtime coalesces the conditional finisher into one cast-scoped candidate across success and A2 fallback. The available Linko `Buff_Radio072_Daffodill`/`currentdisplayID` 1054+A2 caller has no distinct damage consumer in this model and cannot create another finisher or bypass charge, target, field or recovery checks. Its ten-second `ExtraParryDamOK` permission is therefore not a separate runtime token pending evidence of a distinct consumer. The actual final-marker callback decides A2 fallback and preserves one finisher/reward without recursive recharge.

Save owner energy/Cycle ledgers, Resonance pool and captured active-cast n, separate self Insight/entry handle expiry and charges, per-target Insight stacks, parry authorization/permission generation, skill/Burst CD/recovery, suspended Normal stage, pending actors and relevant equipment handles. A0 zero-start cycle can fund120 using six ordinary chains plus Skill: **6*19.902+6.099=125.511**, without invented entry/parry/Break grants. Missing required state rejects an ordinary input; no hidden wait, source mutation reset or disabling of a passive makes it legal.

### Native source resource bindings

`start_resonance` preloads the persistent integer pool in 0..2 before entry;
missing defaults to zero and explicit zero survives inspection/export. This
uses the same native pool/cap as actual party Support admission and Skill
capture, without granting Eyes, a Burst handle or Phantom Steps.
`.resources.resonance` and `.resources.phantom_steps` read settled counters;
`.eyes.active`/`.mods.eyes` and `.phantom_handle.active`/`.mods.phantom_handle`
read the separate self deadlines (seconds), with half-open expiry. These names
do not query target Insight or create a timed handle from a preload.

`simulation/tests/source_resonance.rs` independently checks ordinary 3.001
versus Echoes 2.999 coefficient sums at talent 1, A0/A1 additive stack damage,
two continuing Skills, two actual Zero Supports across field exits, integral
capacity/ownership validation and whole serialized native replay. Party projection
now declares the native clock/cancellation/secondary/Phantom/held models before entry;
ordinary source `burst` pays120. The source scheduler check retains the native
two-charge/30-second handles, including the last microsecond and exact expiry,
without internal configuration overrides. The executable
[A0 source](../../../../examples/daffodil-skill-burst-packets.nte) pays two Finales,
retains continuing ordinary chains/Skills and two Zero Supports, and consumes
one Phantom charge on each ordinary return. Its two-dummy checks preserve each
entry's253952us first contact and area recipients, with full native replay.
Source charged-entry parry and input windows bind the native routes below;
explicit forced interruption remains in the shared Q4-L5 scenario integration.

### Complete effect specification

**Mixed.** Finale's actual progress0 event applies **three separate states**, deduplicated by cast: self InsightEyes general damage+.10 for**30s**, self AppearOK resets**2 charges for30s**, and target Insight to all eligible dummies. Self damage is cap1/refresh, survives switching until its real expiry, never doubles with two marked targets. **Target Insight is45s** from the native custom selector (not legacy30). Without A3 remove one old own stack before applying one, so the chosen normal state has1 refreshed stack. With A3 skip replacement, add to cap2, common expiry**9999s**, refresh on reapplication. Keep that finite source value rather than promoting HasDuration to Infinite; target death/combat exit removes immediately. Field exit/source death does not itself erase the accepted target mark. A3 affects target marks, not the independent30s self bonus/entry charges. Insight stacks are not consumed by Break or a Phantom Step; otherwise the A3 accumulation and repeated A4/A5 count would disappear without a source consumer.

Resonance's persistent two-stack Support pool and captured cast-only damage/Break efficiency are in resources. At n0/1 ordinary Skill, n2 Echoes; GeneralDamage sum+.50*n (A1 replaces by1.00*n), gauge multiplier `1 + existingBreakEfficiency + 1.25*n` (A1 uses2.50*n), only State.Damage.Skill. No stack count is inferred from the magnitude helper itself. Actual successful party Support admission is the grant event, deduplicated once, and a later Support adds only to the next pool after the current cast consumes its snapshot. Native Infinite duration500 is not a500s expiry or an extra two Skill casts.

**Blade Draw B2:** on a real [shared Discord generation-pair callback](../../shared/cycles-and-break.md#selected-aurelia-nova-stain-and-discord-dependencies) to an eligible breakable target, apply/stack native GE_Reaction_AnHunZhou_1054, **.10 of original capacity per stack, cap2, shared refreshed30s expiry**, remove on combat exit. It is active while Daffodil is living on the deployed team, including off field. Explicit native limit2 gives **maximum.20 owner reduction** in the selected non-MMO profile; do not change it to3 to reach the description's30% ceiling. The [independent Discord guide](https://www.nte.wiki/guides/discord-reaction-guide/) also describes a full20% reduction. Preserve the public30% wording as a source conflict/upper ceiling rather than silently asserting three settled stacks. In MMO mode the source calls a native coefficient getter: require an explicitly declared mode coefficient per stack instead of universal.10, retain stack cap2 and overall.30 reduction ceiling. This is a source-parameterized mode, not disabling the mechanic or inventing a getter output.

At each cap change derive `newCap=originalCap*(1-min(.30,n*coefficient))`; if not Broken preserve already depleted gauge amount: `newGauge=clamp(oldGauge + newCap-oldCap,0,newCap)`. If that reaches0 use one causal Break transition, never several for one cap/stat write. Apply the cap step before that Discord's separately declared direct gauge deduction; if already Broken, update its future max only. At cap refresh30 without another reduction; two positive cap stacks never become three by ordinary source refresh. On expiry/exit restore capacity and increase non-Broken gauge by restored capacity delta, preserving damage already dealt to that bar. Snapshot current capacity at a new Break for the shared inferred cap/reset-rate duration; a separately explicit target Broken duration takes precedence. Later cap changes do not restart an ongoing Broken deadline, and recovery restores the then-current cap. **HP Break contribution always uses original capacity**, not the reduced cap, as the independent calculation also distinguishes.

Example declared original70/currentGauge50/Discord fraction.20: first owner stack reduces cap70→63/gauge50→43, Discord14 leaves29; second cap63→56/gauge29→22, Discord14 leaves8; third refresh leaves cap56 then Discord reaches0 once. No invented baseline cap reduction is added to force the described30%. No-bar training targets cannot lose a bar, but retain real Discord callbacks for other eligible owners. This cap effect does not scale with CI, ATK or ordinary damage. Pending accepted cap debuffs retain their own30s if source leaves/dies; fresh dead-granter callbacks are illegal. Source invalidity/zero capacity is a diagnostic rather than a guessed universal bar.

On a real target Break, snapshot Daffodil's own Insight count and source stats, then emit ordinary team contributions once plus A5 own repeats only. A4/Light Prism use their Break-bonus zone, A6 normal DEF-ignore channel; ordinary self Insight+.10 and Phantom Step+.80 do not multiply true Break HP. R6's target Chaos shred is enabled for that new Broken state before its contributions, while the causal direct packet used the pre-Broken state. A5 cannot deplete gauge/retrigger Break; target death cancels unapplied repeats. The reached Pas2DamUp carrier actually writes an unrelated DEF/armor configuration with omitted curve values; it is **not** the offensive.80 binding. Selected outgoing model binds the independent explicit .80 row/description to Phantom Step damage, without inventing missing defensive numeric values. Native enemy AI/mitigation of unmodeled incoming attacks remains outside the declared stationary/explicit applied-HP-event scenario.

### Executable Resonance capture

`daffodil.resonance_skill_capture_v1` binds the persistent two-stack pool to admitted party Support actions with native ability index4, including Daffodil's own QTE. A caller-declared QTE role without that native identity does not grant stacks. The admission callback runs once per cast independently of damage, miss or target count; ordinary field entry and rejected actions grant none. A living off-field Daffodil receives the same grant. `final_actor_states[].resources["daffodil.resonance"]` exposes the remaining pool and cap2.

Skill consumes the pool at accepted admission and stores the count by causal command. Its pending packets retain that count across later Supports and field changes; the next pool is independent. Zero/one stack selects the ordinary eight-packet train; two select six Skill3 packets and one Skill4 finisher. A1 replaces each captured stack's damage/efficiency values and adds nothing at zero stacks. The shared Break adapter adds the captured owner efficiency to eligible live outgoing efficiency, then applies the selected target multiplier once. Normal, QTE, Burst and true Break HP packets receive none of this Skill-only addition.

Independent Rust checks at talent1/ATK1000/DEF factor1/2 reproduce ordinary total1500.5 at zero stacks (including A1),2250.75/3001 at one stack A0/A1, and Echoes2999/4498.5 at two stacks. Raw gauge sums are1.202 ordinary and1.199 Echoes; multipliers are2.25/3.5 at one stack and3.5/6 at two stacks. Two continuing cycles preserve the pool, cap, resources and cooldowns; tests also cover later Support before pending Skill delivery, off-field grants, multi-target packet delivery and rejected admission. Native/WASM/actual Worker scenarios carry Echoes followed by an independently captured ordinary Skill through Break HP settlement, reports, aggregation and replay. These are model checks, not measured game timing: this binding retains the existing authored packet delivery time and does not implement the remaining montage cancellation, engine-owned Support eligibility or complete kit branches.

### Executable target Insight and Break capture

`daffodil.insight_break_capture_v1` binds Finale's progress-zero target application to the shared status service. Each living recipient keeps its own source-owned mark: cap1 with refreshed45s expiry, or cap2 with shared refreshed9999s expiry at selected A3. The self Eyes clock is independently30s at every awakening. Field exit/source death preserves accepted target marks; target defeat clears only that recipient's Insight, through both ordinary and true Break HP settlement. Half-open expiry excludes marks exactly at their deadline.

At a new Break, the engine captures each living contributor's own mark count on that recipient together with its BI/DEF/RES parameters. A4 adds `.15*n` to the Break-bonus zone alongside Light Prism's `.20`. A5 schedules `n` extra copies of that source's captured contribution; each has its own ordinary Break receipt and HP rounding, without crit RNG, raw energy, gauge depletion or another ordinary `2*ATK` packet. The selected causal order retains the original roster contribution group first, then repeats in source-slot order. A defeated target cancels remaining contributions. This ordering is an explicit model choice, not observed native callback order. [R6 target resistance](#executable-broken-linked-r6-resistance) is applied before this capture and stays separate from the personal A6 DEF-ignore channel.

Independent level80/cap30/DEF-factor.5/RES0 checks give baseline21618, one-mark A4 `24320.25`, and two-mark A4 `27022.5`; two-mark A5 emits three own contributions totaling81067.5 exact damage, while an unenhanced ally contributes18015 once. Tests distinguish source/recipient isolation, absent/expired marks, A4 without A5, A5 without A4, stack-cap refresh, the self30s boundary, retained captures and target-death cancellation. A continuing152s fixture pays120 energy for each of two Bursts, funds them with seven ordinary chains per cycle and retains two target stacks; at cap1 its two Burst-triggered generations contain two contributions of810.675, then three of900.75. Native/WASM/actual Worker scenarios transport both A3 and non-A3 continuing cases through full reports, aggregation and replay. Packet delivery still uses the existing authored timing adapter; full montage/cancellation, Phantom Step and Discord-cap mechanics remain separate owner work.

### Executable Broken-linked R6 resistance

`daffodil.broken_chaos_resistance_v1` applies the configured `.15` Chaos RES reduction on each actual new Broken generation from a living deployed R6 Daffodil, including off field. Each provider owns one capped target status. The causal direct packet used pre-Broken resistance; the new debuff is present before the group's Break HP capture and affects every eligible Chaos source, including other characters. Nature and other elemental channels remain unchanged. Source death/field exit preserves the accepted snapshot to the target's Broken deadline. Rose extends only existing statuses to the updated deadline; its stale recovery event cannot remove them. Actual recovery and either ordinary or true Break target defeat clear the status. Absent bars, zero effective packet efficiency, manual semantic tags and dead fresh providers do not fabricate this callback.

Independent level80/ATK1000/talent9/DEF1080/RES.20 checks give Daffodil first-Normal `4944/37` before the transition, `5871/37` during Broken and `4944/37` after recovery, retaining personal A6's DEF factor `20/37`. At original cap30, own Light Prism/A6/R6 Break is `821484/37`, Lacrimosa's Chaos contribution is `68457/4`, and Mint's Nature contribution remains14412. With two own Insight stacks and all six effects, each of the three own contributions is `1026855/37`, using additive Break bonus `.20+2*.15`, separate DEF ignore and target RES.05; both allies contribute once. Tests also distinguish other recipients, source-death extension to an exact new deadline, stale recovery, target-death cancellation and strict acceptance. Native/WASM/actual Worker cases include two funded all-six Burst cycles and a source-equipped Rose extension through reports, aggregation and replay. These checks certify the selected model and transport, not the native consumer, native callback order or unbound montage/Discord/Phantom Step behavior.

### Complete passive and awakening specification

**Mixed.** Source1054AwakenEffect/DT_DaffodillEffectFigure and the named calculators determine individual selection; count-based R3/R6 remain separate. These effects stay enabled whenever their unlock/selection is present, even if a declared no-bar/no-parry scenario naturally supplies no trigger.

| Selection | Concrete complete rule |
| --- | --- |
| Intrinsic Light Prism | Own true Break bonus+.20, not20% more gauge shred or ordinary damage. |
| B2 | Discord cap effect above: native non-MMO2*.10 for30s, no arbitrary3-stack reinterpretation; original cap retained for HP damage. |
| B4 | Phantom Step primary/conditional finisher general damage+.80, owner-only, active for the actually captured Step action; excludes Normal/riposte/QTE main/Skill/Burst/true Break. |
| A1 | Replace each Support stack's Skill general+.50 with1.00, Break Efficiency1.25 with2.50. No unconditional+100% to every ordinary Skill regardless stacks. |
| A2 | Extra conditional Entry3_1; failed Step counter may produce the one high AppearSuccess finisher and full120 energy on positive connection, following the same finite permission/reward route as successful counter. Linko's located1054-display/A2 marker cannot create a free new Step. |
| A3 | Target Insight add/refresh to cap2/duration9999, no old-stack replacement; removed death/combat exit, ordinary field changes retain it. Self Eyes/charges remain30s. |
| A4 | Own Break bonus+.15 per **own source's Insight stack on this recipient**, snapshotted at transition; no sum over other targets/party marks. |
| A5 | Additional own Break contribution once per that target's snapshotted Insight stack; no repeat of the whole team contribution or ordinary fallback2ATK on top. |
| A6 | Own DEF ignore+.15 from native Infinite Level6 modifier, applicable to direct and own Break DEF channel, combined under shared DEF-ignore rules. |
| R3 count>=3 | Normal/Skill/Burst+1, including their alternates, no Support+1. |
| R6 count>=6 | Target Chaos RES-.15 while Broken, cap1 per source/refresh to current Broken end, clear on recovery/death/scene cleanup. Whole target modifier permits other Chaos actors' eligible packets, not an unconditional personal bonus. |

R6's Broken-linked lifetime/transition order is inferred from the stated state condition, not an arbitrary20/30s timer. At target RES.20 it changes the new Broken Chaos factor.8→.95; source A6 against DEF1080/attacker80 gives `1080/(1080+918)=20/37`, not ordinary×1.15. With n2/A4/A5, originalcap30/BI0/DEF.5/RES0 and noR6, Daffodil contributes27022.5 three times (own ordinary plus two repeats),81067.5 total; another actor's contribution is added once. Her ATK/CR and another target's mark count cannot change this. Individual A3 selected alone does not imply A1/A2; countR3 may enable+1 talents on a different three-effect selection without them. Native supplied failed marker/cast paths are not silently converted into successful damage.

### Complete forms and alternate-action specification

**Mixed.** Ordinary Normal chain loops stages1..5, resets on different admitted action/field exit; preserves continuity through recovery+2 action-clock seconds. Displaced Edge suspends/resumes the same stage/progress and its not-yet-dispatched hits, pays dodge time/stamina, does not replay landed damage. Combo-held Mirror on Water requires held Normal input after that stage's Jump gate, then replaces remaining montage with the known9-request branch (source HoldTrigger numeric minimum absent: no invented.2s extra tax). Neutral hold admits the same branch at neutral recovery, no need to pretend all branch rows are independent direct commands. Plunge is one landing hit with authored airtime/height bonus; a successful explicit Critical Dodge grants the one Close Call action, separate from entry counter success.

Finale at progress0 creates the distinct self Eyes/entry and target Insight states. It does not replace all ordinary attack coefficients or give unlimited steps. Ordinary return from a distinct actor with a target and live charge consumes1 Step; route priority is airborne→actual authored parry route→targeted Shift, otherwise non-damaging appearance if no target. A source server-only admission bypass does not waive client-model charges. The targeted Step uses the listed primary train, A2 condition and one final-counter result; untargeted ordinary appearance is not a hidden full AoE train. A qualified adjacent switch starts Support, then automatically admits the charged Step at the end of the QTE animation. An authored successful QTE parry can emit its own finisher and reward before the same transition. An ordinary same-actor switch never procs Entry, Support or a charge reset.

Under the selected dummy model, entry counter success is an explicitly assumed parry while its selected dummy is alive and Step is live before its final marker; it is not automatic on every switch. A0 no-counter entry has its5 primary packets only. A2 may replace the missing successful counter's final result by its actual failed-counter permission, but never both successful and failed finishers. Record one `(cast,finisher)` token, consume permission on matching extra damage or10s expiry; a Linko cue grants that permission under its source marker/display conditions, not another attack or cooldown clear. Authoring a parry after the final event cannot retroactively recharge120 or move a missed packet. Target death/cancel/scene exit follow pending packet and paid charge rules.

Resonance→Echoes is stack replacement at Skill admission. Capture all consumed n/buffs/row choice for the active cast, retain new Support stacks for the next cast. Outside casting, states persist through field changes; no need for another character-wide form reset or hidden special resource. Normal3_3 and Branch5 each spawn one independent one-hit actor; field exit preserves those committed actors while cancelling later montage requests. Burst packets likewise are its14 separate requests, not autonomous off-field attacks after an unrequested event. Insight target marks/accepted Discord cap debuffs retain their own stated clocks after source field exit; self charges/bonus run30s real time. The distinct target9999 setting at A3 does not extend the self admission handle. Every continuing cycle carries all these states.

### Complete recipient and dependency specification

**Mixed.** Native DamageRange_Daffodill_* direct containers are uncapped full-coefficient AoE in the all-in-range dummy model for Normal/branch/plunge/riposte/QTE/Skill/Burst and actual targeted Phantom Step train. The Step requires a living locked target for admission (fallback lowest eligible if prior lock lost), then its bound ranges hit every eligible dummy; an untargeted non-damaging appearance does not fabricate one. Source actor target eligibility is evaluated per impact. Stable target indices order receipts; no division of coefficients by target count. One causal packet grants its listed channels once, not per recipient; self Support stacks, consumed entry charges and full-energy reward likewise once per cast/event. Target Insight count/lifetime, Break gauge/cap/transition and R6 remain per recipient and source.

Use [damage zones](../../shared/damage-and-stat-zones.md), [equipment](../../shared/equipment-builds.md), [energy/gauges](../../shared/energy-and-gauges.md) and [shared cycles/Break plus original base lookup](../../shared/cycles-and-break.md). Chaos+Psyche qualifies Nova, Chaos+Incantation Scorch; a real overlap emits the deduplicated Discord callback. The target profile supplies its actual Discord fraction for gauge loss, while Daffodil's separate cap effect uses its own10% source coefficient/stack limit; no multiplying that fraction by an invented universal value. A target with no eligible Break bar may take ordinary HP damage without a Break event. Breakable scenarios require original capacity, gauge/acceptance and recovery duration/rate, carried across all periods. Apply each actual Break transition once, living party contributions then Daffodil A5 repeats, no global Broken vulnerability.

Implementation state: Normal/suspended stage/progress/continuation deadline; Ultimate/Cycle integer/carry/overflow; Resonance pool and cast snapshot; self Eyes/Step handle and2 charges; per-target source Insight count/deadline and cap-reduction state; Skill/Burst CD/recovery/locks; explicit parry/riposte and ExtraParry permission generations; finite spawned actors/pending packet IDs; Broken target/R6 deadlines and contribution snapshots; source equipment windows. Expire ordinary buffs/marks before equal-time HP, settle direct HP/resource/gauge then new Break/R6/contributions/owner repeats, then next authored input. A cap-stat change can cause one Break but never repeat a dead target's transition or recursively trigger itself. These concrete rules preserve full dependencies without a native spatial or enemy-AI reconstruction.

### Complete implementation handoff

**Mixed; independent continuing-state check passed October19.** Implement the preceding seven complete specifications and their shared dependencies. Native certainty is source values plus explicitly checked inferred clock/lifecycle/selection rules; no new native-body search is required for implementation of this declared model after collection ends. Current simulator execution remains narrower; this handoff adds no simulator code or gameplay-observation claim.

**Legal A0 build:** Daffodil `character:1054`,80/80 BT6, four talents8, B2/B4 unlocked, no awakenings; **Youthful Fantasy/Black Tome `weapon:fork_BlackBook` T1 level80 BT6**, **Diabolos `set:Suit1`**. Declare set count 4 and supply console contributions through `add stats`; the numeric baseline below is preserved without modeling modules.

Static compiler: character644+weapon570=**baseATK1214**, combined AtkUp1.05/flat420→**2908.7 exact ATK**. Author AtkUp.75; the weapon supplies the remaining.30 once. The historical handoff's nearest-integer2909 reference below is a separate arithmetic example; runtime retains exact compiled ATK. HP21780, DEF873, CR.35, additional CD1.10, CE1/cap120, CI0. Static Chaos+.30 includes automatic set2pc.10 plus authored console bonus .20, not another live.30. The old compiler's BI0 omits [Black Tome](../../weapons/black-book/passive.md) unconditional **+60 BI**, which the selected live model applies once; it affects true Break HP, not ordinary damage or gauge shred. The source item name/frozen card versus current Youthful Fantasy label refer to the same native fork_BlackBook ID.

**All equipped behavior remains specified:** Diabolos4pc installs **.12 flat Chaos RES ignore**. When wearer is an actual outgoing/incoming participant in a qualified Nova/Scorch pair, replace it with**.24 for20 World seconds**, refresh one handle, then restore.12; never sum to.36 or activate from another unrelated party member's reaction. It is wearer/packet-scoped and follows off-field delivery, including own Chaos Break RES channel. Target resistance is `original-ignore-shred`; choose shared **standard_rational** for a negative result, explicitly not the alternative linear_scaled. The baseline positive.20-.12=.08 has F_res.92. This completes the set's source base-remove/power-add/base-restore layout.

Black Tome starts **two sealed chains**. Every actual successful party Support admission releases1, once per cast, persists field exit; no ordinary switch or direct fake packet releases a chain. At0 chains Arc input18 is available to active living wearer at a legal action boundary. Native GA_ForkTrigger_BlackBook has **DelayExecContainer.2, normal cooldown0, cancel cooldown30**, no montage in its reached CDO: selected startup/recovery.2s, successful spawn then one active summon20s; an interrupted startup applies30s cancellation cooldown. Only the explicit interrupt event cancels this startup; ordinary Jump/Skill recovery overrides cannot shorten its lock. Re-seal2 on successful use; Supports while the summon is active can charge the next use but cannot create overlapping summons. The active interval is half-open: spawn time is included and the exact20s endpoint is excluded. In the accepted source rotation, Arc admissions at16.1 and46.1 complete at16.3 and46.3, yielding summon intervals[16.3,36.3) and[46.3,66.3); no spawn occurs at input admission and no designation or Break callback occurs at the exact end. Weapon removal/source death cancels future summon admissions; already captured packets retain normal pending-hit handling.

Selected summon designation is one living target at spawn, then+5,+10,+15 (source5s cadence), prefer current living lock else lowest eligible index; the new designation replaces the previous one through the next selection and only while the half-open20s summon interval remains active. Current designated target death leaves no bonus until the next scheduled selection. Wearer Chaos damage to that target gains ordinary elemental+.20 atT1; this does not scale true Break HP. Each positive **Break HP contribution/repeat packet** on that designated target while the summon is alive produces one immediate wearer **2ATK Chaos gear packet**, with live ordinary crit/DEF/RES/eligible bonuses, only that target. Native `GE_Fork_BlackBook_damage` has categoryNULL/GANameNone/empty fallback arrays and zero charge/Cycle/Break; use the refinement coefficient2, not a Normal/Skill/Burst bonus or an autonomous5s damage train. Different real team contributions/A5 repeats can each trigger; these ordinary gear packets are not Break and cannot trigger themselves. No independent DoT coefficient/ticker is fabricated from the current review's loose “small DoT” wording: the reached source describes designation plus Break-triggered damage. These explicit summon/order/recipient/chain choices are inferred, conserving its two-Support funding and finite20s lifetime. Baseline below naturally leaves it sealed; no equipped handler is omitted.

**Party/scenario:** active Daffodil plus living **idle A0 Mint** using her already accepted level80 build [Mint handoff](../mint/action-damage.md#complete-implementation-handoff), no Mint attacks/casts. One immortal all-in-range dummy, DEF1080 at attacker80, original Chaos RES**.20**, explicitly no Break bar, no incoming attacks/parry/dodge, no resource preload, full HP. Mint cap100/CE1 receives the real.6 shared energy. Chaos+Anima are not an adjacent cycle pair, so all authored switches below are ordinary: no QTE/Support, reaction, hidden Resonance stacks or weapon unsealing. Mint's Anima/Skill-triggered gear sees no qualifying Anima damage/Skill, while its listeners remain enabled. This legal idle ally exists to exercise both mandatory Step charges, not to manufacture pair gauge or unresearched filler damage.

**Two continuing75s periods**, offsets0/75 through150, carry all state:

1. Six complete ordinary chains consecutively at0,6.00166464,12.00332928,18.00499392,24.00665856,30.00832320, finish36.00998784. Ordinary Skill37 recovers38.2579917, starts native16s CD at37.18229023. Pre-Burst energy125.511 funds Finale40; no parry preload or120 refund is assumed.
2. Finale40 applies self Eyes/Step handle through70 and target one-stack Insight through85 **at progress0**. All14 HP packets run42.8493748..44.657557, recovery45.0889783, lock ends45.087548. Wait to48, ordinary switch to idle Mint;49 switch back consumes first Step,5 primary packets through50.3767304, recovery50.6450272. No parry event, so no success finisher/recharge at A0.
3. Ordinary switch to Mint52, back53 consumes second Step, last primary54.3767304, recovery54.6450272. Charges now0, not reset by ordinary switch. Normal chain56..62.00166464; wait through75, retaining all clocks/ledgers/mark. Self bonus/entry handle expire70; target Insight still has**10s at75** and expires85 during the second period's pre-Burst Normal activity. The target mark alone does not grant the expired self+.10. Repeat Skill112/Burst115/Step124,128/Normal131 onward; next target mark expires160, not at the150 report boundary.

Each period has **165 positive HP packets**:133 Normal,8 ordinary Skill,14 Burst,10 Phantom Step; total**330**, no optional fake Support/Break/parry/Arc HP packets. Raw energy**151.271/period**. First pre-Burst125.511 caps120 with5 overflow/.511 carry; subsequent25.76 ends **26+.271**. Second ends **26+.542**, cumulative overflow36:302.542=240spent+36overflow+26.542. Mint receives90.7626/period: first90+.7626, second100+.5252 with81 overflow, no Mint spending. Cycle252.208/period stays100+.208 with152overflow, then100+.416 with404cumulative overflow; no qualified-switch spend. Both boundaries Resonance0, Step charges0/expired handle, self Eyes expired, weapon chains2 still sealed, no pending damage/CD, **one target Insight stack/10s remaining**. Do not reset that mark or the idle ally's shared energy at75.

Independent event/Fraction audit  passed hit counts, legality, live self buff, two charge spends per period, mark carry and both actors' energy. Portable direct expectation formula: `coef*2909*(1.3+.1*selfEyesActive+.8*isPhantomStep)*(1+.35*1.10)*.5*.92`. All same-cast HP packets sample the live self bonus; target Insight count is not an extra ordinary damage multiplier. **Expected total378548.44920582, DPS2523.6563280388 over150s**. This is selected-model expectation, not a seeded runtime/game observation. Scratch build/audit files are , `daffodil-build-report.json`, `daffodil-two-cycle-receipt.json`; full portable values/schedule are here.

Conditional acceptance checks are concrete: two actual Support casts give2 stacks and Echoes base1.199Break×3.5=**4.1965**, A1 replacement×6=**7.194**, with no old-plus-new increment; entry parry/A2 fallback produces one5.141 finisher and one120fill, retaining Burst CD; source InsightA3 cap2/9999 versus self30; true Break level80 source3603, Light Prism/A4 zone and A5 own-only repeat counts; native two-stack10% cap example70→63→56, third Discord reaches a singleBreak without a third cap stack; cap restoration and original capacity for HP; R6 state-linked Chaos shred/A6 DEF-ignore, correct shared negative-RES policy; branch9 requests/one actor, plunge/dodge/riposte cost/continuity; active Black Tome two-Support funding/startup/designation/Break callbacks and Diabolos replacement windows. CLI/browser implementations must preserve these rules/assumptions and diagnose any missing reached handler; no new research is required for these declared branches.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,6,3,7 (nineteen hits). The chain loops, retaining native effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`daffodil-normal-chain.json`](../../../../reviews/q10-game-action-binding/daffodil-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`daffodil.skill_animation_profile_v1`, `daffodil.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Daffodill_Skill1`), `AnimNotify_TriggerCooldown` is at 0.182s, gameplay events land at 0.197s & 0.332s (`Skill.1`), 0.670s (`Skill.2`), 0.917s (`Skill.8`), 1.000s (`Skill.9`), and 1.106s (`Skill.5`); cancel windows activate at 0.289s (`BreakSkill_Jump`), 1.210s (`BreakSkill_AnySkill`), and 1.713s (`BreakSkill_Move`) with total duration 4.083s. For Burst (`Daffodill_UltraSkill`), gameplay events land at 2.700s (`EndAbilitySkill1`), 2.849s (`UltraSkill.1`), 2.906s & 2.960s (`UltraSkill.3`), 3.050s (`UltraSkill.2`), 3.100s & 3.150s (`UltraSkill.6`), 3.776s–4.154s (7 hits of `UltraSkill.4`), and 4.658s (`UltraSkill.5`); cancel windows activate at 5.085s (`BreakSkill_Move`), 5.086s (`BreakSkill_Jump`), and 5.089s (`BreakSkill_AnySkill`) with total duration 9.083s.
- **Multi-target delivery (`daffodil.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive the full packet train for Skill (8 hits: 2x `GE_Player_Daffodill_Skill1_Damage` + 5x `GE_Player_Daffodill_Skill2_Damage` + 1x `GE_Player_Daffodill_Skill5_Damage`) and Burst (14 hits: 3x `GE_Player_Daffodill_UltraSkill1_Damage` + 3x `GE_Player_Daffodill_UltraSkill2_Damage` + 7x `GE_Player_Daffodill_UltraSkill4_Damage` + 1x `GE_Player_Daffodill_UltraSkill5_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Action packets and owner effects

- GA_Daffodill_Melee public primary packet is GE_Player_Daffodill_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.167/0.286/0.309; category A/CHAOS metadata stays source-bound.
- GA_Daffodill_Skill public primary packet is GE_Player_Daffodill_Skill1_Damage. Native coefficients at Skill levels 1/8/9 are 0.409/0.701/0.757; category E/CHAOS metadata stays source-bound. on_cast delivers an 8-hit packet train (2x Skill1 + 5x Skill2 + 1x Skill5). At A1, grants +100% damage bonus to GA_Daffodill_Skill across all 8 hits.
- GA_Daffodill_UltraSkill public primary packet is GE_Player_Daffodill_UltraSkill1_Damage. Native coefficients at Burst levels 1/8/9 are 0.493/0.845/0.913; category Q/CHAOS metadata stays source-bound. on_cast enters Insight mode (30s base, 9999s at A3) which grants +10% damage bonus, and delivers a 14-hit packet train (3x UltraSkill1 + 3x UltraSkill2 + 7x UltraSkill4 + 1x UltraSkill5).
- Explicit `GA_Daffodill_Melee` packet inputs now include Branch1–Branch7 and EntryAttack1/2/4. Frozen damage rows name `GA_Daffodill_Melee`; public attack tips bind Branch1–5 and EntryAttack1/2/4, while Branch6/7 appear only in the frozen damage rows. Under `daffodil.branch_entry_direct_packet_inputs_v1`, a caller-selected packet resolves once at the admitted Normal coordinate and impact time. This direct API binding neither advances nor resets the ordinary Normal chain and excludes natural dispatch, multiplicity, animation/timing and target selection. Its separate direct-packet energy policy is documented below. If a caller supplies an explicit target `break_profile`, the separate shared `combat.unbal_value_as_target_gauge_v1` adapter can consume the packet's native `UnbalValue`; this does not resolve Daffodil-specific Break eligibility or effects. Native values and live hit modifiers still flow through the shared packet path.
- Resonance 3 activates at awakening >= 3 and adds +1 level to GA_Daffodill_Melee, GA_Daffodill_Skill, and GA_Daffodill_UltraSkill without manual overrides.
- Awakening 6 (Bloom in the Lake) ignores 15% of the target DEF when dealing damage.
- Canonical arbitrary awakening selections: Effect1 Skill damage, Effect3 Insight duration and Effect6 DEF ignore use selected identities; talent resonance_3 remains count-based. Existing unsupported owner mechanics remain pending; selection does not complete the kit.

## Direct Branch/Entry packet energy

The exact direct packet inputs use `daffodil.branch_entry_direct_packet_energy_v1`, separate from `daffodil.ordinary_hit_energy_v1`. Frozen `DT_SkillDamageData.ChargeAdd` values are Branch1–7: 0.722, 0.594, 2.42, 0.702, 0.984, 0.594 and 0.594; EntryAttack1/2/4: 0.579, 0.47 and 0.94. After positive actual HP loss against target zero, the model grants each living source 1x and each other living configured recipient the frozen CHAOS team-charge relation coefficient (0.6), then applies that recipient's Charge Efficiency, exact fractional carry, capacity and overflow. A fatal hit qualifies. Zero HP loss, a missing/dead target or a cancelled packet grants nothing. Delivery-time recipient/source ownership and off-field delivery follow the tested shared policy. These eligibility, distribution and off-field rules are explicit tested assumptions for these exact direct inputs; frozen ChargeAdd does not establish native Branch/Entry energy dispatch. Natural dispatch, cadence, costs and target rules remain unresolved.

Receipt: [`direct-packet-energy-owners.json`](../../../../reviews/q10-energy-ledger/direct-packet-energy-owners.json).

## Zero-start continuing rotation

The 36-second scenario in [`daffodil-zero-start-continuation.json`](../../../../reviews/q10-energy-ledger/daffodil-zero-start-continuation.json) schedules 30 ordinary Normal inputs from 0.0s through 23.2s, Skill at 24s, Burst at 26s, and five more Normal inputs from 32.0s through 35.2s. With synthetic 1000 ATK, target DEF1080/RES0, fixed noncritical damage, and a 36-second denominator, it settles 155 hit receipts. The first 30 inputs are six level-eight five-input chains: `6 × 5463/2 = 16389`, using the [Normal-chain oracle](../../../../reviews/q10-game-action-binding/daffodil-normal-chain.json). Existing [Skill/Burst packet oracles](../../../../reviews/q10-game-action-binding/daffodil-skill-burst-packets.json) give `5145/2` and `15081/2`; Insight makes the continuing Normal chain `5463/2 × 11/10 = 60093/20`. Total damage is `590133/20` and exact horizon DPS is `196711/240`.

The scenario starts Daffodil, Mint, and Linko at zero energy with explicit 120-unit capacities. Its caller supplies a 120-energy Burst cost; this is a fixture input, not evidence that the native action admission cost or readiness timing is known. Under `daffodil.ordinary_hit_energy_v1`, at the 36-second horizon the source has 20 energy after the one 120-cost spend, with 5 overflow and 413/1000 carry. Each ally ends with 87 energy and 1239/5000 carry. Normal/Skill hit eligibility, the 3/5 CHAOS recipient relation, delivery timing, and chain continuity remain tested hypotheses; the sequence does not establish native animation, cancel, action-admission, or live-readiness behavior.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/daffodil-action-binding-model.json`](../../../../reviews/q10-game-action-binding/daffodil-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/daffodil_actions.rs), [`daffodil-normal-chain.json`](../../../../reviews/q10-game-action-binding/daffodil-normal-chain.json), [`daffodil-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/daffodil-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_daffodil.json`, `data/extracted/neverness_daffodill.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/daffodill.rs) (original `DT_SkillDamageData` export is cache-only), `1054AwakenEffect` (cache-only awakening source), [`daffodil/parameters.rs`](../../../../crates/content/src/characters/daffodil/parameters.rs) (consumed typed parameters; original `DT_DaffodillEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- AirAttack, EntryAttack3, CloseCall and natural Branch/Entry dispatch remain without executable bindings. Branch/Entry packet inputs are available only through the bounded direct API model above; their natural multiplicities, timings, target rules and energy eligibility remain unresolved.
- Native Daffodil Break eligibility and effects remain unresolved. The shared target Break gauge adapter runs only for an explicitly configured target `break_profile` under its separate hypothesis.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/daffodil). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Daffodil (`GA_Daffodill_*`):

| Action / damage row (`GE_Player_Daffodill_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | 0.167–0.627 / 0.286–1.075 / 0.309–1.161 | 22.000 total / 36.800 total / 4.400 total |
| Plunge `AirAttack_Damage` | X | 0.424 / 0.727 / 0.785 | 2.700 / 4.400 / 0.500 |
| Branch attacks `Branch1_Damage` .. `Branch7_Damage` | A | 0.096–0.389 / 0.165–0.667 / 0.178–0.720 | 0.594–2.420 / 0.998–4.065 / 0.122–0.496 per hit |
| Entry attacks `EntryAttack1_Damage` .. `EntryAttack3_Damage` | A | 0.553–0.681 / 0.948–1.167 / 1.024–1.260 | 0.470–0.579 / 0.782–0.962 / 0.097–0.119 per hit |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.040 / 1.782 / 1.925 | 2.600 / 4.300 / 2.500 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 5.600 / 0.000 / 2.500 |
| Skill `Skill1_Damage` .. `Skill9_Damage` | E | 0.337–0.911 / 0.578–1.561 / 0.624–1.686 | 0.685–1.853 / 1.134–3.068 / 0.135–0.365 per hit |
| Burst `UltraSkill1_Damage` .. `UltraSkill6_Damage` | Q | 0.253–2.183 / 0.434–3.741 / 0.468–4.041 | 0.000 / 0.000 / 0.079–0.682 per hit |

Critical Riposte uses GAName `GA_Daffodill_PerfectEvade`, while Support QTE uses `GA_Daffodill_QTE`. All damage rows deal CHAOS damage. Skill delivers an 8-hit train (`Skill1` x2, `Skill2` x5, `Skill5` x1), while Burst delivers a 14-hit train (`UltraSkill1` x3, `UltraSkill2` x3, `UltraSkill4` x7, `UltraSkill5` x1).

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Daffodill_Passive_1` binds `Buff_Daffodill054_Passive1`. `DT_ReactionExtendDataTable` row 1054 binds `ReactionAbilityID=GA_Daffodill_Passive_1`, providing Daffodil's Chaos reaction enhancement.
- **Passive 2 (Breakthrough 4):** `GA_Daffodill_Passive_2` binds `Buff_Daffodill054_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1054AwakenEffect#/rows/1054/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Changes the described per-Support Resonance increments to 100% damage and 250% Break efficiency; the source selector below replaces the ordinary increments rather than adding both sets.
- **A2:** If Phantom Step fails to Parry Attack the target, its final attack deals high damage, generates Break and fully charges Finale.
- **A3:** Insight stops being removed except on target death or combat exit; repeated applications stack up to two. The `1054AwakenEffect` Chinese description specifies combat exit, not character switching.
- **A4:** When a target enters Break under Insight, Daffodil's Break DMG increases by 15% per Insight stack; this is a magnitude bonus, distinct from A5's additional packet.
- **A5:** If a target enters Break under Insight, deals one additional instance of Break Damage per Insight stack.
- **A6:** Ignores 15% of target DEF when dealing damage (`TargetDefIgnore = 0.15`).
- **Resonance 3:** +1 level to Still Waters (Normal), Resonance (Skill), and Witness This Finale (Burst).
- **Resonance 6:** When target is in Break, reduces target Chaos resistance by 15%.

### Insight-count magnitude and native logic configuration, October 18

The initial `Cau_Daffodill_UnbalDamUp_Lv4:CustomCalculateBaseMagnitude` (`#/2`, statements 72..426) obtains InstigatorActor ASC and counts **HitActor** `Buff_Daffodill_InsightTarget` using `GetGameplayEffectCount(class, instigator ASC, true)`. Its successful expression is `UnbalDamUpLv4(coordinate 0) * max(returned count,0)`. The CDO binds `DT_DaffodillEffectFigure:Daffodill_UnbalDamageUpLv4`; the serialized ClampValue=(0,5,0) is not an explicit clamp in this function. Native getter interpretation/capture and application destination remain open. This is not the number of living targets or a team-wide mark count.

The expansion's `player_054_daffodill#/3/Properties` locates `DaffodillLogicObj`: `SkillCntBuff`, `SkillDamUpBuff`, `ExtraUnbalDamageGE`, `UltSkillTargetDebuff`, `AppearCntBuff`, `ExtraParryDamageGE`, and `ExtraParryDamageOKBuff` point to their named Daffodil classes. `ExtraAppearSectionArray` explicitly lists **AppearAttack, AppearAttackShift, AppearAttackParry**. This provides the entry/parry/Break producer references missing from a coefficient-only inventory. Its player export has a parser warning; no native transition body or final packet/Break admission follows from these configured references. Continue at the source-specific mark-count getter and the native DaffodillLogicObj callbacks rather than searching again for those classes.

### Passive description coverage, reviewed October 18

Saved description source: `guide_daffodil.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Blade Draw describes Discord reducing the target Break cap by another 10% for 30s, with a maximum reduction of 30% of the initial cap and restoration on combat exit. Cicada Shell describes Phantom Step damage +80%. The cap basis, qualifying Discord stacks, refresh/exit restoration and the Phantom Step damage-zone consumer remain material; neither passive carrier name nor a direct damage coefficient establishes them.

### Named Pas2 carrier is not the damage multiplier, October 18

The expansion's warning-free `Con_Daffodill_Pas2#/2` validates instigator ASC and returns presence of **Buff_Daffodill054_Passive2**. The selected `Buff_Daffodill054_Pas2DamUp#/1/Properties/Modifiers`, despite its name, supplies **DefUp** through row `Daffodill_Pas2DefUp`, not DamageUpGeneralBase through the saved **Daffodill_Pas2DamUp=0.8**. It also names `Daffodill_Pas2Dur` and SuperArmorCoefficient=0.5. The two former curve names are absent from the frozen figure table inspected here. Thus this carrier does not close the described Phantom Step +80% damage consumer or prove an extra defensive amount/duration. Keep the located marker predicate and separate offensive row without inventing a live binding from the asset name.

### Resonance increments and Insight duration selector recovered, October 18

Initial `Cau_Daffodill_SkillCount_DamUp` and `_UnbalAccEff`, both warning-free `#/2`, validate instigator ASC and select a scalable based on **Buff_Daffodill_Level1**. Damage returns `Daffodill_SkilllDamageUp=0.5` without A1 or `...Lv1=1.0` with it; Break efficiency returns `Daffodill_SkillUnbalAccrueEfficiency=1.25` or `...Lv1=2.5`. Neither function counts or mutates Resonance stacks. `Buff_Daffodill_SkillDamUp#/1/Properties/CustomApplicationModifierInfos` binds them, multiplier 1, to additive **DamageUpGeneralBase** and **UnbalAccrueEfficiencyBase**, both requiring source tag **State.Damage.Skill**. The carrier is Infinite, AggregateByTarget, limit **2**; its serialized duration magnitude 500 is not an expiry. Native stack aggregation, Support delivery and enhanced-Skill consumption still need their consumers, but A1's replace-versus-add choice and zones are located.

`Buff_Daffodill_InsightTarget#/1` is HasDuration, AggregateBySource, limit **2**, with custom duration `Cau_Daffodill_InsightTargetDur`. That warning-free helper (`#/2`) selects **9999** with owner `Buff_Daffodill_Level3`, otherwise **45**, from `Daffodill_UltraInsightTargetDur3`/`...Dur` at coordinate 0. The copied CDO literals 3/10 and Mint target class are unused. A3 is represented by a long finite configured duration, not an Infinite policy. Native death/combat-exit removal and actual clock/refresh remain open; do not equate the long value to a proven permanent effect.

### Discord cap-reduction input and MMO override recovered, October 18

Initial `GE_Reaction_AnHunZhou_1054#/2/Properties/Modifiers` binds additive **UnbalMaxUpTemp** to `Cau_AnHunZhou_Reduce`, multiplier 1. Its warning-free `#/2` function casts InstigatorActor to player: on **IsMMO_Server** it returns native **GetAnHunZhouUnbalReduceCoefficientByMMO**; the other mode and failed-player-cast branches read `Reaction_AnHunZhou1_UnbalMaxReduce_1054=-0.1` at coordinate 0. This is not unconditional use of that table row in every mode.

The carrier is HasDuration through **Reaction_AnHunZhou1_UnbalMaxReduce_BuffTime_1054=30**, AggregateByTarget, limit **2**, and requests its own removal on **LEAVE_BATTLE**. These locate signed stat, duration and cleanup declaration. They do not establish whether the described overall 30% initial-cap reduction includes a base reaction contribution or how the native MMO getter/stack aggregation/cap restoration operates; do not change the carrier limit to three merely to match the prose.

### Insight reapplication removes one source stack without A3, October 18

Initial warning-free **GA_Daffodill_UltraSkill:OnReceiveEvent** (`#/5`) calls its parent and requests **ApplyInsightForTarget** only for **Event.Montage.Player.UltraSkill.Daffodill.Appear**. The reached body (`#/3`, 772..1819; ) requires avatar player and **IsAuthorityOrAutonomousHybridMode**, requests Buff_Daffodill_InsightEyes and Buff_Daffodill_AppearOK (the latter with nStackCount=100), then constructs an InsightTarget container using **DamageRange_Daffodill_UltraSkill1** and the supplied event data. The 100 is that separate self-carrier request, not a target Insight cap or damage count.

With **Buff_Daffodill_Level3**, it directly requests ApplyEffectContainerSpec. Without A3, it first iterates actors from the constructed target data and, for each valid ability character whose **GetGameplayEffectCount(InsightTarget, caster ASC,true) >= 1**, requests removal of **one stack** with the **caster ASC** filter; afterward the common flow requests the new application. This is replacement-before-application scoped to that source, rather than clearing every source's Insight, adding two stacks unconditionally or consuming all marks on Burst. Native getter semantics, application success, target-data duplication, clocks and death/combat-exit removal remain open, but the A3 skip-removal branch and ordinary one-stack removal request are now located.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Daffodill_Melee#/7; Melee.3_3 | PlayerDaffodillMelee3 | (0.0, 0.0, 1.0, 2.0, 1, true) | GE_Player_Daffodill_Melee3_3_Damage |
| GA_Daffodill_Melee#/8; Branch.5 | PlayerDaffodillMelee3 | (0.0, 0.0, 1.0, 2.0, 1, true) | GE_Player_Daffodill_Branch5_Damage |

Melee.3_3 and Branch.5 reuse PlayerDaffodillMelee3 but select different Melee3_3_Damage and Branch5_Damage containers. A shared range/configuration is not evidence that both actions use the same coefficient.

### Insight-enabled entry admission and section selector recovered, October 18

Warning-free **DaffodillAppearCondition:K2_CustomPossessedCondition#/2** explicitly rejects invalid OwningCharacter or Ability; otherwise it returns **OwningCharacter.Has(Buff_Daffodill_AppearOK) OR IsHybridMode_Server(OwningCharacter)**. It is actually bound by **GA_Daffodill_Appear#/3/GAComponents/1**. The distinct **DaffodillSelectStartSection:K2_CustomCondition#/2** requires a player owner with that marker, without this explicit hybrid-server bypass. Its **K2_GetStartSectionName#/3** returns **AppearAttackShift** with valid **Ability.GetAbilityTarget()**, **Appear** otherwise. It is selected at **AdditionalStartSections/2** in that same ability. Component admission and section selection therefore have different predicates; the server branch is not evidence of an unconditional extra entry attack.

The marker's warning-free **#/1** declares HasDuration through **Daffodill_UltraInsightDur=30**, target aggregation and cap **2**, separately from the earlier Burst callback's explicit nStackCount=100 grant request. The caller does not establish 100 settled charges. The ability also retains Falling and two BeParryDuration selectors, including a separate marker-requiring **AppearAttackParry** option. Native selector arbitration, marker consumption, retargeting, section/notify timing and parry admission remain open; the marker alone does not select every attack or remove those other branches. Sources: `additional-form-and-appear-config.txt`, `additional-form-and-appear-callers.txt` and `additional-appear-marker-callers.txt`.

### Linko-linked extra-parry marker request recovered, October 18

The repaired **Buff_Radio072_Daffodill#/1 → GameplayCue.Display.Radio.Daffodill → GC_Linko_Daffodill** join supplies an additional marker caller outside Daffodil's folder. OnWhile **#/2 → graph#/1:15** attempts incoming **EffectCauser as player**, otherwise **EffectCauser.GetOwner as player**, then checks the stored player's **GetCurrentDisplayCharacterID==1054** and **Buff_Daffodill_Level2**. It requests **ExtraParryDamOK on that stored player attributed to itself**, with cue SourceObject/default BufferData. The fallback failure path still pops into the stored-player check rather than explicitly clearing the cache; native cue lifetime/reuse and invalid-context handling remain relevant. There is no explicit damage, positive-hit or role gate in this cue.

The trigger carrier's HasDuration input is 0.3. **ExtraParryDamOK#/1** has HasDuration 10, target cap1, and requests self-removal on **SKILL_AFTER_DAMAGE** under **Con_Daffodill_IsExtraParryDam**. This locates an additional A2 permission request and consumption declaration; it does not prove that a Linko trigger always admits the extra parry hit or settles Insight/Break. Exact sources: , `linko-team-mark-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`. The cited cue and both carriers are warning-free.

## Unknown execution policies

Historical native-certainty limits only; selected implementation decisions for these questions are complete above and do not reopen collection.

- **Real timing/admission:** Montage notify coordinates for Skill (`Daffodill_Skill1`) and Burst (`Daffodill_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Break interaction mechanics:** Native Break damage triggering, UnbalValue accumulation and Insight stack consumption rules remain open beyond the scenario `break_profile` adapter.
- **Spatial geometry & targeting:** Skill AoE and multi-target Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The frozen IDs/coefficients for Branch1–Branch7 and EntryAttack1/2/4 now have a bounded one-packet direct API model, including `daffodil.branch_entry_direct_packet_energy_v1`. Its exact ChargeAdd amounts and settlement path are documented in the known card; positive-hit eligibility, recipients and off-field ownership are tested assumptions, not native dispatch evidence. Natural Branch/Entry/Phantom Step packet multiplicities, timing/cancel rules, target caps/selection, costs, cadence and energy eligibility remain unknown. Bind AirAttack, EntryAttack3 and CloseCall if supported by a precise mapping. Resolve native Daffodil Break eligibility/effects; the shared gauge adapter already consumes native `UnbalValue` only when a caller supplies an explicit target `break_profile` and accepts its separate hypothesis.
- The bounded zero-start continuation now exercises Normal→Skill→Burst→Normal over 36 seconds; see the [known card](#zero-start-continuing-rotation) and [receipt](../../../../reviews/q10-energy-ledger/daffodil-zero-start-continuation.json). The known owner table binds Burst base energy cost 120 and nominal Skill/Burst CD curves 16/20. Additional action costs, spending/admission/readiness clocks, animation/cancel timing and ordinary-hit energy eligibility/recipients remain unresolved beyond the explicit tested scenario hypotheses; missing other cost rows do not mean free actions.

## Prior findings

- [`reviews/q10-game-action-binding/daffodil-action-binding-model.json`](../../../../reviews/q10-game-action-binding/daffodil-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/daffodil_actions.rs), [`daffodil-normal-chain.json`](../../../../reviews/q10-game-action-binding/daffodil-normal-chain.json), [`daffodil-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/daffodil-skill-burst-packets.json).

## Research history

- Daffodil8/8 selected handoff complete. Next campaign owner: **Edgar, packets block**, starting with existing dossier/exports.
- Methods already exhausted: original/supplement GA/notify/actor/condition/cue reads, prior direct-packet receipts and public/frozen comparisons; shared Break base was already collected in UnbaldamagePara and is now bound explicitly. Do not restart native parry/Insight/Break bodies during collection.

## Current shared Break consumer

The [live shared Break consumer](../../shared/content-break-model.md#live-break-hp-contributions) binds Light Prism +.20, personal A6 DEF ignore +.15, target-owned A4 Insight bonus, A5 own contribution repeats and Broken-linked R6 Chaos resistance through the owner adapters above. Independent runtime checks separate the Break bonus, DEF and target RES channels from ordinary ATK/crit/damage bonuses, preserve source/recipient captures and follow recovery/extension/death. The ordinary ExtraUnbalance fallback coefficient is not an additional packet on top of these true Break contributions. Native consumer proof and the remaining kit branches retain the limits stated in their owning sections.

## Executable source packet identities

Normal5 now uses two Melee5 and four Melee6 requests. Ordinary Skill preserves distinct Skill2/6/7/8/9 rows; Finale preserves Ultra1, two Ultra3, Ultra2, two Ultra6, seven Ultra4 and Ultra5. Phantom Step uses EntryAttack1/2/2_1/3/4 and A2-only EntryAttack3_1, followed conditionally by AppearSuccess; qualified Support preserves its QTE row. Only reached missing rows were added to the typed Rust damage table from the frozen cached source. Ultra6 is not an alias of Ultra2: at talent8 the Finale coefficient is13.712, producing7541.6 noncritical HP at1000ATK, DEF.5 and self Eyes+.10. Independent Rust packet identities and level1/8/9 sums retain the manual adapter's timing boundary; the owner clock and selected Entry/Parry path now execute in native Rust. The existing Python verifier retains source-binding, CLI replay and complete WASM parity, with duplicate domain assertions owned by Rust.

## Executable ordinary montage clock

`daffodil.ordinary_clock_v1` binds Normal, ordinary/Echoes Skill and Finale under the selected source-coordinate timing model. Coordinates round once to nearest microsecond; a complete connected chain takes6001664us. Distinct causal packet groups expand every request across living dummies, grant native Charge once and add integrated owner-only lost progress once on a positive HP connection. Zero damage adds no pause or Charge. Normal continuity starts from actual recovery; self Eyes and target Insight apply at Finale admission, well before its2849375us first HP request. Skill's cooldown starts at182290us or Echoes110328us; interrupted pre-marker Skill takes the separate3s cancel cooldown. Replacement, field exit and defeat remove unreached montage events without resetting continuing ledgers. The independent Rust tests cover two continuing chains, three-dummy invariance, exact source contacts, late Finale, truncated-horizon failure and notify-owned Skill cooldown. Selected jump/skill cancels, suspended Dodge, secondary actions, Phantom Step and qualified Support are implemented in their owner routes; the source-coordinate timing remains a modeled implementation, not gameplay-timing measurement.

## Executable Phantom Step and qualified Support parry

`daffodil.phantom_step_v1` requires the ordinary clock, cancel and secondary-action models. Finale commit resets two finite Step charges for30s. A distinct-actor re-entry with a live charge selects the first living dummy and captures it for the whole train; it emits EntryAttack1, EntryAttack2, EntryAttack2_1, EntryAttack3, EntryAttack4, plus EntryAttack3_1 at A2. A live parry before the shifted final marker admits one captured-target AppearSuccess; otherwise the marker admits that same candidate only for A2 fallback. A positive finisher fills owner energy to120 once. Charges are visible in the typed `daffodil.phantom_step_charges` resource and expire on a real-time event. Dead/expired targets, canceled routes, late parries and ordinary un-parried A0 Step do not grant the finisher reward.

When the secondary selector admits a qualified QTE on Daffodil re-entry, QTE takes priority over a simultaneous ordinary Step. With a live Step charge, only an actually eligible pre-marker parry consumes one charge and extends recovery through the finisher marker; the no-parry path retains the ordinary Support recovery plus its connected QTE pause. Same-time ordinary input is deferred until the marker and hit settle. Rust tests independently check exact Step damage with Finale Eyes and the +.80 passive, lowest-living-target selection, captured-target death, A2 fallback, one-time energy fill, unused-charge expiry, QTE parry success/failure/cancel and the exact-ready finisher boundary. The ten-second Linko ExtraParry permission remains outside this runtime model because the selected consumer is the same cast-scoped finisher and has no distinct damage output.

The same reached packet groups now bind exact Cycle under `combat.packet_cycle_accrual_v1`: Normal chain33.196, ordinary Skill10.098, Echoes10.100 and Finale0. Two chains yield66.392Cycle and39.804Charge before caps; Cycle efficiency2 instead yields100 retained/32 overflow/.784 carry without changing Charge. Independent Rust checks cover an initially dead primary dummy, first living AoE delivery, positive-HP eligibility and separate carry/overflow. The legal native build retains these ledgers through aggregation and seeded replay. Only reached rows retain HeterochromeAdd in the typed table.

## Executable Jump, Skill cancels and interruption

`daffodil.action_cancels_v1` requires the ordinary clock and binds every Normal-stage Jump/Skill gate plus the listed ordinary/Echoes Skill and Finale Jump gates. Admission uses only already caused pause; an early ordinary input rejects without becoming an interruption. Accepted Jump cancels unreached requests and clears the Normal pointer. Explicit forced interruption cancels the active montage, releases recovery, applies3s cancel cooldown only before Skill's reached cooldown marker and otherwise preserves its original16s deadline; it never refunds captured stacks or resets Insight. A replacement montage inherits the still-live remainder of the owner pause: interruption at1110000us followed by Normal1120000us retains33089us of the connected Skill-final pause, putting that new hit at1338397us. Rust independently checks every Normal gate, pending-packet counts, pre/post-notify cooldown and completed-action no-op; native legal build cancellation retains aggregation and seeded replay. Paid movement, suspended Normal and hold/secondary/entry routes remain pending.

## Executable full-kit acceptance

The selected owner models also bind held Mirror on Water, paid Displaced Edge with suspended Normal progress, authored plunge, one Critical-Dodge Riposte, qualified Support and charge-funded Phantom Step. An admitted Step captures its recipients, Normal talent and one finisher token; live parry or the A2 fallback can authorize that token once. Qualified QTE retains its Support talent and ordinary recovery until an eligible live parry spends one charge; its AppearSuccess child separately captures Normal talent and settles before an input at its exact extended recovery. B4 adds .80 in the general damage zone only to the captured Step/finisher, alongside live Eyes .10. A3 target Insight, A4/A5 own Break contributions, Broken-linked R6, and the separate two-stack Discord cap effect retain their source/target lifetimes. The Linko A2 permission cue cannot add a second token or bypass a charge/window; its redundant permission does not fabricate a new attack producer.

Rust checks cover independent packet totals/times, source categories, one energy fill across AoE recipients, dead captured targets, early/late/cancelled parry, charge expiry, paid suspension without replay, and the separate Finale switch lock. Shared Discord uses an explicit target fraction, fresh Scorch/Nova application pairs and original-capacity gauge loss; Scorch's ticker phase survives qualified refreshes. Failed overlap admission preserves Cycle, statuses, queued events and sequence. Black Tome's real Support funding, startup/cancellation, designation and live Break callbacks and Diabolos replacement windows have owner integration checks.

The legal A0 native build executes both75s periods above with **330 packets**, exact compiled ATK **29087/10**, carried Insight and both actors' continuing ledgers. The independently reconciled raw critical expectation is **189254705088513/500000000 = 378509.410177026** over150s (**2523.39606784684 DPS**); this differs from the historical2909-ATK arithmetic example solely by its explicit ATK rounding. Native acceptance checks each packet's noncritical/critical zones, energy/Cycle carry and overflow, HP-ledger reconciliation, aggregation and seeded replay. Native source hold/Jump/parry and suspended-Normal inputs now reach these owner routes; explicit forced interruption and final browser acceptance remain in their queue rows. These checks certify the selected simulator model, not measured native animation or observed game damage.


Native source `swap[parry=true]` now binds that explicit success to an eligible
admitted entry. A live charge and a dispatchable Step/qualified Support are
required before switching; same-character, no-charge and unavailable entries
reject without spending or switching. The callback follows its actual canonical
command, before the next source input, rather than being prequeued at the same
timestamp ahead of admission. Source ranges and event indices retain that parent.
Two continuing30s periods preserve two120-energy Bursts, four charged entries,
eight finisher packets on two dummies and a final empty charge pool. Qualified
Support consumes one charge and retains1.667260s real recovery, including when
only the aimed second dummy survives. Serialized native replay matches, with no
incoming-damage or Critical Dodge success inferred. Fixed scenario parries use
the typed `At` request variant; these are checked scenario assumptions, not game
observations of automatic counters.

### Native source combo and Displaced Edge windows

The [continuing input script](../../../../examples/daffodil-input-windows.nte)
executes two held-combo replacements, two paid suspended Normals and two
paid suspensions canceled by Jump. Ordinary source projection includes the
existing held/suspended model even when no held input appears. No new timing
constants or synthetic attacks enter the source compiler.

`window=hold` shares the native Normal Jump predicate; stage2 yields at1164546us
in the checked connected opening, and held replacement removes only its
unreached Melee2_1. Front Dodge at that same gate preserves the parent montage:
Melee2_1 arrives at2363006us and Normal3 can begin at2732900us. Jump82544us
after the Dodge removes the pending parent packet and its remaining suspension;
the following Normal1 connects at1432398us. The shared paid stamina account
persists across both periods. Explicit gaps beyond2s use normal native chain
expiry rather than resetting resources, buffs or pending effects.

`simulation/tests/source_daffodil_controls.rs` checks those independent native
numbers, AoE packet counts/lineage, unchanged observation results, two
notify-started Skill cooldowns, saved source roundtrips and complete native
replay. A default `window=dash` is the Front gate; a following Back Dodge must
still satisfy its own later admission window.

## Relative combat timer binding

`Daffodill_UltraSkill` Common.EndAbilitySkill at 5.099006 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
