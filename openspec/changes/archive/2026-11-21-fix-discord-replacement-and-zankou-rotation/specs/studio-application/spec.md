# Spec Delta

## MODIFIED Requirements

### Requirement: Validated continuing curated rotations

Each delivered new team SHALL have a complete supported shared-engine result using 1,000 iterations over 120 relative combat seconds by default. Scripts SHALL preserve continuing resources, cooldowns and owner state. Exceptions SHALL retain explicit conditions. Research choices SHALL retain source references and reproducible validation evidence; unsupported or incomplete trials SHALL NOT count as a validated team.

#### Scenario: Complete the authored horizon
- **WHEN** a final new team is validated under the default collection profile
- **THEN** all 1,000 trials complete the 120-second relative combat horizon with the declared targets and conditions
- **AND** successful results contain no reached unsupported dependency or action-admission failure
- **AND** repeated rotations do not reset energy, cooldowns, resources or combat state to reproduce the opener

#### Scenario: Improve a candidate rotation
- **WHEN** a build or rotation alternative is retained as an improvement
- **THEN** its evidence identifies both exact scripts, current engine, run conditions and common seed sequence
- **AND** the decision considers complete DPS outcomes and causal resource or buff receipts rather than only an opening burst or an incomplete high score
- **AND** unresolved noise is not presented as a proven improvement or global optimum

#### Scenario: Encounter a simulator defect
- **WHEN** a candidate reaches a reproducible simulator defect
- **THEN** its score remains unsuccessful until the owning defect is corrected and the affected validation is rerun
- **AND** editing away the dependency, hiding failed trials or inventing resource grants cannot turn that failure into validation evidence

#### Scenario: Continue the team until the simulation stops
- **GIVEN** a curated team run with time-based stopping
- **WHEN** the run enters its final relative seconds
- **THEN** ordinary support and carry rotation decisions continue from live state
- **AND** the script does not reserve an entire rotation or replace normal team decisions with a carry-only terminal tail to avoid crossing the endpoint

#### Scenario: Streamlined Zankou Blackbird rotation
- **GIVEN** the curated `zankou-blackbird` preset rotation
- **WHEN** support handoffs and Daffodill Phantom Step entries execute
- **THEN** Daffodill does not perform redundant zero-action switch-ins immediately before Zankou's field entry
- **AND** all 1,000 trials complete the 120-second relative combat horizon with updated catalog DPS reflectively regenerated
