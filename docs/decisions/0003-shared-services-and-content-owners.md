# 0003: Shared services with explicit content owners

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision date: unknown
- Current owner: [architecture](../ARCHITECTURE.md#content-ownership)

## Context and decision

Released characters and equipment need distinct state transitions while sharing clocks, targets, resources and damage services. Use the existing contracts/core/data/content/research boundaries. Keep kit and equipment semantics in their named content owners, with core independent of content and research. Native CLI and WASM compose these layers.

The requirement is clear ownership and consistent shared behavior. Today's crate/module layout is a mechanism, not a requirement to preserve every file or create matching folders for all owners.

## Rationale

Reviewed owner-child packet projection now preserves both identities through
the shared coefficient service. Content owns the exact initiating/packet GA
relationship and its strict-policy assumption; data validates the configured
parent and exact native child row without importing content rules. The caller
supplies the already captured talent coordinate. Nanally's real Underboss
Normal/Skill/Support consumers use this path rather than substituting a Burst
as the initiating action for coefficient lookup. Source metadata and action
parentage stay separate, and an unknown relationship still rejects.

Kit-specific fields or branches in core couple shared changes to individual characters. Duplicating clocks, build projection or callback logic in each consumer makes fixes and causal ordering harder to keep consistent. Named content owners keep a mechanic's transitions, legality and projection discoverable, while core common services provide one shared implementation of deterministic timing, resources, and combat arithmetic.

## Consequences and assumptions

Explicit boundaries impose routing and context plumbing costs. A shared abstraction needs real consumers and a common responsibility; symmetry, a long file or one new kit is insufficient justification. Small owners may remain one file. The structure assumes owner differences can use shared services without losing their causal semantics; convenient reuse must not erase those differences.

## Source form selection (2026-11-02)

Authored aim is a command-indexed native request input, preserved through live
execution, automatic Support, saved source and direct native replay. Admission
places the primary on that dummy; content retains packet expansion, caps,
retargeting and autonomous policies. Primary replacement/cancellation helpers
match the admitted recipient, and ungrouped energy/Cycle credit follows the
same aim instead of a hardcoded dummy zero. Source resolves only the typed
`tN` name and its diagnostic range; it defines no recipient or combat rule.

Switch admission passes the same aim to Cycle qualification, Discord preflight
and owner entry selection. Shared reaction generations/statuses retain that
recipient, while autonomous Blossom selection stays independent. The advisory
Support query explicitly inspects the default target context; actual admission
rechecks the authored switch context without temporarily mutating the request.

Live source execution and readiness queries share a registry selection of the
canonical input template against settled owner state. Blackbird's owner binds
ordinary action names to current ordinary/evolved native abilities and packets.
Both consumers retain the same owner costs, permission and cooldown admission;
executed native exports retain the selected identities for replay. The source
interpreter supplies inputs while the owner defines replacement forms.

## Live source ATK for healing (2026-10-24)

The accepted Iroi healing calculator reads live ATK. Equipment ATK percentages
previously existed only in packet damage hooks, so a callback could not query
the actual source stat without inventing a damage packet. Content now exposes
one packet-independent gear ATK query, shared by damage and healing. It retains
the owner proc's unconditional or existing HP/shield/team-element predicate;
packet-specific damage/crit/DEF bonuses stay in their weapon hooks. Callback
ATK combines compiled ATK, base-scaled live percentages, flat buffs and living
team ATK effects. It never applies target, damage or crit multipliers to healing.

This is a shared content service with two concrete consumers, not a second
formula engine or a core dependency on equipment. Independent owner damage
oracles preserve the moved stats; Iroi checks distinguish ATK from MaxHP,
check outgoing/incoming healing zones, expiry and dead sources/recipients.
Complete native/WASM/Worker equipped reports check the transport boundary.

Koinobori DEF now uses the same packet-independent stat boundary. Its conditional
base-DEF percentage is read by damage, Adler Blessing's existing snapshot and
Adler A4's live overflow cap. The source reader reuses the existing DEF formula
with living-source buffs and flat additions; it does not construct a synthetic
damage packet or multiply the compiled total. Shield timing, eligibility and
mitigation remain Adler-owned. Native three-element/two-element cycles and
actual Worker incoming-damage fixtures check this additional concrete consumer.

Good Boy's unconditional Charge Efficiency follows the existing static build
boundary: the weapon owner selects the frozen refinement ratio; native and
Studio/DSL compilers add it once to the existing received-energy stat. The UI
and runtime consume the same compiled multiplier, including the same fractional
carry and off-field ledger. Runtime callbacks do not infer this static bonus
again from the weapon ID. The additional Ultimate-control ATK branch consumes a typed successful-control
receipt through gear dispatch; static efficiency projection does not imply
control eligibility or fabricate a control event.

Target DEF now has a matching packet-independent reader for a concrete
base-scaled debuff. Sakiri's successful Suppress subtracts a base-DEF percentage
from authored total DEF; direct hits, Break, Blossom and Scorch share the same
exact delivery-time value. Target `base_def` is optional authored metadata,
with total-as-base as the explicit fallback. No enemy catalog or native profile
is inferred. Owner control requests use existing target buff lifecycle and
produce typed successful-control receipts on the causal hit; core remains
independent of Sakiri and of control immunity policy.

Conditional equipment now consumes that same typed control result after the
producing owner observer. Good Boy upgrades only its matching admitted command
and currently owned shared snapshot, retaining the original deadline. The
receipt is passed directly as a domain event and copied into the hit report;
no browser logic or damage/airborne proxy reconstructs successful control.
Existing shared buff reset semantics accept the shorter remaining duration,
so this adds no second clock, polling or core weapon-specific branch.

Party target-death observation now has one registry boundary before target
status cleanup for ordinary packets, reaction damage and finishing damage.
Chaos A4 consumes the real kill and its still-present source-owned Warrant;
the owner selects candidates and reuses the ordinary target-status service.
No scene/expiry event is converted into damage or kill attribution, and mark
application does not dispatch another death. This preserves same-packet AoE
causality: a reached transfer is visible to later recipient deliveries.

## Reconsider when

Jiuyuan's natural Pact binding additionally gives the existing speculative
before-hit context a read-only scenario reference. The owner selects its exact
accepted lifecycle model while legacy explicit fixtures retain their declared
status-only boundary. Projection occurs at admission; contact applications and
periodic delivery use the existing content callback queue. No owner state or
rules move into core, and no new simulation dispatcher is introduced.

Several implemented consumers demonstrate a missing common service, repeated cross-owner changes reveal misplaced responsibility, or a boundary requires speculative generic machinery for otherwise simple behavior. Evaluate a narrower service or move responsibility with all affected consumers. Preserve causal order and core independence even if the file or crate organization changes.

## Captured damage bases and stock rejection

The existing content hit modifiers now admit an optional captured full base.
Chiz's real Coin1 consumer supplies floored settlement proceeds; shared direct
damage replaces stat/coefficient scaling once and still applies live zones.
An optional exact hit-receipt field discloses this choice while retaining the
native fallback coefficient as provenance. Core requires no character branch.
Owner packet semantics may declare an exact source fixed-crit sentinel and
its selected interpretation; the metadata boundary verifies both the raw
sentinel and the accepted model, preserving ordinary probability behavior.

The registry also exposes an owner resource rejection after ordinary timing
checks and before admission mutation. Chiz's exhausted Skill stocks use the
existing typed `Other` admission reason, command/time and diagnostic. Refill
is an actual queued deadline; rejection never inserts a wait or spends energy.
Existing provenance/packet checks retain their order and diagnostics.

The existing hold transport now admits an optional owner-validated stock
reservation. The default registry policy rejects it; Chiz consumes2/3 through
its real stock pool. Omitted options preserve existing requests and reports.
Buffered continuation retains its actual later input and uses the existing
owner recovery override/callback boundaries. Neither the compiler nor core
infers extra casts, financial transactions or charge refunds.

## Wide exact resource compounding

Chiz A2's selected10% interest requires a wider resource representation than
the existing checked-u128 damage/short-resource arithmetic: `450*(11/10)^37`
needs132 numerator bits while its15301.7768... balance is still below16500.
The generic kernel now supplies a capped rational pool with4096-bit component
bounds and atomic failed grants/spends. Its actual content consumers are own
Grain, drawn/repaid loan principal and Surplus interest. Chiz owns own-first
spend, repayment, eligibility, cadence and lifecycle; the kernel supplies only
capped conserved ledgers, including exact partial spends and capacity reads.
Core imports no character data or content. Damage arithmetic is unchanged.

The existing fractional-resource report map accepts either its numeric exact
summary or a wide summary with decimal-string fraction components. The wide
producer compacts only when every component is a safe JSON integer; otherwise
all components in that summary remain strings. Existing small reports retain
their wire representation, and deserialization preserves wide strings through
round trips. Rust/native checks and both WASM codecs retain exact balances and
cap overflow; consumers transport these values without reconstructing finance.

## Verification

The Remora service now accepts Hathor's consumed duration/target-CR policy and
stores it on the shared generation. Ordinary critical calculation reads the
specific recipient's live status after fixed-crit precedence; source policy
selection stays in the owner. This also supplies one consistent lifetime to
Charge and Chaos's pending expiry consumer, without owner IDs in the shared
critical calculation or an unconditional team buff.

[Architecture checks](../../tests/test_architecture.py) protect dependency direction and core boundaries. For behavior-preserving moves, compare deterministic reports, event ordering, resource conservation and RNG; affected browser boundaries also require [native/WASM parity](../../openspec/specs/wasm-transport/reference.md). Existing owner oracles remain authoritative for mechanic behavior.
