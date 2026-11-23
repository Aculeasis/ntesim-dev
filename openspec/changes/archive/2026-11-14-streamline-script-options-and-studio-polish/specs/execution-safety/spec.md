# Spec Delta

## MODIFIED Requirements

### Requirement: Bounded execution

Execution SHALL enforce declared work and event limits and retain the actual stopping cause and available partial report. The default maximum execution horizon SHALL be 3,600 seconds (3,600,000,000 $\mu\text{s}$), and the default maximum event budget SHALL be 100,000 events.

#### Scenario: Event budget is exhausted
- **GIVEN** execution has consumed its event budget with another event pending
- **WHEN** advancement is requested
- **THEN** the limit is reported without settling an extra event

#### Scenario: Execution within extended limits
- **GIVEN** a scenario with duration up to 3,600 seconds and up to 100,000 events
- **WHEN** execution runs without overriding default safety limits
- **THEN** preflight checks and execution accept the scenario without rejecting the duration or event count
