# Hotori action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1052`.

**Responsibility:** Hotori outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete eight-block research specification below, including labeled checked inferences; current executable slice remains partial. Research readiness is not an engine-completeness or native-observation claim.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected complete packet specification

**Mixed source/inference, October19:** this campaign model supersedes historical coincident-packet/runtime summaries. Use `DT_SkillDamageData`, `DT_JinEffectFigure`, original/supplement `GA_Jin_*` and `Jin_*` montages, actor/cue/logic findings in this card and [Time Stop subsystem](time-stop-resource-and-cooldown.md), and permanent owner inputs (optional `hotori.json`). The [current kit mirror](https://www.neverness.app/characters/jin) corroborates the five Katana stages/ten-stack finisher; existing native/public approaches already cover body/dispatch questions, so remaining rules are explicit checked choices. All own damage is **Cosmos/liveATK**, ordinary crit/DEF/RES and eligible category/general/Cosmos bonuses. Preserve full13-level arrays; samples below are **levels1 /8 /9**, effective talent minus one. Resource columns are raw energy/Cycle/Break per positive packet, once irrespective of target count; they are **not Timepiece energy**.

| Suffix after `GE_Player_Jin_` | Count/category | Coefficient1 /8 /9 | Energy / Cycle / Break |
| --- | --- | --- | --- |
| Melee1_Damage |1 Normal1 /A |.200 /.343 /.370 |1.251 /2.087 /.252 |
| Melee2_Damage |1 Normal2 /A |.353 /.605 /.653 |2.206 /3.680 /.445 |
| Melee3_Damage |2 Normal2 /A |.044 /.075 /.081 |.274 /.457 /.055 each |
| Melee9_Damage |1 Normal2 /A |.722 /1.237 /1.336 |4.508 /7.521 /.910 |
| Melee4_Damage |5 Normal3 /A |.113 /.194 /.209 |.703 /1.173 /.142 each |
| Melee5_Damage |2 spawned impacts Normal4 /A |.142 /.243 /.263 |.889 /1.484 /.179 each |
| Melee6_Damage |1 spawned impact Normal4 /A |.666 /1.141 /1.233 |4.160 /6.940 /.839 |
| Melee7_Damage |1 Normal5 /A |.243 /.416 /.450 |1.519 /2.534 /.307 |
| Melee8_Damage |1 Normal5 /A |.451 /.773 /.835 |2.814 /4.694 /.568 |
| Branch1_Damage; Branch2_Damage |2 each over full held wave /A |.240 /.411 /.444 each |1.5 /2.5 /.3 each |
| AirAttack_Damage |1 landing /X |.480 /.823 /.888 |3 /5 /.6 |
| PerfectEvadeAttack_Damage |1 /Z |1.580 /2.708 /2.924 |4 /6.6 /2.5 |
| QTE_Damage |1 /H |2 /3.428 /3.702 |9.5 /0 /2.5 |
| Skill1_Damage |**4** directSkill /E |.400 /.686 /.740 |.947 /1.574 /.187 each |
| Skill3_Damage |**2** directSkill /E |.400 /.686 /.740 |.946 /1.572 /.186 each |
| Skill2_Damage |1 finalSkill /E |.601 /1.030 /1.112 |1.421 /2.362 /.280 |
| UltraSkill1_Damage, UltraSkill1_1..5_Damage |1 of each (6) initialBurst /Q |.170 /.291 /.315 |0 /0 /.032 each |
| UltraSkill2_Damage |1 initialBurst /Q |1.303 /2.233 /2.412 |0 /0 /.244 |
| UltraSkill4_Damage |1 initialBurst /Q |.443 /.759 /.820 |0 /0 /.083 |
| UltraSkill3_Damage |2 initialBurst /Q |2.616 /4.483 /4.842 |0 /0 /.490 each |
| UltraSkillMelee1_Damage |1 Katana1 /Q |.381 /.653 /.705 |0 /0 /0 |
| UltraSkillMelee2_Damage |1 Katana2 /Q |.234 /.401 /.433 |0 /0 /0 |
| UltraSkillMelee2_1_Damage |1 Katana2 /Q |.637 /1.092 /1.179 |0 /0 /0 |
| UltraSkillMelee3_Damage |3 Katana3 /Q |.092 /.158 /.170 |0 /0 /0 |
| UltraSkillMelee3_1_Damage |1 Katana3 /Q |.445 /.763 /.824 |0 /0 /0 |
| UltraSkillMelee4_Damage |1 Katana4 /Q |.255 /.437 /.472 |0 /0 /0 |
| UltraSkillMelee4_1_Damage |1 Katana4 /Q |.297 /.509 /.550 |0 /0 /0 |
| UltraSkillMelee4_2_Damage |1 Katana4 /Q |.254 /.435 /.470 |0 /0 /0 |
| UltraSkillMelee4_3_Damage |1 Katana4 /Q |.424 /.727 /.785 |0 /0 /0 |
| UltraSkillMelee5_Damage |1 Katana5 /Q |.085 /.146 /.157 |0 /0 /0 |
| UltraSkillMelee5_1_Damage |1 Katana5 /Q |2.211 /3.789 /4.092 |0 /0 /0 |
| UltraSkillMelee6_Damage |1 finisher **without B4** /Q |3 /5.141 /5.553 |0 /0 /0 |
| UltraSkillMelee6Pas2_Damage |1 finisher **with B4**, replaces prior /Q |6 /10.283 /11.106 |0 /0 /0 |
| UltraAirAttack_Damage |1 Time Stop landing /X |.480 /.823 /.888 |3 /5 /.6 |

**Correct direct Skill binding:** source `Skill.3` at.18195254/.43722448 maps to **Skill3**, and `Skill.1` at.29164106/.36428303/.49373364/.5782571 maps **Skill1**. Thus count4Skill1+2Skill3+1Skill2, rather than historical1+5+1. Same coefficient mask conceals different resources: level8 total **5.146ATK /7.101energy /11.802Cycle /1.400Break**, not7.098energy. InitialBurst10 packets total**13.704ATK /1.499Break**, zero raw energy/Cycle. Ordinary Normal15 packets total**6.121ATK /22.299energy /37.203Cycle /4.499Break**. Normal4 uses two Melee5 actors and one Melee6 actor, each first damage at source period+.2 (cap1/TriggerOnSpawn=false), not instant extra direct impacts or looping DoT.

Katana five-stage sequence has **13 packets**, stage sums at8 **.653 /1.493 /1.237 /2.108 /3.935**, full **9.426ATK**, zero raw energy/Cycle/Break. Preserve its rows rather than ordinary Normal's arrays/category. `Melee.4_4` is an alternate source tag for the same last4_3 packet, not an extra fifth Katana4 hit; the selected montage emits4_4 once. Two complete five-input Katana chains therefore give26 packets/**18.852ATK**; next finisher adds one selected coefficient. At B4 total27 packets/**29.135ATK**, not both ordinary5.141 and empowered10.283. Complementary native passive predicates choose exactly one. Stored source coefficient10.283 differs from simply2*5.141=10.282 because source arrays are independently rounded; use the actual empowered row. B4 is coefficient replacement, not general+100% damage.

CountSkill notifies occur **once per admitted Katana input**, independently of each input's1/2/4/4/2 damage packets. Increment only at that input's actual count marker, cap10; a miss can still complete its count marker. After count10, let the already admitted stage finish, then next legal Basic requests finisher instead of another ordinary stage. Counts11 are unreachable by legal inputs; do not silently change recovered `==10` into a broad>= test. Finisher has its one actual hit, then clear event; no automatic full train merely from entering Time Stop. A cancelled before-count input does not earn a stack. Later timing/state blocks specify admission, remaining time and cleanup.

Full held wave is4 packets **1.644ATK /6energy /10Cycle /1.2Break** at8; records/control time-slow events add no HP packet. Ordinary and Time Stop plunge use their separate rows/category and multiply coefficient by1+h, with scenario fall bonus h in[0,1] and real authored landing. No universal healing packet is added from the native candidate `Buff_Jin_RecoverHP`: both bound heal-ratio rows are0 and the selected public kit describes no healing branch; the located MaxHP helper alone is not a full heal.

**Recorded-action replay, selected generic packet contract:** a stored record contains the allied real caster ID/build/talent/awakening and committed action/branch descriptor. At playback dispatch that same complete Support or Redirect action **once** via an off-field virtual cast context, source-owned by the recorded ally, using its normal packet classes/elements/stat bases/hit schedules/procs. Do not replace all allies by a Hotori Cosmos coefficient or a single copied final-damage number. Playback uses live actual owner stats/party buffs/target state at each delivery; record captures the action choice/talent, not a stale enemy HP or already-calculated crit. Virtual admission bypasses its original field/cooldown/cost/one-shot branch permission (already paid by the recorded original), so replay does not spend the actual ally's resources or reset its real cooldown. Its actual emitted packets retain ordinary positive-hit energy/Cycle awards and normal source effects; no arbitrary zero-resource/damage-only copy is inserted. Source effects use the real owner identity, so same-source summons/auras obey their existing replace/refresh/cap rules, never a duplicate infinite aura because of a proxy ID.

Replay is marked nonrecordable and dispatched at most once per stored entry; copied Support does not perform another party switch/outgoingCycle payment or independently create a switch reaction. It still emits one semantic Support cast for actual owner/passive listeners, and Redirect emits one semantic cast, with normal nonrecursive source guards. Internal proxy-only cooldown may use the located copy+35s branch, but cannot lock the real player's ordinary Skill. Those semantics are explicit inference from “replays recorded attacks once”, source copy distinction and conservation: a two-hit Mint Skill replays two **Mint Anima** hits with Mint talents/gear and no second real Skill payment, not two Hotori hits or the original damage multiplied again by Hotori's ATK. Recording/replay clocks, slot cap and packet ordering are finalized in the Time Stop subsystem; every recorded owner uses its already selected full dossier, not new research at implementation.

## Selected complete timing specification

**Mixed native coordinates and checked clock inference, October19.** Original `Jin_MeleePre`, `Jin_Melee_001..005`, `Jin_Branch1..3`, `Jin_Skill`, `Jin_UltraSkill`, `Jin_QTE`, `Jin_PerfectEvadeAtk`, `Jin_AirAttackEnd`, supplement `Jin_UltraMelee1Pre/1..6` and `Jin_UltraAirAttackEnd` provide the coordinates below. Use montage rate1 and the actual positive-hit source pause curves: integrate `loss = duration - integral(rate)` once per source hit, regardless of dummy count. Subsequent source montage coordinates shift by that loss; world time, other casters, cooldowns and buff expiry continue. Spawned Normal4 impacts have independent world+0.2 first-hit delays; after delivery their caster pause shifts still-pending caster montage events. Missing pause data means no configured caster dilation, not an invented global frame stop. This is the selected real-time model, rather than treating montage coordinates as observed seconds.

| Ordinary stage | Local source hit coordinates (row suffix) | Queued next Basic coordinate | Any / Skill / Jump / Move cancel coordinates |
| --- | --- | --- | --- |
| Entry1 `MeleePre` | .2824549 (1) | .16457832+9/60 | .54570365 / .43530798 / .30735475 /1.0666401 |
| Loop1 `Melee_001` | .23312181 (1) | .0984512+12/60 | .5316814 / .3962585 / .26278982 /1.0671468 |
|2 | .16594519 (2), .46557027/.5822505 (3), .6810825 (9) | .09820092+57/60 |1.4123749 /1.0259542 /.19712211 /1.5834965 |
|3 | .21691068/.30076867/.3843495/.46571136/.5509628 (4) | .09839369+40/60 |1.8513721 /.85060865 /.25220072 /2.9365225 |
|4 | actor spawn .11666667/.21666667 (5), .5692651 (6); hit each spawn+.2 world | .163863+58/60 |3.4606373 /1.2656903 /.6617818 /3.830398 |
|5 | .3155617 (7), .5664244 (8) | .74953455, then Loop1 | .74953455 / same /.33947802 /1.8169289 |

`BP_JumpSection.nFrame` is modeled as a **60fps delay from its notify start**, not an absolute montage frame. It admits only the queued next Basic at that transition; it does not enable arbitrary earlier Skill. Entry stage is used after chain reset; no artificial two-second input-start reset interrupts a still-running native stage. Leaving field or admitting another ability resets the ordinary chain. After the final allowed window/end, idle next Basic uses Entry1. Cancel before an unissued montage event removes that event; already spawned actors persist and keep their source identity. Native actor contacts occur once per target, no period spam. Full-chain source losses are .0594 / .07425 /0 /.1196 /.09: first two are .0396+2*.0099+.01485 for stage2; stage4 is2*.0198+.08. Actor/caster scheduling, not an instant three-hit cluster, determines stage4 timing.

**Skill:** begin recording at .16747499; cooldown notify .16792336; source hits at .18195254 (Skill3), .29164106/.36428303 (Skill1), .43722448 (Skill3), .49373364/.5782571 (Skill1), .86366606 (Skill2). Skill3 pause loss is .046 each: its native curve knots (0,.5),(.01,0),(.045,0),(.052,1), duration.06 integrate area.014. Final Skill2 loss.1; other Skill hits0. Thus real hits .18195254/.33764106/.41028303/.48322448/.58573364/.6702571/.95566606, and final Any recovery **1.4086605**, Jump1.1930471, Move1.4116817. Real-player Skill cooldown **0**, confirmed by real/copy branch and published no-cooldown rule; no nominal serialized1s or copy35s imposed. Interrupt before cooldown/record marker uses existing cancellation penalty3s; after commitment there is no refund of the reserved resource, and no artificial cancel penalty on a completed Skill.

**Initial Burst:** EnterTimeStop+2.1833334; six Ultra1-family hits+2.31608/2.416466/2.516466/2.6164658/2.716466/2.8166666, Ultra2+2.9, Ultra4+3.9275486, two Ultra3+3.9719899/4.002235. These rows have zero source pause duration. Replay dispatch+4.216431, Any/Move/Jump recovery+4.6042376; native initial switch lock+4.7833333 is extended by the active Time Stop prohibition. Burst real cooldown **0** and conventional Ultimate cost **0**; Timepiece admission and reservation remain mandatory. A generic24s effect carrier is not a Time Stop timer.

| Katana input | Local hits | Count marker | Queued next input before source pauses |
| --- | --- | --- | --- |
|1 loop | .18343106 | .19981226 | .09591392+13/60 |
|1 entry alternative | .23273766 | .2491188 | .14522064+14/60 |
|2 | .15810588/.3330929 | .18316573 | .092007264+22/60 |
|3 | .19980189/.26777712/.3328993/.54983455 | .22543706 | .09848472+38/60 |
|4 | .083743855/.2667762/.4902451/.6328422 | .09951269 | .165074+33/60 |
|5 | .21267165/.27257898 | .24031061 | Any .4222042 |
| Finisher |1.4578571 | no additional count | Any2.1198936; ClearTimeStop1.7351792 |

Selected Burst→Katana starts loop1 (entry alternative only where explicitly selected by an appearance/root); stages1..5 source-pause totals **.012/.024/.060/.100/.050**. Katana4's last source tag4_4 binds the **4_3** damage class, so use its.020 pause, not the unrelated unbound `UltraSkillMelee4_4_Damage` class's .0396. Katana1..4 Any recoveries1.0179805/1.0111797/1.4677055/1.342477; Jump.31745914/.4500885/.6666824/1.341549, Move1.0189998/1.0202198/1.4722255/1.6829026. Add only pauses already delivered by each requested event. Stage5 Jump.3894076, Move1.5010318. Finisher source-pause.01683; clear actual+**1.7520092**, Any+2.1367236, Jump2.141399, Move2.3731506, switch lock2.160101 including source pause. Finisher damage ends the combat Time Stop benefits at its hit; the later clear marker is idempotent cleanup, and the remaining recovery still occupies real time. This follows the published damage-end rule and avoids granting another .277s bonus after the finisher. Expiry before an already admitted attack finishes removes live Time Stop bonuses but lets its already admitted montage finish; no subsequent Katana input is admitted. Katana count is independent of its hit count and misses.

**Held Basic and movement branches:** held ordinary input first uses Entry1; held admission at its native chain transition selects Branch1 instead of next stage, with four Branch hits .082995474/.10800783/.13256061/.15783952 from branch origin. Branch1-class hit loss.035 twice, Branch2-class0; serialize these caster pauses in the actual clock. Branch1 Jump.20705983, queued Normal2 .25209746+30/60, Skill.73723465, Any.886134, Move1.067147. Continue holding selects nondamaging Branch2, sends TimeSlomo at+1.4329981; authored release selects Branch3, sends TimeSlomoEnd+1.5670341 (or removes immediately if cancelled before that event). Branch2 Any/Jump2.4251595, Move2.4581947; Branch3 Any/Jump.51987135, Move1.6165766. Explicit scenario hold/release timestamps supply the duration; do not invent an extra attack or fixed time-control lifetime. Timepiece drain only while control is active. Combat-free Frozen Snow is a non-DPS traversal state and clears on entering combat; it cannot carry a free combat finisher.

**Other actions:** ordinary plunge landing hits+.032851, loss.0594; Jump.14920567, Any.28124887, Move.750726, shifted by that hit. Time Stop plunge loss.1485 and Jump/Any.195412, Move.750726. Authored falling/landing time determines the real descent; air montage loops at.33333334 and yields no extra landing hits. Riposte is admitted only after authored Critical Dodge success: one Z hit+.250314, loss.297, Any/Move+.874201+.297=1.171201; the event at0 is control/setup rather than a second Z packet. Its queued Normal branch at .163402+20/60+.01 cannot bypass this hit's pending source pause. Support hit+.75, loss.32, Any/Jump1.6400457, Move2.577786, switch lock1.6036151. Ordinary dodge has the shared declared stamina cost15 and.9s cooldown; success windows are scenario inputs, not invented invulnerability hits. Cancel/field exit removes unissued ordinary montage hits, retains launched contacts/replay payloads, and never rewinds world time.

**Consistency checks:** Skill's two .046 pauses and final.1 produce recovery1.2166605+.192=1.4086605; adding each pause to the same triggering hit would incorrectly move its own contact. A queued loop Katana train ends in2.886350770666667s; two trains followed by finisher hit take7.230558641333334s, plus initial Burst recovery4.6042376 =11.834796241333334s from Burst commit, before the resource deadline14.1833334. All ten count markers and all26 preliminary Katana hits precede the finisher. A50-dummy AoE adds one caster pause per packet, not50 pauses. No native body/video re-search is required for this chosen clock model; source-coordinate and real-game timing remain distinct evidence levels.

## Selected complete state specification

The normative [Timepiece transaction/clock specification](time-stop-resource-and-cooldown.md#selected-implementation-transaction-and-clock-specification) defines initial/cap120, passive6/s, strict>60 Skill admission/payment,120 Burst reservation, independent rewind clocks, recording slots/overwrite, all live effects and cleanup; its passive matrix covers B2/B4, A1..6 and count-based R3/R6. It also specifies the concrete checked A3 rate6/s inference, source/copy cooldown separation, ordinary uncontrolled branch20/s, ten-input Katana/finisher admission and off-field source continuation. Use those rules with the complete packet/timing tables above, rather than historical executable partial models below. The two complementary finisher predicates select one packet, not two. Native heal helper0 rows create no hidden max-HP heal. No autonomous damage summon is added from visual ghosts; their captured allied actions use the recorded actual caster and original full kit. All temporary state and pending events carry source/action generations to make death/expiry/clear idempotent. The exact-equality count10 route cannot advance to11 through legal inputs; cancelled pre-count actions do not grant a count. City-only traits do not affect this combat model.

## Selected complete recipient specification

**Mixed source and dummy-geometry inference:** native `DamageRange_Jin_*` recipient objects, the public area descriptions and actor/cue maps identify area packets; no selected configured numeric enemy cap or split-damage budget is supplied. Under the declared stationary/all-in-range scenario each actual Hotori Normal/held/Skill/Burst/Katana/finisher/Support/Riposte/plunge packet hits **every living eligible enemy once for the full coefficient**, in stable target-index order, not a divided coefficient or one random recipient. Different range assets do not imply equal physical geometry outside that scenario; explicit action/range eligibility may narrow recipients. Already dead/ineligible targets cannot receive HP damage, resource credit or duplicate death callbacks. A selected primary target is retained if living/eligible, otherwise lowest living target index, for facing/actor anchor; this does not reduce an uncapped area to one recipient. Spawned Normal4 actors each have one-contact-per-target state and one configured first contact, not recurring periodic damage. Replayed ally payloads use **their original owner's** caps, mark selection, bounce/lowest-HP rules and allocation; Hotori's uncapped area rule cannot make a copied Fadia mark uncapped or a Haniel single-target cannon hit every dummy.

One positive HP-loss packet gives its ordinary energy/Cycle/Break once, regardless of area recipients; target-specific Break applies once to each actual target's own gauge, and each target's single death transition remains distinct for A4. Reaction source/selection and all dependencies use [cycles/Break](../../shared/cycles-and-break.md), [energy/gauges](../../shared/energy-and-gauges.md), [damage/stat zones](../../shared/damage-and-stat-zones.md) and [equipment builds](../../shared/equipment-builds.md). Actual qualified switch requires outgoing100Cycle and adjacent elements; HotoriCosmos→MintAnima creates Blossom only when that payment is possible. Ordinary switches at<100 do not produce a reaction merely because the elements match. Copied Support does not pay an outgoing pool or independently create another switch reaction. B2 changes only Bud clocks during Time Stop, as defined in the state specification. Remora/Charge remain separate status/proc dependencies; Timepiece is never ChargeAdd or ChizGrains.

**Recipient-dependent buffs:** A1/A2/A6 are source Hotori only, never copied ally modifiers; A5 counts the current full living-enemy set and benefits the actual party damage actor only against the sole remaining target; R6 recipient baseATK is its own. R3 affects N/E/Q talents only. Shared DEF-ignore additions include Lost Radiance .25 plus A6 .30 plus any eligible weapon .12, with separate DEF-shred zone, rather than multiplying ignore percentages. Party/team passive death cleanup and off-field persistence follow the linked state matrix. Healing/shields of a copied action use its real owner rules, not Hotori MaxHP or the visual proxy's empty stats. Damage ownership, buff source and true action type must survive proxy dispatch to retain ally equipment/resource/awakening triggers.

**Counterchecks:** two eligible dummies take two full Skill trains of5.146ATK each but raw energy remains7.101, not14.202. Two dead targets can grant two eligible A4 kill callbacks, while one overkilled target cannot grant seven. A recorded two-hit Mint Skill remains two Anima packets even while Hotori holds the katana; it does not use Cosmos RES, Hotori ATK or her A6. Shared DEF1080 againstLv80 with ignore.55 yields1080/(1080+486), not1/2*1.55. The one-dummy baseline does not need fabricated incoming AI or a Break bar; scenarios that enable them must supply the shared explicit profile.

## Complete implementation handoff and two continuing cycles

**Mixed, independent Fraction model check, October19.** Implement the packet/timing/state/recipient specifications above as one source-owned event system. Required runtime state: separate conventional energy/Cycle and Timepiece accounts; reserved rewind budgets and exact regeneration thresholds; recording generation/deadline/distinct-owner records; frozen A1 count; ordinary/Katana chain state and per-input count markers; actual versus replay action context; per-source aura/summon caps; source pause intervals; live modifier expiry/death cleanup; authored switches and world-time DPS horizon. A readiness query has no side effects. Reject illegal input with diagnostics before payment, and keep authored waits. Replay uses the **complete recorded owner's** accepted specification. No damage-relevant trigger is skipped because a runtime adapter currently lacks it. Existing primary/coincident runtime receipts below remain historical and are not a replacement for this handoff.

**Legal A0 build:** Hotori character1052 level80/breakthrough6, N/E/Q8 and default Support8, zero selected awakenings; Marching Beyond Time `fork_Time`80/B6/**T1** and Lost Radiance `Suit6`. Set count is 4; console contributions use `add stats`, with no module geometry or specialization inference. Static compiler: character baseATK652 + weapon570 =**1222**; raw static finalATK2558.5→**2559** once nearest/ties-up; HP21720, DEF848, CR.59, additionalCD1.10, CE1, CI/BI0, Cosmos bonus.30 (.20 authored console bonus, .10 automatic set bonus). Live weapon unconditional ATK%+.16 adds195.52 using1222 base, giving **2754.52**, not2559*1.16. Conventional Ultimate cap100 is a real separate pool, unused by World's Tide. B2/B4 are enabled at A0/B6; no awakener effect is silently granted.

Retain [Marching Beyond Time](../../weapons/time/passive.md): own real Redirect enters Wastelab and resets its0..3 counter; eligible other living ally Support/Redirect during it gives one stack per admitted input (unlike Hotori recording, this weapon counter has no per-owner uniqueness requirement). Own Burst consumes count n and ends Wastelab; its **whole initial Burst/Katana/finisher command family**, until mode end/cleanup, has additionalCD+.24+.08n atT1. It is one root Ultimate, not a new stack-consumption event per Katana input. Three consumed stacks additionally grant own DEF-ignore.12 for70 real seconds, cap1 refresh; with fewer than3 no such grant. Wastelab persists off field until own new Redirect/Burst/death; stateCD ends at mode end after final hit, no permanent CD. These lifecycle choices preserve the existing checked weapon model and source values. [Lost Radiance](../../sets/lost-radiance/passive.md) adds own DEF-ignore.25 for20s on Burst commit, cap1 refresh, half-open expiry, including all eligible packets in that window, not +25% damage. Death removes wearer effects; R6/A6 are not selected in this fixture.

**Recorded teammate:** MintA0/B6/N/E/Q8, existing complete legal build `examples/built-game-research.json` `builds[0].build`: Mind RoyaleT5, FirefliesSuit2, its seven recorded modules and substat selection (reuse that exact build). Recompiled static finalATK2319.15→**2319**, baseATK1151, HP17720, DEF727, CR.57, additionalCD.94, CE1, BI60, Anima bonus.10. Passive2 DEF+.20 only while actually active (her13s cast), absent during virtual off-field replay. Her two E8 hits each1.542ATK/2.45energy/4.1Cycle; no arbitrary Hotori conversion. Mind Royale gives18 on actual and copied semantic Redirect if its20s ICD ready. Fireflies gives.08CD after each positive Anima delivery, cap7, individual10s lifetimes/no-refresh-at-cap, retained off field. Thus each isolated two-hit Skill uses CD.94 on hit1 and1.02 on hit2. No incoming damage or MaxHP dependency occurs.

**Scenario:** party index0 Hotori,1 Mint, both fullHP/living; one stationary in-range dummy HP24,000,000, DEF1080, Cosmos/Anima RES.20, explicit **no Break bar**, no incoming attack, no fall/control branches. Actual resources start0 energy/0Cycle/carry0, Hotori Timepiece0, no cooldowns/effects/records; active Hotori. The actions here do not read/spend stamina, so no fabricated account stamina constants are needed. Expected-crit arithmetic below is a deterministic model oracle; an implemented seeded-crit run must report its actual per-hit samples/seed separately. Both use the real200s denominator.

Execute each row twice, once at offset0 and again at offset100, **carrying all live state**:

| Relative real time | Action and consequence |
| --- | --- |
|0 | One full ordinary Hotori Normal chain, queued at its native next-Basic windows. Fifteen packets,6.121ATK,22.299energy,37.203Cycle; complete at **4.351153813333333**. Then explicitly wait11. |
|11 | Present Replay commits/reserves60. First period Timepiece66→6; second period carried120→60. Source record event11.16747499 opens five-second window until16.16747499. Seven Skill hits use real schedule above; recovery12.4086605. Wastelab begins with0 weapon stacks. |
|12.5 | Ordinary switch to Mint, after Hotori recovery. Hotori Cycle49.005 first period /98.010 second, both<100, so no paid QTE and no Blossom. Recording continues off field. |
|13 | Real Mint Redirect Skill. This one distinct ally fills one record and one Wastelab stack. Mind Royale pays18; actual hits13.20017847/13.5654236; recovery13.9673227, cooldown until19.18279111. |
|14.1 | Ordinary switch back Hotori. Mint's outgoingCycle8.2 first period /24.6 second<100; no reaction. At16.16747499 settle rewind, resume passive6/s; A3 not selected. First Timepiece reaches120 at35.16747499, second at126.16747499 absolute, both before their Burst. |
|36 | World's Tide commits full120 reservation; conventional Ultimate untouched. Consumes one Wastelab stack: all Q-family additionalCD1.10+.24+.08=**1.42**. Lost Radiance ignore.25 until56. Time Stop enters38.1833334, resource deadline50.1833334. Initial ten Burst hits38.31608..40.002235. |
|40.216431 | Playback dispatches the sole stored Mint Skill once and clears records. No real field switch, resource/cooldown payment or new switch reaction. Mind Royale ICD expired33, so semantic replay pays18 and next ready60.216431. Mint packets40.41660947/40.7818546 use her live ATK/Fireflies; previous13s stacks expired before playback. Virtual caster pauses do not shift Hotori Katana. |
|40.6042376 onward | Ten queued Katana input starts: **40.6042376,40.928818186666667,41.411492117333333,42.203310170666667,43.018384170666667,43.490588370666667,43.815168957333333,44.297842888,45.089660941333333,45.904734941333333**. Count markers increment once each, giving two five-stage trains/26 packets, no energy/Cycle. |
|46.376939141333333 | Next Basic selects B4 finisher. One10.283ATK hit at**47.834796241333333** ends Time Stop before50.1833334 budget deadline, discarding unused reservation. Native clear marker48.128948341333333 is idempotent; final Any recovery **48.513662741333333**. Passive6/s resumes at actual finisher hit, not at later clear. |
|100 | Explicit wait to period boundary. Timepiece full120 since67.834796241333333, no record/mode/buff/field remaining; repeat above at+100 with retained ordinary resource pools/carries and real cooldown timestamps. After second final recovery148.513662741333333, explicitly wait200. |

**Resource reconciliation:** each100s period has Hotori59 direct packets (15N+7E+10initialQ+26Katana+1finisher), Mint4 (2original+2copied), so **126 total packets over200s**. Hotori raw ordinary energy29.4 plus .6*9.8 from actual/copied Mint =**35.28 per period**. Mint receives .6*29.4 +9.8+2*18 =**63.44**. At100 Hotori35/carry.28, Mint63/carry.44, zero overflow; at200 Hotori70/carry.56, Mint100/carry.88, Mint cumulative overflow26. Conventional energy is not spent by Hotori Q and does not refill Timepiece. Cycle at100 Hotori49.005/Mint16.4; at20098.010/32.8, no cap reset or invalid QTE payment. Timepiece at100/200120, but first and second Skill leave different residual6/60 because their starts are carried, not reset. Records0 and modes passive at both boundaries. Mind Royale ready timestamps60.216431/160.216431 remain real world, not rewound. Fireflies expire50.41660947/50.7818546 and150.41660947/150.7818546, no silent reset at100.

**Independent damage oracle:** with expected crit, no packet rounding, A0, one dummy and the exact live stats above, each period's Hotori non-Q coefficient11.267 uses `2754.52 * coefficient *1.30 *(1+.59*1.10) *.4`; Q-family coefficient42.839 uses `2754.52 * coefficient *1.30 *(1+.59*1.42) *16/35`. DEF factor during .25ignore is4/7, RES factor.8, so16/35, not.4*1.25. Each of the two Mint Skills uses `2319*1.542*1.10*.4 *[(1+.57*.94)+(1+.57*1.02)]`. Total **330598.43858846966857 over200s**, expectedDPS **1652.99219294234834**. Exact totals are3615920422061387/10937500000 and3615920422061387/2187500000000. Independent Fraction/event assertions checked all126 packets, ten count markers per Burst, both real recording windows/replay clears, no qualified reaction from the actual outgoing pools, exact Timepiece admission/recovery, source resources/carries/overflow, Fireflies/Mind Royale and the real denominator. This is model arithmetic, not a claim of game observation or a completed engine replay.

**Readiness/limits:** the full chosen specification is implementation-ready; native resource-body/pause/refund fidelity remains inferred where labeled, and current executable partial adapters still need implementation. Required replay callbacks, source scopes, Timepiece and complete Katana chain are not optional to obtain a successful result. Additional observations may later refine these chosen rules but do not reopen this accepted handoff.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,4,5,3,2 (fifteen hits). The chain loops, retaining native effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`hotori-normal-chain.json`](../../../../reviews/q10-game-action-binding/hotori-normal-chain.json).

## Action packets and owner effects

- GA_Jin_Melee public primary packet is GE_Player_Jin_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.200/0.343/0.370; category A/COSMOS metadata stays source-bound.
- GA_Jin_QTE public primary packet is GE_Player_Jin_QTE_Damage. Native coefficients at QTE levels 1/8/9 are 2.000/3.428/3.702; category QTE/COSMOS metadata stays source-bound.
- GA_Jin_Skill public primary packet is GE_Player_Jin_Skill1_Damage (40%), followed by 5x GE_Player_Jin_Skill3_Damage (40% each) and 1x GE_Player_Jin_Skill2_Damage (60.1%) under hotori.skill_burst_packets_v1; category E/COSMOS metadata stays source-bound.
- GA_Jin_UltraSkill public primary packet is GE_Player_Jin_UltraSkill1_Damage (17%), followed by 5x GE_Player_Jin_UltraSkill1_1..5_Damage (17% each), 1x GE_Player_Jin_UltraSkill2_Damage (130.3%), 1x GE_Player_Jin_UltraSkill4_Damage (44.3%), and 2x GE_Player_Jin_UltraSkill3_Damage (261.6% each) under hotori.skill_burst_packets_v1; category Q/COSMOS metadata stays source-bound. on_cast enters Time Stop (12s base) which triggers A1 (+12% Time Stop dmg per recorded skill) and A2 (+20% Time Stop damage bonus).
- Resonance 3 activates at awakening >= 3 and adds +1 level to GA_Jin_Melee, GA_Jin_Skill, and GA_Jin_UltraSkill without manual overrides.
- Awakening 5 (All-Seeing) increases entire team damage against single target by 15% when Hotori is deployed.
- Awakening 6 (Time Takes Away) ignores 30% of target DEF during Time Stop, and resonance_6 currently grants +20% team ATK in the executable model (owner code uses 1/5). Frozen `DT_JinEffectFigure.Jin_lvExtra_AtkUp` specifies 30%, while the frozen description specifies 20%; the native-source conflict remains unresolved. Current implementation: [awakenings.rs](../../../../crates/content/src/characters/hotori/awakenings.rs).
- Canonical arbitrary awakening selections: Effect1/2 Time Stop damage, Effect5 team single-target damage and Effect6 Time Stop DEF ignore use selected identities. Team ATK and talent levels remain count-based resonance bonuses, preserving the implemented 20% versus native-figure 30% conflict. Existing unsupported owner mechanics remain pending.

## Ordinary-hit Ultimate Energy

The current `hotori.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 22.299 and Skill 7.098 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`hotori-continuing-energy.json`](../../../../reviews/q10-energy-ledger/hotori-continuing-energy.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`hotori.skill_animation_profile_v1`, `hotori.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Jin_Skill`), `AnimNotify_TriggerCooldown` is at 0.168s, gameplay events land at 0.182s & 0.437s (`Skill.3`), 0.292s, 0.364s, 0.494s, 0.578s (`Skill.1`), and 0.864s (`Skill.2`); cancel windows activate at 1.001s (`BreakSkill_Jump`), 1.217s (`BreakSkill_AnySkill`), and 1.220s (`BreakSkill_Move`) with total duration 6.667s. For Burst (`Jin_UltraSkill`), gameplay events land at 2.316s–2.817s (`UltraSkill.1` series), 2.900s (`UltraSkill.2`), 3.928s (`UltraSkill.4`), and 3.972s–4.002s (`UltraSkill.3`); cancel windows activate at 4.604s (`BreakSkill_Move`, `BreakSkill_AnySkill`, `BreakSkill_Jump`) with total duration 10.833s.
- **Multi-target delivery (`hotori.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Hotori's Burst and Skill packet deliveries (Skill: 1x `GE_Player_Jin_Skill1_Damage`, 5x `GE_Player_Jin_Skill3_Damage`, 1x `GE_Player_Jin_Skill2_Damage`; Burst: 1x `GE_Player_Jin_UltraSkill1_Damage`, 5x `GE_Player_Jin_UltraSkill1_1..5_Damage`, 1x `GE_Player_Jin_UltraSkill2_Damage`, 1x `GE_Player_Jin_UltraSkill4_Damage`, 2x `GE_Player_Jin_UltraSkill3_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/hotori-action-binding-model.json`](../../../../reviews/q10-game-action-binding/hotori-action-binding-model.json).
- Owner supplements: [owner tests](../../../../crates/content/src/simulation/tests/hotori_actions.rs), [`hotori-normal-chain.json`](../../../../reviews/q10-game-action-binding/hotori-normal-chain.json), [`hotori-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/hotori-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_hotori.json`, `data/extracted/neverness_jin.json`, [source provenance policy](../../../SOURCES.md), [source provenance policy](../../../SOURCES.md), [consumed typed damage rows](../../../../crates/data/src/native_damage/jin.rs) (original `DT_SkillDamageData` export is cache-only), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), `1052AwakenEffect` (cache-only awakening source), [`hotori/parameters.rs`](../../../../crates/content/src/characters/hotori/parameters.rs) (consumed typed parameters; original `DT_JinEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- Katana combo, Timepiece rewind/replay, and their follow-up damage actions remain unimplemented.
- See [Time Stop resources and cooldown](time-stop-resource-and-cooldown.md) for known scalar/cooldown findings and the resource/timer execution gaps.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/hotori). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Hotori (`GA_Jin_*`):

| Action / damage row (`GE_Player_Jin_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee9_Damage` | A | 0.044–0.722 / 0.075–1.237 / 0.081–1.336 | 0.274–4.508 / 0.457–7.521 / 0.055–0.910 per hit |
| Branch `Branch1_Damage`, `Branch2_Damage` | A | 0.240 / 0.411 / 0.444 | 1.500 / 2.500 / 0.300 |
| Plunge `AirAttack_Damage` | X | 0.480 / 0.823 / 0.888 | 3.000 / 5.000 / 0.600 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.580 / 2.708 / 2.924 | 4.000 / 6.600 / 2.500 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 9.500 / 0.000 / 2.500 |
| Skill `Skill1_Damage` | E | 0.400 / 0.686 / 0.740 | 0.947 / 1.574 / 0.187 |
| Skill `Skill2_Damage` | E | 0.601 / 1.030 / 1.112 | 1.421 / 2.362 / 0.280 |
| Skill `Skill3_Damage` | E | 0.400 / 0.686 / 0.740 | 0.946 / 1.572 / 0.186 |
| Burst initial `UltraSkill1` .. `UltraSkill1_5_Damage` | Q | 0.170 / 0.291 / 0.315 each | 0.000 / 0.000 / 0.032 |
| Burst initial `UltraSkill2_Damage` | Q | 1.303 / 2.233 / 2.412 | 0.000 / 0.000 / 0.244 |
| Burst initial `UltraSkill3_Damage` | Q | 2.616 / 4.483 / 4.842 | 0.000 / 0.000 / 0.490 |
| Burst initial `UltraSkill4_Damage` | Q | 0.443 / 0.759 / 0.820 | 0.000 / 0.000 / 0.083 |
| Time Stop Melee `UltraSkillMelee1..5_Damage` | Q | 0.085–2.211 / 0.146–3.789 / 0.157–4.092 | 0.000 / 0.000 / 0.000 |
| Time Stop Finisher `UltraSkillMelee6_Damage` | Q | 3.000 / 5.141 / 5.553 | 0.000 / 0.000 / 0.000 |
| Time Stop Finisher (P2) `UltraSkillMelee6Pas2_Damage` | Q | 6.000 / 10.283 / 11.106 | 0.000 / 0.000 / 0.000 |
| Time Stop Plunge `UltraAirAttack_Damage` | X | 0.480 / 0.823 / 0.888 | 3.000 / 5.000 / 0.600 |

Critical Riposte uses GAName `GA_Jin_PerfectEvade`, Support QTE uses `GA_Jin_QTE`. All damage rows deal COSMOS damage. Skill delivers 1x `Skill1` + 5x `Skill3` + 1x `Skill2`. Burst delivers the initial `UltraSkill` train and enters Time Stop (12s base), enabling the `UltraSkillMelee` sequence. Under Passive 2, the final slash switches to `UltraSkillMelee6Pas2_Damage`, doubling its base multiplier.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Jin_Passive_1` binds `Buff_Jin052_Passive1`. `DT_ReactionExtendDataTable` row 1052 binds `ReactionAbilityID=GA_Jin_Passive_1` (Cosmos reaction).
- **Passive 2 (Breakthrough 4):** `GA_Jin_Passive_2` binds `Buff_Jin052_Passive2`, enabling the empowered Time Stop finisher `UltraSkillMelee6Pas2_Damage` (6.000 / 10.283 / 11.106).

### Awakenings

Source descriptions and selected IDs: `1052AwakenEffect#/rows/1052/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Increases Hotori's damage during Time Stop by +12% for each recorded Support Skill or Redirect Skill.
- **A2:** Increases Hotori's damage by +20% during Time Stop.
- **A3:** Recovers the energy spent on Present Replay after use and completed recording, at most once every 60s; spending energy during recovery interrupts restoration.
- **A4:** Grants an additional +10 energy to the Non-Closed Timepiece for every target defeated by the team while the pointer is not rewinding (up to 3 times); resets on casting World's Tide.
- **A5:** While Hotori is deployed and only one enemy remains on the field, team damage against it increases by 15%; the effect is lost when Hotori is down.
- **A6:** Hotori ignores 30% of the target's DEF (`TargetDefIgnore = 0.30`) during Time Stop.
- **Resonance 3:** Increases the skill level of Misty Moon Style (Normal), Present Replay (Skill), and World's Tide (Burst) by +1.
- **Resonance 6:** Grants all team members +20% ATK while Hotori is on the team and alive (with a recorded conflict against `DT_JinEffectFigure.Jin_lvExtra_AtkUp` specifying 30%).

### Passive description coverage, reviewed October 18

Saved description source: `guide_hotori.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Ghost Orchid Crest describes Blossom Buds continuing to attack during Time Stop. All Treasures Under Heaven describes the Worlds Tide finisher DMG Ratio increased by 100%. Legacy Cosmos naming does not classify this Blossom extension. These descriptions do not settle which other clocks pause or how the finisher coefficient addition applies across ordinary and empowered packet identities; see the linked Time Stop subsystem.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Jin_Melee#/11; Melee.6 | PlayerJinMelee5_2 | (0.0, 0.0, 0.2, 1.0, 1, false) | GE_Player_Jin_Melee6_Damage |
| GA_Jin_Melee#/12; Melee.5_1 | PlayerJinMelee5_1 | (0.0, 0.0, 0.2, 2.0, 1, false) | GE_Player_Jin_Melee5_Damage |
| GA_Jin_Melee#/13; Melee.5 | PlayerJinMelee5_1 | (0.0, 0.0, 0.2, 2.0, 1, false) | GE_Player_Jin_Melee5_Damage |

Melee.5 and Melee.5_1 bind the same Melee5_Damage configuration; Melee.6 instead binds Melee6_Damage. All three entries explicitly set TriggerOnSpawn=false and a 0.2 Period with cap 1. These inputs distinguish configured triggering from an immediate on-spawn damage assumption and do not resolve Time Stop's real-time clock.

## Unknown execution policies

No unresolved implementation decision remains for this campaign. Native-fidelity notes only: source-coordinate timing and exact native reservation/refund/clock dispatch are not observations; use the complete selected specifications. R6 is selected30% from its bound native modifier and current mirror, correcting historical20%. Physical geometry is outside the stationary all-in-range scenario. Historical partial runner models/receipts below retain their original limits and do not override the handoff.

## Responsibility cards

- [time-stop-resource-and-cooldown](time-stop-resource-and-cooldown.md#unknown-execution-policies) contains the findings and questions for that distinct subsystem.

## Prior findings

- [`reviews/q10-game-action-binding/hotori-action-binding-model.json`](../../../../reviews/q10-game-action-binding/hotori-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owner tests](../../../../crates/content/src/simulation/tests/hotori_actions.rs), [`hotori-normal-chain.json`](../../../../reviews/q10-game-action-binding/hotori-normal-chain.json), [`hotori-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/hotori-skill-burst-packets.json).

## Research history

Hotori8/8 accepted; continue at **Iroi/packets**. Reused original/expansion ability, notify, actor and figure data plus public kit; prior native-body/public attempts counted, remaining decisions are reasoned checked choices above. Do not restart Hotori collection or implement simulator code before184/184.

## Relative combat timer binding

World's Tide commitment through setup, EnterTimeStop and actual finisher, expiry or defeat. Retrieved EnterTimeStop component plus the selected connected active-stop model; no fixed full-lifetime subtraction. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
