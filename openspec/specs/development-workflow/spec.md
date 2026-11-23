# development-workflow Specification

## Purpose
Defines one OpenSpec workflow for selecting, resuming, checking and completing changes without a parallel queue or readiness ledger.

## Requirements

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

### Requirement: Optional private development configuration

Shared instructions SHALL identify one optional ignored location for machine-specific supplements without replacing or duplicating shared instructions. Private paths, permission grants, proxy endpoints and credentials SHALL remain outside versioned project inputs. Ordinary development checks SHALL work without private configuration.

#### Scenario: Checkout without local supplements
- **GIVEN** a supported machine has the documented prerequisites and no private configuration
- **WHEN** a developer follows shared instructions
- **THEN** ordinary setup and checks do not require the author's directories, credentials or permission grants

#### Scenario: Local instructions supplement shared rules
- **GIVEN** optional local instructions exist in the documented ignored location
- **WHEN** an agent starts work in this checkout
- **THEN** it reads those supplements alongside the shared guide without requiring an override copy of that guide

### Requirement: Explicit external acquisition inputs

Tools requiring a local client installation SHALL resolve its binary directory from an explicit environment setting and SHALL fail with an actionable configuration error before reading client files when that setting is missing or invalid. Explicitly selected environment files SHALL be parsed as data without shell execution or implicit variable expansion; inherited environment values SHALL take precedence.

#### Scenario: Missing client location
- **WHEN** native acquisition is requested without a configured binary directory
- **THEN** the tool names the required setting and does not probe an assumed installation

#### Scenario: Environment file contains shell syntax
- **GIVEN** an explicitly selected environment file contains a value with shell metacharacters
- **WHEN** the command runner loads that file
- **THEN** the value is passed literally to the child and no shell expression is executed

### Requirement: Repository portability and privacy validation

Checkpoint SHALL reject commit-eligible private configuration, machine-specific absolute paths and recognizable credential literals, reporting file, line and category without echoing the private value. Synthetic path-rejection fixtures and public attribution SHALL remain valid. Evidence metadata SHALL retain source identity without recording the developer's filesystem layout.

#### Scenario: Forced addition of ignored private configuration
- **GIVEN** a local environment or permission file is forcibly added to Git
- **WHEN** checkpoint validates repository inputs
- **THEN** validation fails even though the file normally matches an ignore rule

#### Scenario: Machine path in evidence metadata
- **WHEN** source metadata contains a developer's absolute filesystem path
- **THEN** checkpoint rejects it without printing that path's value

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

### Requirement: Single maintained Studio preset source

Studio preset definitions SHALL be maintained in `presets/studio/index.json` and its referenced `.nte` files. The derived browser bundle `web/src/generated/script-presets.json` SHALL be ignored by Git and absent from tracked repository inputs. Generating that bundle SHALL NOT require editing another copy of a preset.

#### Scenario: Track a preset edit
- **GIVEN** a developer changes a maintained Studio `.nte` file or its manifest metadata
- **WHEN** the browser preset bundle is regenerated
- **THEN** its entries contain the current manifest metadata and exact script text
- **AND** the generated JSON does not appear among untracked or modified Git files

#### Scenario: Fresh checkout contains maintained sources only
- **WHEN** the repository is checked out without local generated artifacts
- **THEN** the Studio preset manifest and referenced `.nte` files are present
- **AND** no tracked copy of `web/src/generated/script-presets.json` is required

### Requirement: Automatic browser preset preparation

Supported Studio build, development and web validation commands SHALL prepare the browser preset bundle from maintained sources before any consumer reads it. A missing bundle SHALL require no manual preparation. Preparation SHALL preserve preset identity, ordering, labels, selectability, default selection and exact script text, without triggering DPS recalculation.

#### Scenario: Build without a preexisting bundle
- **GIVEN** a checkout has documented build prerequisites and no generated browser preset bundle
- **WHEN** the ordinary Studio build runs
- **THEN** it generates the bundle before compiling its consumers
- **AND** the built site loads the exact maintained source of each selected preset without access to the repository filesystem

#### Scenario: Validate without a preexisting bundle
- **GIVEN** a checkout has documented validation prerequisites and no generated browser preset bundle
- **WHEN** a supported web type check, format check, test or Team list validation command runs
- **THEN** the command prepares the bundle before a consumer needs it
- **AND** it does not fail solely because that generated file was initially absent
- **AND** it does not recalculate saved Team list DPS

#### Scenario: Develop and edit a preset
- **GIVEN** a checkout has documented development prerequisites and no generated browser preset bundle
- **WHEN** the Studio development server starts and a maintained preset is subsequently edited
- **THEN** initial loading succeeds and the served preset source updates to the edited text
- **AND** neither operation requires committing generated JSON or recalculating DPS

#### Scenario: Invalid maintained input
- **GIVEN** the preset manifest is invalid or a referenced script is missing or empty
- **WHEN** browser preset preparation runs
- **THEN** the command or development server reports the input error
- **AND** it does not silently serve an old generated bundle as the current preset collection
