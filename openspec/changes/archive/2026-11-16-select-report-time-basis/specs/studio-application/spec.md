# Spec Delta

## MODIFIED Requirements

### Requirement: Native support summaries

Completed-trial statistics SHALL include gross, applied and excess healing, gross HPS in HP/s, time-averaged available shield HP (SHP), and actual absorbed shield damage as a separate total. HPS and SHP SHALL use the selected actual elapsed time basis. Cards, histograms and exports SHALL share native per-trial values and explicit units. Zero activity with a positive denominator SHALL produce zero; undefined values SHALL remain unavailable. Failed/incomplete trials SHALL remain excluded from successful aggregates.

#### Scenario: Effective healing and overhealing
- **WHEN** a completed 10 second trial records 1000 gross healing with 600 applied and 400 excess
- **THEN** its HPS is 100 and the report retains all three healing totals
- **AND** ordinary and autonomous healing are counted once from their canonical healing receipts

#### Scenario: Healing at full health
- **WHEN** a completed 10 second trial records 1000 gross healing entirely as excess
- **THEN** HPS is 100 even though applied healing is zero

#### Scenario: Shield absorption
- **WHEN** a completed 10 second trial records 300 shield absorption and 200 damage prevented by a separate immunity effect
- **THEN** the absorbed shield damage total is 300 and SHP is calculated independently from available shield capacity over time
- **AND** neither unused shield capacity nor non-shield prevention is counted as absorbed shield damage

#### Scenario: Redirected incoming damage
- **WHEN** an incoming event contains redirected child deliveries
- **THEN** each delivery's explicit shield absorption contributes once to the team total

#### Scenario: Shield without incoming damage
- **WHEN** a completed 10 second trial has 10000 HP of available shield capacity on its active actor for exactly 5 seconds and no shields for the other 5 seconds
- **THEN** SHP is 5000 HP even if incoming damage and absorbed shield damage are zero
- **AND** shield downtime remains part of the averaging duration

#### Scenario: Distribution and units
- **WHEN** a user inspects support cards, selects their histograms or exports trial results
- **THEN** HPS is identified as healing including overheal in HP/s and SHP as mean available shield capacity in HP
- **AND** card means, minima, maxima, standard deviations and quartiles are derived from the same native per-trial values as the histograms and exports for the selected basis

#### Scenario: Support rates across a timer pause
- **GIVEN** a completed trial lasts 10 relative seconds and 20 absolute seconds, with 1000 gross healing, 100 retained energy and 150 granted energy
- **WHEN** the report time basis changes from Relative to Absolute
- **THEN** HPS changes from 100 to 50 HP/s, retained EPS from 10 to 5 and granted EPS from 15 to 7.5
- **AND** gross, applied and excess healing, retained/granted/overflow energy and shield absorption totals remain unchanged

### Requirement: NTE available shield accounting

SHP SHALL average remaining absorbable shield capacity on the active living actor over the completed trial, weighted by the selected clock's advancing time. It SHALL respect native shield stacking, recipients, replacement, depletion, defeat, switches and exact expiry times. Off-field copies SHALL NOT inflate this value. Immunity and non-shield interception SHALL NOT contribute. Missing or invalid observations SHALL NOT become successful zero-valued summaries.

#### Scenario: Actor switch changes available protection
- **WHEN** actor A has a 10000 HP shield and is active for the first 4 seconds of a 10 second trial, then unshielded actor B is active for the remaining 6 seconds
- **THEN** SHP is 4000 HP
- **AND** actor A's off-field shield does not contribute after the switch

#### Scenario: Additive NTE shields and replacement
- **WHEN** the active actor has distinct concurrently absorbable shields with 3000 and 2000 HP remaining
- **THEN** available capacity is 5000 HP for that interval
- **AND** replacing the first shield with a 4000 HP instance changes capacity to 6000 HP rather than counting the replaced instance again

#### Scenario: Expiry during an idle interval
- **WHEN** a 10000 HP shield starts at 2 seconds, expires at 5 seconds, and the next observed event is at 9 seconds in a completed 10 second trial
- **THEN** the shield contributes exactly 3000 HP to SHP
- **AND** the later observation time does not extend its lifetime

#### Scenario: Depletion and defeat
- **WHEN** incoming damage reduces a shield's remaining HP or defeats its recipient
- **THEN** subsequent positive-duration intervals use the reduced remaining capacity or zero capacity for the defeated actor
- **AND** same-time settlements introduce no artificial duration

#### Scenario: Early completion and invalid observations
- **WHEN** a trial completes early
- **THEN** SHP uses its actual elapsed combat time in the selected basis and ignores shield lifetime beyond completion
- **AND** missing or invalid required shield observations prevent successful aggregation rather than supplying a guessed value

#### Scenario: Shield coverage only during stopped time
- **GIVEN** a trial lasts 10 relative seconds and 20 absolute seconds, with 10000 HP of available capacity throughout the 10 paused absolute seconds and no capacity while relative time advances
- **WHEN** SHP is inspected in both bases
- **THEN** absolute SHP is 5000 HP and relative SHP is zero HP
- **AND** changing only the averaging denominator cannot substitute for weighting by the selected clock

#### Scenario: Capacity changes inside a pause
- **WHEN** a shield is granted, depleted, replaced, removed or changes active recipients during a timer pause
- **THEN** absolute SHP uses the actual capacity and absolute duration of each settled interval
- **AND** the pause adds zero weight to relative SHP, while the capacity present after release contributes over subsequent relative time

### Requirement: Duration basis selection

Damage Per Second (DPS) SHALL provide the single upper-right Relative / Absolute selector, defaulting to Relative for a newly displayed run. It SHALL select native duration, DPS, retained/granted EPS, HPS and SHP statistics, their histograms and actor/source/target DPS contributions together. Sim Duration (Dur) SHALL have no selector. The selection SHALL persist for the displayed run across navigation and incremental results, without additional explanatory captions or tooltips.

#### Scenario: Inspect a run with pauses
- **GIVEN** completed seeds each last 120 relative seconds but have different absolute durations
- **WHEN** Results first displays their duration card
- **THEN** Relative is selected in the DPS card and every duration statistic reflects the 120-second relative distribution
- **AND** selecting Absolute updates every duration statistic to the delivered absolute distribution and all basis-dependent summaries and contributions to their absolute values

#### Scenario: Switch without changing the run
- **WHEN** the selector in the DPS card is changed
- **THEN** no simulation or sample regeneration starts, and summary statistics, applicable histograms and DPS contribution charts use the selected basis
- **AND** both duration distributions remain available through subsequent selection changes
- **AND** the absolute damage/cumulative timeline, Sample playback geometry, field-time accounting, raw totals, source identity and completion status retain their meanings
- **AND** Team list rankings, exact sample DPS and comparison eligibility retain their canonical relative basis

#### Scenario: Accessible selection and report image
- **WHEN** the selector in the DPS card is used by keyboard, at a 390-pixel viewport or in a report image export
- **THEN** its selected basis is identifiable without clipping and the exported image reflects the current selection
- **AND** embedded report data retains both native duration distributions and both sets of summary values
- **AND** the selector retains an accessible name without adding visible explanatory text or a tooltip

#### Scenario: One selection across report views
- **GIVEN** Absolute is selected for a displayed run
- **WHEN** another completed chunk arrives or the user visits Sample and returns to Results
- **THEN** the report retains Absolute for every affected summary and chart
- **AND** a newly started or imported run defaults to Relative

### Requirement: Absolute damage timeline coverage

Aggregate damage and cumulative charts SHALL use the panel title `Damage Timeline (absolute time)`, with selector options `Damage per second` and `Cumulative Damage`. Both SHALL use an absolute combat-seconds X-axis covering the largest completed-trial absolute duration, independently of the DPS-card selector. Damage after the relative limit's numeric value SHALL retain its position. Bucket rates SHALL use absolute seconds; cumulative values SHALL remain damage totals.

#### Scenario: Damage after absolute second 120
- **GIVEN** `duration=120` produces a completed trial lasting 140 absolute seconds with damage at second 135
- **WHEN** its damage timeline is rendered
- **THEN** the chart includes second 135 at its actual absolute position and the cumulative endpoint includes that damage
- **AND** the damage is neither clipped nor moved into the bucket ending at second 120

#### Scenario: Trials have different absolute endpoints
- **WHEN** delivered completed seeds end at different absolute times
- **THEN** aggregate bins share absolute boundaries across seeds and retain completed trials' cumulative totals after their individual endpoints
- **AND** no post-completion damage is invented for a shorter seed

#### Scenario: Select either absolute timeline view
- **WHEN** the user selects Damage per second or Cumulative Damage and changes the DPS-card time basis
- **THEN** the panel remains titled Damage Timeline (absolute time) and the selected timeline's points and absolute extent remain unchanged
- **AND** cumulative damage is presented as accumulated damage along absolute time, without dividing it by elapsed time

### Requirement: Explicit report time bases

Reports and exports SHALL retain the authored relative limit, both actual durations and both native summary bases. Whole-run DPS, EPS, HPS, SHP and duration SHALL follow the DPS-card selection without extra visible time-basis explanations. The absolute timeline title and existing Sample dual readouts SHALL identify their time axes. Field time and raw totals SHALL retain their definitions. Canonical exact DPS and comparison eligibility SHALL remain relative.

#### Scenario: Restore exported clock data
- **WHEN** a current-format report with timer pauses is exported and imported without running a simulation
- **THEN** both duration distributions, both summary bases, absolute timeline extent and detailed sample pause spans are restored from recorded data
- **AND** unsupported old contracts or inconsistent elapsed totals produce a readable import error without replacing existing results
- **AND** the imported run initially selects Relative and can switch to Absolute without simulation

#### Scenario: Zero relative duration
- **WHEN** a completed combat has zero relative duration and therefore unavailable relative time-normalized metrics
- **THEN** Results and Sample retain the damage and both durations, and show relative DPS, EPS, HPS and SHP as unavailable rather than zero or infinity
- **AND** unavailable relative DPS does not become a successful comparison through an import, filter or time-basis selection
- **AND** positive absolute duration permits otherwise supported absolute summary values without changing canonical relative eligibility

#### Scenario: Mixed seed availability
- **GIVEN** one completed seed has zero relative duration and another has positive relative duration
- **WHEN** their report statistics and histograms are inspected
- **THEN** each relative metric requiring the missing denominator is unavailable for the full seed set, without averaging only the defined subset
- **AND** absolute statistics use all eligible completed seeds with positive absolute duration
