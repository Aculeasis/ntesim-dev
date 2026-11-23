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

### Requirement: State-driven Zankou Insight rotation

The bundled `zankou-insight` scenario SHALL use its exact equipped roster with support setup and a continuing state-driven Zankou long rotation. It SHALL enter enhanced Sanguine Dash from a legal prerequisite, stay on field through enhanced Soulcross, use Bloodfeast Reverie followed by the earned free Inferno Flamenco, and refresh support only at lawful form boundaries.

#### Scenario: Execute a complete main-damage window
- **GIVEN** the scenario begins with Zankou's actual entry Cycle and the authored comparison resources
- **WHEN** its opener and first full damage window execute
- **THEN** Zankou's handoff to Daffodill establishes actual Scorch, followed by Daffodill/Sakiri/Iroi setup and actual supported Phantom Step returns
- **AND** Iroi uses an available Burst before her Skill and Zankou receives the actual support buffs
- **AND** a legal held attack enables enhanced Sanguine Dash, followed by Illusion attacks until enhanced Soulcross is actually available
- **AND** Zankou does not leave the field in that uninterrupted Illusion window
- **AND** enhanced Soulcross leads to Bloodfeast Reverie and then the first legally admitted free Inferno Flamenco before its deadline

#### Scenario: Stack DoTs and continue without resetting resources
- **WHEN** subsequent damage and support windows execute
- **THEN** qualified producers build and spread both independent DoTs toward their ten-stack caps, including Vile Ash during the full stacking window
- **AND** support handoffs use post-Soulcross/Ultimate or other sourced safe boundaries and retain ongoing DoTs
- **AND** later paid Ultimates require earned energy, cooldowns and continuing owner state
- **AND** the continuing rotation stops at exactly 120 relative seconds, allowing a partial final action or form window

#### Scenario: Late support participation
- **GIVEN** the 120-relative-second Insight preset with seed 42
- **WHEN** its continuing rotation crosses relative 90 seconds
- **THEN** Daffodill, Sakiri and Iroi each admit a later available Skill at a lawful handoff
- **AND** no full-rotation reserve permanently disables support before the endpoint
