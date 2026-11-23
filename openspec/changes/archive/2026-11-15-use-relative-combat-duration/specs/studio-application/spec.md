# Spec Delta

## ADDED Requirements

### Requirement: Duration basis selection

Sim Duration (Dur) SHALL provide an upper-right Relative / Absolute selector, defaulting to Relative for a newly displayed run. Its mean, minimum, maximum, standard deviation and quartiles SHALL use the selected native per-trial duration distribution. Selection SHALL affect only that card's duration display.

#### Scenario: Inspect a run with pauses
- **GIVEN** completed seeds each last 120 relative seconds but have different absolute durations
- **WHEN** Results first displays their duration card
- **THEN** Relative is selected and every duration statistic reflects the 120-second relative distribution
- **AND** selecting Absolute updates every statistic to the delivered absolute distribution

#### Scenario: Switch without changing the run
- **WHEN** the duration selector is changed
- **THEN** no simulation or sample regeneration starts, DPS remains relative, and other metrics, charts, playback geometry, source identity and completion status retain their meanings
- **AND** both duration distributions remain available through subsequent selection changes

#### Scenario: Accessible selection and report image
- **WHEN** the duration selector is used by keyboard, at a 390-pixel viewport or in a report image export
- **THEN** its selected basis is identifiable without clipping and the exported image reflects the current selection
- **AND** embedded report data retains both native duration distributions

### Requirement: Absolute playback with dual clock readouts

Sample playback SHALL use absolute elapsed time for its axis, seek cursor, event stepping and speed. It SHALL show separately labeled relative and absolute elapsed times and their run endpoints, and visually identify native timer-pause spans. Distinct absolute events SHALL remain individually inspectable within a relative-time plateau.

#### Scenario: Attacks during stopped time
- **GIVEN** a recorded timer pause covers absolute seconds 35 through 45 with attacks at 38 and 41
- **WHEN** Sample plays or seeks through the interval
- **THEN** the attacks occupy distinct absolute positions and can be inspected separately
- **AND** the relative readout stays at 35 seconds while the absolute readout advances
- **AND** the pause span is visibly identified without inventing enemy movement or geometry

#### Scenario: Forward and reverse stepping within a pause
- **WHEN** the user steps between events in either direction inside a timer pause
- **THEN** playback uses each event's original absolute timestamp and causal prefix to restore the native state
- **AND** sharing a relative timestamp cannot skip or merge distinct absolute events

#### Scenario: Ordinary Ultimate pause presentation
- **WHEN** Sample plays an Ultimate cinematic for a supported character other than Hotori
- **THEN** the relative readout pauses and the native interval is shaded using the same presentation as other timer pauses
- **AND** the absolute playhead continues and resumes relative progression at that Ultimate's actual cinematic release

#### Scenario: Partial sample and regenerated seed
- **WHEN** a failed sample is inspected or another completed seed is loaded
- **THEN** readouts and pause shading use that sample's own native durations and clipped intervals
- **AND** a regenerated completed sample is verified against its delivered seed summary before display

#### Scenario: Inspect different timer domains inside a pause
- **GIVEN** a sample includes a paused ordinary buff/cooldown and a deployed field whose lifetime progresses during Hotori active Time Stop
- **WHEN** the absolute cursor advances or seeks inside that interval
- **THEN** the inspector retains the ordinary timer remainders while the field can lose remaining lifetime and expire according to native observations
- **AND** presentation does not decrement all timers by the absolute cursor delta or infer exemptions from actor names

### Requirement: Absolute damage timeline coverage

Aggregate damage and cumulative charts SHALL identify their X-axis as absolute combat seconds and cover the largest delivered completed-trial absolute duration. Damage after the relative limit's numeric value SHALL retain its actual position. Bucket rates SHALL identify absolute seconds independently of relative whole-run DPS.

#### Scenario: Damage after absolute second 120
- **GIVEN** `duration=120` produces a completed trial lasting 140 absolute seconds with damage at second 135
- **WHEN** its damage timeline is rendered
- **THEN** the chart includes second 135 at its actual absolute position and the cumulative endpoint includes that damage
- **AND** the damage is neither clipped nor moved into the bucket ending at second 120

#### Scenario: Trials have different absolute endpoints
- **WHEN** delivered completed seeds end at different absolute times
- **THEN** aggregate bins share absolute boundaries across seeds and retain completed trials' cumulative totals after their individual endpoints
- **AND** no post-completion damage is invented for a shorter seed

### Requirement: Explicit report time bases

Report metadata, inspectors and exports SHALL distinguish the authored relative limit, actual relative duration and actual absolute duration. Whole-run DPS and its actor, source and target contributions SHALL identify relative seconds. Existing EPS, HPS, SHP and field-time accounting SHALL retain their absolute-time definitions with inspectable units and time bases.

#### Scenario: Restore exported clock data
- **WHEN** a current-format report with timer pauses is exported and imported without running a simulation
- **THEN** both duration distributions, relative DPS, absolute timeline extent and detailed sample pause spans are restored from recorded data
- **AND** unsupported old clock contracts or inconsistent elapsed totals produce a readable import error without replacing existing results

#### Scenario: Zero relative duration
- **WHEN** a completed combat has zero relative duration and therefore unavailable relative DPS
- **THEN** Results and Sample retain the damage and both durations, and show DPS as unavailable rather than zero or infinity
- **AND** unavailable DPS does not become a successful comparison through an import, filter or duration-card selection

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


### Requirement: Reproducible corrected Insight result

The corrected Insight preset SHALL retain its stable identity, equipment, roster and fifteen-line comparison budget, with Sakiri's selected-node substitution documented. Its ranked result SHALL come from all 1000 supported, complete trials over 120 relative seconds using current engine/source identities. Exact source loading, Worker execution and Sample replay SHALL agree.

#### Scenario: Regenerate after the combat correction
- **WHEN** the corrected source and engine are delivered
- **THEN** its catalog result is regenerated from the exact source and current WASM rather than reusing the old simplified rotation's score
- **AND** affected shared-engine results are refreshed while other authored preset bytes, including S5, remain unchanged
- **AND** incomplete trials, missing form prerequisites and unsupported dependencies cannot produce a successful numeric ranking

#### Scenario: Explain Sakiri's measured contribution
- **GIVEN** matched runs of the corrected rotation with Sakiri A4 absent or selected
- **WHEN** the contribution is reviewed under identical gear, resources, seeds and horizon
- **THEN** the comparison retains actual ATK grants, recipients, buff uptime and affected damage alongside the total-DPS difference
- **AND** the delivered preset has exactly Sakiri `[4]` and Daffodill `[3,4,5]` without increasing either character's selected-node count


#### Scenario: Clock-policy change invalidates prior scores
- **GIVEN** a retained catalog score uses the previous clock identity or absolute-duration policy
- **WHEN** the current engine evaluates its eligibility for comparison
- **THEN** that score is stale until explicitly regenerated under the current relative-duration and timer-policy identity
- **AND** editing the clock policy does not automatically launch a full catalog recalculation or present stale values as current rankings
