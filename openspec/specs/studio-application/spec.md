# Studio application

## Purpose

Defines the browser source workflow, Worker lifecycle, recorded result identity and faithful rendering of native observations.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Executed source identity

Studio results, replay and exports SHALL bind to the executed source and delivered observations independently of subsequent draft edits.

#### Scenario: Edit after execution
- **GIVEN** a run has produced results
- **WHEN** the editor draft changes
- **THEN** existing results retain their original executed source and request

### Requirement: Stop and verified resume

Stopping SHALL retain delivered work; resume SHALL reconstruct and verify that prefix in a fresh Worker before executing remaining seeds.

#### Scenario: Resume with an edited draft
- **GIVEN** a stopped run has delivered seeds and the draft has since changed
- **WHEN** the user resumes that run
- **THEN** Rust verifies the delivered prefix using the executed source and continues its remaining seeds

### Requirement: Native observation rendering

Studio SHALL derive damage, resources, playback and exact exports from native receipts and retain incomplete or failed status through all views.

#### Scenario: Failed sample export
- **GIVEN** a source trial fails before any successful seed completes
- **WHEN** its sample is inspected or exported
- **THEN** the actual diagnostic and available state and report remain accessible without success statistics

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

### Requirement: NTE target and contribution identity

Target information SHALL show level, HP, DEF and six element resistances, using explicit overrides or base resistances. Report element colors SHALL follow the NTE palette. Source damage charts SHALL show a matching character-color legend. In `full`, source damage charts SHALL distinguish ability effects, combos, stances, weapon procs, and periodic/DoT sources without duplicate bare ability titles, and format uncurated effects through a readable humanized fallback.

#### Scenario: Element resistance override
- **WHEN** a target has 20% base resistance and 30% Anima resistance
- **THEN** the Anima entry shows 30% and the other elements without overrides show 20%
- **AND** Anima is green, Psyche blue, Incantation red/pink, Chaos purple, Lakshana yellow and Cosmos pale white

#### Scenario: Source chart selection
- **WHEN** the source character filter changes
- **THEN** the visible character legend matches the selected owners and the colors of their bars
- **AND** in `full`, Scorch, Blossom, Nova and Hexed sources use their own names

#### Scenario: Distinct skill effect labels
- **WHEN** an ability produces damage across multiple distinct effect packets in a simulation run and the chart is in `full`
- **THEN** each source row on the Source DPS and Source Damage Instances charts displays a distinct label identifying the specific effect, combo hit, branch, stance, or DoT
- **AND** no two distinct damage effect rows under the same owner share an identical bare ability title
- **AND** unmapped effect identifiers are humanized to readable names without raw `GE_` prefixes or internal technical suffixes

#### Scenario: Shared reaction ability in compact mode
- **GIVEN** Scorch, Blossom, Nova or Hexed effects are attributed to a shared reaction ability
- **WHEN** the chart is displayed in `comp`
- **THEN** the shared reaction ability is displayed as `Reactions`, with one row per contributing owner

#### Scenario: Target overview layout
- **WHEN** the report is shown on desktop or mobile
- **THEN** target details stay compact and readable, the desktop target panel fills its overview row and narrow screens keep every resistance entry within the viewport
- **AND** target name and primary stats (Level, HP, DEF) appear on a single header line
- **AND** scenarios with three or more targets omit the panel heading to maximize compact presentation
- **AND** no target position geometry is invented

### Requirement: Compact source damage view

The Source DPS and Source Damage Instances panel SHALL have a `full` / `comp` button immediately after its title, styled like the DPS card's time-basis button. The button SHALL show the active mode and be keyboard operable. `full` SHALL be the default for a newly started or imported run. `comp` SHALL show one row per owner and ability, with the sum of its delivered effect means for the selected metric.

#### Scenario: Switch between detailed and compact rows
- **GIVEN** a displayed run with multiple Wildfire effects attributed to the same owner
- **WHEN** the user switches from `full` to `comp`
- **THEN** one row labeled `Wildfire` shows the sum of the mean DPS or mean damage instance counts of all of that owner's Wildfire effects
- **AND** pointer and keyboard inspection show the owner and mean with the selected unit, while the effect details and distribution statistics are available in `full`
- **AND** switching back to `full` restores the detailed effect rows without changing the run or regenerating a sample

#### Scenario: Filter, sort and fold compact sources
- **GIVEN** a run contains effects for multiple abilities and owners
- **WHEN** the user selects `comp` and applies an owner filter
- **THEN** each row keeps its owner's color and contains only that owner's contribution, even if another owner has an ability with the same name
- **AND** the legend matches the filter and the chart ranks the filtered, grouped rows by descending mean
- **AND** with 20 or more grouped rows the folded view shows the leading 15, while fewer than 20 grouped rows are all visible without an expand control

#### Scenario: Change metric and DPS time basis in compact mode
- **GIVEN** `comp` is selected for a displayed run
- **WHEN** the user changes Type or switches between Relative and Absolute DPS
- **THEN** the panel stays in `comp`, and it uses delivered effect means for the selected Type and, for DPS, the selected basis
- **AND** if DPS is unavailable on that basis, the panel shows its unavailable state and Damage Instances remain independently viewable

#### Scenario: Receive additional results in compact mode
- **GIVEN** `comp` is selected for a run with delivered trials
- **WHEN** another result chunk arrives for that run
- **THEN** the panel stays in `comp` and shows the grouped means from the latest delivered aggregate

### Requirement: Native support summaries

Completed-trial statistics SHALL include gross, applied and excess healing, gross HPS in HP/s, available shield HP (SHP), and absorbed shield damage as a separate total. HPS and SHP SHALL use the selected elapsed time basis. Cards, histograms and exports SHALL share native per-trial values and explicit units. Zero activity with a positive denominator SHALL produce zero; undefined values SHALL remain unavailable. Incomplete trials SHALL remain excluded from successful aggregates.

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

### Requirement: Catalog weapon artwork coverage

Studio SHALL provide bundled WebP artwork assets for all catalog weapons (Arcs), mapping each weapon identifier to its valid WebP asset without fallback placeholders.

#### Scenario: Equipping catalog weapons
- **WHEN** an actor in the party equips any of the 51 catalog weapons
- **THEN** the character card and weapon inspector render the corresponding bundled WebP artwork
- **AND** the fallback placeholder is not displayed

### Requirement: Deduplicated trial report exports

Studio report exports SHALL serialize the executed source, request, aggregate, samples, and sample report once without duplicating the delivered sample execution under a second `selected_sample` payload.

#### Scenario: Exporting a completed trial run
- **WHEN** a user exports the trial results bundle from Studio
- **THEN** the exported JSON omits the redundant `selected_sample` duplicate while preserving the complete executed run bundle
- **AND** re-importing the exported file restores the executed source, aggregate statistics, samples, and sample playback without data loss

### Requirement: UI-owned event localization

Studio playback and event inspection SHALL localize actor labels, action names, and event details exclusively from structured report receipts and trace data without parsing prose text from trace events.

#### Scenario: Rendering localized combat events
- **GIVEN** a combat report with structured trace events, hit receipts, healing receipts, and grouped energy fanouts
- **WHEN** Studio builds playback events for display and search
- **THEN** event actor, title, and detail text are formatted in UI localization helpers directly from structured fields and catalog templates

### Requirement: Normalized report and rational consumption

Studio report validators, energy and target auditors, and timeline playback SHALL validate and reconstruct normalized report catalogs, delta state frames, grouped energy fanout entries, and `"<numerator>/<denominator>"` exact rationals.

#### Scenario: Validating and scrubbing a compact report
- **GIVEN** a report using cataloged provenance, delta timeline frames, grouped trace fanouts, and `"numerator/denominator"` exact rationals
- **WHEN** Studio imports or runs the scenario and scrubs through playback frames
- **THEN** observation validation enforces catalog and assumption invariants, reconstructs every actor and buff snapshot cleanly, and parses wide rationals with exact `BigInt` precision

### Requirement: Team list entry and scenario selection

Studio SHALL keep Simulator, Results and Sample as left-aligned view tabs and place an independent Team list dialog button at the horizontal center of the navigation. Team list SHALL replace Load configuration and offer the curated complete team collection alongside the S5 control. Selecting a row SHALL load its exact source into Simulator, close the dialog and require an explicit Run to execute.

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
- **GIVEN** Team list contains the curated complete teams and the S5 control, without the old partial research entries
- **WHEN** the user activates a row by pointer or keyboard while no simulation or sample execution is running
- **THEN** Simulator becomes active with the complete source belonging to that row, including builds, targets, options and actions
- **AND** each character declaration contains `qte=8`
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

Team list SHALL display its last complete supported precomputation, sorted by numeric DPS descending with stable identity for ties. Browsing SHALL NOT simulate. DPS refresh SHALL occur only on explicit request or during explicit release preparation, not on local edits, rebuilds or routine checks. Failed, incomplete and unsupported calculated results SHALL remain unavailable. Recorded source/engine identities SHALL remain truthful.

#### Scenario: Reproduce a displayed DPS value
- **WHEN** a team has a complete supported precomputation
- **THEN** its displayed DPS equals the shared engine's mean of completed per-trial damage rates using actual elapsed combat time
- **AND** the calculation retains the source and engine identity needed to reproduce it

#### Scenario: Reject stale or unsuccessful rankings
- **GIVEN** the script or engine has changed, or a calculation fails, remains incomplete or reaches an unsupported damage dependency
- **WHEN** rankings are explicitly refreshed for release preparation
- **THEN** all delivered selectable scripts are recalculated with the current engine and inputs before delivery
- **AND** an old or partial DPS value cannot substitute for a failed, incomplete or unsupported fresh result
- **AND** the scenario remains discoverable and loadable with unavailable DPS and its reason when a fresh result is unsuccessful
- **AND** a valid zero-DPS calculation remains numeric zero rather than unavailable

#### Scenario: Continue ordinary development with last-calculated DPS
- **GIVEN** a structurally valid catalog has a successful precomputation from an earlier source or engine
- **WHEN** local sources change, WASM is rebuilt, or ordinary development/build/check commands run
- **THEN** the last-calculated numeric DPS may remain visible with its recorded calculation identity
- **AND** those operations do not recalculate DPS or fail solely because calculation identities differ from current sources

#### Scenario: Explicit recalculation
- **GIVEN** a user explicitly requests DPS refresh
- **WHEN** the requested calculation completes against the current engine and inputs
- **THEN** the catalog is updated with the new outcomes and their actual calculation identities
- **AND** failed, incomplete and unsupported outcomes remain unavailable instead of retaining a previous successful number for that refreshed entry

#### Scenario: Deploy Pages with saved rankings
- **GIVEN** the checkout contains a structurally valid committed Team list
- **WHEN** the GitHub Pages workflow builds and deploys Studio
- **THEN** it reuses the committed DPS outcomes and their recorded calculation identities without recalculation
- **AND** current WASM compilation does not trigger a catalog refresh

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

### Requirement: Curated complete team collection

Team list SHALL offer twenty-two distinct curated four-character compositions with equipped builds and executable continuing rotations, alongside the S5 control. It SHALL replace the previous selectable partial research entries. Different equipment alone SHALL NOT count as a distinct composition toward the collection target.

#### Scenario: Replace partial selectable presets
- **WHEN** the curated collection is delivered
- **THEN** Team list contains the `22` complete curated teams and S5, and no selectable S1, S2, S3, S4 or old Baicang domain example
- **AND** non-selectable language and regression fixtures can remain outside the collection
- **AND** every curated entry loads its exact complete script without starting a simulation

#### Scenario: Preserve the control
- **WHEN** the collection and its ranking are regenerated
- **THEN** S5 retains its identity, label, historical build profile, target conditions and default-preset role, with explicit `qte=8` and any timing changes required by the corrected Support Skill rules
- **AND** current-engine recomputation can update its displayed DPS without silently rewriting its source

### Requirement: Mandatory and excluded team content

The collection SHALL contain Nanally / Hotori / Zero / Jiuyuan, Zankou / Iroi / Sakiri / Daffodill, and Shinku / Akane Rin / Elyms / Iroi. Daffodill in the Zankou composition SHALL have exactly A3, A4 and A5 selected; Sakiri in that composition SHALL have exactly A4 selected; and released Elyms, Akane Rin, and their signature weapons are permitted.

#### Scenario: Inspect mandatory compositions
- **WHEN** the mandatory entries are selected and inspected
- **THEN** the Nanally and Zankou entries match their required rosters, with Daffodill at `[3,4,5]` (without A1, A2, or A6) and Sakiri at `[4]`
- **AND** `shinku-elyms` declares Lv80/80 A0 (`awakening=[]`, `talent=8,8,8`, `qte=8`, `refine=1`, 15 useful lines) builds for Shinku (`lostradiance` + `fork_lunarphase`), Akane Rin (`streetboxer` + `fork_prism`), Elyms (`lostradiance` + `fork_elyms`), and Iroi (`speedyhedgehog` + `fork_door`, single `Suit11` carrier), completing 1,000 trials over 120 relative seconds

#### Scenario: Exclude unavailable content
- **WHEN** curated builds and Team list validation checks are inspected
- **THEN** none contains unreleased characters or unresolved placeholders, while released Elyms (`character:1090`), Akane Rin (`character:1057`), `weapon:fork_elyms`, and `weapon:fork_prism` are permitted in curated entries
- **AND** regenerated `web/src/generated/team-list.json` renders `ready` DPS status across all `22` curated teams plus S5

### Requirement: Comparable curated build assumptions

New teams SHALL use 15 useful substat lines per character, level-80 characters/weapons, purchased Normal, Skill and Burst levels 8/8/8 and the historical awakening profile, with role-specific stat allocation. Every Team list character, including the S5 control, SHALL declare purchased QTE level 8. Builds SHALL avoid redundant non-stacking support effects and duplicate stat grants. Justified deviations in other investments SHALL be disclosed with the affected entry.

#### Scenario: Apply the historical investment profile
- **WHEN** a new build uses the collection baseline
- **THEN** it assigns exactly 15 useful substat lines across that character's entire console, with explicit line values and numeric contributions
- **AND** console main-stat/specialization contributions and automatic character, weapon and set bonuses remain separate from that line count
- **AND** character and weapon levels are 80/80, purchased Normal, Skill and Burst levels are 8/8/8 and purchased QTE level is 8, with applicable resonance bonuses retained in effective levels
- **AND** awakening counts are A0 for limited S, A1 for standard S and Zero, and A6 for A-rank characters and Chiz, with selected node indices explicit
- **AND** weapon refinement is T1 unless the entry discloses a justified exception

#### Scenario: Role-specific equipment
- **WHEN** the same character serves different roles in two new teams
- **THEN** each script can declare different weapons, sets and authored stats under the documented budget
- **AND** no new team assigns Speedy Hedgehog to multiple characters
- **AND** character, weapon and declared set bonuses are not counted again in console additions

#### Scenario: Investment exceptions
- **WHEN** a new entry departs from the collection's baseline stat or acquisition assumptions
- **THEN** its reference states the changed investment and reason, including selected awakening nodes and weapon refinement where applicable
- **AND** a higher-investment result is not represented as the baseline build
- **AND** Daffodill `[3,4,5]` in the mandatory Zankou team remains a required exception rather than being reduced to the standard S A1 baseline
- **AND** the baseline count of 15 useful lines is not replaced by an unspecified budget chosen during implementation

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

### Requirement: Calculation cache follows actual inputs

An explicitly requested calculation SHALL reuse cached results only for matching scenario inputs, options, engine, WASM and generator identities. Explicit release preparation SHALL recalculate delivered entries without relying on prior calculation cache. Input changes alone SHALL NOT schedule recalculation.

#### Scenario: Reuse identical requested calculation
- **GIVEN** a cached result matches all calculation inputs and identities
- **WHEN** an ordinary explicit refresh requests that calculation without forcing a new run
- **THEN** the matching cached result may be reused

#### Scenario: Refresh after an engine or generator change
- **GIVEN** a previous result was calculated with different engine, WASM, generator or scenario inputs
- **WHEN** a new explicit refresh requests that calculation
- **THEN** the previous result cannot satisfy the request as a matching cached calculation
- **AND** merely changing the inputs before that request did not launch recalculation

#### Scenario: Prepare release using a warm cache
- **GIVEN** earlier calculation cache entries exist
- **WHEN** explicit release preparation refreshes the catalog
- **THEN** every delivered selectable entry is freshly calculated with the current engine and inputs
- **AND** prior cached success cannot conceal a failed, incomplete or unsupported fresh result

### Requirement: Direct configuration export
The Studio interface SHALL allow immediate file download of the current scenario script without intermediate review dialogs, generating a descriptive timestamped filename that identifies party character composition by human-readable names.

#### Scenario: Export configuration from tools dropdown
- **GIVEN** an authored configuration with configured party characters
- **WHEN** the user selects Export script from the Tools menu
- **THEN** the browser immediately prompts or downloads the `.nte` file directly
- **AND** the file name includes the current timestamp in `YYYY-MM-DD_HH-mm-ss` format and configured human-readable character names

### Requirement: Modal dialog backdrop dismissal
All Studio modal dialogs SHALL close when the user clicks outside the dialog container on the modal backdrop, in addition to Escape key presses and explicit Close buttons.

#### Scenario: Dismiss dialog by backdrop click
- **GIVEN** any modal dialog is currently open in Studio
- **WHEN** the user clicks on the backdrop outside the dialog bounding box
- **THEN** the dialog closes and triggers its normal close handler

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

### Requirement: Studio build source navigation

Studio SHALL replace the upper-right static game title with a small GitHub icon linking to `https://github.com/Aculeasis/ntesim-dev` and a separately linked build identifier opening that repository at the build's commit or explicit deployment tag. Both links SHALL have accessible names and keyboard focus. The header SHALL remain usable without horizontal overflow on narrow screens.

#### Scenario: Local main build
- **GIVEN** main has checked-out HEAD `bb887a5` and may contain uncommitted edits
- **WHEN** Studio is built or its development server starts locally
- **THEN** the identifier is `bb887a5` and links to `https://github.com/Aculeasis/ntesim-dev/tree/bb887a5`
- **AND** it identifies HEAD rather than HEAD's parent, a guessed future commit or a tag discovered locally

#### Scenario: Local branch build
- **GIVEN** the checked-out branch is `some/branch` at HEAD `bb887a5`
- **WHEN** Studio is built or its development server starts locally
- **THEN** the identifier is `some/branch/bb887a5`
- **AND** its link opens the commit tree rather than the moving branch tip

#### Scenario: Local tag checkout
- **GIVEN** local HEAD is detached at a commit bearing tag `v1.2.3`
- **WHEN** Studio starts locally without an explicit deployment identity
- **THEN** it displays the commit's short hash without a fabricated branch prefix or tag label

#### Scenario: Explicit deployment tag
- **GIVEN** a deployment builds the validated tag `v1.2.3`
- **WHEN** its Studio header renders
- **THEN** the identifier is `v1.2.3` without a branch or hash suffix
- **AND** its link opens `https://github.com/Aculeasis/ntesim-dev/tree/refs/tags/v1.2.3`
- **AND** tag names containing slashes or reserved URL characters retain their exact ref meaning in the link

#### Scenario: Keyboard navigation and narrow header
- **WHEN** the header is navigated by keyboard or viewed at a 390 pixel viewport width
- **THEN** the repository icon and version link are separately reachable with visible focus
- **AND** a long branch or tag name remains inspectable without causing page-level horizontal overflow
- **AND** the icon/ref and timestamp occupy adjacent rounded, bordered pills of equal height

### Requirement: Viewer-local build timestamp

Studio SHALL replace `Global · 04 Oct 2026` with the build-start instant rendered in the viewer's current time zone as zero-padded `dd.mm.yy · hh:mm` using a 24-hour clock. When loaded, that instant SHALL reflect the compiled WebAssembly engine's embedded build instant extracted by the client, falling back to the web distribution instant beforehand. That instant SHALL remain fixed for the loaded engine, independently of the viewer's page-open time, selected UI locale or simulation state.

#### Scenario: Same build in different time zones
- **GIVEN** a build started at `2026-11-14T10:20:30Z`
- **WHEN** it is viewed in Europe/Istanbul and UTC
- **THEN** the respective timestamps are `14.11.26 · 13:20` and `14.11.26 · 10:20`
- **AND** both represent the same build instant with daylight-saving and date boundaries determined by the viewer's zone

#### Scenario: Reload an existing build
- **GIVEN** a built Studio artifact or development server is already available
- **WHEN** a viewer reloads it later without a new build or server restart
- **THEN** the timestamp still represents that artifact's or session's original build-start instant
- **AND** it does not become a ticking clock or the commit date

#### Scenario: Engine build instant extracted from WebAssembly
- **GIVEN** an active Studio session loads a WebAssembly engine binary compiled at `2026-11-21T02:00:00Z` while the static web bundle was served at `2026-11-21T04:00:00Z`
- **WHEN** the engine finishes loading
- **THEN** Studio's header displays the engine's embedded build instant `21.11.26 · 02:00` in UTC
- **AND** the semantic time element reflects the WebAssembly engine's embedded build instant

### Requirement: Compact picker dialog headers

Name Search dialogs SHALL display search and an icon close button on one horizontally aligned row, with close immediately to the right and no visible category heading. Each dialog and close button SHALL retain an accessible name. Team list search SHALL be vertically centered with its neighboring header controls on desktop.

#### Scenario: Open a reference picker
- **WHEN** any Name Search category is opened on desktop or at 390 pixel viewport width
- **THEN** search and the close icon appear on one row without horizontal overflow or a visible category heading
- **AND** search, insertion and dismissal remain usable

#### Scenario: Open Team list on desktop
- **WHEN** Team list is opened at desktop width
- **THEN** the title, search, include/exclude buttons and close icon share a vertical center

### Requirement: Duration basis selection

Damage Per Second (DPS) and Field Time Distribution SHALL each provide an upper-right Relative / Absolute button for one shared run selection, defaulting to Relative. Both buttons SHALL select native duration, DPS, retained/granted EPS, HPS, SHP, applicable histograms, actor/source/target contributions and field time together. Sim Duration (Dur) SHALL have no selector. The selection SHALL persist across navigation and incremental results without captions or tooltips.

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

### Requirement: Esper Cycles reaction statistics panel
Studio SHALL present an "Esper Cycles" statistics panel displaying all Esper Cycle reactions available to the deployed team composition and their trigger distribution across completed simulation trials. The panel SHALL adaptively display when at least one reaction is available and hide completely when no reactions are available.

#### Scenario: Team with available reactions renders 3-column overview
- **WHEN** a simulation run completes for a party that contains at least one available Esper Cycle reaction
- **THEN** the report overview row displays a 3-column layout containing "Actions", "Field Time Distribution", and "Esper Cycles" in that order
- **AND** the "Esper Cycles" panel renders a horizontal bar chart displaying every available reaction in the team
- **AND** reactions are sorted in descending order of mean trigger count
- **AND** each reaction displays an error bar (whiskers) representing its trigger distribution (min, max, std) across trials
- **AND** hovering or focusing a reaction bar exposes its mean, min, max, and standard deviation in a tooltip

#### Scenario: Team without available reactions hides Esper Cycles panel
- **WHEN** a simulation run completes for a party with zero available reactions (such as a mono-element team or non-adjacent elements)
- **THEN** the "Esper Cycles" panel is completely hidden from the report
- **AND** the report overview row displays the 2-column layout containing only "Actions" and "Field Time Distribution"

#### Scenario: Available reactions with zero triggers display zero
- **WHEN** an available reaction in the party triggers zero times throughout all simulation trials
- **THEN** the reaction is displayed in the "Esper Cycles" panel with a trigger value of 0 and an empty error bar
- **AND** the tooltip identifies the mean, min, max, and std as 0

#### Scenario: Reaction triggers from character passives are counted
- **WHEN** character skills or passives reapply or refresh an Esper Cycle reaction (such as Baicang B2 Scorch refresh or Blackbird A6 Nova reapplication)
- **THEN** those events increment the total trigger count for the respective reaction alongside qualified switch admissions

#### Scenario: Ternary reaction triggers
- **WHEN** Blossom pistils hit targets affected by active Remora (triggering Charge) or Scorch and Nova overlap on the same living target (triggering Discord)
- **THEN** each occurrence increments the trigger count for Charge and Discord respectively

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
