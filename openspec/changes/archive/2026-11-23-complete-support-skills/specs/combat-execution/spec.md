# Spec Delta

## ADDED Requirements

### Requirement: Reaction-qualified Support Skill entry

An admitted real switch that triggers a qualifying reaction SHALL automatically attempt the incoming character's Support Skill once. Support Skills SHALL have no independent cooldown. Explicit and input-driven switches SHALL use the same rules and spend the outgoing Cycle once on a standard reaction switch. Support readiness SHALL use the same qualification and action guards without changing combat state.

#### Scenario: Consecutive qualified entries
- **GIVEN** the party can fund two qualifying reaction switches into the same character less than 15 seconds apart and all switch and action locks permit them
- **WHEN** both switches are admitted
- **THEN** each triggers its reaction on its locked target, pays the outgoing Cycle cost once and admits one incoming Support Skill
- **AND** elapsed time since the first Support Skill adds no cooldown gate to the second

#### Scenario: Unqualified entry or same-character switch
- **GIVEN** a switch cannot trigger a qualifying reaction, or a command selects the actor already on field
- **WHEN** the command is processed
- **THEN** it produces no Support Skill, Support cast effects or reaction cost

#### Scenario: Read Support availability
- **GIVEN** a pending choice of incoming actor and the settled state used for switch qualification
- **WHEN** the script inspects Support readiness
- **THEN** readiness reflects the same qualification and incoming action guards used for the corresponding switch
- **AND** inspection changes no Cycle, target state, pending effects or RNG

### Requirement: Lacrimosa A6 reaction entry

Selected Lacrimosa A6 SHALL arm a five-second reaction entry after Devilish Gift. A switch into Lacrimosa within that window that can trigger Nova or Scorch SHALL trigger the reaction, including eligible Discord, and admit her QTE without spending outgoing Cycle. This entry SHALL consume the window and have its own five-second internal cooldown.

#### Scenario: Armed Lacrimosa entry with an empty outgoing pool
- **GIVEN** Lacrimosa has selected A6, Devilish Gift has armed the reaction window and the outgoing actor has zero Cycle
- **WHEN** a legal switch into Lacrimosa during the window qualifies for Nova or Scorch on its living target
- **THEN** the reaction and incoming QTE each occur once and the outgoing Cycle stays at zero
- **AND** an overlap that qualifies for Discord settles through the standard reaction rules

#### Scenario: Armed Lacrimosa entry with a full outgoing pool
- **WHEN** the same A6 entry qualifies and the outgoing actor has full Cycle
- **THEN** A6 takes priority, the reaction and Support Skill each occur once and the full outgoing pool is preserved

#### Scenario: Unarmed or expired Lacrimosa entry
- **GIVEN** Lacrimosa has no live A6 window
- **WHEN** an outgoing actor with insufficient Cycle switches to her
- **THEN** it is an ordinary switch with no reaction or Support Skill

#### Scenario: Standard reaction during the A6 internal cooldown
- **GIVEN** Lacrimosa has used her free entry in the last five seconds and the outgoing actor has sufficient Cycle
- **WHEN** the outgoing actor performs a qualifying standard reaction switch into Lacrimosa
- **THEN** the standard reaction spends Cycle and Lacrimosa casts her QTE

### Requirement: Complete owner Support Skill effects

Every supported character SHALL resolve its Support Skill at the effective QTE level with its owner-specific action timing, packets, recipients, resource effects and follow-ups. The engine SHALL retain each effect's source, causal order and owner-specific level rules. An unresolved reached dependency SHALL remain visible and prevent the affected run from being counted as a successful comparison.

#### Scenario: Independently leveled damage and healing
- **GIVEN** a healing Support Skill with damage and heal components that scale with its QTE talent
- **WHEN** a qualified switch admits the skill
- **THEN** its damage and healing use the effective QTE level with their own target rules and delivery times
- **AND** any independently scaled follow-up retains its own talent and source

#### Scenario: Owner-specific Resonance 3
- **GIVEN** two characters with the same purchased QTE level and three selected awakening effects
- **WHEN** one character's Resonance 3 lists QTE and the other's Resonance 3 lists other talents
- **THEN** the first receives its specified QTE upgrade within the legal effective range and the second uses its purchased QTE level

### Requirement: Ordered automatic field-entry actions

Field entry SHALL start no ordinary attack on its own. Earned character-specific entry actions SHALL retain their own prerequisites and timing. When both Support and the regular Phantom Step are available on Daffodil's entry, Support SHALL play its full animation and Phantom Step SHALL start automatically at its end.

#### Scenario: Plain switch into Haniel, Hathor or Edgar
- **GIVEN** the incoming character has no qualified Support or other earned entry action
- **WHEN** the switch is admitted and the player issues no attack input
- **THEN** the character enters the field with no automatic ordinary attack, ordinary attack effects or ordinary attack recovery

#### Scenario: Daffodil has both QTE and Phantom Step
- **GIVEN** a qualifying reaction switch into Daffodil with an available regular Phantom Step
- **WHEN** the switch is admitted
- **THEN** Daffodil begins QTE and its full animation runs before Phantom Step starts
- **AND** Phantom Step starts at that transition without another input or switch
- **AND** both actions retain their own packets and timing, and Phantom Step spends its entry charge once

#### Scenario: Only one Daffodil action is available
- **WHEN** Daffodil enters with only qualified Support or only an available Phantom Step
- **THEN** the available action executes automatically with its own timing and costs
