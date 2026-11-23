# Spec Delta

## MODIFIED Requirements

### Requirement: Statistical report layout

Studio SHALL present a clean, compact report with execution metadata, native statistical summaries, and adaptive charts without redundant static notes or overflow.

#### Scenario: Clean card and panel layout
- **WHEN** a run delivers completed trials on desktop or mobile screens
- **THEN** summary cards and chart panels display primary metrics and controls without redundant static footer captions or duplicated trial notes
- **AND** the result metadata row displays total processed events alongside iterations, duration, seed range, and target count
- **AND** summary cards and subsequent overview panels maintain distinct vertical spacing without card collision
- **AND** interactive inspection remains available through hover, focus and inspector views

#### Scenario: Sample combat playback layout
- **WHEN** the Sample view is active
- **THEN** the playback stage displays the replay timeline and actor cues without redundant static seed/event notes or concatenated text readouts

#### Scenario: Source damage foldable view
- **GIVEN** a run produces a Source DPS or Source Damage Instances chart with 20 or more entries
- **WHEN** the chart is initially rendered
- **THEN** only the first 15 entries are displayed
- **AND** a clickable full-width toggle strip with a centered chevron icon is rendered at the bottom of the chart
- **AND** the toggle strip preserves the card panel background with a subtle border and centered chevron tab
- **AND** clicking anywhere on the strip toggles between the truncated view and the full list, changing the chevron orientation

#### Scenario: Desktop report
- **WHEN** a run delivers completed trials on a desktop screen
- **THEN** summary cards appear in three columns, the Damage Timeline chart spans the report width, three contribution distributions share a row, and actions and field time share a row
- **AND** energy source rows show each character's contribution to the selected gross, retained or overflow accounting without duplicate labels for specialized actions

#### Scenario: Damage timeline view and victory horizon
- **WHEN** a run completes under `stop=victory`
- **THEN** the Damage Timeline chart displays damage over time and cumulative damage toggled via a selector
- **AND** the timeline X-axis bounds to the actual elapsed duration of the run rather than the maximum safety horizon

#### Scenario: Narrow report
- **WHEN** the report is viewed at a 390 pixel viewport width
- **THEN** chart panels stack into one column, statistics remain readable, and controls and charts stay within the viewport

#### Scenario: Results view export and compact header
- **WHEN** the Results view is active
- **THEN** the top action bar displays an Export report dropdown button in place of the Run button
- **AND** clicking Export report opens its format menu without an intermediate dialog
- **AND** selecting an available format directly downloads the generated report file
- **AND** the downloaded report file name includes a timestamp in `YYYY-MM-DD_HH-mm-ss` format and a descriptive suffix
- **AND** the view heading, intro text, and action controls share a compact header row without dead empty space above results
- **AND** the redundant iteration toolbar below party cards is omitted
