# Combat execution and reports

## Purpose

Defines native combat requests, legal action admission, explicit waiting, causal reports and incomplete execution boundaries.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Failed action admission

An invalid ordinary action SHALL halt with its actual admission diagnostic, preserving costs, windows and state instead of adding an implicit wait.

#### Scenario: Skill is on cooldown
- **GIVEN** an actor has a Skill cooldown remaining
- **WHEN** an ordinary Skill input is attempted without an authored wait
- **THEN** admission fails immediately and no elapsed wait is inserted

### Requirement: Horizon and partial reports

Combat reports SHALL retain exact relative and absolute elapsed combat time, the declared relative limit, settled receipts, pending effects and failures. Event timestamps SHALL retain absolute causal time. Incomplete runs SHALL remain distinguishable from successful complete results.

#### Scenario: Delayed hit beyond horizon
- **GIVEN** an admitted attack schedules a hit after the declared relative horizon's absolute completion instant
- **WHEN** the horizon is reached
- **THEN** the hit remains pending and contributes no settled damage

#### Scenario: Defeat or failure during an active pause
- **GIVEN** execution ends before a currently active mode-timer pause would naturally finish
- **WHEN** its report is finalized
- **THEN** paused duration is clipped to actual absolute elapsed time and both elapsed totals describe only executed combat
- **AND** the original defeat or failure status and pending effects remain inspectable

### Requirement: Explicit incoming assumptions

Scenario incoming damage and successful critical-dodge or parry assumptions SHALL retain their declared provenance and native owner costs, timing and effects.

#### Scenario: Authored defensive assumption
- **GIVEN** a supported owner receives a declared successful defensive input
- **WHEN** that input executes
- **THEN** its normal resource and window rules apply and the assumption remains visible

### Requirement: Normalized report provenance and tables

Combat reports SHALL store unique evidence items, packet metadata templates, buff definitions, and resolved model assumption descriptions in shared report catalogs rather than repeating static prose or identical metadata objects per command or hit.

#### Scenario: Repeated ability hits and talent projections
- **GIVEN** a combat scenario executes multiple actions and hits from the same ability and damage effect
- **WHEN** the native report is serialized
- **THEN** each unique evidence record, metadata template, and assumption description is stored once in its shared catalog and referenced by identifier or index
- **AND** identical static action ledger and talent projection entries group their matching `command_indices`

### Requirement: Delta state timeline frames

Recorded `state_timeline` observation frames SHALL retain complete `Initial` and `Final` boundary snapshots and emit incremental actor field deltas and buff diff operations on intermediate `Event` boundaries.

#### Scenario: Reconstructing timeline state across event boundaries
- **GIVEN** an intermediate event updates actor energy carry and adds or refreshes a buff stack
- **WHEN** the resulting `state_timeline` is validated and replayed sequentially from `Initial` to `Final`
- **THEN** intermediate `Event` frames record only changed actor fields and buff `upsert`/`remove` operations referencing `buff_catalog`
- **AND** the reconstructed final actor and buff states exactly match the `Final` snapshot and final ledgers

### Requirement: Data-only structured trace events

Trace entries SHALL record language-neutral structured event kinds, references, and numeric payloads without human-readable prose strings, and SHALL group simultaneous hit-driven energy grants sharing the same source basis.

#### Scenario: Multi-recipient hit energy fanout
- **GIVEN** one settled hit distributes ordinary energy across four party members at the same timestamp
- **WHEN** trace entries are emitted
- **THEN** the grant is recorded as one grouped fanout trace event containing all recipient ledgers without per-recipient prose sentences
- **AND** energy conservation and replay ledgers reconcile identically across all recipients

### Requirement: Canonical exact rational strings

Exact rational values in combat requests, reports, and wire payloads SHALL serialize and deserialize canonically as `"<numerator>/<denominator>"` decimal strings without object wrappers or loss of arbitrary integer precision.

#### Scenario: Serializing and parsing exact damage and rates
- **GIVEN** a scenario contains fractional multipliers and exact damage exceeding JavaScript safe-integer bounds
- **WHEN** the request is parsed and the report is serialized to JSON
- **THEN** exact rationals use canonical `"<numerator>/<denominator>"` strings retaining all numerator and denominator digits
- **AND** `ExactFraction`, `Ratio`, and `WideFraction` values reject legacy `{numerator, denominator}` object shapes

### Requirement: Bounded aggregate defense ignore

Supported packet, equipment and team DEF-ignore contributions SHALL combine additively with an effective maximum of 100% before native direct or Break damage evaluation. This bounded model SHALL preserve exact arithmetic, source applicability and existing unsupported-dependency diagnostics; it SHALL NOT relax individual formula-request validation.

#### Scenario: Zero execution with Lost Radiance and Iroi support
- **GIVEN** Zero's execution packet supplies 75% DEF ignore, Lost Radiance supplies 25% and an applicable Iroi team buff supplies another 5%
- **WHEN** the packet is evaluated against an authored dummy
- **THEN** effective DEF ignore is exactly 100% and the supported composition completes without an out-of-range arithmetic failure
- **AND** the packet does not gain a defense multiplier greater than one

#### Scenario: Aggregate below the bound
- **GIVEN** applicable DEF-ignore sources total less than 100%
- **WHEN** direct or Break damage is evaluated
- **THEN** the exact summed contribution is retained without rounding or rescaling

#### Scenario: Invalid individual formula request
- **WHEN** an individual exact damage-formula request specifies DEF ignore greater than 100%
- **THEN** formula validation continues to reject that request

### Requirement: Bounded custom model declarations

Canonical engine hypothesis declarations SHALL be admitted independently of the 32-entry allowance for custom declarations. Unknown identities and mismatched canonical descriptions SHALL consume the custom allowance. Existing per-entry size bounds, strict-mode acceptance and conflicting reached-hypothesis diagnostics SHALL remain enforced.

#### Scenario: Four-member party uses many canonical models
- **GIVEN** automatic content projection for a legal party produces more than 32 canonical hypothesis declarations
- **WHEN** its request is validated
- **THEN** the canonical declarations do not cause a metadata count rejection and remain in the execution provenance

#### Scenario: Custom declarations remain bounded
- **GIVEN** a request contains canonical declarations and 32 custom declarations
- **WHEN** one more unknown or mismatched declaration is added
- **THEN** validation rejects the excess custom declaration

### Requirement: Explicit mode-timer pause lifecycle

All supported Ultimates SHALL pause the mode timer during their cinematic phase, using explicit content-owned start and release transitions with retained provenance and assumptions. Relative time SHALL equal absolute time minus the union of elapsed timer-pause intervals. Pausing enemies or a local actor SHALL NOT by itself declare a mode-timer pause.

#### Scenario: Overlapping timer pauses
- **GIVEN** timer pauses occupy absolute intervals `[1s,4s)` and `[2s,5s)`
- **WHEN** combat is observed at absolute 7 seconds
- **THEN** total paused time is four seconds and relative elapsed time is three seconds
- **AND** ending one overlapping owner does not resume the timer while another owner still pauses it

#### Scenario: Ordinary Ultimate cinematics pause the timer
- **GIVEN** a supported character other than Hotori legally commits an Ultimate
- **WHEN** its modeled cinematic pause begins and reaches the actual configured release transition
- **THEN** relative mode-timer time stays fixed during that interval while absolute time advances
- **AND** the pause is bound for every supported Ultimate, not only Hotori
- **AND** subsequent buff or summon lifetime does not prolong the cinematic pause without a separate explicit timer-pause rule

#### Scenario: Ordinary Ultimate interruption or defeat
- **GIVEN** an ordinary Ultimate has opened its cinematic timer pause
- **WHEN** cancellation, interruption or owner defeat ends that phase before its natural release
- **THEN** the actual pause closes at that transition without waiting for a stale full-duration deadline
- **AND** already dispatched friendly effects retain their own progress and lifetime policies

#### Scenario: Hotori setup and Time Stop form a connected pause
- **GIVEN** a legally committed World's Tide using the declared Hotori timer model
- **WHEN** its setup transitions into Time Stop and then ends through finisher delivery, resource expiry or owner defeat
- **THEN** the mode timer remains paused from commitment through the actual end and resumes once
- **AND** interruption before Time Stop entry closes only the actually executed setup pause
- **AND** the model assumption remains visible independently of the enemy and Vita Bud pause policies

#### Scenario: Rejected cast and stale cleanup
- **WHEN** World's Tide admission fails or an obsolete generation delivers a cleanup callback
- **THEN** it cannot open an unearned pause, close a newer pause or subtract paused duration twice

### Requirement: Paused ordinary world timers

Ordinary cooldowns, proc internal cooldowns, stack decay, independent timed buffs/shields/debuffs, status lifetimes, passive recovery and timed resource windows SHALL pause during Ultimate cinematics and active Hotori Time Stop. Exact remaining duration and fractional progress SHALL be preserved. Instantaneous supported payments, grants, death and removal SHALL retain their causal rules.

#### Scenario: Buff and cooldown retain different remainders
- **GIVEN** an independent buff has two seconds remaining and an ally Skill cooldown has three seconds remaining
- **WHEN** a ten-absolute-second pause begins and ends without another removal condition
- **THEN** the buff still has exactly two seconds and the cooldown exactly three seconds remaining
- **AND** ordinary time-based resource recovery has gained nothing during the paused interval

#### Scenario: Proc cooldown cannot be farmed during a stop
- **GIVEN** a weapon proc's internal cooldown has one second remaining
- **WHEN** eligible direct attacks land at distinct absolute timestamps during the same frozen world-time interval
- **THEN** those attacks cannot expire that cooldown or repeatedly obtain a newly ready proc through elapsed absolute time alone

#### Scenario: Timed effect is created inside a pause
- **WHEN** an eligible progressing action applies an independent five-second buff during a pause
- **THEN** the buff applies immediately with its full five-second remaining lifetime
- **AND** its lifetime starts advancing when its world clock resumes, without charging time before creation

#### Scenario: Source removal is immediate
- **GIVEN** an effect's duration is paused but its declared source or recipient is defeated, replaced or dispelled
- **WHEN** the removal condition settles
- **THEN** its actual removal occurs immediately and release does not restore it

### Requirement: Paused periodic phases

Ordinary periodic damage/healing SHALL preserve both remaining phase and lifetime during cinematic and Hotori pauses, including carriers applied during the pause. Refresh and stack rules SHALL remain owner-specific. A supported instantaneous application or reaction SHALL NOT be classified as a periodic tick solely because it deals damage during stopped time.

#### Scenario: Resume a partially elapsed tick interval
- **GIVEN** Scorch has 0.25 seconds until its next tick and two seconds of remaining lifetime
- **WHEN** a four-second absolute pause begins and later ends
- **THEN** no periodic Scorch tick occurs during the pause and its next tick is 0.25 advancing world seconds after release
- **AND** its remaining lifetime at release is still two seconds, without a catch-up burst

#### Scenario: Scorch is applied by a replay during Time Stop
- **WHEN** a permitted replay applies or refreshes Scorch during Hotori Time Stop
- **THEN** the selected periodic phase and status lifetime remain paused until release
- **AND** a separately qualified instantaneous reaction can settle once under its existing deduplication rules without authorizing ongoing periodic ticks

### Requirement: Progressing action exceptions

The current cinematic's own action context and Hotori's active Katana, reserved-stop drain and replay direct-action contexts SHALL progress through their supported pause. Unrelated actors' recovery and delayed deliveries SHALL pause. Exceptions SHALL retain actual generation and ownership, compose other applicable pauses and SHALL NOT exempt every future effect from the same owner.

#### Scenario: Cinematic can finish while world timers stand still
- **WHEN** a legal ordinary Ultimate executes its cinematic
- **THEN** its finite direct packets and release event can settle while ordinary cooldowns, statuses and unrelated delayed packets retain their remaining time

#### Scenario: Replay creates a persistent producer
- **WHEN** a Hotori replay dispatches its direct packets and deploys a persistent field
- **THEN** direct packets follow the permitted replay timeline, while field lifetime and periodic production follow the separate object policy
- **AND** actual ally identity, caps, costs and replay deduplication remain preserved

### Requirement: Independent deployed-object lifetime and activity

Deployed field/summon lifetimes and autonomous attack/heal phases SHALL pause during ordinary cinematics. During active Hotori Time Stop, their lifetimes SHALL advance while autonomous production pauses, except for eligible B2 Vita Buds. Finite counts SHALL be conserved, and expiry or replacement SHALL cancel parked work without resurrection or catch-up.

#### Scenario: A field expires while unable to heal
- **GIVEN** a deployed healing field has two seconds of object lifetime remaining
- **WHEN** Hotori active Time Stop lasts four absolute seconds
- **THEN** the field expires after two seconds without periodic healing and its source-dependent effects end with it
- **AND** release does not deliver suppressed heals or restart that field

#### Scenario: B2 bud uses its normal remaining budget
- **GIVEN** a Vita Bud has a finite remaining lifetime and volley count and living Hotori has B2
- **WHEN** active Time Stop begins
- **THEN** eligible emissions continue, actual emitted volleys consume their normal count and the lifetime advances
- **AND** Time Stop supplies neither extra volleys nor free lifetime

#### Scenario: Bud without B2 can expire without firing
- **GIVEN** a bud has two seconds of lifetime remaining and Hotori has not unlocked B2
- **WHEN** active Time Stop lasts four absolute seconds
- **THEN** the bud emits no volley and expires at its actual remaining object-lifetime deadline
- **AND** its unused volleys are discarded without a release backlog

#### Scenario: An ordinary cinematic dominates B2
- **WHEN** an ordinary Ultimate cinematic pauses a live B2-eligible bud
- **THEN** both its lifetime and emission phase pause until cinematic release
- **AND** B2 alone cannot bypass that separate cinematic pause

### Requirement: Hotori resource and Charge clock policy

Hotori's passive Timepiece recovery, recording, Skill rewind settlement and A3 refund/ICD SHALL use paused world time. Her active reserved-stop drain SHALL progress independently of local hitlag and end with the actual mode. Charge reactions during active Time Stop SHALL grant no energy or deferred energy, while unrelated supported instantaneous packet grants SHALL keep their own rules.

#### Scenario: Ally cinematic extends the recording opportunity
- **GIVEN** Hotori has three seconds remaining in her recording window and two seconds remaining in a rewind/refund interval
- **WHEN** an ally's four-second cinematic executes
- **THEN** both timer remainders are unchanged and passive Timepiece recovery gains nothing from those four seconds
- **AND** qualifying ally inputs after release can still use the actual retained recording window

#### Scenario: Charge during the bud exception
- **WHEN** a permitted bud hit qualifies for Charge during active Hotori Time Stop
- **THEN** its reaction may be observed but Charge grants zero energy and accumulates no grant for release
- **AND** no grant-dependent guard starts merely from that suppressed grant

#### Scenario: Finisher closes the active drain
- **WHEN** a legal finisher ends Time Stop before the reserved budget expires
- **THEN** its drain ends at actual finisher delivery, the remaining reservation follows the existing discard rule and paused world timers resume with retained remainders

### Requirement: Disclosed deterministic timer assumptions

Reached timers without a measured native pause rule SHALL use the declared shared default or explicit modeled owner fallback. Reports SHALL retain policy identity and source-versus-assumption provenance. Missing measurements covered by a selected policy SHALL NOT alone make a run incomplete or imply verified native fidelity; unsupported mechanics and execution failures SHALL remain visible.

#### Scenario: Cinematic release field was not serialized
- **WHEN** a supported Ultimate lacks a sourced cinematic-release event
- **THEN** it uses its explicitly declared owner action-completion fallback and records that choice as a modeled rule
- **AND** it cannot silently omit the pause or require an unavailable recording before the bounded model can execute

#### Scenario: Timer remainder is inspected during a pause
- **WHEN** native observations or replay inspection query a paused cooldown or timed modifier
- **THEN** they expose the same remaining value and clock basis used by admission and effect delivery
- **AND** advancing an absolute cursor alone cannot expire a paused world timer

### Requirement: Relative damage-per-second accounting

Canonical team, actor, source and target DPS SHALL use each run's actual relative elapsed seconds in exact native analysis, experiments and comparisons. Studio SHALL additionally receive native absolute DPS for report selection without changing canonical relative values or eligibility. An undefined rate SHALL remain unavailable without fabricated denominators or a success-only subset average. Exports SHALL retain the basis of each value.

#### Scenario: Damage within stopped time
- **GIVEN** a completed trial deals 120000 damage over 120 relative seconds and 140 absolute seconds
- **WHEN** its damage rates and contributions are calculated
- **THEN** canonical team DPS is exactly 1000 and all canonical actor, source and target contributions use the same 120-second denominator
- **AND** the absolute report view uses the 140-second denominator for its displayed rates without changing canonical relative DPS

#### Scenario: Zero relative elapsed duration
- **GIVEN** all targets are defeated during a timer pause before any relative time has elapsed
- **WHEN** the completed combat is reported or compared
- **THEN** damage, absolute duration, pause provenance and completion remain available but relative DPS is explicitly unavailable
- **AND** no infinity, epsilon denominator or absolute-time fallback is presented as relative DPS
- **AND** comparisons or multi-seed DPS statistics requiring that undefined rate remain unavailable
- **AND** a defined absolute report rate does not restore relative comparison eligibility

### Requirement: Compact dual-clock observations

Detailed reports SHALL retain one absolute event stream and compact actual timer-pause intervals from which relative event times can be reconstructed. Batch observations SHALL retain both elapsed totals and native summary values for both bases without detailed timelines for every seed. Additional summary data SHALL NOT duplicate detailed reports, event streams, timeline series or static contribution identities. Timer recording SHALL remain event-driven.

#### Scenario: Many trials with one inspectable sample
- **WHEN** a multi-seed run includes repeated timer pauses
- **THEN** each seed retains relative and absolute duration and summary values for statistics, while only the existing detailed sample path retains its complete playback observations
- **AND** the report contains neither a duplicated relative event stream nor periodic frames created solely to sample the timer or shield capacity
- **AND** a basis selection requires no new combat execution or relative damage timeline

#### Scenario: Restore and replay recorded clocks
- **WHEN** a detailed report is exported, restored and replayed
- **THEN** its intervals reconcile absolute minus paused time to relative elapsed time exactly, including a pause clipped by failure
- **AND** malformed or inconsistent clock data is rejected before replacing a valid displayed report

### Requirement: Native report summary time bases

Native report summaries SHALL calculate DPS, retained/granted EPS and gross HPS from the same settled totals divided by each trial's actual relative or absolute seconds. Native SHP SHALL weight active living actor capacity by advancing time in the corresponding basis. Statistics SHALL summarize per-trial values in each basis. A zero denominator SHALL make that basis-dependent value unavailable, even for zero activity.

#### Scenario: Per-trial rates precede aggregation
- **GIVEN** two completed trials each deal 1200 damage, last 10 relative seconds, and last 20 and 30 absolute seconds respectively
- **WHEN** native summary statistics are calculated
- **THEN** relative mean DPS is 120 and absolute mean DPS is 50, from absolute per-trial rates 60 and 40
- **AND** the absolute mean is not calculated as mean damage divided by mean absolute duration
- **AND** minima, maxima, standard deviation and quartiles use the corresponding per-trial values

#### Scenario: Relative shield weighting excludes stopped intervals
- **GIVEN** a trial lasts 10 relative seconds and 20 absolute seconds, with 10000 available shield HP only during its 10 paused absolute seconds
- **WHEN** native summary observations are finalized
- **THEN** absolute SHP is 5000 HP and relative SHP is zero HP
- **AND** grant, depletion, replacement, active-recipient switch, defeat and expiry retain their actual causal boundaries

#### Scenario: Zero relative support denominator
- **GIVEN** a completed trial has positive absolute duration and zero relative duration
- **WHEN** support summary observations are finalized
- **THEN** relative EPS, granted EPS, HPS and SHP are unavailable and supported absolute values remain inspectable
- **AND** an aggregate requiring an unavailable value does not omit that seed or substitute an epsilon denominator

#### Scenario: No pause control
- **WHEN** a supported completed trial contains no mode-timer pause and has positive duration
- **THEN** both summary bases agree for DPS, EPS, HPS and SHP
- **AND** changing the report basis cannot change damage, grants, healing, admitted inputs, event order or completion

### Requirement: Reaction-qualified Support Skill entry

An admitted real switch that triggers a qualifying reaction SHALL automatically attempt the incoming character's Support Skill once. Support Skills SHALL have no independent cooldown. Explicit and input-driven switches SHALL use the same rules and spend the outgoing Cycle once on a standard reaction switch. Support readiness SHALL use the same qualification and action guards without changing combat state.

#### Scenario: Consecutive qualified entries
- **GIVEN** the party can fund two qualifying reaction switches into the same character less than 15 seconds apart and all switch and action locks permit them
- **WHEN** both switches are admitted
- **THEN** each triggers its reaction on its locked target, pays the outgoing Cycle cost once and admits one incoming Support Skill
- **AND** elapsed time since the first Support Skill adds no cooldown gate to the second

#### Scenario: Unqualified entry or same-character switch
- **GIVEN** a switch cannot trigger a qualifying reaction, or a command selects the actor already on field
- **WHEN** the command is processed
- **THEN** it produces no Support Skill, Support cast effects or reaction cost

#### Scenario: Read Support availability
- **GIVEN** a pending choice of incoming actor and the settled state used for switch qualification
- **WHEN** the script inspects Support readiness
- **THEN** readiness reflects the same qualification and incoming action guards used for the corresponding switch
- **AND** inspection changes no Cycle, target state, pending effects or RNG

### Requirement: Lacrimosa A6 reaction entry

Selected Lacrimosa A6 SHALL arm a five-second reaction entry after Devilish Gift. A switch into Lacrimosa within that window that can trigger Nova or Scorch SHALL trigger the reaction, including eligible Discord, and admit her QTE without spending outgoing Cycle. This entry SHALL consume the window and have its own five-second internal cooldown.

#### Scenario: Armed Lacrimosa entry with an empty outgoing pool
- **GIVEN** Lacrimosa has selected A6, Devilish Gift has armed the reaction window and the outgoing actor has zero Cycle
- **WHEN** a legal switch into Lacrimosa during the window qualifies for Nova or Scorch on its living target
- **THEN** the reaction and incoming QTE each occur once and the outgoing Cycle stays at zero
- **AND** an overlap that qualifies for Discord settles through the standard reaction rules

#### Scenario: Armed Lacrimosa entry with a full outgoing pool
- **WHEN** the same A6 entry qualifies and the outgoing actor has full Cycle
- **THEN** A6 takes priority, the reaction and Support Skill each occur once and the full outgoing pool is preserved

#### Scenario: Unarmed or expired Lacrimosa entry
- **GIVEN** Lacrimosa has no live A6 window
- **WHEN** an outgoing actor with insufficient Cycle switches to her
- **THEN** it is an ordinary switch with no reaction or Support Skill

#### Scenario: Standard reaction during the A6 internal cooldown
- **GIVEN** Lacrimosa has used her free entry in the last five seconds and the outgoing actor has sufficient Cycle
- **WHEN** the outgoing actor performs a qualifying standard reaction switch into Lacrimosa
- **THEN** the standard reaction spends Cycle and Lacrimosa casts her QTE

### Requirement: Complete owner Support Skill effects

Every supported character SHALL resolve its Support Skill at the effective QTE level with its owner-specific action timing, packets, recipients, resource effects and follow-ups. The engine SHALL retain each effect's source, causal order and owner-specific level rules. An unresolved reached dependency SHALL remain visible and prevent the affected run from being counted as a successful comparison.

#### Scenario: Independently leveled damage and healing
- **GIVEN** a healing Support Skill with damage and heal components that scale with its QTE talent
- **WHEN** a qualified switch admits the skill
- **THEN** its damage and healing use the effective QTE level with their own target rules and delivery times
- **AND** any independently scaled follow-up retains its own talent and source

#### Scenario: Owner-specific Resonance 3
- **GIVEN** two characters with the same purchased QTE level and three selected awakening effects
- **WHEN** one character's Resonance 3 lists QTE and the other's Resonance 3 lists other talents
- **THEN** the first receives its specified QTE upgrade within the legal effective range and the second uses its purchased QTE level

### Requirement: Ordered automatic field-entry actions

Field entry SHALL start no ordinary attack on its own. Earned character-specific entry actions SHALL retain their own prerequisites and timing. When both Support and the regular Phantom Step are available on Daffodil's entry, Support SHALL play its full animation and Phantom Step SHALL start automatically at its end.

#### Scenario: Plain switch into Haniel, Hathor or Edgar
- **GIVEN** the incoming character has no qualified Support or other earned entry action
- **WHEN** the switch is admitted and the player issues no attack input
- **THEN** the character enters the field with no automatic ordinary attack, ordinary attack effects or ordinary attack recovery

#### Scenario: Daffodil has both QTE and Phantom Step
- **GIVEN** a qualifying reaction switch into Daffodil with an available regular Phantom Step
- **WHEN** the switch is admitted
- **THEN** Daffodil begins QTE and its full animation runs before Phantom Step starts
- **AND** Phantom Step starts at that transition without another input or switch
- **AND** both actions retain their own packets and timing, and Phantom Step spends its entry charge once

#### Scenario: Only one Daffodil action is available
- **WHEN** Daffodil enters with only qualified Support or only an available Phantom Step
- **THEN** the available action executes automatically with its own timing and costs
