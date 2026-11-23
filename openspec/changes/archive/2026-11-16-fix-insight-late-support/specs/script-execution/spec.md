## MODIFIED Requirements

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
