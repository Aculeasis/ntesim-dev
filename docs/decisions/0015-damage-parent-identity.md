# 0015: Preserve the actual parent of triggered damage

- Status: accepted
- Decision date: 2026-11-01
- Current owners: [reaction model](../mechanics/shared/cycles-and-break.md), [WASM/report parity](../../openspec/specs/wasm-transport/reference.md)
- Related: [0011](0016-spread-replacement-causality.md), [0014](0014-autonomous-owner-event-identity.md)

## Context and decision

Hexed creates damage from an actual positive HP-loss packet. Its parent can be
an ordinary action, Support, a Cycle packet or an autonomous effect. A command
index alone cannot identify that earlier hit or explain why the follow-up has
no independently admitted ability. The existing report lost Support switch
provenance and browser replay rejected a valid native result.

Lacrimosa's An Hun Zhou II also follows a concrete Nightmare/shatter hit while
Broken or under the selected Discord status. It retains that hit's identity.
Its independent Discord-application path instead registers an autonomous event
under ADR 0014 with the actual application command/event as its cause; command
zero cannot stand in for this cause. Native passive metadata identifies the
passive packet directly, without claiming a separately admitted player cast.

Skia's Fang Thrust consumption on an actual Normal hit uses this same parent
link for its coordinated Shadow Attack. The independent timed Hound path keeps
the Skill command as its origin. A triggering Normal command alone cannot be
misrepresented as a separately admitted Skill cast.

Emit `parent_hit_index` on the follow-up, referencing the original report's
earlier receipt vector entry. Retain the parent's command/autonomous identity,
actor, target and switch origin. The Hexed status applier does not replace the
damage owner. Standalone hits have no damage parent; no synthetic cast is added.

Rust checks backward ordering, time and matching identities before exporting a
report. Browser consumers check the same structural transport relationship and
resolve admission through the earlier parent, without recomputing eligibility
or damage. Existing native metadata/provenance checks remain in force.

## Alternatives and limits

Relabeling the follow-up as the parent's ability would obscure its independent
reaction category. Accepting a matching command without a hit link would allow
unrelated packets and lose equal-time causal identity. A general event graph is
unnecessary for this concrete one-parent relationship.

The reference uses native receipt order before display sorting. It currently
models a same-owner, same-target damage follow-up, not ownership replacement or
recipient redistribution. Multiple damage parents or cross-report merging would
require reconsidering this boundary.

## Verification

Native Hexed tests retain independent CI0/CI108 damage checks, ordinary and
Support parents, and rejection of self links, wrong owners and lost switch
identity. Current native/WASM/actual Worker report, aggregation and replay
checks retain the link and reject a forged forward/self parent.
Lacrimosa tests cover continuing two-target Nightmare parents, the unchanged
4 ATK coefficient and both actual Discord application command identities.
Skia's existing mark-consumption check preserves the 1,350,100us packet and
checks its same-time Normal parent; live source equipment parity transports
that relationship through the actual Worker, aggregate validation and replay.
