# 0007: Halt on failed ordinary action admission

- Status: accepted
- Recorded: 2026-10-17
- Decision source: explicit human review instruction on 2026-10-17
- Current owners: [execution](../../openspec/specs/script-execution/reference.md), [syntax](../NTE_DSL.md), [runner](../../openspec/specs/combat-execution/reference.md)

## Context and decision

The previous pending execution contract proposed automatically waiting for an unconditional input to become ready. The current runner already stops at a rejected action. The human review explicitly confirmed that a broken rotation must stop immediately and explain the failure.

Keep immediate failure for ordinary actions rejected by cooldown, energy, recovery, field ownership or owner-state admission. Preserve the timestamp, source location and typed reason. The executor must not silently insert time, retry, skip the action or grant resources. Failed trials remain excluded from successful aggregates.

Authored `wait`, `wait_until`, `wait_for_energy` and future explicit resource/state waits remain event-driven. A future authored readiness condition may skip its body. This changes the previously planned automatic-wait policy, not the shared Rust interpreter direction in [0006](0006-sequential-rotation-language.md).

## Rationale and consequences

The rotation expresses the user's intended inputs and timing. Inserting a wait changes that experiment and can conceal invalid resource or timing assumptions. Immediate, source-bound failures make those mistakes reviewable. Authors supply waits or conditions when they want them; explicit waits continue to settle intervening callbacks and preserve the DPS denominator.

## Reconsider when

A human explicitly requests a separate, opt-in scheduling policy with distinct input identity and output provenance. Do not reinterpret ordinary action admission through a convenience fallback.

## Verification

Native/WASM/Worker checks retain immediate typed cooldown/resource/state failures and source locations. Explicit energy gates use projected owner capacity, resume on settled events, and preserve aggregation/replay parity. A successful explicit wait does not authorize automatic retries for subsequent actions.

### Owner-selected utility inputs (2026-11-01)

A live owner state can replace an ordinary authored ability input with a
non-damaging utility, as Nanally's Authority grants RemoveSkill. Content supplies
a typed replacement gate and recovery. Read-only availability and actual native
admission use the same selected gate, preserving ordinary field/life/recovery
checks and immediate typed rejection. A utility can preserve the original
ordinary cooldown without spending its energy cost or running ordinary cast
observers. Its admission keeps the authored input and nominal packet template
with explicit accepted Other/utility provenance; the template is never delivered.
This is an actual owner transition, not an automatic wait, retry or fake cast.
Source replay, native CLI aggregation and bounded WASM/Worker retain this result.
