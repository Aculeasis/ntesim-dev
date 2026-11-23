# Finite cycle evaluation

## Purpose

Defines explicit finite observation-boundary checks without resetting combat or claiming infinite sustainable behavior.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Declared finite conditions

Cycle evaluation SHALL check the exact declared observation count, interval tolerance and resource floors and relations against delivered snapshots.

#### Scenario: Missing cycle boundary
- **GIVEN** three observations are required but only two matching labels are delivered
- **WHEN** cycle conditions are evaluated
- **THEN** the observation-count violation is retained and the check does not pass

### Requirement: Continuous cycle state

Cycle checks SHALL retain continuing resources, cooldowns, targets, pending effects and RNG rather than repairing or resetting between observations.

#### Scenario: Resource drops at a boundary
- **GIVEN** a resource rule requires non-decreasing values
- **WHEN** a later observed boundary has a smaller value
- **THEN** the resource relation fails without topping up the resource
