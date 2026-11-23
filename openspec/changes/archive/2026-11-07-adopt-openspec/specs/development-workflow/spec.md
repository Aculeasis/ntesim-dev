# Repository development workflow

## Purpose

Defines one OpenSpec workflow for selecting, resuming, checking and completing changes without a parallel queue or readiness ledger.

## ADDED Requirements

### Requirement: Selected resumable change

Planned work SHALL keep its scope, design, behavioral deltas and verifiable tasks in one selected OpenSpec change; continuation SHALL resume that change rather than inventing another task.

#### Scenario: Continue after interruption
- **GIVEN** a selected change has unfinished tracked tasks
- **WHEN** work resumes after an interruption
- **THEN** the agent reads the saved artifacts and continues those tasks

### Requirement: Single requirement ownership

Accepted behavioral requirements SHALL live under openspec/specs, with linked detailed references and mechanics evidence; obsolete duplicate contracts and workflow ledgers SHALL be removed with their consumers updated.

#### Scenario: Move a contract
- **GIVEN** a current contract is migrated into a capability
- **WHEN** the migration is delivered
- **THEN** its current consumers resolve to that capability and the obsolete document is absent

### Requirement: Verification before completion

Delivery SHALL run the applicable existing native, transport or browser checks and checkpoint, including strict OpenSpec validation; document validity SHALL remain distinct from simulator correctness.

#### Scenario: Tooling-only change
- **GIVEN** a change modifies workflow documents and Python tooling without engine or web changes
- **WHEN** checkpoint selects its checks
- **THEN** document, OpenSpec and Python checks run without treating them as mechanics or browser acceptance

### Requirement: Complete archive

A completed change SHALL synchronize its behavioral deltas into accepted specs and retain completed tracked tasks in its OpenSpec archive.

#### Scenario: Finish a behavioral change
- **GIVEN** all tracked work and applicable checks have passed
- **WHEN** the change is archived
- **THEN** its accepted deltas are in the main specs and its archived task list has no unfinished checkboxes
