# Causal state observations

## Purpose

Defines causally placed observations of actual simulation state and their limits as audit records rather than restoration checkpoints.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Causal current state

An observation SHALL capture the state at its delivered effect boundary without advancing time, consuming RNG or predicting pending effects.

#### Scenario: Observation before delayed damage
- **GIVEN** an action spent a resource and consumed a roll but its delayed hit has not landed
- **WHEN** an observation is delivered
- **THEN** it sees the spent resource and consumed RNG state while target health still excludes the delayed hit

### Requirement: Observation limits

Observation records SHALL remain distinct from complete scheduler checkpoints and SHALL NOT reset state or imply that pending task contents can be restored from them.

#### Scenario: Repeated observation label
- **GIVEN** two observations share a cycle label
- **WHEN** the second observation is delivered
- **THEN** it captures continuing state without resetting resources or RNG
