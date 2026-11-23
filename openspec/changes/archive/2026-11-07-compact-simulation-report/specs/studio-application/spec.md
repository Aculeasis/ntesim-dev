# Spec Delta

## ADDED Requirements

### Requirement: Statistical report layout

Studio SHALL present a compact report with metadata, six native statistical summaries, target information and an outcome histogram, separate full-width damage and cumulative timelines, three contribution distributions, ranked character/source contributions, paired actions/field time and energy accounting. The report SHALL adapt to narrow screens without page-level horizontal overflow.

#### Scenario: Desktop report
- **WHEN** a run delivers completed trials on a desktop screen
- **THEN** summary cards appear in three columns, both time charts span the report width, three contribution distributions share a row, and actions and field time share a row
- **AND** energy source rows show each character's contribution to the selected gross, retained or overflow accounting

#### Scenario: Narrow report
- **WHEN** the report is viewed at a 390 pixel viewport width
- **THEN** chart panels stack into one column, statistics remain readable, and controls and charts stay within the viewport

### Requirement: Faithful chart statistics

Report charts SHALL use delivered native aggregates, identify units and statistical series, retain source/character filters, and expose values through pointer and keyboard inspection. Character colors SHALL remain consistent across contribution and resource charts. Character-element grouping SHALL be identified as such; unavailable game-specific aggregates SHALL NOT be invented.

#### Scenario: Inspect time statistics
- **WHEN** a damage or cumulative timeline point is hovered or focused
- **THEN** its real simulation time, mean, minimum, maximum, standard deviation and quartiles are available
- **AND** cumulative plots identify percentile curves while damage plots identify mean and variability

#### Scenario: Change contribution or energy selection
- **WHEN** a user changes a source character, contribution metric or energy accounting selection
- **THEN** chart values and units reflect the selected native aggregate without changing the executed run

#### Scenario: Partial results
- **WHEN** a run is running, stopped or failed after delivering completed trials
- **THEN** the report retains that status and completed/requested counts and plots only delivered completed-trial aggregates
