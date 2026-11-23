# Proposal

## Why

Historical package dependencies, stale hash gates, Java quality tooling and obsolete verifiers complicate ordinary development. Cleanup must establish which capabilities and inputs remain necessary: replacing deleted files with inline catalogs, empty defaults or smaller test samples preserves the old dependency while losing data and coverage.

## What Changes

- Review each affected command, adapter and dataset against accepted requirements, documented user workflows and actual consumers. Record concrete retain/migrate/remove decisions in this change's design; a historical name, absent internal caller or deleted input file is insufficient evidence for removing a public capability.
- **BREAKING**: Retire `data/global-2026-10-04` and raw audit dumps after required consumers have migrated. Remove commands and adapters whose sole purpose is the retired package; preserve needed capabilities through explicit current inputs. Resolve material uncertainty about capability removal with the user.
- Store retained tabular presets and shared test datasets in maintained CSV files with a concrete consumer, documented schema and applicable source attribution. Do not paste these datasets into Rust, TypeScript or tooling source strings. Keep consumed combat constants typed in Rust and raw acquisitions in `cache/`.
- Replace duplicated character inventories with one authoritative typed registry, preserve missing-input errors, and derive report counts from actual inputs and execution results. Remove empty package placeholders, invented fallback coverage and references to nonexistent default files.
- **BREAKING**: Permanently delete `configs/` and `tools/quality/`. Remove PMD/Java quality scripts (`tools/rust_quality.py`, `tools/rust_quality_reports.py`, `tools/quality_tools.py`, `tools/setup_rust_quality.py`) and use standard `cargo clippy` and `cargo fmt`.
- Remove active dependencies on external scrape layouts and obsolete preset paths. Preserve truthful attribution and applicability for retained data; cleanup must not erase evidence of its origin.
- Keep Team list DPS as the last successful precomputation during local edits, ordinary builds and checks. Recalculate only on explicit request and during release/publication preparation; source edits or WASM rebuilds alone must not trigger or require recalculation. Preserve reproducibility metadata and real cache identities when a calculation is requested.
- Retire `reviews-legacy.sha256` and obsolete local hash gates while retaining structural validation and integrity checks required by surviving source-bound consumers.
- Retire obsolete verifiers and data package tools after checking their purpose and coverage; migrate distinct checks for retained capabilities into the appropriate existing boundary.
- Reduce the number of tests and repeated expensive work by removing obsolete cases and consolidating overlapping checks at their owning boundary. Reuse existing parameterized checks for necessary CSV/cache/checkpoint regressions instead of adding parallel suites. Profile toward sixty seconds under the shared four-CPU limit while retaining distinct numeric, causal, failure, incomplete-result and transport guarantees.
- Fix checkpoint classification of deleted `configs/` files and register maintained CSV consumers in focused check selection and dependency fingerprints.
- Repair documentation, notices, links and supported command examples alongside their consumers.

## Capabilities

### Modified Capabilities

- `data-packages`: Retire historical raw-package dependencies while preserving required current inputs, validation, explicit failures and accurate report inventories.
- `development-workflow`: Require justified removal, maintained CSV datasets, fewer redundant tests and focused dependency-aware verification without obsolete tooling or ephemeral receipts.
- `studio-application`: Allow last-calculated DPS during ordinary development, with explicit refresh and mandatory recalculation before release/publication.

## Impact

- **Rust/native CLI**: Needed commands use explicit current inputs or maintained bundled data; package-only interfaces and their consumers are removed together. Missing package metadata cannot silently select fabricated defaults.
- **WASM/Worker/Studio**: Retained APIs consume the same maintained inputs as native execution. Bundling a CSV file is allowed; handwritten copies of its contents in source are not. Ordinary development reuses last-calculated DPS; explicit refresh and release preparation calculate against current inputs.
- **Tests/tooling**: Java/PMD and obsolete verification dependencies are retired with evidence that distinct retained checks remain covered. Performance measurements state their scope and prerequisites.
- **Documentation/data**: Current schemas, consumers, attribution and supported commands stay accurate; obsolete links and unused inputs are deleted.
