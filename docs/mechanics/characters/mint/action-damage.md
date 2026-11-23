# Mint action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1019`.

**Responsibility:** Mint outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** the native clock executes the selected ordinary, held and secondary mechanics, including paid controls, qualified Support and continuing cancellation/resource state. Native source integration executes this selected model; tenacity/incoming declarations remain Q4-L5; historical direct-packet slices below retain their narrower scopes.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Complete packet specification

Accepted research specification, October 18. This section selects implementable rules; historical executable slices below remain evidence of their narrower behavior. **Source:** `DT_SkillDamageData/rows/GE_Player_Mint_*`, selected `GA_Mint019_Melee`, `GA_Mint019_Skill`, `GA_Mint019_UltraSkill`, `GA_Mint019_QTE`, `GA_Mint019_PerfectEvade` event maps, and the [alternate-action investigation](#alternate-actions-and-packet-inputs). All listed coefficients multiply live final **ATK** once, use Anima/NATURE and ordinary seeded per-packet critical sampling. Apply [shared damage zones](../../shared/damage-and-stat-zones.md); categories A/E/Q/X/Z/H remain distinct. Level columns below are 1 / 8 / 9. Every intermediate released level is explicitly available in Neverness extraction `skills/*/stats/*/valuesPerLevel`; native coefficient values, rather than interpolation between these three columns, take precedence at any supported coordinate.

| Action | Ordered damage row suffixes after `GE_Player_Mint_` | ATK fractions, level 1 / 8 / 9 | Count/category |
| --- | --- | --- | --- |
| Normal 1 | Melee1_Damage_New, Melee2_Damage_New | .104+.231 / .178+.396 / .192+.428 | 2/A |
| Normal 2 | Melee3_Damage_New | .170 / .291 / .315 | 1/A |
| Normal 3 | Melee4_Damage_New, Melee5_Damage_New | .262+.341 / .449+.584 / .485+.631 | 2/A |
| Normal 4 | Melee6_Damage_New | .184 / .315 / .341 | 1/A |
| Normal 5 | Melee7_Damage_New, Melee8_Damage_New | .255+.523 / .437+.896 / .472+.968 | 2/A |
| Approach/entry Normal | same Normal-1 pair | same Normal-1 values | alternative 2/A, never added to Normal 1 |
| Hold opener | Evade1_Damage three times, Evade4_Damage once | 3*.283+.467 / 3*.485+.800 / 3*.524+.864 | 4/A |
| Hold spin | Evade5_Damage | .133 / .228 / .246 each | at most 30/A, stop on cancellation/exhaustion |
| Plunge landing | AirAttack1_Damage | .650 / 1.114 / 1.203 | 1/X |
| Critical Riposte | PerfectEvadeAttack_Damage | 1.740 / 2.982 / 3.221 | 1/Z |
| Redirect Skill | Skill1_Damage_New twice | 2*.900 / 2*1.542 / 2*1.666 | 2/E |
| Burst | UtraSkill1_Damage_New, UtraSkill2..6_Damage_New, UtraSkill7_Damage_New | 1.326+5*.588+3.735 / 2.273+5*1.008+6.401 / 2.454+5*1.088+6.913 | 7/Q |
| Support | QTE1_Damage | 2.000 / 3.428 / 3.702 | 1/H |

**Inferred, geometry abstraction:** plunge accepts an explicit scenario `fall_damage_bonus` in [0,1] and applies `(1+fall_damage_bonus)` to its packet base. This is the description's height-dependent bonus, represented directly because position/fall geometry is outside the dummy model. The minimal-height scenario uses 0, the stated maximum uses 1; intermediate values must be declared scenario conditions, not an invented meters-to-damage curve. Air descent itself deals no packet; landing does. These endpoints distinguish +100% (2x) from a mistaken +200% (3x). Geometry-to-bonus conversion is an accuracy note, not an implementation dependency for declared fall conditions.

**Derived checks, before rounding:** at level 8 the Normal chain sums to **3.546 ATK**, Skill **3.084**, Burst **13.714**, Support **3.428**, and full hold **9.095**. At ATK1000 and DEF factor .5, no crit/bonuses/RES, these produce **1773, 1542, 6857, 1714, 4547.5**. Plunge endpoints produce **557 / 1114**. A hold stopped after three spin packets is **2.939 ATK**, not 9.095. Every damage packet is separate for live modifiers, target HP thresholds, energy and gear callbacks; a full train cannot be collapsed to one coefficient.

**Source plus applicability inference:** the selected kit has no additional autonomous damaging summon or damage-over-time carrier. Wind is part of the held action. The unbound `Buff_MintActor_PasEffect`/`GA_MintActorWind_Skill` route is an alternate archived candidate, not an extra concurrent packet source: there is no producer in the reached action/cue graphs, and adding it would contradict the selected two-hit Skill and 30-entry hold maps. Keep its raw findings below without admitting an extra train. HitBack/Float, display actors, QTE2..4 and test/legacy damage rows likewise add no selected damage packet. This absence decision is bounded to the selected released kit, not a claim that those assets never existed.

## Effect and passive specification

**Mixed basis.** Magnitudes, event bindings, requirements and conflicts are source facts from [awakening investigation](#awakening-conditions-recipients-and-conflicts), [defensive passive](#defensive-passive), [Blossom enhancement](#blossom-enhancement), `1019AwakenEffect1` and `DT_MintEffectFigure`. The following lifecycle choices close the formerly open gameplay decisions. They are research models requiring implementation; the existing runtime's partial/strict-threshold receipts are preserved.

| Effect | Complete modeled lifecycle and recipients |
| --- | --- |
| A1 Task Force Operation | On a **connected Critical Riposte damage hit**, if Mint's Skill cooldown is running and A1 is selected and the reset guard is absent: remove that Skill cooldown and add a self guard for **3 World seconds**. First target callback consumes the opportunity; further recipients cannot reset again or extend the guard. The .3s target carrier is a transport lifetime, not the reset guard. NeverRefresh/NeverReset apply to the guard. A miss or dodge alone does nothing; expiry of cooldown before the hit means no reset/guard. |
| A2 Re-examination | While selected, each of the two Redirect Skill packets adds **.40** to its ordinary damage-bonus zone. No opener/spin/Support/Riposte/Burst application merely because a name contains Skill. |
| A3 Attendance Bonus | Any party member's successful **Blossom or Hexed activation** gives Mint **.15 ATK% for 15s**, one self instance; reapplication refreshes expiry, never stacks magnitude. Choose the matching EN/CN/public description over the conflicting parent QTE_BEGIN event, consistently with the already authorized reaction-trigger model. QTE without either reaction gives no bonus; a reaction without Mint QTE does. Active off field, removed on Mint death; subsequent reactions while dead grant none. |
| A4 Lunch Break | At each Mint packet, query scenario-declared nearby eligible enemy dummies, unique by target index. Take at most 10, as the source query config does; if returned count <=1, Mint gets **.12 CRIT Rate** for that packet. Baseline all-in-range dummies count as nearby; an explicit proximity condition may select a narrower set. Zero also satisfies the graph but produces no damage without a recipient. Two dummies disable it. This replaces the historical global-single-target approximation in the new specification. |
| A5 First Instinct | On Skill **Display.1** (before first hit, whether or not it hits), if A5 selected, give Mint **.25 additional CRIT DMG for 6s**, source-owned cap1, refresh duration on reapplication. Persists across field exit. Does not trigger from hold's Common.Display. Death removes it. |
| A6 Elite Member | Selected A6 adds **.15 general DMG bonus** to Mint-tagged damage against a living target with **current/max HP <= .40 immediately before that packet**. The explicit LessEqual graph includes .40. A prior hit can activate the bonus for a later one. |
| Resonance 3 | At least three selected awakening identities: +1 effective Normal, Skill, Burst level, within native available coordinates; Support level is unchanged. Exactly two selections give no increase. Do not gate on A3 identity alone. |
| Resonance 6 | Six selected identities: each Skill Display.1 gives Mint's own Cycle pool **+2**, once per cast and independent of hit success. No Ultimate Charge and no automatic team grant. The calculator's unused local 50 contributes nothing. |
| Passive 1 | At breakthrough2/unlocked: Mint's party presence extends each allied Vita Pistil's damage radius by configured **400 source units**; no coefficient, extra bud or additional pistil. Stays active off field. Apply once, not once per hit or stack. Under all-in-range dummies no outgoing damage changes; an explicitly narrower proximity scenario can differ. |
| Passive 2 | At breakthrough4/unlocked: on Mint entry (including initial active assignment), apply **.20 DEF%** and **30% interruption resistance**; remove on field exit/death, do not stack on repeated entry. Initially benched Mint has neither. It does not add outgoing damage. Incoming attacks/parries use an explicit scenario outcome; the resistance does not create automatic immunity. |

**Inferred lifecycle rationale and counterchecks:** descriptions name singular timed buffs, and source children select the corresponding stat rather than a stack-dependent magnitude; cap1 plus refresh is the conservative nonmultiplicative reading. A3 at t0,t10 therefore yields .15 through t25, not .30 through t15; A5 at t0,t4 yields .25 through t10. The A1 guard's explicit NeverRefresh is the exception: hits at t0,t1 leave the original t3 guard expiry. Applying A5 before damage honors the Display.1→Melee.1 order and produces a crit benefit on a first successful hit even if a prior cast missed. Alive-only admission and cleanup avoid retaining owner effects after removal while preserving off-field descriptions.

**Independent zone checks:** base ATK1000 plus existing .50 ATK% and A3 is **1650**, not 1725. General .20 plus A2 .40 gives **1.60**, not 1.68. General .20 plus A6 .15 gives **1.35**. Additional CD .50 plus A5 .25 gives critical factor **1.75**. CR .95 plus A4 .12 clamps to1, not1.07 probability. Passive2 changes DEF1000 to1200 and does not change an ATK-based packet. There are no other selected combat passives/resonances in the ability/awakening lists; city life skills have no dummy-DPS trigger and contribute none.

## Timing and continuation specification

**Mixed source/profile model, `mint.source_schedule_v2`.** Source notification offsets below come from the selected `Mint_Melee_001..005`, `Mint_PreMelee`, `Mint_AppearMelee`, `Mint_Skill_01`, `Mint_Skill_001_Begin/End`, `Mint_UltraSkill`, `Mint_QTE`, `Mint_PerfectEvadeAtk/Back/Front`, `Mint_AirAttack/End` montages. The historical [timing investigation](#skill-and-burst-timing-investigation) has already exhausted static/native and public approaches; another export does not resolve real rate. For an implementable profile choose nominal animation rate **1 source unit per real second**, as in the previously accepted Skill/Burst profile, with the explicit hitstop transform below. This is a disclosed timing inference, not measured frame timing or an assertion about the native default. Preserve exact decimal offsets, convert to microseconds once using nearest/ties-up, and use one real simulation timeline for the denominator.

| Action/section | Nominal damage offsets in source units from section input | Earliest next ability / movement | Earliest jump |
| --- | --- | --- | --- |
| Normal1 | .15463588, .30972266 | .83232725 / .8833587 | .33334768 |
| Normal2 | .08951393 | .66574407 / .78354454 | .24849394 |
| Normal3 | .24190265, .37875503 | 1.1834577 / 1.2852283 | .51673234 |
| Normal4 | .14993508 | .7160012 / .7985704 | .16646957 |
| Normal5 | .25931817, .6000772 | .98615026 / 1.6164002 | .6682273 |
| Approach Normal | .20584701, .36020482 | .83131856 / .91683537 | .45083547 |
| Entry Normal | .19989, .350109 | .776779 / .883359 | .416638 |
| Skill | .20017847, .5159236 | .8683227 / 1.2020172 | .6846825 |
| Burst | 1.6159108, 1.754891, 1.9379535, 2.143918, 2.2606597, 2.4272692, 3.499036 | 3.9079924 / 3.9079924 | no ordinary jump window before recovery |
| Support | .7394236 | 1.3667033 / 1.3667033 | 1.3667033 |
| Riposte Atk | .22721595 | .44350475 / .44350475 | 0 |
| Plunge landing | .06451421 after landing | .33345124 / .86579 | .21685006 |
| Hold opener | .302511, .601565, .730626, 1.196146 | Skill/any ability from 1.8187463 / normal ending 6.049112 | 0, then 1.2763493 |
| Hold End, after release/exhaustion | no damage | .61663866 / .61663866 | 0 |

**Derived spin schedule:** for k=0..29 use the following Skill.5 offsets (not a guessed tick rate): `1.4329805,1.5673442,1.6999723,1.8319359,1.9661987,2.099678,2.2312322,2.3666878,2.4997375,2.632322,2.7670934,2.8991168,3.033328,3.165678,3.299132,3.4328017,3.5677314,3.699915,3.8334503,3.9669638,4.1006613,4.2325974,4.366074,4.50004,4.6329565,4.767249,4.9007025,5.0329213,5.1667,5.2997556`. The source hold timer expires at5.41 and requests End; Skill.7 is at5.425367. Select5.41 as the action's natural end-section transition; the final spin hit at5.2997556 precedes it. End then permits the next ability after .61663866. The public five-second spin/tooltip six-second duration are rounded descriptions with different boundaries; do not stretch 30 hits into a new invented tick train.

**Timer consistency inference:** SkillTriggerEnd's5.41 is measured in this hold's accumulated action progress, while A3/A5/guard/cooldown deadlines use World time. Hitstop slows that progress; the reached player-tenacity cue additionally pauses **only SkillTriggerEnd** while its pause request is held, restoring the remaining duration on unpause. Its .55s carrier lives in real time and its .01 actor-dilation request combines with hitstop using the minimum rate. Do not pause Ultimate/Cycle or all buffs. With three connected opener hitstops, the last spin is at5.4482556 real seconds and natural End begins at5.5585; both retain their source ordering. A real5.41 timer would delete that last configured packet and contradict the selected complete train, so it is not used. Nested calls use a pause-source set: one unpause cannot erase another source's active pause. Release/switch still cancels the timer even while paused.

**Inferred hitstop integration:** a connected packet with explicit AttackDilationTime `d`, rate `q`, and optional DelayTime `l` adds an actor-local interval `[impact+l, impact+l+d)` with animation progress rate `q`; otherwise progress is1. Real cooldowns, buff lifetimes, other actors, already spawned independent effects and the DPS denominator continue. Overlapping intervals on the same actor use the minimum rate and expire individually. A multi-target event requests one interval, not one per dummy. Current explicit `(d,q)` inputs are Normal1 first hit `(.0167,.01)`, Normal2 `(.03,.01)`, Normal3 second `(.05,.01)`, Normal4 `(.06,.01)`, Normal5 second `(.06,.01)`, each Skill hit `(.05,.01)`, each hold Evade1 `(.05,.01)`, Riposte `(.3,.01)`, Support `(.4,.2)` delayed `.1`. Selected other child/parent CDOs do not supply a positive AttackDilationTime; the bounded profile makes no additional interval from an omitted duration (including Burst7's rate-only .15). This is an explicit default inference, not proof that the native default is zero. Rationale: use all reached nonzero pause inputs without inventing an unbounded native duration; the public Nanally trace proves that a generic asset-time identity is not an observed universal rule, so this profile must remain labeled.

**Independent timing checks:** connected Skill hit2 is at `.5159236 + .05*(1-.01) = .5654236` real seconds; recovery is `.8683227 + 2*.0495 = .9673227`. A miss leaves .5159236/.8683227. Ten dummies do not make recovery `.8683227+20*.0495`. A full Normal chain's explicit hitstop loss is `.99*(.0167+.03+.05+.06+.06)=.214533`; chaining at the table's next-ability boundaries takes `4.38368048+.214533=4.59821348s`. Skill cooldown starts at the reached TriggerCooldown coordinate **.18279111**, lasts **6 World seconds**, and continues off field. Burst spends/starts its **15s** cooldown at admission. These start policies are inferred from the source notify/commit distinction and prevent a cancelled pre-notify Skill from starting a cooldown. Skill cancelled after the notify keeps its cooldown; A1 can remove it. The configured cancel curve3 is retained as a source input, not a separate automatic three-second cooldown added to an ordinary full cast.

**Cancellation/switch model:** next ability uses the AnySkill boundary above, movement uses its own boundary, and jump uses its own boundary. Switch follows the next-ability boundary except explicit CantChangePlayer intervals: Burst blocks switch until **3.7921667**, Support until **1.3332379** nominal units; switch may then occur before recovery and cancels the old montage. This distinction avoids assuming AnySkill means switch or montage length means recovery. For other actions no separate positive switch lock was found; conservatively use their next-ability boundary. A legal cancel, field exit or external interruption removes future **montage-owned** notifies/packets, keeps delivered damage/resource gains, and performs hold cleanup. A packet already dispatched at the same instant settles before authored switching; a later unissued melee hit does not follow the owner off field. Independent already-spawned reaction actors retain their source. No payment is refunded after commitment; failed admission mutates nothing. This inference follows the shared OnInterrupted→EndAbility path and distinguishes a pending notification from a launched actor.

**Continuity:** Normal stages loop1..5. Another ability, approach/entry replacement, death or field exit resets to1; a same-owner no-op switch does not. Keep the prior explicit input-start gap rule (>2s resets, exactly2s continues). A held Basic reaches Begin after its section-specific hold threshold in [hold selectors](#hold-selectors-and-critical-response-routes-october-18); treat the listed requirement as excluding re-entry during WindEnd rather than requiring that exit marker to start the first hold. This inference permits an initial hold and prevents recursive re-entry during exit. Earlier ordinary hits are not retroactively erased when the hold threshold is crossed.

### Executable ordinary-clock dependency

`mint.ordinary_clock_v1` binds the grounded Normal/Skill/Burst coordinates above
through the existing exact owner-local hitlag kernel. Every distinct causal
packet expands across living eligible stationary dummies; one connected group
adds its pause once, while zero HP loss adds none. Normal input-start gap rules,
the existing live damage/energy/Cycle callbacks and passive handlers remain
shared with the current engine. Skill's182791us notify starts its real6s
cooldown; explicit interruption before it takes the separate3s cancel cooldown,
and interruption afterward preserves that original deadline. Pending owner
deliveries cancel, while only the still-live part of an already caused pause
carries into the replacement montage.

`simulation/tests/mint_clock.rs` checks a connected4598213us chain, two
continuing chains on one/three dummies, zero-loss recovery without pause and
pre/post-notify interruption. A replacement Normal at220000us after the first
Skill contact inherits29.87622ms lost progress and connects at404513us. Two
zero-energy60s periods each contain nine ordinary chains, two Skills and one
paid100-energy Burst:113.291 raw Charge per period, final carry291/500,
200 spent energy and332 packets/103592 noncritical damage across two dummies
at1000 ATK and.5 DEF. No resources, cooldowns or pending state reset between
periods. The existing native CLI aggregation/replay boundary also exercises
the new model and its distinct200178us/565424us Skill contacts.

The native clock now also admits canonical held Normal through
`mint.hold_attack_v1`. Reached Normal contacts survive the section-specific
threshold, with its caused pause shifting Begin rather than erasing the prefix.
The four opener contacts and thirty finite spin opportunities use the same
owner clock. Each spin pays exactly2 times the declared account use efficiency
before issuing any recipients, including a miss. An unaffordable opportunity
keeps its remaining stamina and enters End. Wind suppresses recovery from
1416667us hold progress until exit, preserving the real2s spend deadline and
other source-owned recovery gates. Release/exhaustion/natural expiry enter
616639us End once; redirect, interruption, field exit and death cancel future
hold notifications and release only this action's gate. Two held cycles retain
their real Normal prefixes,72 packets per dummy,60 paid opportunities and
56.920 raw Charge without resetting the account or fractional energy carry.
The nested stamina check also retains an unrelated gate and a failed fractional
payment without refreshing the spend deadline.

Native `tenacity_events` declare the selected .55s player cue explicitly,
without deriving stagger from incoming damage. Each application owns a separate
source: actor progress uses the minimum .01 rate while only SkillTriggerEnd is
fully paused. Expiry releases that source; release/interruption cancels the
hold timer without cancelling a still-live cue. The two overlapping applications
at3/3.2s cover a .75s union: animation loses .7425s, the named timer loses .75s,
and the held End completes at7341672us. Native checks preserve all thirty paid
spins and replacement-animation inheritance. A Skill's real6182791us cooldown
deadline survives the cue and held interruption unchanged; the next Skill
connects at6382969/6748215us. CLI aggregation/replay transports
the same cue declarations. Source lowering remains with Q4-L5.

Native secondary routes use this same clock. Real unqualified field entry arms
the next Basic's entry prefix; an explicit `approach_inputs` selection uses the
approach prefix without geometry. Both replace stage1 and advance to2; a
same-owner switch does not reset either chain state or the entry opportunity.
Paid Dodge keeps its shared15-stamina cost and900000us cooldown independently
of Front/Back gates. Front Skill/Jump gates are340205/82544us; Back Skill and
movement gates are448540/797188us. No earlier Back Jump bound is selected, so
it conservatively uses the full998398us recovery. Successful explicit parry
during a paid Dodge requests Back after20000us, without another payment,
Riposte packet or resource grant.

A new admitted Jump opens one flight. Plunge requires positive declared
landing delay and height bonus0..1, then delivers at64514us after touchdown;
its333451us recovery and216850us Jump gate begin at touchdown. This bounded
flight model requires that declared landing before another grounded input;
explicit interruption, field exit and defeat can clear the flight. It does
not infer an ordinary Jump's landing or motion. Entry/approach hold thresholds
are450000us; the plunge threshold is530000us after touchdown. A short release
retains reached prefix contacts without entering the paid spin.

`simulation/tests/mint_clock/secondary.rs` independently checks two paid
Ripostes at1627216/11627216us,1491 noncritical damage per dummy and28.4 raw
Charge with two A1 resets and30 total stamina. Two declared flights connect
at664514/1381364us, with height endpoints giving557/1114 damage at the same
1000-ATK/.5-DEF oracle. Qualified switching produces exactly two Support
contacts at739424/5739424us,1714 damage per dummy, and pays the outgoing
owners'100 Cycle; an unfunded third entry produces no Support. Its delayed
contact pause gives1653238us switch admission versus1686703us full recovery.
Exact Normal Jump gates, rejected early controls, landing cancellation and
parry's delayed Back gate preserve settled packets and payments. Existing CLI
aggregation/replay also covers two continuing Dodge/Riposte/flight/approach
periods without resetting state.

Native source now initializes this clock before party entry, including an
initially benched Mint. `attack[approach=true]`, paid Front/Back Dodge,
Critical Riposte, explicit Jump/Plunge, landing hold and family input windows
lower into the same native state. Pending landing hold keeps default source
completion beyond ordinary Plunge recovery; a release during the second opener
pause retains31065us*.99 lost progress in End, completing at2347394us.
`simulation/tests/source_mint.rs` preserves saved-source roundtrip, full native
replay, actual occurrence completion and typed UTF-16 diagnostics. The legal
[Mind Royale/Fireflies baseline](../../../../examples/mint-native-cycles.nte)
runs one/two continuing60s periods on one/two dummies:83 packets per dummy per
period,100 energy paid,31 whole overflow and291/1000 carry per period. Compiled
ATK is(596+555)*1.75+420=2434.25; first-hit noncritical damage is
.178*2434.25*1.10*.5*.8=190.65046. Two source held inputs preserve120 stamina
spent and72 packets per dummy. Source parry yields at468540us and its following
Skill connects at668718us. Qualified switching retains two Support contacts and
an unfunded ordinary entry. The frozen scheduled Studio adapter retains its
separate timing profile; native and browser source execution share owner rules. Source-coordinate rates,
contact, conservative flight and interruption policies remain explicit checked
hypotheses rather than measured native playback.

## Alternate-state specification

**Mixed source and inferred state machine.** Grounded Normal, approach Normal and entry Normal use the packet/timing tables; the latter two replace stage1 and advance to stage2. Stationary dummy baseline chooses ordinary Normal1 without a movement approach. Airborne Basic enters descent with no immediate damage; the scenario declares landing time (must be after admission) and fall bonus, then schedules the one landing packet. This represents spatially variable descent as an explicit scenario condition rather than a zero-time teleport.

Hold has `Begin -> Spin -> End -> Idle`. Holding from a valid Basic section crosses that section's recorded threshold, schedules the four opener hits, enters Wind/energy-stop at nominal1.4166666, then requests at most the30 spin packets. Releasing early transitions immediately to End, removing all future opener/spin notifications; exhaustion does the same before the unaffordable spin packet. Successful Redirect Skill cancellation at/after1.8187463 goes directly into that Skill, with no hidden extra End recovery. Other allowed ability cancellation follows the same boundary. A jump uses the table's own window and does not award a second plunge without a subsequent landing. Natural5.41 expiry enters End once; neither the legacy six-second tooltip nor100s fallback grants extra ticks.

On every hold exit remove SkillTriggerEnd, Energy/EnergyStop and Wind; apply WindEnd once, retain it for the exit section and clear it on exit-section completion or an admitted replacement action. The source .1 SkillEnd carrier is an exit signal, not an extra damage packet, stamina rebate or .1s global action delay. Field exit removes these markers immediately and cancels future hold damage. A second hold creates a fresh cast instance; it does not reset stamina/cooldowns or revive old scheduled entries. Two owners cannot share Mint's Wind carrier by name.

Critical Dodge and successful parry are explicit scenario events, distinct from an ordinary dodge. A Critical Dodge arms one Riposte input opportunity; the next Basic while that event is current consumes it and requests Atk, rather than Normal1 plus Atk. A parry route's source request delays **.02** before PerfectEvadeBack; Back permits Skill at.44853982, movement at.79718804, any ability at.9983977. PerfectEvadeFront permits jump at.082544, Skill/falling at.34020537 and any ability at1.0040848. Neither Front nor Back has a damage event in the selected montage, so neither grants the Atk packet on its own. Model the opportunity as consumed by the next admitted owner action or field exit; repeated Basics cannot multiply one success. An explicit Atk/Riposte scenario may directly select the damage response after a successful Critical Dodge. Parry resource carriers are not unconditional grants for pressing dodge. This event-scoped admission avoids inventing an unobserved response-window duration.

Support is a **qualified switch-in**, not an ordinary free button; it uses exactly one Support packet and becomes Mint's active action. Ordinary switch-in uses entry behavior only when a Basic is actually requested. There is no separate Mint form lasting after Burst, no charge/recast for Skill, no autonomous damage summon and no persistent ordinary attack while benched. A1 resets the one Skill cooldown; it does not create a free second charge. Off-field A3/A5 and the party Blossom passive persist under their effect rules, while all other future behavior remains attributed to its real producer.

## Resource and admission specification

Source `.mint.mods.riposte_guard` returns the remaining World seconds of the
existing A1 Skill-reset guard; `.mint.riposte_guard.active` reads its half-open
deadline. Both are initially zero. Only the native positive Critical Riposte
callback creates the3s guard; a query does not pay stamina, grant a Riposte or
reset Skill. [Rust source checks](../../../../crates/content/src/simulation/tests/source_forms.rs)
perform two continuing paid Critical Dodge/Riposte sequences, check the native
reset callbacks and exact guard expiry, and reconcile native replay.

The managed `.mods.attendance`/`.attendance.active` and
`.mods.crit_damage`/`.crit_damage.active` bindings read the actual A3 and A5
buffs on living Mint. A3 participation in Blossom/Hexed refreshes its15s ATK
timer; A5 Skill refreshes its separate6s CRIT DMG timer. Queries do not recreate
these effects. [Source checks](../../../../crates/content/src/simulation/tests/source_managed_modifiers.rs)
perform two qualified switches and two continuing Skill casts, preserving their
independent deadlines, exact expiry and seeded native replay.

**Mixed basis:** native rows below, the [declared-build resource model](../../shared/energy-and-gauges.md#declared-build-resource-model-for-the-collection-handoff), and the already checked [continuing energy rotation](#continuing-energy-rotation). Ultimate capacity is100, initial amount is declared0..100 (baseline0), Burst costs100 at admission, GAStrengthCost0. No passive natural Ultimate or Cycle regeneration is added. Skills/Normals have no source cost row; **infer zero Ultimate cost**, because their positive ChargeAdd and zero-start continuing scenario distinguish them from the explicitly costed Burst, and the descriptions impose no Ultimate admission. Do not infer a free Support from that omission; it requires the shared qualified switch and outgoing Cycle payment.

| Packet sequence | Raw Ultimate ChargeAdd | Raw Cycle HeterochromeAdd | Raw Break UnbalValue |
| --- | --- | --- | --- |
| Normal1 | .578,1.283 | .966,2.142 | .116,.257 |
| Normal2 | .943 | 1.575 | .189 |
| Normal3 | 1.455,1.893 | 2.429,3.161 | .291,.379 |
| Normal4 | 1.023 | 1.708 | .205 |
| Normal5 | 1.417,2.907 | 2.366,4.853 | .283,.581 |
| Skill, each | 2.450 | 4.100 | .500 |
| Burst impact, five claws, slash | 0 each | 0 each | .249, .110 five times, .700 |
| Hold opener | 1.418 three times,2.335 | 2.361 three times,3.888 | .282 three times,.465 |
| Hold spin, each | .667 | 1.111 | .133 |
| Plunge | 3.300 | 5.400 | .700 |
| Riposte | 4.400 | 7.300 | 2.500 |
| Support | 8.400 | 0 | 2.500 |

Rows are source-bound `DT_SkillDamageData`; approach/entry use Normal1 resources and are replacements. Raw values are not rounded tooltip displays. One complete Normal chain gives **11.499/19.200/2.301**; full hold gives **26.599/44.301/5.301**. The per-recipient Ultimate ledger and per-owner Cycle ledger retain fractions; Normal/Skill output cannot be multiplied by dummy count. Burst never funds itself. A5 does not create energy; R6 gives2 Cycle at Display.1 once.

**Stamina inference chosen after the existing source/native and public-description approaches:** each of the30 spin cost requests debits **2*stamina_use_efficiency** from the declared account pool, paired atomically with the corresponding spin opportunity. Use the selected StrengthExecution input -2 and unit-cost multiplier from the account profile; do not multiply by the unrelated current awakening number or exempt A1 because a legacy tag matched. No separate opener stamina cost is declared by its selected event map. The two nearby cost/damage notifies can differ by submilliseconds; pairing them prevents a free hit when the final cost check fails. Require sufficient stamina before each paired hit, pay even if it misses, and enter End on insufficiency. At unit use efficiency a full spin costs60; with remaining5 exactly two further spin packets pay4 and the next fails, leaving1. No arbitrary38*duration debit is added on top. Tooltip38/s and the5/6s display disagreement remain **factual conflicts**, resolved in favor of the reached per-event calculator for this model. They do not prove the source-based60 is the native player's displayed drain.

Normal, ordinary Skill, Burst and Support have no additional modeled stamina payment. Plunge/Riposte themselves add no unbound native cost; the preceding jump/dodge belongs to an explicit scenario event with its own account payment and time, and is not repeated for free. Input admission requires living active Mint, the action's phase/cancel boundary, a living target for targeted attacks, available cooldown, resources and corresponding grounded/airborne/critical-response state. Hold requires its section threshold and no active hold-exit gate. A failed ordinary input is a diagnostic, never an implicit wait or a resource reset. Early cancelling Skill before its cooldown notify does not retroactively grant energy or Display.1 effects.

**Readiness limits:** the account stamina capacity and recovery rate are explicit scenario inputs, as required by the shared model; their actual account/native numeric values are not certified here. The native held clock implements validation, continuous recovery, source-owned suppression, exact paired spending and exhaustion using those declarations. The no-stamina baseline and the resource inequalities below do not depend on a fabricated refill rate. Secondary routes use the selected native clock; native source integration retains the same account. Explicit player-tenacity cues retain their named-timer pause/restore.

## Recipient and dependency specification

**Mixed basis:** current selected Mint damage ranges have no recovered positive recipient cap; public Skill, Burst, plunge, hold and Riposte describe area damage, while native normal ranges are trace volumes. For the stationary all-in-range scenario, select **all eligible living dummies** for every selected damage packet, once each, in ascending target index; no falloff, coefficient splitting or invented cap. This extends the already authorized Skill/Burst AoE convention to the reached Normal/alternate trace volumes as a labeled recipient inference. It does not claim that an omitted native MaxCount field proves an unlimited live-game cap. A4's **query cap10** is only its nearby-count predicate and does not cap outgoing damage. Targeting assistance/lock priority selects the action's origin/primary target, not a second damage multiplier.

At each scheduled hit evaluate the live recipient set, HP, buffs, resistance and A6 threshold. Deduplicate repeated overlap/mesh results by target index. A target killed by an earlier packet receives no later packet. No living eligible target means a miss with no HP loss, damage-triggered energy/A1 or gear proc. A5/R6 Display.1 still fires without damage; stamina is spent for a scheduled spin opportunity even when it misses. Source effect maps' damage plus HitBack entries are one damage event, not two hits. On ten full-health dummies the level8 two-hit Skill has20 damage deliveries totaling **30840** at the1000-ATK/.5-DEF oracle, but gives4.9 raw owner Ultimate once across each packet and does not grant49. A one-dummy A4 test gains.12 CR; two nearby dummies gain none even when one was the locked target.

Required shared rules are now explicit in [Mint cycle/Break dependencies](../../shared/cycles-and-break.md#selected-mint-handoff-dependencies): switch admission, Blossom's five emissions/cap3/ordinary damage row, Hexed/Charge causal behavior, CI/level capture, Break contribution/recovery and equal-time order. Passive1 adds400 range units without a damage replacement. [Resource ownership](../../shared/energy-and-gauges.md#declared-build-resource-model-for-the-collection-handoff) distinguishes per-owner Cycle, team Ultimate and account stamina. [Equipment builds](../../shared/equipment-builds.md#internal-static-build-transport) defines explicit console additions and automatic 2-/4-piece set effects. Weapon and set triggers use their own current dossiers; compiled gear is not applied again as a live stat bonus. These links are implementation decisions, while old source notes below retain their native-fidelity uncertainties.

## Complete implementation handoff

**Accepted mixed specification, not a completed simulator.** The seven preceding specification sections cover the selected full kit, alternate actions, six individually selectable awakenings, both resonances, party passive, target/resource/effect and clock policies. Historical source/description disagreements below are native-fidelity limits; they no longer postpone these explicit implementation choices. Do not request new native body recovery to implement this declared model. Additional proof of accuracy belongs after executable coverage, unless a concrete contradiction requires correcting a rule.

**Build baseline:** use the existing tested `builds[0].build` in [built-game-research.json](../../../../examples/built-game-research.json): Mint80/80 breakthrough6, purchased Normal/Skill/Burst8, **no awakenings**, Mind Royale80/80 T5, Fireflies `Suit2` count4, with console contributions explicitly authored in `additive_attributes` (the script uses `add stats`). Resolve stats through the exact build compiler; the example's historical manually supplied static summary is not a substitute for recompilation. Use the full compiled HP, no incoming damage, seeded crit42, one stationary training dummy, source DEF1080/Anima RES.20/HP24,000,000 and explicit **no Break bar** (the test source row's bar list is empty). This avoids guessing a Break bar from its nonzero UnbalMax; a declared breakable scenario uses the shared full rule instead. Initial Ultimate0/100, Cycle0/100, carries0, no timed buffs or cooldowns, active Mint. Passive2 is active from initial entry. Passive1 is unlocked and enabled; in a solo Anima scenario there is no compatible pair to create a bud, rather than a suppressed required proc.

Keep both gear effects: [Mind Royale](../../weapons/mind-royale/passive.md) grants18 self Ultimate on an admitted Redirect Skill,20s real ICD, with current declared grant/efficiency policy; [Fireflies](../../sets/fireflies/passive.md) gives .10 Anima bonus plus .08 CD per connected Anima **delivery**, cap7, individual10s lifetimes, no refresh at cap, persisting off field. The triggering hit does not use its new stack. These existing checked gear models remain disclosed hypotheses; this handoff does not claim their native unresolved fields were recovered. A nearby-team multi-dummy variant applies Fireflies's per-delivery rule, which is intentionally different from the resource per-packet grant rule.

**One continuing120s sequence, with cycle labels for explanation only:**

| Time(s) | Authored action/state |
| --- | --- |
| 0..36.78570784 | Eight complete five-input Normal chains, issuing each next input at the timing profile's ability boundary; one chain is4.59821348s including connected explicit hitstop. |
| 36.78570784..40 | Explicit wait. |
| 40 | Skill; Mind Royale grants18 once. Skill recovery finishes40.9673227, cooldown finishes46.18279111. |
| after recovery..47 | Explicit wait; buff/cooldown/resource events continue. |
| 47 | Second Skill;20s weapon ICD prevents another grant. |
| after recovery..49 | Explicit wait. |
| 49..53.59821348 | Ninth full Normal chain. |
| 54 | Burst spends100; seven hits finish57.499036; ability recovery57.9079924. |
| after recovery..60 | Explicit wait. Do not reset anything at60. |
| 60..96.78570784 | Eight continuing full Normal chains. |
| 100,107 | Third/fourth Skills, with explicit waits to those times; Mind Royale grants18 at100 only. |
| 109..113.59821348 | Eighteenth full Normal chain. |
| 114 | Second Burst spends100; last hit117.499036; recovery117.9079924. |
| after recovery..120 | Explicit wait to the true horizon; retain all final state. |

**Independent resource and legal-time check:** each cycle supplies9*11.499+2*4.9=**113.291 raw ordinary energy**, plus18 weapon energy, no Burst energy. At unit Charge Efficiency the first Burst leaves integer0 and carry291/1000, discarding31 integer overflow over the cycle; second leaves0 and carry291/500, cumulative overflow62. Weapon ICD deadlines60/120 are not reset by the cycle labels. Cycle grants are9*19.2+2*8.2=189.2 each; with no qualified switch its integer pool remains100, carry progresses.2→.4 and integer overflow89→278 cumulatively. It does not automatically trigger Support. All Burst starts are60s apart (>15); both second Skills are7s after the preceding cast (>notify .18279111+6). Normal input-start gaps inside a chain are below2s. The8-chain runs fit their40s slot, the ninth chain ends before Burst54, and Burst recovery finishes before the next labeled cycle. The sequence never reads/spends stamina, so its legality does not depend on a guessed stamina-recovery rate.

**Independent packet oracle:** 83 direct packets/cycle,166 over120s. In the separate noncritical1000-ATK, zero-bonus/RES, half-DEF arithmetic fixture, each cycle is9*1773+2*1542+6857=**25898**, total**51796**, DPS**431.633333...** using120s, not first-hit-to-last-hit time. This fixture validates coefficients/count/denominator, not the equipped seeded total; the actual build must evaluate the compiled stats, evolving Fireflies stacks and RNG per hit and expose them in receipts. Never copy51796 into the equipped result. Independent Fraction checks were run in ; its receipt identifies itself as model arithmetic only, with game observations and engine execution both false.

**Alternate-branch handoff checks:** at least120 stamina at unit use efficiency admits two full60-cost holds even before any nonnegative recovery, with normal recovery still processed under the declared account profile; start each only after the prior End/recovery. Remaining5 permits two spins, then End with1. A successful A1 Riposte while Skill is cooling resets once and blocks another reset for3s; a miss leaves cooldown untouched. A qualified Cosmos→Mint switch spends outgoing Cycle100, emits one Mint Support and a five-emission bud; Mint A3 (when selected) is refreshed by that reaction even if Support misses. Test falling input with declared landing and both fall-bonus endpoints, not a doubled base plunge in every scenario. Switching after Skill/Burst recovery leaves no future montage damage; existing reaction buds continue. These are the required alternate semantics, not additional arbitrary baseline resets.

**Implementation acceptance:** add content-owned action profiles/state transitions and the named shared policies without moving character logic into core. CLI/Studio must run the same authored two-cycle rotation against the Rust/WASM engine and preserve model assumptions, pending effects, exact resources, event order and full120s denominator through replay/aggregation and an actual Worker. Reached missing account/landing/proximity inputs produce diagnostics, never hidden defaults. The current executor remains a partial slice documented below; this research batch implements no simulator code and establishes no new runtime parity claim.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Accepted bounded product models (Authorized October 17, 2026)

Following human authorization on October 17, 2026 to lift the research pause and accept bounded product hypotheses:

- **Action timing profiles (`mint.skill_animation_profile_v1`, `mint.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Mint_Skill_01`), TriggerCooldown is at 0.18s, hit 1 is at 0.20s, hit 2 is at 0.52s (delta 0.316s), Jump cancel is at 0.68s, AnySkill cancel is at 0.87s, and Move cancel is at 1.20s. For Burst (`Mint_UltraSkill`), hits 1..7 are spaced across 1.62s–3.50s, with AnySkill cancel at 3.91s.
- **A3 reaction trigger (`mint.a3_reaction_trigger_v1`):** A3 Attendance Bonus grants Mint +15% ATK (`Mint_Adcance_3_1 = 0.15`) for 15s upon triggering Blossom or Hexed against an enemy by any party member, resolving the asset `QTE_BEGIN` conflict in favor of published in-game behavior. Applied via `mint.a3_attendance_bonus` buff.
- **Multi-target AoE (`mint.aoe_all_in_range_dummy_v1`):** Mint Super Claws and Thunderous Whirlwind Slash deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Each packet index lands on every target in ascending `target_index` order at that packet's impact time; Skill hit 2 keeps its .365246s stagger on every target. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Ordinary Normal chain (current model)
Five admitted Normal inputs deliver eight packets in stages [Melee1,Melee2], [Melee3], [Melee4,Melee5], [Melee6], [Melee7,Melee8]. Descriptions match native decompositions at all nine released talent levels. Native effective talent projection includes resonance_3. Input receipts retain the admission binding; hit receipts identify actual stage packets. Every child samples live modifiers independently, including A6 HP threshold changes between hits.

Execution policy: The chain loops after stage five. A different admitted ability or field exit resets it; same-actor switches preserve it. An input-start gap greater than 2 seconds resets the chain; the exact boundary continues. This tested continuity convention follows existing ordinary-chain models and is not measured animation timing. Each stage delivers ordered packets at the input's research primary impact time; exact animation/travel intervals remain unresolved. Already scheduled packets retain their actor after field exit. Native animation/cancel evidence is not established by this model. Evidence: [`mint-normal-chain.json`](../../../../reviews/q10-game-action-binding/mint-normal-chain.json).

## Action packets and owner effects

- GA_Mint019_Melee public primary packet is GE_Player_Mint_Melee1_Damage_New. Native coefficients at Normal levels 1/8/9 are 0.104/0.178/0.192; category A/Anima (`DAMAGE_TYPE_NATURE`) metadata stays source-bound.
- GA_Mint019_Skill admits GE_Player_Mint_Skill1_Damage_New and delivers it twice. Native coefficients at Skill levels 1/8/9 are 0.900/1.542/1.666 per hit; category E/Anima metadata stays source-bound.
- GA_Mint019_UltraSkill admits GE_Player_Mint_UtraSkill1_Damage_New, then delivers UtraSkill2..6 once each and UtraSkill7 once. At levels1/8/9, impact coefficients are1.326/2.273/2.454, each claw .588/1.008/1.088, slash3.735/6.401/6.913; category Q/Anima metadata stays source-bound. The former Incantation wording in this card was incorrect; the selected frozen damage rows and `DT_Character/1019/ElementData/CharacterElementType` both specify NATURE. Historical receipts are preserved.
- mint.skill_burst_packets_v1 discloses ordered delivery on target zero: Skill hit 1 at the selected impact time and connected hit 2 .365246s later, so the first hit's Fireflies stack is live for the second; a miss leaves the nominal `.5159236` coordinate and delivers no damage. Burst packets stay ordered at the selected impact time. Native intervals beyond those offsets, AoE, travel and interruptions remain unresolved. See mint-skill-burst-packets.json.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill, selecting native coordinate level + 1 without manual overrides.
- Existing A2 Re-examination (+40% DMG bonus to Skill), A4 Lunch Break (+12% Crit Chance on single target), A5 First Instinct (+25% Crit DMG for 6s on Skill), A6 Elite Member (+15% DMG on target below 40% HP), and A6 Resonance 6 (+2 cycle energy on Skill) remain preserved as partial kit models.
- Canonical arbitrary awakening selections: Ordinary Effect2/4/5/6 gates use selected identity. Talent resonance_3 and cycle energy resonance_6 use count; other packet/cycle applicability limits remain pending. Existing unsupported owner mechanics remain pending.

## Continuing energy rotation

The zero-start A0 verification scenario joins Mint's modeled Normal chain, two Skill casts and seven-packet Burst: eight five-input Normal chains, two Skills, one additional Normal chain before Burst, then one continuation chain after the 100-energy spend. It uses the existing `mint.ordinary_hit_energy_v1` policy; each successful modeled Normal or Skill packet applies its frozen ChargeAdd after positive target HP loss. At base Charge Efficiency, each five-input Normal chain contributes 11.499 raw energy and each two-packet Skill contributes 4.900. Eight Normal chains plus the first Skill leave 96.892; the second Skill makes Burst affordable at 101.792 before cap settlement. The additional Normal chain brings total pre-Burst ChargeAdd to 113.291; the 100-cap pool records 13 overflow, pays 100, and the following chain settles to 11 energy with 79/100 fractional carry. The two other living configured recipients each finish at 74 energy with 437/500 carry under the 3/5 relation.

The authored timing profile spaces ordinary inputs by 800ms, places the first Skill at 32.0s and the second at 38.5s (6.5s apart, satisfying the modeled 6.0s cooldown), the pre-Burst chain at 39.7s, Burst at 43.7s, and the continuation chain at 48.7s after the modeled 2.0s Burst recovery and a 3.0s authored wait. These are compiler/profile timestamps, not measured native animation timings.

At level 8, 1000 ATK, 0.5 defense multiplier, and zero damage bonuses, the independent damage oracle is ten Normal chains × 1773, two Skills × 1542, and one Burst × 6857 = 27671 across 91 hit receipts. The Burst's seven packets have zero ChargeAdd and do not grant energy. See the [Mint continuing-energy receipt](../../../../reviews/q10-energy-ledger/mint-continuing-energy.json) and authored [`mint-continuing-energy.nte`](../../../../examples/mint-continuing-energy.nte).

The test asserts that the compiled rotation funds and spends Burst energy through the existing adapter. It does not establish native ordinary-hit eligibility, cadence, recipient rules, or live DSL readiness; `mint.ordinary_hit_energy_v1` remains a bounded model hypothesis.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/mint-action-binding-model.json`](../../../../reviews/q10-game-action-binding/mint-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/mint_actions.rs), [`mint-normal-chain.json`](../../../../reviews/q10-game-action-binding/mint-normal-chain.json), [`mint-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/mint-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_mint.json`, `data/extracted/neverness_mint.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/mint019.rs) (original `DT_SkillDamageData` export is cache-only), `1019AwakenEffect1` (cache-only awakening source), [`mint/parameters.rs`](../../../../crates/content/src/characters/mint/parameters.rs) (consumed typed parameters; original `DT_MintEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

The consumed constant EffectFigure values for A2/A4/A5/A6 live in the owner `parameters.rs`; display rows and unbound source parameters are not runtime data. This migration preserves the existing strict-below-40% model and the modeled resonance energy channel. Rust checks independently cover additive A2 damage, A4 crit, both sides of the A6 HP boundary, and A5's 6s expiry across two continuing casts with deterministic replay. The separate A3 helper check does not establish an automatic reaction producer.

- AirAttack, Evade1/4/5, hold and Critical Riposte remain unimplemented. The ordinary Melee1–Melee8 chain is already modeled above; its native timing/reset policy remains unknown.
- A1 Task Force Operation and A3 Attendance Bonus are not integrated.

## Full-kit source investigation (October 17, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Full asset paths and warning records are in the existing initial and supplementary acquisitions. The data-only inspection is saved under ; these are optional research inputs, not a new runtime package. The installed source reports PC_140 / logic 566459. Current descriptions were also checked against [Reference guide](https://neverness.app) on October 17 (page's profile date: June 23, 2026) and [Neverness](https://neverness.app/characters/mint). Source configuration and description agreement do not establish native callback clocks or defaults.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows supply the following **ATK fractions**, with separate raw Ultimate ChargeAdd, HeterochromeAdd and UnbalValue inputs. All these rows are NATURE/Anima; resource/gauge fields are not proven grants per hit, per target or per input. In particular, X/Z/H are distinct native categories and must not be silently recategorized as E.

| Action / damage row (`GE_Player_Mint_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue per configured row |
| --- | --- | --- | --- |
| Plunge `AirAttack1_Damage` | X | 0.650 / 1.114 / 1.203 | 3.3 / 5.4 / 0.7 |
| Hold opener, each of three `Evade1_Damage` packets | A | 0.283 / 0.485 / 0.524 | 1.418 / 2.361 / 0.282 |
| Hold opener final `Evade4_Damage` | A | 0.467 / 0.800 / 0.864 | 2.335 / 3.888 / 0.465 |
| Hold spin, each `Evade5_Damage` | A | 0.133 / 0.228 / 0.246 | 0.667 / 1.111 / 0.133 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.740 / 2.982 / 3.221 | 4.4 / 7.3 / 2.5 |
| Support `QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 8.4 / 0 / 2.5 |

These coefficients match `neverness_mint.json#/skills/0/stats` for plunge/riposte/jump/hold, and the Support description. The Riposte row's GAName is **GA_Mint_ExtremEvadeAtk**, whereas the configured executable class is **GA_Mint019_PerfectEvade**; keep initiating action and packet metadata distinct. The Support event map requests only QTE1 damage; supplementary QTE2/3/4, old Melee/Skill packets and `Skill4_Damage_New` existing in the archive do not establish additional hits. The currently selected hold map uses **Evade4**, not the similarly named **Skill4** row.

Raw resource sums distinguish the channels: the eight ordinary packets sum to ChargeAdd **11.499** and HeterochromeAdd **19.200**, while the two Skill packets sum to **4.900 / 8.200**. `DT_GameplayAbilityTipsData/GA_Mint019_Skill/GALevelUpArray` binds Cycle Rate to `DT_MintEffectFigure/Skill_Heterochrome_Show=8`; the saved Reference guide multiplier says 18. Preserve that disagreement and the separate display value instead of converting 18 into native Ultimate energy or replacing the raw 8.2 by a guessed integer-rounding policy. Burst's seven rows have zero in both resource channels. The existing continuing-energy model above covers only its explicitly stated Ultimate Charge policy, not a verified Cycle producer.

`GA_Mint019_Melee#/2/Properties/MontageToPlays` selects the ordinary five montages, AirAttack/AirAttackEnd, MeleePre, AppearMelee and hold Begin/End. `Mint_PreMelee` and `Mint_AppearMelee` both emit Melee.1/2, the same two damage bindings as ordinary stage one; they are approach/entry alternatives, not extra unconditional stages. Their native selector precedence and chain continuity remain unknown. Falling selection (`#/5`) admits AirAttack with LandOffDistanceMin=0; landing checks request AirAttackEnd, whose single AirAttack.1 event requests the plunge packet. The description promises height-dependent damage up to +100%, but no height-to-bonus function, threshold or capture point is present in the reached Mint GE/parent chain (which ends at native DamageExecutionCalculation). Do not invent a flat double-damage plunge.

### Hold entry, damage, stamina and exit

The HoldTriggerCheck objects in `GA_Mint019_Melee` request section **Begin**. Following JumpSectionMap and object ordinals gives configured HoldTimeTrigger values: Melee1 0.40, Melee2 0.20, Melee3 0.50, Melee4 0.25, Melee5 0.70, MeleePre 0.45, AirAttackEnd 0.53 and AppearMelee 0.45. Each has an OptionalRequirements entry with TargetRequirement.HasAllGEArray=`Buff_MintSkill_WindEnd`; AirAttackEnd additionally has an empty alternative. The native predicate/optional-list semantics and hold clock are unresolved; those numbers are not measured admission delays. Melee1/MeleePre also have a custom transition requiring `Buff_ExtremeEvade_Detection` and requesting `GA_Mint019_PerfectEvade:Atk`.

The selected `Mint_Skill_001_Begin#/32/Properties` montage has Begin -> Loop -> End and total asset length 6.85. Its segments contain a 1.4166666 opener, ten 0.4 copies of the spin animation (four individually placed plus a six-repeat segment), and a 1.433333 ending. Begin emits Skill.1/3/2/4, bound by the **Melee** ability to three Evade1 packets and one Evade4 packet. Between coordinates 1.4329805 and 5.2997556 are **30 Skill.5 event entries**, each mapped to Evade5. Thus the configured opener sums to 1.316 ATK at level 1 / 2.255 at level 8, and the complete 30-entry spin adds 3.990 / 6.840 respectively. At 1000 ATK, no bonuses/crit/RES and a 0.5 DEF multiplier, the conditional level-eight opener-plus-spin arithmetic is **4547.5 before packet rounding**. This is not proof that every event is delivered once in live combat or that an interrupted cast delivers the full train.

There are also 30 Common.StrengthCost entries. They request `Buff_MintSkill_EnergyCost`, whose native StrengthExecution captures source SkillCoefficient and applies an additive scalable input **-1 × Mint_Adcance_6_AddEnergy = -2**. Its TargetTags ignore `Ability.Player.Mint.Upgrade.Level1`, and CheckStrengthCostGameplayEffectComponent requests End on failed check. That does not prove two stamina units settled per entry: native execution/efficiency and tag filtering remain open. The ignored tag appears on the supplementary legacy `Buff_Mint019_Level1`, while the selected awakening row grants `Buff_Mint019_Level1_New`; do not infer a current A1 free-stamina effect from the legacy asset. Tooltip bindings separately advertise Stamina_Persecond_Show=38 and Attack_Duration_Show=6; the inspected [Reference guide guide](https://neverness.app) reports natural expiry after five seconds and at most 30 spin hits. The sources use different possible start/end references; neither 38 × an assumed duration nor 30 × -2 establishes a legal stamina budget without the consumer.

Skill.8 requests `Buff_Mint019_SkillTriggerEnd` at montage start (stored coordinate approximately zero); its configured duration is 5.41 and FINISH requests the 0.1-duration `Buff_Mint019_SkillEnd`. `GA_Mint019_Melee`'s Begin section has a local check for that end carrier and requests End. Skill.6 adds `Buff_MintSkill_Energy` and Wind; the energy carrier adds EnergyStop at BEGIN and removes it at SKILL_FINISH/FINISH. EnergyStop explicitly overrides **StrengthGetEfficiency=0** and adds MoveSpeedGroundMaxMult=0.3. It does not modify Ultimate Charge efficiency. Melee ability end removes SkillTriggerEnd and Energy; Wind has removal requests at Melee montage finish and CHANGE_ROLE_OUT_BEGIN. These carriers' 100-duration fallbacks are not permission to run a 100-second hold. Skill.7/End also requests WindEnd. Native removal reasons, FINISH propagation, release/Skill cancellation, pending-hit survival and switch cleanup remain open. The guide's recommendation to cancel/parry with Redirect Skill is qualitative evidence only.

### Skill and Burst timing investigation

The priority A0 Skill question was followed through the complete saved selected action/montage/parent/notify chain. `GA_Mint019_Skill` selects **Mint_Skill_01**, not the similarly named **Mint_Skill_001_Begin** hold montage. Its event map assigns Melee.1/2 to the same Skill1 damage GE with distinct range classes (Skill2_New / Skill3_New), so this is two damage requests rather than one event duplicated by assumption.

| `Mint_Skill_01#/2/Properties/Notifies` entry | Asset LinkValue | Resolved request |
| --- | --- | --- |
| 14 | 0.18279111 | AnimNotify_TriggerCooldown |
| 17 | 0.19956753 | Display.1: owner A5/R6 requests |
| 18 | 0.20017847 | Melee.1: first Skill damage request |
| 29 | 0.5159236 | Melee.2: second Skill damage request |
| 30 | 0.6846825 | CanBreakSkill: Jump, cached input 0.5 |
| 32 | 0.8683227 | CanBreakSkill: AnySkill, cached input 0.5 |
| 33 | 0.9920734 | TriggerEndAbilityEffect |
| 34 | 1.2020172 | CanBreakSkill: Move, cached input 0.5 |

The two damage-event coordinates differ by **0.31574513 asset units**; all listed cancel states end at asset coordinate 4.5. The damage GE explicitly configures AttackDilationTime=0.05 and AttackDilationRate=0.01. The native [shared montage caller](../../shared/action-timing.md#reached-shared-montage-caller), hitstop, notify dispatch and cooldown consumer prevent treating those coordinates as real hit times, the cooldown notify as proven countdown origin, AnySkill as earliest switch, or montage length as mandatory recovery.

`Mint_UltraSkill#/2/Properties/Notifies` maps UltraSkill.1..7 at coordinates **1.6159108, 1.754891, 1.9379535, 2.143918, 2.2606597, 2.4272692, 3.499036** to the seven established packet rows. Its ability declares StartLimitChangePlayer=Default, an AbilityGamePaused component, a skill sequence and immunity/display carriers; the montage contains CantChangePlayer at zero and AnySkill/Move at 3.9079924. No real pause duration, earliest switch or survival of later packets after interruption follows from those declarations. Support's QTE.1 request is at 0.7394236 in `Mint_QTE#/6`; Riposte's Melee.1 is at 0.22721595 in `Mint_PerfectEvadeAtk#/1`; plunge's AirAttack.1 is at 0.06451421 in `Mint_AirAttackEnd#/1`. These are source coordinates, not simulator timing profiles.

Warning scope: initial Mint_Skill_01 has nine recorded warnings (particle/water notify objects); the hold montage has 29 (particle/water/HideInGame); Burst has 26 (also RotateToTarget); QTE seven (particles), Riposte four (particles), AirAttack one (root motion), AirAttackEnd four (particle/water). The cited action CDO/event-map and send-event/cancel objects are present, but this is not a complete warning-free animation or movement decode.

**Method stop:** (1) saved dependency/Blueprint tracing now reaches the named native rate/task/notify consumers; (2) the already saved static loader investigation reaches the common protected procedure but does not restore native gameplay bodies, and was not repeated; (3) current English/Chinese guide searches and the saved public-trace/[calculator](../../../DRIVE_CALCULATOR_RESEARCH.md) findings supply no applicable Mint action-input/hit/switch/cooldown timeline. Current public pages corroborate descriptions and qualitative hold cancellation, not frame timing. The three approaches have no supported real-time answer. Resume this branch on a recovered rate/notify/cancel/cooldown consumer or an already available attributable Mint recording/trace with command and cooldown observations; do not rerun unchanged exports or ask for gameplay measurements.

### Awakening conditions, recipients and conflicts

- **A1:** `GA_Mint019_PerfectEvade#/1/Properties/EffectContainerMap` binds `Event.Montage.Player.Melee.1` to the Riposte damage, hit-back and `Buff_Mint019_Level1_1_Target_New`, using `DamageRange_Mint_PerfectEvade`. That target carrier lasts 0.3; its BEGIN request has `TargetTrigger=true` and requests `Buff_Mint019_Level1_1_New`. `Con_Mint_Lv1_New:CanApplyGameplayEffectContent` checks a valid target ASC, casts SourceActor to the player and requires both `GE_CoolDownBase` and `Buff_Mint019_Level1_New` on that source. The requested reset carrier removes owning-tag matches for `CoolDown.Player.Skill.Mint`, has a 3-second duration bound to `Mint_Adcance_1_1_CD`, target aggregation/limit 1, NeverRefresh/NeverReset, and rejects a target already carrying that same reset class through `Con_Mint_Lv1_1`. This supports the described hit-triggered cooldown removal with a separate three-second re-entry guard. Exact callback recipient mapping, simultaneous multi-target ordering, application failure and cooldown start/progress still need the native consumers; a dodge alone is not the configured trigger. Do not confuse the 0.3 target-carrier lifetime with the three-second guard.
- **A2:** both Skill hits select `GE_Player_Mint_Skill1_Damage_New#/2/Properties/CustomApplicationModifierInfos`, which binds additive **DamageUpGeneralBase=0.40** through `Con_Mint_Lv2`'s source Level2-marker requirement. This is an ordinary damage-bonus-zone addition for those packets, not a relative coefficient increase or an independent 1.4 multiplier. For general 20% plus A2 40%, level-eight two-hit raw damage at 1000 ATK and half DEF is 2467.2, not 2590.56 from multiplying 1.2 × 1.4.
- **A3:** `1019AwakenEffect1#/rows/1019/AwakenEffectStructList` selects `Buff_Mint019_Level3`; both its English/Chinese description and the current public description say any character's Blossom/Hexed grants Mint 15% ATK for 15 seconds. However, the selected asset's `#/1/Properties/BuffEventEffectData/0` declares **QTE_BEGIN**, `BySelf=true`, `TargetTrigger=false`, and requests `Buff_Mint019_Level3_1`. That child binds additive **AtkUp** to `Mint_Adcance_3_1=0.15` and duration to `Mint_Adcance_3_2=15`. The values/stat zone are supported, but the reaction-versus-QTE trigger conflict is unresolved. Do not silently implement both triggers, or infer stacking/refresh from the parent's stack limit (the child omits those fields).
- **A4:** `Buff_Mint019_Level4#/1/Properties/CustomApplicationModifierInfos` supplies additive `CritBase=0.12`, gated by `Con_Mint_Lv4`. Its graph casts **SourceActor**, calls `GetTargetsByTargetType` with `LockRange_MintPas2`, and returns **array length <= 1**, without branching on that call's boolean success. `LockRange_MintPas2#/1/Properties` configures SphereRadius 500, MaxCount 10, ObjectTypes 3/12 and `bIgnoreCantTrace=false`. This is a local source-centered query, not proof of a global living-enemy count. An empty returned array takes the same comparison branch as one entry. Query units/filtering/deduplication and modifier evaluation time remain native dependencies; the current single-target model is narrower than this source predicate.
- **A5:** `GA_Mint019_Skill#/2/Properties/EffectContainerMap` requests `Buff_Mint019_Level5_1` on the owner at **Display.1**. `Con_Mint_Lv5` requires the source's `Buff_Mint019_Level5` after its valid-ASC/cast checks. The child binds additive `CritDamageBase=0.25`, duration 6 and AggregateBySource; an omitted stack limit/refresh policy is not evidence of infinite stacking. This locates a montage-event trigger, rather than proving an admission-time buff or requiring a damage hit.
- **A6 damage:** `Buff_Mint019_Level6#/1/Properties/CustomApplicationModifierInfos` supplies additive `DamageUpGeneralBase=0.15`. `Con_Mint_Lv6` checks target HPMax, reads target current/max HP, and requires the damage effect's asset tags to include both `Ability.Player.Mint` and `State.Damage`. Its comparison is **HPCurrent / HPMax <= 0.4** (`LessEqual_DoubleDouble`, statement 787), while both saved descriptions say **below** 40%. At exactly 400/1000 HP the graph accepts and a strict-below model rejects. The current executable model retains its documented strict threshold; no gameplay policy is changed by this research. Failed ASC/cast and nonpositive-max-HP paths must not be promoted to a known boolean default.
- **R6 resource:** the same Skill **Display.1** event requests `Buff_Mint019_LevelExtra_AddEnergy`; `Con_Mint_Extra` requires the source's `Buff_Mint019_LevelExtra_New`. Its `HeterochromeExecution` custom modifier calls `Cau_Mint019Advance6_AddCharge:CustomCalculateBaseMagnitude`. On a valid OwnerActor player cast and successful curve evaluation, it calls that owner's ASC **AddHeterochromeValue(2)** and returns zero. The initialized local 50 is unused. This is the separate Cycle/Heterochrome channel, not ordinary Ultimate ChargeAdd; the native application frequency, settlement and team propagation remain unproved.

All the specific action/effect/condition/calculator exports above have no recorded parser warnings in the initial bundle. A3's disagreement is present in the selected data, not explained away by a known skipped field. Independent arithmetic checks must preserve the shared [stat zones](../../shared/damage-and-stat-zones.md): with base ATK 1000 and existing AtkUp 0.50, an admitted A3 gives 1650 rather than 1725; general 20% plus an eligible A6 15% gives a 1.35 bonus factor rather than 1.38; existing additional CRIT DMG 0.50 plus A5 0.25 gives critical factor 1.75.

### Defensive passive

`Buff_Mint019_Passive2_New#/1/Properties/BuffEventEffectData` requests `Buff_Mint019_Passive2_1_New` at BEGIN and CHANGE_ROLE_INT_BEGIN, and removes it at CHANGE_ROLE_OUT. Both carriers use target aggregation/limit 1. The child supplies additive `DefUp=0.20` via `Mint_Pas2_DefAdd`, `bActiveSuperArmor=true` and `SuperArmorCoefficient=0.7`; the public description says 20% DEF and 30% interruption resistance while on field. The DEF zone and configured entry/exit lifecycle are supported; 0.7's actual native interruption calculation is not recovered. The initial BEGIN branch alone does not prove how an initially off-field party member is initialized. This affects survival and legal continuation under incoming damage; it is not an outgoing damage multiplier.

`DT_CharacterAbilityConfig/1019/PassiveAbilityList` binds Passive_1 to breakthrough 2 and Passive_2 to breakthrough 4, with one purchase/unlock row each; they are distinct from awakening A1/A2. `DT_CharacterAbilityEffectConfig/GA_Mint_Passive_1` selects `Buff_Mint019_Passive1`; Passive_2 selects the **New** carrier above. Supplementary same-numbered legacy passives are not additional simultaneous bonuses. Console stats are supplied explicitly through `add stats`; [equipment builds](../../shared/equipment-builds.md#internal-static-build-transport) applies only the declared set effects automatically.

### Blossom enhancement

`DT_ReactionExtendDataTable/1019/ReactionDataArray/0` selects `ReactionAbilityID=GA_Mint_Passive_1`, requires `Buff_Mint019_Passive1`, sets **AdditionalDamageRange=400**, and supplies **NewDamageGE=null**. The selected passive carrier is Infinite, AggregateByTarget/limit 1 and has `GlobalGameplayEffectComponent.CheckSourceDeath=false`. Together with the description, this supports a configured Vita Pistil area extension, without a new Mint damage coefficient or an additional projectile-count rule. The current reaction producer, conditional scope, native range-combination units, recipient cap and death/off-field behavior remain with the [shared reaction findings](../../shared/cycle-source-findings.md). CheckSourceDeath=false alone does not prove all native team/death propagation. This Mint row matches the frozen row in the later reaction-extension recovery; the original table's four warnings concern other nested owner types and are not a reason to discard this recovered row.

### Hold selectors and critical-response routes, October 18

The initial bundle's `GA_Mint019_Melee#/2/Properties/JumpSectionMap` resolves section-specific hold selectors, rather than one universal hold delay:

| Starting section | Hold subobject ordinal | `HoldTimeTrigger` | Requested section |
| --- | --- | --- | --- |
| `AppearMelee` | 9 | 0.45 | `Begin` |
| `Melee5` | 10 | 0.7 | `Begin` |
| `MeleePre` | 11 | 0.45 | `Begin` |
| `Melee2` | 12 | 0.2 | `Begin` |
| `Melee3` | 13 | 0.5 | `Begin` |
| `Melee4` | 14 | 0.25 | `Begin` |
| `AirAttackEnd` | 15 | 0.53 | `Begin` |
| `Melee1` | 16 | 0.4 | `Begin` |

These are configured hold-check inputs, not measured deadlines. Each selector includes a target-requirement entry for `Buff_MintSkill_WindEnd`; `/15` additionally contains an empty target-requirement entry. Native requirement combination and clock selection remain open under the [montage boundary](../../shared/action-timing.md#reached-shared-montage-caller).

The same map resolves `/7` and `/8` custom checks for `Buff_ExtremeEvade_Detection`, requesting `GA_Mint019_PerfectEvade`, section `Atk`. Separate `JumpSection_PlayerTriggerParrySucceed` subobjects `/20..27` request that ability's `PerfectEvadeBack` section with `fDelayTimeTrigger=0.02` and their own montage-position intervals. Critical Dodge detection and successful parry are distinct configured entry paths, not an unconditional dodge packet. `/18` and `/19` are landing checks requesting `AirAttackEnd`, with an explicit 0.3 delay only in `/18`. Missing fields do not establish effective native defaults. These objects decode without warnings in `bundle-v1`.

The current [Reference guide guide](https://neverness.app), checked October 18, still describes Skill cancellation of the spin and a five-second natural expiry. It supplies no stamina ledger or input trace resolving these native checks; the earlier duration/source conflict remains open.

### Damage-trigger table applicability checked, October 18

The [shared table/caller join](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) locates rows named **PlayerMintSkill1, PlayerMintSkillTest2, PlayerMintUltraSkill4** and **ActorMintWindSkill**, but finds no literal DamageTriggerConfigID caller for them in the selected saved roster ability/projectile exports. In particular, a test row's MaxTriggerCount=8 does not establish Mint's current Skill hit count. Keep the actually reached montage, hold, effect-container and projectile routes above; an unbound row is a lead with a missing caller, not a new admitted packet train.

### Wind-actor periodic request and trigger parent recovered, October 18

Warning-free expansion **Buff_MintActor_PasEffect#/1** is HasDuration **1.5**, Period **0.15**, **bExecutePeriodicEffectOnApplication=false**, source-cap1. Its **BUFF_EVENT_PERIODIC** requests **GA_MintActorWind_Skill**. That ability (`#/1..2`) calls the [recovered GA_TriggerGEBase dispatcher](../../shared/action-timing.md#reached-trigger-ability-dispatcher-recovered-october-18), and CDO **#/3** binds **Event.NoMontage.Skill.Common.SpawnActor** to **GE_Player_Mint_Melee5_Damage_New** plus Melee1_HitBack_New through **DamageRange_Mint_WindSkill**. It separately configures **SkillReleaseCondition.CoolDownTime=0.35** and **AbilityChargeCost**; the cost subobject omits scalar fields and does not establish free activation or a numeric cost.

This locates carrier-period → requested ability → commit/CanApply gate → packet request. The selected exports supply **no external producer for Buff_MintActor_PasEffect**. Consequently the 1.5/0.15 settings do not prove ten currently applicable Skill hits, especially with a separate 0.35 release condition and native cost/dispatch admission. Do not overwrite the actual player Skill train or the five-second public-description conflict with this unbound actor-carrier route. The similarly named ActorMintWindSkill actor-table row still lacks its own literal config caller. Sources: `mint-wind-grant-config.txt`, `mint-wind-grant-callers.txt`, `remaining-action-target-config.txt` and `trigger-subclass-headers.txt`.

**Actor_MintUltraSkill_TimeDilation** was separately inspected: BeginPlay is empty and BeginOverlap only calls Actor's parent; its inherited CDO has a read warning. It supplies no Blueprint duration/rate scheduler to close Mint's Burst real-time policy. See `overlap-and-projectile-end-functions.txt`; its name does not make every native process use the montage clock.

### Wind cue overlap graph checked, October 18

The separate GameplayCue pass now reads **GC_Mint_Wind** outside the ability folder. **OnWhileActiveEvent#/5 → ExecuteUbergraph#/4** casts MyTarget to player, caches it and attaches its Sphere to that player's Mesh at the **FireSocket** variable. Its bound begin/end-overlap callbacks **#/2..3** accept only a valid **ActorReaction_1**; each calls GetTargetsByTargetType with that cached player and **Target Type Class**, then writes **ReactionActorNum=HitResults.Length**. Begin-overlap additionally requests K2_ActivateGameEffectsWithClassByActor on that player with **GEClass=NoObject**, level1/stack0/strength1. It does **not** name Buff_MintActor_PasEffect, and the native null-class convention is not a valid substitute for a found producer. The graph neither clamps the counted HitResults nor turns them into a list of damage recipients. Its complete functions are readable, but the CDO tail has a scoped native property-read warning; omitted FireSocket/Target Type defaults are not recovered settings. Buff_MintSkill_Wind#/1 names GameplayCue.Display.Mint.Wind, while native cue-class registration and actual overlap delivery remain separate. The other selected Mint Skill/SkillEnd/UltraSkill/WindEnd cue callbacks add visual/component requests without a located producer for the Wind ability carrier. Sources: `cue-mint-nanally-jiuyuan-functions.txt` and `cue-first-owner-tag-callers.txt`.

The subsequent HaloBuffActor recovery rereads those five Mint cues without their initial tail warnings and confirms each matching configured tag. Wind's CDO **#/6** now supplies its native GameCueParticleEffects declaration, whose particle socket **rootSocket** is separate from the Blueprint **FireSocket** variable used for Sphere attachment. FireSocket and Target Type Class remain omitted; do not copy a particle socket into that overlap query or replace the function's null GE class. The same raw asset/functions and the missing Wind-carrier producer are unchanged. Current manifest: `roster-cue-halo-extraction.json`; exact configured joins: `cue-carrier-joins.json`.

### Shared player-time cue pauses the named Skill-end carrier, October 18

The warning-free shared supplement joins **Buff_Tenacity_TimeDilation_Player#/2 → GameplayCue.Display.Tenacity.TimeDilationPlayer → GC_CommonDisplay_TenacityTime_Player**. OnWhile **#/3 → graph#/1:183** casts MyTarget to ability character, requests **SetCustomTimeDilation(0.01,5,false,true)**, then **K2_SetGameplayEffectTimerPauseOrNot(Buff_Mint019_SkillTriggerEnd,true,2)** on that same actor. OnRemove **#/2 → 10** requests **SetCustomTimeDilation(1,5,false,true)** and unpauses that exact carrier (**false,2**). There is no explicit Mint-ID, role or own-carrier-existence test in the cue body. The shared carrier has HasDuration input 0.55 and the WeakControlState.TimeDilation asset tag.

This is a concrete pause/unpause request for Mint's Skill-end carrier, not a recovered universal GE clock, all-resource pause, attack recovery or Hotori Time Stop policy. The [native parameter metadata](../../shared/action-timing.md#time-function-parameter-metadata) now binds dilation arguments to **DilationType=5 (TimeDilationArmorBroken), bReplicate=false, bUsedParticle=true** and pause arguments to **bPause=true/false, PausedType=2**. The pause enum identity, receiver ownership, duration clock, overlapping calls and successful timer restoration remain open. Preserve the explicit class-specific call alongside the separate generic dilation; scaling every timer by 0.01 would discard that distinction. Exact sources: , `shared-transform-control-carriers.txt`, `shared-cue-extraction.json` and `cue-supplemental-joins.json`.

### Skill display actor dependency checked, October 18

Warning-free **Buff_MintSkill_Display#/1 → GC_Mint_Skill** reaches a separate post-effect actor, **BP_SpaceBase**, under the cue's selected net-mode/local-client gate. It spawns/attaches that actor or replays its timeline; OnRemove calls EndEffect. A targeted read of the exact external dependency is warning-free: **BP_SpaceBase.EndEffect#/3 → graph#/4:448** reverses its timeline, whose reverse-finished path destroys the actor; its updates change a local radius, mesh scale and material opacity. The carrier separately requests removal on Melee montage finish/role-out and has a 100 fallback duration. This reached display chain supplies no Wind damage producer, stamina write or native hold/recovery deadline; its 1.1 timeline is not an attack duration. The inherited native visual-overlap component is not a recovered combat executor. Sources: `mint-space-effect-functions.txt`, `space-effect-extraction.json`, `remaining-control-cue-carriers.txt` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Real timing/admission:** Skill/Burst/Normal/alternate action hit spacing, real recovery, earliest next input versus switch, native chain reset/selector priority, cooldown origin/progress and interruption/pending-packet behavior. Use the three-method stop above and shared native consumers; no 1:1 asset-time default is established.
- **Hold/plunge/Riposte/Support legality:** native hold predicate and release/cancel handling, actual stamina settlement/efficiency/regen/end callbacks, height-to-plunge scaling, Critical Dodge/Parry admission and Support Cycle-cost/activation consumer. `GA_Mint019_QTE#/3` and `GA_QTEBase#/6` retain an AbilityHeterochromeCost with no scalar properties; the inherited activation component declares numeric condition 20. These are not proof of zero cost, a free QTE, or a recovered enum meaning.
- **Damage/resource recipients:** hit multiplicity per recipient, cap/selection/distribution for in-range dummies, target deduplication/death/miss/zero-HP-loss behavior, ChargeAdd versus Cycle settlement/recipients and R6 team propagation. Selected range children omit MaxCount; do not infer an unlimited or one-target native cap. Spatial AoE shape/radius/placement is outside the simulator scope, but recipient policy remains relevant.
- **Passives/awakenings:** A3 reaction-versus-QTE trigger conflict and child stacking/refresh; A4 local-query semantics versus global enemy count; A6 inclusive-versus-strict threshold and native capture/default paths; A1 target/source callback routing and shared guard ordering; A5/R6 event delivery and expiry clocks; passive initialization/death/field transitions and Blossom range consumer.

These questions remain material even though coefficients, source event maps and stat zones are now located. Return on the named native consumer/default source or applicable independent observations that discriminate the alternatives. A missing reader adapter is an implementation gap; none of the above is closed merely by exporting more same-name assets.

## Prior findings

- [`reviews/q10-game-action-binding/mint-action-binding-model.json`](../../../../reviews/q10-game-action-binding/mint-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/mint_actions.rs), [`mint-normal-chain.json`](../../../../reviews/q10-game-action-binding/mint-normal-chain.json), [`mint-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/mint-skill-burst-packets.json).

## Research history

- Mint has accepted all eight blocks; use the complete implementation handoff above. Next open owner/block is **Nanally/packets**. Collection continues there before simulator implementation.
- Already used for Mint: selected action/notify/calculator and parent graphs, protected-native metadata/body attempts, public Mint/cycle/account-stamina pages and the saved public trace/calculator investigations. Do not repeat these probes for optional native accuracy. Account stamina is an explicit required profile, not a discovered universal numeric default.
- Starting batch ref: `4fd37a547e52a73d183e0aba14d3d1c5de5ad2b6`. Fraction-only two-cycle check: ; no game code or simulator changes. Remaining native uncertainties below/above are factual limits of the chosen model, not requests for another collection pass.

## Relative combat timer binding

`Mint_UltraSkill` Common.EndAbilitySkill at 3.903489 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
