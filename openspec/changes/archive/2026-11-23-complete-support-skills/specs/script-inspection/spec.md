# Spec Delta

## MODIFIED Requirements

### Requirement: Inspection without execution

Inspection SHALL return the complete ordered canonical party and exact compiled static stats and initial resources without executing the rotation. Each inspected character SHALL expose its purchased QTE level separately from the Normal, Skill and Burst levels.

#### Scenario: Configuration with an unbounded rotation loop
- **GIVEN** a valid party configuration is followed by a syntactically valid unbounded loop
- **WHEN** the source is inspected
- **THEN** party information is returned without unrolling the loop or advancing combat state

#### Scenario: Inspect distinct character investments
- **GIVEN** two characters explicitly declare different legal QTE levels and identical Normal, Skill and Burst levels
- **WHEN** the source is inspected
- **THEN** each character reports its own purchased QTE level

### Requirement: Atomic invalid configuration

Inspection SHALL return a null party on syntax, binding or configuration failure rather than presenting a partial roster as valid.

#### Scenario: Invalid second party member
- **GIVEN** a script declares a valid first actor and an invalid second actor
- **WHEN** inspection validates the configuration
- **THEN** the party is null and a typed source diagnostic identifies the failure

#### Scenario: Missing QTE in a party declaration
- **GIVEN** one party member omits `qte`
- **WHEN** inspection validates the configuration
- **THEN** the party is null and the source diagnostic identifies the missing field at that character declaration
