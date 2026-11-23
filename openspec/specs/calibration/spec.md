# Independent calibration and arithmetic checks

## Purpose

Defines independently attributed arithmetic comparisons, caller tolerances and honest limits on what calibration establishes.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Declared tolerance

Tabular checks SHALL compare the documented raw formula values against explicit references and tolerances; a documented discrepancy SHALL remain distinct from a passed check.

#### Scenario: Documented discrepancy exceeds tolerance
- **GIVEN** a case exceeds its tolerance and includes a discrepancy note
- **WHEN** its report is generated
- **THEN** it is classified as a documented discrepancy, does not increment passed cases and produces incomplete CLI exit 2

### Requirement: Zero-reference semantics

A zero reference SHALL match only zero computed damage; percentage tolerance SHALL NOT conceal a nonzero result against a zero reference.

#### Scenario: Nonzero damage against zero
- **GIVEN** a case has reference damage zero
- **WHEN** the computed raw damage is nonzero
- **THEN** the case rejects the undefined percentage comparison

### Requirement: Evidence scope

Passing arithmetic checks SHALL retain applicability and model quality and SHALL NOT establish complete kit, team DPS or native game correctness.

#### Scenario: Synthetic oracle passes
- **GIVEN** a synthetic direct-damage oracle matches exactly
- **WHEN** the verification report is exported
- **THEN** the arithmetic passes while the report retains its stated incomplete game-model scope
