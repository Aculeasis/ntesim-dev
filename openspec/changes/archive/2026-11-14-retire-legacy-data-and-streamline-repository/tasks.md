# Tasks

The former completed hash-gate task 2.1 is reopened for the explicit-refresh policy. Java-tooling and legacy-review-hash removal retain their prior completion status at 3.1 and 3.2; this planning update does not certify their receipts. Tasks are reordered so data extraction and consumer migration precede package deletion.

## 0. Consumer and capability review

- [x] 0.1 Trace affected public CLI/WASM entry points, Studio/Worker callers, documented manual workflows, data readers and verifiers against accepted requirements. Record concrete retain/migrate/remove decisions with references in design.md; resolve material removal ambiguity with the user. Verify every affected capability and dataset has a justified disposition.

## 1. Preserve needed inputs, migrate consumers, then delete

- [x] 1.1 Restore or extract needed curated team/preset tables and shared test datasets into maintained CSV files before accepting their old locations' deletion. Document schemas, consumers and applicable provenance; verify required rows and distinctions, parser acceptance and removal of handwritten dataset copies from source.
- [x] 1.2 Replace duplicated character arrays and coverage catalogs with one authoritative typed inventory, preserving explicit availability where required. Remove unused empty package constants; reuse existing inventory/build checks with independent expectations to verify the current contract.
- [x] 1.3 Remove the implicit missing-manifest fallback from `constant_report`; migrate retained operations to explicit current inputs and remove obsolete package identities without shims. Reuse existing checks for required missing/malformed/mismatched input failures and intentionally package-independent operations.
- [x] 1.4 Update retained native, WASM/Worker and distribution consumers to read or bundle the maintained CSVs. Run the minimal applicable existing transport/package smoke checks, including execution outside the checkout root where supported, and verify no hidden one-team production default remains.
- [x] 1.5 Derive deliverable and manifest counts from the same validated inputs and results. Consolidate the existing acceptance cases to verify custom-dataset count agreement and mixed completed/failed or unsupported outcomes.
- [x] 1.6 After 1.1-1.5 and the applicable CSV check-selection work in 3.4, retire `data/global-2026-10-04`, raw audit dumps and confirmed obsolete package-only interfaces with their adapters, unused inputs, tests and active documentation. Verify retained workflows run without the historical directories and current CLI help/examples match the resulting surface.
- [x] 1.7 Remove active dependencies on obsolete scrape/preset layouts and repair references. Preserve truthful attribution, applicability and required notices for retained inputs and mechanics evidence; verify references resolve and retained workflows need no historical scrape directory.

## 2. Explicit DPS refresh and correct calculation cache

- [x] 2.1 Update Studio/catalog validation, ordinary build/check entry points and development docs to allow last-calculated DPS after source edits or WASM rebuilds without automatic refresh or freshness failure. Reuse existing structural checks to verify local commands remain usable and do not launch recalculation.
- [x] 2.2 Restore real engine, WASM and generator identities in calculation metadata and cache keys alongside scenario inputs/options. Consolidate existing tooling checks to verify unchanged explicit requests can reuse cache, changed calculation inputs cannot reuse old results, and local edits alone do not trigger calculation.
- [x] 2.3 Keep on-demand regeneration explicit and wire complete fresh catalog calculation into release/publication preparation after the current engine/WASM build. Reuse the existing refresh/release boundary to verify all delivered selectable scripts are recalculated, failed/incomplete/unsupported results remain unavailable, and ordinary builds/checkpoints do not execute this release-only work.
- [x] 2.4 Remove routine full-DPS recomputation and stale-source rejection tests made obsolete by the new policy. Keep lightweight catalog structure/source-loading checks and bounded refresh/cache cases at the existing owner; verify routine tests do not calculate the full catalog merely because sources changed.

## 3. Repair checkpoint and reduce redundant tests/tooling

- [x] 3.1 Delete Java PMD/CPD quality tools (`tools/rust_quality.py`, `tools/rust_quality_reports.py`, `tools/quality_tools.py`, `tools/setup_rust_quality.py`, `docs/RUST_QUALITY.md`) and rely directly on standard `cargo clippy` and `cargo fmt`.
- [x] 3.2 Remove `configs/reviews-legacy.sha256` and retire `tests/test_reviews_policy.py`, and verify Python unit tests pass cleanly.
- [x] 3.3 Fix checkpoint classification of deleted `configs/` files without permitting new files in that retired directory. Extend the existing selection matrix/rejection case and run those focused checks; verify deletion selects tooling validation instead of raising the unknown-input error.
- [x] 3.4 Register actual maintained CSV locations in affected check selection, dependency fingerprints and native/WASM build inputs. Extend the existing selection matrix to verify test-only CSV edits select owning tests, bundled CSV edits rebuild affected artifacts, and unrelated full browser suites stay unselected.
- [x] 3.5 Review removed verifiers and checkpoint consumers; retire confirmed obsolete/redundant checks, retain unique guarantees at existing boundaries and update `tools/verifier-paths.txt` and policy tests. Run focused tooling checks and verify discovery and checkpoint reference only surviving tools.
- [x] 3.6 Reduce the test suite by deleting obsolete cases and consolidating overlaps, including already deleted CLI tests and reduced research/WASM fixtures. Restore only an otherwise lost distinct guarantee for retained behavior; use existing parameterized cases instead of one-for-one replacements or new suites. Report removed/consolidated tests and run the affected owning checks.
- [x] 3.7 Profile selected native, web and Python suites under the shared four-CPU limit, one validator at a time. State suite scope, build/cache prerequisites and elapsed time against the sixty-second target; eliminate measured duplication and report any unmet target without weakening distinct required guarantees or forcing full DPS refresh.

## 4. Integration and delivery

- [x] 4.1 Review active docs and examples for obsolete paths/commands/tooling and the explicit DPS refresh/release policy, preserving source attribution and evidence; verify documented workflows match their migrated contracts.
- [x] 4.2 Run the existing Markdown/link validation boundary and verify links resolve without forwarding stubs or nonexistent maintained inputs.
- [x] 4.3 Validate OpenSpec with `python tools/openspec.py validate --all --strict --no-interactive --concurrency 4`; verify consistency between proposal, design, all three capability deltas and implementation tasks.
- [x] 4.4 Review the stable diff, reuse valid receipts and run `python tools/checkpoint.py run` before committing. Verify selected native, transport/browser, tooling and document checks pass without routine DPS refresh; report behavior, test reductions, measured performance and material limits.

## Workflow follow-up

- Archive only after revised tasks and applicable checks are complete, synchronizing accepted specs with implementation.
- Archive with `python tools/openspec.py archive retire-legacy-data-and-streamline-repository --yes`.
- Publication remains separately authorized. Before an actual release/publication, run the explicit preparation workflow that recalculates the delivered catalog using current engine/WASM.
