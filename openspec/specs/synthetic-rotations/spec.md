# Synthetic rotation timelines

## Purpose

Defines bounded synthetic timeline source compilation, continuous repeated actions and source-linked request identity.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Explicit synthetic timeline

Synthetic timeline compilation SHALL retain authored ordering, declared action timing and source identity without claiming legal game rotations.

#### Scenario: Repeat an authored timeline
- **GIVEN** a synthetic rotation repeats an action sequence
- **WHEN** the compiled timeline runs
- **THEN** resources and deadlines continue across repetitions without hidden resets

### Requirement: Bounded translation

Synthetic rotation inputs SHALL reject malformed syntax and work beyond declared translation or execution bounds.

#### Scenario: Malformed command
- **GIVEN** a timeline contains an unknown command
- **WHEN** it is compiled
- **THEN** a diagnostic rejects the input instead of silently dropping the command
