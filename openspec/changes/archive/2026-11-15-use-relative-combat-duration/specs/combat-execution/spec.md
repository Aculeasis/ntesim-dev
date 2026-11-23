# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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

Team, actor, source and target DPS SHALL use each run's actual relative elapsed seconds. Exact native analysis, experiments, comparisons, exports and Studio aggregates SHALL share that basis. An undefined rate SHALL remain unavailable without fabricated denominators or a success-only subset average.

#### Scenario: Damage within stopped time
- **GIVEN** a completed trial deals 120000 damage over 120 relative seconds and 140 absolute seconds
- **WHEN** its damage rates and contributions are calculated
- **THEN** team DPS is exactly 1000 and all actor, source and target contributions use the same 120-second denominator
- **AND** changing the displayed duration basis cannot change those rates

#### Scenario: Zero relative elapsed duration
- **GIVEN** all targets are defeated during a timer pause before any relative time has elapsed
- **WHEN** the completed combat is reported or compared
- **THEN** damage, absolute duration, pause provenance and completion remain available but relative DPS is explicitly unavailable
- **AND** no infinity, epsilon denominator or absolute-time fallback is presented as relative DPS
- **AND** comparisons or multi-seed DPS statistics requiring that undefined rate remain unavailable

### Requirement: Compact dual-clock observations

Detailed reports SHALL retain one absolute event stream and compact actual timer-pause intervals from which relative event times can be reconstructed. Batch observations SHALL retain both elapsed totals without requiring detailed timelines for every seed. Timer recording SHALL remain event-driven.

#### Scenario: Many trials with one inspectable sample
- **WHEN** a multi-seed run includes repeated timer pauses
- **THEN** each seed retains relative and absolute duration values for statistics, while only the existing detailed sample path retains its complete playback observations
- **AND** the report contains neither a duplicated relative event stream nor periodic frames created solely to sample the timer

#### Scenario: Restore and replay recorded clocks
- **WHEN** a detailed report is exported, restored and replayed
- **THEN** its intervals reconcile absolute minus paused time to relative elapsed time exactly, including a pause clipped by failure
- **AND** malformed or inconsistent clock data is rejected before replacing a valid displayed report
