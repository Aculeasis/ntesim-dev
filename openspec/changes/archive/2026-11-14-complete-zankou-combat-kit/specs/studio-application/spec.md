# Zankou Insight preset delta

## MODIFIED Requirements

### Requirement: Mandatory and excluded team content

The collection SHALL contain Nanally / Hotori / Zero / Jiuyuan and Zankou / Iroi / Sakiri / Daffodill. Daffodill in the second composition SHALL have exactly A3, A4 and A5 selected; Sakiri in that composition SHALL have exactly A4 selected. New compositions SHALL exclude Elyms, Akane Rin and both characters' signature weapons.

#### Scenario: Inspect mandatory compositions
- **WHEN** either mandatory entry is selected and inspected
- **THEN** its four declared characters match the required roster
- **AND** Daffodill in the Zankou entry has three awakenings with selected indices `[3,4,5]`, without A1, A2 or A6
- **AND** Sakiri in the Zankou entry has selected indices `[4]`, retaining the one-node investment profile

#### Scenario: Exclude unavailable content
- **WHEN** the new collection's builds are inspected
- **THEN** none contains Elyms, Akane Rin or their signature weapons
- **AND** source recommendations involving them have been replaced by independently justified available choices rather than unresolved placeholders

## ADDED Requirements

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
- **AND** the shorter tail completes the exact 120-second horizon without inventing a full final rotation

### Requirement: Reproducible corrected Insight result

The corrected Insight preset SHALL retain its stable identity, equipment, roster and fifteen-line comparison budget, with Sakiri's selected-node substitution documented. Its ranked result SHALL come from all 1000 supported, complete trials over 120 real seconds using current engine/source identities. Exact source loading, Worker execution and Sample replay SHALL agree.

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
