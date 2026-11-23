# Spec Delta

## MODIFIED Requirements

### Requirement: Bounded execution

Execution SHALL enforce declared work and event limits and retain the actual stopping cause and available partial report. Defaults SHALL allow 60,000 script operations per trial, a maximum execution horizon of 3,600 seconds (3,600,000,000 $\mu\text{s}$), and 100,000 events.

#### Scenario: Event budget is exhausted
- **GIVEN** execution has consumed its event budget with another event pending
- **WHEN** advancement is requested
- **THEN** the limit is reported without settling an extra event

#### Scenario: Execution within extended limits
- **GIVEN** a scenario with duration up to 3,600 seconds and up to 100,000 events
- **WHEN** execution runs without overriding default safety limits
- **THEN** preflight checks and execution accept the scenario without rejecting the duration or event count

## ADDED Requirements

### Requirement: Readable execution limit diagnostics

Script operation, event and victory time-limit diagnostics SHALL identify the exhausted resource and configured numeric limit in readable units. Operation messages SHALL distinguish work from simulated combat time. A rotation ending before victory SHALL NOT claim the time limit was reached when it was not. Typed causes, source locations and incomplete-result semantics SHALL remain available through native and WASM execution.

#### Scenario: Script operation budget is exhausted
- **GIVEN** a trial consumes its configured script operation budget
- **WHEN** another operation is attempted
- **THEN** its diagnostic identifies the script operation limit and explains that it is independent of combat time

#### Scenario: Event budget is exhausted
- **GIVEN** execution consumes its configured event budget with another event pending
- **WHEN** advancement is requested
- **THEN** its diagnostic identifies the event-count limit and reports its configured value

#### Scenario: Victory time limit is reached
- **GIVEN** a victory run has living targets at its combat-time limit
- **WHEN** further advancement is required
- **THEN** its diagnostic identifies the combat-time limit in seconds and explains that victory was not reached

#### Scenario: Rotation ends before victory
- **GIVEN** a victory rotation and its pending effects end before the combat-time limit with living targets
- **WHEN** the run is finalized
- **THEN** its diagnostic explains that the rotation ended without victory rather than claiming that the time limit elapsed
