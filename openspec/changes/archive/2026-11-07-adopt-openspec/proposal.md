## Why

The repository has competing queues, readiness estimates, completion logs and partially adopted behavior specs. Historical delivery phases and duplicate instructions obscure current ownership and make saved work harder to resume.

## What Changes

- Adopt the standard OpenSpec spec-driven workflow and generated Codex skills.
- Relocate existing behavioral contracts to capabilities with concise requirements/scenarios and detailed companion references; keep language manuals and relevant mechanics evidence.
- Delete obsolete queues, phase completion logs, duplicate instructions, progress counters and their tests. Remove research completion boilerplate from cards and generators.
- Pin CLI setup and extend checkpoint's existing document/tooling boundary to validate specs, metadata and archived tasks.
- Replace phase-based work selection and GUI freezes with selected changes and affected-boundary verification.

## Capabilities

### New Capabilities
- `development-workflow`: one resumable change lifecycle, accepted requirement ownership and verified completion.

### Modified Capabilities
None. Existing simulator contracts are relocated and normalized as the initial spec baseline; combat behavior and public runtime APIs are unchanged. Requirements describe accepted intent rather than certifying every implementation branch.

## Impact

AGENTS.md, documentation ownership and links, tools/openspec.py, tools/checkpoint.py, obsolete progress tooling, mechanics card generators and shared-reference inspection. The CLI runtime is cached locally; generated Codex skills are tracked. Two emitted Rust calibration-reference strings follow the relocated documentation. Combat arithmetic, public schemas and Studio behavior are unchanged; no release is published.
