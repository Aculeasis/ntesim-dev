# Synthetic seed batches

## Purpose

Defines explicit synthetic seed ensembles, exact aggregate identities and all-or-none completeness of reported statistics.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Explicit independent seeds

A synthetic batch SHALL execute each distinct explicit seed from the original input state while preserving request order and continuous state within each run.

#### Scenario: Independent runs
- **GIVEN** two explicit seeds share one fixture and rotation
- **WHEN** the batch executes
- **THEN** each starts from the fixture state and the output retains seed order

### Requirement: All-or-none statistics

Aggregate synthetic batch statistics SHALL be withheld if any requested run fails or has pending events.

#### Scenario: One incomplete seed
- **GIVEN** one of the requested seeds ends with a pending hit
- **WHEN** the batch report is assembled
- **THEN** its partial row remains visible, aggregate target statistics are null and the CLI exits 2
