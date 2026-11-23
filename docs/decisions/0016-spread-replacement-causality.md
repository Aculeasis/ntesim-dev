# 0016: Ownership replacement remains distinct from damage parentage

- Status: accepted
- Decision date: 2026-11-01
- Supersedes: [0011](0016-spread-replacement-causality.md)
- Current owners: [Zankou Scorch](../mechanics/characters/zankou/action-damage.md), [WASM/report parity](../../openspec/specs/wasm-transport/reference.md)

## Context and decision

The native Zankou model spreads an existing Scorch snapshot to other living
dummies. It preserves offensive ownership and the hit that caused ownership
replacement. A later Scorch packet on a new recipient therefore legitimately
references a replacement hit on the original recipient. ADR 0011's requirement
that every delivered packet share the triggering hit's target was too narrow.

Keep `esper_cycle_replacement` with its original switch participants, replacement
owner and earlier `trigger_hit_index`. Validate receipt order and time, and retain
the existing participant/owner checks. Recipient selection and status spreading
remain native owner mechanics. The replacement cause is not a direct damage
parent, so delivered recipient equality is not part of that transport relation.

The [0015 damage parent](0015-damage-parent-identity.md) relation remains separate:
Hexed inherits an actual damage packet's owner, recipient and command origin.
Its strict same-target check still applies. No new cast, duplicate target field,
or inferred status-spread event is required to describe Scorch's existing cause.

## Rationale and limits

Retargeting the stored trigger would invent an ownership replacement on the
spread recipient. Dropping the trigger would erase the actual cause. Preserve
the native snapshot and constrain consumers to structural identity checks,
without reconstructing spread eligibility or damage in TypeScript.

This retains ADR 0011's switch/trigger separation, third-party owner support,
backward references and original receipt-vector indexing. It changes only the
overly strict target equality for ownership replacement. Model eligibility and
game fidelity still require independent native mechanics evidence.

## Verification

Native two-dummy continuing Scorch tests retain the original replacement cause
across recipient spread. Native/WASM/actual Worker aggregation and replay compare
complete reports and continue rejecting self/forward triggers, missing origins
and unknown replacement owners. Direct damage-parent corruption remains rejected.
