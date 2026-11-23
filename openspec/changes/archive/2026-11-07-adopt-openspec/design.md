## Context

See proposal.md for the problem and scope. OpenSpec 1.16.1 supplies the standard schema and six core Codex skills. Existing contracts mix behavioral requirements, precise public formats, implementation detail and obsolete campaign status.

## Decisions

- Keep one repository-local openspec/ root using standard spec-driven. No external store or custom schema is required.
- Put concise accepted requirements and concrete scenarios in spec.md. Retain detailed current interface formats and limits in each capability's reference.md, linking mechanics evidence instead of flattening acquisition material into specs. This preserves exact public contract detail while keeping normal context reads bounded. Companion references are maintained with their capability; OpenSpec merges spec.md deltas, so reference edits land explicitly in the same code change.
- Relocate existing contracts directly as the initial baseline. Only the new workflow capability is a behavioral delta in this change. Do not fabricate archived changes for historical work or turn migrated requirements into implementation-certification claims.
- Delete obsolete workflow documents, superseded decisions, counters and counter tests. Remove counter emission from card generators and make shared-reference inspection discover actual reference files.
- Pin CLI 1.16.1 in tools/openspec.py, installed with npm lifecycle scripts disabled into cache/. Keep generated skills intact. Their plain openspec command uses the documented installed CLI; the wrapper is the reproducible checkpoint and fresh-checkout entry point.
- Extend checkpoint's existing document/tooling selection. OpenSpec metadata and skills trigger strict spec validation and archived-task checks without selecting Rust/WASM/browser work. Keep existing source/report/Worker boundaries intact.

## Trade-offs

Migrated requirements are not a gameplay audit. Current source/tests and explicit model limits remain the evidence for executable behavior. Detailed references need deliberate updates beside deltas; they are not automatically merged by the CLI. CLI upgrades are explicit changes to the pin and regenerated skills, avoiding silent upstream instruction drift.

## Verification

Strict OpenSpec validation, archived-task validation, all local Markdown links, focused checkpoint and CLI-wrapper tests, global/pinned CLI invocation and checkpoint on the final tree. Two emitted calibration-reference strings move with the documentation, so checkpoint also checks the affected native research/CLI consumers and runner. There is no WASM schema or browser implementation change.
