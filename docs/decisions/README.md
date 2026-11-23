# Architectural decision records

These records explain why significant choices were made, what they cost and when to reconsider them. [Requirements](../../openspec/specs/product-scope/reference.md) own product scope; [architecture](../ARCHITECTURE.md), [web architecture](../WEB_ARCHITECTURE.md) and linked contracts own current boundaries. Code and tests define executable capability; A decision record is neither a second specification nor evidence of completed implementation.

## Find the relevant decision

Read the applicable row before changing its boundary or technology. Follow only the owner links needed for the task; ordinary kit bindings do not require loading this directory.

| Record | Read when changing | Summary rationale |
| --- | --- | --- |
| [0001: Rust and one engine](0001-rust-product-engine.md) | Engine language, duplicated simulation, native/WASM composition | Single codebase ensures consistent combat semantics between CLI and web without maintaining duplicate logic |
| [0002: React and static browser](0002-react-static-browser.md) | UI framework, browser/server responsibilities, hosting | Static browser execution with Web Workers avoids backend server complexity and operational overhead |
| [0003: Shared services and content owners](0003-shared-services-and-content-owners.md) | Crate dependencies, per-kit state, shared abstractions | Decouples individual kit semantics into content modules while sharing common math and runtime services in core |
| [0004: Event-driven simulation](0004-event-driven-simulation.md) | Scheduling, readiness, polling, clock and RNG behavior | Event/deadline advancement avoids tick-polling overhead and guarantees exact deterministic causality |
| [0005: DPS modeling boundary](0005-dps-modeling-boundary.md) | Fidelity, scenario scope, omissions and unsupported dependencies | Models direct and indirect causal paths to team DPS while bounding costly unrelated game systems |
| [0006: Sequential rotation language](0006-sequential-rotation-language.md) | Public script semantics, interpreter ownership, compiler migration | Human-readable DSL preserves author intent and enables per-trial dynamic live state inspection |
| [0007: Failed action admission](0007-failed-action-admission.md) | Admission failure, automatic retry/wait policy, explicit waits | Invalid ordinary inputs halt immediately; authors explicitly request waits or conditional skips |
| [0008: Architecture consolidation and hygiene](0008-architecture-consolidation-and-hygiene.md) | Domain duplication, hypothesis strings, crate boundaries, tooling sprawl, WASM boundary | Restores strict single domain ownership in Rust, purges TS domain/tests, extracts strings, consolidates tooling while preserving <= 4 CPU limit |
| [0009: Anonymous targets and typed content](0009-anonymous-targets-and-typed-content.md) | Target scope and runtime data | Explicit script-authored dummies; only consumed typed Rust constants, without enemy or JSON runtime catalogs |
| [0012: Admitted Arc input identity](0012-admitted-arc-input-identity.md) | Player-input weapon effects and delayed report/replay ownership | Link weapon packets to the actual Arc activation without inventing an ordinary ability cast |
| [0014: Autonomous owner event identity](0014-autonomous-owner-event-identity.md) | Autonomous activations, descendant report/replay origins and HP settlement observers | Preserve actual causes and captured recipients without inventing authored actions |
| [0015: Triggered damage parent identity](0015-damage-parent-identity.md) | Damage follow-up report/aggregation/replay causality | Retain the actual earlier damage packet and its command, owner, target and switch identity without inventing a cast |
| [0016: Spread replacement causality](0016-spread-replacement-causality.md) | Ownership replacement followed by status spreading to other recipients | Preserve the original replacement cause while native owners select later recipients; distinguish it from direct damage parentage |
| [0017: Sequential admission replay](0017-sequential-admission-replay.md) | Source exports, same-time native replay and owner generation identities | Retain settled source input boundaries without changing scheduled native admission or real timestamps |
| [0018: Authored input windows](0018-authored-input-windows.md) | Source action completion, early controls and cancellation-oriented authoring | Explicitly yield at native owner gates while keeping actual admission, costs and pending effects in the engine |
| [0019: Causal state observations](0019-causal-state-observations.md) | Native report timelines, field ownership and settlement observations | Sparse event-boundary state and exact settlement receipts without inferred combat inputs or resumable-state claims |
| [0020: Source replay checkpoints](0020-source-replay-checkpoints.md) | Native cancellation, checkpoint boundaries and continuation | Verified deterministic prefix reconstruction preserves the complete scheduler without serializing observation frames as combat state |
| [0021: Browser trial prefix reconstruction](0021-browser-trial-prefix-resume.md) | Browser Stop/resume and retained trial batches | Reproduce delivered seeds in Rust before continuing in a fresh Worker; retain executed-source identity |

Records explain retained architecture choices. OpenSpec capabilities own accepted behavior and change design.md files explain proposed changes. Keep durable ADR rationale linked rather than copying it into requirements.

## Use and revise

Distinguish three things while reasoning: the product requirement to preserve, the architectural mechanism selected to meet it, and a transitional implementation limit. For example, consistent native/browser combat semantics is a requirement, one Rust engine is a chosen mechanism, and browser compile-time rotation expansion is a migration limit. A current workaround is not automatically an architectural commitment.

For a proposed architectural change, state the relevant decision and original reason, the assumption that changed (or new evidence that outweighs it), alternatives and the new trade-off. A reconsideration condition starts analysis; it neither authorizes an unrelated scope change nor automatically requires asking the user. Follow the current human instructions and normal task authorization.

Use the [template](TEMPLATE.md) for choices that materially affect boundaries, dependencies, public interfaces, execution or product-wide quality. Do not create records for ordinary bindings, routine fixes, every commit or implementation progress. Keep each record about a coherent choice, normally one page; link detailed rules and checks to their owners instead of copying them.

Record status as `proposed`, `accepted`, `superseded` or `deprecated`. A proposal is not an active rule. When changing an accepted choice, create the next unused numbered record, link the old and new records, mark the old one superseded and update the current owners in the same change. Keep historical reasoning; do not silently rewrite it to make the replacement look inevitable. Link/factual corrections and newly confirmed recollections may be added as dated clarifications without replacing the original account. Keep the router current, with superseded entries clearly marked and pointing to replacements.

## Verification

Check that an architectural change preserves the required behavior using the relevant owner checks; a Markdown check verifies links, not architectural correctness. Keep verification expectations in the record and actual run results in their existing receipts. Do not add a second test or coverage registry.

The lightweight format adapts [Nygard's ADR proposal](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions), [Fowler's guidance on rationale and reconsideration](https://martinfowler.com/bliki/ArchitectureDecisionRecord.html) and [MADR](https://adr.github.io/adr-templates/). These references explain the format, not project-specific choices.
