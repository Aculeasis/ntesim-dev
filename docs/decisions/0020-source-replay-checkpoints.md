# 0020: Source input boundary replay checkpoints

- Status: accepted
- Recorded: 2026-11-06
- Current owner: [native lifecycle](../NATIVE_LIFECYCLE.md)

## Context and decision

The source interpreter already yields after authored inputs while the native
scheduler retains pending contacts, owner clocks and RNG. The report observations
in [0019](0019-causal-state-observations.md) omit that state. Serializing every
owner's internal state would introduce a large second persistence contract.

Use deterministic replay checkpoints at complete source input boundaries,
including initialization and the boundary before source exhaustion drains pending
events. Bind the exact compiled source, engine/content fingerprint, source and
execution limits and seed. Retain the completed prefix as an incomplete report;
store its boundary index and verification digest in the small checkpoint token.
Resume reconstructs the entire trial from its original input, verifies the reached
prefix, and continues that same interpreter and scheduler without resetting at
the boundary. Replay restores lexical scopes and work counters as well as combat.

## Consequences and limits

This supports portable native continuation without describing report frames as
resumable scheduler state. It costs CPU proportional to the replayed prefix;
repeated small chunks can cost quadratic total work. The checkpoint token needs
the unchanged source and executable. It is an integrity/reproducibility receipt,
not an authentication mechanism. Changed identities and inconsistent prefixes
reject before new inputs execute.

Cancellation is cooperative between complete inputs. An in-progress action,
explicit wait, lexical evaluation or final event drain runs to its bounded end.
The callback is suppressed while reconstructing the saved prefix. Event/work
limits and admission failures remain terminal results, never resumable tokens.
Browser Worker prefix reconstruction has its own [decision](0021-browser-trial-prefix-resume.md); an observation frame still is not a serializable scheduler. Reconsider full state serialization if measured replay cost or
cancellation latency prevents a concrete workflow.

## Verification

Rust lifecycle checks persist/reload repeated chunks and compare complete source
receipts with uninterrupted continuing RNG, Hotori Time Stop, lexical variables
and incoming/shield scenarios. CLI integration checks cancellation-file signalling,
retained prefix, relocated checkpoint and resumed equality. Identity/prefix and
unreachable-boundary rejection remain explicit failures.
