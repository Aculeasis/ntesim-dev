# Spec Delta: script-execution

## MODIFIED Requirements

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
