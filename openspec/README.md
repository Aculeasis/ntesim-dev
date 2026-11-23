# OpenSpec capability index

Accepted requirements live in `specs/<capability>/spec.md`. Detailed interface references stay with their capability; mechanics evidence remains under [docs/mechanics](../docs/mechanics/README.md). Requirements are not a declaration that every scenario currently passes.

Active work lives in `changes/<name>/`: proposal, design, specification deltas and verifiable tasks. Completed changes go to `changes/archive/`; obsolete pre-migration queues and completion logs are deleted rather than imported.

## Working with changes

```powershell
python tools/openspec.py setup
python tools/openspec.py list
python tools/openspec.py view
python tools/openspec.py status --change CHANGE
python tools/openspec.py validate --all --strict --no-interactive --concurrency 4
```

`view` is the terminal dashboard of changes and tasks. It does not calculate product readiness or certify simulation accuracy. [Development workflow](../docs/DEVELOPMENT.md#openspec-workflow) owns selection, generated Codex skills, checks and archive.

## Capabilities

| Capability | Requirement owner |
| --- | --- |
| Product scope and modeling | [product-scope](specs/product-scope/spec.md) |
| Event and resource kernel | [simulation-kernel](specs/simulation-kernel/spec.md) |
| Execution safety | [execution-safety](specs/execution-safety/spec.md) |
| Combat execution and reports | [combat-execution](specs/combat-execution/spec.md) |
| Live script execution | [script-execution](specs/script-execution/spec.md) |
| Script inspection and diagnostics | [script-inspection](specs/script-inspection/spec.md) |
| Synthetic seed batches | [synthetic-batches](specs/synthetic-batches/spec.md) |
| Paired synthetic comparisons | [synthetic-comparison](specs/synthetic-comparison/spec.md) |
| Finite cycle evaluation | [cycle-evaluation](specs/cycle-evaluation/spec.md) |
| Causal state observations | [state-observations](specs/state-observations/spec.md) |
| Synthetic rotation timelines | [synthetic-rotations](specs/synthetic-rotations/spec.md) |
| Source-bound research batches | [source-bound-batches](specs/source-bound-batches/spec.md) |
| Budgeted research experiments | [research-batches](specs/research-batches/spec.md) |
| Studio application | [studio-application](specs/studio-application/spec.md) |
| Native WASM and Worker transport | [wasm-transport](specs/wasm-transport/spec.md) |
| Data package interfaces | [data-packages](specs/data-packages/spec.md) |
| Runtime content and authored builds | [runtime-content](specs/runtime-content/spec.md) |
| Distribution requirements | [distribution](specs/distribution/spec.md) |
| Independent calibration and arithmetic checks | [calibration](specs/calibration/spec.md) |
| Lacrimosa behavior | [characters/lacrimosa](specs/characters/lacrimosa/spec.md) |
| Linko behavior | [characters/linko](specs/characters/linko/spec.md) |
| Nanally behavior | [characters/nanally](specs/characters/nanally/spec.md) |
| Shinku behavior | [characters/shinku](specs/characters/shinku/spec.md) |
| Skia behavior | [characters/skia](specs/characters/skia/spec.md) |
| Zankou behavior | [characters/zankou](specs/characters/zankou/spec.md) |
| Bounded synthetic search | [synthetic-search](specs/synthetic-search/spec.md) |
| Repository workflow | [development-workflow](specs/development-workflow/spec.md) |

This inventory migrates existing contracts and character specifications. Other content owners remain in their existing mechanics references and Rust tests; introduce or refine their capability specs when a real change touches them. Do not fabricate completed change archives or a new roster coverage registry.

Generated workflow instructions retain [OpenSpec attribution](THIRD_PARTY_NOTICES.md).
