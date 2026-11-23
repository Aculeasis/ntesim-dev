# 0012: Preserve admitted Arc identity for delayed weapon hits

- Status: accepted
- Recorded: 2026-10-24
- Decision date: 2026-10-24
- Current owners: [WASM/report contract](../../openspec/specs/wasm-transport/reference.md), [Black Tome](../mechanics/weapons/black-book/passive.md)

## Context and decision

An explicit Arc is a player input, while its later weapon hit can coexist with
ordinary ability admissions. Black Tome's Break listener previously assigned
command zero to every follow-up, losing the actual activation when Support
casts and renewed Arcs preceded it. An ordinary ability ledger cannot supply
the missing parent without inventing an ability cast.

Rust now retains each Black Tome activation's command in its owner state and
exports the resolved Arc ability in its admitted player-input receipt. Delayed
hits retain that activation, even when an ally supplies the later Break or the
wearer has left the field. Renewal replaces the parent for future emissions.
The same optional receipt identity is emitted by the shared Arc admission path.
Existing non-Arc inputs keep their representation.

## Rationale and consequences

The existing player-input ledger is the actual admission record. Reusing it
avoids fake ability occurrences, fixed command indices and browser inference
from equipped item names. TypeScript checks command, owner, resolved ability,
ordering and the owner-child relationship before aggregation/replay. Resource,
eligibility, designation and damage rules remain in Rust.

The field is an additive optional JSON/wire-v1 report field; older serialized
receipts remain decodable. Its absence cannot certify an Arc parent in mixed
ability reports. Receipt validation checks structure without recomputing
mechanics. Other owners retain their own
pending-effect attribution rules.

## Reconsider when

A concrete further input producer needs a different causal relationship.
Preserve the real admitted input and independent triggering-event ownership
rather than assigning a convenient unrelated command.

## Verification

The October27 owner-input clarification runs the committed input lifecycle hook
before Arc activation with its real command index. This closes Chaos's old
montage/held-continuation source and preserves independently committed weapon
effects, just as other admitted inputs do. A post-recovery full Verdict test
checks that Arc cannot leave a stale short-hold continuation available.

Rust Black Tome tests check all five refinements, automatic Support unlocks,
two continuing activations, live/off-field Break callbacks, deadline behavior,
designation and source defeat. The shared [script](../../examples/black-tome-cycles.nte)
is consumed by native and actual Worker comparisons with aggregation/replay.
Transport tests reject unrelated commands, missing/wrong resolved abilities and
future input timestamps. The checkpoint binds native and WASM artifacts.
