# Product scope and modeling

## Purpose

Defines reproducible team DPS modeling, explicit combat assumptions and the supported product boundary.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Declared dummy scenarios

The simulator SHALL evaluate anonymous stationary dummies with explicit stats and combat assumptions, preserving owner-specific recipient caps and distribution.

#### Scenario: Uncapped and capped attacks
- **GIVEN** three eligible dummies are declared and all are in range
- **WHEN** an uncapped AoE and then an attack capped at two recipients execute
- **THEN** the AoE can hit all three dummies and the capped attack hits at most two

### Requirement: Visible model limitations

Results SHALL retain applicable assumptions, unsupported dependencies and DPS-relevant model limits; a reached unsupported damage dependency SHALL prevent a complete successful result.

#### Scenario: Unsupported reached passive
- **GIVEN** an equipped passive has an unsupported reached damage dependency
- **WHEN** a rotation reaches its trigger
- **THEN** the dependency is diagnosed and the run is excluded from successful comparisons

### Requirement: Continuing combat

Repeatability checks SHALL preserve resources, buffs, cooldowns, targets, pending effects and RNG across at least two continuing cycles.

#### Scenario: Second cycle retains state
- **GIVEN** the first cycle leaves a cooldown and a delayed effect pending
- **WHEN** the next cycle begins
- **THEN** both continue from their existing state without reinitialization
