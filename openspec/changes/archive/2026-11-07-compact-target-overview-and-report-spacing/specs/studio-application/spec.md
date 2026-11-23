# Spec Delta

## MODIFIED Requirements

### Requirement: Statistical report layout

Studio SHALL present a clean, compact report with metadata, six native statistical summaries, targets and histogram, full-width timelines, contribution distributions, ranked bars, paired actions and energy accounting without redundant static footer notes. The report SHALL adapt to narrow screens without page-level horizontal overflow.

#### Scenario: Clean card and panel layout
- **WHEN** a run delivers completed trials on desktop or mobile screens
- **THEN** summary cards and chart panels display primary metrics and controls without redundant static footer captions or duplicated trial notes
- **AND** summary cards and subsequent overview panels maintain distinct vertical spacing without card collision
- **AND** interactive inspection remains available through hover, focus and inspector views

#### Scenario: Desktop report
- **WHEN** a run delivers completed trials on a desktop screen
- **THEN** summary cards appear in three columns, both time charts span the report width, three contribution distributions share a row, and actions and field time share a row
- **AND** energy source rows show each character's contribution to the selected gross, retained or overflow accounting

#### Scenario: Narrow report
- **WHEN** the report is viewed at a 390 pixel viewport width
- **THEN** chart panels stack into one column, statistics remain readable, and controls and charts stay within the viewport

#### Scenario: Results view export and compact header
- **WHEN** the Results view is active
- **THEN** the top action bar displays a standard Export report button in place of the Run button
- **AND** the view heading, intro text, and action controls share a compact header row without dead empty space above results
- **AND** the redundant iteration toolbar below party cards is omitted

### Requirement: NTE target and contribution identity

Target information SHALL show level, current/maximum HP, DEF and six named element-icon resistances, using each explicit override or the base resistance. Report element colors SHALL follow the NTE palette. Source damage charts SHALL show a matching character-color legend and distinguish named Esper Cycle, periodic and summon sources while retaining native owners.

#### Scenario: Element resistance override
- **WHEN** a target has 20% base resistance and 30% Anima resistance
- **THEN** the Anima entry shows 30% and other elements without overrides show 20%
- **AND** Anima is green, Psyche blue, Incantation red/pink, Chaos purple, Lakshana yellow and Cosmos pale white

#### Scenario: Source chart selection
- **WHEN** the source character filter changes
- **THEN** the visible character legend matches the selected owners and the colors of their bars
- **AND** Scorch, Blossom, Nova and Hexed sources use their own names rather than an indistinguishable custom-ability label

#### Scenario: Target overview layout
- **WHEN** the report is shown on desktop or mobile
- **THEN** target details stay compact and readable, the desktop target panel fills its overview row and narrow screens keep every resistance entry within the viewport
- **AND** target name and primary stats (Level, HP, DEF) appear on a single header line
- **AND** scenarios with three or more targets omit the panel heading to maximize compact presentation
- **AND** no target position geometry is invented
