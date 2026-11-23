# Hathor action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1025`.

**Responsibility:** Hathor outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected8/8 source/inference handoff below. Historical executable slices are narrower; collected implementation readiness is distinct from current runtime coverage and native-fidelity certainty.

### Executable Normal-clock prerequisite

`hathor.ordinary_clock_v1` binds the
five-stage1/5/2/3/2 train with separate contacts and one resource/pause
opportunity per connected causal packet. Melee1/2/3 preserve the16.666ms
pause onset delay, distinct from hit time. Their reached rate segments are
constant0 over33.333ms; Melee2_1 uses16.666ms, Melee4/4_1 use33.333ms, and
Melee4_2 uses50ms with its negligible native rate rounded to0 integer PPM.
Melee5 uses16.666ms at.01; Melee5_1 integrates the specified curve over its
native133.333ms window. Only positive actual HP loss advances these owner
pauses; recipient count cannot multiply them or energy/Cycle. Subsequent
contacts and recovery project through the same owner clock; real DPS time,
resource carry and world systems continue.

Each authored coordinate rounds to whole microseconds, with recovery at the
first representable deadline reaching its projected offset. Connected stage
recoveries are1.134310/1.517336/1.317210/1.867414/2.388345s, sum8.224615s.
Exactly queued inputs advance stages; idle, field exit and interruption reset
them and cancel unissued contacts. Tap uses the separate Power model below;
held/Branch/plunge/Burst and paid secondary bindings compose in their own
prerequisites and the [full-kit acceptance](#executable-full-kit-acceptance).

Independent Rust checks preserve two continuing trains, one/three-dummy
counts and resources, exact early-input diagnostics and interrupted packets.
Legal A0 Raging Flames T1 + four-piece Street Boxer native acceptance uses nine chains
per100s period, repeated at+100:234 packets, exact compiled ATK5949/2,
CR49/100 and per-packet additive gear zones. After200s energy retains100+
209/500 with301 overflow; Cycle retains100+391/500 with567 overflow. Native
aggregation/seeded replay retain those ledgers without resetting. This is a
clock dependency fixture, not complete kit acceptance. The existing packaged
CLI/source-binding/WASM oracle retains its narrower caller-timed coverage.

**Executable Tap/Power prerequisite:** `hathor.tap_power_v1` composes that
Normal clock with canonical unheld Skill: request2 Power at source offset
.283333, start16s CD at.300073, and emit Skill1/1/11 at.516245/.699503/.916369
on the action clock. Skill1/11 integrate the selected curve over33.333/83.333ms.
On a fresh connected cast the three real impacts occur.516245/.732420/.982202
and recovery1.577961, retaining integer-microsecond coordinate/curve projection
rather than rounding the entire analytical action once. Interruption before
the CD marker starts3s from interruption; at/after it retains the started16s
deadline. It cancels unissued contacts/notifies and keeps actual earlier gains.
Field exit preserves Power and the continuing world cooldown.

Power is an exact0..6 pool. Each actual accepted positive delta grants5 direct
self energy per unit through `hathor_power` receipts, without Charge Efficiency
or teammate sharing. Excess requests remain in the Power overflow ledger and
pay0. A5's separate lifetime earned counter increments only by accepted deltas,
cap15; spending/loss cannot turn it into the current balance. Source death
drains current Power and retains that earned count; `hathor_power_death`
identifies this loss in trace, separately from any future Cyclone payment.
Breakthrough4 own applied-damage kills enqueue1 Power per real target death
after the causal AoE settles, so A5 cannot retroactively change sibling
recipients. Another owner's kill, a zero-loss packet and lower breakthrough
cannot pay this passive. The shared death observer includes owned Break damage
on and off field; another owner's reaction kill cannot credit Hathor.

Native source binds `start_power` to an integer0..6 preload (default0),
before combat-entry effects. It sets the actual Power pool and Emergency
snapshot input without granting energy, incrementing A5 earned Power or using
A1's first-held reward. `.hathor.resources.power` reads the settled pool;
`.hathor.emergency_delivery.active` uses the same native half-open admission
predicate as actions. `source_power` checks two continuing taps with Power5/6:
only one accepted unit in the first case pays5 direct energy, independent of
Charge Efficiency and teammate sharing; both later taps overflow at cap.
Queries preserve complete seeded reports and direct native replay. Public
Cyclone/held input selection remains the source-action integration task.

Rust checks exact contacts/recovery/CD boundaries, notify-before-interruption
causality, one/three-dummy resources, direct energy with Charge Efficiency2
and a living peer, capped gains/A5, death, field exit and kill ownership. Legal
A0 native acceptance adds Tap at0/20/40 and nine Normal chains from2/11/22/31/
42/51/60/69/78, repeated+100 without reset. It settles252 packets in200s with
exact5949/2 ATK and Street Boxer CR49/100. Power finishes6 from12 requested,
with6 excess; only the first period's six accepted units pay30 direct energy.
Energy retains100+103/250 with382 overflow, Cycle100+122/125 with652 overflow.
Native aggregation/seeded replay and JSON/binary adapter parity preserve these
continuing ledgers. Gaze, held/A1, Burst/Emergency/Cyclone, combat transitions
and secondary actions compose in their own prerequisites below; the
Burst/Emergency model binds A3's live non-Boss target count.

**Executable ordinary Gaze prerequisite:** `hathor.gaze_v1` composes the Tap/Power
model with intrinsic ordinary Gaze and selected A2. Each living dummy independently
accumulates five unmarked seconds of declared proximity, then carries a nonrefreshing
five-second generation. Consumption/expiry requires a fresh five-second accumulation;
field exit preserves both timers. Explicit proximity changes reset accumulation,
while an existing generation retains its original expiry. Source/target death clears
its generations without a natural-completion award. All deadlines use real time and
stop scheduling beyond the authored horizon.

Typed `dummy_combat_events` declare anonymous-dummy `attack_start` observations,
with target and living party-recipient indices, or `proximity` eligibility changes.
An attack start consumes one ordinary generation for one requested Power even when
no damage event follows. A2 alone pays one on natural expiry. Expiry precedes a
same-time attack; a proximity change precedes accumulation completion. These inputs
contain no enemy identity, skill definition, damage producer or autonomous AI.

Rust checks independent targets, exact application/expiry/rearm boundaries, source
and target death, a dead recipient, off-field source/recipient, selected A2/A5,
cap overflow, invalid declarations and finite scheduling. Legal A0 native acceptance
retains the252-packet two100s Normal/Tap fixture and adds attack starts at9.9,
109.9 and119.8s. Only9.9/119.8 consume;109.9 follows the continuing generation's
expiry. Power requests14, accepts6 and records8 excess; energy/Cycle keep the
same exact continuing ledgers above. Aggregation, seeded replay and JSON/binary
adapter checks preserve the request and receipts. Held Five-Star Tracking and
incoming-contact provenance remain the next dependency; source lowering remains
native workflow. This prerequisite does not complete the owner kit.

**Executable held-Skill prerequisite:** `hathor.held_skill_v1` composes those
Normal/Tap/Power/Gaze models with canonical stationary unreserved Skill holds.
Below.22s selects Tap; otherwise total hold caps at5s. Actual actor birth is
.686667 and real ticks begin.936667 every.25s strictly before release. Full hold
emits17 ticks and10 separate PowerEnd contacts. Release<=1s uses ShortEnd's
three contacts and one tap2 request, including a release with no actor tick.
Release coincident with a would-be actor tick cancels that tick. No empty.5s
actor-table entry adds another train.

Each retained packet row consumes only native thirteen-level ATK coefficients,
category/type, Charge/Cycle/Break and fixed-crit fields. Power ticks cause no
owner pause; the ten Skill2/3/4 End contacts each use16.666ms at.01. Under the
integer coordinate/curve model, full recovery is7.214817s; ShortEnd recovery is
release+1.294583. CD starts16s from.236652 if that held marker occurs before
release; otherwise ShortEnd supplies release+.016368. Once started it never
restarts at section transitions. Interruption cancels unissued ticks/contacts
and notifies, preserves actual rewards and retains the selected3s/started16s
CD rule. The first actual birth with selected A1 requests6 once per battle,
marking it used even at cap; field exit/death/reporting periods do not reset it.
Combat leave/reentry remains a separate requirement.

Each actual actor tick applies enhanced Gaze even if its packet rounds to0 HP
loss. It replaces ordinary Gaze without natural-completion rewards; repeated
ticks retain the first five-second expiry. Selected A2 alone pays3 on that
natural expiry. A qualifying attack start or positive explicitly dummy-attributed
incoming HP/shield contact removes the enhanced generation before requesting3,
so a single attack-start/contact pair cannot pay6. Zero/unattributed incoming
events and paid HP costs do not qualify. After consumption a later real actor
tick may create a new generation; after release no ghost timer can do so.
`incoming_dummy_sources` explicitly binds incoming event indices to anonymous
dummy indices, retaining separate incoming HP/shield receipts and replay inputs.

Rust checks exact27-packet damage/resources/contact times, release/tick and
CD thresholds, interruption/birth causality, A1 once/cap/field exit, never-refresh
expiry/A2, zero-damage marking, incoming/start deduplication, actual shield contact,
HP-cost exclusion and invalid inputs. Legal A0 native acceptance uses two100s
periods with nine Normal chains, Tap0/40 and held20, with an explicit attack start
at25.5, repeated+100. It retains300 packets, including34 actor ticks/20 End
contacts; exact ATK5949/2 and Street Boxer CR49/100. Power requests14, accepts6
and records8 excess. Energy retains100+9/100 with433 overflow; Cycle retains
100+227/500 with737 overflow. Native aggregation/seeded replay and JSON/binary
adapter checks preserve these continuing states. Burst/Emergency/Cyclone,
conditional passives and secondary controls compose in the following
prerequisites and the [full-kit acceptance](#executable-full-kit-acceptance).

**Executable Burst/Emergency prerequisite:** `hathor.burst_emergency_v1` adds
canonical100-energy Burst/CD15, with the state event at1.499991, two collision
contacts2.483287/2.883287 and landing3.816348; recovery4.530561. There is no
immediate collision tick. Nonhybrid Emergency lasts8 World seconds from application.
It captures actual then-current accepted Power n as+.15*n base-ATK%, and the same
owner stat provider supplies outgoing damage and live-source ATK queries. It never
multiplies compiled final ATK by1+.15*n or recomputes after Power loss. A late gain
before the state marker counts; a rejected gain during Emergency cannot raise it.
Source field exit/death ends the snapshot immediately.

The optional `hathor.hybrid_emergency_v1` selects5s starting at initial Burst
recovery. Explicit interruption after state application ends that animation and
resumes5s from interruption. Only this admission timer suspends; cooldowns, Gaze,
hits and real combat time continue. Actual state expiry removes ATK/A6/R6 before
equal-time damage. This prerequisite has no Cyclone ATK-only retention branch yet.

Power-generation requirements reject before entering the pool during Emergency,
recording requested/accepted0 as `hathor_power_rejected`, with no energy/A5 award
or deferred delivery. The actual Gaze generation is still consumed and ordinary
proximity continues. Pool `granted`/overflow describe admitted grant requests and
cap clipping; rejected state-ineligible attempts remain distinct in trace. A3 now
uses each damage delivery's living non-Boss dummy count, including explicit zero;
Boss targets do not contribute to that count. A6 adds.30 CRIT damage and R6 adds
.10 DEF ignore only while the state is live. Private owner admission receives
actual combat time, including at an equal-time expiry.

Independent Rust checks cover n0/n2/n3 state snapshots, staticATK3000/base1000
producing3300, live-source agreement, exact recovery/CD/expiry, hybrid interruption,
mark consumption/rejected gains/A5, field exit/death, changing non-Boss counts and
rank6 live zones. Legal A0 native two100s periods retain five Normal chains,
Tap0/20, held40, explicit start45.5 and Burst50 per period. They emit202 contacts;
exact compiled ATK2974.5 becomes4153.5 from Power6/base1310. Raging Flames Q
uses one additive1.35 zone with DEF.5/RES.8. Without Cyclone spending, final Power
is6 from14 requests/8 excess, energy22+341/500 with132 overflow/200 spent, and
Cycle100+331/500 with440 overflow. Native aggregation, seeded replay and JSON/
binary adapter checks preserve those continuing states. Cyclone's real chain,
1/2/3 spends, DEF/A4 and ATK-only cleanup remain the next dependency, followed
by secondary actions/combat controls; this fixture is not full-kit acceptance.

### Executable paid Cyclone prerequisite

`hathor.cyclone_v1` composes the Burst/Power clock with canonical Skill
replacement during live Emergency. Each stage input spends actual Power1/2/3
at commitment, without refund. Ordinary Skill CD continues hidden underneath
and restores before equal-time ordinary admission at Emergency expiry. No
Power means a diagnosed input, never Tap fallback. The source-owned child
packets retain `GA_Hathor_Skill2`, E category and ordinary purchased Skill
talent plus resonance, rather than the grant's literal level1.

Seven Skill5, Skill6/7/8 and Skill9/10 contacts use the selected source
coordinates and connected owner pauses above. Integer-microsecond transition
gates are1.343016/1.450007s; final stage recovery1.627516s gives a4.420539s
connected chain. Quantization explains the approximately1us difference from
the analytical source-coordinate total. A late input must remain inside the
projected source window; outside it, ordinary recovery applies.

Source Skill.1 notifies1.066650/1.366748s occur after the first two packet
trains. Positive applied HP damage plus that marker increments A4 once per
stage, including AoE; stage3 increments at successful completion. Each stage
uses its captured previous count at+.25 CR per success, with sampled probability
saturating at100% and the source counter capped10. Receipts retain nominal
additive CR above1; saturated hits consume no RNG draw. Zero-damage contacts cause no success or
pause; interruption before the success marker gives no increment. Actual
equal-time markers precede explicit SettlementAndDeath interruption.

An admitted stage installs+100% own base DEF until real completion or
interruption/field exit/death. Independent before-DEF incoming checks start
from base1000, static+.50 and flat100: DEF1600 yields403 HP from raw1000 at
dummy level80; Cyclone DEF2600 yields293. An equal-time incoming hit precedes
an explicit interruption, while a recovery boundary removes DEF before that
time's hit. No outgoing scaling or immunity is inferred.

At Emergency expiry, state admission/A4/A6/R6 end immediately. If a Cyclone is
in progress, only its captured base-ATK modifier remains until the earlier of
real completion or two seconds; explicit interruption removes it sooner.
Connected pauses discovered after state expiry reproject pending ATK expiry
against that same original safety bound, rather than freezing an early finish.
`hathor_emergency_state_end` and `hathor_emergency_end` distinguish these
deadlines. Cooldown restoration occurs once and cannot overwrite a later
ordinary cast. Spending to zero leaves the admitted stage and ATK snapshot
intact. Gaze/world/resource timers continue throughout.

The legal A0 native fixture now adds a full chain at55/56.343016/57.793023s
in each of two100s periods, plus paid Dodge/Riposte at75/76s:230 total contacts,
exact compiled ATK2974.5 and
Emergency ATK4153.5. Raging Flames stages use additive zones1.40/1.45/1.45
versus Burst1.35, independently checked per packet. Power requests16, spends12,
clips3 and finishes1; final energy57+239/500 with191 overflow and200 spent,
Cycle100+167/250 with538 overflow. Paid Dodge spends/replenishes30 stamina.
Native aggregation/seeded replay and JSON/
binary adapter checks preserve the continuing ledgers. Rust checks also cover
insufficient/zero Power, three-dummy A4 deduplication, zero rounded damage,
late-state ATK-only retention and its2s safety bound. Branch/fallback dispatch
composes in the [next prerequisite](#executable-held-branch-prerequisite); the
composed owner evidence is in the [full-kit acceptance](#executable-full-kit-acceptance).

### Executable battle lifecycle prerequisite

`hathor.combat_lifecycle_v1` composes the Gaze/Power and Burst clocks with
explicit alternating combat leave/reentry. Exit cancels owner montage/mark
events, Emergency including pending ATK and Cyclone DEF, drains current Power
without a positive-delta award, and clears the A1 once-per-battle marker/A5
earned count. Power's exact simulation-wide ledger records that loss as a
spend with a distinct `hathor_power_combat_exit` reason. Energy, Cycle,
ordinary cooldowns and RNG continue. Exit before Tap's CD marker preserves
the actual3s cancellation cooldown; reentry does not reset it. No action or
Power generation is allowed outside combat, including a later equal-time
B4 reward callback queued by a hit before exit. Field exit/report boundaries
retain battle markers; defeat alone still preserves A1/A5 within that battle.

Reentry starts unmarked5s proximity accumulation from the latest authored
dummy proximity value, including changes made while Hathor was out of combat.
It cannot replay a past attack or reset another party Hathor. Stored deadlines
and generation checks reject stale timers; exit/death removes pending owned
mark events. Every next Gaze deadline is retained even beyond the horizon and
reported as `hathor_gaze_deadline`; no background event beyond the horizon is
queued. The continuing main fixture explicitly checks95.5→100.5 and
195.5→200.5, preserving the phase across its100s reporting boundary.

Independent A1/A5 checks use two full holds in two battles:54 contacts,
12 Power granted,6 lost at exit and6 current, without overflow. A5 is+.06
in each battle, never+.12; each first Power tick is50509/100 noncrit at
ATK1000/level8 and DEF.5. Total energy127+169/250 conserves60 direct Power
energy plus67+169/250 packet energy, with27 overflow at capacity100.
Separate checks cover canceled early Tap, out-of-combat admission, exact
cooldown restoration, pending-Cyclone cancellation and mark rearm/proximity.

The legal A0 native lifecycle fixture has112 Normal/Tap/held contacts over
two explicit battle episodes. It reconciles10 Power granted/5 lost/5 current,
energy100+439/500 with123 overflow, unchanged live stat zones and exact native
aggregation/seeded replay. JSON/binary WASM preserves combat receipts and
future Gaze deadlines. This reset fixture supplements the main two continuing
periods, which deliberately contain no combat reset. Branch/fallback dispatch
composes in the [next prerequisite](#executable-held-branch-prerequisite); the
composed owner evidence is in the [full-kit acceptance](#executable-full-kit-acceptance).

### Executable grounded secondary prerequisite

`hathor.secondary_actions_v1` requires the accepted combat lifecycle and
Cyclone models. Shared stamina pays15 per front/back Dodge with its own
cooldown/recovery gate. One paid Critical Dodge or explicit parry during the
living active Dodge window arms one response; a canonical Normal consumes
that permission into two native Riposte contacts at341403/649874 source us.
Their recovery is1183548, with immediate Skill/Jump cancellation. No caller
can directly manufacture an unpaid response. Front/back Dodge use separate
996277/1009564 recovery,267501/331208 Skill and82544/0 Jump gates. Ordinary
Normal, Entry, Tap/held Skill, Cyclone and Burst retain their selected Jump
and switch gates; no Move input or geometry is inferred.

The first actual positive Riposte contact requests1 Power and starts a
nonrefreshing5s reward gate. Its second packet and AoE recipients cannot
repeat that reward. Gate admission precedes the deferred resource callback,
so A5 cannot change later recipients of that same packet; it can affect the
second packet. Emergency/cap rejection still consumes the reward gate.
Field exit clears response permission but retains that gate. Explicit combat
exit resets both. Deferred Riposte/B4 rewards capture battle identity and
cannot cross equal-time exit/reentry into a new battle.

Only an engine-qualified switch creates Support: two QTE contacts at633462
and824235 source us, recovery1350067 and switch gate1210463. Native QTE1
preserves400000 real us at rate.2, QTE2 preserves33333 at rate.01: second
contact1144235, recovery1703067 and switch1563463 when connected. These
constant intervals are read from the cached native GE assets; they also transform later actions if
interrupted mid-pause. These are owner-local pauses. An ordinary unqualified
switch changes field control without starting an attack.
Riposte uses purchased Normal plus R3; purchased QTE is unchanged by R3.
Positive packets grant their own exact Charge/Cycle once per causal group.

Independent Rust checks cover two-contact damage582.5/1114 at ATK1000,
R3 values629.5/1203, AoE-before-A5 settlement, exact5s eligibility, rejected
Emergency awards, zero rounded damage, paid cancellation, source control
boundaries, continuing dilation and stale battle callbacks. Legal native
Support checks both packets,8.2 Charge, outgoing100 Cycle spend, live
Psyche→Lakshana Stain and off-field Bit Game's+.10 baseATK for the active
recipient (2974.5+131=3105.5). The main two-period native fixture includes four
Riposte contacts and continuing ledgers; native aggregation/seeded replay
and bounded JSON/binary WASM transport retain the same causal result.

### Executable AirAttack prerequisite

`hathor.airattack_v1` composes grounded secondary controls with the shared
explicit stamina account. A falling Normal requires positive current Strength;
this is an admission predicate, without an upfront ten-point payment. Authored
`plunge_inputs` provides real airborne time and normalized height h. The one
native X landing packet scales by1+h, using Normal talent plus R3. Contact is
116417 source us after actual landing, recovery584348 and Jump484688; the
AirAttack.2 marker does not create another hit. Switch uses full landing
recovery. Inherited owner dilation transforms source costs/landing animation,
while the explicitly authored real landing time remains a world deadline.

The retained native `Hathor_AirAttack` and `Hathor_AirAttack1` sections both
have entry0..0.45 and a self-loop0.45..0.95. Thus the repeating span is0.5s,
while0.95s is the montage endpoint. `StrengthCost` sits at source0.6166667:
costs execute at616667+500000*n source us, rather than every0.95s. This
clarifies the selected timing shorthand below. Branch's separate1.5-origin loop has no
such stamina notify. No cosmetic transforms or source payload enter runtime.

Each actual loop callback spends up to10×declared use efficiency and resets
the shared2s recovery gate, conserving the exact account even on exhaustion.
Strength<=10 enters landing immediately at airborne entry or after a cost.
Authored landing precedes an equal-time cost. Source35 with zero recovery
therefore pays10 at616667/1116667/1616667, reaches5, and lands early: its
single impact is1733084. Initial1..10 lands without cost; zero rejects before
admission mutation. At efficiency2/current15 the actual payment is15, then
one landing. All unissued callbacks/impact cancel on interruption, field exit,
death or combat exit; actual spending remains. Pending costs are scheduled
one deadline at a time and bounded by landing and the scenario horizon.

Independent Rust checks bind damage1071/1606.5/2142 at ATK1000 and h0/.5/1,
R3 damage1157 at h0, actual causal costs, exact early recovery, equal-time
landing, AoE resource deduplication and zero rounded damage. A QTE interrupted
inside its.2-rate interval delays the next source cost to1603437 while keeping
the authored real landing1800001 and contact1916418. Positive connected
landing grants6.3 Charge and10.4 Cycle once; zero damage grants neither.

Legal A0 native acceptance supplements the main230-contact scenario with two
continuing15s periods:28 contacts (two landings plus26 Normals), energy57+101/500,
Cycle94+499/500, zero overflow and60 actual stamina spent/recovered. Compiled
ATK2974.5 and the additive1.25 gear zone give AirAttack h.5 noncrit
5949×3213/4000=4778.53425. Native aggregation/seeded replay and bounded WASM
JSON/binary transport preserve the result. Held Branch/fallback dispatch binds
in the next prerequisite.

### Executable held-Branch prerequisite

The public source binds these same typed inputs with
`hathor attack[hold=1.5s,launchable=true,landing=2s]`, or
`launchable=false,landing=3s,height_bonus=0.5` for fallback. Height defaults to0;
landing is a positive real duration from command admission, not a guessed motion
trajectory. Launchability is an explicit input assumption, not a target catalog
lookup. Both choices require the authored hold and the explicit shared stamina
profile. `hathor swap[hold=0.6s,launchable=true,landing=2s]` binds the entry
Branch to the real switch command; implicit switches preceding an ordinary
held Normal retain the hold/Branch on that Normal's actual index. Native owner
admission owns all reached-stage gates, costs and cancellation rules.

The source party projection selects native clock, Power, Gaze, held Skill,
hybrid Burst, combat, Cyclone, secondary and Remora dependencies before any
owner instruction, including off-field initialization. Ordinary inputs use the
same clocks; Branch additionally selects AirAttack and Branch. This does not synthesize dummy observations or
combat exits. [Rust source checks](../../../../crates/content/src/simulation/tests/source_branch.rs)
retain two launchable/fallback cycles across two dummies (four BranchEnd and
four fallback receipts,40 actual stamina spent), first Normal contact216746us,
first landing2000000us, explicit/implicit switch indexing, reached-gate failure
and native replay. Two finite5s held Skills retain54 contacts per cast across
two targets, with the second first tick at17936667us and no resource reset.
Ordinary source checks in `simulation/tests/source_directions.rs` retain two
five-stage Normal chains/26 contacts, first impacts216746/8441361us, energy44
plus301/500 carry and native replay. Front/Back paid Critical Dodge/Riposte
cycles retain the separate five-second Power-reward carrier: short-spaced
cycles grant one Power, while longer explicit waits admit the second reward.
At unit Charge Efficiency, qualified native Support keeps both QTE contacts and8.2 owner packet energy;
each living peer receives `8.2*3/5=4.92`, once across AoE recipients. Native
Chaos/Chiz CLI checks retain these packets/ledgers alongside their independent
Remora-end and timed Charge channels, with independent report attribution.

`hathor.branch_v1` composes the ordinary Normal clock and the AirAttack
fallback with authored Normal/entry holds. A hold opens Branch only when its
declared duration strictly exceeds the source lower bound of the reached
stage:1.3s for Melee1,0.5s for Melee2/3,0.9s for Melee4,0.6s for Melee5 and
0.5s for the automatic entry Normal. That lower bound is the causal gate. At
the gate, contacts of the interrupted stage that have not yet been delivered
are cancelled while already delivered contacts remain. The Branch lock starts
at admission, extends the owner lock to the gate when ordinary recovery ends
earlier, rejects Skill/Burst until it ends, and clears on every end/cancel.

A launchable dummy follows Branch/BranchLoop without a separate Branch stamina
cost until the authored real landing, then receives exactly one BranchEnd
contact at that instant with the Normal talent/R3 coefficient4.113, category
A, Charge12/Cycle20/Break2.4, recovery737229, Jump549863 and switch753203
before its connected owner pause over83.333ms (curve(0,.3),(.016666,.03),
(.1,.03),(.216666,1)). Integer deadlines round up: a continued Normal at
2815813 is accepted one microsecond after the2.815812s analytical boundary.
The gate and landing are reported as `hathor_branch_gate` and
`hathor_branch_landing`.

A non-launchable dummy transitions at branch progress0.52 into the AirAttack1
fallback loop, reported as `hathor_branch_fallback`. The first
10×use-efficiency Strength cost is at fallback+616667 actor microseconds, then
every+500000. Strength<=10 lands immediately at entry or after a cost; an
authored real landing wins over an equal-time cost. Fallback landing emits one
AirAttack packet scaled by1+declared height after116417 actor microseconds,
recovery584348, Jump484688 and switch584348. Actual costs are capped by
available Strength and conserve the shared exact ledger with its2s recovery
gate; interrupt, switch, field exit, combat exit and death cancel unissued
Branch contacts or fallback costs and impacts without refunding actual spends.
Every packet settles once across all recipients, and no geometry, knockback
direction or automatic target motion is modeled.

Entry Branch binds the automatic owner entry Attack through the qualified
`SwitchActor` command index, not an `InvokeAbility`; the generic Baicang and
Blackbird hold validators skip that index so the owner-specific validation
still requires the SwitchActor targeting Hathor, its authored hold, a
stationary unreserved input, a positive landing time and height in0..1.

Independent Rust checks cover the stage contact plus one BranchEnd, deliveredcontact retention at a stage-two gate, fallback authored landing/height,
equal-time landing precedence, immediate low-Strength landing, the entry
0.533333s gate, reached-stage hold validation, switch/interrupt/death
cancellation, fallback interruption keeping only actual spends, three-dummy
resource deduplication and two continuing cycles without reset. The legal A0
native fixture below executes two100s periods with a launchable Normal Branch
and a non-launchable fallback: fourteen connected packets, exact
45.814 Charge/76.126 Cycle, ten actual10-point fallback spends (100 spent and
fully recovered) and four gates. Native aggregation and seeded replay reconcile
the same receipts and continuing resources.


**Accuracy notes:** historical [execution gaps](#unknown-execution-policies) retain provenance; selected blocks settle the concrete campaign model.

## Selected full packet specification

**Mixed source/checked inference, October19:** this selected campaign model supersedes the historical partial executable slice. Source inputs: `DT_SkillDamageData`, `DT_HathorEffectFigure`, original `GA_Hathor_*`/montages/notify collision and jump objects, actor timer/condition/cue graphs below, permanent owner table (optional `hathor.json`). Public [current description](https://neverness.app/characters/hathor) and [independent kit table](https://www.icy-veins.com/neverness-to-everness/hathor-profile-skills) corroborate held finisher/Cyclone multiplicities. Earlier native/public investigations exhaust the producer/body approaches; missing settlement details receive concrete inferred rules below. The saved [video lead](https://www.youtube.com/watch?v=W2H_SpIG3wE) could not be fetched in this pass; no timing measurement or visual observation is claimed from it.

All selected packets deal **Lakshana**, scale from Hathor's **live ATK**, use normal crit/DEF/RES and their source category. Preserve full13-entry arrays, samples **levels1 /8 /9** below, effective talent minus one. Normal owns ordinary/Branch/plunge/riposte, Skill owns unheld/held/Cyclone (grant level1 is an ability-grant parameter, not forced talent1), Burst owns Q, purchased QTE unchanged by R3. Resources are raw energy/Cycle/Break **per positive connected packet**, never per dummy.

| Suffix after `GE_Player_Hathor_` | Count in complete action/category | Coefficient1 /8 /9 | Energy / Cycle / Break |
| --- | --- | --- | --- |
| Melee1_Damage |1 /A |.243 /.416 /.450 |1.219 /2.028 /.246 |
| Melee2_Damage |1 /A |.161 /.276 /.298 |.807 /1.343 /.163 |
| Melee2_1_Damage |4 /A |.136 /.233 /.252 |.681 /1.132 /.137 each |
| Melee3_Damage |2 /A |.133 /.228 /.246 |.664 /1.105 /.134 each |
| Melee4_Damage |1 /A |.224 /.384 /.415 |1.121 /1.865 /.226 |
| Melee4_1_Damage |1 /A |.479 /.821 /.887 |2.403 /3.997 /.485 |
| Melee4_2_Damage |1 /A |.994 /1.704 /1.840 |4.982 /8.289 /1.005 |
| Melee5_Damage |1 /A |.441 /.756 /.816 |2.212 /3.680 /.446 |
| Melee5_1_Damage |1 /A |1.099 /1.883 /2.034 |5.505 /9.159 /1.111 |
| Branch_Damage |1 final BranchEnd impact /A |2.400 /4.113 /4.442 |12 /20 /2.4 |
| AirAttack_Damage |1 landing /X |1.250 /2.142 /2.314 |6.3 /10.4 /1.3 |
| PerfectEvadeAttack_Damage |1 first riposte /Z |.680 /1.165 /1.259 |1.717 /2.851 /.859 |
| PerfectEvadeAttack1_Damage |1 second riposte /Z |1.300 /2.228 /2.406 |3.283 /5.449 /1.641 |
| QTE1_Damage |1 /H |.639 /1.095 /1.183 |2.620 /0 /.799 |
| QTE2_Damage |1 /H |1.361 /2.333 /2.519 |5.580 /0 /1.701 |
| Skill1_Damage |2 unheld /E |1 /1.714 /1.851 |2.833 /4.733 /.567 each |
| Skill11_Damage |1 unheld final /E |1 /1.714 /1.851 |2.833 /4.733 /.567 |
| Skill1Power_Damage |1 per actual held actor tick /E |.556 /.953 /1.029 |1.359 /2.267 /.270 |
| Skill2_Damage |4 held-End /E |.102 /.175 /.189 |.250 /.416 /.050 each |
| Skill3_Damage |1 held-End /E |1.225 /2.099 /2.267 |2.995 /4.995 /.596 |
| Skill4_Damage |5 held-End /E |.552 /.946 /1.022 |1.348 /2.248 /.268 each |
| Skill5_Damage |7 Cyclone1 /E |.429 /.735 /.794 |1.271 /2.129 /.257 each |
| Skill6_Damage |1 Cyclone2 /E |1.432 /2.454 /2.651 |2.435 /4.046 /.501 |
| Skill7_Damage |1 Cyclone2 /E |1.136 /1.947 /2.103 |1.931 /3.208 /.397 |
| Skill8_Damage |1 Cyclone2 /E |1.432 /2.454 /2.651 |2.435 /4.046 /.501 |
| Skill9_Damage |1 Cyclone3 /E |1.969 /3.375 /3.644 |3.115 /5.192 /.609 |
| Skill10_Damage |1 Cyclone3 /E |3.531 /6.052 /6.536 |5.585 /9.308 /1.091 |
| UltraSkill1_Damage |2 collision-window ticks /Q |.962 /1.649 /1.781 |0 /0 /.206 each |
| UltraSkill2_Damage |1 landing /Q |5.076 /8.699 /9.395 |0 /0 /1.088 |

Level-eight derived totals: ordinary Normal13 packets **7.628ATK /22.301energy /37.099Cycle /4.498Break**; unheld Skill3 packets **5.142 /8.499 /14.199 /1.701**; held End10 packets **7.529 /10.735 /17.899 /2.136**, plus each actually emitted Power tick; Cyclone stages7/3/2 packets **5.145 /6.855 /9.427ATK**, full12 packets **21.427 /24.398 /40.703 /4.898**. Burst3 packets **11.997ATK /0 /0 /1.500**; QTE2 packets3.428ATK/8.2energy/2.5Break; riposte2 packets3.393ATK/5energy/8.3Cycle/2.5Break. Source riposte's second1.300 at1 differs from a saved public1.500; choose actual row, not an unbound third/doubled actor hit. `Melee.4_3`, Branch1/2/4 are control effects, not more damage rows.

**Held model:** actor manually requests one Power damage every**.25s** after its spawn, with first tick+.25, no immediate damage. Native separate actor-table.5s sentinel route has an empty damage payload and does not add a second timer. Entry held threshold.22, Power Skill.6 actor spawn at+.46666664 =>birth **input+.68666664**. For maximal **5s total input hold**, actor ticks at **input+.93666664+.25*k**, k0..16 (17 ticks), then release cancels future actor ticks and enters its10-packet End. Full selected hold therefore27 packets **23.730ATK /33.838energy /56.438Cycle /6.726Break**. The source4.5s displayed charge is rounded actor-working time after entry; do not expand it into another20 ticks without accounting for startup. Early release uses only ticks already issued, then its appropriate End route; interrupted action cancels unissued End requests rather than dumping the full coefficient.

**Burst collision inference:** `NotifyState_CollisionDamageCheck` opens at**2.0832872**, duration.8833289, `bMultiDamage=true`, period**.4**. Select first periodic damage after+.4, then+.8, at **2.4832872 /2.8832872**, plus source landing3.8163476; this gives the independently published/native2+1 hit count. An extra immediate collision tick would give3+1 and contradict the multiplier. The event-data target set is all declared eligible in-range dummies. Emergency state applies earlier1.4999913, so neither collision nor landing can borrow a future stack count or skip its existing state bonus.

**Other branches:** successful To The Skies produces one4.113 BranchEnd hit; an unlaunchable target follows source AirAttack1/Final Dispatch and produces the2.142 plunge instead, not both. Plunge coefficient multiplies by1+h for declared fall bonus h in[0,1], paying real descent/landing. Riposte actor/direct paths deduplicate the one second packet, and its five-second Power-reward carrier is independent of its two damage hits. Native orphan `GuangXiangFive_Damage` has fixed1.5ATK/categoryP/zero resources, but no selected current passive/ability producer: the described Remora enhancement is duration/crit, so do not append an invented1.5ATK explosion to each expiry merely from that row's name. Keep the unbound row as historical candidate data; it is not an applicable automatic current-kit proc. Power, Gaze and Emergency ATK snapshot/cleanup rules in subsequent blocks are required for all these packets.

## Selected full timing specification

**Mixed:** native montage coordinates advance1/s on one real clock, transformed by each connected packet's configured pause. Where a rate curve is bound, choose linear interpolation of its recorded knots in seconds since pause onset and integrate over `AttackDilationTime`; action-clock loss is duration minus that integral, with constant endpoints. This is a declared timing-consumer model, not measured game animation speed. Preserve delayed onset separately. Missing positive duration gives0 extra pause; no global time stop, no per-dummy multiplication. Fraction audit  retains exact values and original curve knots; the following values/rules suffice without another source lookup.

Pause losses: Melee1/2/3 each.033333 (Melee3 twice), Melee2_1 each.016666; Melee4/4_1 each.033333; Melee4_2 **.04999999957543668**; Melee5.01649934; Melee5_1 **.120874654833335**. Native curves: Melee1(0,0),(.016666,0),(.066666,0),(.1,.5); Melee2(0,0),(.033333,0),(.066666,.5); Melee3 last value1 instead.5; Melee4(0,0),(.05,0),(.066666,.5); Melee4_2(0,0),(.016666,1.389485e-8),(.066666,0),(.1,.5). Melee5_1(0,.075),(.016666,0),(.066666,0),(.083333,.01),(.1,.10000001),(.116666,.3),(.15,.9). Melee1/2/3 pause onset delay.016666; others0.

Branch loss**.0785831**, curve(0,.3),(.016666,.03),(.1,.03),(.216666,1), configured duration.083333. Skill1/11 share curve(0,.05),(.016666,0),(.066666,0),(.083333,.5), durations.033333/.083333 =>losses **.03291635/.0787496**. QTE1 loss.32, QTE2.03299967. Skill2/3/4 each.01649934, Skill5 each.03233301, Skill6.016666, Skill7/8zero duration, Skill9.0495, Skill10.07833302. Power ticks/initialBurst/collisionBurst/riposte/plunge have no reached positive-duration owner pause in this profile, hence0. The unused Burst final rate curve cannot create duration by itself.

| Action | Damage coordinates / source events | Next ability / jump / move before caused pause |
| --- | --- | --- |
| Normal1 |.21674645 row1 |1.1009765 /.31691593 /.6337666 |
| Normal2 |.19996408 row2; .38319457,.48346406,.6332085,.75006443 row2_1 |1.417339 /.8698995 /1.9172547 |
| Normal3 |.29969767,.38374716 row3 |1.2505444 /.71656144 /1.5502797 |
| Normal4 |.1997965 row4,.41657436 row4_1,.799878 row4_2 |1.7507477 /.9499469 /1.5006015 |
| Normal5 |.3991964 row5,.9409834 row5_1 |2.2509708 /1.5002936 /1.7997766 |
| Entry Normal1 |.21638927 |1.1008182 /.31668073 /.6335462 |
| Approach Normal1 |.21648204 |1.1007631 /.3174634 /.6342689 |
| Tap Skill |.51624507,.69950277 row1; .9163689 row11 |1.4333775 /1.0578158 /1.7333286 |
| ShortEnd route from released held action |.23336136,.41661298 row1;.6334531 row11 |1.1500001 /.7666795 /1.4500123 |
| PowerEnd after release |.3499315,.5332783,.7332178,.8498296 row2;1.0498486 row3;1.6162666,1.6498623,1.68302,1.7163224,1.7495753 row4 |2.0498233 /1.8334148 /2.2500243 |
| Cyclone1 |.7499164,.8000236,.8500306,.90003324,.95003736,1.000019,1.0500342 row5 |1.914334 /1.5821643 /2.1001263 |
| Cyclone2 |.5166235 row6,.94992375 row7,1.2664492 row8 |2.169196 /1.833353 /2.4182043 |
| Cyclone3 |.28306246 row9,.7666661 row10 |1.499682 /1.2483995 /1.6501306 |
| Burst |state1.4999913; collision damage2.4832872,2.8832872; final3.8163476 |4.5305605 all; switch lock4.5228405 |
| QTE |.6334618 row1,.8242353 row2 |1.3500668 /1.183819 /1.8345151; switch lock1.2104626 |
| Riposte Atk |.34140253 row0,.6498742 row1 |1.1835483 all; Skill/jump0 |
| Plunge landing |.11641741 (AirAttack.2 at.08290854 is not a second damage) |.5843476 /.48468775 /.7673271 |
| BranchEnd |0 (source9.313226e-10 treated as0 at microsecond conversion) |.7372286 /.5498631 /.7532025 |

Full Normal recovery is**8.22461439440877s** under the curve model (exact sum retained by Fraction audit). Tap recovery**1.5779598**, ShortEnd1.2945824, held PowerEnd**2.2148167**, QTE1.70306647, ordinary Burst4.5305605. No coincident multi-hit replacement. Author next inputs at legal recovery/queued-combo windows, including all already caused pauses; an early jump/move cancels unissued events and cannot borrow their damage/resource grants. Field exit obeys switch lock/recovery, clears on-field montage events but not already emitted independent impacts; committed energy/Power is not refunded. Normal resets on another admitted action/field exit or explicit unqueued idle; a queued ordinary chain follows five stages.

**Held Skill:** direct hold selects Power at input+.22, source Power montage spawns its actor at+.46666664. First timer is birth+.25; grow range at actor age1.5 and3 inclusively, leaving coefficient unchanged. Max total input hold5 transitions PowerEnd immediately and stops actor before later timer work. Release at hold<=1 selects ShortEnd; >1 selects PowerEnd; this is the selected inclusive threshold interpretation of the native release-map1/999. Early release before hold threshold takes ordinary tap. Power actor ends on release/interrupt/field exit/death, not after a final ghost timer callback from its source false branch. Full5s hold has17 ticks (range stages5/6/6), lastinput+4.93666664, PowerEnd recoversinput+7.2148167. A release with no emitted tick still uses its appropriate end train; do not grant a full hidden20-tick loop. Held CD16 starts **input+.23665186** (.22 + Power notify.01665186); tap starts.3000733; ShortEnd path uses its notify.016368438 from that section if the original marker has not fired. One CD starts once, never restarts for each section. Pre-marker interruption3s from interruption; later interruption retains started16 deadline.

**Cyclone combo transitions:** queued Skill during source jump windows moves from stage1 to2 and2to3 only if live Power/state predicates pass. Native notify starts/durations are(.50001776,.66657865) and(.6666748,.81668955), with nFrame37/46. Select **60fps frame offset from notify start**: nominal transitions **1.1166844266667 /1.4333414666667**, both inside their windows and after each stage's damaging train/Skill.1 marker. An absolute37/30 reading lies outside the first window; this consistency check selects the frame model without claiming measured FPS. Add actual caused pause => **1.3430154966667 /1.4500074666667** real stage times. Stage3 recovery**1.62751502**, full queued combo**4.4205379833334s**. If no queued input or insufficient Power, use that stage's ordinary recovery; cannot skip to stage3. The granted Cyclone uses its separate recovery/Power gate and **does not wait for ordinary Skill CD**, which continues underneath and is restored later. No extra numeric Cyclone cooldown is configured.

**Emergency mode timing:** select nonhybrid native **8 World seconds from state event1.4999913**. Shared cinematic suspension retains the carrier's full remaining duration until release; the historical4.9694308s post-recovery calculation assumed uninterrupted absolute expiry and is not the current model. Hybrid source returns5; its selected initial-animation suspension composes with the shared pause without double-counting. These assumptions do not establish a measured native playable window. Switching/death ends state immediately. Source ATK cleanup can retain **only its ATK carrier** for an in-progress Cyclone until earliest montage finish or2s after state end; A4/A6/R6/live state admission end immediately. It is not an unconditional two-second buff or permission to start another stage after expiry.

**Held Normal/air alternatives:** enter Branch through each source stage's open hold window: Normal1(1.3,1.6), stage2/3(.5,.8), stage4(.9,1.2), stage5(.6,.9), entry(.5,.8). If another queued input would have ended a stage earlier, author the hold instead; do not automatically jump before its gate. Already delivered Normal hits stay delivered. Branch control events.35005626/1.2830819/1.4666667 deal0 HP damage. For a launchable target choose actual authored landing/release into BranchEnd (native OnLanded check.97, loop origin1.5), emit its one impact then recovery. No fixed movement/height is invented; scenario declares `can_be_launched`, landing and h. At branch progress.52, a target still not in Floating state takes AirAttack1 fallback; it emits only ordinary plunge at landing. Skill/Burst lock starts Branch admission, clears on every exit/cancel/end, never persists after a failed branch. Airborne entry lasts.45s and repeats the.5s section ending at source.95; native stamina−10 starts at.6166667, then every.5s; exit to landing at authored landing or native Strength<=10 stop. Activation while falling requiresStrength>0. Account profile and authored descent determine costs/time; zero/<=10 are guards, not a universal ten-point admission fee. Critical Dodge costs common15/.9s, with front/back any.996277/1.0095644, Skill.26750147/.33120757, jump.082544/0, backMove.7951953. Successful explicit response arms one riposte, not free repeated inputs.

## Selected full resource specification

**Mixed:** energy starts0..100/cap100, Burst spends**100** at commitment/CD15, Cycle separate0..100. Use [shared per-positive-packet resources/carries](../../shared/energy-and-gauges.md), living off-field.6 distribution and qualified outgoing100-Cycle Support switches. Ordinary/held Skill, Normal/Branch/plunge/riposte/Cyclone have0 Ultimate cost; their Power, state, stamina and cooldown admissions remain real. Base Skill is replaced by Cyclone during Emergency; with no Power its input fails rather than using ordinary tap to refill itself. No generic auto wait or refund on a failed/committed input. Fractional carry persists across caps/spends/switches/periods.

**Power pool:** integer0..6, baseline0, Infinite lifetime, survives field exit; no ten-second decay from inactive duration scalar. Admit positive generation **only while not Emergency Delivery**. Clamp to6, actual positive delta drives listeners; overflow is recorded and grants no extra energy/A5 count. At accepted delta d grant **5*d direct self Ultimate Energy**, ignoring Charge Efficiency and without teammate sharing, independently of ordinary packet awards. This follows native positive-stack-change callback/curve5 and [independent resource explanation](https://neverness.app). Treat its one-second CheckStack as this callback's delta context, consumed/cleared after one settlement, not an accumulator that pays old gains again. Two closely spaced gains2 then1 grant10 then5, not10 then15; cap5 +request3 accepts1 and grants5, not15. A preload is not a positive-change event. Source death clears Power; a new combat resets it, while reporting boundaries do neither. Death/current combat loss does not manufacture A5 gains.

| Actual producer | Complete admission/award |
| --- | --- |
| Tap Skill.5 |+2 once at source.28333333, not per hit; ShortEnd.5 at section0 if that release path selected. Held PowerEnd does not also grant tap2. |
| A1 first held Skill.6 |At actor-spawn event input+.68666664 in battle, selected A1 and unused battle marker: request+6, clip at6, set once marker even at cap; field exit/death does not reset battle marker, combat exit does. A hold cancelled before this event grants none. |
| Excellent Rating Lock consumption |One own normal Gaze generation removed on marked enemy skill-begin targeting the same owning player's living team =>+1 once. Does not require HP loss. |
| Five-Star Tracking consumption |One own GazeEx generation removed on eligible marked enemy skill-begin **or** qualifying hit on that player's team =>+3 once, deduplicate both routes by generation removal before grant. |
| A2 natural Gaze/GazeEx expiry |Selected A2 only: +1 /+3 once on actual normal timer expiry. Consumption, dispel, replacement, source death or combat exit are not natural completion and do not also pay. |
| Riposte |First positive riposte hit while its five-second nonrefreshing reward gate is absent =>+1; its second packet cannot pay again. Gate starts on actual reward request, remains through field exit; ordinary dodge without hit gives0. |
| B4 kill |One target death caused by Hathor-owned applied damage =>+1 once; off-field owner damage eligible, team kill by another source is not. Emergency rejects generation. |

Any marked enemy event during Emergency still consumes that generation, with0 Power accepted and no deferred grant. No rank/kill success is synthesized by stationary dummies; explicit enemy attacks/target deaths are scenario/events. Source application requirements are side-effecting: evaluate each once on real delivery, never once for readiness prediction and again for mutation. A2 duplicate CDO/component callback lists deduplicate the same generation.

**Cyclone spends:** stage1 requires Emergency+Power>=1, snapshot relevant cast context then **spend1** at commitment. Queued stage2 requires live Emergency+remaining>=2, spends2 on transition. Stage3 requires live Emergency+remaining>=3, spends3. Thus6 ->5 ->3 ->0 enables all three, not three one-Power casts; with2, stage1 leaves1 and stage2 fails, though a later separate stage1 may spend the remaining1 while state lasts. No committed Power refund after cancellation; unadmitted later stage spends nothing. Last spend removes further grant permission, but source `RemoveAbilityOnEnd` allows the already admitted last stage to finish. Stage effective talent is ordinary Skill's purchased+R3 level, not literal grant-level1. State expiry/switch prevents new transitions; queued inputs must recheck current resources and live admission.

**Emergency ATK:** at state application capture current accepted Power n, compute **+.15*n baseATK%** once through the located magnitude helper, retain that snapshot through state lifetime/explicit ATK-only pending-Cyclone cleanup. Spending Power does not retroactively shrink the already applied modifier. At n6 this is+.90 baseATK, not flat+.15, not a multiplier1.90 of finalATK; n0 contributes0. Native helper evaluates count at application and the special ATK cleanup when grant/Power ends supports this choice. A later Emergency cast replaces the snapshot, no stacking. Generation is blocked for the state, so another kill cannot raise it midcast. Crit/DEF/resonance channels use their own live state rules; no generic all-stats snapshot is implied.

Stamina profile must declare capacity/initial/use efficiency/recovery and uses shared2s recovery gate. Ordinary hold Skill/Cyclone have no additional reached stamina cost and are modeled0, whereas falling Normal repeats its explicit10 loss and dodge15. Airborne admission/stop predicates are bound by the [AirAttack model](#executable-airattack-prerequisite). This complete conditional account rule avoids fabricating a universal capacity or replenishment speed. Waits and real action time are authored; each continuing cycle retains all pools, marks, timers and pending impacts.

## Selected full effects specification

**Mixed:** all magnitudes come from native figure/condition/cue findings above; remaining lifecycle choices follow the descriptions and causal counterchecks. Do not repeat side-effecting application requirements during readiness checks. Marks are keyed by(source Hathor,enemy,generation), pool/buff state by actual source; a display/cue name never creates an extra damage packet.

**Excellent Rating Lock / Gaze:** the [independent complete kit](https://www.icy-veins.com/neverness-to-everness/hathor-profile-skills) includes the proximity mechanic omitted by the frozen shorter public description. While Hathor belongs to the team and a living enemy remains in declared proximity for**5s**, apply one ordinary Gaze to that enemy for**5s**, no refresh while present. Default all-in-range stationary scenario declares proximity true; an explicit proximity change resets that enemy's accumulation. After natural expiry/consumption/replacement, require a fresh5s unmarked proximity interval before reapplication; this rearm convention is inferred to prevent continuous readiness polling from generating instant endless marks. Each eligible enemy has an independent timer, stable target-index order. Existing GazeEx suppresses new ordinary Gaze and resets the proximity accumulator, rather than installing both at once. Hathor may be off field; source death/party removal clears her proximity/mark state. Buff UI Gaze_Stack/Menu is not an independent Power source.

Ordinary Gaze's first qualifying **enemy SkillBegin targeting any living character in the same owning player's equipped party** removes the generation and requests1 Power. The recovered check compares owning player state, not just Hathor object equality; an attack on another teammate qualifies even if Hathor is off field. SkillBegin is sufficient, actual damage/landing not required. No party-directed enemy attack => no award except selected A2 natural expiry. The source's Gaze5s duration is never increased by polling or hitting the marked enemy. At t5 application expires10: an attack at9.9 consumes it, gives1; at10 after expiry cannot consume that generation. A2's expiry callback may already have paid once before the10s attack, never both.

**Five-Star Tracking / GazeEx:** each actual held-actor tick requests one enhanced mark on each eligible hit-range enemy, whether its damage subsequently rounds to0 or not. First accepted request creates a **5s**, source-cap1, **NeverRefresh** generation, removes ordinary Gaze/its UI stack, and resets ordinary proximity accumulation. Further requests while it exists retain original expiry; after consumption/expiry a later actual tick may create a fresh generation. First qualifying enemy SkillBegin on the source player's party **or** enemy positive hit whose last damage instigator belongs to that player consumes it and requests3 Power. Remove generation before entering either callback, so attack-start plus contact cannot pay6. The source LastDamageInstigator/attack-target routes are not a reward for Hathor simply hitting the marked enemy. Mark consumption remains real even at Power cap/Emergency rejection; no stored future refund. UI cleanup may run+.2 after missing both marks, but cannot extend gameplay duration or award Power.

Held actor growth changes only recipient range at ages**1.5 /3**, inclusive; declared all-in-range eligible dummies fit every stage unless scenario specifies per-stage range eligibility. Actor lifetime is scoped to its real held action; stop at release/ability end/source death/field exit, invalidating pending callbacks before dispatch. Empty.5s actor-table configuration does not duplicate the manual.25s timer. If a timer and release coincide, selected end/expiry processing wins and cancels that timer; no destroy-then-one-last-hit from the native false-branch continuation. Example full hold40: birth40.68666664, first mark40.93666664, expiry45.93666664 despite16 further requests. Enemy SkillBegin45.5 consumes it for3 requested Power; a hit45.6 cannot pay again, since actor stopped45 and cannot reapply it. A2 natural completion pays0 for the consumed generation.

**Emergency Delivery:** one self state, source-mode duration chosen in timing block, field exit/death removes it immediately. It pauses ordinary **Power gain admission**, not cooldown/world/mark timers. At state entry snapshot Power n and install+.15*n **baseATK%**; cap1/reapplication replacement. Movement+.30 is outside stationary DPS movement, but cannot change action/recovery speed without a separate source. Base Skill slot becomes Cyclone and requires Power. At state end remove A4 stack pool/A6CD/R6DEFignore and new-Cyclone permission immediately. Keep only already applied ATK until an in-progress Cyclone finishes or2s safety deadline (earliest), per located conditional retention; otherwise ATK ends with state. No blanket two-second extra Emergency state is granted.

**Cyclone DEF:** native figure `Buff_Hathor_Skill2_DefUp=1.0` gives **+100% of own baseDEF** during an admitted Cyclone stage/chain, summed with otherDEF%; remove on action/chain end, interruption, field exit or death. It matters to explicitly modeled incoming damage; it is not outgoing DEF scaling, immunity or a second attack bonus. No own heal/shield or hidden autonomous summon damage is configured. The summoned motorcycle is the admitted finite Skill packet train, not an off-field turret. Break/RNG/gear listeners still see its actual E category.

## Selected full passive specification

**Mixed:** select individual A1..6 by identity, R3/R6 by count. Preserve all resource/effect conditions even if a chosen fixture does not select them.

| Selection | Complete rule |
| --- | --- |
| B2 Delay Warning | Qualifying Cosmos+Lakshana Remora with living party Hathor/B2 uses **12s** instead of baseline5, target cap1/refresh. Every eligible living allied damage actor gets **+.10 CR** against that enhanced Remora target, evaluated live per packet, not just Hathor or a team-wide unconditionalCR buff. Fixed crit/no-crit reactions retain their own overrides. Target status/bonus lasts its selected generation even if Hathor switches; no new dead-source enhancement. |
| B4 Efficiency Boost | Hathor-owned actual kill grants1 Power once when not Emergency, including owned off-field damage. No team-global kill credit or per-overkill packet grant. |
| Intrinsic Gaze proximity | Team membership enables the5s proximity/5s ordinary Gaze rule in effects; it is not a city passive or disabled because the saved abbreviated kit omitted it. |
| A1 | First actual held Skill.6 in battle requests6 Power and sets battle marker; no new grant each held cast, no reset on field switch/reporting period. |
| A2 | Only normal expiry of Gaze/GazeEx grants1/3; no consumption/dispel/death duplicate. |
| A3 | Own general damage bonus+.15 per live **non-Boss** target, additive ordinary damage zone; require declared target classification, no fixed boss assumption for every dummy. Applies to owned direct/proc damage if ordinary bonus eligible, not automatic extra HP damage. |
| A4 | During Emergency each **successful Cyclone stage** snapshots previous counter k for that stage's eligible E packets: CR+.25*k, clamped with totalCR. After stage1/2 positive packet train, source Skill.1 marker increments once for the next stage/input; stage3 increments once at successful completion if state still live. Native carrier cap10, no invented cap4. An interrupted stage which never reached success marker does not increment; no positive damage =>not successful. Counter persists across chained stages/separate Cyclone inputs only within current Emergency, then clears. |
| A5 | On actual accepted positive Power delta d while marker exists, lifetime earned count+=d, cap15; general damage+.01*count. Spending Power does not reduce earned count. Overflow/rejected/Emergency requests grant0; not currentPower*.01. Preserve through field exit/death/revival during combat, clear at combat exit. No initial synchronization from a preloaded pool. |
| A6 | SelfCD+.30 while Emergency live, removed at state end even if ATK-only retention remains. |
| R3 | Normal/Skill/Burst effective level+1, including Branch/plunge/riposte and all held/Cyclone rows, not QTE; native13 cap. |
| R6 | Self targetDEFignore+.10 while Emergency live, ordinary additive ignore zone, removed at state end; not10% DEF shred on every target or a Mental bonus. |

A4 consistency: with fresh counter0 and full six-Power combo, stage1 has no addedCR; stage2+.25; stage3+.50. Its own successful stage cannot retroactively buff earlier hits. If state expires between hits, live-state gating removes A4/A6/R6 from later hits despite retained ATK; no universal frozen offensive stats. A5 gains2+2+actual2 from an overcapped3 request =>earned6, bonus.06; spend1+2+3 leavesPower0 but bonus.06. Subsequent accepted+6 raises lifetime earned12, not6. This follows the native positive stack-delta cue and separates source energy count from ordinary per-hit ChargeAdd.

Enhanced Remora integration: use12s status for Charge eligibility/other consumers, so Chaos's separately specified timed Remora explosion sees12s but keeps its own source formula. The+.10 CR applies only on that Remora target and only to crit-eligible packets; raw resource awards do not change. Shared Stain's own actor-CI factor and equipped Street Boxer triggers remain separate, no double innate crit from the enhancement. Native `GuangXiangFive_Damage` is not an automatically admitted new Remora explosion; no producer is described/bound in the selected current kit. Life skills affect city deliveries/Fons only, not Express Delivery Power or combat time.

### Executable B2 Remora dependency

`hathor.b2_remora_v1` now binds Delay Warning through the shared qualified-switch
Remora service. A living party Hathor with breakthrough2 selects12s and target
CR+.10 at each real application. One refreshed status stores those consumed
values; it keeps its generation after source exit/death, while a new application
after source defeat replaces it with the ordinary5s/zero-bonus policy. The
damage actor's living state and specific recipient are checked at delivery.
Native fixed-crit overrides retain precedence. No orphan damage row is retained
or dispatched and no additional energy callback is introduced.

Independent Rust checks cover5s/12s and strict expiry, refresh, source defeat,
new unenhanced generation, allied elements, fixed crit, and native multi-target
Skill recipients. Blossom's five real pistil emissions at2/4/6/8/10s now all
qualify through an enhanced Remora, granting50 total charge instead of the
baseline20 without duplicating emissions or grants. Street Boxer retains its
own replacement rule; the target-specific Remora bonus joins CR separately.
Two continuing native generations retain real switch costs and seeded
aggregation/replay. This prerequisite does not close the remaining Hathor
Gaze/Power/action/form work.

## Selected full alternate-state specification

Ordinary input states: grounded five-stage chain, held Branch/launch sequence, airborne descent/landing, explicit Critical Dodge/riposte and qualified Support. Branch grants its source Skill/Ult lock before entering; remove on every end/cancel/field exit/death. Launchable scenario target follows Branch/BranchLoop until authored landing, then one BranchEnd impact. Nonlaunchable target still notFloating at source check.52 follows AirAttack1 fallback and its declared landing/cost, never the Branch4.113 plus ordinary plunge. Default stationary boss-like dummy can declare nonlaunchable; a launchable training scenario is equally legal. No spatial AI is required to choose that input flag.

Skill states: `Tap -> Idle` or `.22 hold -> Power(entry/loop) -> ShortEnd/PowerEnd -> Idle`. Base tap+2 and A1 first-hold+6 are independent source events, never both automatically at initial commitment. Full hold chosen>1 uses no tap2. Actor manual timer ceases at end transition, existing target marks continue for their own5s unless consumed/death/combat cleanup. No-field control on release stops actor; a party switch cancels unissued held-End damage and removes on-field actor, not already installed mark generations unless its cause is explicit source death/combat end. A mark may still be consumed by an authored team-targeted attack while Hathor is benched, granting Power/energy to her actual source outside Emergency. Natural mark expiry A2 is likewise off-field.

Burst state: ordinary Burst action runs on its full clock; Emergency begins at1.4999913 and independently grants the one slot replacement/ATK snapshot. During it Skill means granted Cyclone; base Skill CD continues, no cooldown reset or additional charge. Cyclone stages are finite admitted actions with costs1/2/3, each requires actual queued input at the source transition window; a script may stop after an affordable earlier stage and later start a new stage1. Last spend removes permission but current admitted stage finishes under RemoveAbilityOnEnd. State expiry prevents a not-yet-admitted next stage; field exit/death cancels future owner notifies and cleans locks/DEF/permission, retaining only already dispatched nonrecursive impacts. ATK-only cleanup never authorizes a new Cyclone.

There is no off-field motorcycle attack, persistent post-Burst normal form, heal-on-Power gain or repeatable free Burst at zero energy. The only Power-to-energy handler is positive accepted delta*5; ordinary resource/cooldown/gaze state continues across authored cycles. Previews may compute legal predicates without mutating side-effecting conditions, while actual events execute each once. Failed inputs expose the violated state/Power/CD/target/stamina gate.

## Selected full recipient specification

**Mixed:** all ordinary/held/Cyclone/Burst/QTE/riposte/plunge damage ranges use full coefficients on every eligible living stationary dummy in range, stable target-index order, no damage splitting. Event-data collision Burst uses that same declared eligible set; each of its two periodic ticks is one causal packet even with multiple recipients. Gaze/GazeEx/proximity have independent source-target generations/caps; a single marked enemy event grants one source Power request, not one per team target. Different marked enemies may each consume independently, then source Power cap/actual delta handles overflow. Rate/pause/resource awards are once per packet or actual accepted pool delta, never per AoE recipient.

Held growth uses age-selected range eligibility sets; default all-in-range dummies are in every stage. If a scenario declares outsideEarlyRange, that target receives only actually eligible later ticks and its first GazeEx timer starts then, never at cast. Branch target must have declared launchability and falling scenario must provide landing/h and account stamina; critical/parry/enemy attack starts are explicit event inputs. No fictional attack is inferred just because Gaze would benefit from it. Empty visual/control containers are not HP packets. Source/target death skips future invalid packets and removes its marks as specified, never reallocates dead target's stored Gaze reward to an unmarked enemy.

Use [damage/stat zones](../../shared/damage-and-stat-zones.md), [resources](../../shared/energy-and-gauges.md), [cycles/Break](../../shared/cycles-and-break.md) and [equipment/build inputs](../../shared/equipment-builds.md). Lakshana+Cosmos creates Remora, Lakshana+Psyche Stain under qualified-switch admission; ordinary hits do not unconditionally create either status. Enhanced Remora12 and target-conditioned partyCR+.10 are owner dependencies; Charge keeps its own pistil/recipient rule, no automatic5energy per hit from Power merely because a gauge is full. No-bar dummy does not Break; breakable targets use original cap and explicit recovery/Discord inputs, no invented percentage. Direct packet resources, Power grants and Chakra-like Cycle are distinct; HP-shield decisions only matter under incoming/branch scenarios.

Equal-time: expiry/recovery and native state cleanup first; already admitted hit/HP/resources/gauge then kill/proc callbacks. Mark-consuming event removes generation **before** Power mutation; accepted delta then pays5energy and selected A5 count once. Burst cost is paid on commit, state snapshot reads Power at its later native event. Conditional selection never mutates readiness. Release/death wins over a same-time unissued held tick; a final mark expiry's A2 normal-completion callback runs once and an enemy attack at that deadline cannot consume it again. These finite causal guards preserve required mechanics without native-body dispatch recovery.

## Selected complete implementation handoff

**Mixed:** implement all seven selected sections together. Owner state includes Normal/Branch/air/held/Cyclone admissions and clocks, actual Power cap/delta energy callback, Gaze/GazeEx generations/proximity phase, battle-once A1 marker, A5 lifetime earned count, A4 next-stage counter, Emergency deadline/ATK snapshot/ATK-only cleanup, explicit DEF and conditional crit/ignore channels. Keep side-effecting predicates out of readiness prediction. Shared services own damage zones, live targets, real clocks, exact resources/RNG, reaction/Break and replay. No native-body recovery is required to implement these selected rules; preserve their inference labels in CLI/browser. Do not retain the old fixed+.15 ATK or one-Power-per-Cyclone simplifications.

**Legal A0 recipe:** Hathor `character:1025`80/breakthrough6, Normal/Skill/Burst8, QTE8, no awakenings; Raging Flames `weapon:fork_wushoutieyu`80/6/**T1**, Street Boxer `set:Suit5`. Set count is 4; console contributions use `add stats`, with no module geometry or specialization inference. baseATK**1310=644+666**, AtkUp.95, flat420 =>**2974.5 exact staticATK** (the hand forecast below uses display-rounded2975); HP21598, DEF884, compiled CR.49/CD**1.34**, CE1, CI/BI0, staticLakshana bonus.10.  Its incomplete flag omits dynamic handlers specified here.

Keep [Raging Flames](../../weapons/fork_wushoutieyu/passive.md): unconditional **+.15 Lakshana** once (not in this compiler's.10); Burst commitment starts10s world window, E/Q damage bonus+.10 and resets its E stacks. Each actually admitted Redirect/Cyclone **stage input** within window adds one E-only+.05 stack before that stage's damage, cap2, no window extension. This is the selected semantic cast-hook model for the three real Skill inputs even though native stages share an ability handle; do not count every hit/notify as another cast. Stage1E bonus.15 over permanent zone, stages2/3.20; Q gets only.10. Switching preserves timer, expiry clears window/stacks before hits. Keep [Street Boxer](../../sets/street-boxer/passive.md):2pcLakshana+.10 already compiled;4pcbaseCR+.14 compiles once (authored/baseCR.35 ->**.49**). Actual team Remora/Stain generation selects the replacement.14 for20s, cap1/refresh; it replaces the baseline rather than adding another.14, preserving `sets.street_boxer_v2`. Solo Lakshana does not create that conditional buff; its handler stays enabled. Crit expectation is**1+.49*1.34=1.6566**.

**Full-kit hand forecast (display-rounded ATK):** the following cached analytic rotation uses2975/4154 for its damage arithmetic. These analytic totals are not the exact compiled/native fixture numbers; the legal A0 fixtures below use2974.5. Preserve the exact compiled fraction when binding this full schedule.

**Two continuing100s periods:** solo fullHP21598 Hathor, Ultimate0/cap100, Cycle0/cap100, carries0, Power0, no timed effects/CD/marks; target80/HP24million/DEF1080/LakshanaRES.20, no Break bar, declared boss/nonlaunchable (no Branch used), in every held range and continuous proximity. Seed42, no stamina-dependent action. B2/B4/intrinsic proximity are active; solo Lakshana cannot create Remora and target survives, so no fabricated reaction/kill grants. Author one enemy **SkillBegin toward Hathor at45.5 each period**; the attack misses, with no HP contact, Critical Dodge or parry reward. This is an explicit qualifying start event under the recovered Gaze attack-target/player-state condition, not an invented AI schedule or free HP-loss proc.

| Relative time, repeat+100 without reset | Action/state |
| --- | --- |
| 0 | Tap Skill: +2Power at.28333333, direct10energy; its3 hits retain ordinary energy8.499. Recovers1.5779598, CD16.3000733. |
| 2,11 | First two full Normal chains, each8.22461439440877s. Explicit waits respect their whole13-packet trains. |
| 20 | Second tap, Power2->4/direct10energy, full3 hits; recovery21.5779598/CD36.3000733. |
| 22,31 | Third/fourth Normal chains, last recovers39.22461439440877. |
| 40 | Full held Skill: Power entry40.22, actor40.68666664,17ticks40.93666664..44.93666664. First GazeEx expires45.93666664; repeated ticks do not refresh it. Release45 stops actor,10-packet End recovers47.2148167. Ordinary Skill CD56.23665186. No tap+2 from this PowerEnd. |
| 45.5 | Marked enemy SkillBegin consumes GazeEx once; +3requested clips Power4->6, actual delta2 =>direct10energy, overflow1 Power earns nothing. A2 unset, so consumed mark gives no later natural-expiry award. |
| 50 | Burst spends100, gear E/Q window until60, CD65. |
| 51.4999913 | Emergency state begins, nonhybrid8s ->59.4999913; capturesPower6 and+.90*base1310=**1179 liveATK**, so liveATK**4154**, not2975*1.90. |
| 52.4832872,52.8832872,53.8163476 | Two collision hits and landing Burst hit, all with current ATK4154 and gearQ+.10. Recovery54.5305605. |
| 55 | Cyclone stage1, spend1:6->5, seven hits, weaponE stacks1. It is legal despite base Skill CD continuing to56.23665186, because granted Cyclone has its own admission. |
| 56.3430154966667 | Queued stage2 spends2:5->3, three hits, weaponE stacks2. |
| 57.7930229633333 | Queued stage3 spends3:3->0, two hits; already admitted action survives permission removal. Full combo recovers**59.4205379833333**, before Emergency expires59.4999913. ATK snapshot stays4154 despite spending all Power. |
| 60 | Gear window/stacks end. |
| 65 | Fifth ordinary Normal chain, recovers73.22461439440877 with ordinary ATK2975. Explicit wait100. |

Every period emits **113 damaging packets**:65 Normal+6tapSkill+17heldticks+10heldEnd+3Burst+12Cyclone. Natural direct rawenergy**186.739**, plus accepted Power delta6*5=**30 direct selfenergy**, total216.739. At100/200 energy**46+.739 /47+.478**, cumulative overflow**70 /186**, conservation `433.478=200 spent+186 overflow+47.478 retained`. Cycle**311.034** each period =>100+.034 /100+.068, cumulative overflow211 /522. Power is0 through real1+2+3 spending, not an authored reset. HP stays full because the explicit enemy start has no contact.

**Preserve proximity phase:** default unmarked Gaze starts5/15/25/35 and expires10/20/30/40, A2 unset. Held GazeEx starts40.93666664, consumption45.5 resets its proximity accumulator. Ordinary Gaze then applies50.5,60.5,70.5,80.5,90.5, each5s; at boundary100 it is absent with **4.5s proximity accumulated and next application100.5**, which must remain pending into period2. Repeat consumption145.5 similarly leaves next200.5 at boundary200. Reporting does not erase/restart that background timer. Emergency blocks Power generation but does not pause these source-target timers; no attacks on later marks means no extra Power. All damage/held actors/Cyclone/weapon/state/CD effects are finished before period boundaries; this legitimate proximity timer and resource carries continue.

Independent  / `hathor-two-cycle-receipt.json` uses Fraction source-row sums, explicit event order and a separate proximity-generation queue: **226 packets/200s**, full combo admission before expiry, exact energy/Cycle/Power delta and cross-boundary next-mark100.5/200.5. Model arithmetic only, not engine/game verification. Each period expected damage**307195.461926496**, total**614390.923852992**, DPS**3071.95461926496**. Independent aggregate check is:

`2 * [(5*7.628 +2*5.142 +23.730)*2975*1.25 +11.997*4154*1.35 +(5.145*1.40 +(6.855+9.427)*1.45)*4154] *1.6566 *.5*.8`.

Permanent1.25 combines1 +.10set +.15weapon; Q adds.10, E stage1 adds.10+.05 and later stages.10+.10. These are additive bonus zones, not multiplying final damage by several independent percentages. Denominator includes waits, charge,10-hit End, full Burst/Cyclone recoveries and both periods. All alternate/conditional source rules remain implemented requirements; exact native timer/frame/default fidelity can be refined after the fixed campaign ends, without reopening these decisions.

## Executable full-kit acceptance

The selected Hathor owner model is fully executable: ordinary Normal, unheld and
held Skill, intrinsic Gaze/GazeEx, Burst/Emergency, Cyclone spending/DEF/A4/ATK
cleanup, explicit combat leave/reentry, paid Dodge/Critical Riposte/qualified
Support, authored plunge, held Branch with its launchable BranchEnd and its
AirAttack fallback, and the B2 Remora dependency. Model rows are selected by
exact description; strict scenarios name every prerequisite a fixture needs.
No accepted fixture resets resources, buffs, cooldowns or pending deadlines at a
reporting boundary, and every fixture retains native aggregation plus seeded
replay; the AirAttack, Branch, lifecycle and secondary fixtures also retain
bounded JSON/binary WASM transport.

Legal A0 evidence uses `fork_wushoutieyu` R1 T1 with four-piece Street Boxer,
`add stats atk=420 atk%=0.95 hp=5600 cr=0.30 cd=0.60`, a24,000,000-HP dummy
with DEF1080/RES.2 and two continuing100s periods unless a bullet notes a
different fixture:

- [Normal clock](#executable-normal-clock-prerequisite):234 packets,
  energy100+209/500 with301 overflow, Cycle100+391/500 with567 overflow.
- Tap/Power:252 packets, Power6 from12 requested with6 overflow,
  energy100+103/250 with382 overflow, Cycle100+122/125 with652 overflow.
- Gaze:252 packets with real proximity generations and attack starts,
  retaining next100.5/200.5 deadlines beyond each reporting boundary.
- Held Skill:300 packets including34 finite actor ticks and20 End contacts.
- Burst/Cyclone:230 packets, Power16 granted/12 spent/3 overflow/1 current,
  energy57+239/500 with191 overflow, Cycle100+167/250 with538 overflow,
  stamina30 spent/recovered, exact Raging Flames E/Q zones and the Power6
  Emergency snapshot4153.5 ATK.
- [Branch/fallback](#executable-held-branch-prerequisite):14 packets, two
  BranchEnd4.113 landings and two fallback AirAttack3213/2 contacts at h.5,
  exact45.814 Charge/76.126 Cycle, ten10-point Strength spends100 spent and
  fully recovered, four gates and the exact damage ledger.
- [AirAttack](#executable-airattack-prerequisite):28 packets, energy57+101/500,
  Cycle94+499/500 and60 stamina spent/recovered.
- [Combat lifecycle](#executable-battle-lifecycle-prerequisite):112 packets over
  two explicit battle episodes,10 Power granted/5 lost/5 current,
  energy100+439/500 with123 overflow.
- [Support](#executable-grounded-secondary-prerequisite):two QTE contacts at
  633462/1144235 in a separate5s Haniel/Hathor party fixture, Cycle spend100
  and8.2 owner Charge.
- [B2 Remora](#executable-b2-remora-dependency):twelve packets across two
  target-conditioned generations in a separate Hotori/Hathor scenario with
  live crit/Cycle costs.

`hathor_native_branch_fallback_keeps_two_continuing_periods_and_exact_resources`
selects the Branch, AirAttack, Tap/Power, Gaze, Burst, Cyclone, combat lifecycle
and secondary models simultaneously; the held-actor, Support, lifecycle and B2
fixtures add their own compositions. Each checked receipt preserves its source
coefficient/category and live ATK/CR/CE/DEF/RES zones, and every fixture
reconciles the damage ledger, continuing resources and seeded replay. Remaining
source hold/jump/parry/combat-event lowering stays native workflow and shared owner
interactions stay Q3/C-Interactions; these checks certify the selected
simulator model, not measured native animation or observed game damage.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,5,2,3,2 (thirteen hits). The chain loops, retaining native effective talent projection including resonance_3. Stamina and positional movement are not inferred from ordinary input execution.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`hathor-normal-chain.json`](../../../../reviews/q10-game-action-binding/hathor-normal-chain.json).

## Action packets and owner effects

- GA_Hathor_Melee public primary packet is GE_Player_Hathor_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.243/0.416/0.450; category A/LAKSHANA metadata stays source-bound.
- The frozen GE_Player_Hathor_Branch_Damage row has Normal-coordinate rates 2.400/4.113/4.442 at levels 1/8/9, category A, and ChargeAdd 12.0. The frozen GE_Player_Hathor_AirAttack_Damage row has rates 1.250/2.142/2.314, category X, and ChargeAdd 6.3. An explicit GA_Hathor_Melee packet input accepts either exact ID as one packet at the caller-selected impact time and Normal coordinate and leaves the ordinary chain unchanged. These inputs do not use the ordinary Normal/Skill energy rule; positive target-zero HP loss can grant their exact ChargeAdd once under the separate `hathor.branch_airattack_direct_packet_energy_v1` hypothesis. Its source/ally distribution, efficiency, carry, capacity, overflow and delivery-time ownership reuse the tested shared policy; native eligibility remains unresolved. With an explicit target break_profile, the separate shared UnbalValue-to-gauge hypothesis may subtract the packet's native UnbalValue; this does not establish Hathor-specific Break rules. The Branch tips row binds its damage GE but has Name=None; AirAttack is named in its tips row. These mappings do not establish native dispatch.
- GA_Hathor_Skill public primary packet is GE_Player_Hathor_Skill1_Damage, delivering a 3-hit packet train (two GE_Player_Hathor_Skill1_Damage plus one GE_Player_Hathor_Skill11_Damage). Native coefficients at Skill levels 1/8/9 are 1.000/1.714/1.851 per hit; category E/LAKSHANA metadata stays source-bound. The bounded model treats an unheld cast outside Emergency Delivery as granting 2 Express Delivery Power before the packet train. It does not model the held variant or the A1 first-hold grant. During Emergency Delivery, the base Aerial Command input is rejected because the native slot exposes the separate Cyclone Strike action.
- GA_Hathor_Skill2 is the distinct Redirect Skill: Cyclone Strike, which spends Express Delivery Power during Emergency Delivery. It is rejected by the bounded model because its native packet train, exact cost, timing, and A4 crit consumption are unresolved.
- GA_Hathor_UltraSkill public primary packet is GE_Player_Hathor_UltraSkill1_Damage, delivering a 3-hit packet train (two GE_Player_Hathor_UltraSkill1_Damage plus one GE_Player_Hathor_UltraSkill2_Damage). Native coefficients at Burst levels 1/8/9 are 0.962/1.649/1.781 for UltraSkill1 and 5.076/8.699/9.395 for UltraSkill2; category Q/LAKSHANA metadata stays source-bound. The bounded model starts Emergency Delivery for the frozen 5s duration (+15% ATK), retains existing Power, and prevents modeled Skill Power gains for the duration. Switching away ends Emergency Delivery immediately and retains Power.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill without manual overrides.
- Awakening passive effects modeled: A3 grants +15% damage bonus to non-boss targets; A5 grants +1% damage per Express Delivery Power stack up to 15%; A6 grants +30% Crit DMG during Emergency Delivery; resonance_6 grants 10% DEF ignore during Emergency Delivery. A1's first-hold grant, A2's natural-expiry grant, and A4's +25% Crit Rate on the next successful Cyclone Strike are not modeled because their hold, expiry, or Cyclone dispatch transitions are unsupported.
- Canonical arbitrary awakening selections: modeled damage effects use selected identities; the unmodeled A1/A2/A4 action transitions do not grant effects from selection alone. Emergency Delivery DEF ignore and talent levels remain count-based resonance bonuses. Existing unsupported owner mechanics remain pending.

## Configured alternate Skill grant

**Serialized source fact, not a complete held-Skill/Cyclone transition.** The compact effect table (effect_parameters.jsonl), record `Buff_Hathor_AddSkill2`, retains `/4/Properties` `AbilitiesGameplayEffectComponent.GrantAbilityConfigs`: `Ability=GA_Hathor_Skill2`, explicit level parameter **1**, InputID **2**, and removal-policy label **`RemoveAbilityOnEnd`**. This resolves the grant's class and configured input/level/removal fields. Native admission, effective talent mapping, hold/Power prerequisites, replacement, delivery and cleanup remain in [unknown](#unknown-execution-policies); a class name alone does not establish a Cyclone packet or its cost.

## Configured Gaze completion and Power sources

**Source findings for client `PC_140`, logic `566459`; native callback delivery and frozen-cutoff applicability remain open.** The initial acquisition contains the following `AdditionalEffectsGameplayEffectComponent.OnCompleteNormal` lists, now retained at `/0/Properties` in the compact effect table:

| Carrier | Ordered completion request | Exact export identity |
| --- | --- | --- |
| `Buff_HathorAnger_Gaze` | `Buff_HathorAngerStack_GazeEndAdd` | `asset:8012a30265d450c9291240d63707f3146f922fc8329b8a7d9d603b4175273830` |
| `Buff_HathorAnger_GazeEx` | `Buff_HathorAngerStack_GazeExEndAdd` | `asset:d7f5d4fd2edd55928677c800fffd20670d561557e40f17422d30deafe7c79961` |

Both carriers serialize duration scalar **1** against their respective `DT_HathorEffectFigure` Gaze/GazeEx duration rows, each constant **5**; they aggregate by source with limit **1**, `NeverRefresh` and `NeverReset`. The CDO's `RoutineExpirationEffectClasses` repeats the same request. That duplicated configuration does not establish two callbacks or two grants. Their `SKILL_BEGIN` requests use `UseSourceObject=true`; GazeEx also has a separate `SKILL_AFTER_HIT` path. Do not equate all removal, hit and normal-completion paths.

The end-add carriers bind `Con_Hathor_GazeEndAddAnger` and `Con_Hathor_GazeExEndAddAnger` under `/2/Properties/ApplicationRequirements` (`asset:dbd70c352077111434e1bde4ea286c3d88b37d56ce0e6ef459a5db1e79e48ce1`, `asset:356c060ca49e5e74fe06f8cd405dbe1f7d2454357434075699e2739fdb387613`). Their `/2/ScriptBytecode` casts **InstigatorActor**, tests its `Buff_Hathor025_Level2`, then directly requests `Buff_HathorAngerStack` on that actor. Frozen `1025AwakenEffect` binds **Effect2** to this Level2 carrier and describes natural-expiry Power; this mapping is checked independently of the asset suffix.

- Base end-add graph (`asset:4bd38695538ab4751f5cc304447285ffbbb3d55ad30a489e59f576b4b0d872b8`): carrier test at **79**, activation at **167**, explicit true assignment at **309**. The literal `BufferData` has level **1** and stack-count argument **0**. Level 1 is not proof of one granted Power; the native zero-stack argument convention is unresolved.
- GazeEx end-add graph (`asset:5da1ed03c172db477712bc8d88df4e8bd1ac619e0fc269b590f48309a083d71d`): carrier test **79**, evaluate `Hathor_GazeExAddAnger` at coordinate **0** at **167**, convert and **FTrunc** at **303**, assign `nStackCount` at **563**, activate at **719**. The frozen constant row is **3**, giving a configured request for three stacks. Curve-result failure and failed cast/carrier paths reach the return without an explicit false assignment. These are side-effecting requirement graphs, not pure predicates to evaluate repeatedly.

`Buff_HathorAngerStack` (`asset:67507b80a991ea9e6e9699bc0bbd819db483fd7af5bcc4928ad6f7a6a95432d5`) serializes `/2/Properties` **Infinite**, `AggregateByTarget`, **StackLimitCount=6**; its scalar duration field **10** does not define a timer under Infinite. `/3/Properties/ApplicationRequirements` also requires `Con_Hathor_AngerStack` and `Con_IsServerOrStandalone`. The former (`asset:7212c0390961b70a34d2e44775f8675253d572fc8bf8cafd21279a29ffdcf685`, `/2/ScriptBytecode`) casts **TargetActor**, tests `Buff_HathorAngerActive` at **108**, and returns its negation at **189**. The serialized limit and no-Emergency predicate are located; native overflow, mutation and final admission are not established by those fields.

For A1, frozen **Effect1** binds `Buff_Hathor025_Level1` and states six Power on the first held Skill in battle. `Buff_HathorAngerStack_SkillAddOnce` (`asset:88b5838b9c15c68c6c96ab2e410ec40ba3026ce904d26d5cd7fcdd9da26b1646`, `/1/Properties/BuffEventEffectData/0`) requests **six** Power stacks and the once marker at BEGIN, gated by `Con_Hathor_SkillAddAngerOnce`. That graph (`asset:0ef2c452a8b725aa2374bbb1a8974b752695ea19ec8a38b0c10a2fdd6da5ae9d`, `/2/ScriptBytecode`) checks **InstigatorActor**: `not IsInPeace`, Level1 present and marker absent, returning the conjunction at **403**. The marker (`asset:10a7a11e8ec79bfe13f420ccb361834cb8f2e86c6f7e971f30661780bcce4fdc`, `/1/Properties`) is **Infinite** and requests self-removal on `BUFF_LEAVE_BATTLE`. Its reset request is located; the owner Skill.6 caller is now located below; native battle-event delivery and activation ordering remain open.

Exact graphs and the 164-export, hash-checked inspection inventory remain in ; none of the selected exports has recorded parser warnings. This does not certify complete defaults or native execution. Only the two completion arrays above add new compact data; graph bodies remain cache-only. The existing executable Power policy is unchanged.

## Ordinary-hit Ultimate Energy

The `hathor.ordinary_hit_energy_v1` policy uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 22.301, Skill 8.499 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. The direct Branch/AirAttack input has its own hypothesis and is excluded from those ordinary totals. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`linko-skia-hathor-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/linko-skia-hathor-ordinary-hits.json).

Direct-input energy receipt: [`hathor-direct-packet-energy.json`](../../../../reviews/q10-energy-ledger/hathor-direct-packet-energy.json).

### Zero-start continuing rotation

The checked A0 slice uses four five-input Normal chains, two unheld Skills, one 100-energy Burst, then a fifth Normal chain after the spend. All 74 damage packets are checked in order; the 71 ordinary Normal/Skill packets use the existing energy hypothesis. With 1000 ATK, zero bonuses, and a 0.5 defense multiplier, the independent total is 31,110.275 (five Normal chains 19,070; two Skills 5,142; Burst 6,898.275). Burst damage samples the modeled Emergency Delivery +15% ATK; this field expires before the continuation. ChargeAdd reaches 106.202 before Burst: 106 whole energy is generated, 6 overflows at capacity 100, and the Burst spends 100. The continuation ends at 22 energy with 503/1000 carry; each other living recipient ends at 77 with 509/5000 carry.

Inputs use the tested 800ms Normal recovery, 1.2s Skill recovery/6s cooldown, and 2s Burst recovery. The 36s NTE rotation, Rust request, source/ally packet ledgers, observations, aggregation/replay and native/WASM/Worker output are checked together. This establishes only the named ordinary chain/unheld Skill/Burst sequence and existing energy, packet, and Emergency Delivery hypotheses; natural alternate Skill/held modes, native eligibility, packet timing, readiness waiting and other kit gaps remain open. Receipt: [`hathor-continuing-energy.json`](../../../../reviews/q10-energy-ledger/hathor-continuing-energy.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`hathor.skill_animation_profile_v1`, `hathor.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Hathor_Skill1_Short`), `AnimNotify_TriggerCooldown` is at 0.300s, gameplay events land at 0.283s (`Skill.5`), 0.516s & 0.700s (`Skill.1`), and 0.916s (`Skill.7`); cancel windows activate at 1.058s (`BreakSkill_Jump`), 1.433s (`BreakSkill_AnySkill`), and 1.733s (`BreakSkill_Move`) with total duration 6.283s. For Burst (`Hathor_UltraSkill`), gameplay events land at 1.500s (`UltraSkill.3`) and 3.816s (`UltraSkill.2`); cancel windows activate at 4.531s (`BreakSkill_Move`, `BreakSkill_AnySkill`, `BreakSkill_Jump`) with total duration 9.500s.
- **Multi-target delivery (`hathor.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Hathor's Burst and Skill packet deliveries (Skill: 2x `GE_Player_Hathor_Skill1_Damage` plus 1x `GE_Player_Hathor_Skill11_Damage`; Burst: 2x `GE_Player_Hathor_UltraSkill1_Damage` plus 1x `GE_Player_Hathor_UltraSkill2_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/hathor-action-binding-model.json`](../../../../reviews/q10-game-action-binding/hathor-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/hathor_actions.rs), [`hathor-normal-chain.json`](../../../../reviews/q10-game-action-binding/hathor-normal-chain.json), [`hathor-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/hathor-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_hathor.json`, `data/extracted/neverness_hathor.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/hathor.rs) (original `DT_SkillDamageData` export is cache-only), `1025AwakenEffect` (cache-only awakening source), [`hathor/parameters.rs`](../../../../crates/content/src/characters/hathor/parameters.rs) (consumed typed parameters; original `DT_HathorEffectFigure` evidence is cache-only).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

- The executable owner binds the [ordinary clock](#executable-normal-clock-prerequisite), Tap/Power, Gaze, held Skill/A1, Burst/Emergency/A3/A5/A6/R6, Cyclone/DEF/A4/ATK cleanup, combat exit/reentry, paid Dodge/Riposte/Support, authored plunge, held Branch/fallback and B2 Remora. The direct Branch/AirAttack packet API remains a narrower caller-timed input and does not by itself establish natural dispatch; the natural routes are bound by `hathor.branch_v1`/`hathor.airattack_v1` as disclosed causal models.
- Source hold/jump/parry/combat-event lowering and measured native animation/timing remain native workflow; shared owner interactions remain Q3/C-Interactions. The [configured completion/Power sources](#configured-gaze-completion-and-power-sources) locate A1/A2 requests and a six-stack carrier limit; the current model does not enforce that as a verified general native cap.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/hathor). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Hathor (`GA_Hathor_*`):

| Action / damage row (`GE_Player_Hathor_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_Damage` .. `Melee5_1_Damage` | A | 0.133–1.099 / 0.228–1.883 / 0.246–2.034 | 0.664–5.505 / 1.105–9.159 / 0.134–1.111 per hit |
| Branch `Branch_Damage` | A | 2.400 / 4.113 / 4.442 | 12.000 / 20.000 / 2.400 |
| Plunge `AirAttack_Damage` | X | 1.250 / 2.142 / 2.314 | 6.300 / 10.400 / 1.300 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 0.680 / 1.165 / 1.259 | 1.717 / 2.851 / 0.859 |
| Critical Riposte `PerfectEvadeAttack1_Damage` | Z | 1.300 / 2.228 / 2.406 | 3.283 / 5.449 / 1.641 |
| Support `QTE1_Damage` | H | 0.639 / 1.095 / 1.183 | 2.620 / 0.000 / 0.799 |
| Support `QTE2_Damage` | H | 1.361 / 2.333 / 2.519 | 5.580 / 0.000 / 1.701 |
| Skill unheld `Skill1_Damage` (x2) | E | 1.000 / 1.714 / 1.851 | 2.833 / 4.733 / 0.567 |
| Skill unheld `Skill11_Damage` (x1) | E | 1.000 / 1.714 / 1.851 | 2.833 / 4.733 / 0.567 |
| Skill variants `Skill1Power`, `Skill2`..`Skill4` | E | 0.102–1.225 / 0.175–2.099 / 0.189–2.267 | 0.250–2.995 / 0.416–4.995 / 0.050–0.596 per hit |
| Cyclone Strike / Skill2 `Skill5`..`Skill10_Damage` | E | 0.429–3.531 / 0.735–6.052 / 0.794–6.536 | 1.271–5.585 / 2.129–9.308 / 0.257–1.091 per hit |
| Burst `UltraSkill1_Damage` (x2) | Q | 0.962 / 1.649 / 1.781 | 0.000 / 0.000 / 0.206 |
| Burst `UltraSkill2_Damage` (x1) | Q | 5.076 / 8.699 / 9.395 | 0.000 / 0.000 / 1.088 |
| Special / Reaction `GuangXiangFive_Damage` | P | 1.500 / 1.500 / 1.500 | 0.000 / 0.000 / 0.000 |

Critical Riposte uses GAName `GA_Hathor_PerfectEvade`, Support QTE uses `GA_Hathor_QTE`. All damage rows deal LAKSHANA damage. Skill delivers 2x `Skill1_Damage` plus 1x `Skill11_Damage`. Burst delivers 2x `UltraSkill1_Damage` plus 1x `UltraSkill2_Damage` and activates Emergency Delivery (+15% ATK, 5s). Cyclone Strike (`GA_Hathor_Skill2`) delivers multi-hit sequences (`Skill5` through `Skill10`) spending Express Delivery Power.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Hathor_Passive_1` binds `Buff_Hathor025_Passive1`. `DT_ReactionExtendDataTable` row 1025 binds `ReactionAbilityID=GA_Hathor_Passive_1` with `Buff_Reaction_3_new_1025` (Lakshana / Reaction 3 enhancement).
- **Passive 2 (Breakthrough 4):** `GA_Hathor_Passive_2` binds `Buff_Hathor025_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1025AwakenEffect#/rows/1025/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Holding the Redirect Skill button for the first time in battle grants 6 stacks of Express Delivery Power (`Buff_HathorAngerStack_SkillAddOnce`).
- **A2:** Grants Hathor Express Delivery Power when Excellent Rating Lock and Five-Star Tracking naturally expire (`Buff_HathorAngerStack_GazeEndAdd`, `Buff_HathorAngerStack_GazeExEndAdd`).
- **A3:** Hathor deals 15% increased damage to non-Boss targets.
- **A4:** Increases the Crit Chance of the next Cyclone Strike by 25% each time Cyclone Strike is successfully cast while in Emergency Delivery state. Cleared when state ends.
- **A5:** Increases damage against targets by 1% for every stack of Express Delivery Power gained (up to 15%); removed on combat exit.
- **A6:** Increases Crit DMG by 30% while in Emergency Delivery state.
- **Resonance 3:** Increases the skill level of Rapid Delivery (Normal), Aerial Command (Skill), and Rider Express (Burst) by +1.
- **Resonance 6:** Ignores 10% of target DEF (`TargetDefIgnore = 0.10`) while in Emergency Delivery state.

### Recovered Power predicates and stamina boundaries, October 18

The initial bundle's `JumpSection_Con_Hathor_Skill2_2_Check:CanApplyEffect` (`#/1/ScriptBytecode`, statements 121/203/277/311) reads the mesh owner's ASC and accepts **`Buff_HathorAngerStack` count >= 2 AND `Buff_HathorAngerActive` present**. `JumpSection_Con_Hathor_Skill2_3_Check` uses threshold **3**. The later caller pass locates them in the saved **Hathor_Skill2_1/2** notify objects (`#/18`); those animation exports retain unrelated notify-read errors. Native jump priority, consumption and ordering against the active carrier remain necessary for a complete Cyclone sequence. Source:  and `hathor-condition-callers.txt`.

`Calc_Hathor_AngerActiveAtkUp:CustomCalculateBaseMagnitude` (`#/2`, statements 0..379) evaluates `DT_HathorEffectFigure:Buff_HathorAngerActive_AtkUp` at coordinate zero and multiplies it by `max(owner ASC Buff_HathorAngerStack count, 0)` on successful curve lookup. The expansion's `Calc_Hathor_AddQEnergy` instead reads the **instigator's** `Buff_Hathor025_AddQEnergy_CheckStack` and multiplies by `Buff_Hathor025_AddQEnergy` at coordinate zero. Neither helper consumes stacks. Do not conflate their owners or invent read/consume ordering; failed paths are not inferred grants.

`JumpSection_Con_NoStrength:CanJumpSection` (`#/1`, statements 214/293) compares player-state Strength **<= 10**, with explicit true when player state is invalid. `GA_Hathor_Melee` references it from its two AirAttack-end checks (`#/18`, `#/19`). Separately, its `K2_CanActivateAbility` (`#/7`, statements 234..537) returns `NOT(IsFalling AND Strength <= 0)` on valid player state; invalid player state uses `IsSummonPlayerCharacter`. These are distinct stop/admission predicates, not a universal ten-point action cost. The selected [AirAttack model](#executable-airattack-prerequisite) binds cost/recovery and parent admission as disclosed policies; measured native timing remains native workflow. The cited objects have no decode warnings.

### A5 accumulated damage-bonus magnitude recovered, October 18

`Buff_Hathor_DmgUpLevel5#/1/Properties/CustomApplicationModifierInfos` binds additive **DamageUpGeneralBase** to `Calc_Hathor_DmgUpLevel5`. Its warning-free function (`#/2`, statements 0..406) returns `max(instigator count of Buff_Hathor_DmgUpLevel5_CheckStack,0) * Buff_Hathor_DmgUpLevel5`, with the figure row constant **0.01** at coordinate 0. The count carrier (`#/1`) is Infinite, AggregateByTarget, **StackLimitCount=15**; `/2` requires `Con_IsServerOrStandalone`. Supplied counts 0/10/15 yield 0/0.10/0.15 additive damage bonus. The calculator has no separate clamp or stack mutation, so a hypothetical count 16 returns 0.16 even though the configured count carrier caps at 15. Native per-hit additions, reset/death/switch handling and modifier capture remain open; serialization does not itself prove all 15 stacks arrive in a rotation.

### Passive description coverage, reviewed October 18

Saved description source: `guide_hathor.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Delay Warning describes Remora duration extended to 12s and Crit Rate +10% against Remora targets. Efficiency Boost describes one Express Delivery Power stack after defeating a target. Native reaction refresh, target-condition evaluation, kill attribution, off-field generation and Power ledger are separate from the already located hold selectors and A5 magnitude.

### Kill-to-Power request and state gate recovered, October 18

`Buff_Hathor025_Passive2#/1/Properties/BuffEventEffectData/0` requests `Buff_HathorAngerStack` on **KILL**, BySelf=true, TargetTrigger=false, with StackCount=-1. The count carrier (`#/2`) is Infinite, AggregateByTarget and capped at **6**; its BEGIN/FINISH request addition/removal of `Buff_Hathor_AddSkill2`. The serialized magnitude 10 does not make an Infinite carrier expire after ten seconds, and -1 does not by itself establish one settled stack.

Its `/3` requires `Con_Hathor_AngerStack` and `Con_IsServerOrStandalone`. The former's warning-free function (`#/2`) casts TargetActor to Character and returns **NOT Has(Buff_HathorAngerActive)** on that valid path. This is a located Emergency Delivery exclusion on the requested count carrier; a generic kill callback cannot automatically add Power during that state. Native count grant convention, cap overflow, kill attribution and delivery/reset ordering remain open.

### Emergency Delivery duration mode split recovered, October 18

Initial `Buff_HathorAngerActive` and `Buff_HathorAngerActive_AtkUp`, both `#/1/Properties/DurationMagnitude`, select `Calc_Hathor_AngerActiveDuration`. Its warning-free `#/2` function asks **IsHybridMode(OwnerActor)**: true evaluates `Buff_HathorAngerActive_Duration=5` at coordinate 0; false explicitly returns **8**. These are distinct configured mode paths, not interchangeable five/eight-second measurements. Actual mode selection, clock and pending-state removal remain with the native lifecycle; no mode is selected for the simulator by this research.

### Cyclone spending and held-Normal lock requests recovered, October 18

Initial warning-free **GA_Hathor_Skill2:K2_CanActivateAbility** (`#/5`) checks the owning player's ASC for **Buff_HathorAngerStack**. Its activation (`#/4` → `/3`, 572..627) calls the montage parent, then on server/standalone requests removal of **one stack**, with the owning ASC as source filter. Its **TriggerJumpSectionImpl** (`#/6` → `/3`, 548..567) calls the parent before the server/standalone branch requesting removal of **two stacks for Skill2_2** or **three for Skill2_3**, with null source filter for those later requests. Thus the source spending requests are 1 at activation and separate 2/3 at the named transitions; they are not each a one-Power cost. The threshold checks above are still evaluated against the native ledger, and reaching every transition is not established by summing these requests. Actual removal, refunds/interruption, source aggregation and callback order remain open. 
**GA_Hathor_Melee#/9** → `/4`, 299..615 requests **Buff_CantUseSkillAndUltra** before the parent jump only for exact section **Branch**. **K2_OnEndAbility** (`#/8` → `/4`, 275..294 → 124..270) requests removal of all stacks after the parent end callback, without branching on cancellation. The carrier's `/2` is Infinite, target-aggregated, limit one, granting **State.Property.CantUseSkill** and **State.Property.CantUseUltSkill**. This locates the held-Normal lock and end-removal declaration. Native tag enforcement, child section exits, delivered cleanup and the earliest legal next Skill/Burst input are still missing; it does not establish a fixed cancel timestamp.

### Held-Skill timer, growth stages and end callback recovered, October 18

Initial **DamageActor_HathorSkill1Power#/5 → ExecuteUbergraph#/2**, 1209..1318, validates its owner player and registers **AsyncTaskSetTimer(self,null,0.25,true,false,-1,true)**, binding its timer callback and the player's **OnAbilityEndEvent**. The timer callback (`#/4` → `/2`, 942 → 15) asks whether the player's **current montage contains a section named Power or PowerLoop**. It does not ask whether the current section is one of those names. If neither exists it requests actor destruction at 927. Crucially, both branches then pop to the previously pushed **225** continuation: the false branch does not simply return after destruction. The continuation selects **DamageRange_Hathor_Skill1Power1** at GetSpanLifeTimeSinceCreation(true) **<1.5**, **...Power2** at **1.5≤age<3**, or **...Power3** at **age≥3**. Exact boundary samples 1.5 and 3 therefore select stages 2 and 3; native destruction/application ordering determines whether the last request can settle.

Each reached callback obtains the owner's **GA_Hathor_Skill** level by CharacterItemNetID, constructs/applies a container using the selected range and CDO's **GE_Player_Hathor_Skill1Power_Damage, Buff_HathorAnger_GazeEx and Buff_HathorAnger_Gaze_Menu**. These locate the held-action requests feeding Gaze rather than relying only on the buff's completion events. The AbilityEnd callback (`#/3` → `/2`, 1366..1447) destroys the actor only when the ended ability's exact class equals **GA_Hathor_Skill**; ReceiveDestroyed ends the timer task. Native task clock/first callback, age scaling, per-target application, Gaze acceptance and pending-hit cleanup remain open. The named CDO fields are reached and functions decode completely; the actor CDO retains an inherited-field read warning. Source:  and `field-source-config.txt`.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Hathor_Skill#/10; Skill.6 | PlayerHathorSkill1Power | (0.0, 0.0, 0.5, 5.0, 0, true) | no explicit damage class |

Skill.6 binds PlayerHathorSkill1Power with Period=0.5, duration=5, MaxTriggerCount=0 and an empty target/GE payload. This does not add another 0.5-period damage train to the separately recovered held-Skill actor's manual 0.25 timer. The subclass callback and native actor scheduler are different source routes.

### Gaze consumption and Anger-generation predicates recovered, October 18

The warning-free shared **Condition/Player/Hathor** graphs are **side-effecting application requirements**, not merely names for native resource calls. **Con_Hathor_GazeAddAnger#/2**, bound by **Buff_HathorAngerStack_GazeAdd#/2**, requires TargetActor and InstigatorActor ability-character casts, a valid **TargetActor.GetAttackTarget()**, and equality of that attack target's **GetHTPlayerState()** with the Instigator's player state. On this path it requests removal of **all Buff_HathorAnger_Gaze** from TargetActor, then **Instigator.K2_ActivateGameEffectsWithClass(self,Buff_HathorAngerStack,BufferData(...))**, and returns true. It does not compare the two character objects directly or query current damage instigator.

**Con_Hathor_GazeExAddAnger#/2** has the analogous attack-target/player-state gate, removes **GazeEx**, then passes **nStackCount=trunc(Hathor_GazeExAddAnger)=3** to the AngerStack request after a successful coordinate-zero curve lookup. **Con_Hathor_GazeExHitAddAnger#/2** instead checks **TargetActor ASC.LastDamageInstigator** for validity and equality of its player state with Instigator's; it removes GazeEx and requests the same three-stack input. These are distinct attack-initiation and last-damage source routes, not a single guarantee of Anger on every team hit. Failed casts/lookups leave their unassigned return paths; native context, request success, removal/completion ordering and concurrent callbacks still matter.

The already recorded **Gaze/GazeEx#/2** event entries actually name these Add/Hit requirement carriers: GazeAdd and GazeExAdd at **SKILL_BEGIN**, GazeExHitAdd at **SKILL_AFTER_HIT**, with **UseSourceObject=true**. GazeEx BEGIN removes Gaze and Gaze_Stack; Gaze BEGIN removes Gaze_Stack. Their separate A2 EndAdd/natural-completion chain above must not be merged with these successful attack/hit consumption paths. Native callback classification/deduplication still decides whether the requested Gaze removal invokes a completion reward.

The destination's already recorded no-AngerActive gate and cap apply independently: a located generation request is not proof that stacks increase during Emergency Delivery. **Con_Hathor_AddSkill2#/2** instead requires **both AngerActive and AngerStack** on TargetActor and gates the [configured Skill2 grant](#configured-alternate-skill-grant). AngerStack and AngerActive each request this grant carrier on BEGIN and remove it on FINISH; AngerActive additionally requests self-removal on CHANGE_ROLE_OUT. Native stack arithmetic, grant arbitration, effective level, in-progress Skill2 cleanup and resource clocks remain open.

### First-Skill, Riposte and Skill2 conditional consumers recovered, October 18

The upstream A1 request is now located: **GA_Hathor_Skill#/9** requests the already documented **SkillAddOnce** carrier on owner at **Skill.6**, while **Skill.5** requests **Skill1ShortAdd**, whose BEGIN requests two Anger stacks. These are distinct source event/payload routes, not proof that every Skill input gives six stacks or of the native peace/battle classification and successful settlement. Source pointers are `EffectContainerMap/5` and `/4`, respectively.

Both **PerfectEvadeAttack_Damage** and **PerfectEvadeAttack1_Damage**, **#/2**, bind **Con_Hathor_EvadeAtkAddAnger#/2**. It requests **AngerStack_EvadeAtkAdd** on a successful Instigator character cast and explicitly returns true even on its failed-cast path. The intermediate carrier (`#/1`) has duration input **5**, target-cap1, NeverRefresh/NeverReset and a BEGIN AngerStack request. This locates a requested Riposte reward and exclusion carrier; it is not evidence that querying the damage predicate always delivers another reward or that every proc settles before a subsequent hit.

The independently recovered **Con_Hathor_AngerActive_AtkUp_Del#/2**, bound at the like-named removal carrier's **#/2**, returns true without directly removing ATK only when Instigator's current montage resolves to a valid ability whose **class display name exactly equals GA_Hathor_Skill2_C**. Otherwise, after a successful character cast, it requests removal of all **AngerActive_AtkUp** and returns false. AngerActive FINISH requests this helper; the admitted helper's **#/1** has HasDuration **2** and requests ATK removal on **FINISH**, and separately on **MONTAGE_SKILL_FINISH** with **CheckTriggerObjectName=GA_Hathor_Skill2**. This locates conditional retention and two subsequent cleanup requests, not an unconditional lingering ATK buff or a proved two-second real-time extension. Native montage-instance lookup, event delivery and clock remain open.

**Skill2CritUp#/1** adds **CritBase=0.25** under **Limit_Hathor_Skill2CritUp#/1**, which checks the gameplay-effect **class's asset tags** for **Ability.Player.Hathor.Skill2**. Carrier admission separately requires Instigator **Level4**, despite the helper's Skill2 name; **GA_Hathor_Skill2#/7/EffectContainerMap/6**, **Skill.1 → UseOwner**, supplies its actual request. The carrier's target cap **10** does not establish ten admitted bonus stacks. AngerActive's two conditional modifiers separately add **CritDamageBase=0.30** with Instigator Level6 and **DefIgnore=0.10** with Instigator Level6_extra; these are different stat zones and markers.

The separate Normal Branch check (`GA_Hathor_Melee#/20 → JumpSection_Con_Hathor_Branch_Check#/1`) returns **NOT target.IsInFloatingEffect(0)** for a valid ability target, explicit true on failed target cast. This differs from the already recorded Skill2 stage-count checks and does not prove successful native jump priority, final spending or real cancel windows.

All cited effect/condition exports are warning-free; animation warnings are scoped above. Source projections: , `hathor-gaze-end-functions.txt`, `hathor-condition-callers.txt`, `hathor-gaze-carriers.txt`, `hathor-gaze-consumer-carriers.txt`, `hathor-resource-producer-callers.txt` and `ninth-trigger-config.txt`; the scalar rows are in **DT_HathorEffectFigure**. The full roster/shared function catalog, rather than the earlier name-filtered survey, now supplies the remaining candidate list.

### Anger stack listeners and Gaze cleanup recovered, October 18

The warning-free repaired **GC_Hathor_BasePas** is joined from **Buff_Hathor_BasePas#/1** by `GameplayCue.Display.Hathor.BasePas`. **OnWhileActiveEvent#/4 → graph#/1:1180** requires MyTarget to be a player and **IsServerOrStandalone**, caches that player and subscribes to **Buff_HathorAngerStack** stack changes. Callback **#/2 → 165** computes **delta = NewStackCount - OldStackCount** and acts only for delta > 0. It first requests **Buff_Hathor025_AddQEnergy_CheckStack** on the cached player with nStackCount=delta, then **Buff_Hathor025_AddQEnergy** on the same player with default BufferData (level1/stack0/strength1). No initial stack synchronization is present. OnRemove validates and ends its saved task.

The CheckStack carrier (`#/2`) has **HasDuration 1**, target cap **6**, OnlyServer and **BuffAddToAddQueue=false**. The energy carrier (`#/2`) names **ChargeExecution**, with Source/snapshot additive **PParameter1** from the previously recovered **Calc_Hathor_AddQEnergy**: instigator CheckStack count × **DT_HathorEffectFigure:Buff_Hathor025_AddQEnergy=5**. Thus a captured count of 2 supplies input **10**. The callback does not explicitly clear the earlier checker, so two closely spaced positive changes cannot be assumed to grant exactly five Charge per newly added Anger stack: native stacking/refresh, capture/context and successful Charge settlement remain necessary. Omitted DurationPolicy on the energy carrier is not inferred from its duration-magnitude field.

**GC_Hathor_Level5**, joined from **Buff_Hathor025_Level5#/1**, likewise subscribes on a server/standalone MyTarget player. Its positive-delta callback additionally requires that cached player currently have **Buff_Hathor_DmgUpLevel5**, then requests **DmgUpLevel5_CheckStack** with nStackCount=delta. This supplies the missing producer for the already recovered **max(count,0) × 0.01** general-damage modifier and target cap15. It tracks positive Anger changes while that marker exists; the graph has no separate per-hit gate or initial synchronization. OnRemove calls the saved task's EndTask without a validity guard. Native stack settlement and re-entry remain open.

**GC_Hathor_GazeMenu**, joined from **Buff_HathorAnger_Gaze_Menu#/1**, subscribes to Gaze_Stack changes on MyTarget's ASC. NewStackCount <= 0 schedules **Delay 0.2**, then removes Gaze_Menu only if both Gaze and GazeEx are absent at that later check. Its separate death delegate is bound through **EffectContext instigator player's player state**. A death callback whose character is a player with **MainCharacterID=1025** requests removal of all Gaze, Gaze_Stack, GazeEx and Gaze_Menu on the cached target; no equality check to the original instigator is present. OnRemove ends a valid stack task, without an explicit death-delegate unsubscribe in the reached body. These are cleanup requests, not a recovered Power refund or clock rule.

Sources: `hathor-anger-energy-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-carrier-joins.json`. The accepted Halo repair supplies the matching configured tags without the earlier inherited CDO warnings.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Hathor_Skill1_Short`) and Burst (`Hathor_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Alternate-Skill grant & Cyclone Strike:** The [alternate-Skill grant](#configured-alternate-skill-grant) now has source-bound class/input/level/removal fields. The source gate now requires AngerActive plus AngerStack; resolve final admission/effective talent mapping, replacement with an active Skill, and removal/expiry ordering.
- **Gaze completion & Power source chain:** The [Gaze completion and Power source chain](#configured-gaze-completion-and-power-sources) now locates ordered completion requests, A1's six-stack request/predicate/leave-battle marker, A2's instigator/Level2 gates, GazeEx's truncated three-stack argument and the six-stack Power carrier limit. The [held-action caller](#held-skill-timer-growth-stages-and-end-callback-recovered-october-18) is recovered; resolve native battle reset and completion classification/delivery, base end-add's zero-stack argument convention, final Power admission/overflow and ordering against Gaze removal.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The [alternate-Skill grant](#configured-alternate-skill-grant) now has source-bound class/input/level/removal fields. The source gate now requires AngerActive plus AngerStack; resolve final admission/effective talent mapping, replacement with an active Skill, and removal/expiry ordering. It does not by itself identify a natural Cyclone transition.
- Resolve action costs/stamina, interruption/cancel windows, packet timing, and any target cap/selection required by the declared all-in-range scenario.
- Natural Branch/fallback dispatch, costs, landing precedence and packet multiplicity are bound by the selected [held-Branch model](#executable-held-branch-prerequisite). Measured native hold/jump windows and target motion remain native workflow; the bounded direct packet API still establishes none of those transitions by itself.
- The [Gaze completion and Power source chain](#configured-gaze-completion-and-power-sources) now locates ordered completion requests, A1's six-stack request/predicate/leave-battle marker, A2's instigator/Level2 gates, GazeEx's truncated three-stack argument and the six-stack Power carrier limit. The [held-action caller](#held-skill-timer-growth-stages-and-end-callback-recovered-october-18) is recovered; resolve native battle reset and completion classification/delivery, base end-add's zero-stack argument convention, final Power admission/overflow and ordering against Gaze removal. Do not count the CDO and component copies as independent callbacks or repeatedly invoke side-effecting requirement graphs.
- Held Skill / Five-Star Tracking, A1/A2 transitions, Cyclone dispatch/cost/timing/A4 and Power spending are bound by the accepted held-Skill and [Cyclone](#executable-paid-cyclone-prerequisite) models; measured native timing and the serialized Power mutation/cap remain unverified.
- Ordinary-hit energy uses a tested caller/recipient hypothesis, not a native caller trace. Explicit Branch and AirAttack inputs can grant ChargeAdd under the separate `hathor.branch_airattack_direct_packet_energy_v1` scenario hypothesis after positive target-zero HP loss; the natural dispatch path now binds the same policy for held Branch/BranchEnd and fallback AirAttack landings, while native eligibility remains unverified despite their positive frozen ChargeAdd values.

## Prior findings

- [`reviews/q10-game-action-binding/hathor-action-binding-model.json`](../../../../reviews/q10-game-action-binding/hathor-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/hathor_actions.rs), [`hathor-normal-chain.json`](../../../../reviews/q10-game-action-binding/hathor-normal-chain.json), [`hathor-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/hathor-skill-burst-packets.json).

## Research history

- **Accepted:**8/8 selected handoff, build and independent two-cycle audit complete. Native/public/video lead approaches already used; no repeated body/export hunt is needed.
- **Next:** Hotori's first unchecked block and its existing time-stop/resource owner. Preserve Hathor's selected rules for implementation; defer optional native-fidelity polishing.

## Relative combat timer binding

`Hathor_UltraSkill` Common.EndAbilitySkill at 4.518597 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
