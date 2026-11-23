# Specification: Nanally (character:1010)

## Selected executable model

This contract owns the current Nanally model. The [mechanics dossier](../../../../docs/mechanics/characters/nanally/action-damage.md) retains packet arithmetic, sources and limits. The selected assumptions below are executable policies, not frame-verified game facts. Replace a policy and its consumers/tests together when better evidence arrives. No alternate legacy model is retained.

## Ordinary Normal and Aim clock

- Five ordinary inputs select native stages with1/1/3/7/2 damage packets. Finite rate-one asset progress, rounded upward to microseconds, owns impact and recovery; caller hit delay cannot move these markers. A strictly greater-than-two-second idle gap after the latest ordinary/heavy phase or preserving Dash resets the chain. A fresh admitted Dash preserves the next ordinary stage and refreshes this selected idle window; its actual recovery/stamina still cost time/resources. Other abilities, Jump/Special, field exit and interruption reset it.
- Ordinary melee packets hit all living stationary dummies; Aim captures only the primary recipient. Each packet group grants ordinary energy once after its first positive actual HP loss, including a surviving non-primary recipient. Recipient multiplicity cannot duplicate energy or contact pause.
- Normal3's final packet adds130ms at actor progress rate.01. Normal5's first and second packets add60/120ms at that rate. Only actual positive HP loss starts these pauses. Shared exact hitlag projects future owner packets and recovery; equal-time recipients remain at the same marker. These source windows and conservative playback are selected models, without frame verification.
- Next-ability/switch, Jump and movement gates use their respective asset progress. Legal cancellation removes unissued owner packets and unlaunched Cat groups, preserving settled results and already dispatched independent children. Explicit interruption can cancel before first contact. Aim's movement gate is zero, allowing cancellation before its16.958ms projectile marker; its next-ability recovery is133.047ms. Aim creates no Cat group.
- A positive Aim packet grants the native0.7 ChargeAdd once; its single recipient does not change the shared living-party distribution/efficiency/carry policy.

## Plunge

- `plunge[airtime=...,height_bonus=...]` requires positive declared real time until landing and a dimensionless bonus within0..1. This profile supplies the airborne-entry assumption without geometry. Descent emits no damage. Landing uses native Normal-talent AirAttack1, category X/NATURE, at50170us after touchdown; multiply its coefficient by1+bonus, leaving its2.5 ChargeAdd unchanged.
- Landing hits all living stationary dummies. Positive actual HP loss starts one60ms/.01 contact pause per packet group. Landing-relative next-ability/switch255141us, Jump123558us and movement749161us become314541/182958/808561us after contact. Flight consumes real time and is not shortened by contact pauses. Interruption before impact cancels unissued landing/Cat packets without damage or energy.
- A live Underboss selects Cat_AirAttack at the landing marker, with the creating Burst talent, native category A/NATURE, no height multiplier and no ordinary resources. The selected Cat landing is uncapped AoE, sharing one performed group for A1/A5; the summon must still be alive/unreplaced at delivery. Expiry is half-open. Captured timing, recipients and fall bonus are revisable execution policies.
- Two source-authored continuing landings preserve flight, positive-contact recovery, five total charge and native replay. This does not replace full-kit continuing acceptance.

## Native Cycle generation

- Connected Normal/heavy/Aim/plunge/Riposte/Skill packets grant their frozen HeterochromeAdd through the shared native Cycle ledger, once per scoped packet group. Only the living source owner receives Cycle, using its separate `cycle_gain_efficiency`, exact carry,100 cap and explicit overflow. Fatal positive HP loss qualifies; cancelled/unavailable/zero-loss packets do not. Recipient count cannot multiply the grant. No Charge Efficiency or party Ultimate distribution is reused.
- A five-input Normal chain grants27.701 Cycle, independently of16.599 ordinary Ultimate charge. Heavy sections grant21.801 plus1.9 from the actual Normal1 prefix. Plunge grants4.2 without the height multiplier; Aim1.1, Riposte6.9 and each Skill packet2.08. Burst, Support, Cat, A3 and Fair Duel have explicit native zero channels.
- Qualified Support switching spends the same100-point pool without resetting carry or overflow. The source continuing fixture generates110.804 from four chains, spends100, then generates1.9 more after return: final2,112 whole granted,100 spent,10 overflow and88/125 carry. Its two real Cosmos/Nature handoffs create20 enhanced Bud emissions; no initial Nanally Cycle or hidden refill supplies the spend. Strict execution requires the reached shared accrual assumption explicitly.

## Continuous held Normal

- `attack[hold=3s]` starts the actual next Normal stage or the available paid Critical Riposte. It holds that one button for the authored real duration. At reached source progress, a still-held button may enter Heavy1: Normal1/2/3/5 require progress greater than.3/.35/.6/.3s, Riposte greater than.9s. These positions already exceed the initial.25s hold requirement. Normal4 instead enters Normal5 after progress greater than.9s; it cannot jump directly to Heavy1. Exclusive bounds use the next microsecond.
- Heavy1 enters Heavy2 after progress greater than.6s; Heavy2 enters Heavy3 after progress greater than.98s, while the existing button remains held. Their.2s hold requirement is already satisfied. Release is half-open and consumes real time, including contact pauses; release at a reached boundary prevents the next section. A started section finishes its native train unless a legal cancel/interruption removes unissued packets. Remaining hold after Heavy3 creates no extra attacks and prevents a new ordinary Normal before release. Explicit Jump/movement/Skill/switch cancellation retains its own source gate and may release the button early.
- Heavy sections contain2/3/1 packets, with no unrequested Outro2/Float damage. Native source markers and per-contact slowdown use the dossier's heavy profiles. Each player packet is uncapped AoE and grants native charge once after positive HP loss. Branching cancels the predecessor's unissued packets, including Normal5's second packet after its early heavy transition; it does not undo settled damage, resources or launched independent Cat children.
- A live summon selects one Cat heavy group per reached section: Branch1; Branch2+3+4; Branch5 three times. These selected groups are uncapped AoE, use captured Burst talent and count once per group for A1/A5. They have no ordinary charge/Cycle/Break. A4 doubles these Cat packets, preserving direct heavy/Burst damage. Expiry, replacement and death are checked at delivery.
- The original admitted input owns all subsection packets. Native heavy category A remains separate from the initiating input role, including a Riposte prefix. Sections create no fabricated player commands or additional cast observers. Real duration, conservative playback, coincident Cat dispatch, subsection input accounting and recipients are revisable models rather than frame-verified claims.
- Two A0 source-authored held chains retain14 packets,28 whole energy plus239/500 carry and native replay. The joined equipped full-kit baseline below adds required passive/equipment paths.

## Preserving Dodge continuation

- Published kit text describes both Normal and heavy sequences as surviving dodge. The retained lifetime is unresolved; the selected model preserves a fresh next stage on admitted Dash and rebases its existing two-second idle window at the real Dash commitment. No readiness polling, refund or restoration of cancelled future packets occurs. Failed inputs do not mutate the continuation. Waiting creates no new progress; Jump/Special, other abilities, field exit, defeat and explicit interruption clear it. Same-actor no-op switching preserves it.
- A reached Heavy1/2 section may leave one next-section permission after Dash. A new held Normal lasting at least200ms resumes Heavy2/3 after consuming those200ms of real time, with its native packet/cancel/contact profile. It does not repeat prior sections or add a Normal prefix. The new command owns the resumed train. Less than200ms or an ordinary tap takes the Normal route and clears the heavy permission. A stale permission also takes the ordinary route; exactly2s after the preserving Dash is still fresh, one microsecond later is stale.
- A paid successful Critical Dodge takes priority: its next Normal/hold selects Critical Riposte and consumes the old heavy permission. A sufficiently long new Riposte hold can start its own Heavy1 route. No successful Parry or free dodge is inferred.
- Two source-authored preserving periods retain the same14 player packets and28+.478 Ultimate charge /47+.402 Cycle as their continuous counterparts, with30 paid stamina and longer real elapsed time. The H1/Dash/H2-H3 example's first-period hits are191054/760520/893587/2445293/2706842/2936193/3689517us. The retention window, new hold-entry delay and subsection accounting remain explicit revisable models.
- At positive contact, a full five-input chain takes5.337783s before the next ordinary input. Source execution advances through this actual recovery and exports the same native admissions and explicit waits for replay.

## Support and Critical Riposte clock

- Engine-owned Support retains its qualifying switch parent and native 666410us impact. Nominal next-ability recovery is 1108275us, with a separate 1083416us switch gate. A positive packet group schedules one 250ms pause at rate .2, beginning 60ms after impact. This adds 200ms to the corresponding gates. AoE recipients share the interval and one native 7-energy opportunity.
- One explicitly successful paid Critical Dodge authorizes one following Normal as Critical Riposte. The native marker is267562us and nominal next-ability recovery841593us. Positive actual HP loss adds one300ms pause at rate.01, giving1138593us recovery; a miss retains the nominal window. Jump and Skill may cancel at zero progress, removing unissued owner damage/unlaunched Cat groups without refunding stamina or permission. Ordinary cooldown/field/life gates still apply to the selected next input.
- Both routes use the same finite owner clock as Normal/Aim. They retain native talent/resource policies and independent launched Cat children. A pause already issued by settled contact continues across interruption and affects the next owner action; cancellation cannot undo that contact. Contact, cancellation and recipient interpretations are selected revisable models. Conservative asset playback is not a measured animation rate.

## Authority and Skill

- Ordinary Skill uses rate-one native progress: five uncapped AoE packet groups at429637/497363/563001/631341/696485us, rounded upward from source markers. Its cooldown starts at425571us and lasts16 World seconds. Authority applies separately at429917us, after the first hit. Next-ability/Jump recovery is1164219us; movement requires1512797us. Pending owner packets/notifications follow actor contact progress; no extra pause duration is inferred from rate-only E rows.
- Authority adds 30 percentage points of CRIT DMG, lasts12s (15s with selected A6), and ends on field exit, death, expiry or explicit early removal. It does not select another Normal coefficient chain.
- A6 captures duration at Authority application:15s in combat,20s outside combat through explicitly authored native combat transitions. Later leave/re-entry does not refresh or rescale an existing timer; without A6 the duration remains12s. Capture at application is a selected interpretation of the configured duration calculation.
- Live Authority replaces Skill with non-damaging RemoveSkill after2 World seconds from its application (2429917us after a fresh rate-one E). Readiness/cooldown queries use that selected gate; ordinary field, life and recovery admission still apply. Removal preserves the original ordinary cooldown, A5 and Underboss, creates no Cat group/new Authority/energy payment, and runs no ordinary Skill cast observers. No finisher or cooldown prorating is inferred. The admission retains the authored Skill and nominal packet template with utility/Other provenance; that nominal packet is never delivered.
- Interruption before cooldown notification leaves no Skill cooldown; after notification the reached cooldown survives. Unissued packets and unborn Authority cancel with their actual admitted command. Already settled damage/resources and applied Authority survive ordinary interruption; field exit, death and explicit removal retain their separate rules. At equal time the shared BuffTickOrExpiry notification precedes the explicit SettlementAndDeath interruption. No global ordering override is introduced.

## Burst and Underboss

- Ordinary Burst pays100 energy and starts15s cooldown at commitment. Seven uncapped AoE groups use rate-one source progress2300192/2382032/2468449/2547640/2634465/2716783/3015465us: UltraSkill1, UltraSkill2 five times, UltraSkill3. Each samples current live stats; no positive pause duration is inferred from rate-only Q rows.
- One Underboss generation applies at the separate3415870us source notification, after the seven direct hits, with the admitted Burst's captured effective talent. It lasts6 ObjectLifetime seconds, or9 with selected A2, with half-open expiry. A new generation replaces the old only when its summon actually applies. Interruption before birth creates no new summon; after birth its remaining independent lifetime survives. Death and field exit keep their separate policies below.
- Next-ability/Jump recovery is3582140us; switch requires3583025us and movement3756871us. Source execution advances through general recovery, so a later switch/movement gate needs an explicit authored wait. Interruptions retain paid energy, the15s cooldown and settled effects while cancelling unissued owner packets/unborn notifications. At exact birth time BuffTickOrExpiry application precedes the explicit SettlementAndDeath interruption.
- Summon creation and direct Q hits create no extra Cat damage, A1 energy or A5 stacks. An existing Cat does not coordinate those direct Q hits either.
- Lifetime is the summon budget. There is no independent eight-group counter and no three-charge auxiliary-minion pool. Reference guide's five-Normal/three-heavy recommendation is a practical rotation observation, not sufficient evidence for a universal input cap.
- Later supported Normal/Skill inputs select their native Cat packet group. Support, Critical Riposte and plunge use their owner-specific group bindings. Aim does not request a Cat attack. Waiting and another actor's attacks create no autonomous Cat packets.
- A group uses the summon generation's captured effective Burst talent. Current post-parent, coincident delivery is a selected approximation; no synthetic0.3s FIFO is imposed.
- Each delivery rechecks living owner/recipient, generation and half-open deadline. A field exit preserves the remaining Cat lifetime and eligible pending groups; death, target loss, replacement or expiry cancels affected deliveries.

## Follow-up accounting

- One performed Cat group is one follow-up attack for A1/A5, including groups containing multiple damage packets. Recipient multiplicity creates no extra callback. A surviving dispatch to an eligible living recipient counts even if subsequent integer HP rounding yields zero; no-recipient/cancelled groups do not count. This is an explicit interpretation of performing an attack.
- A1 grants2.5 raw self energy through recipient Charge Efficiency, exact carry/cap/overflow and one shared1s guard. Summoning cannot fund itself.
- A5 adds one2% base-ATK-scaled stack per group, cap10. The current15s refresh is a bounded duration model. Field exit/death and explicit combat exit clear the wearer buff and stack pool. Pending off-field or outside-combat groups cannot regrant it; re-entry creates no stacks. At half-open expiry the next group starts a fresh stack pool rather than reviving an expired count.
- A3 requires selected Effect3, live Authority and positive settled owner damage; it queues one Normal-scaled single-target follow-up with its own1s guard. Its own packet cannot recurse. A connected A3 packet can request A1/A5 through their shared owners.
- A4 adds100 percentage points to the Cat damage-bonus zone; it does not amplify direct Q. Resonance at six selected awakenings adds15 percentage points of general damage bonus.

## More Than Passionate

- A living Nanally unlocked at breakthrough2 enhances newly born party Blossom Buds to ten Vita Pistils, one per second with the first at birth+1s. No Authority, Underboss or awakening is required.
- Each Bud captures that schedule at creation. Nanally can remain off field; later switching or defeat does not rewrite existing Buds. New births recheck the living provider. A locked/defeated provider leaves the base five emissions at2s intervals.
- The published enhancement grounds count/cadence. Party membership at creation and captured eligibility are selected policies; the saved negative countdown value alone does not prove a timer. Duplicate providers do not stack.
- Cycle damage retains its actual captured owner, level and CI. Enhancement changes neither recipient settlement nor the ordinary three-Bud cap, oldest-generation replacement, Jiuyuan's separate cap/count extension or Iroi's copied-pistil owner.

## Fair Duel and Cycle callbacks

- Breakthrough4 unlocks Fair Duel while Nanally is living, active and has live Authority. Positive settled Esper Cycle damage by any party member requests one native `GE_Nanally010_Lv1_Damage` packet with an independent2s guard. Ordinary damage alone does not request it.
- Fair Duel uses effective Basic Attack talent: ATK coefficients0.600 at level1,1.028 at level8 and1.199 at level10. It retains both the actual Authority Skill command and actual triggering damage receipt as autonomous `coordinated_hit` parents, without creating another player input.
- Capture the triggering recipient if still living, otherwise the lowest-index living dummy. A later loss cancels the captured packet without retargeting. Positive-loss qualification, recipient fallback and ordered same-time dispatch are selected execution policies.
- Nanally-owned Cycle damage and connected Fair Duel can request A3 through its existing independent1s guard. Other actors' Cycle damage cannot directly request her A3; the resulting Fair Duel can. A3 cannot recursively request itself.
- Connected Fair Duel and actual Nanally-owned Hexed follow-ups each request A1/A5 once through their shared accounting; separate recipients do not duplicate these single-recipient callbacks. Ordinary reaction creation without damage requests neither.

## Public source observations

`.nanally.authority.active`, `.nanally.underboss.active`, `.nanally.mods.authority` and `.nanally.mods.underboss` read native half-open deadlines; known absent/expired modifiers return zero seconds. `.nanally.aiming.active` reads the native aim state. Queries change no state or RNG. There is no fabricated summon-charge query/preload.

Committed non-damaging player inputs retain their actual command for interruption.
Interrupting a paid Critical Dodge releases its recovery without refunding stamina
or revoking the already granted one-use Riposte permission; the next ordinary
input, field exit and death retain their existing consumption/removal rules.

## Acceptance and remaining scope

Rust tests in `crates/content/src/simulation/tests/nanally_*` and affected resource/equipment tests own exact numbers, callback units, expiry, field/death/generation and continuing energy checks. `source_queries.rs` checks two continuing E/Q/Normal cycles and all-field serialized native replay. Full native checks and the affected native/WASM/Worker transport smoke boundary apply to changes.

The [equipped A0 full-kit source](../../../../presets/native/nanally-full-kit.nte) runs two continuing periods. It reaches two funded Bursts, Normal/heavy/plunge/Aim/Riposte/Support, Raging Flames windows and stacks, qualified Cycle spending, 40 Bud packets and 10 Fair Duel packets. Four paid Dashes spend 60 stamina. Its 275 hits retain native aggregation and seeded replay.

Ultimate accounting is initial100 +252 whole granted = final100 +200 spent +52 overflow, with101/125 carry. Cycle accounting is initial0 +382 granted = final11 +200 spent +171 overflow, with53/250 carry. A level80/CI0 Bud packet against the declared DEF1080 dummy deals4500 even while Nanally is off field. Independent owner tests retain packet, recipient, contact/cancel, timing and awakening checks beyond this single-dummy A0 baseline. Current-artifact bounded WASM/actual Worker smoke applies at this mechanics exit.

This accepts the selected executable model, not frame-verified game fidelity or complete report/search coverage. Conservative playback, held/Dodge retention and input accounting, Cat dispatch/recipients and remaining owner hypotheses stay explicit and revisable. Native calibration and final browser acceptance remain their own queue tasks.
