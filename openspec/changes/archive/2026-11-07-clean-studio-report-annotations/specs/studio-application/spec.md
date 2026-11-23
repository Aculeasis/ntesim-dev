# Spec Delta

## MODIFIED Requirements

### Requirement: Statistical report layout

Studio SHALL present a clean, compact report with metadata, six native statistical summaries, targets and histogram, full-width timelines, contribution distributions, ranked bars, paired actions and energy accounting without redundant static footer notes. The report SHALL adapt to narrow screens without page-level horizontal overflow.

#### Scenario: Clean card and panel layout
- **WHEN** a run delivers completed trials on desktop or mobile screens
- **THEN** summary cards and chart panels display primary metrics and controls without redundant static footer captions or duplicated trial notes
- **AND** interactive inspection remains available through hover, focus and inspector views

#### Scenario: Desktop report
- **WHEN** a run delivers completed trials on a desktop screen
- **THEN** summary cards appear in three columns, both time charts span the report width, three contribution distributions share a row, and actions and field time share a row
- **AND** energy source rows show each character's contribution to the selected gross, retained or overflow accounting

#### Scenario: Narrow report
- **WHEN** the report is viewed at a 390 pixel viewport width
- **THEN** chart panels stack into one column, statistics remain readable, and controls and charts stay within the viewport
