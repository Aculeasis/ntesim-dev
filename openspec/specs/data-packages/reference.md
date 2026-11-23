# Frozen data contract v1

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

This document describes the baseline logical Global package and remaining legacy reader/audit requirements during the [consumed typed content migration](../../../docs/STORED_DATA_POLICY.md). New findings follow the [rolling data policy](../product-scope/reference.md#rolling-data-policy); source collection is not a prerequisite for mechanics implementation. Later-package support remains separate from the baseline.

`data/global-2026-10-04/manifest.json` identifies the frozen logical Global cutoff, schema and table inventory. `sources.json` retains the original evidence IDs, historical paths, SHA-256, retrieval metadata, upstream commit and author's version claim. Client compatibility is an independent field and remains incomplete. [Storage policy](../../../docs/STORED_DATA_POLICY.md) owns the compact physical representation and optional original-byte cache.

`tables/` preserves remaining native rows with a source ID. Original `evidence/` bytes are cache-only. `storage.json` binds the exact committed file inventory/hashes to the unchanged original source identities. Ordinary Python/Rust audits verify storage, row counts, ownership and partial-model references without game/cache access. Comparing original exports is an explicit optional check; an integrity hash alone does not independently prove a number. Native units and asset coordinates are deliberately not converted to runtime seconds or multipliers here.

Enemy tables and other unconsumed tables were removed under [decision 0009](../../../docs/decisions/0009-anonymous-targets-and-typed-content.md). Remaining adapters are transitional: required runtime constants are moving to typed Rust structures, without unused source fields or JSON storage.

`coverage.json` enumerates supported character, weapon and set identities using kind-prefixed native IDs. The roster establishes 23 selected released characters. Release evidence, numeric values, activation conditions and executable completeness remain separate statuses; inventory membership does not certify mechanics coverage or strict readiness.

`presets/` freezes historical build selections and conditions. [ACCEPTANCE.md](../calibration/spec.md) marks these as optional legacy references; snapshots are partial historical inputs, not current product acceptance or a complete executable preset. Embedded source filenames in these snapshots are historical provenance, never runtime paths. They do not impose limits on universal builds. Snapshots also retain their original assumptions and candidate values; importing them does not validate those assumptions.

Migration accepts an explicit acquisition root. Ordinary audits and tests require only this product directory and Python's standard library:

```powershell
python tools/check.py -- python tools/data_package.py audit data/global-2026-10-04
python tools/check.py -- python -m unittest discover -s tests
```

To reproduce from the frozen legacy acquisition workspace:

```powershell
python tools/data_package.py import --legacy-root .. --out target/reproduction/global-2026-10-04
```

Run these commands from the product directory. Import requires a destination that does not exist, including an empty directory; use a new destination for every reproduction attempt. It validates acquisition inputs before exclusively creating the destination and never overwrites an existing package. A failed write may leave a partial destination; inspect it and choose a new path for retry. Audit the tracked package to verify it without regenerating its bytes. New content patches require a separately reviewed package identity and importer/auditor support; changing the output directory alone does not change the package version.

The generated `mechanics.json` native research backlog is cache-only and is not loaded, reconstructed or hashed by runtime readers. Package metadata consists of `manifest.json`, `sources.json` and `coverage.json`, plus the storage receipt when present. Audits reject unestablished runtime-readiness claims; removing a source inventory does not establish mechanics coverage.

`gaps [--owner ID]` returns `package-scope-gaps-v1`: audited manifest scope declarations, optional known canonical owner identity, metadata hashes and explicit incomplete readiness. Native source-mechanic filters and candidate counts are removed. Scenario execution reports expose reached assumptions and failures; the active [OpenSpec changes](../../README.md) owns development work. Optional acquisition tools may read a historical backlog from an explicitly supplied cache evidence root.

`research/fixtures.json` lists 13 legacy partial models and 134 referenced local export identities. Selected models replace asset references with package source IDs; audits verify their stored bytes, original identities and false readiness. Original fragments and decoder snapshots remain in the optional cache. Original-to-normalized reconstruction uses `python tools/package_storage.py audit-originals data/global-2026-10-04 --evidence-root target/repository-audit/global-original`. Missing historical overrides remain explicit gaps. Ordinary audits need neither those originals nor game archives/mapping binaries. This does not establish correctness of every decoder schema or native runtime compatibility.

Generation is byte-deterministic and checks table and partial-model source hashes before writing. A package can be moved independently of the acquisition workspace. The current migration intentionally excludes bulk game assets. Complete release evidence, executable mechanic contracts, generic actions, all-level build legality and native compatibility remain queued; raw table coverage is not playable content coverage.

## Supplemental implementation parameters

The separate [combat-data contract](../runtime-content/reference.md) owns `data/combat/active.json`, its compact versioned parameter package and optional source-reconstruction reader. It adds selected action, effect, curve and gear inputs; raw timeline/function/export payloads stay in the acquisition cache. It does so without rewriting this frozen package. Existing native stat/damage/refinement projectors remain the numeric consumers for their owned tables; the supplement supplies additional source-bound implementation inputs and preserves native policy gaps.
