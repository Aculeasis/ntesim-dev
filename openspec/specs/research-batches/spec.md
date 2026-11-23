# Budgeted research experiments

## Purpose

Defines explicit research seeds, timing perturbations and bounded retained experiments with visible model limitations.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Explicit timing perturbation

Research timing experiments SHALL apply only their declared recovery and hit-delay scaling and SHALL NOT silently repair a resulting illegal rotation.

#### Scenario: Timing collision
- **GIVEN** a declared timing multiplier makes a later input illegal
- **WHEN** that trial executes
- **THEN** the actual failure is retained rather than inserting a wait

### Requirement: Bounded retained results

Research batches SHALL enforce event and retained-receipt budgets and withhold all timing statistics if any requested trial cannot complete or be retained.

#### Scenario: Retention budget reached
- **GIVEN** the next completed receipt exceeds the declared retention budget
- **WHEN** the batch reaches that trial
- **THEN** the retained prefix and budget diagnostic remain visible while aggregate statistics are withheld
