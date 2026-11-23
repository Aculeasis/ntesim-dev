# Studio application delta

## ADDED Requirements

### Requirement: Curated complete team collection

Team list SHALL offer approximately twenty distinct new four-character compositions with equipped builds and executable continuing rotations, alongside the unchanged S5 control. It SHALL replace the previous selectable partial research entries. Different equipment alone SHALL NOT count as a distinct composition toward the collection target.

#### Scenario: Replace partial selectable presets
- **WHEN** the curated collection is delivered
- **THEN** Team list contains the new complete teams and S5, and no selectable S1, S2, S3, S4 or old Baicang domain example
- **AND** non-selectable language and regression fixtures can remain outside the collection
- **AND** every new entry loads its exact complete script without starting a simulation

#### Scenario: Preserve the control
- **WHEN** the collection and its ranking are regenerated
- **THEN** S5 retains its identity, label, exact script, builds, target conditions and default-preset role
- **AND** current-engine recomputation can update its displayed DPS without silently rewriting its source

### Requirement: Mandatory and excluded team content

The collection SHALL contain Nanally / Hotori / Zero / Jiuyuan and Zankou / Iroi / Sakiri / Daffodill. Daffodill in the second composition SHALL have exactly A3, A4 and A5 selected. New compositions SHALL exclude Elyms, Akane Rin and both characters' signature weapons.

#### Scenario: Inspect mandatory compositions
- **WHEN** either mandatory entry is selected and inspected
- **THEN** its four declared characters match the required roster
- **AND** Daffodill in the Zankou entry has three awakenings with selected indices `[3,4,5]`, without A1, A2 or A6

#### Scenario: Exclude unavailable content
- **WHEN** the new collection's builds are inspected
- **THEN** none contains Elyms, Akane Rin or their signature weapons
- **AND** source recommendations involving them have been replaced by independently justified available choices rather than unresolved placeholders

### Requirement: Comparable curated build assumptions

New teams SHALL use 15 useful substat lines per character, level-80 characters/weapons, purchased talents 8/8/8 and the historical awakening profile, with role-specific stat allocation. Builds SHALL avoid redundant non-stacking support effects and duplicate stat grants. Justified deviations SHALL be disclosed with the affected entry; S5 SHALL retain its original assumptions.

#### Scenario: Apply the historical investment profile
- **WHEN** a new build uses the collection baseline
- **THEN** it assigns exactly 15 useful substat lines across that character's entire console, with explicit line values and numeric contributions
- **AND** console main-stat/specialization contributions and automatic character, weapon and set bonuses remain separate from that line count
- **AND** character and weapon levels are 80/80 and purchased talents are 8/8/8, with applicable resonance bonuses retained in effective levels
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

Each delivered new team SHALL have a complete supported shared-engine result using 1,000 iterations over 120 real combat seconds by default. Scripts SHALL preserve continuing resources, cooldowns and owner state. Exceptions SHALL retain explicit conditions. Research choices SHALL retain source references and reproducible validation evidence; unsupported or incomplete trials SHALL NOT count as a validated team.

#### Scenario: Complete the authored horizon
- **WHEN** a final new team is validated under the default collection profile
- **THEN** all 1,000 trials complete the 120-second real combat horizon with the declared targets and conditions
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

## MODIFIED Requirements

### Requirement: Team list entry and scenario selection

Studio SHALL keep Simulator, Results and Sample as left-aligned view tabs and place an independent Team list dialog button at the horizontal center of the navigation. Team list SHALL replace Load configuration, offer the curated complete team collection alongside the unchanged S5 control. Selecting a row SHALL load its exact source into Simulator, close the dialog and require an explicit Run to execute.

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
- **AND** the dialog closes without starting a simulation or relabeling previous results as results of the selected team
- **AND** Load configuration is no longer shown in the editor
