# Spec Delta

## MODIFIED Requirements

### Requirement: Duration basis selection

Damage Per Second (DPS) and Field Time Distribution SHALL each provide an upper-right Relative / Absolute button for one shared run selection, defaulting to Relative. Both buttons SHALL select native duration, DPS, retained/granted EPS, HPS, SHP, their applicable histograms, actor/source/target DPS contributions and field time together. Sim Duration (Dur) SHALL have no selector. The selection SHALL persist across navigation and incremental results without explanatory captions or tooltips on the buttons.

#### Scenario: Inspect a run with pauses
- **GIVEN** completed seeds each last 120 relative seconds but have different absolute durations
- **WHEN** Results first displays their duration card
- **THEN** both buttons show Relative and every duration statistic reflects the 120-second relative distribution
- **AND** selecting Absolute updates every duration statistic to the delivered absolute distribution and all basis-dependent summaries and contributions to their absolute values

#### Scenario: Switch without changing the run
- **WHEN** the user changes the basis from either card
- **THEN** both buttons show the new basis and summary statistics, applicable histograms, DPS contribution charts and Field Time Distribution use that basis
- **AND** no simulation, replay or sample regeneration starts
- **AND** both duration distributions remain available through subsequent selection changes
- **AND** the absolute damage/cumulative timeline, Sample playback geometry, raw totals, source identity and completion status retain their meanings
- **AND** Team list rankings, exact sample DPS and comparison eligibility retain their canonical relative basis

#### Scenario: Accessible selection and report image
- **WHEN** either button is used by keyboard or viewed at a 390-pixel viewport or in a report image export
- **THEN** each button's current basis is identifiable without clipping and the exported image reflects the current selection
- **AND** embedded report data retains both native duration distributions and both sets of summary values, including field time
- **AND** each button retains a card-specific accessible name without visible explanatory text or a tooltip

#### Scenario: One selection across report views
- **GIVEN** Absolute is selected for a displayed run
- **WHEN** another completed chunk arrives or the user visits Sample and returns to Results
- **THEN** both buttons and every affected summary and chart retain Absolute
- **AND** a newly started or imported run defaults to Relative

### Requirement: Explicit report time bases

Reports and exports SHALL retain the authored relative limit, both actual durations and both native summary bases, including field time. Whole-run DPS, EPS, HPS, SHP, duration and field time SHALL follow the shared selection without extra visible explanations. The absolute timeline title and Sample dual readouts SHALL identify their time axes. Raw totals SHALL retain their definitions. Canonical exact DPS and comparison eligibility SHALL remain relative.

#### Scenario: Restore exported clock data
- **WHEN** a current-format report with timer pauses is exported and imported without running a simulation
- **THEN** both duration distributions, both summary bases including field time, absolute timeline extent and detailed sample pause spans are restored from recorded data
- **AND** unsupported old contracts, missing field-time bases or inconsistent elapsed totals produce a readable import error without replacing existing results
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

## ADDED Requirements

### Requirement: Field time distribution on both clocks

Field Time Distribution SHALL show each character's mean seconds as the active character over completed trials and their percentage of the sum of those means on the selected basis. Absolute field time SHALL include all active intervals. Relative field time SHALL count only portions during which the combat timer advances. Each trial's character times SHALL sum to its elapsed time on each basis.

#### Scenario: A pause changes the field-time shares
- **GIVEN** one character is active for eight absolute seconds including four paused seconds, and another is active for four unpaused seconds
- **WHEN** the user inspects Field Time Distribution on both bases
- **THEN** Absolute uses eight and four seconds with shares of about 66.7% and 33.3%
- **AND** Relative uses four seconds each with shares of 50% each
- **AND** chart inspection shows seconds on the selected basis

#### Scenario: No relative field time
- **GIVEN** every completed seed has positive absolute duration and zero relative duration
- **WHEN** the report shows Relative
- **THEN** Field Time Distribution shows an empty state with no percentage slices
- **AND** switching to Absolute displays the delivered absolute field time

#### Scenario: Combine completed seeds across trial chunks
- **GIVEN** completed seeds with different pause lengths include a seed with zero relative duration
- **WHEN** Results displays Field Time Distribution after another completed chunk
- **THEN** the displayed means include every completed seed, with zero relative seconds for characters in the fully paused seed
- **AND** the selected basis determines both the shares and inspected seconds
