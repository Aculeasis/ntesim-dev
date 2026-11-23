# Design

## Context

See [proposal.md](proposal.md) for motivation and scope. The current implementation has deleted historical package files while retaining readers and replacing some inputs with inline data. This is unfinished migration, not evidence that either the data or its consumer is unnecessary.

The reviewed working tree contains inline team CSVs in research tests and `crates/data/src/package.rs`, repeated 23-character inventories, empty package constants, and a missing-manifest branch in `crates/cli/src/package/mod.rs` that fabricates coverage. Some tests now exercise one or seven variants instead of the former 45. Acceptance deliverables use input-derived counts while their manifest still defaults to 45 variants and 16 archetypes. These are concrete repair targets, not a requirement to preserve the historical sample size.

## Goals / Non-Goals

**Goals:**

- Finish migration by tracing each affected capability from its public entry point to its inputs and outputs.
- Give retained tables one maintained CSV source and preserve validation, applicability and behavioral coverage.
- Keep native and WASM consumers consistent and ordinary execution independent of raw acquisition directories.
- Retire obsolete package infrastructure, Java tooling and stale local-development gates without hidden fallbacks.
- Reduce redundant tests and routine recalculation while retaining focused checks at the owning boundary.

**Non-Goals:**

- Changing combat formulas, character mechanics or the treatment of unsupported dependencies.
- Restoring a historical package wholesale or preserving obsolete input compatibility.
- Removing public capabilities merely because their implementation refers to an old package.
- Meeting a test-time budget by weakening the scenarios being checked.

## Decisions

### Decision 1: Establish purpose before removal

For each affected command, adapter and dataset, inspect accepted requirements, documented/manual user workflows, public CLI and WASM entry points, Studio/Worker callers, tooling and tests. Record the concrete disposition and supporting references here as implementation proceeds; do not create a separate tracking ledger.

- **Retain and migrate** when a current capability needs the input or behavior. Specify its current consumer, canonical input, validation and replacement for the old package dependency.
- **Remove together** when the interface exists solely to inspect or maintain the retired package and no retained requirement or workflow needs it. Remove its entry points, implementation, unused inputs, tests that only check the removed feature and active documentation.
- **Resolve ambiguity** when evidence cannot establish whether a public workflow is still required. Keep that capability intact pending a focused user decision; absence of internal callers is insufficient.

The current proposal and the age or name of a file are not independent evidence of obsolescence. Apply the same reasoning to verifiers and already deleted CLI tests before accepting their removal.

Consumer review dispositions:

| Boundary/input | Disposition and evidence |
| --- | --- |
| Native stat/build/projection and game commands | Retain: `reports/stats.rs`, `reports/simulation.rs`, native suites and documented Runbook workflows use the same typed compilers as Studio. Remove the obsolete package-directory argument and audited-file wrapper; use current typed inputs directly. |
| Published variants, compositions, gear binding, acceptance, pilot and finalist commands | Retain as explicitly incomplete research tools: Runbook documents them and WASM exposes composition enumeration. Maintain curated variant/gear CSVs and authored acceptance conditions; custom paths remain explicit. |
| `audit`, `coverage`, `gaps`, package `validate`, raw normalization/storage adapters | Remove: they inspect the retired manifest/source/coverage package or its partial dependency flags. Current executable source validation remains `validate-script`; source inspection and execution diagnostics retain the supported behavior. No Studio consumer uses raw-package inspection. |
| Historical character JSON, raw coverage/source/storage manifests and partial model exports | Remove: typed character definitions, content profiles and authored simulation requests own current consumers. Derive released inventory from explicit availability in typed definitions rather than copying raw metadata. |
| `inspect_game_data.py`, its presentation-only tests and `studio_growth_oracle.py` | Retire: these adapters dump the removed raw table layout or import the deleted review fixture reader. Supported stat/catalog inspection remains in package-free native commands and typed Rust checks. Acquisition tools still accept explicit independent workspaces; the obsolete frozen-directory exclusion is removed. |
| Team-list generated DPS and scenario source bundle | Retain: Studio picker and transport/source-loading checks consume them. Keep prior calculation identity locally; recomputation belongs to explicit refresh and release preparation. |
| Java/PMD, retired snapshot verifiers and deleted CLI kit suites | Retire duplicates of owning Rust mechanics and current source/Worker boundaries. Preserve source execution/export/replay, distribution, actual Worker and independent numeric/causal checks; migrate unique retained cases into those existing boundaries rather than restoring copied suites. |

### Decision 2: Maintain required tables as CSV files

Preserve genuinely needed team lists, presets and shared tabular test inputs as ordinary checked-in CSV files. Use a maintained location matching ownership, such as `presets/research/` for retained research presets or crate-local `tests/fixtures/` for test-only inputs. Select the concrete file paths and schema after the consumer review; reuse an existing suitable file rather than creating duplicate tables.

Each retained CSV must have a concrete consumer, documented columns and identity rules, applicable provenance and a named validation boundary. Preserve required rows and distinctions. An obsolete directory name does not justify dropping data. Retaining a curated input does not authorize copying an unused raw export or reintroducing a runtime content catalog.

Use the same file for consumers that require the same dataset. Native readers may load it explicitly; a packaged binary or WASM module may bundle that file with `include_str!` or the existing build pipeline. The file remains the editable source of truth. Do not paste its contents into Rust, TypeScript or tooling literals, generate a source file containing a second handwritten table, or use one test team as the production default.

This supersedes the former blanket ban on preserving any file from the historical package and on compile-time file inclusion. It also supersedes replacing shared tabular fixtures with source strings. Small literals for delimiter, quoting, invalid-header or similar parser-specific cases remain appropriate.

### Decision 3: Preserve typed domain ownership and explicit inputs

Consumed combat constants remain typed Rust content. Character identity and supported inventory come from one authoritative typed registry in the existing data/content ownership boundary; remove repeated ID arrays and JSON coverage catalogs. Represent release availability explicitly if a retained consumer requires it rather than inferring it from simulation support.

Remove unused constants such as empty character and gear placeholders. Supply real conditions where required; an empty value is valid only when the current contract explicitly permits it. Replace stale default paths with existing maintained files or explicit request inputs.

Remove the automatic missing-`manifest.json` branch that supplies empty tables and synthetic coverage. If a retained function becomes package-independent, give it an explicit current contract and update callers and tests together. A missing, malformed or mismatched required input must remain an actionable error. Remove obsolete package identities from migrated contracts rather than adding compatibility shims.

### Decision 4: Derive reports from the run

Compute composition, variant and archetype counts from the validated inputs and actual enumeration; pass those values into the manifest. Manifest and deliverable totals must agree for the same run, including custom datasets. Preserve explicit incomplete and unsupported statuses and independent successful/failed execution cases.

Do not repair the contradiction by changing manifest assertions to another hardcoded historical count or by replacing an incomplete mixed-result sample with a sample that only succeeds.

### Decision 5: Prune tooling with coverage preserved

Replace Java/PMD tools with `python tools/cargo.py clippy --workspace --all-targets --locked --offline -- -D warnings` and `python tools/cargo.py fmt --all -- --check`. Update verifier discovery and checkpoint consumers when removing configuration and obsolete tools.

Reduce the test suite: remove obsolete cases, consolidate overlapping checks and keep each guarantee at its appropriate existing mechanics, package, replay, transport or UI boundary. Do not restore every deleted test or replace every removed verifier one-for-one. Restore or migrate only a distinct guarantee for retained behavior that would otherwise be lost; prefer extending an existing parameterized case over adding a new test or suite. Do not introduce a new verifier framework, copied kit suites or redundant assertions across layers.

Use maintained CSV fixtures for retained shared tabular scenarios. A smaller focused dataset is acceptable when it still checks the intended behavior, including errors and incomplete results. Historical counts and hashes are not substitutes for that behavior, and lowering expected totals does not demonstrate preserved coverage.

Profile before optimizing. Separate build/startup costs from measured test execution, state which suites and cache prerequisites the sixty-second target covers, and obey the shared four-CPU limit with one validator at a time. Report an unmet target rather than deleting distinct cases to claim success.

The initial full content run identified the continuing Tiger Tally case as taking over sixty seconds by itself. Retain its independent damage-zone, boss, expiry and continuing-energy checks and its source causal replay; remove the repeated generic JSON roundtrip/full re-execution inside each refinement/boss case, which the shared transport boundary already covers. Apply the existing moderate dev-package optimization policy to core and content as well as data/CLI (`opt-level = 1`), retaining debug assertions and overflow checks. This trades some compilation time for faster exact-arithmetic tests without changing mechanics or weakening assertions.

The same Tiger Tally timeline is source-executed and replayed once: refinement changes its damage/ATK parameters, not admission clocks or energy grants. Both endpoint refinements are then bound through the production native projection against that timeline, preserving every independent expected value and expiry/resource assertion. Existing projection tests retain all five refinement curves.

Engine identity fingerprints include runtime Rust sources, manifests/toolchain and bundled research inputs, excluding separate test modules. Test-only edits do not change runtime identity or force every dependent crate to rebuild. Checkpoint test-success fingerprints still include test sources and fixtures, so this does not permit reuse of an untested changed test suite.

Measurement scope is the full native workspace plus Python tooling on this Windows host under the shared four-CPU limit; WASM/browser integration and compilation are separate costs. The observed native binary execution totals about 90.6 seconds and Python tooling about 26.9 seconds, so the combined sixty-second target remains unmet. The focused continuing Tiger Tally case completes in about 30.8 seconds after consolidation. Compared with HEAD, this cleanup removes 103 Rust and 60 Python test functions without adding test functions; distinct retained assertions and transport checks remain. Raw timings/build costs stay in `cache/checkpoint/` and `cache/tool-logs/`; these measurements do not claim a universal host budget or permit weakening checks.

### Decision 6: Refresh DPS explicitly and before release

Keep the last successfully calculated Team list DPS visible during local source edits, WASM rebuilds, ordinary application builds and routine checks. These operations must neither start recalculation nor fail because calculation metadata differs from current sources. Browsing remains passive. Structural validation remains required. Existing numeric values describe their recorded calculation inputs; source/engine metadata must remain truthful.

Use the existing explicit team-generation command for on-demand updates. Before release/publication, prepare the current engine/WASM and recalculate every delivered selectable script; validate those newly calculated outcomes before packaging for delivery. This is a separate release preparation boundary, not a condition on every local build or ordinary checkpoint. A local distribution smoke build does not imply publication or require fresh DPS unless it is explicitly preparing release contents. Failed, incomplete and unsupported fresh results remain unavailable rather than acquiring an old successful number.

Keep real source, engine, WASM and generator identities in calculation metadata and cache keys together with scenario options. Source edits may change the key for a future explicit request but must not themselves schedule computation. A requested update cannot reuse an old-engine result; release refresh calculates every delivered entry without relying on prior calculation cache. Keep the existing explicit forced-refresh mechanism and verify the relevant cache behavior within the existing tooling test boundary.

Remove routine tests that perform full DPS recalculation merely to compare the committed catalog with the current engine. Retain lightweight structure/source-loading checks and test refresh/cache behavior with bounded existing fixtures. Full catalog recomputation belongs to the explicit refresh/release workflow.

### Decision 7: Select focused checks for maintained files

Fix checkpoint handling of removed `configs/` inputs: classify deletion paths without failing the unknown-input guard, while newly introduced files in that obsolete directory remain rejected. Select the tooling checks required for the deletion and extend the existing selection matrix and rejection case.

Once CSV ownership and locations are known, register them in checkpoint selection, relevant dependency fingerprints and build inputs. A change to a test-only CSV selects its owning existing test boundary; a CSV bundled into native/WASM artifacts selects the affected rebuild and minimal transport smoke boundary. It must not fall through to privacy-only checks or automatically select the full browser suite. Combine cases in the existing selection matrix rather than creating separate tests for each path.

### Decision 8: Repair references

Remove the legacy review hash file. Preserve structural/schema validation and surviving source-bound identity and integrity requirements.

Remove dependencies on obsolete scrape layouts and active links to deleted inputs. Preserve truthful attribution and applicability for retained data and mechanics evidence, including required notices. Do not purge source identity solely because an adapter or path has been retired. Update current examples and documentation to the migrated contract without forwarding stubs.

## Risks / Trade-offs

- **[Risk: A public command has no internal callers but is used manually]** -> Review documented workflows and accepted requirements; resolve material uncertainty before deleting that capability.
- **[Risk: File deletion hides data loss behind smaller passing tests]** -> Review dataset semantics and distinct test cases, migrate needed CSVs and verify mixed outcomes and report consistency.
- **[Risk: A bundled CSV is absent from native or browser distribution]** -> Check existing distribution and WASM/Worker boundaries for retained consumers, including operation outside the repository root.
- **[Risk: Independent expected results become coupled to production data]** -> Keep independently authored numeric and causal expectations; share input fixtures only when their purpose is shared.
- **[Risk: Local DPS describes an earlier engine or script]** -> This is permitted until an explicit refresh. Preserve its calculation identity and recalculate all delivered entries during release/publication preparation.
- **[Risk: An explicit refresh reuses stale calculation cache]** -> Keep real calculation identities in the cache key and make release refresh bypass previous calculation cache.

## Migration Plan

1. Review affected consumers and already deleted tests/tools; record justified dispositions in this design.
2. Restore or extract needed curated tables into maintained CSVs and migrate retained native, WASM and tooling consumers.
3. Remove inline datasets, duplicate inventories, empty placeholders, implicit fallbacks and obsolete contracts; derive consistent manifests.
4. Delete confirmed obsolete package infrastructure only after retained data, consumers and focused checks have migrated. Repair documentation and run checkpoint without forcing DPS refresh.
5. Recalculate and validate the complete delivered catalog during explicit release/publication preparation; ordinary development and archiving this cleanup do not constitute publication.
6. Archive only when the revised tasks and applicable checks are complete. Until then, leave implementation repairs visible as unfinished tasks.
