# Spec Delta

## MODIFIED Requirements

### Requirement: NTE target and contribution identity

Target information SHALL show level, HP, DEF and six element-icon resistances, using each explicit override or base resistance. Report element colors SHALL follow the NTE palette. Source damage charts SHALL show a matching character-color legend, distinguish individual ability damage effects, multi-hit combo components, stance attacks, weapon procs, and character periodic/DoT sources without duplicate bare ability titles, and format uncurated effects through a readable humanized fallback.

#### Scenario: Element resistance override
- **WHEN** a target has 20% base resistance and 30% Anima resistance
- **THEN** the Anima entry shows 30% and other elements without overrides show 20%
- **AND** Anima is green, Psyche blue, Incantation red/pink, Chaos purple, Lakshana yellow and Cosmos pale white

#### Scenario: Source chart selection
- **WHEN** the source character filter changes
- **THEN** the visible character legend matches the selected owners and the colors of their bars
- **AND** Scorch, Blossom, Nova and Hexed sources use their own names rather than an indistinguishable custom-ability label

#### Scenario: Distinct skill effect labels
- **WHEN** an ability produces damage across multiple distinct effect packets in a simulation run
- **THEN** each source row on the Source DPS and Source Damage Instances charts displays a distinct label identifying the specific effect, combo hit, branch, stance, or DoT
- **AND** no two distinct damage effect rows under the same owner share an identical bare ability title
- **AND** unmapped effect identifiers are humanized to readable names without raw `GE_` prefixes or internal technical suffixes

#### Scenario: Target overview layout
- **WHEN** the report is shown on desktop or mobile
- **THEN** target details stay compact and readable, the desktop target panel fills its overview row and narrow screens keep every resistance entry within the viewport
- **AND** target name and primary stats (Level, HP, DEF) appear on a single header line
- **AND** scenarios with three or more targets omit the panel heading to maximize compact presentation
- **AND** no target position geometry is invented
