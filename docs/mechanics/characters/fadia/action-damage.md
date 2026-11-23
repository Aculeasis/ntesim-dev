# Fadia action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1039`.

**Responsibility:** Fadia outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected8/8 implementation handoff below, with source facts and checked inferences distinguished. The executable `fadia.full_kit_v1` model binds the main Skill/Burst/Lilith, healing/mark, membership and reversible Nova-stat paths below, alongside the bounded incoming-sharing prerequisite. Paid secondary actions and source-coordinate cancel gates are bound under the same declared model. Historical executable slices are narrower; research completeness does not certify current simulator coverage.

**Accuracy notes:** historical [execution gaps](#unknown-execution-policies) retain provenance; selected sections settle all implementation choices for this campaign.

Native build projection binds `fadia.full_kit_v1` before initialization,
including idle party membership and scripts with only Burst. Skill retains
its owner action binding. Readiness consumes the same native ability/packet/
mode admission checks; source inputs do not select the historical five-second
partial Burst permission.

## Selected full packet specification

**Mixed, October19.** This and subsequent selected sections supersede the historical partial executable slice. Source: `DT_SkillDamageData`, `DT_FadiaEffectFigure`, original `GA_Fadia_*`/montages and supplement `Fadia_UltraSkill_Melee1..5`/damage CDOs, with permanent owner inputs (optional `fadia.json`). Public [native-description mirror](https://www.neverness.app/characters/fadia) corroborates the five follow-up stages; native/table and public investigations have already been used for the missing execution consumers, so remaining decisions are explicit checked inference, not another native-body search.

All rows preserve the full thirteen-level source arrays; samples below are **levels1 /8 /9**, indexed by effective owning talent minus one. `Melee`, plunge/riposte/Support use live **ATK**; Skill/Burst/Lilith use live **MaxHP**, never current HP. Resources are raw energy/Cycle/Break **per positive connected packet**, once regardless of number of recipients.

| Suffix after `GE_Player_Fadia_` | Count/category | Coefficient1 /8 /9 | Energy / Cycle / Break |
| --- | --- | --- | --- |
| Melee1_Damage |1 /A |.433 /.742 /.801 |2.707 /4.511 /.536 |
| Melee2_Damage |1 /A |.297 /.509 /.550 |1.859 /3.098 /.368 |
| Melee3_Damage |1 /A |.360 /.617 /.666 |2.247 /3.746 /.445 |
| Melee4_Damage |1 /A |.771 /1.321 /1.427 |4.819 /8.031 /.955 |
| Melee5_Damage |1 /A |.283 /.485 /.524 |1.768 /2.947 /.350 |
| Melee6_Damage |1 /A |.332 /.569 /.615 |2.075 /3.459 /.411 |
| Melee7_Damage |1 /A |.273 /.468 /.505 |1.703 /2.839 /.338 |
| Melee8_Damage |1 /A |.803 /1.376 /1.486 |5.022 /8.370 /.995 |
| AirAttack_Damage |1 landing /X |.600 /1.028 /1.111 |3 /5 /.6 |
| PerfectEvadeAttack_Damage |1 /Z |1.780 /3.051 /3.295 |4.5 /7.4 /2.5 |
| QTE1_Damage |1 /H |2 /3.428 /3.702 |7.6 /0 /2.5 |
| Skill1_Damage; Skill2_Damage |1 each /E |.025 /.043 /.046 MaxHP |1.742 /2.878 /.341 each |
| Skill3_Damage |1 /E |.016 /.027 /.030 MaxHP |1.111 /1.835 /.217 |
| UltraSkill1_Damage |1 /Q |.039 /.067 /.072 MaxHP |0 /0 /.643 |
| UltraSkill2_Damage |4 /Q |.012 /.021 /.022 MaxHP |0 /0 /.203 each |
| UltraSkill3_Damage |1 /Q |.063 /.108 /.117 MaxHP |0 /0 /1.045 |
| UltraSkillMelee1_Damage |1 in follow-up1 /Q |.020 /.034 /.037 MaxHP |1.085 /1.803 /.218 |
| UltraSkillMelee2_Damage |1 in follow-up2 /Q |.032 /.055 /.059 MaxHP |1.753 /2.913 /.353 |
| UltraSkillMelee3_Damage |1 in follow-up2 /Q |.036 /.062 /.067 MaxHP |1.920 /3.190 /.386 |
| UltraSkillMelee4_Damage |2 in follow-up3 /Q |.009 /.015 /.017 MaxHP |.501 /.832 /.101 each |
| UltraSkillMelee5_Damage |1 in follow-up3 /Q |.015 /.026 /.028 MaxHP |.837 /1.390 /.168 |
| UltraSkillMelee6_Damage |5 in follow-up4 /Q |.023 /.039 /.043 MaxHP |1.219 /2.026 /.245 each |
| UltraSkillMelee7_Damage |1 in follow-up4 /Q |.056 /.096 /.104 MaxHP |3.006 /4.994 /.604 |
| UltraSkillMelee8_Damage |1 in follow-up5 /Q |.059 /.101 /.109 MaxHP |3.200 /5.317 /.643 |
| ZhouYin_Damage |1 per accepted mark transfer /E |special fixed HP settlement below, source HP coefficient0 |.005 /.008 /.001 |

**Element correction:** Normal/plunge/riposte/QTE/Skill use `DAMAGE_TYPE_PSYCHE`. All initial Burst and Lilith rows explicitly use **`DAMAGE_TYPE_PSYCHICALLY` = Mental**, distinct from Psyche; historical blanket “all PSYCHE” prose is incorrect. Mental bypasses DEF, uses Mental RES and eligible ordinary general/Q/Mental bonuses, and these character packets **can crit** (A5 would otherwise have no purpose). They are not Nova reaction damage and do not inherit Nova's no-crit/no-ordinary-bonus rules. ATK scaling never substitutes for HP scaling. Source table fields and public descriptions independently agree on this distinction.

Normal groups [1],[2,3],[4],[5,6],[7,8] sum **6.087ATK /22.2energy /37.001Cycle /4.398Break** at8. Skill's source event order is **Skill3 then Skill1 then Skill2** (despite numeric row names), giving3 hits **.113MaxHP /4.595energy /7.591Cycle /.899Break**. Initial Burst6 hits sum **.259MaxHP /2.500Break**. Five Lilith inputs are **13 packets**, summed **.599MaxHP /18.898energy /31.401Cycle /3.799Break**. Preserve distinct hits for live HP, crit, target death and resources. Public full multiplier .020+.032+.036+2*.009+.015+5*.023+.056+.059=.451 at1 checks the row multiplicities; it is not five single packets. Plunge coefficient multiplies by `1+h` for scenario fall-height bonus h in[0,1]; include authored landing time, no invented metre curve.

**Mark settlement inference:** on Fadia's actual applied HP loss `d>0`, each live own mark requests fixed reflected HP loss `min(r*d,remaining_budget,target_current_HP)`, r3 or A2 r6, with resolved budget **.08 or A2 .25 × Fadia MaxHP snapshotted at mark application**. The cap is enemy damage **after amplification**, not an input budget amplified again. The specialized `FadiaTransferDamageExecution` receives this amount; it is not an ordinary zero-HP-coefficient hit, cannot crit and does not apply DEF/RES/ordinary bonus a second time. Retain Psyche attribution/E-category/causal listeners and the source .005/.008/.001 resource/gauge awards on each positive transfer. An A2 removal settlement is the unspent budget once, not multiplied by6. This choice follows the description's “damage taken cap” and the native specialized executor, and avoids silently disabling a non-ATK damage dependency.

Check at captured MaxHP10000: A0budget800, damage1000 -> enemy800, cap exhausted; A2budget2500, damage200 ->enemy1200 then expiry settlement1300, total2500 (not7800 more). A transfer has no player HP-damage child, cannot recursively redirect or feed its own mark. Each A6 mark owns an independent budget. Exact rounding is once at final HP settlement; report capped/lost amount separately. Remaining survival, mark lifecycle, HP extraction and form rules are in the following selected blocks.

## Selected full timing specification

**Mixed:** use native seconds at rate1 with explicit hitpause loss. Normal rows1..6 bind piecewise-linear rate curves; interpret their Time as seconds since pause onset, integrate rate over configured duration and add lost action time `duration-integral(rate dt)`. Constant endpoints apply. This is a concrete consumer inference using the stored curves, not measured frames. Preserve original curve objects for replay; missing curve uses native constant rate, missing duration contributes0. Losses for Normal rows1..8 are **.1283625, .100, .099, .133650, .0998666666667, .080, .099, .198s**. Delayed starts on rows7/8 are.09/.06; add their loss only after those onset events, not before the damage. Curves1/2 respectively have knots(0,0),(.06,0),(.12,.01),(.16,1) and(0,0),(.06,0),(.10,0),(.13,1); Curve3 stays.01 through.11 then1at.16; Curve4 stays.01 through.13 then1at.14. This independently yields row1 **.1283625**, not a presumed constant.1287. Once per packet, not per target. Buff/cooldown/resource clocks keep real time.

| Action | Damage montage coordinates (mapped rows) | Next ability / jump / move, before caused pause |
| --- | --- | --- |
| Normal1 |.31666666 row1 |1.0844268 /.36865452 /1.0844268 |
| Normal2 |.350 row2,.750 row3 |1.6722474 /.38001114 /1.6722474 |
| Normal3 |.5833333 row4 |1.4528977 /.63154423 /1.4528977 |
| Normal4 |.26666668 row5,.56666666 row6 |1.5331101 /.31467816 /1.5331101 |
| Normal5 |.3138703 row7,.91613543 row8 |1.6163554 /.9660202 /2.0322943 |
| Entry Normal1 |.32541144 |1.0999999 /.3737318 /1.0999999 |
| Approach Normal1 |.32620162 |1.0999999 /.37503928 /1.0999999 |
| Plunge after landing |.032851078 |.19592033 /.11524462 /.750726 |
| Riposte attack |.20070943 |.874201 /0 /.874201 |
| QTE |.68598294 |1.4766717 all; switch lock1.4337974 |
| Skill |.21566328 row3,.72862625 row1,.9063684 row2 |1.246059 /.97746533 /1.7743812 |
| Initial Burst |2.7073529 row1;3.1014013,3.2262611,3.3528275,3.4838865 row2;3.8934803 row3 |4.4159207 /4.4159207 /4.4504356; switch lock4.4001317 |
| Lilith1 |.16627146 row1 |1.0824109 /.2168765 /1.0824109 |
| Lilith1 approach |.16836093 row1 |1.1183854 /.22478104 /1.1183854 |
| Lilith2 |.09925115 row2,.44996005 row3 |.6162512 /.50033474 /.7667357 |
| Lilith3 |.1993722,.2832472 row4;.46692932 row5 |1.3160992 /.5164901 /1.3160992 |
| Lilith4 |.3671867,.46793976,.56608295,.6657423,.76678205 row6;.89864427 row7 |1.3424771 /1.0335917 /1.5475974 |
| Lilith5 |.3296403 row8 |.675917 /.34720343 /1.5159018 |

Other per-packet losses: plunge.099 at delayed.09 onset, riposte.297, QTE.250, Skill3.0495/Skill1zero(explicit duration omitted)/Skill2.099 at delayed.04 onset; initial Burst rows duration omitted =>zero extra. Lilith1/2/3 each.0198, Lilith4zero duration, Lilith5.099, Lilith6.0198 each, Lilith7.147 delayed.06, Lilith8.196 delayed.06. Their full stage recoveries are **1.1022109,.6558512,1.4150992,1.5884771,.871917**. Generic no-valid-form early cancel gates are .34252572 / .5731249 / .7653274 /1.0880553 for first four stages; active valid form cannot borrow those gates. Earlier Skill-only cancel gates for Normal1..4 are.60178936,.98785704,.7527374,.99777794; Lilith1/3 are.34980404/.77378446. They forfeit unissued hits and end form if another skill is actually admitted.

Skill CD **16s** starts native notify.19837612, after only earlier caused pauses (first damage at.215). Before that marker interruption uses cancel curve **3s from interruption**; after marker retain original16s deadline. Burst **20s CD** begins accepted commitment; HP-threshold energy fill does not clear it. Full Skill recovers**1.394559s**, with real mark event at **.95792557** (source .90842557 plus .0495). Row2 damage lands.9558684, but its .04-delayed pause starts **.9958684**, after the mark request. Do not shift an already-dispatched mark retroactively by a future pause. Full Normal chain and each subsequent input use the same source-coordinate scheduler; independent audit below carries exact rational losses rather than rounding every intermediate action.

**Form clock:** source `Cau_Fadia_ChangeDuration` returns9.42 in nonhybrid and5 in hybrid/standalone. Select baseline **nonhybrid9.42 World seconds from Burst commitment**. The current shared cinematic policy freezes that permission until release, so the old calculation subtracting the entire4.4159207s animation no longer describes the executable continuation window. Hybrid's explicit initial-animation suspension composes with the shared pause; overlapping pauses are counted once. These are selected timer policies rather than a measured native continuation window. A follow-up admitted strictly before expiry captures its form eligibility and may finish afterward; new inputs at expiry cannot. Fifth section admission snapshots A5/high-Q eligibility then removes further follow-up permission; finish its already admitted packets before cleanup of that cast context. This resolves the native early `UltraMelee5` grant-removal request without deleting its fifth hit or letting expiry grant a sixth input.

Ordinary switch waits next-ability recovery and any switch lock, removes form and cancels unissued on-field notifies. Dodge front/back generic recoveries1.0065707/1.0070164, Skill.333754/.494283, jump.082544/0, back move.801503; no damage from movement alone. Critical Dodge explicitly arms one next Basic riposte, consumed on that action/other action/field exit. Parry route's .02 request delay is separate; parry success is never inferred merely from pressing dodge. Normal resets after another admitted action/field exit or explicit unqueued idle; queued next stage advances at its gate. Already emitted transfer/proc events retain causal lineage; no post-cancel full packet dump or committed cost refund.

## Selected full resource specification

**Mixed source/model:** owner energy0..120 (baseline0), Cycle0..100, independent exact fractional carries/overflow, shared [per-packet grants](../../shared/energy-and-gauges.md). Burst requires/spends**120** at admission; Skill/Normal/Lilith/plunge/riposte have0 Ultimate cost, consistent with separate positive source awards and zero-start normal generation. Skill requires its own CD; Lilith requires live form permission + correct next stage, on-field living source and recovery. Support is the shared outgoing100-Cycle qualified adjacent switch, not a free extra button. Source `Cau_Fadia039SkillAddCharge` requests **0 extra Cycle**, not its unused local50; retain ordinary3-packet **7.591 Cycle** rather than the stale100 tooltip or a fabricated50 bonus. Transfer tiny awards remain nonzero as in the packet table.

Use `DT_GlobalCommonData.FadiaQLimitHp` at **character level**, constant interpolation/endpoints; this is the chosen native-getter binding. Values1/20/40/60/80 are705/2340/4596/6852/**8544**. Do not scale it with equipment/A3/temporary MaxHP: the source curve already has80 character-level coordinates. On actual player HP-loss settlement, if prior missingHP<T and new missingHP>=T, **fill Fadia's integer Ultimate pool to120 once**, retaining fractional carry, without extra CE or per-teammate distribution. The source PParameter3=1 is interpreted as full-cap fill, matching the description; it is not+1 energy. Record fill delta separately for conservation. Further damage while still above threshold gives no new fill. A later heal below threshold re-arms the crossing naturally. MaxHP-only reconciliation/stat clamps do not masquerade as incoming HP loss or produce fills/mark damage/gear stacks.

At Burst commitment snapshot enhanced branch **T>0 and (MaxHP-currentHP)>T**, strict source predicate; equality gives energy fill but **ordinary** Burst/heal. Then cost, applicable gear/entry state, high-Q flag and immediate self-heal settle; attack damage uses live MaxHP later. Source activation requests buff before heal; healing cannot turn off a captured enhanced choice. Ordinary heal is15% current MaxHP (A4 replaces30%); enhanced30% (A4 replaces60%), not multiplication by an additional2 after replacement. Apply actual healing modifiers once then clamp to missingHP; expose attempted/effective/overheal. CurrentHP never enters outgoing HP-coefficient formulas. At MaxHP10000/T8544, crossing HP1457->1456 fills; Burst at1456 ordinary, at1455 enhanced. This preserves the recovered equality conflict rather than rounding both into one rule.

No additional held-channel resource or separate Lilith stock exists. Native grant duration and stage counter limit it. Dodge uses the selected common15-stamina/.9s cooldown profile, no owner override; consumption efficiency/recovery/cap come from the explicit shared account profile. Jump/plunge require their authored landing/time/action cost, rather than a free Critical success. The selected baseline uses no stamina-dependent action, so does not invent an account rate. Failed ordinary inputs are diagnostics; no implicit waits, energy prepayment from future threshold events or reset at cycle boundaries.

## Selected full effects specification

**Mixed:** direct source bindings and descriptions are preserved below; native/public approaches already supplied the values. The following selects finite lifecycle/settlement rules with counterchecks, instead of leaving `ApplyFadiaZhouYin` or native C++ recovery as implementation blockers.

**Incoming damage / All Eyes on Me:** Fadia's passive is party-wide and works off field while living. Resolve an incoming enemy/scenario damage amount through its initial target's ordinary defenses, then split **before shields**; child shares are fixed amounts and do not apply defenses twice. If another teammate is targeted and Fadia starts above1HP, route **r=.50 (A1 .75)** to Fadia, leave1-r on that teammate. Each child's own shield absorbs before HP. Mark all redirected children nonredistributable, so neither another Fadia nor the reverse share can recurse. While at least one other party member lives, Fadia's HP has a **floor1**; the excess is prevented damage, not a second redirect. At1HP she stops intercepting incoming teammate hits. If she herself is targeted at1HP, use the same source-bound share ratio r as a **total reverse-share pool**, divide it equally among other living equipped teammates, apply their shields/HP once, and leave1-r to Fadia with her living-ally floor. This reverse-branch ratio/distribution is an inference reusing the actual .50/.25 carriers rather than inventing a new percentage. A1 replaces total r, not multiplicative reductions.

No other living teammate => no redirect/reverse-share protection, Fadia can die. A direct hit starting above1HP uses that starting branch for the whole event, reaches the floor if protected, and only the next incoming event reads the1HP reverse branch; do not duplicate a mid-hit reverse pass. Direct Fadia damage above1 is otherwise her own damage. Source `ImmuneDeadByTeammates=1` and the described survival behavior support the floor; prevent only the excess, not the paid HP portion. Dead/removed Fadia stops interception immediately; surviving actors still use their own effects. HP/stat clamps, city damage and scripted noncombat changes do not fabricate enemy attacks; explicit self HP costs do not redirect, but paid positive HP loss may feed her mark/threshold when their cause is admitted as combat HP loss. Report each initial pool, shield absorption, applied HP, overkill and floor prevention for conservation.

Checks: incoming1000 to teammate, r.5, Fadia100HP/another ally alive/no shields -> original ally500HP loss, Fadia99HP loss,401 prevented, sum1000; her mark sees **99**, not500 or1000. Next1000 targeting Fadia at1 with two other allies ->250 each, Fadia500 prevented. Shield100 on first ally changes that branch to150 HP+100shield, no second redistribution. A1 r.75 changes initial split to750/250; it is not62.5%. If both other allies are dead before a later event, Fadia takes its own lethal remainder normally.

**Destructive Experience:** apply at Skill.2 real event .95792557, after its direct three-packet sequence requests (the last delayed hitpause can follow). Without A6 choose highest **current effective MaxHP** enemy among living in-range candidates, stable lowest target index tie. With A6 choose up to10 highest-MaxHP candidates, stable ties. Native searchMaxCount10 limits marked recipients, not ordinary direct Skill AoE. Each selected enemy receives its own **15s** generation, snapshot Fadia MaxHP for cap, accumulated applied transfer0 and r3/A2r6. Later MaxHP buffs do not retroactively enlarge this generation's cap. Refresh/recast ends old generation first (A2 remaining settlement if eligible), then creates a new independent15s/budget. Marks survive Fadia field exit, remove on source/target death, combat exit, expiry or budget exhaustion. Damage on another teammate only matters after actual redirected HP reaches Fadia.

Each Fadia HP-loss event feeds **all** live own marks independently once, ordered target index; no budget division. Apply the reflected fixed amount from the packet section, debit actual enemy HP lost, then remove exhausted mark. A2 remainder on ordinary expiry, recast, dispel or living source combat exit is `max(0,budget-spent)`, clamped to living target HP and settled **once**; do not multiply by6 again. Source or target death/scene destruction cancels future transfer without attacking a dead actor or granting a postmortem purge burst. End combat may settle a living mark before stat rollback; record that causal order. A0 never receives a removal filler. A6 ten targets can each take their full capped reflected amount; no target gets ten copies from the source search count.

**High-Q / healing:** missingHP branch snapshots at Burst commitment before its self-heal; enhanced+.30 **general damage bonus**, limited to the initial six Q-tag packets and their admitted Lilith continuations, additive with ordinary Q/Mental/general zones. It is Infinite under the source carrier, removed on form end; the inactive10s scalar is not a timer. Each admitted fifth-section cast keeps its captured high-Q/A5 eligibility until its last packet finishes, then releases that context. Other on-field actions, expiry or switch remove future permission and live state immediately; already emitted admitted final-stage packets retain only their captured eligibility, not frozen live HP or all buffs. Healing is one immediate self-target request at activation, ordinary.15/A4.30 or enhanced.30/A4.60 of current MaxHP, with live healing modifiers/clamp. Neither every Lilith hit nor each enemy receives another self-heal. No overheal becomes damage unless a selected equipment handler explicitly says so.

## Selected full passive specification

**Mixed source/checked inference:** B2 Furnace of Guilt and B4 Denial and Plunder are enabled at their breakthroughs, not dependent on awakenings. B4 adds **+.10 HPMaxUp** to each equipped party actor's own baseHP, including Fadia and off-field/dead members. Party membership governs it; death does not withdraw the party stat, revival re-add must deduplicate its single identity. Remove on leaving party/combat teardown. CurrentHP stays unchanged on MaxHP increase and clamps on decrease; no implicit heal/HP-damage callback. Example base10000/up.20/flat500 ->12500, with B4 ->13500, not13750. Same source/recipient cap1, no accidental duplicate native revival grant.

**B2 Nova extraction:** use shared Nova end lifecycle (one end callback per generation, including refresh/dispel while target remains alive). Source component's Infinite5s-period wrapper is treated as a listener while Nova exists, not a free autonomous extraction every second forever. Public end-trigger and native5s Nova cadence agree on one natural end; they do not justify five per-second grants. Snapshot Fadia **combat base MaxHP B** (after A3 base increase, before gear/HP%/flat) for the combat. Each ended Nova on a living marked target requests **+.10B flat MaxHP to every configured party member**, with per-Fadia bonus cap **+.50B per member** over the combat. For accepted increment `a` per member, target loses **20*a MaxHP**, so the first increment is2B (200% base) and retains the native `TargetLostRatio=20` without pretending it is2. Ratio/cap roles are an explicit inference: interpret the source .50 as the cumulative **source-gain** limit; applying .50B to enemy loss before a2B first drain would contradict the same description's200%. This chooses a consistent finite model and records that textual conflict.

Use one source-wide accepted-increment ledger so party size cannot multiply the target drain: four allies gain a each while target loses20*a once, not80*a. Five complete increments saturate the .50B gain; later Nova ends give no further gain/drain for that Fadia. Enemy MaxHP is floored at1; if less than20*a can be removed, reduce a proportionally before granting, preserving20:1 and that source's cap. No additional numerical target/MMO cap is invented from omitted `LimitTargetRatio/MMOLimitTargetRatio`; the selected stationary PvE profile uses the explicit living-floor/cumulative-source rule. A different declared mode must expose its extra encounter limit as a scenario restriction if known; it cannot silently treat missing native defaults as0. Multiple Fadias, if allowed by a custom scenario, keep source ledgers; normal legal roster forbids duplicate owner builds.

MaxHP extraction is a **stat transfer**, not an ordinary HP damage/heal packet: keep currentHP unchanged on ally gains, clamp enemy currentHP to its reduced MaxHP, expose a structural-HP-change receipt (not damage DPS, crit, energy, gauge, mark or incoming-damage event). This still changes later death thresholds and HP-scaled damage. At combat end restore all actor MaxHP contributions/target reductions and clamp only, no resurrection/heal. B2's source carrier explicitly `CheckSourceDeath=false`: already enabled membership/listener and extraction ledger persist if Fadia dies; do not erase existing gains or roll back enemy caps on field exit/death. New source combat ends/removal performs rollback. Dead party members can carry the MaxHP stat while HP0; revival does not grant an extra extraction.

Check B10000: first Nova end targetMax100000->80000, each allyMax+1000; fifth ->targetMax1 if a small target bound intervenes, with accepted increments reduced consistently rather than a hidden kill. On an adequately large target five ends drain100000 total and grant5000 each; sixth0. Enemy remaining5001 permits only5000 removal, so a250 gain, not1000. This distinguishes the .10 source gain and20 intermediate ratio, and keeps currentHP/stat changes out of damage-trigger recursion. Limits and source base are fixed for the combat; temporary HP gear does not increase B again.

| Selection | Rule |
| --- | --- |
| A1 | Intercept/reverse total share ratio.75 replaces.50; not two successive multipliers. |
| A2 | Each enemy mark r6/cap.25 snapshotted Fadia MaxHP; one remaining-budget removal settlement as above. |
| A3 | On combat entry increase Fadia **base MaxHP by30%** once, before HP%/flat; remove at exit. Do not add the native `MaxHPEachAdd=1000000` helper field as a million HP bonus: it is not a delivered stat magnitude and conflicts with the explicit30% rule. Captured B for extraction includes this base increase. |
| A4 | Ordinary/enhanced activation self-heal ratios replace.15/.30 with.30/.60; outgoing HP coefficients unchanged. |
| A5 | Lilith state CR+.50 percentage points, summed/clamped with ordinary CR; includes initial Burst since form is granted at activation. Cleanup follows form marker; fifth captured section preserves its eligibility through delivery. |
| A6 | Up to10 independently budgeted/duration-marked targets, stable MaxHP order; does not change direct AoE count. |
| R3 | Effective Normal/Skill/Burst+1, including Normal plunge/riposte and Q-owned Lilith; QTE remains purchased level, native13 cap. |
| R6 | Party+.10 ATK% of each own baseATK while Fadia belongs to team, including off-field; one deduplicated identity, no multiply of finalATK. |

Awakening identities are selected independently; resonance uses count. No city/life passive modifies combat Fons, Grains, HP damage or timing. Explicit baseHP17089 with A3 becomes22215.7 before multiplicative HP zones, not a flat.30 of already compiled55543. These conditional handlers remain part of implementation even when A0 baseline does not select them.

## Selected full form specification

Source `.fadia.mods.lilith` returns the remaining native permission in World seconds; `.fadia.lilith.active` is the same half-open permission predicate. Absent permission, expiry, field exit or fifth-stage commitment reads0 even if a previously committed packet remains pending. `.fadia.out_of_combat.active` reads the distinct native combat flag; field exit alone is not combat leave. No `.resources.lilith` stock is invented. Rust source/native replay checks cover two continuing paid cycles, exact9.42s expiry and field-exit cleanup, plus a Burst-only script using the same selected model. Other source action/assumption bindings remain L3/L5.

**Mixed:** Burst activation grants `GA_Fadia_UltraSkill_Melee`, sets Lilith state, samples enhanced flag and self-heal as above; initial six hits remain its Burst action. From recovery, Basic selects follow-up stages1..5, replacing the ordinary Normal chain. Stages have1/2/3/6/1 packets, not five damaging hits total. No independent summon, autonomous DoT, Burst recast or second health drain is configured. Stage1Pre is the approach alternative to stage1 and uses its own startup/recovery; no extra normal hit precedes it.

Permission ends on9.42 nonhybrid deadline, fifth-section commitment, ordinary field exit, source death or actual activation of the native exit-class set Appear/ordinary Melee/QTE/Skill. Normal input while permission is valid routes to Lilith rather than accidentally activating the ordinary exit class. Dodge/Riposte can interrupt current notifies but do not themselves match that exit set; real form deadline continues and an interrupted stage is not replayed for free. A later admitted Basic resumes the next not-yet-committed Lilith stage if permission still valid. Skill/qualified Support/ordinary switch clear form/high-Q/A5 and any queued unissued stages; no resource reset. Expiry while a committed fifth stage plays keeps its captured eligibility until that cast finishes, no new sixth input. Other unfinished stages at natural expiry likewise finish their already admitted action, then return to ordinary state.

The source visual weapon-hide/dissolve sequence adds no combat packet or invented fixed recovery; input windows are the montage gates. Infinite `ChangeState`120 and high-Q10 serialized scalars are inactive and do not overrule the timed grant. Ordinary field entry has only authored approach Normal (if requested) or non-damaging entry; no automatically inserted Support. Plunge uses one actual landing and h, critical response uses its own single token, neither duplicates both ordinary and enhanced Basic.

## Selected recipients and dependencies

Direct Normal/Skill/Burst/Lilith/plunge/riposte/QTE damage uses all eligible living in-range stationary dummies, full coefficient per target, stable ascending index. Source range wrappers do not imply a shared damage budget. Skill mark has its separate highest-MaxHP/10-target cap; party redirection/healing is never selected through enemy enumeration. Transfer remains one mark recipient; resource grants once per causal packet, not recipient. Source death cancels unissued owner notifies/marks; B2 stat ledger and B4/R6 party membership effects follow their explicit persistence rules. Incoming critical/parry success requires authored enemy events, never enemy AI invented by the simulator.

Use [shared zones](../../shared/damage-and-stat-zones.md) with distinct Mental/Psyche RES, Mental DEF bypass but ordinary crit/bonuses for Q/Lilith, and [energy/gauges](../../shared/energy-and-gauges.md). Psyche+Chaos qualified switch creates Nova; Psyche+Lakshana Stain. Ordinary Nova is no-crit reaction damage and never borrows Fadia's Q/Mental weapon bonus. Fadia Q/Lilith are Mental and do **not** receive Psyche Stain/Psyche bonus even though she is a Psyche character; her ordinary attacks/Skill do. [Shared Nova/Discord/Break](../../shared/cycles-and-break.md) defines generation end, damage owner, cap reset and equal-time callbacks; her B2 listener fires once per eligible generation end. No-bar target has no team Break contribution despite nonzero sourceUnbal. A breakable target uses original cap and declared duration/Discord fraction; no arbitrary universal Discord percentage.

Equal-time: old expiries/HP-stat clamp before new hits; then initial enemy/scenario damage split/shields/playerHP, threshold fill, mark-transfer queue, incoming-HP equipment callbacks, then another authored input. Transfer target HP/gauge/kill settles before the next mark index. A Burst commitment snapshots enhancement before self-heal; its gear HP grants precede calculating that heal/live outgoing HP damage, with no synthetic threshold event from the grant itself. MaxHP-only changes never feed mark/R6/HP-loss gear. Causal flags prevent redirection, reflection and HP-extraction recursion without turning those mechanics off.

## Selected complete implementation handoff

**Mixed:** implement the seven selected blocks as one owner state machine, preserving their source rows and inference flags in both CLI/browser results. Required state: Normal/Lilith stage + admitted cast contexts, form deadline, high-Q/A5 eligibility, threshold crossing, HP/shield/floor/share ledger, independent enemy mark budgets/generations, Nova source-wide stat-extraction ledger/rollback, B4/R6 party auras, exact energy/Cycle carry and overflow, cooldowns and equipment timers. Ordinary Psyche and Mental must remain different channels. Synthetic receipts establish model arithmetic, not native C++ behavior or current executable completeness. No protected native consumer needs to be recovered before implementing these selected rules.

**A0 build:** Fadia `character:1039`80/breakthrough6, Normal/Skill/Burst8 (QTE8), no awakenings; Eternal Waltz `weapon:fork_Arachne`80/6/**T1**, Tiny Big Adventure `set:Suit10`. Set count is 4; console contributions use `add stats`, with no module geometry or specialization inference. Compiler static baseHP17089, HPMaxUp**1.6225**, flatHP5600, baseATK**1020=596+424**, flatATK420 =>ATK**1440**, DEF909, CR.35/CD1.10, CE1, CI/BI0, Psyche/Mental static bonus0.

Apply **Eternal Waltz's unconditional+.20 HP% once**, which this static report has not included, and B4 **+.10 HP%** once: `round_half_up(17089*(1+1.6225+.20+.10)+5600)=` **55543 MaxHP**. Do not add either to finalHP multiplicatively or compile them twice. [Weapon model](../../weapons/fork_arachne/passive.md): Burst commitment grants wearer+.10 **Mental** damage bonus for10s, cap1/refresh; no Psyche bonus. [Tiny Big Adventure model](../../sets/hp-suit/passive.md):+.10 staticHP already included;4pc positive settled HP loss grants one+.04 baseHP stack, independent10s lifetimes/cap10/no refresh at cap. Burst requests10 stacks at admission, fills remaining slots only. CurrentHP unchanged on growth, clamps on expiry; such clamps are stat reconciliation, not another hit. Both gear mechanics persist off field under their timers.

**Scenario:** solo active A0 Fadia, initial fullHP55543, Ultimate0/cap120, Cycle0/cap100, carries0, no cooldowns/marks/stacks/form. Stationary target80/HP24million/DEF1080, PsycheRES.20 and **MentalRES.20**, explicit no Break bar, seed42. All eligible dummies in range; baseline one. B2/B4 enabled. Solo Psyche creates no Nova rather than disabling extraction; the full conditional ledger is specified above. No dodge/jump, no account stamina read. At relative52 in **each**75s period author one incoming combat hit against Fadia with **8550 post-defense/pre-shield damage**, no shields; this is a declared incoming-damage fixture, not an invented enemy attack coefficient. No other incoming hit or heal. It remains nonlethal without teammates and exercises actual mark/threshold/gear dependencies.

| Relative time, repeat+75 without reset | Events/state |
| --- | --- |
| 0,8.2969165667,16.5938331333,24.8907497000,33.1876662667,41.4845828333 | Six complete ordinary chains, recovery49.7814994. Full exact source/profile chain duration8.2969165667; microsecond boundary rounding occurs once. Explicit wait50. |
| 50 | Skill3/1/2 damage **50.21566328**, **50.77812625**, **50.9558684**; first hit's pause affects later events, never its own impact. Mark installs50.95792557, budget.08*55543=**4443.44**, expiry65.95792557 if not exhausted. Skill recovery51.394559, CD66.19837612. |
| 52 | Incoming8550: HP55543->**46993**, missing8550>8544; threshold fill120 (already capped here, actual fill delta0). Mark reflects its complete**4443.44** budget once and removes. Source awards.005 energy/.008Cycle/.001Break for that positive transfer. Then one HP-set stack raises MaxHP to**56226**, currentHP remains46993; stack expiry62. |
| 54 | Burst requires/spends120, captures enhanced branch; gear supplies nine remaining HP stacks (expiry64), MaxHP**62378**, Mental+.10 until64. Enhanced high-Q+.30 general. Self-heal attempts**18713.4**, effective**15385**, reaches full62378. CD74. |
| 56.7073529..57.8934803 | Six initial Mental Burst hits, liveMaxHP62378, no raw energy/Cycle. |
| 58.4159207,59.5181316,60.1739828,61.5890820,63.1775591 | Five admitted Lilith follow-ups; the last admission is before form deadline63.42. All13 packets remain separate. At62 the older singleHP stack expires: MaxHP**61695**, currentHP clamps; four later row6 hits, row7 and row8 use61695. |
| 63.5071994 | Final Lilith8 hit uses its captured high-Q eligibility despite form-admission expiry63.42, liveHP61695 and weapon bonus still active. Final recovery64.0494761. At64 nineHP stacks and weapon bonus expire, MaxHP/currentHP clamp to**55543**. |
| 65 | Seventh ordinary Normal chain, finishes73.2969165667. Explicit wait75. Both cooldowns, gear buffs and form have ended; mark already exhausted. Repeat with continuing resources/carries. |

Every period emits **79 damaging packets**:56 Normal+3Skill+6Burst+13Lilith+1mark. Natural rawenergy**178.898** includes the tiny mark award, no energy from self-heal/stat clamps. Ends at75/150: **41+.898 /41+.796**, cumulative overflow**17 /76**, conservation `357.796=240 spent+76 overflow+41.796 retained`. The crossing handler runs both periods but grants0 extra at cap; a lower-energy counterfixture crossing from missing8543->8544 with balance0 must instead fill120 and preserve the same exact-equality ordinary-Burst distinction. Cycle**298.007** each period =>end100+.007 /100+.014, cumulative overflow198 /496. HP is full55543 at each period boundary through actual heal+stat expiry, never a reset instruction. No pending packets, mark or extraction generation at75/150.

Independent Fraction/event audit  / `fadia-two-cycle-receipt.json` checks158 packets/150s, all five admissions before9.42, late final packet, live MaxHP expiry, self-heal clamp, resource conservation, transfer cap and floor arithmetic. Expected damage uses CR/CD expectation and fixed mark settlement:

`2 * [7*6.087*1440*1.385*.5*.8 + .113*55543*1.385*.5*.8 + ((.259+.246)*62378 + .353*61695)*1.4*1.385*.8 + 4443.44]`

= **249118.065092**, expected DPS **1660.787100613333** over150s. Here1.4 is the **sum** of1 +.30 high-Q +.10 Mental weapon, Mental has no.5 DEF factor, and mark4443.44 is already resolved fixed loss. The .246/.353 split is the Lilith HP coefficient before/after62, not a new invented attack coefficient. All animation recovery, incoming event and waits remain in the denominator. Required off-field/share/HP-extraction/A2/A6 handlers are specified even though this legal A0 solo fixture selects only their applicable conditions.

## Executable full-kit acceptance

`fadia.full_kit_v1` binds the selected main action clocks, Skill cooldown marker and mark notify, strict enhanced-Burst snapshot before its single heal, all thirteen Lilith contacts, A2 independent mark budgets/removal settlement, A3 baseHP, A5 captured critical eligibility, A6 stable ten-target selection, B4/R6 membership and B2 Nova extraction. Direct transfers bypass ordinary damage/critical zones once while retaining source resources. Source death clears marks; field exit preserves marks and membership while removing future form permission. Explicit living combat leave purges A2 marks before structural rollback; re-entry deduplicates membership.

Rust checks use independent11000 effective-MaxHP fixtures: missingHP8544 is ordinary while8545 captures enhanced damage; healing is1650/3300/6600, and the A2 budget2750 settles1800+950 once. Six Nova ends with combat base10000 grant5000 per party member and remove100000 targetMaxHP total, with no extra sixth grant; a5001 target permits only250 per-member gain and remains at1. Dead source membership and listener persist, and rollback restores MaxHP without healing or resurrection. A3 with base10000, HP%20 and flat500 gives17400 after B4, with flat500 preserved through leave/re-entry.

Structural changes have typed `structural_hp_changes` receipts identifying time, source, actor/target recipient, cause and before/after MaxHP/currentHP. They contribute no damage, healing, energy or HP-loss event. Native aggregation and replay preserve the same receipts.

The selected scenario above describes full combatHP55543, but membership growth itself cannot heal: the current compiler already includes Waltz's unconditional20% HP, yielding53834 before B4. Native legal acceptance therefore starts at compiled53834 currentHP, grows MaxHP to55543 at entry, and retains that1709 missingHP until the actual first Burst heal. The first incoming hit leaves45284 and the first effective Burst heal is17094; subsequent periods follow the full-HP boundary. Outgoing live-stat arithmetic and the independent249118.065092 expected two-period damage remain unchanged. These are model checks, not measured game timing. Native secondary controls and qualified Support are accepted below. Additional public-source control/approach/parry-route lowering belongs to native workflow; the accepted field-entry baseline is non-damaging without qualified Support.

The same model now binds explicit front/back Dodge, successful Critical Dodge to one next Basic Riposte, landing/height-scaled plunge, qualified Support and exact owner cancel gates. Source-coordinate gates include only already caused hitpause; failed early inputs preserve pending contacts and resources, while admitted cancels forfeit unissued contacts. Jump or Dodge retains an existing Lilith admission clock, Riposte preserves its future stage and A5 eligibility while receiving no Mental/high-Q bonus, and Skill/Support/field exit removes future permission. No movement damage or free Critical success is inferred.

Independent secondary fixtures check two continuing periods at ATK1000/DEF-half: Riposte1525.5 plus Normal371 once per target, total14.414 raw energy and23.822 Cycle regardless of AoE recipients, with30 paid stamina. A .6s landing and height bonus.5 yield one771 plunge at632851us; jump cancellation before its delayed pause keeps that already caused pause on the next cast. Normal2 jump at1692802us cancels its second contact; one microsecond earlier rejects atomically. The first Normal-to-Skill gate is730152us after integrating the actual source curve. Back Dodge's494283us Skill gate differs from front333754us. Support delivers one1714 packet per target at685983us in the same ATK1000 fixture, grants7.6 raw energy once and requires the actual outgoing100-Cycle switch;99 Cycle gives non-damaging entry.

A second legal native fixture uses the equipped1440 ATK build for two paid Ripostes and two authored plunges, preserving independent1757.376/888.192 noncritical packets,15 energy,24.8 Cycle,30 stamina and seeded aggregation/replay. The main158-packet fixture and the Nova refresh/natural-end structural receipts remain accepted together with these secondary paths. Timing and incoming-success policies remain disclosed product models, not measured game animation/parry windows.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five inputs emit packet groups [Melee1], [Melee2, Melee3], [Melee4], [Melee5, Melee6], [Melee7, Melee8], for eight total Psyche ATK-scaled hits. The frozen native ATK arrays match the extracted public ratio strings across levels 1-9. This model has no MaxHP or current-HP damage term.

Execution policy: The chain loops after stage five and resets on a different admitted ability or field exit. A gap greater than 2 seconds between input starts resets; exactly 2 seconds continues. Ordered multi-hit delivery occurs at the primary impact time; timing and travel remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`fadia-normal-chain.json`](../../../../reviews/q10-game-action-binding/fadia-normal-chain.json).

## Recovered threshold and damaged-HP branches

**Source facts for PC_140/logic-566459, not a completed executable Burst.** The frozen `DT_GlobalCommonData` (optional cached source `DT_GlobalCommonData`) already supplies `FadiaQLimitHp`; original export SHA-256 `d15fecf599b4a51567972c3c759b63ce5ae1ec4d54c60cc5f8a9a5cef19c4386`. It has 80 constant-interpolation keys with constant endpoints. At coordinates 1/20/40/60/80 the values are **705 / 2,340 / 4,596 / 6,852 / 8,544**. The tooltip's `CLGlobalCommonDataCurveID=FadiaQLimitHp` identifies this threshold row, distinct from talent-level figure curves. Native getter lookup coordinate/scaling remains open; these samples are not automatically live thresholds.

`GA_Fadia_UltraSkill`, export SHA-256 `2d73b68ac47842b4fe7af750148cd03f24d5e16d263739d7bdfe1922301f7027`, `/2/ScriptBytecode`, calls the parent activation, requires a valid PlayerCharacter with authority and a valid ability system, then reads current HP, `AttributeComponent:GetFadiaQLimitHp` (`T`) and `GetHPMax(false)`. Offsets **405, 552, 627, 665, 703** select the enhanced path only when:

```text
T > 0 AND (MaxHP − CurrentHP) > T
```

The true path requests `Buff_Fadia_HighQ_DamUp` at 717, then `GE_Fadia_HighQ_RecoverHP` at 859. The false path requests `GE_Fadia_Q_RecoverHP` at 1006. This establishes named branch requests and order; native effect settlement, damage eligibility, cast/packet timing and remaining state lifecycle are separate.

`Player_039_Fadia`, SHA-256 `c84a291ecdd38c47de573405f9b352b795f9f991b1c87647cd9eb982e767b37d`, `/10/ScriptBytecode` (`OnHPChange_Event`), has a different threshold boundary. It requires `IsServerOrStandalone` and `T>0`, reads live `GetHPMax(false)` for old/new loss calculations, and requests `GE_Fadia_Q_RecoverEnergy` at **722** only if **OldReducedHP < T AND NewReducedHP >= T** (comparisons 565/632, conjunction 670). The callback's actual binding/delivery and native getter behavior remain open. The player export retains one class-read error; preserve it rather than certifying the entire player object from these function streams.

The energy GE, SHA-256 `f543d95a1c571968374067c1df95902d5468b2a11711331d2402893dc0685494`, `/1/Properties/Executions/0`, declares native `ChargeExecution` with a source-snapshot additive **`PParameter3=1`**. That is an execution parameter, not evidence for a one-unit energy grant. The description's full-energy restore still needs native mutation/recipient/cap semantics or a labeled model; no ordinary-hit grant policy is substituted.

**Independent boundary example:** with explicit MaxHP 10,000 and `T=8,544`, OldHP 1,457 → NewHP 1,456 crosses from loss 8,543 to 8,544 and takes the energy-request branch. At CurrentHP 1,456 the Burst enhancement condition is false; at 1,455 it is true. These are static predicate counterexamples, not combat measurements. The published/frozen description uses greater-than-or-equal wording for the empowered Burst, whereas this client graph uses strict greater-than. Preserve the conflict and cutoff applicability; do not silently unify both predicates.

The source bundle retains exact graph/effect bytes and checks. Configured thresholds and branch locations are resolved; the [unknown card](#unknown-execution-policies) owns native getter, callback, conflict and state/settlement decisions. Historical model receipts and runtime behavior are unchanged.

## Burst healing, enhancement and Lilith source chain

**Source facts for the same PC_140/logic-566459 client; no runtime policy revision.** The follow-up acquisition retains the exact function/effect sources, their warnings and static checks.

`Con_Fadia_Q_DamUp`, SHA-256 `e2299a9c5e0f82ff821206bbc3b656caa7ff1c6322065716af8496c2072d6488`, `/2/ScriptBytecode`, constructs the single tag `State.Damage.UltraSkill` at 0, casts the supplied gameplay effect at 65, and returns that effect's `AssetTagHasAllMatchingGameplayTags` result (181/232). It does not inspect a target's actor tags. `Buff_Fadia_HighQ_DamUp`, SHA-256 `3306093049c4be40b9e1fed5be1a00202cd37c3fcccad746053e902a9e8ab7c7`, `/1/Properties/CustomApplicationModifierInfos/0`, gates an **additive `DamageUpGeneralBase`** modifier through this predicate; its explicit scalar is 1 and curve is `DT_FadiaEffectFigure:FadiaHighQDamUp=0.30`. This is a general damage-bonus carrier under the [shared zones](../../shared/damage-and-stat-zones.md), not a separately multiplied final-damage term. Actual native application/capture and eligibility of each delivered effect remain separate.

The checked initial `GE_Player_Fadia_UltraSkill1_Damage` and follow-up `GE_Player_Fadia_UltraSkillMelee1_Damage` classes both declare `GE_Player_Fadia_UltraSkill_BaseDamage` as `Super`. That parent explicitly carries `State.Damage.UltraSkill` in both its asset-tag component and CDO combined tags. This supplies an inherited-tag source path for those two examples; native construction/application is still separate, and no blanket classification of every follow-up is inferred.

That enhancement buff has **Infinite** duration policy, target aggregation and stack limit 1. Its serialized duration scalar 10 is not an active ten-second expiry. The player graph contains a specific removal request at `/6/ScriptBytecode`, offset 75, following `ChangeFadiaState(false)` at 60; reaching that branch and native mutation still require callback/lifecycle evidence. `Buff_Fadia_ChangeState` is separately Infinite: its serialized scalar 120 is likewise not a two-minute form window.

Both Burst heal GEs use native `CustomReplyHealthExecution` and source-snapshot additive `SkillCoefficient`, with ordinary/enhanced custom magnitude classes `Cau_Fadia_Q_RecoverHP` / `Cau_Fadia_Q_HighRecoverHP`. The ordinary calculator, SHA-256 `5b5a8de68fc156dcff60f04193d94ff0c26a90d7f07e1ac99f7fb67d36760dc9`, `/2/ScriptBytecode`, initializes temporary MaxHP and ratio to zero; reads the scalar fields, then evaluates their curve rows at **coordinate 0**, assigning returned row values on result byte 0. Flow-stack destinations 852 → 260 → 687 establish evaluation before the owner-ASC read and final product. With a valid owner ASC it reads `GetHPMax(false)` at 357/408, adds the second ratio only when `Buff_Fadia039_Level4` is present (464/516/584/659), then returns `TempHPMax × TempMaxHPMul` at 745/820. The table's constant endpoints give ordinary ratios **0.15**, plus **0.15** under that buff. This is the custom magnitude's source expression; native healing bonuses, recipient, overheal/clamp, rounding and HP-event delivery are not proved by it.

The enhanced calculator, SHA-256 `8e9b8f21f5a27c1bcbf93873a49e393d9a9031331cd0af356e4dda0feed3193a`, declares the ordinary calculator as `Super` and changes the two row names to `FadiaHighQRecoverHp` / `FadiaHighQRecoverHpLv4`, each **0.30**. It contains no new Function export. Its scalar/curve-table members are omitted in the child projection; preserve the delta/default boundary rather than flattening them into certified effective values. The row override and parent function are recovered, while effective child construction remains a separate dependency.

`Buff_Fadia_GiveUltraMelee` grants `GA_Fadia_UltraSkill_Melee` with `RemoveAbilityOnEnd`, target aggregation/limit 1, a server-or-standalone requirement and an `Event.Common.OnlyServer` asset tag. Its self role-out event requests removal of the grant buff. Duration uses `Cau_Fadia_ChangeDuration`, SHA-256 `0889fb0091d0bb0997f5cd3dd24c502ed77a39c338376d6442a589caf9c40469`, `/2/ScriptBytecode`: after an owner-player cast, **IsHybridMode OR IsOnlyStandalone** chooses `FadiaUltraSkillDur=5`; neither chooses `FadiaUltraSkillDur_ol=9.42`. Both curve calls use coordinate 0 (262/369), with successful row results assigned at 508/476. Native mode classification, duration units/clocks, refresh and expiry ordering remain unresolved. The existing five-second model receipt is still a bounded scenario policy.

The granted ability's `/3/ScriptBytecode` requires the grant buff in its `K2_CanActivateAbility` result. Its `/1/ScriptBytecode` requests grant-buff removal on authority when an ended ability's start section is `UltraMelee5` (300/328/370/412/460), or when a jump-section request is named `UltraMelee5` (549/591, then 15/20/62/72). The latter path also calls `ClearTriggerComboSkill` at 142. These are section-based removal requests, not proof of five delivered hits, a combo timer or natural callback delivery. The player graph separately requests `Buff_Fadia_ChangeState` on/off and high-Q-buff cleanup; native `FadiaLogicObj` and delegate dispatch retain lifecycle responsibility.

In the player graph, `OnActivateAbilityEvent_Event` enters at 322, requires authority and a valid ability, and requests the grant buff at 583 for an ability class descending from `GA_Fadia_UltraSkill`. Otherwise, an exact-class array containing `GA_Fadia_Appear`, `GA_Fadia_Melee`, `GA_Fadia_QTE` and `GA_Fadia_Skill` (708/755) routes to grant-buff removal at 10. `OnGiveAbilityDelegate_Event` enters at 1037 and routes a granted `GA_Fadia_UltraSkill_Melee` descendant to `ChangeFadiaState(true)` at 40; `OnRemoveAbilityDelegate_Event` enters at 1242 and routes its removal to the false-state/high-Q-cleanup branch at 60/75. `InitChangeState` enters at 167 and checks whether a granted melee ability can be resolved. These recover the callback bodies and class tests; their actual delegate binding/delivery and native grant/removal ordering remain open.

The Skill's `OnReceiveEvent`, `/4/ScriptBytecode`, calls the parent handler, then compares the event to **`Event.Montage.Player.Skill.2`** (28/84). After player/`FadiaLogicObj` casts, offset 407 requests `ApplyFadiaZhouYin(NOT HasBuff_Fadia039_Level6, Buff_Fadia_ZhouYin, DamageRange_Fadia_Skill_SearchTarget)`. The boolean's meaning, target selection, mark application, redistribution and state mutation remain native responsibilities; no recipient or redirect ratio is inferred from its name.

**Independent explicit-input checks:** with valid MaxHP 10,000, successful ordinary curve reads yield custom magnitude 1,500 without the level-four buff and 3,000 with it. Applying the enhanced row names under an explicitly assumed parent-field resolution yields 3,000/6,000; those two numbers are conditional. With an existing eligible damage bonus 0.20, adding this 0.30 yields **1.50**, not `1.20 × 1.30 = 1.56`. Neither arithmetic example proves native healing settlement or effect admission. Exact-equality threshold behavior remains as recorded above.

## Skill mark, transfer and teammate-damage source inputs

**Serialized source facts, not a completed redistribution model.** The targeted transfer acquisition preserves the mark, transfer execution, teammate-sharing carriers and their exact identities. The native component and execution bodies remain separate from their configured inputs.

The frozen Reference guide description, retrieved October 4 with source-page SHA-256 `527328921a2d08338f7c89ed9923ea3f3b024086ad61cbac8c81eeb4320bb813`, supplies a **description-level rule**: above 1 current HP, Fadia redirects part of teammates' incoming damage before shields, and redirected damage is not redistributed again. At 1 HP she stops that redirection; damage received by Fadia is instead shared with living teammates before shields, with the remainder absorbed by Fadia. Skill describes the enemy with highest MaxHP, ending its mark on expiry, cap exhaustion or leaving combat. Its displayed parameters are 15 seconds, ratio 300% and cap 8% MaxHP. Awakening 1 describes a 75% teammate-redirect ratio. Awakening 2 describes ratio 600%, cap 25% **of Fadia's MaxHP**, and an immediate cap-minus-already-taken difference on mark removal. These published policies are available inputs for a bounded model; they do not prove native shield/HP event order, selection ties, same-time admission, recursion guards or removal delivery. The generic unknown must not obscure these already stated rules.

`Buff_Fadia_ZhouYin`, SHA-256 `a1ad0994c41af36e2ba62493122121cff96015f6943303e1b3434ffbbf2349ca`, `/2/Properties`, has HasDuration policy, explicit scalar 1 with `FadiaESkillZhouYinDur=15`, source aggregation and stack limit 1. Its `BUFF_LEAVE_BATTLE` event requests self removal, and its asset tags include `Event.Common.OnlyServer`. None of those fields proves a real-time duration, first/refresh clock or actual leave-battle dispatch.

Its `/3/Properties` belongs to native **`FadiaTransferDamageGEComp`**. The component property schema matches the public map's eight outer names/order/count. Primitive/compound gen-flag comparison retains mapped nested types; it does not recover constructors or prove every nested default. The source binds:

| Field | Serialized input |
| --- | --- |
| `TransferRatio` | explicit scalar 1 × `FadiaESkillTransRatio=3` |
| `TransferCapacityMaxHPRatio` | explicit scalar 1 × `FadiaESkillMaxHPRatio=0.08` |
| `SecondRatioRequiredBuffOnSource` | `Buff_Fadia039_Level2` |
| `TransferRatio2` | `FadiaESkillTransRatioLv2=6`; scalar omitted |
| `TransferCapacityMaxHPRatio2` | `FadiaESkillMaxHPRatioLv2=0.25`; scalar omitted |
| `TransferDamageGEClass` | `GE_Player_Fadia_ZhouYin_Damage` |
| `bApplyRemainingDamageOnRemove` | explicit true |
| `ApplyRemainingDamageRequiredBuffOnSource` | `Buff_Fadia039_Level2` |

All named curves already exist in frozen `DT_FadiaEffectFigure` as single coordinate-1 keys with constant endpoints. The alternate-row values are available, but omitted scalars are not silently initialized to one. Field names do not prove whose MaxHP is read, whether the capacity applies before or after multiplication, how reapplication resets accumulated state, or whether every removal reason flushes remaining damage. The true flag and required-buff reference are configuration, not a recovered native condition.

The October 17 [compact projection](../../../../openspec/specs/runtime-content/reference.md#schema-and-omissions) now also retains this component in `effect_parameters.jsonl` under `Buff_Fadia_ZhouYin`, with exact curve bindings, source-buff/effect aliases and the remaining-damage flag. Its selected parameters can be read without the installation or raw cache; native application semantics remain unresolved.

`GE_Player_Fadia_ZhouYin_Damage`, SHA-256 `d86609eea3f773e4675ad40395c134f7b45fca0bb90d499ccc4981323b720f8d`, `/2/Properties/Executions/0`, uses **`FadiaTransferDamageExecution`** with empty calculation-modifier/tag arrays. Its asset-tag component and combined tags contain `State.Damage` and `State.Damage.Skill`; no ATK/MaxHP damage formula, repeat count, crit, resistance/buff eligibility or extra resource grant is recovered from an empty modifier list. A recorded transfer amount must not be replaced with the ordinary Skill coefficient merely because both carry the Skill tag.

The search class `DamageRange_Fadia_Skill_SearchTarget`, SHA-256 `ecf8815b5201ef91f55ef2e4bd90f7bcc405b99bd211f5347f0288e7aec29dfa`, `/1/Properties`, supplies radius **440**, actor offset **(80,0,0)**, `MaxCount=10`, object-query entries 3/12, `bDamageInstanceObj=false` and a null exclusion-class entry. These are source coordinates/query categories; physical units, collision-channel mapping, ordering, exclusions and the native `ApplyFadiaZhouYin` boolean/recipient selection remain unresolved.

Separate Infinite carriers `Buff_Fadia_ShareOutTeammatesDamage` and `Buff_Fadia039_Level1` both add to **`ShareOutTeammatesDamageMul`**, with explicit scalar 1 and respective curves **0.50** / **0.25**. The first aggregates by source, the second by target; both have stack limit 1. `Buff_Fadia_NoDieShareDamage` adds literal **1** to **`ImmuneDeadByTeammates`**, with target aggregation/limit 1. These bind attribute destinations and values; they do not establish recipient admission, application lifetime, a universal damage reduction, one-HP floor, redirection recursion or death callback order. That carrier is distinct from the enemy-mark component above.

**Independent discriminating inputs:** if an explicitly modeled transfer capacity is `0.08 × declared MaxHP 10,000 = 800`, a 1,000-unit stored input and ratio 3 give **2,400** when capacity is applied before amplification, versus **800** when the amplified result is capped. The serialized fields alone do not choose either ordering; the description refers to damage taken by the marked target. Under the description-level awakening-2 model, explicit cap 2,500 minus already-taken damage 1,200 gives a residual **1,300**, not another sixfold amplification to 7,800. If both sharing carriers are explicitly admitted, their attribute additions sum to **0.75**; treating them as successive reductions would instead give `1 − (1 − 0.50) × (1 − 0.25) = 0.625`. These checks distinguish candidate models without certifying native settlement. Keep admission/settlement visible in [unknown](#unknown-execution-policies).

## Team MaxHP passive source inputs

**Source facts, not an integrated team buff.** `Buff_Fadia039_Passive2`, export SHA-256 `09e991d95a66c7953888f02281929eeb6f151f8fe7401ca5588d84968333231d`, `/1/Properties/Modifiers/0`, adds to **`HPMaxUp`** with an explicit scalar 1 and `DT_FadiaEffectFigure:FadiaHpUpPas2=0.10`. It is an Infinite carrier with target aggregation/limit 1, `DeathCanBeAdded=true` and an `GlobalGameplayEffectComponent`; that component has no serialized properties. The frozen Reference guide support description states team MaxHP +10% while Fadia is in the team. This supplies its destination/ratio and described scope, not native team membership, grant/withdraw, death or HP reconciliation semantics. No omitted component default is inferred.

Under the shared additive HP-percent model, explicit base HP 10,000, existing HPMaxUp 0.20 and flat HP 500 give MaxHP **12,500** before this carrier and **13,500** with it, not 13,750 from multiplying the already compiled total by 1.10. The independent **+1,000** change is conditional on admission of this carrier and does not establish how current HP is changed or whether a second Fadia can supply another contribution. The passive acquisition preserves exact sources; native application belongs in [unknown](#unknown-execution-policies).

`DT_CharacterAbilityEffectConfig` binds `GA_Fadia_Passive_1` / `_2` to their respective passive buff classes. Their `bReliveNeedAddAgain` values are explicitly **false / true**. This supplies a configured re-add distinction; actual grant/withdraw and revival dispatch remain native. The first passive carrier is Infinite, target-aggregated/limit 1, with `NeverRefresh`, `NeverReset` and an explicit `GlobalGameplayEffectComponent.CheckSourceDeath=false`. The second component's absence of Properties must not be substituted with that first carrier's explicit false value.

### Nova MaxHP extraction source chain

`DT_ReactionExtendDataTable`, row **`1039`**, binds `ReactionAbilityID=GA_Fadia_Passive_1`, a condition on `Buff_Fadia039_Passive1`, a null `SourceGEClass` and target class **`Buff_Reaction_4_new_1039_Passive1`**. This is the reached extension source, separate from the ordinary Nova damage carrier. The table's Chinese description speaks of extraction during Nova every second; frozen English/Reference guide text describes extraction when Nova ends. Preserve this source discrepancy and cutoff applicability.

The extension's `/2/Properties` declares **Infinite** duration, target aggregation/limit 1, `bExecutePeriodicEffectOnApplication=false`, a self-removal request on `BUFF_LEAVE_BATTLE`, and period scalar **1** with curve `DT_ReactionEffectFigure:Reaction_AnHun3_PeriodTime_1039=5`. Its serialized duration binding (`1.02 × Reaction4_BuffTime`) is inactive under Infinite policy. This does not establish a per-second clock, a single expiry trigger or a native extraction count.

Its `/3/Properties` is native **`ExtractAttributeGEComp`**, with type label **MAXHP** and condition tag **`Ability.Player.Buff.Reaction.AnHun`**. The public and native type labels agree at ordinal 1; native enum/layout evidence records the comparison. Four scalable fields name these `DT_ReactionEffectFigure` rows, each with a single coordinate-1 key and constant endpoints:

| Component field | Curve | Row value |
| --- | --- | ---: |
| `AddAttributeVale` | `Reaction_AnHun3_AddMaxHPVale_1039` | 0 |
| `SourceAddRatio` | `Reaction_AnHun3_AddMaxHPPer_1039` | 0.10 |
| `TargetLostRatio` | `Reaction_AnHun3_LoseMaxHPPer_1039` | **20** |
| `LimitRatio` | `Reaction_AnHun3_LimitMaxHPPer_1039` | 0.50 |

All four scalars are omitted in this component projection. The eight-field property schema matches the public map's names/order/count; its additional **`LimitTargetRatio`** and **`MMOLimitTargetRatio`** fields are also omitted in the export. Neither omission proves zero or one, and neither limit is supplied by guessing what a displayed percentage refers to.

The October 17 compact content revision retains these six serialized component fields under `Buff_Reaction_4_new_1039_Passive1`, with the four reached curves in `curves.jsonl`. The two absent target limits and all four absent multipliers stay absent. This resolves portable parameter availability, not the native extraction/default questions.

The frozen description states target MaxHP loss of 200% of Fadia's base MaxHP, ally gain of 10% of that base, a 50% stated total-drain cap and restoration on combat exit. It does not explain why the target-loss curve stores 20 or how the two additional target limits apply. With explicit base HP 10,000, `0.10 × base = 1,000`; multiplying that intermediate by 20 yields **20,000**, consistent numerically with the described 200%, while multiplying base directly by 20 yields **200,000**. This is a discriminating arithmetic example, **not a selected native formula**: omitted scalars or another native intermediate can also explain the factor. Do not replace the saved row value with 2, convert it automatically to 2,000%, apply the 0.50 limit to an invented owner, or decide the extraction clock from the text alone. Source/base actor capture, live versus base MaxHP, recipient distribution, caps across events/targets, current-HP reconciliation and reversal remain native dependencies.

### Skill resource callback source boundary

The Skill's owner-target `Event.Montage.Player.Display.1` binding reaches `Buff_Fadia039_AddEnergy`. Its `/2/Properties/Executions/0` uses **`HeterochromeExecution`** with source-snapshot additive `PParameter1` supplied by `Cau_Fadia039SkillAddCharge`. The calculator's `/2/ScriptBytecode` casts `OwnerActor` to player, evaluates `FadiaESkillAddEnergy` at coordinate **0** (102), and on result byte 0 calls **`AddHeterochromeValue`** with the returned curve value (223), then sets return value **0** (286). The curve has one constant key **0**. The local `HeterochromeValueParry=50` assignment at 0 is not passed to that call. This binds a named resource callback and zero configured argument; it does not establish a 50-unit grant, ordinary-hit Ultimate Energy, total Skill gains or free action admission. Native execution/recipients and the displayed Cycle Energy parameter remain separate.

## Ordinary-hit Ultimate Energy

The public shared-script Burst now consumes the source 120 energy from the
120-capacity owner pool. Rust rejects 100/119 without spending energy, emitting
packets or granting Eternal Waltz, and admits exactly 120. The continuing weapon
script funds both Bursts from zero through Normal packets and preserves 240
spent across two cycles; native/WASM/Worker compare its full reports and replay.
This replaces the historical generic 100-cost authoring fallback only for Fadia;
full-HP initial Burst applicability and its other disclosed owner limits remain.

The current `fadia.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 22.2 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipts: [`fadia-zankou-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/fadia-zankou-ordinary-hits.json), [`fadia-continuing-energy.json`](../../../../reviews/q10-energy-ledger/fadia-continuing-energy.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`fadia.skill_animation_profile_v1`, `fadia.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Fadia_Skill`), `AnimNotify_TriggerCooldown` is at 0.198s, gameplay events land at 0.216s (`Skill.4`), 0.729s (`Skill.1`), and 0.906s–0.908s (`Skill.3`, `Skill.2`); cancel windows activate at 0.977s (`BreakSkill_Jump`), 1.246s (`BreakSkill_AnySkill`), and 1.774s (`BreakSkill_Move`) with total duration 5.067s. For Burst (`Fadia_UltraSkill`), gameplay event `UltraSkill.1` lands at 2.707s, 4 hits of `UltraSkill.2` land at 3.101s–3.484s, and `UltraSkill.3` lands at 3.893s; cancel windows activate at 4.416s (`BreakSkill_AnySkill`, `BreakSkill_Jump`) and 4.450s (`BreakSkill_Move`) with total duration 8.0s.
- **Multi-target delivery (`fadia.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Fadia's initial GA_Fadia_UltraSkill full packet train (1x `GE_Player_Fadia_UltraSkill1_Damage`, 4x `GE_Player_Fadia_UltraSkill2_Damage`, 1x `GE_Player_Fadia_UltraSkill3_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/fadia-action-binding-model.json`](../../../../reviews/q10-game-action-binding/fadia-action-binding-model.json).
- Owner supplements: [`fadia-initial-burst-eternal-waltz.json`](../../../../reviews/q10-game-action-binding/fadia-initial-burst-eternal-waltz.json), [`fadia-normal-chain.json`](../../../../reviews/q10-game-action-binding/fadia-normal-chain.json).
- Frozen inputs: `data/extracted/neverness_fadia.json`, [consumed typed Fadia damage rows](../../../../crates/data/src/native_damage/fadia.rs) (`DT_SkillDamageData` raw source stays cache-only), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), `1039AwakenEffect` (cache-only awakening source), cache-only source table `DT_FadiaEffectFigure.json` (optional cached source `DT_FadiaEffectFigure`).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

The opt-in `fadia.ordinary_clock_v1` prerequisite executes the five ordinary Normal stages with exact piecewise-linear hitpause, packet-scoped resources and continuing chain state. Core integrates overlapping curves at their actual crossings and inverts analytic progress to integer microsecond deadlines; interruption retains the unexpired curve rather than replacing it with a flat loss. Two continuing chains independently check 16 packets, 6.087 ATK per chain, 22.2 energy and 37.001 Cycle, including three-target resource deduplication. This bounded prerequisite does not enable Skill, damaged-HP Burst, Lilith or the remaining passives. Runtime HP projection preserves coherent unrounded static HP%/flat composition before later temporary HP bonuses; explicit legacy overridden totals remain authoritative.

- Skill and damaged-health/awakening Burst variants, Lilith follow-ups, healing/redirect/survival, incoming-damage redistribution, plunge/riposte, and team passives are not integrated.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/fadia). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Fadia (`GA_Fadia_*`):

| Action / damage row (`GE_Player_Fadia_` prefix) | Native category | Scaling | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee8_Damage` | A | ATK | 0.273–0.803 / 0.468–1.376 / 0.505–1.486 | 1.703–5.022 / 2.839–8.370 / 0.338–0.995 per hit |
| Plunge `AirAttack_Damage` | X | ATK | 0.600 / 1.028 / 1.111 | 3.000 / 5.000 / 0.600 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | ATK | 1.780 / 3.051 / 3.295 | 4.500 / 7.400 / 2.500 |
| Support `QTE1_Damage` | H | ATK | 2.000 / 3.428 / 3.702 | 7.600 / 0.000 / 2.500 |
| Skill `Skill1_Damage`, `Skill2_Damage` | E | Max HP | 0.025 / 0.043 / 0.046 | 1.742 / 2.878 / 0.341 per hit |
| Skill `Skill3_Damage` | E | Max HP | 0.016 / 0.027 / 0.030 | 1.111 / 1.835 / 0.217 |
| Skill Mark `ZhouYin_Damage` | E | Max HP | Transfer / Special | 0.005 / 0.008 / 0.001 |
| Burst initial `UltraSkill1_Damage` | Q | Max HP | 0.039 / 0.067 / 0.072 | 0.000 / 0.000 / 0.643 |
| Burst mid `UltraSkill2_Damage` | Q | Max HP | 0.012 / 0.021 / 0.022 | 0.000 / 0.000 / 0.203 (4 hits) |
| Burst finisher `UltraSkill3_Damage` | Q | Max HP | 0.063 / 0.108 / 0.117 | 0.000 / 0.000 / 1.045 |
| Lilith Melee `UltraSkillMelee1..8_Damage` | Q | Max HP | 0.009–0.059 / 0.015–0.101 / 0.017–0.109 | 0.501–3.200 / 0.832–5.317 / 0.101–0.643 per hit |

Critical Riposte uses GAName `GA_Fadia_ExtremEvadeAtk`, while Support QTE uses `GA_Fadia_QTE`. All damage rows deal PSYCHE damage. Noticeably, Normal chain, Plunge, Riposte and QTE scale from ATK (`AtkRateBaseArray`), whereas Skill and Burst (both initial Eternal Waltz and transformed Lilith mode melee attacks) strictly scale from Max HP via `HPRateBaseArray`.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Fadia_Passive_1` binds `Buff_Fadia039_Passive1`. `DT_ReactionExtendDataTable` row 1039 binds `ReactionAbilityID=GA_Fadia_Passive_1` with target class `Buff_Reaction_4_new_1039_Passive1` carrying `ExtractAttributeGEComp` (Nova / AnHun reaction 4 attribute extraction).
- **Passive 2 (Breakthrough 4):** `GA_Fadia_Passive_2` binds `Buff_Fadia039_Passive2`, adding +10% Max HP (`FadiaHpUpPas2 = 0.10`) to the entire team while Fadia is in the party.

### Awakenings

Source descriptions and selected IDs: `1039AwakenEffect#/rows/1039/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Fadia's damage redirect ratio is increased to 75% (`ShareOutTeammatesDamageMul + 0.25`).
- **A2:** Destructive Experience (Skill) damage ratio enemies take increased to 600% (`FadiaESkillTransRatioLv2 = 6`), cap raised to 25% of Fadia's Max HP (`FadiaESkillMaxHPRatioLv2 = 0.25`). Deals immediate damage equal to cap minus already taken damage when Destructive Experience is removed (`bApplyRemainingDamageOnRemove = true`).
- **A3:** Upon entering combat, Fadia increases her base Max HP by 30%.
- **A4:** Increases Fadia's healing ratio to 30% of Max HP while in Lilith form; further increases healing ratio to 60% of Max HP when lost HP is >= Destructive Threshold.
- **A5:** Increases Fadia's CRIT Rate by +50% while in Lilith state.
- **A6:** Destructive Experience inflicts on up to 10 targets within range; duration and cap calculated individually.
- **Resonance 3:** Increases the skill level of Wordless Rejection (Normal), Existence (Skill), and Agony to Euphoria (Burst) by +1.
- **Resonance 6:** Grants all team members +10% ATK while Fadia is on the team.

### Lilith exit requests and native logic boundary, October 18

The expansion's **Player_039_Fadia#/3/Properties** locates `FadiaLogicObj`. `RemoveUltraMeleeOnActivate` explicitly lists **GA_Fadia_Appear, GA_Fadia_Melee, GA_Fadia_QTE and GA_Fadia_Skill**. It separately binds `ChangeStateBuff=Buff_Fadia_ChangeState`, `GiveUltraMeleeBuff=Buff_Fadia_GiveUltraMelee`, `UltraSkillDamUpBuff=Buff_Fadia_HighQ_DamUp`, `RecoverEnergyGE=GE_Fadia_Q_RecoverEnergy`, and the ordinary/UltraMelee ability classes. This identifies the configured form-exit action set and markers; it does not prove that every field exit, expiry or other input removes the form, nor define pending-hit cleanup.

The same object binds `MaxHPEachAdd` to `FadiaHPAddTempLv3` with explicit multiplier 1, `AddMaxHPRequiredBuff` to `Buff_Fadia039_Level3`, and `InherentMaxHPRatio` to `FadiaHPMaxRatioLv3` **without an explicit multiplier**. Preserve that distinction rather than inventing effective stat growth from the raw ratio row. The player export has one parser warning; this is scoped observed configuration. The existing mark-transfer, threshold/heal and extraction graphs already reach `ApplyFadiaZhouYin`, `FadiaTransferDamageExecution`, `GetFadiaQLimitHp`, and `ExtractAttributeGEComp`; those native consumers still own the unresolved settlement/default/clock decisions. Re-exporting the same player or rerunning synthetic model tests will not answer them.

### Form marker A5 lifetime and weapon cleanup recovered, October 18

Warning-free **Buff_Fadia_ChangeState#/1** is Infinite, with BEGIN requesting **Buff_Fadia039_Level5CritUp** under **Con_Fadia_Lv5**, and FINISH removing it. The condition **#/2** checks **InstigatorActor ASC for Level5** on its valid path; it does not inspect the target or independently require an on-field actor. The child **#/1** is Infinite, target cap1 and additive **CritBase=FadiaCritUpLv5=0.5**, multiplier1. This binds the described 50-percentage-point A5 gain to the form-marker lifecycle rather than a timed 120-second form or a second multiplier. Existing Crit .1 plus an admitted .5 term gives .6 before separate clamping/crit eligibility. Native grant/removal callbacks and actual context still govern the effective window.

The marker joins **GC_Fadia_ChangeState**. OnWhile **#/4 → graph#/2:492** casts MyTarget to player, sets **bCanTransitionNewFightLeisure=true**, requests current weapon **SetCustomHide(20,true)** and, when a LevelSequenceActor is valid, binds its sequence player's **OnStop**. That callback reaches a next-tick dissolve-actor spawn/replay; entry does not directly dispatch a new damage ability. OnRemove **#/3 → 1000** handles the effect actor, then reaches player cleanup even when that actor is invalid. It requests weapon unhide type20 and clears bCanTransitionNewFightLeisure via stored execution-flow continuations.

The optional pack-up path additionally requires **WeaponHoldState==2 AND NOT IsUsingAnyMontageAbility()**. It requests **CheckEquipWeapon(false,true,false,false)** only under **IsClientOrStandalone**; the separate continuation tests **NOT IsInCharacterState(17)** before **PlayAnimMontageEx(char_fadia_skill_packup_Montage,true,1,NoWeapon,false)**. That latter path is not enclosed by the client test. Native parameter metadata names CheckEquipWeapon's inputs **bHoldOnHand=false, bImmediately=true, bIgonreNotInPeace=false, bForceState=false** (native spelling preserved), and this PlayAnimMontageEx overload's inputs **Anim, bReplicated=true, InPlayRate=1, StartSectionName=NoWeapon, SelfClientNeedPlay=false**, with separate float ReturnValue. Its true is not a recovered “stop all active abilities” flag, and its play rate is not a one-second recovery deadline. Weapon/form reconciliation, input availability, successful playback and pending packets remain native dependencies.

Sources: `cue-form-weapon-wrapper-functions.txt`, `fadia-form-crit-carrier.txt`, `fadia-form-crit-condition.txt`, `native-weapon-wrapper-function-metadata.json`, `roster-cue-halo-extraction.json`, `cue-supplemental-joins.json` and **DT_FadiaEffectFigure:FadiaCritUpLv5**. All cited carrier/condition/cue sources are warning-free; the separate dissolve helper's material-body errors do not supply a combat executor.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Fadia_Skill`) and Burst (`Fadia_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **MaxHP passive consumers & Nova extraction:** Resolve the [MaxHP passive consumers](#team-maxhp-passive-source-inputs): native grant/withdraw/revival and current-HP reconciliation for the team buff; extension admission, `ExtractAttributeGEComp` effective scalar/target-limit defaults, base/stat owner capture, cap aggregation and reversal for Nova. The reaction row, target carrier, field/curve values, type ordinal and configured revival flags are located. Reconcile the target-loss row's 20 with the described 200%, the Infinite periodic carrier with the described end trigger, and the table description's per-second wording.
- **Damage redirection and transfer execution:** Resolve `ApplyFadiaZhouYin`, `FadiaTransferDamageGEComp` and `FadiaTransferDamageExecution` admission/capture/settlement: target tie/order rules, effective alternate scalars, cap/multiplication order and MaxHP owner/read time, refresh/accumulated state, removal-reason flush and transfer packet crit/bonus/resource eligibility.
- **Threshold gating & Lilith state transitions:** Establish `GetFadiaQLimitHp` lookup coordinate/scaling, actual HP/ability callback binding/delivery, and native ChargeExecution parameter mutation/cap behavior. Reconcile the saved strict Burst comparison with the description's equality-inclusive wording at the frozen cutoff. Resolve effective enhanced-heal child struct/delta construction, native healing bonuses/recipient/clamp/rounding, mode classification and duration clocks/refresh.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Resolve the [MaxHP passive consumers](#team-maxhp-passive-source-inputs): native grant/withdraw/revival and current-HP reconciliation for the team buff; extension admission, `ExtractAttributeGEComp` effective scalar/target-limit defaults, base/stat owner capture, cap aggregation and reversal for Nova. The reaction row, target carrier, field/curve values, type ordinal and configured revival flags are located. Reconcile the target-loss row's 20 with the described 200%, the Infinite periodic carrier with the described end trigger, and the table description's per-second wording. Do not silently pick a scaling explanation or cap owner. Target MaxHP removal must not become an ordinary damage receipt or resource callback without evidence.
- Resolve `ApplyFadiaZhouYin`, `FadiaTransferDamageGEComp` and `FadiaTransferDamageExecution` admission/capture/settlement: target tie/order rules, effective alternate scalars, cap/multiplication order and MaxHP owner/read time, refresh/accumulated state, removal-reason flush and transfer packet crit/bonus/resource eligibility. The [transfer/survival source owner](#skill-mark-transfer-and-teammate-damage-source-inputs) now binds mark/search fields, ratios/caps, separate sharing/no-death attribute carriers and a native eight-field layout comparison. Published descriptions already state highest-MaxHP selection, pre-shield sharing, no further redistribution, the 1-HP branch and awakening-2 removal difference; these stated policies are not missing. Native distribution across living allies, protection at a threshold-crossing hit, shield/HP event order and application lifetime remain unresolved.
- Remaining action timing/cost and heal-recipient policies stay open. The [known threshold findings](#recovered-threshold-and-damaged-hp-branches) bind the global curve, reached Burst heal/buff selection and HP-crossing energy request. The [healing/Lilith chain](#burst-healing-enhancement-and-lilith-source-chain) additionally binds the ordinary custom-heal expression, level-four addition, effect-tag damage-bonus predicate, mode-selected duration rows and section-based removal requests. Those source locations/expressions are not missing.
- Establish `GetFadiaQLimitHp` lookup coordinate/scaling, actual HP/ability callback binding/delivery, and native ChargeExecution parameter mutation/cap behavior. Reconcile the saved strict Burst comparison with the description's equality-inclusive wording at the frozen cutoff; keep its boundary distinct from the HP-crossing predicate. Resolve effective enhanced-heal child struct/delta construction, native healing bonuses/recipient/clamp/rounding, mode classification and duration clocks/refresh. Native conditional-modifier capture, `FadiaLogicObj`, combo progression/packet multiplicity, cleanup and continuing-rotation ordering remain open. Do not substitute the inactive 10/120 duration scalars for the reached duration policy or treat section five as five landed packets.
- Ordinary-hit energy is only tested for the modeled Normal chain; it does not imply grants for unsupported forms/Skill/DoT.
- The [Skill resource callback](#skill-resource-callback-source-boundary) now has a reached `AddHeterochromeValue` argument and calculator return path. Native Heterochrome execution/capture/recipient behavior, actual montage-event delivery and any other Skill resource grants remain open; its zero configured curve and unused local 50 do not settle action admission or total Ultimate/Cycle Energy.

## Prior findings

- [`reviews/q10-game-action-binding/fadia-action-binding-model.json`](../../../../reviews/q10-game-action-binding/fadia-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [`fadia-initial-burst-eternal-waltz.json`](../../../../reviews/q10-game-action-binding/fadia-initial-burst-eternal-waltz.json), [`fadia-normal-chain.json`](../../../../reviews/q10-game-action-binding/fadia-normal-chain.json).
- The transfer investigation's exact-literal search across 21,904 saved JSON paths found the base-sharing/no-death class names only in their own exports and `DT_GameplayEffectMappingData`. It did not recover an activation caller. Mapping presence is not buff admission; native constructors or numeric/indirect lookups remain possible. Resume at a source-bound native admission/attribute consumer or an explicitly declared description-level model, rather than repeating the same literal scan. Acquisition retains the scan identity and source boundaries.

## Research history

- **Accepted:**8/8 selected model; native/public approaches and independent arithmetic already used. Historical body/default unknowns are accuracy notes, not research blockers.
- **Next:** Haniel's first unchecked block, reusing its source/actor/reaction findings. Keep Fadia's full selected rules for implementation and defer optional native-fidelity polishing.

## Relative combat timer binding

`Fadia_UltraSkill` Common.EndAbilitySkill at 4.417366 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
