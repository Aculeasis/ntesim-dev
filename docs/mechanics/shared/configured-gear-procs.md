# Configured gear proc interface

Responsibility: admission of explicit research proc parameters, independently of native item legality or passive semantics.

Structured research requests accept exact supported `GearProcConfig` variants. Duplicate kinds on an actor reject so counters/cooldowns cannot alias. Native builds/Studio obtain source-bound parameters through item owners; arbitrary research parameters do not establish native legality or full passive coverage. Unknown fields/kinds and unrepresentable resources/deadlines reject.

For configured timed stacks, duration is positive, ratio denominators nonzero and cap is in `1..=64`; this is a technical safeguard, not a native game cap. The shared [buff service](buffs-and-callbacks.md) and [energy accounting](energy-accounting.md) own their exact state transitions. Item eligibility/recipients/order, including the distinction between research solo and ordinary team paths, belong to [Fireflies](../sets/fireflies/passive.md) and other [gear owners](../weapons/README.md).

`tools/verify_gear_procs.py`, scoped content tests and native/batch fixtures check independent Fraction/expiry/HP accounting, boundaries, invalid requests and replay. Saved receipts prove their declared inputs only; unanswered native policies remain in [unknown gear](../weapons/README.md).
