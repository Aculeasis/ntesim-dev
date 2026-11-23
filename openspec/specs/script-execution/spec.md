# Live script execution

## Purpose

Defines live per-trial script interpretation, exact expressions, event-driven waits and source-bound execution diagnostics.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Live branch evaluation

Script conditions and read-only queries SHALL evaluate against each trial's current state without consulting future random draws.

#### Scenario: Resource changes before branch
- **GIVEN** an earlier action changes an owner resource
- **WHEN** a later condition queries that resource
- **THEN** the condition uses the updated live value

### Requirement: Event-driven waits

Authored waits SHALL advance through actual events and relevant deadlines on the absolute combat-time axis without fixed-rate readiness polling. A wait's seconds SHALL measure absolute elapsed time even while the relative mode timer is paused.

#### Scenario: Sparse wait interval
- **GIVEN** no event or transition deadline occurs before an explicit wait endpoint
- **WHEN** the wait executes
- **THEN** time advances to the endpoint without intermediate readiness polling

#### Scenario: Wait inside a timer pause
- **GIVEN** the mode timer remains paused throughout an authored `wait(1)`
- **WHEN** the wait completes without another stopping condition
- **THEN** absolute combat time has advanced by exactly one second and relative combat time has not advanced
- **AND** eligible friendly effects settle according to their own clock policies

### Requirement: Authored source identity

Execution, export and replay SHALL retain the authored script and source-bound input identities, including repeated occurrences and failed admissions.

#### Scenario: Repeated source range fails
- **GIVEN** a loop reaches the same source range more than once
- **WHEN** a later occurrence fails admission
- **THEN** the diagnostic identifies that occurrence and its actual input rather than an earlier repetition

### Requirement: Default critical policy and scenario options
The script compiler SHALL default the critical strike evaluation policy to stochastic RNG (`crit=random`). Scenarios requiring deterministic, non-random calculations SHALL declare `crit=fixed`. Scenario options SHALL NOT accept a `mode` parameter.

#### Scenario: Script omits crit option
- **GIVEN** a scenario script without an explicit `crit=` option
- **WHEN** the script options are compiled
- **THEN** `crit` evaluates to `random` critical policy
- **AND** stochastic crit projections are populated for configured actors

#### Scenario: Deterministic test declares fixed crit
- **GIVEN** a test script declares `crit=fixed`
- **WHEN** the script is executed
- **THEN** attacks evaluate without random critical strike rolls

#### Scenario: Legacy mode option is omitted
- **GIVEN** a scenario script omits a `mode` option
- **WHEN** the script options are compiled
- **THEN** compilation succeeds without requiring mode configuration

#### Scenario: Mode option is rejected
- **GIVEN** a scenario script contains an explicit `mode` option
- **WHEN** the script options are compiled
- **THEN** compilation fails with an unsupported option diagnostic

### Requirement: Flexible stop policy

The simulation SHALL support `stop=time` and `stop=victory`. Under `stop=time`, execution SHALL finish successfully at the authored relative duration or all-target defeat. Under `stop=victory`, execution SHALL ignore authored `duration` and continue to all-target defeat or a safety limit. Admission failures, unresolved energy gates and safety limits SHALL retain their diagnostics.

#### Scenario: Stop on target defeat under time policy
- **GIVEN** a scenario with `stop=time` where all targets reach 0 HP before the horizon
- **WHEN** the killing blow settles
- **THEN** execution terminates with success and completion marked true
- **AND** no error or dead-target action failure diagnostic is produced

#### Scenario: Stop on time horizon under time policy
- **GIVEN** a scenario with `stop=time` where targets remain alive at the duration endpoint
- **WHEN** the relative time horizon is reached
- **THEN** execution finishes cleanly at the authored relative duration

#### Scenario: Victory policy ignores duration
- **GIVEN** a scenario with `stop=victory` and an authored `duration=20s`
- **WHEN** targets remain alive at 20 seconds
- **THEN** simulation continues beyond 20 seconds until all targets are defeated or limits are exhausted

#### Scenario: Victory policy allows rotation duration access without declared duration
- **GIVEN** a scenario with `stop=victory` and no declared `duration`
- **WHEN** rotation code evaluates `.duration`
- **THEN** `.duration` evaluates successfully to the execution safety limit without diagnostic error
- **AND** that query does not turn the safety limit into an authored relative stop target

#### Scenario: Timer pause extends absolute execution
- **GIVEN** `stop=time duration=120`, living targets and exactly twenty seconds of mode-timer pause before completion
- **WHEN** execution reaches 120 relative seconds without a safety or admission failure
- **THEN** it completes at 140 absolute seconds and retains damage settled during the pause

#### Scenario: Exact endpoint and later pending damage
- **GIVEN** an admitted action has a hit at the absolute instant the relative horizon is first reached and another hit one microsecond later
- **WHEN** the horizon settles
- **THEN** the endpoint hit and its equal-time causal children are included and the later hit remains pending
- **AND** a new player input at that completed boundary cannot extend the run by opening a timer pause

#### Scenario: Time stop during a command
- **GIVEN** an admitted action or ordinary authored wait starts before the relative duration and ends after it
- **WHEN** stop=time reaches that duration
- **THEN** execution completes successfully at the exact endpoint, including eligible equal-time causal effects
- **AND** later effects remain pending without contributing damage or requiring the command to finish
- **AND** no subsequent source input or expression is evaluated, including after a truncated implicit switch

### Requirement: Relative live combat time

Live `.time` SHALL return exact relative mode-timer seconds; authored `.duration` SHALL return the relative limit under `stop=time`. The absolute causal scheduler SHALL apply the declared ordinary-world and exception clock policies. Live cooldown, timed-modifier and owner-resource queries SHALL use those same paused or progressing clocks without deriving lifetime from absolute timestamps alone.

#### Scenario: Live query during Time Stop
- **GIVEN** absolute time is 45 seconds and the recorded union of elapsed timer pauses is ten seconds
- **WHEN** a live branch evaluates `.time`
- **THEN** it observes exactly 35 seconds using current settled state
- **AND** advancing through eligible attacks during the same pause changes absolute timestamps without advancing `.time`

#### Scenario: Continuing rotation uses the relative limit
- **GIVEN** an authored rotation checks `.time < .duration` under `stop=time`
- **WHEN** an actual timer pause extends the absolute run
- **THEN** later iterations observe relative elapsed time without resetting resources, cooldowns, pending effects or RNG at the pause boundary

#### Scenario: Cooldown and modifier queries during a stop
- **GIVEN** an ordinary Skill cooldown has three seconds remaining and an independent modifier has two seconds remaining
- **WHEN** permitted source execution reads their cooldown and `.mods` fields before and after one absolute second inside a continuing pause
- **THEN** the queries retain three and two seconds respectively, consistently with native readiness and effect delivery
- **AND** an absolute wait alone cannot make the frozen Skill ready or the modifier expire

#### Scenario: Absolute declaration after the relative target
- **GIVEN** a 20-relative-second run whose executed pause extends its absolute endpoint beyond 25 seconds
- **WHEN** the script declares an interruption at absolute 25 seconds within the independent absolute ceiling
- **THEN** configuration accepts it and execution delivers it at that absolute instant
- **AND** declarations beyond the caller's absolute ceiling remain rejected independently of the relative target

### Requirement: Explicit purchased Support Skill level

Every script character declaration SHALL require one explicit `qte=<integer>` purchased Support Skill level within the legal per-character progression range. The existing `talent` tuple SHALL continue to declare Normal, Skill and Burst. Compilation SHALL reject missing, duplicate or invalid QTE values with a source diagnostic.

#### Scenario: Author a distinct QTE level
- **GIVEN** a character declaration has legal `talent=8,8,8 qte=6`
- **WHEN** the script is compiled and executed or exported
- **THEN** the native scenario retains purchased QTE level 6 separately from the three purchased talent levels
- **AND** the effective QTE uses that level and the applicable owner progression bonuses

#### Scenario: Omit the QTE level
- **GIVEN** a character declares `talent=8,8,8` and omits `qte`
- **WHEN** the script is compiled
- **THEN** compilation fails with a diagnostic identifying the character declaration and missing field

#### Scenario: Duplicate or illegal QTE level
- **WHEN** a character declaration repeats `qte` or supplies a malformed or out-of-range QTE level
- **THEN** compilation fails with a diagnostic identifying the invalid declaration
