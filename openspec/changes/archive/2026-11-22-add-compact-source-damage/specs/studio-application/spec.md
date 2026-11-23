# Studio source damage delta

## ADDED Requirements

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

## MODIFIED Requirements

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
