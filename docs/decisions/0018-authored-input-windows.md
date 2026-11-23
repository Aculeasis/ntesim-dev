# 0018: Authored input windows over native owner gates

- Status: accepted
- Recorded: 2026-11-03
- Decision date: 2026-11-03
- Current owners: [syntax](../NTE_DSL.md), [execution](../../openspec/specs/script-execution/reference.md), [native acceptance](../../openspec/README.md)

## Context and decision

Advancing every source action through full recovery prevents ordinary inputs
from reaching implemented unfinished-montage branches and cancellation gates.
Returning at commitment for every action would change existing live queries
and require the executor to infer which following input the author intended.
That is especially ambiguous across branches and dynamically evaluated values.

Keep full recovery as the ordinary source completion point. An explicit
`window=attack|hold|jump|dash|skill` asks that producer to yield at its native input
gate instead. Content supplies the gate from current owner state; the source
executor copies no frame constants and performs no cancellation. A family
without an early override retains ordinary full recovery. The next authored
input rechecks all native conditions, including recovery, costs, cooldowns,
field ownership and branch permissions, under [ADR 0007](0007-failed-action-admission.md).

Advance only to the next queued event or the selected deadline, then recompute
after contacts, hitlag, interruption and other owner transitions. A query does
not consume RNG. Preserve the requested window on the source occurrence and
actual admitted timestamps in native replay; no extra physical input is invented.

## Rationale

The explicit producer parameter keeps decisions and parameter evaluation on
one real clock without syntactic lookahead, rewinding, speculative admission
or a hidden retry policy. It supports guarded scripts and leaves all gameplay
rules with existing Rust owners. An explicit window is additional authoring
surface, but avoids requiring authors to reproduce montage coordinates.

## Consequences and assumptions

A same-character swap stays a zero-time native no-op and does not wait out
an earlier producer. Occurrence completion means the source yielded at its selected point; pending
montage requests can remain. Exhaustion or observation alone does not cancel
them. Spawned trains, paid costs, resource state, native refunds and subsequent
admission stay engine-owned. Invalid reached states still diagnose. The initial
hold-window binding was Blackbird's implemented branch selector. Chaos now
uses its existing Normal/Appear/Plunge combo and full-Verdict continuation
predicates, including minimum overlap and post-recovery source lifetime.
Daffodil uses her existing Normal Jump gate for held-combo admission. Her
ordinary source projection selects the native held/suspended-Normal model,
so paid Front Dodge can preserve remaining contacts without an unrelated
held input elsewhere in the script. Default Dodge-window observation does
not predict a later Back choice; that command rechecks its own native gate. Other
input families use the ten implemented owner control profiles. Further owner choices
require explicit owner support. Native CLI, WASM and the live Studio Worker share the source execution policy.

## Reconsider when

Evidence requires an additional owner input family, or a distinct explicitly
requested execution policy. Preserve immediate failed admission and the same
native causal timeline when extending or replacing this surface.

## Verification

[Rust source-window checks](../../crates/content/src/simulation/tests/source_windows.rs)
cover ordinary/evolved continuing branches, caused hitlag, native Jump controls,
committed extras and spawned trains, paid Dodge/Riposte, unchanged observation
results, immediate rejected taps, source artifact/native replay and typed source
ranges. [Chaos source checks](../../crates/content/src/simulation/tests/source_chaos_controls.rs)
add continuing funded Verdicts, post-recovery yields, captured-form predicates
across natural expiry and immediate longer/idle-hold failures.
[Daffodil source checks](../../crates/content/src/simulation/tests/source_daffodil_controls.rs)
retain continuing held replacement, paid suspension, Jump cancellation and
notify-started Skill cooldown without changing packet lineage. Existing owner
tests retain independent frame, cost and packet oracles.

2026-11-06 clarification: Hotori exposes `window=attack` at the existing native queued-next-Normal gate. Full source recovery respects her later fifth-stage Normal gate without shortening Any recovery for other inputs. Explicit early yielding permits both Katana chains and a finisher; slower ordinary inputs can instead reach timeout. Continuing source checks preserve both outcomes, prohibited switches, real waits and native replay.
