# Spec Delta

## ADDED Requirements

### Requirement: Default critical policy and scenario options
The script compiler SHALL default the critical strike evaluation policy to stochastic RNG (`crit=random`). Scenarios requiring deterministic, non-random calculations SHALL declare `crit=fixed`. Scenarios SHALL NOT be required to specify `mode=research`.

#### Scenario: Script omits crit option
- **GIVEN** a scenario script without an explicit `crit=` option
- **WHEN** the script options are compiled
- **THEN** `crit` evaluates to `random` critical policy
- **AND** research crit projections are populated for configured actors

#### Scenario: Deterministic test declares fixed crit
- **GIVEN** a test script declares `crit=fixed`
- **WHEN** the script is executed
- **THEN** attacks evaluate without random critical strike rolls

#### Scenario: Legacy mode option is omitted
- **GIVEN** a scenario script omits `mode=research`
- **WHEN** the script options are compiled
- **THEN** compilation succeeds and defaults mode to research

### Requirement: Flexible stop policy
The simulation SHALL support `stop=time` and `stop=victory` policies. Under `stop=time`, simulation execution SHALL terminate cleanly and successfully when either the time horizon expires OR all targets are defeated. Under `stop=victory`, simulation execution SHALL ignore authored `duration` and continue until all targets are defeated or an execution safety limit is reached.

#### Scenario: Stop on target defeat under time policy
- **GIVEN** a scenario with `stop=time` where all targets reach 0 HP before the horizon
- **WHEN** the killing blow settles
- **THEN** execution terminates with success and completion marked true
- **AND** no error or dead-target action failure diagnostic is produced

#### Scenario: Stop on time horizon under time policy
- **GIVEN** a scenario with `stop=time` where targets remain alive at the duration endpoint
- **WHEN** the time horizon is reached
- **THEN** execution finishes cleanly at the authored duration

#### Scenario: Victory policy ignores duration
- **GIVEN** a scenario with `stop=victory` and an authored `duration=20s`
- **WHEN** targets remain alive at 20 seconds
- **THEN** simulation continues beyond 20 seconds until all targets are defeated or limits are exhausted

#### Scenario: Victory policy allows rotation duration access without declared duration
- **GIVEN** a scenario with `stop=victory` and no declared `duration`
- **WHEN** rotation code evaluates `.duration`
- **THEN** `.duration` evaluates successfully to the execution safety limit without diagnostic error
