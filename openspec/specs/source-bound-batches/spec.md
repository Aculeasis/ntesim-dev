# Source-bound research batches

## Purpose

Defines source-bound build compilation feeding research trials without hidden pilot execution or lost input provenance.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Compile once without a pilot

A source-bound batch SHALL compile its build request once and execute only the requested trials, retaining binding receipts and original request identities.

#### Scenario: Equivalent compiled scenarios
- **GIVEN** two different build request byte strings compile to the same scenario
- **WHEN** their batches are produced
- **THEN** compiled scenario identities can match while original build request identities remain distinct

### Requirement: Preserve research quality

Source-bound research batches SHALL retain incomplete model quality and withhold statistics for failed or truncated experiments.

#### Scenario: Complete research experiment
- **GIVEN** all requested research trials finish
- **WHEN** the result is exported
- **THEN** experiment completeness is reported without upgrading strict game or DPS acceptance flags
