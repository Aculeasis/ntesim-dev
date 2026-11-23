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
- **AND** energy source rows show each character's contribution to the selected gross, retained or overflow accounting

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

### Requirement: Studio tools and script import

Studio SHALL provide a non-modal Tools dropdown menu with adaptive viewport placement and an indicator arrow, support script imports up to 50 MB and simulation report imports up to 100 MB, and display team cards and editor presets without redundant script line pointers or initial resource panels. MB denotes 1,000,000 bytes.

#### Scenario: Tools dropdown menu
- **WHEN** the Tools button is clicked
- **THEN** a dropdown menu appears adaptively positioned above or below the button to remain within the visible viewport
- **AND** selecting an action or clicking outside dismisses the menu without opening a full-page modal

#### Scenario: Large report import
- **WHEN** a user imports a valid JSON, compressed or PNG simulation report with source up to 100 MB through Import report
- **THEN** the file is accepted subject to the independent decoded-report limit and its source is normalized for execution
- **AND** files exceeding 100 MB are rejected with an explicit error
- **AND** Import script retains its 50 MB limit for scripts and JSON exports imported as scripts

#### Scenario: Separate report and script import
- **WHEN** a user imports a complete simulation report via Import report
- **THEN** the editor loads the source script, simulation state loads the aggregate and trial results, and the active view switches to Results
- **AND** Import script updates the editor draft without overwriting or navigating to simulation results

## ADDED Requirements

### Requirement: Report export format menu

Export report SHALL offer exactly JSON, Compressed report and PNG with report in a non-modal dropdown with adaptive placement, an expanded-state indicator and keyboard operation. Selecting a format, clicking outside or pressing Escape SHALL dismiss the menu. Exports SHALL require delivered results and expose preparation failures visibly.

#### Scenario: Choose a format
- **GIVEN** stable delivered results are available and no export is being prepared
- **WHEN** the user opens Export report by pointer or keyboard
- **THEN** all three formats are discoverable and selectable without leaving Results
- **AND** keyboard focus can move between entries, Escape dismisses the menu and focus returns to its opener

#### Scenario: No delivered result
- **WHEN** no simulation result is available
- **THEN** report download actions are unavailable rather than producing empty files

#### Scenario: Capture only stable results
- **GIVEN** simulation or sample execution is active
- **WHEN** the user opens Export report
- **THEN** PNG with report remains visible but is unavailable with an explanation until execution stops or finishes
- **AND** JSON and compressed exports can retain a snapshot of the delivered results

### Requirement: Portable report restoration

JSON, Zstandard level-9 compressed reports and report-bearing PNGs SHALL carry the same complete executed report bundle. Import report SHALL restore its source, request, aggregates, samples, playback report and failure/cancellation context without simulation, precision loss or duplicate sample execution payloads. Draft edits SHALL NOT change exported execution identity.

#### Scenario: Restore each exported format
- **GIVEN** a completed run with exact values outside JavaScript's safe-integer range
- **WHEN** its JSON, compressed report or PNG with report is exported and imported
- **THEN** the same executed source, native observations, exact digits, aggregates, samples and playback data are restored
- **AND** no simulation is automatically started

#### Scenario: Export after a draft edit
- **GIVEN** results exist and the editor draft has subsequently changed
- **WHEN** any report format is exported
- **THEN** the saved report contains the executed source and request belonging to those results

#### Scenario: Preserve an unsuccessful or partial run
- **GIVEN** a stopped or failed run has delivered completed-trial results
- **WHEN** its report is exported and imported in any of the three formats
- **THEN** completed/requested counts, failure details, cancellation context and partial/evidence status retain their recorded meanings
- **AND** the import does not promote the run to a successful complete comparison

### Requirement: Faithful report image with embedded data

PNG with report SHALL save the displayed Results content from the character cards through Character Energy at End of Simulation, inclusive, at the current layout width and with current selections and expansion state. It SHALL include the complete compressed report as metadata, remain viewable as a PNG, and omit later comparison and Run details panels.

#### Scenario: Capture the complete selected region
- **GIVEN** Results extends below the viewport and uses selected chart filters or expanded cards/source rows
- **WHEN** PNG with report is exported
- **THEN** the image includes the entire selected region, including off-screen content, with those selections and expansion states
- **AND** the energy panel's heading and complete contents are included
- **AND** navigation, the export menu, comparison panels and Run details are excluded

#### Scenario: Match the current responsive presentation
- **WHEN** PNG with report is exported at desktop or 390 pixel viewport width
- **THEN** its cards, artwork, chart labels, colors and arrangement match the displayed layout without clipped content or missing assets
- **AND** exporting does not change the live page's controls or expansion state

#### Scenario: View and restore the same image
- **WHEN** the exported PNG is opened in an ordinary image viewer or imported through Import report
- **THEN** the viewer displays the report image and Studio restores its embedded complete report, respectively
- **AND** a failed capture produces a visible error instead of downloading an incomplete image

### Requirement: Bounded report files and decompression

Report files SHALL be limited to 100,000,000 bytes and their decoded UTF-8 report JSON SHALL independently be limited to 100,000,000 bytes. The limits SHALL apply to imports and exports of all report formats, including the entire PNG file. Compressed input SHALL NOT cause output allocation or decompression beyond the decoded limit, regardless of untrusted size declarations.

#### Scenario: Enforce the file boundary
- **WHEN** an otherwise valid report file contains exactly 100,000,000 bytes
- **THEN** it is eligible for import subject to decoded size and format validation
- **AND** a file of 100,000,001 bytes is rejected before reading its contents

#### Scenario: Enforce the decoded boundary
- **WHEN** a valid compressed report represents exactly 100,000,000 bytes of report JSON within the file limit
- **THEN** it is eligible for import
- **AND** a standalone or PNG-embedded report representing 100,000,001 decoded bytes is rejected before oversized output is allocated

#### Scenario: Reject a decompression bomb or false size
- **GIVEN** a small compressed file or PNG attachment declares an allowed output size but attempts to expand beyond it or beyond 100 MB
- **WHEN** Import report decodes it
- **THEN** decoding fails within the permitted output capacity without first materializing the oversized report
- **AND** forged large frame sizes or decoder workspace requests cannot trigger allocations based on unchecked metadata

#### Scenario: Bound generated files
- **WHEN** an export would exceed the decoded-report limit or produce a JSON, compressed or complete PNG file above the file limit
- **THEN** Studio shows a size-limit error and does not offer an oversized download

### Requirement: Atomic report import diagnostics

Report import SHALL distinguish unsupported formats or versions, damaged report data, absent report data and size-limit violations with readable English errors. It SHALL validate the input before replacing the editor draft or displayed run. PNG import SHALL extract report metadata without decoding the image pixels.

#### Scenario: Image without report data
- **WHEN** a normal PNG without the report attachment is imported
- **THEN** Studio explains that the image contains no embedded simulation report
- **AND** the current draft and results remain unchanged

#### Scenario: Damaged or unsupported input
- **WHEN** a report has a truncated or inconsistent container, corrupt PNG attachment, invalid checksum, invalid report JSON, missing required report fields or unsupported format/version
- **THEN** Studio reports the relevant readable error and preserves the current draft and results
- **AND** repeated attempts do not accumulate codec resources or prevent a subsequent valid import
