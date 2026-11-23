# Evidence inventory and acquisition leads

New research follows the [rolling data policy](../openspec/specs/product-scope/reference.md#rolling-data-policy): use current checkable evidence, retain source links/file references and material conflicts, and do not require hashes or exact Global/patch certification. This inventory does not refresh external pages; entries below describe saved findings, not a guarantee that a source is current.

Existing runtime inputs follow the rolling data policy. [Storage policy](STORED_DATA_POLICY.md) separates consumed typed constants and retained semantic inputs from original exports.

Current mechanical findings and unanswered questions are owned by the [dossier](mechanics/README.md); this document owns source discovery, acquisition and provenance. [Research-only prompts](mechanics/README.md) define investigation handoffs.

## Scope and leads
- The pinned Waifus-Grace/NTE_Assets extraction supplies native tables; its author Global 1.4.5 claim does not settle installed-client/hotfix compatibility or candidate release.
- Static local graphs, struct and enum findings supplement tables, with leads and applicability gaps documented in mechanics cards.
- The [compact combat tables](../openspec/specs/runtime-content/reference.md) retain selected implementation parameters and source hashes. Normal table reading and tests require only the runtime package.
- [Drive Calculator audit](DRIVE_CALCULATOR_RESEARCH.md) retains pinned acquisition/measurement limits; mechanical findings and conflicts live in the linked dossier owners. External source is read as evidence, never executed.
- [Character dossiers](mechanics/characters/README.md) own decoded owner findings and modeled rules; their matching unknown cards retain scope/form/dispatch questions. [Checkpoint targets](../openspec/README.md) provide saved scenario references.
- [Equipment contract](mechanics/shared/equipment-builds.md) owns portable static compilation; published versus native +20 values, allocation-pool applicability, passive execution and release remain separate questions.
- [gcsim comparison](GCSIM_REFERENCE.md) is architecture/workflow evidence only, not a source of NTE mechanics or permission to execute/copy third-party code.

## Before binding evidence

Inspect the relevant source rows/graphs or external observations. Bind units, activation, recipients, resources, timing/clocks, refresh/ICD, snapshots/crit/RNG and off-field behavior to a practical checked model. Serialized presence alone does not establish execution. Damage spreadsheets, independent calculations and consistent combat logic can validate models without extracting exact client dispatch; uncertain rules remain explicit hypotheses. Hash audits are optional for research, not a prerequisite to recording a supported finding.

The owning card records the current question, evidence/conflicts, affected scenarios and stop/return condition. Resume stopped investigation only on new evidence/method or a justified revised approach. Reuse existing snapshots/reviews without regenerating them. New material needs a source link or local file/field reference; a separate receipt or immutable package is not required. Inaccessible evidence remains a named gap; do not discard useful observations solely because an exact patch cannot be certified.

Three preserved formula reviews still cite the retired `docs/COMBAT_FORMULA_CONTRACT.md`: [report check](../reviews/combat-formula-audit/report-check-2026-10-12.md), [formula review](../reviews/combat-formula-audit/review.md) and [Rust audit](../reviews/combat-formula-audit/rust-contract-audit-2026-10-12.md); their historical bytes remain unchanged. Current formula ownership is [damage and stat zones](mechanics/shared/damage-and-stat-zones.md).

Acquisition tools that read or generate the historical workspace require `--legacy-root PATH`; their shared root resolver rejects a frozen-package destination. Imported acquisition helpers retain the same explicit root and leave each tool's remaining arguments intact. The PowerShell asset reader uses `-LegacyRoot PATH`. These tools may require separately available historical inputs or reviewed parsing libraries; ordinary product checks do not.

Public-page acquisition stages downloads into temporary workspace directories, stores completed bytes at a content-hash identity under `data/sources/evidence`, and retains prior receipts in `snapshot_history.json`. The URL manifest is the current lookup, not an immutable source identity. Source readers, cached retrieval and extraction check the declared hash and workspace containment. Pinned historical readers can resolve archived receipts; unavailable attempts have no completed evidence file. Proxy use requires the caller's explicit `--proxy`; there is no host-specific fallback route.
