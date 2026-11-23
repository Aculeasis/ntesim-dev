# Nanally action and damage model

Native identity: `character:1010`. Frozen baseline: Global2026-10-04.

## Current contract and readiness

The [selected executable specification](../../../../openspec/specs/characters/nanally/spec.md) owns current behavior. This dossier separates consumed coefficients, chosen execution policies and historical acquisition findings. The selected native full kit is accepted: canonical actions, held/plunge/Dodge routes, natural resources, passives and bounded awakenings retain independent owner checks and the joined equipped A0 baseline below. Further work is scoped by the selected OpenSpec change; this reference does not declare a remaining-work queue. Historical investigations below are evidence, not competing executable contracts.

## Joined equipped A0 acceptance

`presets/native/nanally-full-kit.nte` runs two continuing periods with M1
Raging Flames, all main actions, natural Cycle spending, enhanced Buds and
Fair Duel. It reaches 275 hits: 142 Normal packets,12 heavy,2 plunge,4 Aim,
2 Riposte,15 Skill,14 Burst,2 Nanally Support,4 Zero Support,28 Cat,
10 Fair Duel and 40 Bud packets. Cat Skill/Normal/heavy/plunge groups and two
weapon windows/ordinary Skill stacks are reached without a free Burst assist.
The first owned Bud packet at41777110us lands while Zero is active, dealing
9000*(1+0/600)*.5=4500 against the declared DEF1080/resistance0 dummy.

Two Bursts spend200 Ultimate energy. Initial100+252 whole grants reconcile
with final100+200 spent+52 overflow and101/125 carry. Natural Cycle starts0:
382 whole grants reconcile with final11+200 spent+171 overflow and53/250 carry.
Four explicitly authored paid Dashes spend all60 stamina without recovery;
critical success is a declared scenario assumption, not an inferred enemy action.
The second period preserves resources, buffs, cooldowns, pending Buds and RNG.
Native Rust/CLI aggregation, seeded replay and bounded WASM/actual Worker smoke
consume the same source and engine. Independent component checks retain AoE,
awakening, cancellation, exact-boundary, zero-loss and death/generation coverage.

This accepts the selected executable mechanics model. It does not certify
measured animation rates or complete native report/search/browser work. Historical
acquisition unknowns remain evidence limits rather than competing live contracts.

## Selected decisions and rationale

Selected2026-11-01 following owner review and user authorization to choose revisable models without requiring frame-by-frame proof:

- Authority12/15s in combat; Underboss6/9s. E re-entry removes Authority without a finisher or prorated cooldown. Authority grants critical damage rather than another Normal chain. These agree with [Reference guide's skill descriptions](https://neverness.app) and the [Chinese skill card](https://wiki.mysqil.com/characters/nanali).
- Use only the summon lifetime as budget. Reference guide says five Normal plus three heavy stages exhaust the usual A0 window, but does not establish a universal eight-input cap. An [NGA player report republished by GamerSky2026-05-09](https://www.gamersky.com/handbook/202605/2137595.shtml) describes more coordinated stages with overlapping teammate actions/ultimate pauses. That report is not a verified reproduction, but argues against promoting one recommended combo into a hard counter. Remove the unused three-minion field; it has no reached deployment consumer.
- One performed Cat group counts once for A1/A5, regardless of its packet count or recipient count. The published follow-up wording does not settle this unit, so this is the selected attack-group model. Group dispatch requires a living recipient; later integer HP rounding does not undo the performed attack. A3 retains its separate positive-HP-loss callback rule.
- Remove both the initial Cat_AirAttack summon hit and the seven direct-Q-triggered Cat hits. The source summon marker follows direct Burst damage, and published Cat coefficient lists bind that AirAttack row to plunge. Q does not coordinate with an old summon either.
- Normal/Aim/Support/Riposte/Skill/Burst use finite owner clocks with explicit source markers and selected contact/cancel policy. Burst's later summon applies independently from its direct damage. Cat groups retain selected coincident post-parent delivery; no0.3s serialized FIFO is invented. These models are not measured animation rates.

Web sources were reviewed2026-11-01. Agreement among guides is corroboration, not proof of native execution. Published A5 gives2% ATK per follow-up up to20%, removed on exit; its15s refresh and attack-group unit are selected models. New evidence should replace the relevant policy, Rust consumers, independent tests and docs together.

## Executed Authority removal

Live Authority selects source-granted `GA_Nanally_RemoveSkill` through the shared typed utility-invocation boundary. Its2s gate replaces ordinary Skill cooldown admission/readiness without changing field, life, recovery or monotone-time checks. A successful utility ends Authority, preserves the original cooldown and current A5/Underboss, and emits no damage, Cat group, callback energy or ordinary Skill cast proc. The authored input and its nominal packet template retain source identity with explicit utility/Other provenance; no nominal packet is delivered. No finisher, refund or prorating is modeled. Zero utility recovery is a disclosed approximation. Source E now starts its16s cooldown at425571us and Authority at429917us through separate reached notifications. Its first removal gate is2429917us; cancellation cannot undo a notification that already settled.

Rust `nanally_removal.rs` checks two continuing legal E/removal periods0/2/16/18s,10 total E packets and12.6 ordinary raw energy. An attempted ordinary E at20s still rejects against the preserved32s deadline. A request at1.999999s rejects against2s without mutating the accepted prefix;2s succeeds. Field exit restores the original cooldown query; removal preserves A5 so the next Normal1 at ATK1000/DEF1/2 is148*1.02=3774/25, while two Normal groups yield exactly two Cat packets/two A5 stacks. Raging Flames records one stack from the actual E and none from either rejected or accepted utility removal. Native source `nanally-authority-control.nte` checks selected gates, continuing carry and all-field replay; CLI and bounded WASM/Worker consume the same file.

## Executed combat transitions

Nanally consumes the existing explicit native combat-transition ledger. Native `DT_NanallyEffectFigure` rows `Nanally_WalkDur`, `Nanally_Lv6_BattleWalkDur` and `Nanally_Lv6_NoBattleWalkDur` provide12/15/20 respectively; only the consumed duration constants are retained. Initial state is in combat. A6 captures15s there or20s outside combat at Authority application; non-A6 remains12s. The configured duration calculation motivates application-time capture: later leave/re-entry does not refresh, rescale or truncate a current deadline. This is a selected model pending more precise evidence. Combat exit clears A5's modifier and stored stacks, outside-combat follow-ups cannot regrant it, and re-entry creates neither stacks nor resources. Field exit, death and explicit removal retain independent termination rules; Underboss/cooldowns/Cycle are not reset by a combat event.

Rust `nanally_combat.rs` checks three continuing E applications at0/20/45s with leave5/re-entry25: A6 deadlines15/40/60 remain unchanged by intervening transitions. Normal1 at ATK1000/DEF1/2 deals148 before CRIT; its potential critical damage is1332/5 under Authority's additional0.3 versus222 otherwise with base critical bonus0.5. Non-A6 expires after12s even outside combat. A5 callbacks at1.0001/5.5001s each start one stack, while exit3 clears the first and a connected outside-combat group4.0001 cannot regrant it. CLI native aggregation and seeded replay preserve actual combat receipts and continuing input clocks. Source lowering of combat assumptions remains the shared native workflow scenario-integration task.

## Executed Blossom enhancement

More Than Passionate now supplies a concrete native emission profile through the content kit registry. A living breakthrough2 Nanally enhances new party Buds to ten emissions at1s intervals, beginning1s after creation. The existing native extension row's `AdditionalBulletCount=5` and published ten-pistil/one-second description ground the selected count/cadence; `FixFireCountDown=-1` is not independently interpreted as a timer. Living party-provider eligibility at birth and a captured generation schedule are explicit revisable models. Neither Authority nor an awakening is required. Nanally's presence does not take the actual reaction owner's captured level/CI attribution.

Rust `nanally_blossom.rs` checks off-field A0 contribution across two continuing Buds, locked/defeated providers, provider loss between births and oldest-Bud cancellation. With captured owner level80/CI600, native base9000, DEF mitigation1/2 and resistance factor4/5, each Pistil deals7200; ten deal72000 per Bud rather than36000. Death at4.5s preserves the first Bud's1..10s schedule; a later birth at15s without a living provider returns to17/19/21/23/25s. A fourth Bud born at6s replaces the oldest before its equal-time ReactionOrBreak emission under shared admission ordering: five settled hits survive, giving35 emissions across the four births. Jiuyuan's separate two-Bud extension composes with ten emissions per Bud rather than replacing the count/cap owner. Source `nanally-reaction-control.nte` now has21 hits: one Support, five E, ten Pistils and five Fair Duel. Own one-second Cycle hits may request A3 more often than foreign Cycle's two-second Fair Duel bridge, while A3 retains its independent guard.

## Executed Fair Duel and Cycle bridge

Fair Duel now consumes the native `GE_Nanally010_Lv1_Damage` coefficient row in the typed Nanally damage table. Unrelated source fields stay outside Git. The Basic Attack curve includes1.199 at level10, not a rounded1.200. Source category A, NATURE, GA_Nanally_Melee and zero ordinary Charge/Break remain native packet metadata.

The [selected contract](../../../../openspec/specs/characters/nanally/reference.md#fair-duel-and-cycle-callbacks) binds breakthrough4, active Authority, positive settled party Cycle damage and an independent2s guard. Native registration retains the real Authority command plus actual damage receipt through the existing autonomous coordination ledger. It accepts own and foreign Cycle owners without changing Blackbird's owner eligibility. Captured live-recipient fallback, positive-loss gating and simultaneous delivery are revisable execution choices; the source callback's global subscription alone does not prove them.

Rust `nanally_fair_duel.rs` checks two continuing periods with own/foreign Cycle ownership, mixed talent coordinates, guard independence, no ordinary-hit trigger, locked breakthrough, absent Authority, zero HP loss, field exit, overlapping Bud schedules, fatal-hit recipient fallback, own Hexed A1/A5 accounting and native replay. Independent numbers with ATK1000, DEF mitigation1/2 and no other bonuses are514 at Normal8 and1199/2 at Normal10. Five selected A5 Fair Duel stacks produce300,306,312,318,324 damage and five A1 grants total12.5 raw self energy per period. The native source fixture `presets/native/nanally-reaction-control.nte` transports the same passive without authored passive inputs through CLI and bounded WASM/Worker checks.

## Native packet coefficients and proposed additional routes

### Executed Normal and Aim clock

`nanally.action_clock_v1` binds the saved rate-one asset markers in
`characters/nanally/clock/profile.rs`; only consumed native packet/gate/contact
coordinates are retained. Each coordinate rounds upward to a microsecond bound.
The conservative next-ability recovery and separate Jump/movement gates follow
the timing evidence below. Alternate montage rate curves remain research;
they are not simultaneous executable modes. Finite positive-contact pauses
use shared exact actor progress and never a readiness polling loop.

Independent first-five positive-contact input starts are0,611861,1482102,
2693850,4025363us; the next legal input is5337783us. Normal3 recovery adds
130000*(1-.01)=128700us; Normal5 adds(60000+120000)*(1-.01)=178200us.
At ATK1000 and DEF mitigation1/2, five level1 inputs deal4313/2; adding the
sixth Normal1 gives4609/2. Two such continuing six-input periods deal4609
and grant35 energy with237/500 carry from2*17.737 ordinary raw charge.
Two recipients double damage without multiplying contact pause or charge.
Zero actual HP loss adds no pause. A legal Normal4 Jump after498069us of
progress preserves its first two packets and cancels the five unissued packets.

Aim's native first marker is16958us, next-ability/Jump recovery133047us and
movement gate0. It selects one recipient and no Cat group. Cancellation before
its first marker creates no damage. Source execution/replay checks two full
five-input chains through `presets/native/nanally-normal-control.nte`.
`nanally_clock.rs` owns independent numeric and causal checks; owner death,
interruption, field exit and new admitted actions cancel unissued clock groups.
Continuous held sections and preserving Dodge now execute; joined acceptance is recorded below.

### Executed landing and Basic resource bindings

Nanally now accepts the shared typed `plunge[airtime=800ms,height_bonus=1/2]`
profile. Flight is declared real time; the finite landing clock begins at
touchdown. AirAttack1 hits at50170us with next-ability/Jump/movement gates
255141/123558/749161us, extended by59400us only after actual positive contact.
The single uncapped landing group starts one60ms/.01 pause regardless of dummy
count. Source profiles require positive airtime and height bonus within0..1;
damage uses native Normal-talent coefficient*(1+bonus), not a spatial curve.
At ATK1000/DEF1/2: L1/no bonus250; L8/half bonus2571/4; L9/full bonus925.
Charge is always the unscaled native2.5 per connected packet group, subject to
shared recipient efficiency/carry/capacity. Aim explicitly binds its native.7
charge under the same policy; two Aim packets grant owner1+.4 and a living
Nature peer0+.84. Cancellation/zero HP loss grants nothing.

A captured live Underboss selects one Cat_AirAttack group at landing impact,
using its creating Burst talent without the player's height multiplier. The
selected Cat landing hits all living dummies, counts once for A1/A5 and grants
no ordinary resources. Expiry/replacement/death are checked at delivery;
interruption before landing removes both unissued groups. These are chosen
causal/recipient policies rather than frame-verified dispatch claims.
`nanally_plunge.rs` checks coefficients, cancellation gates, two-recipient
group accounting, one A1/A5 callback and half-open summon expiry.
`presets/native/nanally-plunge-control.nte` supplies two continuing flights:
impacts850170/1964711us, total5 energy, no resets, native aggregation/replay.
The joined continuing acceptance below covers the full selected model.

### Executed continuous held sections

The typed held duration and `attack[hold=3s]` source input now retain one actual
Normal/Riposte press and its real release time. Content chooses each next section
at the exclusive source position plus1us, projected through actual contact lag;
it does not turn the duration into a requested stage. Normal1/2/3/5 branch at
300001/350001/600001/300001us, Riposte at900001us. These already satisfy the
initial.25s hold floor. Normal4 at900001us selects Normal5, whose early heavy
branch cancels its unissued second packet. Heavy1/2 continue at600001/980001us
without re-adding a.25s delay to an already held button. Release is half-open;
started trains complete unless legally cancelled. Extra held time after Heavy3
remains real elapsed input time and produces no extra attacks. Jump/movement/
Skill/switch retain their source cancel gates. Preserving Dodge is described below.

All6 native heavy packets and7 Cat-heavy packets now have consumed typed rows.
Heavy1 usesExtraAtk1+2; Heavy2 Intro1 twice+Intro2; Heavy3 Outro1. Unrequested
Outro2 and Float-only request8_1 remain absent. Cat groups areBranch1;2+3+4;
Branch5x3, using the creating Burst talent, selected uncapped recipients and one
A1/A5 accounting group per section. No Cat ordinary charge/Cycle/Break is added.
A4 affects those Cat packets only. The original admitted input owns every
subsection packet; native packet categoryA remains separate from that input's
role (including a Riposte prefix), without fake commands or extra cast observers.
These hold/dispatch/input-accounting choices remain revisable models.

Independent L8 check at1000ATK/DEF1/2: heavy5.822*500=2911, plus actual
Normal1 prefix.507*500 gives6329/2; Cat-heavy4.876*500=2438, or4876 withA4.
Native heavy raw charge.231+2.596+.448*2+3.267+6.111=13.101; addingNormal1
1.138 gives14.239, unaffected by recipient multiplicity. Continuous positive
contact timings are191054,760520,893587,1376795,1638344,1867695,2621019us;
next Normal is legal at3706608us. Zero-loss timings omit contact delays and
grant no energy. Exact section-boundary release and interruption tests preserve
settled packets only. `nanally_held.rs` also checks Normal4/5 cancellation and
paid Riposte provenance. `presets/native/nanally-heavy-control.nte` executes two
continuing source chains:14 packets,28 whole charge plus239/500 carry, with
actual timestamps retained in native aggregation/replay. Full-kit acceptance
is recorded by the joined continuing baseline below.

### Executed preserving Dodge

The [published kit text](https://neverness.app)
states that dodge preserves both ordinary and heavy sequences; it does not
establish a retention deadline. The selected execution model now retains fresh
progress on admitted Dash and rebases the existing2s idle window at that real
commitment. A greater gap resets it; exactly2s remains fresh. All real dodge
recovery/stamina still apply. Unissued old-phase packets/Cats cancel and are
never restored. Jump/Special, another ability, field exit, defeat and explicit
interruption clear the continuation; a same-actor no-op swap preserves it.

A reached Heavy1/2 leaves only its next section eligible. The next held Normal
requires at least200ms and consumes those200ms before starting Heavy2/3; it
uses native packet/contact/cancel progress without repeating earlier sections
or adding a Normal prefix. A shorter hold or ordinary tap takes the Normal
route and clears the permission. A paid Critical Dodge's one-use Riposte has
priority and clears the old heavy route; a long Riposte hold can then reach
its own Heavy1. No extra commands, refunded stamina or successful Parry is
created. Retention, entry delay and subsection accounting remain revisable
policies; published text alone is not frame certification.

Independent checks: Normal1/Dash/Normal2 impacts191054/1320884us. After the
third Normal, a Dash at2994300us rebases the idle window and permits Normal4
at3843778us even though the original Normal3 input was over2s earlier. The
H1/Dash/H2-H3 split has first-period hits191054/760520/893587/2445293/
2706842/2936193/3689517us. Resuming Heavy3 after H2 instead hits3350737us.
The new.2s hold boundary and2s idle boundary are checked at exact/+1us cases;
critical Riposte priority, ordinary tap reset and no-op swap retain real inputs.
`presets/native/nanally-dodge-control.nte` executes two preserving periods:
14 packets,28+.478 Ultimate charge,47+.402 Cycle and30 paid stamina, with
native aggregation/replay. At1000ATK/DEF1/2, two periods deal6329 exactly;
their longer elapsed time is retained rather than compared as instantaneous
damage. Full-kit acceptance still requires the joined legal continuing gate.

### Executed native Cycle channel

The owner now binds the consumed native HeterochromeAdd values to the shared
`combat.packet_cycle_accrual_v1` ledger. Positive connected player groups grant
once, including a surviving secondary dummy after primary defeat. There is no
party distribution or Charge Efficiency; Cycle gain efficiency, exact carry,
100 cap, overflow and qualified100 spend remain shared engine behavior. Fatal
positive damage qualifies, while cancellation and zero HP loss grant nothing.
Native Burst/Support/Cat/A3/Fair Duel channels are explicit zero. Metadata or a
previous authored full pool alone never established this actual accrual binding.

Independent checks: a full five-input chain grants
1.9+2.49+.784+1.344+8.151+7*.446+4.948+4.962=27.701 Cycle, separately from
16.599 Ultimate charge. At1.5 Cycle gain efficiency this is41.5515, leaving
Ultimate charge unchanged. Heavy-only .384+4.320+.746*2+5.436+10.169=21.801;
with its actual1.9 Normal1 prefix this is23.701, hence two source-held periods
give47+.402. Two height-bonused landings grant unscaled8+.4; Aim two2+.2;
Riposte6+.9; five Skill packets10+.4. A3/Underboss callbacks add no Cycle.

`presets/native/nanally-cycle-control.nte` starts Nanally at zero, accrues
110.804 across four full chains and spends100 on the actual Cosmos/Nature
handoff. Return and another Normal preserve old carry: initial0/current2,
whole granted112/spent100/overflow10/carry88/125. Two real handoffs create
20 enhanced Bud emissions. Eighty total packets, exact ledger conservation
and source/native replay are checked without resets or hidden fills. Zero's
authored initial100 supplies its separate return handoff, not Nanally's spend.
Strict tests explicitly accept the reached shared accrual model after rejecting
its omission. Source values are frozen rows; contact/recipient/unit interpretation
remains the shared selected resource model, not a frame-based certification.

### Executed Burst and summon notifications

`nanally_burst_clock.rs` binds seven rate-one progress markers2300192/2382032/
2468449/2547640/2634465/2716783/3015465us and the separate3415870us summon
notification. General ability/Jump3582140us, switch3583025us and movement3756871us
remain distinct; an explicit authored gap can reach the later switch/movement
gate. Q starts15s cooldown and pays100 energy at commitment. Interruption retains
payment/cooldown and settled damage, cancels the unissued train/unborn summon,
and preserves a summon already applied. Exact-time birth settles at the shared
BuffTickOrExpiry phase before explicit SettlementAndDeath interruption. The
committed source command and effective Burst talent are captured for that birth.

Independent Burst8/ATK1000/DEF1/2 damage remains8569 with no creation/Cat/A1/A5
payment. Interrupt2400000us preserves the first two Q packets, cancels five and
creates no summon. Interrupt3415869us cancels birth; at3415870us the reached
summon survives and can coordinate a later Normal. Original cooldown still
rejects a funded/otherwise eligible input at14999999us. Normal inputs begin after
Q's actual recovery; nine legal A2-window inputs still exceed the proposed
eight-group cap and produce16 Cat packets. A0's full five-input chain plus E
can issue its E Cat group at9349560us before the9415870us summon deadline.

The native source fixture `presets/native/nanally-burst-control.nte` funds two
continuing Bursts through twelve full chains and two Skills:212 hits,211 whole
granted energy plus197/250 carry,200 spent,11 overflow and100 final energy.
The source checks real recovery, both summon deadlines and original cooldowns;
native CLI checks aggregation/seeded replay and bounded WASM/Worker transport
uses that same file. Every ordinary input and packet retains its real elapsed
time; no eight-group counter, auxiliary stocks, polling or synthetic Cat FIFO
is introduced. The selected playback/dispatch policies remain revisable.

Committed non-damaging inputs also retain actual command identity. A100ms
interrupt during a paid critical dash preserves its15 stamina payment and
already granted one-use Riposte permission; a later200ms attack uses the native
467562us marker in the explicit interrupt fixture. This retains costs and
admitted causality without inventing a free dodge or a successful Parry.

### Executed Skill notifications

The finite clock now binds Skill packets at429637/497363/563001/631341/696485us
and separate cooldown425571us/Authority429917us progress notifications. It uses
the same upward rounding, actor contact tracker and command-owned cancellation
as other bound routes. Every packet is uncapped AoE with one energy grant per
positive group. Native next-ability/Jump recovery is1164219us and movement
requires1512797us. Rate-only source E children imply no extra positive pause.
The first packet precedes Authority and lacks its additional30pp CRIT DMG.

At ATK1000 and DEF mitigation1/2, five level1 E packets deal2500 noncrit;
their potential critical damages are750 then900 four times at base CRIT DMG50%.
With selected A3/A5 and effective Skill2, the second E requests A3 before the
third E: first two direct packets deal540 each, A3 deals270, and the final three
sample the new2%-base-ATK stack for2754/5 each. A1 still grants once through its
independent guard. A6 captures combat state at the actual application, so an
explicit combat exit200ms after commitment selects20s at429917us rather than15s.

`nanally_skill_clock.rs` checks native markers, a dead primary with surviving
AoE recipient, first-hit-before-Authority, reached/unborn notification boundaries
and A6 capture. Interruption425570us cancels cooldown; at425571us the shared
BuffTickOrExpiry notification precedes SettlementAndDeath interruption, so the
deadline16425571us survives. At427000us no damage/Authority has happened; at
430000us one settled hit and applied Authority survive while four packets cancel.
The existing Authority source fixture and native CLI aggregation/replay check
two continuing E/removal periods and12.6 energy without resetting cooldowns.

### Executed Support and Riposte clock

Both routes now use the finite owner clock. Support's source impact666410us
adds one delayed interval only after positive actual HP loss: start+60000us,
duration250000us, progress rate.2. Recovery1108275us therefore becomes1308275us;
the separate switch gate1083416us becomes1283416us. Riposte's nominal841593us
recovery becomes1138593us after one300000*(1-.01)=297000us contact loss. Zero
HP loss adds no pause, and multiple AoE recipients cannot multiply one interval.
Riposte Jump/Skill cancellation is allowed from zero progress and removes the
unissued packet without refunding paid dodge stamina or its consumed permission.
Forced interruption cancels an unissued Support group on all recipients while
retaining the actual switch command in its interruption receipt.

`nanally_secondary_clock.rs` checks these boundaries and two continuing source
periods in `presets/native/nanally-secondary-control.nte`: each period has one
Support and one paid Riposte, with native elapsed completion2.946868s and
19.946868s, total22.4 self energy and30 spent stamina. Native CLI aggregation,
seeded replay and bounded WASM/Worker transport use the same source. Playback
and positive-loss contact interpretation remain selected models.

The following table retains source-grounded coefficients. A row alone does not establish a public source action: the continuous held and preserving Dodge routes execute; joined acceptance is recorded below.

| Action | Packet suffix after `GE_Player_Nanally_` | ATK fractions1 / 8 / 9 | Count/category |
| --- | --- | --- | --- |
| Normal1 | Melee1_Damage | .296 / .507 / .548 | 1/A |
| Normal2 | Melee2_Damage | .388 / .665 / .718 | 1/A |
| Normal3 | Melee3,4,5_Damage | .122+.210+1.270 / .209+.360+2.177 / .226+.389+2.351 | 3/A |
| Normal4 | Melee6_Damage | .069*7 / .118*7 / .128*7 | 7/A |
| Normal5 | Melee9,10_Damage | .771+.773 / 1.321+1.325 / 1.427+1.431 | 2/A |
| Heavy1 | ExtraAtk1,ExtraAtk2_Damage | .060+.673 / .103+1.153 / .111+1.246 | 2/A |
| Heavy2 | ExtraAtkIntro1 twice,ExtraAtkIntro2_Damage | .116*2+.847 / .199*2+1.452 / .215*2+1.568 | 3/A |
| Heavy3 | ExtraAtkOutro1_Damage | 1.585 / 2.716 / 2.934 | 1/A |
| Aim | BranchPraojectile_Damage | .130 / .223 / .241 | 1/A, single target |
| Plunge landing | AirAttack1_Damage | .500 / .857 / .925 | 1/X |
| Critical Riposte | PerfectEvadeAttack_Damage | 1.660 / 2.845 / 3.073 | 1/Z |
| Skill | Skill1_Damage | 1.000*5 / 1.714*5 / 1.851*5 | 5/E |
| Burst | UltraSkill1, UltraSkill2 five times,UltraSkill3_Damage | 2.931+5*.423+4.954 / 5.023+5*.725+8.490 / 5.425+5*.783+9.170 | 7/Q |
| Support | QTE1_Damage | 2.000 / 3.428 / 3.702 | 1/H |

The Heavy1/2 split here **corrects the earlier brief inventory**, which placed intro packets under the wrong stage. `Nanally_ExtraAttack1` sends Melee.1_1 then2_1; ExtraAttack2 sends4_1 twice then4_2; ExtraAttack3 sends5_1 once plus8_1 **Float only**. Melee.3_1's Outro2 row is unrequested in these selected montages. It is not a seventh heavy damage hit. Approach/entry Normal are alternatives to Normal1, never extra unconditional packets. Descent has no packet until landing; use declared landing time and fall bonus0..1, multiplying the landing packet by1+bonus as [Mint's spatial abstraction](../mint/action-damage.md#complete-packet-specification), not a guessed meters-to-coefficient curve.

| Underboss coordinated action | Suffix after `GE_Player_Nanally_Cat_` | ATK fractions1 / 8 / 9 | Group count |
| --- | --- | --- | --- |
| Normal1 | Melee1_Damage | .250 / .428 / .463 | 1 |
| Normal2 | Melee2_Damage | .225 / .386 / .416 | 1 |
| Normal3 | Melee3,4_Damage | .425+.400 / .728+.686 / .787+.740 | 2 |
| Normal4 | Melee5_Damage | .317*3 / .543*3 / .587*3 | 3 |
| Normal5 | Melee6,7_Damage | .450+.600 / .771+1.028 / .833+1.111 | 2 |
| Heavy1 | Branch1_Damage | .311 / .533 / .576 | 1 |
| Heavy2 | Branch2,3,4_Damage | .220+.201+.715 / .377+.344+1.225 / .407+.372+1.323 | 3 |
| Heavy3 | Branch5_Damage | .466*3 / .799*3 / .863*3 | 3 |
| Skill | Skill_Damage | .240*5 / .411*5 / .444*5 | 5 |
| Plunge | AirAttack_Damage | .500 / .857 / .925 | 1 |
| Riposte | PerfectEvadeAttack_Damage | .857 / 1.469 / 1.586 | 1 |
| Support | QTE_Damage | 1.500 / 2.571 / 2.776 | 1 |

Every Cat packet is A/NATURE, tagged as Nanally follow-up/Underboss damage, with **zero ChargeAdd, HeterochromeAdd and UnbalValue**. Cat heavy groups are1/3/3, not the earlier rough2/2/1 grouping. This agrees with the separate published ratios: Heavy2 contains Branch2 plus3 plus4, Heavy3 repeats Branch5 three times. Fair Duel's separate `GE_Nanally010_Lv1_Damage` is .600/1.028/1.111; A3's `GE_Nanally010_Lv3_Damage` is .500/.857/.925; both have zero resource/gauge fields, Normal scaling, follow-up identity and a single enemy recipient. Neither is an ordinary Cat packet just because it follows an attack.

**Burst applicability decision:** Q has seven direct packets and creates a summon; neither event admits a Cat damage group. Cat_AirAttack remains a plunge packet, not a free summon assist. The native summon marker after direct hits supports this choice, while its later source notification now executes through the finite clock; playback remains a selected rate-one model.

**Independent level8 sums:** ordinary Normal7.390, heavy5.822, Skill8.570, direct Burst17.138, Cat Normal5.656 and Cat heavy4.876. ATK1000 and DEF mitigation1/2 give3695,2911,4285,8569,2828 and2438 respectively. A4 leaves direct Q8569 unchanged. These sums check coefficients, not measured timing or implementation of every listed route.

## Independent executable checks

`simulation/tests/nanally_underboss.rs` checks seven direct Q packets for8569 damage at Burst8/ATK1000/DEF1/2, no Cat hit or A1/A5 grant from summon creation, no hard eight-group cap (nine legal A2-window Normal inputs yield16 Cat packets), mixed Normal/Burst talent capture, A4's additive zone, group-based A5 with expiry restart, A1 efficiency/carry, half-open expiry, generation replacement and field/death cancellation.

`nanally_continuing_energy.rs` preserves a zero-start continuing resource fixture:129 packets and93719/2 total damage after removing the eight free Burst Cat packets; ordinary team energy/carries remain unchanged. `source_queries.rs` checks two authored E/Q/Normal cycles and full native replay with continuing energy. `nanally_removal.rs` and its source fixture check legal non-damaging early removal, selected readiness and original cooldown preservation.

Support retains its native switch parent and 0.666410-second impact. Each reaction-qualified entry can cast it once, subject to the normal action gates. Support and Critical Riposte use the finite positive-contact clock described above, including miss-dependent recovery and distinct cancel gates. The selected timing uses source montage windows and modeled hit pause.

## Timing evidence and limits

Source `Nanally_Skill` damage markers are.42963672,.49736238,.5630008,.631341,.6964841, with separate Authority marker.4299165. Burst's summon marker is3.4158695 nominal. The Skill markers now execute through the selected rate-one finite clock, including first-hit-before-Authority; real animation rates remain unverified. Burst now executes its seven markers and later summon notification under the same conservative profile. Acquisition hashes and historical research are retained without importing raw payloads into runtime constants.

## Candidate source action clocks

Normal/Aim/Support/Riposte/Skill/Burst/plunge now execute the conservative profile through
the finite owner clock above. Continuous held heavy and preserving Dodge now execute these
source coordinates with the explicit retention/entry policies below.
Source coordinates, inferred playback/pause policy and executable coverage remain
separate. The optional fast-combo profile remains unimplemented research.


**Mixed source evidence and prior clock proposal.** Selected montage notification offsets are nominal source coordinates. Use the same explicitly inferred rate1 plus **actor-local hitstop integration** as [Mint's timing model](../mint/action-damage.md#timing-and-continuation-specification), with Nanally's own values below; do not borrow Mint's cooldowns or measured-rate status. Saved public trace gives .7834902/.8000040 first-to-last Burst spans versus .7152724 nominal; this is a real discrepancy and the rate1 profile is not claimed to fit its network/patch-confounded timeline. Missing pause durations produce no additional interval under this disclosed bounded profile; no native zero-default fact is asserted. Whole-run DPS uses relative elapsed time; real timestamps and local hitstop remain on the absolute action axis.

| Action | Nominal hit offsets from its input | Next ability / movement | Jump |
| --- | --- | --- | --- |
| Normal1 | .19105396 | .6118604 / .61098737 | .24179313 |
| Normal2 | .20902252 | .8702402 /1.133133 | .25919926 |
| Normal3 | .23292544,.31681442,.50813806 |1.0830476 /1.3834089 | .28285894 |
| Normal4 | .34947714,.39946318,.5087315,.63086766,.71712416,.7705018,.8338208 |1.331513 /1.4348118 | .49806824 |
| Normal5 | .24936193,.80146337 |1.13422 /1.6496695 | .33225998 |
| Approach/entry Normal1 | .20930798 (entry .209308) | .6326137 /.6909955 | .26669306 |
| Heavy1 | .46051872,.5143856 | .86601347 /.9679487 | .41683364 |
| Heavy2 | .22929224,.44084182,.62019247 |1.0097181 /1.0097181 |1.0527335 |
| Heavy3 | .2945152 |1.1821046 /.9496715 |1.1821046 |
| Aim | .016957 (Fire_LoopAim), .016957164 (Fire) | .13304666 / loop retains aiming movement; Fire exit .13303788 | .13304666 |
| Plunge landing | .050169364 after declared landing | .25514066 /.74916047 | .123557165 |
| Riposte Atk | .26756155 | .8415922 /.8415922 |0 (Skill also allowed0) |
| Skill | .42963672,.49736238,.5630008,.631341,.6964841 |1.1642182 /1.5127965 |1.1642182 |
| Burst |2.3001919,2.3820317,2.4684486,2.54764,2.634465,2.7167826,3.0154643 |3.5821395 /3.756871 |3.5821395 |
| Support | .6664094 |1.1082748 /1.1082748 |1.1082748 |

Skill's cooldown begins at **TriggerCooldown .4255706**, lasts **16 World seconds** and continues off field. Authority's recast lock begins with Skill.3, lasts **2 World seconds** from `GE_NanallyRemoveSkillCD`; it is separate from16s normal Skill cooldown. Burst costs/starts **15s CD** at admission; summon start3.4158695 is not cooldown origin. The cancellation curve3 is a source input for aborted casts, not a full-cast cooldown overriding16. Failed admission starts no clock; a pre-notify Skill cancellation leaves no Skill cooldown but no unissued hits/state either.

Explicit source hitstop `(duration,rate,delay)` values: Normal3's final Melee5 `(.13,.01,0)`; Normal5 Melee9 `(.06,.01,0)` and Melee10 `(.12,.01,0)`; Heavy1 ExtraAtk1 `(.08,.01,0)` and ExtraAtk2 `(.17,.01,0)`; Heavy2 each Intro1 `(.1,.5,0)` and Intro2 `(.1,.01,0)`; Heavy3 Outro1 `(.2,.01,.05)`; plunge `(.06,.01,0)`; Riposte `(.3,.01,0)`; Support `(.25,.2,.06)`. Rate-only Normal/Skill/Burst children supply no additional positive duration. Cat follow-ups do not impose an invented hitstop on the player. Overlaps use minimum actor progress rate; one source event's AoE creates one interval. A full conservative Normal chain takes **5.0308812+.99*(.13+.06+.12)=5.3377812s**. Heavy1 hit2 is .5935856 after the .08/.01 first pause; recovery1.11351347. A three-target heavy hit does not triple that loss.

**Switch/cancel inference:** use separate next-ability, movement and jump windows. Switch normally uses next-ability boundary; explicit CantChangePlayer ends **Burst3.5830247**, Support1.083416. Thus Burst switch is the later3.5830247, slightly after AnySkill3.5821395, and Support switch may precede next ability1.1082748. Legal cancellation cancels future montage notifications, retains delivered damage/resources and explicit independent already queued Cat packets under generation/lifetime rules. A same-time dispatched hit precedes authored switch. Forced interruption aborts the active player sequence and pending notifies; it does not refund Burst or reset cooldowns. Authority survives an ordinary stagger unless another source rule removes it; field exit always removes it.

**Held transitions and optional fast-combo timing inference:** map `GA_Nanally_Melee#/5/JumpSectionMap` into the actual section checks. Melee1/2/3/5 and Riposte may transition to Heavy1 after holding.25s and source position lower bounds .3/.35/.6/.3/.9 respectively; ExtraAttack1â†’2 requires hold.2 and position>.6, ExtraAttack2â†’3 requires hold.2 and position>.98. Take the later hold/position bound, treating exclusive bounds as the next microsecond; do not delay every heavy input by an extra .25 after its predecessor has already satisfied the hold. Normal4's held branch requests Normal5 after position>.9, not Heavy1. Entry/approach may preserve stage1â†’2 through their source hold rule, not duplicate stage1.

The conservative next-ability profile above is the selected baseline. A possible continuous Normal-combo research profile uses `max(last stage damage marker, jump-window start)` as a source-bound proxy for the next Basic; this is a distinct **inference**, since the native chain-input consumer is unavailable. Its nominal full chain is2.64441462 plus .3069 explicit hitstop=**2.95131462s**. Five Normals then three heavies may fit before6s with that profile, consistent with the public combo recommendation; this does not establish measured timing. The faster profile is not currently implemented or selected. If future evidence selects it, update the contract, code, comparisons and tests together.

## Provenance

**Additional serialized evidence, October 16:** the client bundle contains `GA_Nanally_Skill` export SHA-256 `d14c115429658284fd31bef9d29be4864fc06d289623055bc493b1ac4aaeb781` and `Nanally_Skill` montage export `64388f3e6b6b572192e276441b7250fa51f4029871725992b19bb66085ec5aa1`. The GA's `EffectContainerMap` maps both `Event.Montage.Player.Skill.1` and `.2` to `GE_Player_Nanally_Skill1_Damage`; `.3` instead names wall-walking/remove-Skill effects. In the montage, `/1/Properties/Notifies/{13,20,22,24,27}` link the five damage-dispatch markers at asset coordinates `0.42963672`, `0.49736238`, `0.5630008`, `0.631341`, `0.6964841`; the separate `.3` marker is at `0.4299165`. These support a distinct configured five-marker train, not coincident native delivery. The original investigation retained a coincident runtime model; the selected finite Skill clock above now supersedes that implementation limit without claiming measured playback. Parser warnings remain attached to the source export in the bundle; unrelated particle warnings are not silently erased or applied to unrelated export types.

- Model and checks: [`reviews/q10-game-action-binding/nanally-action-binding-model.json`](../../../../reviews/q10-game-action-binding/nanally-action-binding-model.json).
- Owner supplements: [native owner tests](../../../../crates/content/src/simulation/tests/nanally_actions.rs), [`nanally-normal-chain.json`](../../../../reviews/q10-game-action-binding/nanally-normal-chain.json), [`nanally-skill-packets.json`](../../../../reviews/q10-game-action-binding/nanally-skill-packets.json).
- Frozen inputs: `data/extracted/guide_nanally.json`, `data/extracted/neverness_nanally.json`, the consumed [typed damage rows](../../../../crates/data/src/native_damage/nanally.rs) (raw evidence remains cache-only), `1010AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

### Transfer probe findings, October 17, 2026

The method transfer probe from the Mint pilot recovered selected configuration for Nanally Skill -> Underboss -> Cat coordination, using client asset extraction (`bundle-v1`), montage notify timelines, and calculation bytecode. The coordinates below are serialized montage inputs; real elapsed timing, playback/hitstop and native callback delivery remain open under the [shared timing boundary](../../shared/action-timing.md):

1. **`GA_Nanally_Skill` montage (`Nanally_Skill`):**
   - Cooldown trigger: `TriggerCooldown` at `0.4255706s`.
   - Five damage markers: Hit 1 (`Skill.1`) at `0.42963672s`, Hit 2 (`Skill.1`) at `0.49736238s`, Hit 3 (`Skill.1`) at `0.56300080s`, Hit 4 (`Skill.1`) at `0.63134100s`, Hit 5 (`Skill.2`) at `0.69648410s` (packet spacing ~66ms). Aux removal marker `.3` at `0.4299165s`.
   - Cancellation windows: `BreakSkill_AnySkill` & `BreakSkill_Jump` at `1.1642182s`, `BreakSkill_Move` at `1.5127965s`.
2. **`GA_Nanally_UltraSkill` montage (`Nanally_UltralSkill`):**
   - Direct Burst packet markers: Hit 1 at `2.3001919s`, Hits 2..6 at `2.3820317s`–`2.7167826s`, Hit 7 at `3.0154643s`.
   - Cat summon marker: `Event.Montage.Player.UltraSkill.5` requesting `GE_Nanally_Cat` is at stored coordinate `3.4158695`, after the seven direct damage markers in this montage. This does not establish real-time settlement or callback order.
   - Cancellation window: `BP_CanBreakSkill_C` (`AnySkill`/`Jump`) at `3.5821395s`.
3. **Cat Coordination NotifyState (`BP_NanallyCatFuncNS_C`):**
   - The asset `HT/Content/Blueprints/Share/Notify/BP_NanallyCatFuncNS.uasset` inherits native `NotifyState_NanallyCatFunc` with `bEnableCondition: true` and `NotifyConditions: [{BuffClass: GE_Nanally_Cat_C}]`.
   - Configured across stored coordinates `0.0`–`4.917` in `Nanally_Skill`, and in `Nanally_Melee_001`..`005`, but *absent* from the inspected `Nanally_UltralSkill` notify list. This locates one possible dispatch path; it does not establish final native eligibility, exclude another callback path, or prove that direct Burst hits cannot coordinate. The existing immediate-Burst model remains a separate hypothesis.
   - Cat summon duration calculation `Cau_Nanally_Lv2CatDur_C:CustomCalculateBaseMagnitude` bytecode evaluates `DT_NanallyEffectFigure` curves `Nanally_CatDur` (6.0s) and `Nanally_Lv2_CatDur` (9.0s with Awakening Effect 2).
   - `DT_NanallyEffectFigure.json` provides `Nanally_CatInterval_Show` = `0.3s` (300ms nominal coordinated packet spacing).
   - Remaining limits: Native C++ dispatch mechanics within `NotifyState_NanallyCatFunc` and spatial targeting/recipient geometry for the Cat summon.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/nanally). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The selected `DT_SkillDamageData` rows supply coefficients at talent levels 1 / 8 / 9, categories, and resource channels:

| Action / damage row (`GE_Player_Nanally_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Plunge `AirAttack1_Damage` | X | 0.500 / 0.857 / 0.925 | 2.500 / 4.200 / 0.500 |
| Aim `BranchPraojectile_Damage` | A | 0.130 / 0.223 / 0.241 | 0.700 / 1.100 / 0.100 |
| Heavy1 opener `ExtraAtk1_Damage` | A | 0.060 / 0.103 / 0.111 | 0.231 / 0.384 / 0.046 |
| Heavy2 intro `ExtraAtkIntro1_Damage` (twice) | A | 0.116 / 0.199 / 0.215 | 0.448 / 0.746 / 0.089 |
| Heavy1 finisher `ExtraAtk2_Damage` | A | 0.673 / 1.153 / 1.246 | 2.596 / 4.320 / 0.515 |
| Heavy2 finisher `ExtraAtkIntro2_Damage` | A | 0.847 / 1.452 / 1.568 | 3.267 / 5.436 / 0.648 |
| Heavy3 `ExtraAtkOutro1_Damage` | A | 1.585 / 2.716 / 2.934 | 6.111 / 10.169 / 1.213 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.660 / 2.845 / 3.073 | 4.200 / 6.900 / 2.500 |
| Support `QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 7.000 / 0.000 / 2.500 |

Critical Riposte uses GAName `GA_Nanally_ExtremEvadeAtk`, while Support QTE uses `GA_Nanally_QTE`. All these actions deal NATURE/Anima damage. Hold attack montages (`Nanally_ExtraAttack1`, `Nanally_ExtraAttack2`, `Nanally_ExtraAttack3`) chain up to 3 stages; descriptions specify that dodging preserves the combo, and the branch chains after Critical Riposte and Normal stages 1, 2, 3, 5. Plunge damage description promises fall-height scaling up to +100%, terminating in native `DamageExecutionCalculation` without a captured height formula.

### Underboss Cat coordinated attack mapping

Underboss coordinates with all of Nanally's attack forms, emitting dedicated Cat packets that all belong to Category A/NATURE with ChargeAdd 0.0, HeterochromeAdd 0.0, UnbalValue 0.0:

| Coordinated action | Cat packet (`GE_Player_Nanally_Cat_` prefix) | Level 1 / 8 / 9 coefficients | Initiating player action |
| --- | --- | --- | --- |
| Normal chain | `Melee1_Damage` .. `Melee7_Damage` | 0.250–0.600 / 0.428–1.028 / 0.463–1.111 | `GA_Nanally_Melee` (9 children across 5 inputs) |
| Skill | `Skill_Damage` | 0.240 / 0.411 / 0.444 | `GA_Nanally_Skill` (5 packets) |
| Hold stage 1 | `Branch1_Damage`, `Branch2_Damage` | 0.311/0.533/0.576, 0.220/0.377/0.407 | `Nanally_ExtraAttack1` |
| Hold stage 2 | `Branch3_Damage`, `Branch4_Damage` | 0.201/0.344/0.372, 0.715/1.225/1.323 | `Nanally_ExtraAttack2` |
| Hold stage 3 | `Branch5_Damage` | 0.466 / 0.799 / 0.863 | `Nanally_ExtraAttack3` |
| Plunge | `AirAttack_Damage` | 0.500 / 0.857 / 0.925 | Plunge impact |
| Critical Riposte | `PerfectEvadeAttack_Damage` | 0.857 / 1.469 / 1.586 | `GA_Nanally_ExtremEvadeAtk` |
| Support QTE | `QTE_Damage` | 1.500 / 2.571 / 2.776 | `GA_Nanally_QTE` |

Cat summon base duration is 6.0s (`Nanally_CatDur` in `DT_NanallyEffectFigure`), extended to 9.0s by A2 (`Nanally_Lv2_CatDur`). Nominal coordinated packet spacing is 0.3s (`Nanally_CatInterval_Show`).

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Nanally_Passive_1` binds `Buff_Nanally010_Passive1`. `DT_ReactionExtendDataTable` row 1010 binds `ReactionAbilityID=GA_Nanally_Passive_1`, `AdditionalBulletCount=5`, and `FixFireCountDown=-1.0`. The published Blossom description ("More Than Passionate") specifies ten Vita Pistils per Bud (base five plus five) at one-second intervals. The negative saved field is not itself a one-second timer: native sentinel/default interpretation and actual firing cadence remain unproved.
- **Passive 2 (Breakthrough 4):** `GA_Nanally_Passive_2` binds `Buff_Nanally010_Level1`. "Fair Duel": whenever any party member deals Esper Cycle DMG while Nanally is in Ichi-daime's Authority state, Nanally deals 1 follow-up attack dealing Anima DMG scaling with Basic Attack talent level: `GE_Nanally010_Lv1_Damage` (0.600 at Lv1, 1.028 at Lv8, 1.111 at Lv9, 1.295 at Lv11), ICD 2.0s (`Con_Nanally_Pas2NoCD`).

### Awakenings

Source descriptions and selected IDs: `1010AwakenEffect#/rows/1010/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Follow-up attack -> grants Nanally 2.5 Ultimate Energy, ICD 1.0s (`Con_Nanally_Lv1`).
- **A2:** Extends Underboss duration by 3.0s (6.0s -> 9.0s, evaluated by `Cau_Nanally_Lv2CatDur`).
- **A3:** During Authority, dealing damage -> 1 follow-up attack dealing Anima DMG scaling with Basic Attack: `GE_Nanally010_Lv3_Damage` (0.500 at Lv1, 0.857 at Lv8, 0.925 at Lv9, 1.079 at Lv11), ICD 1.0s (`Con_Nanally_Lv3`).
- **A4:** +100% damage bonus to Underboss attacks (`DamageUpGeneralBase = 1.00`, additive damage-bonus zone). Direct Burst hits are excluded.
- **A5:** Follow-up attacks grant Nanally +2% ATK per stack up to 20% (10 stacks; `Buff_Nanally_ExtraAtkTimes`, `Buff_Nanally_ExtraAtkTimesMax`). Buff clears on field exit or combat exit.
- **A6:** Extends Authority duration to 15.0s (20.0s out of combat, evaluated by `Cau_Nanally_Lv6WalkDur`).
- **Resonance 3:** +1 level to Basic Attack, Skill, and Ultimate.
- **Resonance 6:** +15% total damage bonus.

## Historical implementation investigation

Consumed EffectFigure parameters are typed in [`nanally/parameters.rs`](../../../../crates/content/src/characters/nanally/parameters.rs); unconsumed source rows are cache-only. The existing owner model and its remaining implementation limits are preserved.

These were earlier implementation gaps. Current aim, Support/Riposte Cat bindings, Fair Duel and non-damaging Authority removal supersede those old limits; joined native full-kit acceptance is retained below and in the completion archive. No independent Cat charge pool or immediate Burst assist is retained.

### Authority removal and passive binding, October 18

In the initial bundle, `GA_Nanally_RemoveSkill#/4/Properties` requests removal of `GE_Nanally_WalkOnWall` and `GE_Nanally_GrantRemoveSkill` on activation through a hybrid-server component. `/3` supplies Skill cooldown tags and shows the WalkOnWall duration. `Con_Nanally_Using_E:CanApplyGameplayEffectContent` returns presence of that carrier after a successful source-player cast, with an explicit false failed-cast branch. The early-end route is located; it is not another ordinary five-hit Skill request. Native replacement admission/cost, delivery and pending-packet cleanup remain separate.

Do not assign the second passive by asset suffix. `DT_CharacterAbilityEffectConfig/GA_Nanally_Passive_2/GameplayEffectToActivate` selects **`Buff_Nanally010_Level1`**, not `Buff_Nanally010_Passive2`. The latter's `/1/Properties/BuffEventEffectData` requests `Buff_Nanally010_Pas2Charge` on SKILL_AFTER_DAMAGE through `Con_Nanally_Lv5`; it is not a second independently unlocked passive without a caller. The charge child `/2` names `ChargeExecution`, captures Source `PParameter1`, binds `Nanally_Pas2Charge`, and requests `GE_Nanally010_Pas2CD` at BEGIN. Its `/3` requires `Con_Nanally_Pas2NoCD`, which rejects an existing CD carrier after valid source-ASC checks. This locates the exclusion chain, not native Charge settlement or efficiency. The child's 0.01 duration and 100 period do not prove repeated regeneration.

`NanallyUltraSkillCat` in the expansion has a BoxComponent parser warning. Its decoded functions implement appearance/disappearance particles and owner lookup, not the native coordination executor. Preserve that warning and superclass boundary for future actor/charge claims.

The [current Reference guide guide](https://neverness.app), checked October 18, recommends five Normal inputs followed by three held inputs to exhaust Underboss at A0. This constrains a future charge-budget test, without establishing exact per-packet consumption or direct Burst coordination.

The numeric A4/resonance discrepancy with older saved guide descriptions is now resolved at the source-configuration level. `Buff_Nanally010_Level4#/1/Properties/CustomApplicationModifierInfos` binds additive DamageUpGeneralBase to `DT_NanallyEffectFigure#/rows/Nanally_Lv4_DamUp=1.0`, multiplier 1, requiring source tag **Ability.Player.Nanally.UltraSkillCat**. `1010AwakenEffect` resonance_6 selects `Buff_Nanally010_LevelExtra`; its `/1/Properties/Modifiers` adds DamageUpGeneralBase through `Nanally_LvExtraFarTarget`, rather than an independent final multiplier. The awakening table and current guide both describe **15%**. Preserve the older guide's +30%/+10% as historical descriptions, not competing native curve values. No runtime policy changes here; actual tagged packet eligibility and modifier capture remain separate.

### Projectile repair and Cat dispatch lead, October 18

The ProjectileBase field repair was also applied to the three saved Nanally actor assets. Cache  records the latest targeted reads; `nanally-projectile-rendering.txt` retains the reached functions. BranchProjectile and UltraSkillProjectile now recover their component/effect prefix, and the subsequent PrimitiveComponent repair removes their earlier component-read errors without changing CDO properties. No explicit collision delegate is recovered; native collision handlers remain separate. Neither reached CDO serializes a ProjectileFuncArray or explicit overlap interval/count; omissions do not prove one hit, no native dispatch or default timing.

NanallyUltraSkillCat's reached functions (`5251bd63990b487e.json`, `/2..6` in the cache's `projectile-components/data/local_assets/`) call the parent BeginPlay/EndPlay and appearance/disappearance effects, with a client/standalone gate for those effects. They do not supply a coordination charge ledger or Burst-hit trigger. Its earlier component warning is also resolved; that does not change the functions' visual scope. This targeted repair therefore does not resolve NotifyState_NanallyCatFunc, pending-group cleanup or native Cat packet cadence; do not promote visual lifecycle callbacks to damage triggers.

### Projectile trigger ability's guard and spawn route recovered, October 18

Initial warning-free **GA_Nanally_TriggerSpawnProjectile:K2_CanActivateAbility#/3** requires a valid avatar player and ASC and returns **NOT Has(GE_NanallySpawnProjectileCD)**, with explicit false on its invalid paths. The same ability's CDO (`#/4`) receives **Event.NoMontage.Skill.Common.SpawnActor**, names **NanallyUltraSkillProjectile** in its spawn map (`#/5`), and supplies **GE_Player_Nanally_UltraSkillPraojectile_Damage** in a same-event effect container whose target class is null. Its add/remove component (`#/6`) requests the guard on activation. This locates a concrete projectile gate and grant caller; the parent dispatcher and cue-side producer are recovered below, while native source context and the relation between direct-container/projectile delivery remain open. Do not double-count both routes as two confirmed hits.

Expansion **GE_NanallySpawnProjectileCD#/2** inherits **GE_CoolDownBase**, whose warning-free initial CDO (`#/2`) is HasDuration. The child explicitly replaces DurationMagnitude with **ScalableFloat=0.3**; it does not select the parent's custom cooldown-magnitude helper. The trigger ability separately declares **DelayEndAbility=0.5**. These are different configured guard/end inputs, not one universal measured ICD, nor the one-second A1 proc rule. Native guard clock, admission/commit ordering and simultaneous events still require their consumers. The saved actor table's PlayerNanallyUltraSkill row has no matched literal DamageTriggerConfigID caller in this selected source pass and is not substituted for this bound projectile path. Sources: `projectile-guard-and-slomo-config.txt`, `pause-and-cooldown-parent-config.txt` and `damage-trigger-callers.json`.

### Trigger parent and projectile targeting candidate checked, October 18

The selected class header confirms **GA_Nanally_TriggerSpawnProjectile → GA_TriggerGEBase**. Its [recovered parent](../../shared/action-timing.md#reached-trigger-ability-dispatcher-recovered-october-18) performs commit and CanApply checks before requesting the same-tag direct/spawn containers. This narrows the earlier parent-activation gap. The cue-side producer below locates the activation request; native guard clock and delivery multiplicity remain open.

The warning-free **NanallyProjectileLockRange:GetTargets#/1** first queries **TargetType_AttackTarget** using OwningActor. If returned HitResults is nonempty, it forwards that array with empty OutActors/TargetData; otherwise it forwards the **native GATargetType:GetTargets** fallback's outputs. CDO **#/2** configures MaxCount=1, bIgnoreCantTrace=true and range inputs. The later CallElement cue below supplies a concrete SearchAndLockTarget caller; this range is not a universal one-recipient proof for Nanally projectiles. Source: , `remaining-action-target-config.txt`, `mint-wind-ability-callers.txt` and `trigger-subclass-headers.txt`.

### Element cue's Tick-to-projectile caller recovered, October 18

Warning-free **GE_Nanally_CallElement#/3** names **GameplayCue.Display.Nanally.UltraSkill.CallElement**. The newly read **GC_Nanally_CallElement:OnWhileActiveEvent#/9 → ExecuteUbergraph#/1** resets **ElapsedTime=0**, caches MyTarget as player, and obtains ElementComp from that player's weapon-attach arm's first child when valid. **ReceiveTick#/10** calls its Actor parent, adds DeltaSeconds to ElapsedTime, then proceeds only with valid cached player and ElementComp. Once **ElapsedTime >= DelayAppearTime**, it updates/shows the element; on **IsLocalClientRole AND valid LockTarget AND ElapsedTime >= DelayAppearTime+DelayShootTimeAfterAppear**, it requests **TryActivateAbilityByClass(GA_Nanally_TriggerSpawnProjectile,true)**. There is **no ElapsedTime reset on this request**: it can request admission again on subsequent Tick, while the already located ability guard/commit path controls accepted activations. Its recovered derived CDO **#/11** supplies **DelayShootTimeAfterAppear=0.5** and a NanallyProjectileLockRange soft-class reference; DelayAppearTime's effective default is omitted. The inherited CDO tail has a LensEffectEmitterClass read error, so this is not a clean full-object decode or a measured first-shot/interval rule.

**On Update Element Rotation#/2** prefers the player's current GetAttackTarget; without one it clears cached LockTarget and requests **SearchAndLockTarget(NanallyProjectileLockRange)**. A previously cached target can be used in the current update; do not assume immediate lock acquisition or that the soft-class variable overrides the class literal. **OnRemoveEvent#/8** requests element disappearance and clears cached player, ElementComp and LockTarget. This locates a concrete producer and a bound target-query call, narrowing both earlier gaps; pending projectile destruction, native cue registration/multiplicity, pause/cancel clock and direct-container versus projectile hit admission remain unresolved. The callback requests neither two guaranteed hits nor a fixed 0.5-second firing cadence.

### Authority cue's conditional follow-up requests recovered, October 18

**GE_Nanally_WalkOnWall#/2** names GameplayCue.Display.Nanally.Skill.WalkOnWall. The matching newly read **GC_Nanally_WalkOnWall:OnWhileActiveEvent#/8** caches its player and invokes **HandleCharacterAttack#/3**. That path excludes IsHybridMode_Server, requires **IsAuthorityOrAutonomousHybridMode**, then subscribes **AsyncTaskCharacterAttacked(self,player,true,true)**. Its complete callback **#/4 → ExecuteUbergraph#/2** validates damaged character, DamageGE and cached player. One branch requires **InstigatorCharacter==cached player, individual Level3, no Lv3_CD and LastPas1DamTime != GetGameTimeInSeconds**; it requests Lv3_CD on the player, queries **GA_Nanally_Melee** level and requests **Lv3_Damage on the damaged character with that player as instigator**, then writes the current game time. A separate branch requires **individual Level1, no Lv1_CD and LastLv1DamTime != current game time**; it accepts either the DamageGE's **State.Damage.Reaction** tag, or target **buff_reaction_display2_new** with the player's element byte **2 or 3**. That branch requests Lv1_CD then Normal-level Lv1_Damage on the same damaged character, then writes its separate time. The second branch has no explicit same-instigator test, and neither visible branch compares DamageAmount >0. The [native callback parameter metadata](../../shared/buffs-and-callbacks.md#native-task-parameter-declarations-recovered-october-18) identifies this subscription as **bGlobal=true, bAutoEnd=true**; the Level3 branch still explicitly tests instigator equality while the Level1 branch does not. Native global scope, source propagation and re-entry/positive-loss policy remain separate.

The distinct marker/CD/tag/time tests must survive modeling: an unrestricted after-damage proc, or one shared timestamp for both branches, would change eligibility. The earlier passive/awakening description summaries do not by themselves explain why this local cue gates the named Lv1 damage packet on individual Level1; retain the actor/marker-specific source route alongside those descriptions rather than relabeling it as the unconditional passive2 trigger or as an Energy write. OnRemove ends the attack/state/tag-query tasks and clears cached player. These complete functions are readable. The subsequent HaloBuffActor recovery rereads both CallElement and WalkOnWall CDOs without their initial tail warnings, confirming the exact configured tags and CallElement PrimaryActorTick.bStartWithTickEnabled=true. DelayAppearTime remains omitted; this does not establish a real Tick interval or callback clock. Sources: `cue-first-owner-tag-callers.txt`, `nanally-awakening-packet-conditions.txt` and `roster-cue-extraction.json`. Native live registration, settlement, resource grants and pending deliveries remain open.

## Historical native-evidence questions

- **Real timing/admission:** Montage notify coordinates for Skill (`Nanally_Skill`) and Burst (`Nanally_UltralSkill`) provide serialized asset markers. Real-time conversion, hitstop, animation playback dilation, and earliest cancellation remain subject to the shared 3-method timing barrier.
- **Burst coordination dispatch:** `NotifyState_NanallyCatFunc` is present in Skill and Normal montages, but absent from the inspected `Nanally_UltralSkill` notify list. Whether direct Burst hits trigger Underboss coordination natively or only later owner attacks qualify remains an unresolved dispatch question; the current selected model excludes an immediate Burst assist.
- **Authority recast and cleanup:** `GE_Nanally_GrantRemoveSkill` grants `GA_Nanally_RemoveSkill` on InputID 2 (`CancelAbilityImmediately`). The exact sequence of carrier removal, Authority state termination, recast resource cost (none serialized in cost table), and delivery of already queued Cat packets remains unverified natively.
- **Spatial geometry & targeting:** Recipient caps, AoE falloff, and Underboss summon spatial delivery are bounded by the all-in-range dummy contract; native geometry is outside product scope.


Historical investigation questions follow. Their old execution limits are superseded by the current contract above; only explicitly listed active-queue gaps remain implementation requirements.

- The [removal-ability declaration](#selected-decisions-and-rationale) is now portable. Resolve the granting GE's actual admission, replacement/recast eligibility and costs, and ordering of carrier removal, Authority state and already pending Cat deliveries. The configured cancellation label alone does not bind those events.
- Resolve Skill packet timing/interruption and per-hit recipient cap/selection/distribution; spatial AoE geometry is outside scope.
- Determine Underboss coordination eligibility on immediate Burst hits versus later owner attacks, summon combo timing, and Authority transition conditions.
- The later ordinary Normal/Skill group model does not resolve native charge consumption/exhaustion, animation intervals, delivery-time cancellation, or recipient rules. A1 caller cadence and A5 refresh duration remain bounded choices; the source-backed 1s A1 ICD and A5 field-exit clearing are now executed.
- Live A3 now executes the native Normal coefficient and source-backed 1s ICD. Positive-HP-loss caller gating, surviving target-zero selection, post-parent ordering, and ICD persistence across field changes remain explicit hypotheses in the known card.
- Exact cancel windows and unbound invulnerability remain timing limits. Current explicitly paid Critical Dodge/Riposte and shared stamina policies are executable; aim/form admission alone does not establish a successful dodge.

## Prior findings

- [`reviews/q10-game-action-binding/nanally-action-binding-model.json`](../../../../reviews/q10-game-action-binding/nanally-action-binding-model.json) records the historical modeled boundary and pending findings.
- Related owner investigations: [native owner tests](../../../../crates/content/src/simulation/tests/nanally_actions.rs), [`nanally-normal-chain.json`](../../../../reviews/q10-game-action-binding/nanally-normal-chain.json), [`nanally-skill-packets.json`](../../../../reviews/q10-game-action-binding/nanally-skill-packets.json).


## Research history

- Nanally has accepted all eight implementation-data blocks. Next owner/block is **Adler/packets**. Implement the selected model only after the full184-block collection campaign.
- Methods already used: current selected action/montage/calculator/cue source graphs; saved native dispatch/default and projectile repair attempts; the public Reference guide/Neverness kit and five-Normal/three-heavy recommendation; public trace timing bounds. No source producer for the alternative CallElement carrier was found; do not add its unbound bullet coefficient or repeat native/visual probes for optional polish.
- Research scratch: ; independent Fraction/schedule audit . Batch base `4fd37a547e52a73d183e0aba14d3d1c5de5ad2b6`. The handoff resolves model choices while retaining source/native uncertainty and partial runtime coverage separately.

## Relative combat timer binding

`Nanally_UltralSkill` Common.EndAbilitySkill at 3.584608 montage seconds. Retrieved serialized notify; selecting it as the global timer release is a model assumption. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
