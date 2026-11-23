# Adler action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1033`.

**Responsibility:** Adler outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** partial executable slice; whole-kit coverage is incomplete. This records implemented and tested scenarios only; hypotheses do not establish native behavior.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Equipped DEF consumers

`weapons.koinobori_v2` adds the qualified party's base-scaled DEF percentage to both the existing Blessing snapshot and A4 live overflow query, using the same gear stat query as damage. With base/final DEF1000, Skill1 and T1 Koinobori, the shield is1880 and the independent A4 cap550; shield efficiency does not multiply the A4 cap. The [two-cycle equipped source](../../../../examples/koinobori-shield-cycles.nte) and Rust/native/WASM/actual Worker incoming-damage fixtures retain source resources and distinguish qualified three-type from unqualified two-type parties. This fixes a missing equipment stat without changing the bounded admission snapshot, recipient model or the remaining native dispatch work.

## Complete packet specification

**Accepted mixed handoff, October18.** Source: selected `GA_Adler_Melee/Skill/UltraSkill/QTE/PerfectEvade` maps and montages, `DT_SkillDamageData`, `DT_AdlerEffectFigure` and the existing full-kit investigation below. Every value at an intermediate released talent coordinate comes from the native array;1/8/9 columns below are compact checks, not interpolation endpoints. All damage is **Incantation**, with A/E/Q/X/Z/H category retained; live scaling stat is stated per row, not assumed ATK for all actions. Regular direct packets sample seeded crit independently. **Skill periodic damage has source FixedCritRate=.5**, replacing CR with50% and keeping live CD; it is not a noncritical or normal-CR DoT.

| Action | Row suffix after `GE_Player_Adler_` | Stat and coefficient at1 /8 /9 | Count |
| --- | --- | --- | --- |
| Normal1 | Melee1_Damage | ATK .201 /.344 /.372 |1 |
| Normal2 | Melee2_Damage | ATK .577 /.989 /1.068 |1 |
| Normal3 | Melee3_Damage | ATK .154 /.264 /.285 each |7 |
| Normal4, selected source profile | Melee4_1,4_2,4_3_Damage | ATK .219+.231+.746 /.375+.396+1.279 /.405+.428+1.381 |3 |
| Normal5 | Melee5_Damage | ATK .875 /1.500 /1.620 |1 |
| Aim/Liberation | Aim_Damage | ATK .104 /.178 /.192 |1, single target |
| Plunge/Ignorance | AirAttack_Damage | ATK .360 /.617 /.666 |1 landing |
| Riposte/Revelation | PerfectEvadeAttack_Damage | ATK1.760 /3.016 /3.258 |1 |
| Support | QTE_Damage | ATK2.000 /3.428 /3.702 |1 |
| Skill initial | Skill3_Damage twice, Skill2_Damage once | DEF .700 /1.200 /1.296 each |3 |
| Skill periodic | Skill_Damage | DEF .200 /.343 /.370 each |10 at1s intervals |
| Burst | UltraSkill_Damage | DEF1.000 /1.714 /1.851 each |5 per target,10 if sole target at admission |

**Concrete source/description conflict, resolved for implementation:** the historical public/model Normal4 contains two hits and the old chain has12 packets,5.452 ATK at level8. The selected `Adler_Melee_004` now has Melee.4/.6/**.7** at.23622027/.48173124/**.73400503**, and `GA_Adler_Melee` maps the third to **Melee4_3_Damage**, which has a nonzero1.279 ATK row at8. Choose all three reached requests once in the selected source profile rather than discarding the third damage action. This gives **13 packets**,3.927/6.731/7.269 ATK at1/8/9. It is a mixed inference about successful delivery, not a recovered combat observation. Preserve the legacy12-hit implementation/receipts below; do not silently use their energy oracles for this13-hit handoff. Normal3 retains the separately described seven projectile hits; its two request markers are not multiplied into fourteen hits. Approach/entry/Branch_001 replace Normal1 and do not add another unconditional chain stage.

**Skill route choice:** Skill.3/.4 each request Skill3 damage once. Skill.2's reached actor container includes Skill2 damage plus the periodic Skill carrier. Apply Skill2 once on actor arrival and install the10-tick carrier, **without an immediate periodic tick**; this matches the three initial .7 DEF description and the10s/1s curves. The actor table's11s duration is a container lifetime, not an eleventh periodic hit or attack recovery. HitBack/Float/display actors add no damage. Recasting replaces the previous periodic generation; its already dispatched initial packets retain their cast snapshot. Delayed ticks survive field exit while source/target live and retain the original consumed-Karma damage bonus, while DEF/CD/target modifiers are live per hit.

Burst creates five Sunya projectiles, each hits every eligible dummy once per volley; second volley only for the admission-captured sole-target branch. Five source spawn entries and the public five-per-enemy description establish this count; neither five multiplied by all projectiles again nor target death mid-volley creates extra strikes. No separate `UltraSkill_Damage1` train is admitted alongside the selected row merely because it exists. Plunge uses an explicit scenario landing and fall bonus0..1, multiplying base by1+bonus as [the shared spatial abstraction](../mint/action-damage.md#complete-packet-specification); never double every plunge. There is no finite charged-melee chain beyond aim/fire in the selected descriptions/ability list; long Basic chooses Liberation, not an invented heavy action.

**Independent arithmetic:** at level8, ATK1000/DEF1000, half target DEF mitigation, no crit/bonus/RES, selected Normal chain deals**3365.5** versus legacy2726. Skill initial1800 plus periodic1715 gives3515 before Karma and periodic crit. Consumed20 Karma multiplies that Skill total by1.40 ->**4921**; selected A2 gives1.60 ->5624. Actual sampled DoT may crit at fixed50%, so3515 is a raw noncritical oracle, not an expected simulated total. Burst sole-target8570, two targets4285 each (same summed8570, different recipients/crit). Aim89, base plunge308.5/max617, Riposte1508, Support1714. Live DEF adds must use base DEF, preserving [stat zones](../../shared/damage-and-stat-zones.md).

## Timing specification

**Mixed `adler.source_schedule_v2`.** Reuse the disclosed rate1 + per-owner hitstop integration in [Mint timing](../mint/action-damage.md#timing-and-continuation-specification), using **Adler's own** source coordinates/pauses. Existing source/native and public approaches are recorded below; no frame measurement is asserted. Durations/cooldowns are real unless explicitly called action progress; all actual elapsed time enters the denominator.

| Action | Nominal damage offsets | Next ability / movement | Jump |
| --- | --- | --- | --- |
| Normal1 | .15702307 |1.0805783 /.68710935 | .18160343 |
| Normal2 | .09250939 | .8673032 /.5236144 | .11665799 |
| Normal3 | seven offsets `a+j*(b-a)/6`, j0..6, a=.11747372,b=.5587595 |1.3841664 /.94059736 | .12617594 |
| Normal4 | .23622027,.48173124,.73400503 |1.3977767 /1.1257335 | .24901962 |
| Normal5 | .5995566 |1.0022049 /1.1980758 | .62172705 |
| Approach Normal1 | .23336193 |1.0897675 /.6163142 | .24901962 |
| Entry Normal1 | .157023 |1.0679594 /.649143 | .17347847 |
| AimFire | .052044153 | .5 repeating fire section; explicit exit may interrupt aim | no separate emitted jump window |
| AimEnd shot | .016957 | .30121806 /.30121806 | .30121806 |
| Plunge landing | .050169 after declared landing | .195412 /.82639194 | .195412 |
| Riposte Atk | .34929872 |1.054915 /.8377732 |0 (Skill also0) |
| Skill initial | .0840776,.33390695,.6835115 | .85161036 /.9903655 | .108601965 |
| Skill ticks | actor arrival.6835115 +1..10 World seconds | independent off-field carrier | not a new input |
| Burst volley1 / conditional2 |2.1851034 /2.343074 |3.0821378 /3.0821378 |3.0821378 |
| Support | .68333334 |1.0404353 /1.0404353 |1.0404353 |

Normal3's evenly spaced seven-hit schedule is an **inference constrained by the two reached request endpoints and the seven-hit description**; it is not seven serialized notifies. It keeps nonzero progression and the known total without an arbitrary800ms cadence or fourteen duplicate hits. Projectiles deliver at their declared scenario travel delay (contact baseline0 after startup), not an extra damage event for direct-map plus projectile requests. Burst uses the two reached montage marker times as volley boundaries; the second event also owns Karma/A5 callbacks, not five additional unbounded volleys.

Explicit `(duration,rate,delay)` hitstop: Normal1 `(.067,.1,.04)`, Normal2 `(.067,.1,.03)`, Normal4 first/second `(.033,.1,.04)` each, Normal4 third `(.1,.1,.04)`, Normal5 `(.033,.1,.04)`, Riposte `(.3,.01,0)`, Support `(.4,.2,0)`. Omitted positive durations create no added interval under the disclosed bounded profile, not a certified native zero. Each source event's AoE pauses the player once, not once per recipient. The nonoverlapping explicit pauses add **.2997s** to the full Normal chain: nominal5.7320295 ->**6.0317295s**. A no-hit/miss does not add its pause.

Skill CD starts at TriggerCooldown **.04943889**, lasts16 World seconds. Skill.5 first marker **.06664077** captures/clears held Karma once for this cast; its later.7512654 repeat is idempotent for that generation, not a second capture of zero overwriting the active bonus. This chooses the recovered copy callback timing over the historical admission-time approximation. A cancellation before the first marker preserves held Karma. Skill.1 grants shield at **.57470286**, not at cast admission. Skill.2 actor spawn coordinate.5835115 plus table DelaySpawnTime.1 yields arrival.6835115; positive duration/period0 do not themselves authorize repeated initial Skill2 hits. Burst starts its20s CD and spends120 at admission; A1 Karma/A5 energy callbacks occur at **UltraSkill.2 2.343074**, if source lives, once each. They are not universal cast-begin refunds. Delayed independent projectiles/ticks preserve the paid cast and owner.

For switch/cancel use the separate table boundaries, except Burst's CantChangePlayer ends3.0340629 and Support's ends1.0037187, allowing switch before their next-ability recovery. Other action switches use next-ability boundary. Cancel/interruption removes future montage notifies, but not already installed Skill generations or launched projectiles; death cancels Adler-owned future damage. No committed resource refund except the explicit selected A5 callback. Normal chain resets on other admitted ability, field exit/death or input-start gap>2s; exactly2s continues, no-op switch does not reset. Descent time and successful Critical Dodge/parry are scenario events and consume their declared action time/cost. Front/Back movement is non-damaging: Front jump/Skill/any .082544/.33066738/1.0070616, Back0/.33313835/1.0155504 with movement.7853796. The explicit parry .02 delay belongs only to its configured route; a dodge alone cannot produce Riposte. These are complete selected admission/timing policies while native branch priority remains an accuracy limit.

## Resources and admission specification

Native source `.adler.resources.karma` reads the whole held pool (default0, cap20), without an expiry or a fabricated `.mods.karma`. It is separate from consumed per-cast snapshots. Source query tests preserve a held stack across16 idle seconds and consume it through Skill in two continuing cycles, with native replay agreement. Additional explicit start-resource authoring remains Q4-L4.

**Mixed source/model.** Adler's Ultimate capacity is **120**, explicitly `DT_PlayerPackData/rows/Adler_base/ChargeMax`, and Burst cost120 (`DT_GameplayAbilityCostData`), not the other owners'100 default. Initial amount0..120 is declared, baseline0. Use [shared grant/carry/overflow/recipient rules](../../shared/energy-and-gauges.md#declared-build-resource-model-for-the-collection-handoff), with per-owner capacity120 and recipient-specific Charge Efficiency. Direct successful packets grant once independent of target count; Skill periodic, Skill3 and Burst raw grants are0. A5's20 **flat restoration** is a separate owner grant, no Charge Efficiency and no team .6 share; retain existing fractional carry. Cycle capacity100 is separate, gains only the listed owner Heterochrome values, no team matrix. No passive natural energy/Cycle regeneration.

| Selected packet | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- |
| Melee1 |1.254 /2.095 /.248 |
| Melee2 |3.599 /6.010 /.712 |
| Each of7 Melee3 | .963 /1.608 /.191 |
| Melee4_1 |1.364 /2.278 /.270 |
| Melee4_2 |1.440 /2.405 /.285 |
| Melee4_3 |4.659 /7.780 /.922 |
| Melee5 |5.458 /9.115 /1.080 |
| Aim | .700 /1.100 /.100 |
| Plunge |2.300 /3.800 /.500 |
| Riposte |4.400 /7.300 /2.500 |
| Support |6.100 /0 /2.500 |
| Skill2, once |4.000 /6.700 /.800 |
| Skill3, each; Skill periodic |0/0/0 |
| Burst, each |0 /0 /.500 |

The selected13-packet Normal chain sums to **24.515 Ultimate /40.888 Cycle /4.858 Break**; the historical12-hit values19.856/33.108/3.936 omit Melee4_3 and do not fund the new handoff. Five chains+Skill give **126.575 raw Ultimate**. At unit efficiency/cap120, Burst can spend120, overflow6 integers and retain.575 carry; this is not zero energy reset.

**Karma:** source `Buff_AdlerUltraSkill_QiHai` is **Infinite**, AggregateByTarget, cap20, removed on combat exit. Its serialized15s duration curve is inactive under Infinite, so do not invent a15s expiry. Gain1 per connected Normal or plunge packet and2 per connected aim packet, once per packet regardless of dummy count; Riposte/Support/Skill/Burst damage do not gain Karma except selected A1's explicit20 grant. Clamp20, record overflow/consumed count, preserve across field exit. Skill's first CutCount event copies held count into this cast's private cap20 snapshot then clears the held pool; the same cast's repeat event does not recapture. Captured bonus lasts through that cast's12s source-carrier window (1.2*10) and applies only to its Skill initial/tick packets. Future Normal gains belong to the held pool and cannot modify the old captured value. Recast captures a new snapshot and replaces the old periodic generation; already dispatched independent packets retain the old snapshot. First partial cancellation before CutCount spends no Karma; after capture there is no refund. Two full chains cap20, rather than26; consuming20 leaves0, one later Normal gives1 without altering the previous40% Skill final bonus.

Non-Burst ordinary attacks/Skill have no serialized Ultimate cost and are modeled as0 Ultimate, consistent with zero-start generation and their descriptions. No reached finite hold-stamina consumer exists for Liberation; infer0 additional stamina for aim/fire and ordinary attack actions. A preceding jump/dodge uses the explicit account action cost/time and recovery model, not a free critical-success input. Support uses the qualified outgoing-Cycle switch. Admission also requires active/living Adler, live target where targeted, correct grounded/airborne/aim/critical-response state and the timing/cooldown boundary. Failed inputs diagnose without spending, implicit waiting or resetting any continuing state.

## Effects and passives specification

Source `.adler.mods.blessing`/`.adler.blessing.active` read the actual positive shield wherever its living party carrier is, with a living Adler source. Reads never refresh or reconstruct capacity. Skill authoring declares the selected party-interception model; fixed incoming recipients remain fixed even after the pool moves. Rust source/native replay checks cover A0/A6 eight/twelve-second deadlines over two cycles, depletion while Adler is off field, and source death. Shared incoming numeric fixtures now direct shielded hits to active Mint and retain a separate unshielded named Adler hit.

**Mixed source/inference.** Source magnitudes and unlocks are the existing Karma calculator/shield-cue findings, `1033AwakenEffect`, `DT_AdlerEffectFigure`, `DT_ReactionExtendDataTable/1033` and the selected carriers. Lifecycles below select implementable rules despite the unrecovered native shield/recipient defaults.

| Effect | Concrete rule |
| --- | --- |
| Righteous Heart, passive2/B4 | Add **.20 DEF%** once to Adler's base-DEF zone, active in party on/off field; no duplicate on switch. Reinitialize once on revival/equip, remove on death/unequip. A base1000/existing.50 DEF% source becomes1700, not1800. This affects Skill/Burst/shield but not ATK-based Normals. |
| Karma Skill bonus | Per consumed stack add **.02 FinalDamageUp**, or .03 with selected A2, from `.02 + Adler_lv2(.01)`. Sum this one zone then multiply direct/periodic Skill damage by1+sum; no second general held-Karma bonus. Snapshot count at first CutCount. |
| Blessing | At Skill.1 .57470286 choose effective Skill talent and snapshot Adler's live DEF/shield efficiency. Capacity `round_half_up((base[level]+coef[level]*DEF_Adler)*(1+outgoing_shield_efficiency+.20_if_A3))`; base1..9 **1000,1080,1166,1260,1361,1470,1587,1714,1851**; coef **.8,.86,.93,1,1.08,1.17,1.26,1.37,1.48**. Native arrays continue through13 in the source table. Lifetime **8 World seconds**,12 if A6. Refresh/recast replaces that source's pool/expiry, never stacks two same-source pools. |
| Blessing party interception, inferred | One Adler-owned **party shield pool protects the current living active actor**, beginning with caster Adler. It follows field ownership without resetting capacity, snapshot or expiry. Keep the carrier/stat source as Adler: the reached cue's MyTarget is Adler under UseOwner. This selects the public active-character shielding role and avoids changing shield capacity to an ally's DEF on switch. The old runtime recipient-bound model below is narrower and must be extended; this is not a claim that native global interception was recovered. Source death/unequip removes the pool; field exit alone preserves it. Recipient death transfers future protection to the next living active actor without resurrecting anyone. |
| A1 Form | Burst UltraSkill.2 adds **20 held Karma**, capped20, once; a missed volley still reaches the event if the montage continues. Does not add damage, copy Skill snapshot or refill Ultimate. |
| A2 Sensation | Karma's per-consumed-stack Skill final-zone addition becomes.03, not a separate.03 times current held count on every action. |
| A3 Perception | Adds **.20** in outgoing ShieldEfficiency for Blessing capacity only, not DEF or emergency A4 absorption. |
| A4 Formation | On **damage depletion** of Blessing, after ordinary shield absorption and before HP loss, absorb up to `.50*live_DEF_Adler` of the remaining incoming event once, nearest/ties-up. It is a one-event overflow mitigation, not a persistent second shield, heal or stored remainder. No ShieldEfficiency/A3 multiplier. Expiry/replacement/field exit/cleanup do not trigger it. Exact ordinary-shield depletion with no overflow records0 extra absorption. |
| A5 Discernment | UltraSkill.2 restores **20 flat Ultimate** once to living Adler after the paid Burst, retaining carry and overflow. Not scaled by Charge Efficiency; no allied grant. Interrupted montage before event gives no restoration. |
| A6 All Forms Are Void | Select12s Blessing carrier instead of8s; remove prior source carrier/pool on refresh. No duplicate base+Lv6 shield. |
| R3 Cleanse | At least3 awakening identities: effective Normal/Skill/Burst+1, Support unchanged. Selection identity and count remain separate. |
| R6 Enlightened Guardian | While living Adler's positive, nonexpired Blessing protects a living active actor, add **.10 Incantation DMG bonus** to living party packets, including off-field sources. One source contribution, no duplicate copies via display/60s fallback carrier. Depletion/expiry/death removes eligibility immediately. No effect on another element or True damage. |
| Temperance, passive1/B2 | Each successful Scorch **application**, not each tick, selects one of three target debuffs for15s: ATK-.20; resistance-.10; target Break acceptance+.10. Same debuff cap1/refresh15s, different results coexist. Seeded selection/precise channels are below. Active while Adler is a living party member, including off field. |

**Shield inference rationale/checks:** UseOwner gives a stable Adler ASC for the cue's level/DEF initialization; public kit promises protection of active characters and the shield survives field changes. A party interception pool preserves both without reading a newly active ally's unrelated DEF or refilling on every switch. At Skill8/DEF1000/no shield bonuses, capacity**3084**, A3 capacity3700.8 ->**3701**. A shield at remaining100 taking800 with liveDEF1000 and A4 absorbs100 ordinary+500 emergency, loses200 HP; it does not grant500 for the next hit. Switching with remaining100 preserves100. At ordinary depletion with overflow0, A4 adds0. R6 adds.10 to the existing Incantation bonus sum, not an independent1.1 multiplier; no shield means no R6 even if a stale visual carrier exists. Multiple shield sources settle in stable application/owner order and A4 can mitigate that event only once per actually broken source.

**Temperance source details/inference:** `_1033_1` modifies target AtkUp; `_1033_2` explicitly modifies **ordinary physical, all six Esper channels, and Mental resistance**, not only the short description's six elements; `_1033_3` modifies target UnbalAntiAccrueEfficiencyBase. Use exact -.20/-.10/+.10 magnitudes and additive stat zones. Select uniformly among the source-ordered three candidates with seeded unbiased bounded-index draw; probabilities are an **inference from an unweighted random candidate array**, not observed odds. Repeat selection refreshes only that identity, without rerolling until a different result appears. All three may coexist after independent applications; none stacks magnitude. Source death stops new applications but existing target debuffs keep their15s lifetime; death of target removes them. An unselected similarly named `_1033` DoT-crit carrier is not a fourth random outcome or an extra generic bonus without its own admitted producer.

The executable model records `adler.temperance_v1` and draws from a dedicated reaction RNG stream. The attack outcome subtracts 20% of authored original ATK from the reported live target ATK. Authored incoming damage remains an explicit input. The resistance outcome affects subsequent Scorch ticks and other relevant damage. The Break outcome adds 0.10 to incoming gauge efficiency for eligible losses, including Discord.

At originalRES.20, the RES result gives.10 (10 percentage points), not.18. A source Break subtraction.5 with target acceptance1 and this debuff becomes.55, not.6 and not an outgoing damage bonus. Three applications each have1/3 marginal choice; probability of three distinct identities is6/27=2/9, so modeling guaranteed cycling over all three would be wrong. ATK reduction changes declared incoming damage/shield depletion where relevant; stationary no-incoming scenarios get no invented outgoing multiplier from it. No other combat awakening/resonance is selected by the source list; city skills do not affect dummy DPS.

## Forms and off-field specification

**Mixed source/model:** Adler has ordinary chain, entry/approach alternatives, aim/fire, airborne landing, explicit Critical Dodge/Riposte and qualified Support. No additional transformation or finite charged-melee sequence is described in the selected current kit. Aiming stops ordinary chain advancement; each fire input produces one Aim packet at its source startup, repeated at the.5s source section cycle. Releasing aim uses the AimEnd shot only when that explicit firing-exit action is authored, not an extra automatic shot on every toggle. Branch_001 is a Normal1 alternative. Plunge requires a later declared landing and fall bonus; Critical Dodge arms one next-Basic Riposte opportunity consumed by that action or field exit. Front/Back don't deal damage; their source transition to Atk may be selected by an explicit successful critical-response event. Neither ordinary dodge nor a parry animation unconditionally awards the Atk damage.

Skill creates one independent per-source periodic generation at actor arrival. Ten ticks at+1..+10 from arrival, then cleanup, retain cast's Karma snapshot but read live DEF/modifiers/target life. Recast cancels future old ticks (not already dispatched damage); field exit keeps ticks, source death cancels them. No target respawn/retarget creates an eleventh tick. The displayed Sunya actors/11s damage container are not separate summons with autonomous additional attacks. Burst projectiles, once emitted, finish their captured five/ten-hit train after a legal switch while source lives; interrupting before spawn emits nothing. Shield interception and remaining pool follow the active actor independently of Adler's montage. Off-field damage keeps Adler's source, DEF scaling, crit policy, resource metadata and gear effects, not the newly active actor's stats.

Karma is held owner state across field switches; captured Skill count is a different immutable cast field, and Righteous Heart remains active on bench. Combat exit clears held Karma and owned combat effects, whereas a labeled second cycle is ordinary continuing combat and clears nothing. Blessing depletion/A4/R6, damage ticks and gear shield checks use actual settled pools, never just a display tag. This preserves indirect DPS dependencies without requiring visual actors, enemy motion or native AoE geometry.

## Recipients and shared dependencies

Aim is explicitly single target. All other selected melee/Skill/plunge/Riposte/Support packets use full coefficient on every eligible living in-range stationary dummy, once per packet/target in ascending index; this is the disclosed uncapped dummy inference where native caps are unavailable. Projectiles are delivery routes for those selected packets, not automatic duplicate direct+actor hits. Burst captures eligible living population at admission: sole target gets10, multiple targets each5; later deaths never expand a surviving target's quota. Skill periodic recipients are captured at field creation, each tick rechecks survival; a later added dummy is outside that cast's declared target set. A miss/dead/rounded-zero target gives no damage-triggered resources/Karma. Fatal positive damage counts as a connected hit for grants. Target count never multiplies Karma/energy for one source packet.

Use [shared resource/Support admission](../../shared/energy-and-gauges.md#declared-build-resource-model-for-the-collection-handoff), [damage/crit/DEF/RES/Break zones](../../shared/damage-and-stat-zones.md), and [selected cycle/Break policy](../../shared/cycles-and-break.md#selected-mint-handoff-dependencies). Adler's Incantation+Anima switch creates Hexed under that model; Incantation+Chaos creates [Scorch](../../shared/cycles-and-break.md#selected-adler-scorch-dependency). Blessing is one party interception pool, not independently full capacity for every slot. A4 is incoming-only and R6 reads its real positive remaining state. Equipment dependencies use their existing owners; shields must remain available to shield-dependent weapon procs and expiry/removal must disable them on the exact boundary.

## Complete implementation handoff

**Accepted mixed specification:** all seven sections above contain selected complete-kit rules, with raw-source conflicts and explicit inferences retained. Implement these decisions without repeating the protected-native/shield/packet discovery. Existing code/receipts below are narrower12-hit, cast-snapshot/captured-shield scenarios; their passing status does not mean the new13-hit, event-timed, party-interception model is already executable. Simulator code is unchanged by this research.

**Concrete source-constraint-legal A0 baseline:** character1033, level80/80 breakthrough6, talents8/8/8/Support8, no awakenings; **Solifugae Vita `weapon:fork_snowman` level80/80 breakthrough6 T1**, **Kingdom's Guard `set:Suit7`**. Declare set count 4 and supply console contributions through `add stats`; the numeric baseline below is preserved without modeling modules.

Authored static receipt: baseATK**832**, ATK**1252**, baseDEF**994**, exact staticDEF**318577/100**, HP**19902**, CR**.35**, additional CD**1.10**, Charge Efficiency**1**, outgoing ShieldEfficiency**.20**. Under existing static quantization DEF3185.77 ->**3186**, Righteous Heart then adds994*.20=198.8, giving liveDEF**3384.8**, not3186*1.2. Keep these stages in receipts; no gear/passive double application.

Kingdom's Guard's.15 DEF% is already in static DEF and its.20 outgoing shield bonus is applied once at grant. Snowman's [T1 live shield bonus](../../weapons/snowman/passive.md) adds **.18*832=149.76 ATK** while this active wearer is actually protected: ATK becomes1401.76, not1477.36. It changes ATK-scaling Normal/aim/plunge/Riposte/Support, not DEF Skill/Burst. A0 Skill8 Blessing capacity is `round((1714+1.37*3384.8)*1.2)=round(7621.4112)=**7621**`; no A3 is selected. Preserve real pool/expiry so Snowman is neither unconditional nor suppressed. All necessary recipe/stat values are written here.

Scenario: active Adler, full compiled HP, Ultimate0/cap120, Cycle0/cap100, held Karma0, carries0, no cooldown/effects/shield, seed42. One stationary level80 training dummy, DEF1080/Incantation RES.20/HP24,000,000, explicit empty Break-bar profile and no incoming attacks. Both ordinary passives unlocked/enabled; a solo Incantation party has no Chaos switch to apply Scorch, rather than a disabled Temperance callback. Required incoming/Scorch/shield-switch branches remain fully specified above for scenarios that reach them.

**Two continuing60s periods on one120s real timeline:**

| Time(s) | Authored actions/state |
| --- | --- |
|0..30.1586475 | Five full Normal chains at6.0317295s each,13 packets each. Karma reaches cap20 and stays held; excess gains do not create a second pool. |
|after chain..32 | Explicit wait. |
|32 | Skill: CD32.04943889..48.04943889; at32.06664077 consume20 Karma/capture.40 FinalDamageUp; initial hits32.0840776,32.33390695,32.6835115; shield32.57470286..40.57470286. |
|33.6835115..42.6835115 | Ten independent periodic Skill hits, fixed50% crit and captured.40 Skill final bonus, live DEF/modifiers. |
|34 | Burst pays120, sole-target volleys at36.1851034 and36.343074, five packets each; no A1/A5 callbacks at A0. Recovery37.0821378. |
|40..46.0317295 | Sixth full Normal chain, overlapping the final Skill ticks. First Normal hit40.15702307 benefits from Snowman before shield expires40.57470286; later Normal hits do not. Held Karma becomes13. |
|after chain..60 | Explicit wait, all events continue; no resource/Karma reset at60. |
|60..90.1586475 | Five further Normal chains begin from held Karma13 and exact resource carry, recapping20. |
|92 | Second Skill, consume20; equivalent clocks shifted+60. |
|94 | Second paid Burst; no new reset or assumed refund. |
|100..106.0317295 | Twelfth Normal chain, overlapping last periodic ticks; shield/Snowman expiry is again honored. |
|after chain..120 | Explicit wait to true horizon, preserve final balances/Karma. |

**Independent checks:** unit Charge Efficiency and per-packet exact carry give126.575 raw pre-Burst grant per period (5*24.515+4). After period1's spend and continuation: energy**25+.09**, overflow6, held Karma13. Period2 starts from that exact state and ends**25+.18**, cumulative overflow37, Karma13. Total302.18 =240 spent+37 overflow+25.18 retained. Both Burst cooldowns20s and Skill cooldowns16s fit; no source state was reset. Each captured Skill bonus window ends cast+.06664077+12, after its final tick cast+10.6835115. Both off-field and overlapping-tick branches retain their own source/capture policy. Karma is Infinite, so waiting to the next period never grants a fictional15s reset.

**Packet/formula oracles, separate from the equipped seeded result:** each period has6*13 Normal+3 Skill initial+10 DoT+10 Burst=**101 packets**,202 total. ATK/DEF1000, no gear/bonuses/RES, half DEF mitigation gives raw noncritical per-period33684, total**67368**. This is coefficient arithmetic only: the actual DoT retains its mandatory fixed50% crit. With bare-fixture ordinary CR.05/CD.50, expected unrounded per-period is `(6*3365.5+2520+8570)*1.025 +2401*1.25 =**35066.325**`; two periods70132.65 and expected raw DPS584.43875 at120s. That distinguishes .50 fixed DoT chance from ordinary .05; it is not a seeded rounded damage promise. The real equipped build uses CR.35/CD1.10, liveDEF3384.8, shield-dependent ATK and exact per-packet RNG/quantization. Do not substitute these bare numbers into its report.

The independent audit verified the13-hit sum, action-time inequalities,120-cost conservation, Karma continuation, tick/capture windows and7621 shield. It explicitly reports model arithmetic only, with game observation and new engine execution both false. `build-stats` establishes only the stated existing compiler acceptance. Implementation must add content-owned corrected packets, event-timed Karma/shield/volley dispatch, active-party shield interception and shared Scorch/recipient logic, then verify the same authored continuing source through CLI/WASM/actual Worker, replay and aggregation. Unsupported runs remain incomplete and excluded from successful DPS comparisons. Additional precision is later accuracy work, not another collection gate.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,1,7,2,1 (twelve hits: Melee1, Melee2, 7x Melee3, Melee4_1+Melee4_2, Melee5). The chain loops, retaining native effective talent projection including resonance_3.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`adler-normal-chain.json`](../../../../reviews/q10-game-action-binding/adler-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`adler.skill_animation_profile_v1`, `adler.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Adler_Skill`), `AnimNotify_TriggerCooldown` is at 0.049s, hits land at 0.067s (`Skill.5`), 0.084s (`Skill.3`), 0.334s (`Skill.4`), 0.575s (`Skill.1`), 0.584s (`Skill.2`), and 0.751s (`Skill.5`); cancel windows activate at 0.109s (`BreakSkill_Jump`), 0.852s (`BreakSkill_AnySkill`), and 0.990s (`BreakSkill_Move`). For Burst (`Adler_UltralSkill`), direct hits land at 2.185s (`UltraSkill.1`) and 2.343s (`UltraSkill.2`); cancel windows for Move, AnySkill, and Jump activate simultaneously at 3.082s (`BP_CanBreakSkill_C`).

## Action packets and owner effects

- GA_Adler_Melee admits GE_Player_Adler_Melee1_Damage Native coefficients at Normal levels 1 and 8 are 3.181 and 5.452 across the chain; category A/INCANTATION metadata stays source-bound. Normal attack receives +1 level from resonance_3 at selection count >= 3.
- Under `adler.skill_all_in_range_dummy_v1`, each living scenario-declared dummy is treated as in range and in the attack area. The model schedules one GE_Player_Adler_Skill2_Damage and two GE_Player_Adler_Skill3_Damage coincident initial DEF packets per captured target, then GE_Player_Adler_Skill_Damage at +1s through +10s inclusive; delivery rechecks target survival. The train is target-major, has no geometry/cap checks, and spends cost once. This is a product dummy semantic, not a native target-selection claim. Additional Skill2 packets do not extend the current target-zero-only energy adapter. The mapped Skill2 row and published 70% DEF x3 description disagree about multiplicity; the two Skill3 activations and exact impact/refresh timing remain explicit bounded hypotheses. The 1s/10s frozen DT_AdlerEffectFigure curves ground the periodic train. Recast replaces all previous target streams, living owners continue off-field and defeated owners or individual targets stop their own future ticks. One Skill admission preserves the Blessing grant.
- Connected normal-chain packets add one held Karma stack and aimed Liberation adds two, capped at 20; A1 Burst admission adds 20 up to 20. Skill admission captures and clears held stacks before damage. Each captured stack adds +2 percentage points to additive FinalDamageUp, or +3 points when effect 2 is selected, on every initial and periodic packet keyed to that admitted command, preserving delayed initial packets across recasts. This is an explicit tested hypothesis from optional PC_140 decoded local assets AdlerSkill_DamUp.uasset (local:cde2e6c246e3d5ed5234e1dc7741e35242c66031826484d685b8a653462e16bd) and Buff_AdlerQiHai_SkillDamUP.uasset (local:b0ff9c388aeff5a4a28ba111db07776ba95d2c939d33d31ab844f4e4398f8ade): reconstructed additive FinalDamageUp is .02 per count without A2 and .03 with A2. Their exact hotfix/precedence, Global cutoff applicability and runtime reachability are unproven. The reached calculator now explains frozen Adler_lv2=0.01 as an increment on the 0.02 base; see the Karma consumer below. No separate held-stack general A2 bonus is modeled, avoiding duplicate application of that captured Skill carrier. Other Karma producers/consumers remain pending.
- The public Burst action delivers GE_Player_Adler_UltraSkill_Damage five times to each living declared target, or ten times to the sole living target, using the exact DEF coefficient: 1.0 at level 1, 1.714 at level 8 and 1.851 at effective level 9 after resonance_3. DT_GameplayAbilityTipsData.Rows.GA_Adler_UltraSkill.AbilityDescription[0].Description specifies five Sunyas, five instances per enemy and a second strike against a sole enemy; the native damage row binds DEF/category Q/Incantation and FixedCritRate 0. adler.burst_sunyas_v1 explicitly captures living population at admission and uses strike-major target order with coincident impact times. Delivery rechecks owner/target survival and live modifiers, preserves off-field ownership, and settles separate crit/proc opportunities. One cast spends its admitted energy cost once. Native intervals and selection time remain hypotheses; a defeat during the captured train does not expand it.
- When effect 5 is selected, the public Burst model schedules exactly one callback for 20 flat Ultimate Energy to the captured Adler at the second-volley marker (2.343074s). The callback bypasses Charge Efficiency and leaves fractional carry unchanged. A defeated source receives no refund, and field changes do not redirect it. The frozen `1033AwakenEffect.json` Effect5 text and `DT_AdlerEffectFigure.Adler_lv5` establish the amount. The older packet-only model retains its explicitly authored admission-time timing.
- Native DEF delivery uses source-projected base DEF, static DEF percent and flat additions separately, then applies live DEF percent to base DEF and live DEF flat additions before generic exact mitigation, crit and damage bonuses. Percent modifiers reject when base-DEF provenance is absent; old requests without typed scaling continue to default to ATK.
- Resonance 3 activates at selection count >= 3 and adds +1 level to Skill, UltraSkill, and Melee.
- PlayerInput::Special toggles Aim mode (GE_Player_Adler_Aim_Damage); normal chain does not advance during aiming.
- On successful GA_Adler_Skill admission, a same-time callback grants the captured active actor a native flat+DEF-coefficient Blessing shield using effective Skill level (including resonance 3), cast-time effective DEF and Adler outgoing ShieldEfficiency. The shield is keyed by granter and recipient, replaces only that key on recast, persists across switching, and expires half-open; A3 adds 20% in the additive ShieldEfficiency zone as a tested hypothesis. Solifugae Vita reads positive nonexpired settled wearer shields at hit delivery and adds its frozen base-ATK-scaled ATK percentage. See adler-blessing-shield.json and q1-snowman/receipt.json.
- At hit delivery, one additive +10 percentage-point Incantation damage bonus applies to team packets while a living R6 Adler has a positive, nonexpired Blessing shield on any living actor. Off-field packet owners and active-field switches do not change eligibility; depletion, expiry, recipient death or Adler death turns it off. Multiple eligible Adler sources do not stack. This is a tested bounded hypothesis from 1033AwakenEffect.json resonance_6; the referenced Buff_Adler033_Level6_extra blueprint does not expose the native property or protected-actor scope. DamageUpIncantationBase is the matching native-style bonus property; the model uses the existing additive damage-bonus zone.
- Effect4 establishes the break trigger and a 50%-of-Adler-DEF basis. Its native settlement consumer remains unrecovered despite the now-available exports; live settlement-time DEF instead of cast-time DEF, nearest-integer rounding, one-event cap, living-source eligibility, no ShieldEfficiency/A3 multiplier, no stored remainder, and first-broken-shield ordering across multiple sources are bounded choices. The chosen rule applies after ordinary shields and before HP/overkill; exact depletion records zero absorption. Expiry/removal/replacement do not trigger, and no shield/heal/revival is created.

## Ordinary-hit Ultimate Energy

The current `adler.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 19.856, Aim 0.7, Skill 4 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipts: [`adler-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/adler-ordinary-hits.json), [`adler-continuing-energy.json`](../../../../reviews/q10-energy-ledger/adler-continuing-energy.json).

## Individual awakening selection

Effect indices gate A1–A6 individually; resonance uses selection count. `[5]` enables the Burst refund without Karma or R3; `[6]` selects Effect6’s native 12s Blessing duration without A3 or R6; `[1,2,5]` enables A1/A2/A5 and R3, without A3/A4/A6. Selection outcomes are independently tested; unsupported producers remain partial.

## Special and dash inputs

PlayerInput::Special toggles Aim mode and Aim packets use the native aimed damage row; normal-chain progression stops while aiming.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/adler-action-binding-model.json`](../../../../reviews/q10-game-action-binding/adler-action-binding-model.json).
- Owner supplements: [`adler-blessing-shield.json`](../../../../reviews/q10-game-action-binding/adler-blessing-shield.json), [`adler-normal-chain.json`](../../../../reviews/q10-game-action-binding/adler-normal-chain.json).
- Frozen inputs: `data/extracted/guide_adler.json`, `data/extracted/neverness_adler.json`, the consumed [typed native damage rows](../../../../crates/data/src/native_damage/adler.rs), `1033AwakenEffect` (cache-only awakening source), cache-only source table `DT_AdlerEffectFigure.json` (optional cached source `DT_AdlerEffectFigure`), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), `DT_GameplayEffectMappingData` (cache-only source), cache-only source table `DT_AttributeStaticData.json` (optional cached source `DT_AttributeStaticData`), [source provenance policy](../../../SOURCES.md), [source provenance policy](../../../SOURCES.md), [`reviews/q10-game-action-binding/adler-blessing-shield.json`](../../../../reviews/q10-game-action-binding/adler-blessing-shield.json).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- Remaining Karma producers/consumers and non-modeled passives; the current card covers only Normal/Aim/Burst gains and Skill spending.
- The supported Skill/Burst packet counts and resource costs are modeled, but native Skill packet multiplicity, target cap/selection and exact delivery schedule remain unanswered in the linked unknown card; the Skill all-in-range behavior is only a scenario product assumption.
- Adler A5 refund trigger is known from Effect5 text; native callback timing/applicability retains the bounded hypothesis described above.

## Client shield source availability

**PC_140 source facts, conditional on [applicability](../../shared/evidence-applicability.md).** The shield blueprints are now present in the permanent effect table (effect_parameters.jsonl); their previous unavailability is no longer a valid research blocker. `GE_Player_Adler_Skill_Shield` (`asset:54d8efe6423b691342d76fcd0b3be4393097dbb1cfeae6bc855bd30d6575b98e`) and `_Lv6` (`asset:eb7b8514ce3cc974d42480777d1ae5e0c034f667f4b7e813a1bef046ea6b7142`) have `/2/Properties/ApplicationRequirements` pointing respectively to `Con_Adler_notLv6` and `Con_Adler_Lv6`.

Both `/3/Properties` declare `DeadRemove=true`, `HasDuration`, target aggregation and limit 1, `State.Shield` and `Event.Common.OnlyServer`. They request the display effect on BEGIN and removal on FINISH; CHARACTER_UNEQUIPPED requests removal of the respective shield. The Lv6 variant also requests base-shield removal on BEGIN. Duration curves are explicitly `Adler_SkillShield_time` versus `Adler_SkillShield_lv6_time`, each with scalar 1.

The display carrier (`asset:10b7bd1d1fe91c339a0bc72d718275a46aaa40031627d8862228143f586fb68d`) declares Infinite at `/2/Properties`, despite a serialized duration magnitude of 15, and its global component has `CheckSourceDeath=false` at `/3/Properties`. Do not turn that display magnitude into shield lifetime or equate this flag with complete source-survival behavior. The [recovered cue](#shield-initialization-and-karma-copy-callbacks-recovered-october-18) now supplies initialization and cleanup requests; native shield settlement/recipient/quantization remains unresolved. The current Blessing model is unchanged.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/adler). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels:

| Action / damage row (`GE_Player_Adler_` prefix) | Native category | Scaling stat / Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_Damage` | A | ATK: 3.181 / 5.452 across chain | 19.856 total / 33.108 total / 3.936 total |
| Plunge `AirAttack_Damage` | X | ATK: 0.360 / 0.617 / 0.666 | 2.300 / 3.800 / 0.500 |
| Aim `Aim_Damage` | A | ATK: 0.104 / 0.178 / 0.192 | 0.700 / 1.100 / 0.100 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | ATK: 1.760 / 3.016 / 3.258 | 4.400 / 7.300 / 2.500 |
| Support `QTE_Damage` | H | ATK: 2.000 / 3.428 / 3.702 | 6.100 / 0.000 / 2.500 |
| Skill initial `Skill2_Damage` & `Skill3_Damage` | E | DEF: 0.700 / 1.200 / 1.296 | 4.000 / 6.700 / 0.800 (Skill2) |
| Skill periodic `Skill_Damage` | E | DEF: 0.200 / 0.343 / 0.370 | 0.000 / 0.000 / 0.000 |
| Burst `UltraSkill_Damage` | Q | DEF: 1.000 / 1.714 / 1.851 | 0.000 / 0.000 / 0.500 |

Critical Riposte uses GAName `GA_Adler_ExtremEvadeAtk`, while Support QTE uses `GA_Adler_QTE`. All Adler damage rows deal INCANTATION damage. Unlike ATK-scaling actions, Skill and Burst strictly bind `DefRateBaseArray`; their `AtkRateBaseArray` in source data is empty.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Adler_Passive_1` binds `Buff_Adler033_Passive1`. `DT_ReactionExtendDataTable` row 1033 binds `ReactionAbilityID=GA_Adler_Passive_1`, requiring `Buff_Adler033_Passive1_C` and supplying `RandomTargetGEClassArray` with 3 random target classes (`Buff_Reaction_5_new_1033_1..3`). This supports Adler's Scorch/Ignition reaction enhancement.
- **Passive 2 (Breakthrough 4):** `GA_Adler_Passive_2` binds `Buff_Adler033_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1033AwakenEffect#/rows/1033/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Casting Tranquility (Burst) grants Adler 20 stacks of Karma.
- **A2:** Each stack of Karma grants Adler +3% DMG bonus (`FinalDamageUp` additive zone).
- **A3:** Increases Blessing shield amount by 20% (`ShieldEfficiency` additive zone).
- **A4:** When Blessing shield breaks, absorbs overflow damage based on 50% of Adler's DEF.
- **A5:** Casting Tranquility grants Adler +20 flat Ultimate Energy (evaluated by `ChargeExecution20`, bypassing Charge Efficiency).
- **A6:** Extends Blessing shield duration to 12.0s (curves in `DT_AdlerEffectFigure`).
- **Resonance 3:** +1 level to Deliverance (Normal), Evil's Bane (Skill), and Tranquility (Burst).
- **Resonance 6:** +10% team Incantation DMG while protected by Blessing.

### Karma increment reconciled against its consuming graph, October 18

The initial bundle's warning-free `AdlerSkill_DamUp:CustomCalculateBaseMagnitude` (`#/2`) tests `Buff_Adler033_Level2` on **InstigatorActor ASC**. Without it, successful lookup of **Adler_SkillDamUp=0.02** multiplies by the ASC's `GetGameplayEffectCount(Buff_AdlerQiHai_SkillDamUP_Count, null, true)` (statements 382..720). With it, the graph reads **Adler_lv2=0.01**, adds that to the same **0.02**, and multiplies the sum by the returned count (752..1165). The 0.01 row is therefore an **increment**, not a competing claim of 1% total per stack. The earlier prose calling that row a coefficient conflict is superseded by this located consumer.

For an explicit returned count 20, the source expression gives **0.40 without A2**, **0.60 with A2**; treating the increment as the whole rate would incorrectly give 0.20. The existing bounded model's 2%/3% rates agree with this algebra, while native count semantics, capture before/after consumption, carrier reachability and final modifier application remain open. Do not promote the new general "+3% DMG per held stack" summary into a second unconditional bonus in addition to the captured Skill carrier.

### Passive description coverage, reviewed October 18

Saved description source: `guide_adler.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Temperance describes three possible Scorch debuffs for 15s: ATK -20%, Esper RES -10%, or Break efficiency +10%, without stacking the same debuff. Righteous Heart describes self DEF +20%. RandomTargetGEClassArray locates three candidates, but does not prove equal selection probability, coexistence between different candidates, proc frequency or native shield/Break interactions.

### Righteous Heart source stat recovered, October 18

The initial `Buff_Adler033_Passive2#/1/Properties/Modifiers` supplies additive **DefUp**, multiplier 1, through `DT_AdlerEffectFigure:Buff_Adler033_Passive2=0.2`; the carrier is Infinite, AggregateByTarget, limit 1, NeverRefresh/NeverReset. This resolves a 20% base-DEF contribution rather than 20% final DEF or outgoing damage. At base DEF 1000 it supplies 200 DEF before other zones. Selected ability configuration already identifies this passive; initialization/death behavior remains a lifecycle question.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Adler_Skill#/14; Skill.2 | PlayerAdlerSkill | (0.1, 0.0, 0.0, 11.0, 1, true) | GE_Player_Adler_Skill_Damage, GE_Player_Adler_Skill2_Damage |

Skill.2 requests a single actor container containing both Skill_Damage and Skill2_Damage, alongside Float. This locates both candidate packet classes at a reached event. Native per-effect application conditions still determine whether both settle on a recipient; the two class names alone do not authorize summing both coefficients.

### Shield initialization and Karma copy callbacks recovered, October 18

Both **GE_Player_Adler_Skill_Shield and _Lv6#/3** declare `GameplayCue.Display.Adler.SkillShield`, now joined to warning-free **GC_Adler_SkillShield**. **OnWhileActiveEvent#/5 → graph#/1:2052** validates MyTarget, caches its ASC, binds the ASC shield component's **OnShieldHealthValueChange**, subscribes to effect-time changes filtered by **Ability.Player.Adler.ShieldTag**, then reaches shield initialization. The same initialization is reached by the time-change callback **#/2 → 1276**; this is not a once-only cast snapshot in the Blueprint.

Initialization casts **MyTarget to player**, obtains that player's net ID and **K2_GetCharacterAbilityLevelWithNetID(..., GA_Adler_Skill)** return value, and evaluates **Adler_SkillShield_cofficient / Adler_SkillShield_base** at that coordinate. It reads **CachedASC.GetDef**, then requests **InitializeShieldHealth(self, base + coefficient × DEF, additional)**. Without **Buff_Adler033_Level4 on CachedASC**, additional=0. With that marker and a successful **Adler_lv4=0.5** lookup, additional is a **separate** GetDef read ×0.5, passed as the third argument. Curve inputs at Skill coordinates 1/8/9 are coefficient **0.8/1.37/1.48**, base **1000/1714/1851**. At coordinate 1 and a stable DEF 1000 the two call arguments are **1800 and 500** with Level4, or **1800 and 0** without it. This does not prove those two native pools sum to 2300, nor incorporate ShieldEfficiency or rounding; those remain native initialization/absorption semantics.

The explicit receiver for DEF, level and Level4 is therefore the **cue target's ASC/player**, not a located source-Adler lookup. This materially differs from treating the existing model's source-Adler basis as recovered native execution. Effective cue-target assignment, source sharing, refresh behavior and native stat/pool settlement must resolve that discrepancy before changing the model. The graph does not itself require a server role or multiply by ShieldEfficiency.

After either shield-initialization branch, a separate **CachedASC Level6_extra** gate requests **Buff_Adler033_Level6_ZhouDamUp on MyTarget**, with MyTarget also SourceObject. Its warning-free carrier (`#/2`) adds **DamageUpIncantationBase=0.1**, HasDuration 60, target cap1. OnRemove **#/3 → 2482** unbinds the shield-health delegate, calls **CleanUpShieldHealth(self)** on MyTarget ASC and removes this damage carrier through CachedASC. The reached body does not end the time-change task explicitly. Health callback **#/4 → 2731** requires **NearlyEqual(CurShield,0,tolerance 1e-6)** and **ShieldActorName equal the cue's class display name converted to Name**. It then removes **ordinary GE_Player_Adler_Skill_Shield**, its display carrier and the damage carrier. It does not explicitly name _Lv6 for removal, test incoming damage, or distinguish depletion from another native zero notification. Preserve this asymmetry and avoid claiming every zero callback implements the model's break/refund ordering.

The separate **Buff_AdlerSkill_CutCount#/1 → GC_Adler_Skill_CutCount** tag join supplies the missing Karma-copy request. OnWhile casts MyTarget to ability character, reads **GetGameplayEffectCount(QiHai,null,true)** from its ASC, requests **SkillDamUP_Count** there with that count as nStackCount, then **SkillDamUP** with default stack0. The latter carrier's BEGIN removes all QiHai; the cue does not itself atomically capture-and-clear at action admission. Count is target cap 20; both copied-count and damage-bonus carriers have duration multiplier **1.2 × Adler_SkillDam_Time(10)=12**, while the triggering CutCount input is **0.033**. Native admitted stacking/context and delayed packets/recasts remain open; the earlier per-command snapshot is still a tested model. A captured count 20 supplies **0.4 FinalDamageUp**, or **0.6** with Level2 under the already recovered calculator, rather than multiplying damage by20.

Sources: `adler-cue-consumer-carriers.txt`, `roster-cue-halo-extraction.json`, `cue-carrier-joins.json` and **DT_AdlerEffectFigure**. The cosmetic SkillShield_Display graph adds particles/audio on removal and does not settle a separate break benefit.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Adler_Skill`) and Burst (`Adler_UltralSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Skill packet multiplicity:** Source text and Reference guide cite 70% DEF ×3 for the initial hit, while the mapped `Skill2_Damage` and `Skill3_Damage` rows suggest a two-activation pattern; the current executable model uses an explicit bounded hypothesis.
- **Burst single-enemy detection:** Burst delivers 5 strikes per enemy or 10 strikes to a sole living enemy; the exact timing of living-enemy population sampling remains an admission-capture hypothesis.
- **Spatial geometry & targeting:** Multi-target recipient distribution and Blessing shield recipient selection use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- Skill count/recipient behavior: source prose says one initial event and Reference guide reports 70% DEF ×3, while the mapped Skill2/Skill3 rows support a different activation pattern. The executable model now applies a separate all-in-range dummy product assumption; it does not resolve native count, target cap/selection, or exact impact/refresh times. Resolve those questions only with Global-cutoff-bound execution evidence; spatial geometry stays outside product scope.
- Burst: native spacing and the one-enemy branch population decision time are unknown. Five strikes per eligible enemy and ten for one enemy are source-backed description rules; admission-capture and coincident strike order are model choices.
- A5: the +20 flat Ultimate Energy amount is source-backed; callback admission timing, living captured-owner gate, and flat restore behavior need applicable Global execution evidence.
- Karma: the consuming source graph resolves the base/increment arithmetic to 2%/3% per returned count. Native count interpretation, read/consume timing, applicability and carrier reachability remain open; the 0.01 increment is no longer an unexplained conflict.
- Blessing: Effect4 break trigger and 50%-of-DEF basis, and R6 protected-shield condition/+10% Incantation bonus, are source-backed text. The shield blueprints and configured condition/removal/tag/duration fields are now available in the known card. Settlement rounding/order, native recipient/cue behavior and survival semantics remain tested hypotheses; the configured removal flags alone do not settle them.
- Ordinary-hit energy source constants are frozen; caller eligibility/cadence/recipient behavior remains the named tested hypothesis in the cited receipt.
- Stamina, cancel windows, invulnerability, and Critical Dodge are unsupported; do not infer them from aim/form admission.

## Prior findings

- [`reviews/q10-game-action-binding/adler-action-binding-model.json`](../../../../reviews/q10-game-action-binding/adler-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [`adler-blessing-shield.json`](../../../../reviews/q10-game-action-binding/adler-blessing-shield.json), [`adler-normal-chain.json`](../../../../reviews/q10-game-action-binding/adler-normal-chain.json).

## Research history

- Adler is8/8 with the complete handoff above. Next owner/block: **Aurelia/packets**. The full184-block collection still precedes simulator implementation.
- Already used: selected action/montage/effect/calculator/cue and projectile/container graphs, previous native/default attempts, current Reference guide/Neverness kit, independent cycle analysis. Current-source Normal4's third damage event and the ordinary Scorch cap1 correct concrete historical-model conflicts; do not reopen accepted blocks for optional native certification.
- source certainty, model readiness and current executable coverage remain distinct.

## Typed consumed Skill constants

The existing Skill train and Blessing now read [typed owner constants](../../../../crates/content/src/characters/adler/values.rs):13 flat/DEF coefficient pairs, a1-second tick interval and a10-second train lifetime. These replace only the four consumed `DT_AdlerEffectFigure` rows. The other12 rows and raw curve metadata stay outside Git. No new Karma, shield-interception or passive branch is implied by this storage change.

Independent DEF1000/no shield-bonus checkpoints give talent1 `1000+.8*1000=1800`, talent8 `1714+1.37*1000=3084` and talent13 `2519+2.01*1000=4529`. A3 and outgoing shield-efficiency settlement remain separate existing operations. Levels0/14 reject instead of supplying a fallback; exact ratios, live DEF capture, deadlines and scheduled packets retain their existing owner behavior.

## Relative combat timer binding

`Adler_UltralSkill` Common.EndAbilitySkill at 3.025935 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
