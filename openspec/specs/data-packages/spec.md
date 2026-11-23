# Data package interfaces

## Purpose

Defines consumed package identity, validation, portability and explicit supplemental implementation parameters.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Package integrity

Retained data consumers SHALL validate required identities, schemas and integrity before accepting source-bound inputs. Ordinary simulation SHALL NOT require the retired historical raw-package directory. Required current inputs SHALL remain explicit; their absence SHALL NOT silently select fabricated data.

#### Scenario: Mismatched package identity
- **GIVEN** a request names an identity that does not match its package
- **WHEN** the source-bound request is loaded
- **THEN** validation rejects it rather than substituting another package

#### Scenario: Missing external raw package directory
- **GIVEN** the retired historical raw package directory is absent and required current inputs are available
- **WHEN** simulation execution or source compilation is performed
- **THEN** execution proceeds without requiring the historical directory

#### Scenario: Missing current input
- **GIVEN** a retained operation requires a dataset that is absent or invalid
- **WHEN** the operation loads its inputs
- **THEN** it reports the input error instead of substituting an empty dataset, reduced sample or invented inventory

### Requirement: Explicit supplemental values

Supplemental implementation parameters SHALL retain their applicability, units and provenance without inventing absent source details.

#### Scenario: Absent versus zero
- **GIVEN** one source omits a parameter and another explicitly records zero
- **WHEN** the parameters are projected
- **THEN** their distinction remains available to the consuming validation and model

### Requirement: Consistent dataset reports

Reports and manifests for the same operation SHALL derive dataset and execution counts from validated inputs and actual results. Custom datasets SHALL retain their own counts and incomplete or unsupported outcomes.

#### Scenario: Custom variant inventory
- **GIVEN** a valid custom dataset contains one variant and one archetype
- **WHEN** deliverables and their manifest are generated for that dataset
- **THEN** both report one variant and one archetype
- **AND** neither reports historical default inventory counts

#### Scenario: Mixed execution outcomes
- **GIVEN** a retained comparison evaluates a dataset with both completed and failed or unsupported variants
- **WHEN** the comparison report is generated
- **THEN** it preserves those outcomes and the required incomplete status without substituting an all-success sample
