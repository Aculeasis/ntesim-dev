# Spec Delta

## ADDED Requirements

### Requirement: Justified capability retirement

Cleanup SHALL establish the purpose, requirements and consumers of affected capabilities before removing them. Retained workflows SHALL migrate with their inputs, validation, tests and documentation. Material uncertainty about removal of a public capability SHALL be resolved with the user.

#### Scenario: Command without internal callers
- **GIVEN** a public command has no internal caller but serves a documented current user workflow
- **WHEN** historical package dependencies are retired
- **THEN** the workflow remains supported through current inputs
- **AND** absence of internal callers alone does not justify deleting it

#### Scenario: Retired package-only interface
- **GIVEN** review establishes that an interface serves only the retired package and no retained requirement or workflow needs it
- **WHEN** that interface is removed
- **THEN** its entry points, adapters, unused inputs and obsolete active documentation are removed together

### Requirement: Maintained tabular inputs

Retained tabular presets and shared tabular test datasets SHALL be maintained as CSV files with concrete consumers, documented schemas and applicable provenance. Source files SHALL NOT contain handwritten copies of those datasets. Consumed combat constants SHALL remain typed Rust content.

#### Scenario: Shared team dataset
- **GIVEN** native, WASM or test consumers need the same team dataset
- **WHEN** the maintained CSV changes
- **THEN** those consumers read or bundle that file through their normal loading or build boundary
- **AND** no duplicate dataset literal must be edited in implementation or test source

#### Scenario: Parser-specific literal
- **GIVEN** a parser test checks a delimiter, malformed header or another isolated text-format case
- **WHEN** its minimal input is authored
- **THEN** a local literal is permitted without creating a shared preset catalog

### Requirement: Bounded test execution and verifier boundaries

Cleanup SHALL reduce redundant tests and target sixty seconds for core checks under the shared four-CPU limit, with measurement scope and prerequisites stated. Distinct retained guarantees SHALL remain checked at their owning boundary. Verifiers SHALL run without obsolete package directories or prior unversioned review receipts. Routine development SHALL NOT require DPS refresh.

#### Scenario: Focused core check execution time
- **GIVEN** a supported checkout with documented cache and build prerequisites
- **WHEN** the selected core suites are profiled under the CPU limit
- **THEN** measured execution time and suite scope are reported against the sixty-second target
- **AND** distinct behavioral coverage is preserved even if the target remains unmet

#### Scenario: Verifier independence from ephemeral artifacts
- **GIVEN** a clean checkout without previous cache review outputs
- **WHEN** verification scripts in tools/ execute
- **THEN** verifiers validate current code and maintained inputs without failing on missing legacy review receipts

#### Scenario: Independence from external site presets
- **GIVEN** a checkout with required current inputs and no historical scrape directory
- **WHEN** a retained workflow runs
- **THEN** it uses maintained inputs without requiring the obsolete scrape layout
- **AND** any retained data keeps its applicable source attribution

#### Scenario: Consolidate redundant checks
- **GIVEN** multiple tests or verifier scripts repeat the same behavior at overlapping boundaries
- **WHEN** cleanup reduces the suite
- **THEN** redundant cases are removed or combined into an existing owning boundary
- **AND** each distinct required guarantee remains checked without one-for-one replacement of deleted tests

#### Scenario: Preserve a distinct regression case
- **GIVEN** a slow test or obsolete verifier checks a distinct failure or incomplete-result case for a retained capability
- **WHEN** cleanup optimizes or retires that check
- **THEN** the behavior remains verified at an appropriate existing boundary
- **AND** lowering expected counts on a smaller success-only sample does not count as preserving that case

#### Scenario: Local Studio source edit
- **GIVEN** a structurally valid Studio catalog whose recorded engine or WASM source hash differs after a local edit
- **WHEN** the development server, ordinary build or routine checks run
- **THEN** source-hash disagreement alone neither blocks the operation nor triggers DPS recalculation
- **AND** required structural validation remains active

### Requirement: Focused checks follow maintained inputs

Checkpoint SHALL select the affected existing validation and build boundaries when maintained data files change, including dependency fingerprints used for reusable checks. Deleted obsolete configuration inputs SHALL NOT fail classification. New unsupported input categories SHALL remain rejected.

#### Scenario: Test fixture changes without Rust edits
- **GIVEN** only a maintained crate-local CSV test fixture changes
- **WHEN** checkpoint selects checks
- **THEN** the owning test boundary is selected and its cached success depends on that CSV
- **AND** unrelated browser and full regression suites are not selected solely for that fixture

#### Scenario: Bundled CSV changes
- **GIVEN** a maintained CSV is bundled into native or WASM artifacts
- **WHEN** that CSV changes without an implementation source edit
- **THEN** checkpoint selects the affected artifact rebuild and minimal applicable validation boundary

#### Scenario: Obsolete configuration deletion
- **GIVEN** tracked files under the retired configuration directory are deleted
- **WHEN** checkpoint classifies the deletion diff
- **THEN** it selects applicable tooling validation without rejecting the deleted paths as new unknown inputs
- **AND** introducing new files into that unsupported directory remains rejected
