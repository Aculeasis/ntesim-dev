# Spec Delta

## ADDED Requirements

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
- **AND** no target position geometry is invented

### Requirement: Native support summaries

Completed-trial report statistics SHALL include gross, effective and excess healing, effective healing per second and actual shield damage absorption per second. Rates SHALL use each trial's actual elapsed combat time. Zero activity SHALL produce zero statistics. Failed/incomplete trials SHALL remain excluded from successful aggregates.

#### Scenario: Effective healing and overhealing
- **WHEN** a completed 10 second trial records 1000 gross healing with 600 applied and 400 excess
- **THEN** its effective HPS is 60 and the report retains all three healing totals
- **AND** ordinary and autonomous healing are counted once from their canonical healing receipts

#### Scenario: Shield absorption
- **WHEN** a completed 10 second trial records 300 shield absorption and 200 damage prevented by a separate immunity effect
- **THEN** shield absorption per second is 30
- **AND** neither unused shield capacity nor non-shield prevention is counted as absorbed shield damage

#### Scenario: Redirected incoming damage
- **WHEN** an incoming event contains redirected child deliveries
- **THEN** each delivery's explicit shield absorption contributes once to the team total
