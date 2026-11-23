# 0006: Authored sequential rotations with shared live semantics

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision date: unknown
- Current owners: [syntax](../NTE_DSL.md), [execution](../../openspec/specs/script-execution/reference.md), [requirements](../../openspec/specs/product-scope/reference.md)

## Context and decision

Users author readable builds and sequential combat inputs with the bounded control flow and queries defined by the public DSL. Native content owns action costs, timing, packets and state transitions. The accepted direction is shared Rust parsing and per-trial live interpretation for CLI and WASM; detailed syntax and excluded constructs remain in their owners.

Readable reproducible rotations with legal causal execution are requirements. The custom DSL and shared interpreter are selected mechanisms. Existing browser compile-time expansion and bounded live slices are transitional capabilities, not permanent semantic restrictions or evidence that the full interpreter exists.

## Rationale

A human-readable text-based domain specific language (DSL) allows rotations and builds to be authored, shared, and version-controlled cleanly without exposing internal engine implementation details. Live per-trial interpretation in the shared engine allows rotations to observe current resources and event outcomes dynamically rather than being frozen to a precomputed static schedule. Content owners define the semantics and timing of actions, keeping simulator logic out of user scripts.

## Consequences and assumptions

2026-10-17 clarification: [0007](0007-failed-action-admission.md) records the human-confirmed immediate-failure admission policy. Explicit authored waits remain supported; the former pending automatic-wait requirement is withdrawn.

The project maintains a parser, diagnostics, source-preserving editing and nonprogress budgets. Bounded syntax intentionally trades general programming power for an auditable combat interface. This assumes ordinary rotations fit the supported language and live queries can expose needed state without future RNG access or arbitrary engine mutation. Existing pending implementation must remain visible until checked integration.

## Reconsider when

Concrete ordinary rotations cannot express needed behavior within the accepted language, or measured maintenance costs justify a different frontend. Evaluate whether a small owner-defined input/query solves the need before generalizing the language. Preserve authored source, legality and shared per-trial semantics if syntax or implementation changes.

## Verification

The native source executor suspends its exact lexical interpreter at authored
inputs and waits, then advances the existing combat queue to owner deadlines.
Potential input assumptions are bound before combat initialization; combat rules
and deadline changes remain with content owners. `run-script`/`run_script` retain
the source and dynamic occurrences. Native compilation saves executable source
and verifies engine/configuration identities on load. The scheduled Studio export remains
available only to explicit structured/scheduled API consumers. Studio Run and
sample selection use the shared live source executor and Rust-owned trial
observations; no scheduled compilation or first-seed action replay mediates
their control flow. Report and state-replay views consume the executed source and native observations.

Use the execution owner's source-preservation, live-branch/readiness, nonprogress and actual Worker/WASM checks. Seed-dependent outcomes and equal-time observations must agree across consumers. A syntax test or compile-time fixture alone cannot establish live interpretation or complete kit execution.

2026-10-25 implementation clarification: fixed named `incoming` declarations
compile in Rust into the existing engine event list, independently of rotation
admission. Stable event ordering and the compiled `incomingDamageLocations`
array preserve each report/replay event index's authored source location.
This binds the existing scenario-input contract without adding compiler-owned
mitigation or owner mechanics. Periodic sampled hurt remains a separate gap.
