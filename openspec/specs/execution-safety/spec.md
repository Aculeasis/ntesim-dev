# Execution safety

## Purpose

Defines bounded execution, checked arithmetic, causal ordering and auditable partial failure without fabricated success.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Bounded execution

Execution SHALL enforce declared work, event and absolute elapsed-time limits independently of relative duration, retaining the stopping cause and partial report. Defaults SHALL allow 60000 script operations, 100000 events and a 3600-second absolute execution ceiling. Relative duration requests up to 3600 seconds SHALL pass preflight, subject to those independent runtime limits.

#### Scenario: Event budget is exhausted
- **GIVEN** execution has consumed its event budget with another event pending
- **WHEN** advancement is requested
- **THEN** the limit is reported without settling an extra event

#### Scenario: Execution within extended limits
- **GIVEN** a scenario with relative duration up to 3,600 seconds and up to 100,000 events
- **WHEN** execution runs without overriding default safety limits
- **THEN** preflight checks and execution accept the scenario without rejecting the declared duration or event count
- **AND** a reached independent absolute safety ceiling remains an incomplete runtime stop if the relative target has not been achieved

#### Scenario: Relative timer does not progress
- **GIVEN** a timer pause keeps relative time below the authored limit
- **WHEN** absolute execution reaches its configured ceiling or an event or operation budget is exhausted
- **THEN** execution stops with that actual safety cause and preserves both elapsed totals
- **AND** it is not reported as successful relative-duration completion

#### Scenario: Long sparse pause
- **GIVEN** an active timer pause has a known release event and a long interval with no intervening work
- **WHEN** execution advances through that interval
- **THEN** it reaches the next actual event or absolute safety boundary without polling the relative clock at fixed time steps

### Requirement: Atomic rejected operations

A rejected operation SHALL preserve the state covered by its transaction and retain previously committed events for partial replay.

#### Scenario: Rejected resource spend
- **GIVEN** an operation requires more resources than are available
- **WHEN** admission fails
- **THEN** its costs and state changes are not partially applied and previous settled effects remain available

### Requirement: Readable execution limit diagnostics

Script operation, event and absolute time-limit diagnostics SHALL identify the exhausted resource and numeric limit in readable units. Diagnostics SHALL distinguish absolute safety limits from relative completion and script work. Early rotation exhaustion SHALL NOT claim a time limit was reached. Typed causes, source locations and incomplete status SHALL survive native and WASM transport.

#### Scenario: Script operation budget is exhausted
- **GIVEN** a trial consumes its configured script operation budget
- **WHEN** another operation is attempted
- **THEN** its diagnostic identifies the script operation limit and explains that it is independent of combat time

#### Scenario: Event budget is exhausted
- **GIVEN** execution consumes its configured event budget with another event pending
- **WHEN** advancement is requested
- **THEN** its diagnostic identifies the event-count limit and reports its configured value

#### Scenario: Victory time limit is reached
- **GIVEN** a victory run has living targets at its absolute combat-time safety limit
- **WHEN** further advancement is required
- **THEN** its diagnostic identifies the absolute combat-time limit in seconds and explains that victory was not reached

#### Scenario: Rotation ends before victory
- **GIVEN** a victory rotation and its pending effects end before the combat-time limit with living targets
- **WHEN** the run is finalized
- **THEN** its diagnostic explains that the rotation ended without victory rather than claiming that the time limit elapsed

#### Scenario: Absolute cap precedes relative completion
- **GIVEN** `stop=time` has not reached its relative duration when the absolute safety cap is exhausted
- **WHEN** the partial run is inspected
- **THEN** the diagnostic names the absolute cap and shows the relative progress separately from that cap
- **AND** the result remains ineligible as a complete comparison
