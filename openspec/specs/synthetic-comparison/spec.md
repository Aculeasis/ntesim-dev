# Paired synthetic comparisons

## Purpose

Defines comparable synthetic contexts, exact seed pairing and uncertainty over complete candidate-minus-baseline differences.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Comparable context

Paired comparisons SHALL validate shared duration, clock, target and stream context and the combined work budget before either variant executes.

#### Scenario: Different target health
- **GIVEN** the variants declare different initial health for the same target
- **WHEN** a paired comparison is requested
- **THEN** preflight rejects the comparison without running either side

### Requirement: Complete paired differences

Differences SHALL be candidate minus baseline for matching seeds, with aggregate paired statistics withheld unless every requested pair completes.

#### Scenario: Incomplete candidate
- **GIVEN** a baseline run completes but its paired candidate fails
- **WHEN** the comparison is reported
- **THEN** that pair has no difference, partial diagnostics remain visible and paired statistics are null
