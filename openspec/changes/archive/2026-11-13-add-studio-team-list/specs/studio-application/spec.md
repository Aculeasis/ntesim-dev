# Studio application delta

## ADDED Requirements

### Requirement: Team list entry and scenario selection

Studio SHALL keep Simulator, Results and Sample as left-aligned view tabs and place an independent Team list dialog button at the horizontal center of the navigation. Team list SHALL replace Load configuration, list S1, S2, S3, S5 and Baicang, and exclude the 20-second S4 scenario. Selecting a row SHALL load its exact source into Simulator, close the dialog and require an explicit Run to execute.

#### Scenario: Separate view tabs from the centered catalog button
- **WHEN** Studio is viewed on desktop
- **THEN** Simulator, Results and Sample remain grouped at the left edge as view tabs
- **AND** Team list is centered independently of their width and remains outside the view tablist
- **AND** narrow screens can place the centered Team list button on a separate row without horizontal overflow

#### Scenario: Browse and dismiss without changing the draft
- **WHEN** the user opens Team list from any Studio view and then closes it
- **THEN** the existing draft, active view and executed results remain unchanged
- **AND** the dialog is available before any simulation has run

#### Scenario: Select a bundled team
- **GIVEN** Team list contains the five retained configurations and no S4 entry
- **WHEN** the user activates a row by pointer or keyboard while no simulation or sample execution is running
- **THEN** Simulator becomes active with the complete source belonging to that row, including builds, targets, options and actions
- **AND** the dialog closes without starting a simulation or relabeling previous results as results of the selected team
- **AND** Load configuration is no longer shown in the editor

### Requirement: Compact team list rows

Each row SHALL show the scenario label, four ordered character slots and its DPS or unavailable status. Character cards SHALL be approximately half the size of collapsed Simulator cards, with Arc and cartridge icons on the card and no separate weapon row. Existing shorter rosters SHALL retain empty slots without adding actors.

#### Scenario: Compact rows without calculation details
- **WHEN** the user browses Team list
- **THEN** rows show the roster and DPS without a Details control or expandable calculation report
- **AND** an unavailable DPS retains its short reason without requiring an expander

#### Scenario: Distinguish builds with the same characters
- **WHEN** scenarios share a roster but differ in awakening or equipment
- **THEN** scenario labels, character names, awakening levels and equipment icons allow the user to distinguish them
- **AND** equipment names and refinement or set-piece counts are available through pointer and keyboard inspection

#### Scenario: Partial roster
- **WHEN** a scenario contains one, two or three characters
- **THEN** its row preserves their declared order and fills the remaining positions with visibly empty slots
- **AND** empty slots contribute no character to search or roster filtering

### Requirement: Team list character search and roster filters

Studio SHALL combine case-insensitive character-name search, Include and Exclude. Every whitespace-separated search term SHALL match a character name in the team. Every included character SHALL be present and every excluded character SHALL be absent. Empty controls SHALL impose no restriction. Filtering SHALL preserve DPS order and show an explicit empty state when nothing matches.

#### Scenario: Require all included characters
- **GIVEN** Include contains Mint and Sakiri
- **WHEN** the team list is filtered
- **THEN** only teams containing both characters remain
- **AND** selecting Zero in Exclude hides any remaining team containing Zero

#### Scenario: Search across team members
- **WHEN** the user searches for "mint SAK"
- **THEN** only teams with character names matching both terms remain, including Mint with Sakiri
- **AND** search is evaluated together with both roster filters

#### Scenario: Conflicting filters and no matches
- **GIVEN** the same character is selected in Include and Exclude
- **WHEN** the filters are applied
- **THEN** no team matches and the dialog shows an empty state
- **AND** the selections remain visible and removable

### Requirement: Team list filter pickers

The dialog header SHALL place search centrally and Include, Exclude and a close icon at the right on desktop. Include and Exclude SHALL display their selection counts and open a portrait grid with names, visible selection state and Clear All. Clicking a portrait SHALL toggle that character in the active filter independently of the other filter.

#### Scenario: Toggle and clear a filter
- **WHEN** the user opens Include and selects two character portraits
- **THEN** both portraits are visibly selected, the Include count becomes two and the list updates immediately
- **AND** Clear All empties Include without changing Exclude or the search

#### Scenario: Close a picker or reopen the list
- **WHEN** the user clicks outside an open picker or presses Escape
- **THEN** the picker closes without clearing its selections
- **AND** reopening Team list during the same app session retains search and filter selections

### Requirement: Precomputed team DPS ranking

Team list SHALL display mean damage per second from a complete shared-engine run of each exact bundled script and sort numeric DPS descending, using stable scenario identity for ties. Browsing SHALL NOT execute simulations. Failed, incomplete, unsupported or stale calculations SHALL have no ranked numeric DPS and SHALL appear after valid results with an explicit reason.

#### Scenario: Reproduce a displayed DPS value
- **WHEN** a team has a complete supported precomputation
- **THEN** its displayed DPS equals the shared engine's mean of completed per-trial damage rates using actual elapsed combat time
- **AND** the calculation retains the source and engine identity needed to reproduce it

#### Scenario: Reject stale or unsuccessful rankings
- **GIVEN** the script or engine has changed, or a calculation fails, remains incomplete or reaches an unsupported damage dependency
- **WHEN** rankings are prepared for delivery
- **THEN** an old or partial DPS value cannot be delivered as a current successful ranking
- **AND** the scenario remains discoverable and loadable with unavailable DPS and its reason
- **AND** a valid zero-DPS calculation remains numeric zero rather than unavailable

### Requirement: Bounded accessible team list dialog

Team list SHALL occupy a bounded portion of the desktop viewport with a persistent header and an internally scrolling list. It SHALL remain operable at a 390 pixel viewport without page-level horizontal overflow, support keyboard row and filter interaction, keep focus within the modal and restore focus to its opener on dismissal.

#### Scenario: Desktop and narrow layout
- **WHEN** the list is opened on desktop or at a 390 pixel viewport width
- **THEN** search, both filter controls, close control, team slots and DPS remain reachable
- **AND** scrolling the rows does not scroll the dialog header out of view
- **AND** the narrow layout can wrap character slots while keeping them grouped under one scenario

#### Scenario: Protect an active execution
- **GIVEN** a simulation or sample execution is running
- **WHEN** the user browses Team list
- **THEN** rows remain inspectable but selection is disabled with an explanation until execution finishes or is stopped
- **AND** browsing does not cancel or modify that execution
