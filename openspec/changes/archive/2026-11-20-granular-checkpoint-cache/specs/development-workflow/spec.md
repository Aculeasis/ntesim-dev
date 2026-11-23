# Spec Delta

## MODIFIED Requirements

### Requirement: Focused checks follow maintained inputs

Checkpoint SHALL select the affected existing validation and build boundaries when maintained data files change, including dependency fingerprints used for reusable checks. Checkpoint SHALL cache successful test results at the package boundary using upstream dependency closures, skipping passing packages whose sources and transitive dependencies are unchanged. Deleted obsolete configuration inputs SHALL NOT fail classification. New unsupported input categories SHALL remain rejected.

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

#### Scenario: Downstream Rust crate edit reuses passing upstream crate tests
- **GIVEN** a previous checkpoint successfully validated all packages in a worktree
- **WHEN** only a downstream crate changes without modifying its upstream dependency crates
- **THEN** checkpoint reuses the successful test results of unchanged upstream packages
- **AND** executes tests only for packages whose upstream dependency closures are affected
