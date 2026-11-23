# 0021: Browser trial prefix reconstruction

- Status: accepted
- Recorded: 2026-11-06
- Current owner: [browser lifecycle](../BROWSER_LIFECYCLE.md)

## Context and decision

Stop destroys the Worker so a long native call cannot delay cancellation. A new
Worker cannot reuse its Rust collector, and retained report observations are not
serialized scheduler state. The existing native input-boundary checkpoints in
[0020](0020-source-replay-checkpoints.md) solve continuation within one seed;
Studio needs continuation of an independent-seed trial batch.

Reconstruct the delivered completed prefix through the shared Rust source
executor in a fresh Worker. Validate the compiled engine/content/source identity
and compare all retained sample fields. Rebuild native aggregates from actual
reports, then execute the remaining contiguous seeds. Keep the executed source
independent of the editor draft. Do not trust caller-provided aggregate values.

## Consequences and limits

This keeps aggregation, eligibility and reconstruction in Rust and preserves
immediate Worker termination. It costs replay time and grows memory with retained
observations. It deliberately discards undelivered work and does not resume an
interrupted seed. Full collector serialization would avoid replay but add another
durable native state contract. Reconsider it if measured reconstruction costs
prevent useful browser workflows.

## Verification

Rust adapter checks compare uninterrupted and resumed native statistics across
different seed chunks, reject corrupted prefixes and identities, and ensure a
failed replacement leaves no session. Worker/controller checks cover transport,
executed-source ownership, stale events and recovery after termination.
