# Documentation map

Start with the selected [OpenSpec change](../openspec/changes/) and [capability index](../openspec/README.md). [AGENTS.md](../AGENTS.md) owns concise contribution rules; [development](DEVELOPMENT.md#openspec-workflow) explains the working loop. Accepted requirements live in OpenSpec, not in a parallel task queue or progress ledger.

| Need | Reference |
| --- | --- |
| Build, run and check | [Runbook](RUNBOOK.md), [development](DEVELOPMENT.md) |
| Product and model boundary | [Product scope](../openspec/specs/product-scope/spec.md) and its detailed reference |
| Architecture | [Rust domains](ARCHITECTURE.md), [web architecture](WEB_ARCHITECTURE.md), [decisions](decisions/README.md) |
| Character, weapon or set evidence | One relevant [mechanics owner](mechanics/README.md), then necessary shared dependencies |
| Script authoring | [Syntax reference](NTE_DSL.md), [execution](../openspec/specs/script-execution/spec.md), [inspection](../openspec/specs/script-inspection/spec.md) |
| Browser and exact transport | [Studio](../openspec/specs/studio-application/spec.md), [WASM transport](../openspec/specs/wasm-transport/spec.md) |
| Calibration and release | [Calibration](../openspec/specs/calibration/spec.md), [distribution](../openspec/specs/distribution/spec.md) |

Specs state intended accepted behavior. Code and current tests establish executable capability; source facts, hypotheses and applicability limits remain explicit in mechanics references. Read only the relevant sections. Generated receipts and acquisition payloads belong in `cache/`, not a second documentation history.
