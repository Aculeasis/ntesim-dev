# 0019: Native causal report observations

- Status: accepted
- Recorded: 2026-11-06
- Current owner: [runner contract](../../openspec/specs/combat-execution/reference.md)

## Context and decision

Final pools and textual traces cannot supply typed resource, buff, target and
shield timelines. Reconstructing these rules in consumers would violate the
single Rust engine boundary. Existing [autonomous origins](0014-autonomous-owner-event-identity.md)
and [damage parents](0015-damage-parent-identity.md) remain the causal identities
of packets; a new observation must not become another combat input.

Capture complete initial/final state and sparse changed actor/target/buff/stamina
observations after native event settlement. Preserve equal-time event ordinals
and exclusive hit/healing/trace receipt-prefix ends. This joins typed energy
receipts, autonomous origins and packet parents without parsing presentation
text. Capture the active actor in each hit at contact, independently of its
damage owner or original cast. All healing settlements use the shared exact
service and retain source/recipient, base, bonuses and gross/applied/excess HP.

## Consequences and limits

Observation never schedules work, consumes RNG, changes admission or polls a
deadline. Buff/status/shield records retain their real expiry fields; removal
is observed at the next settled boundary, potentially after an idle gap. A frame
is the state after one event, not a claim about each internal callback's net
state. Explicit healing, incoming damage and paid HP receipts retain gross
settlements even when net HP hides them. Reports reconcile exact damage,
per-packet rounding, applied target HP loss and overkill separately. Partial
receipts remain inspectable; completion/quality gates alone permit DPS.

Snapshots cost CPU and retained bytes. Capture only changed slots between the
complete boundary frames; native performance work measures that cost. These
observations omit pending scheduler/owner/RNG state and are **not resumable
combat checkpoints**. The alternative of parsing traces is rejected because
text cannot preserve exact typed identity and resource arithmetic.

### Native export clarification (2026-11-06)

Source exports retain these typed observations together with the executed source,
limits, identities, exact contribution views and compiled build decomposition.
Decomposition records native attribute contributions before the existing stat
settlement, rather than reconstructing builds in a consumer. CSV preserves exact
integer terms and nested typed JSON cells. Report replay compares both the full
source rerun and the native request replay; matching a failed prefix never changes
its execution status or comparison eligibility.

Studio consumes these same sparse frames and receipt prefixes. Trace rows and
hit receipts remain separate observations; consumers never parse diagnostic
prose to pair them or claim intermediate callback state. Exact browser exports
rerun the bound source/seed in Rust, verify source/native equality, and return
native-authored JSON/CSV. Decoded dynamic JSON integers may be presentation
strings and must not be round-tripped back as native report inputs.

## Verification and reconsideration

Native source/replay equality covers complete and failed prefixes; report checks
reconcile boundary states, receipt ranges, healing conservation and contribution
totals. Reconsider capture granularity if measured retention exceeds the native
performance budgets or a concrete consumer requires an internal transition
that existing settlement receipts do not preserve.
