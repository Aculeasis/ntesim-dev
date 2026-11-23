# 0017: Sequential admission boundaries in native replay

- Status: accepted
- Recorded: 2026-11-01
- Decision date: 2026-11-01
- Current owners: [DSL execution](../../openspec/specs/script-execution/reference.md), [runner](../../openspec/specs/combat-execution/reference.md), [event scheduling](0004-event-driven-simulation.md)

## Context and decision

A live source input follows the settled callbacks, resources and death at its
current timestamp. An exported timestamp alone cannot retain that order:
prequeued native inputs ordinarily use ActionAdmission priority, before hits
and callbacks. A Jiuyuan Skill at the same time as an existing Pact tick can
therefore exchange critical draws and damage order on replay.

The native request carries typed `action_admission_order`: `scheduled` uses
ordinary scheduled admission; `after_settled_events` queues authored inputs at
the existing Observation boundary. Live source compilation selects the latter
for its canonical request and executed export. Source injection uses that same
boundary. Inputs still run native admission and can enqueue earlier-priority
descendants; each causal closure settles before the next sequential input.

Action-bound explicit parry assumptions retain `after_action {command_index}`
in the canonical request. They derive owner/time from the supported parent and
enter ProcCallback priority only after that input is admitted. Native replay
indexes these references once; live source registers each actual occurrence.
A rejected input schedules no callback. Fixed `at {timestamp_us,actor_index}`
parries remain independent scenario events. A timestamp alone cannot distinguish
an existing callback before a source input from this input's own descendant.

Owner generation identity must also be independent of how many future inputs
were queued. Jiuyuan Pact allocates a source-local generation on an accepted
new application, rather than using the global queue insertion sequence.
Talent projection receipts follow command index order, independent of whether
projection resolved all owners in advance or one live input at a time.

## Rationale

This retains one event engine, real timestamps, costs, RNG and partial failures
while making the executed request sufficient for replay. Changing all native
priority ordering would change intentionally co-scheduled research scenarios.
Adding a microsecond would change actual combat time and deadlines. Comparing
only totals would hide critical-draw, resource and death discrepancies.

## Consequences and assumptions

Scheduled native scenarios retain their explicit semantics. The new field is
Rust-owned transport metadata, not another DSL option or an alternate combat
rule. Serialization preserves it for CLI/WASM/Worker consumers. Owner callback
phases remain unchanged. This does not provide resumable combat state or
complete pending source action/cancellation integration.

## Reconsider when

Native input artifacts need several distinct observation boundaries at one
timestamp, or a supported sequential continuation cannot be represented by
this causal closure. Preserve source/native event order, RNG and real time.

## Verification

`simulation/tests/source_causality.rs` checks a due Pact tick before a same-time
Skill, whole serialized native replay and the opposite intentional scheduled
order. It also checks equal-time owner death before source admission and the
scheduled admitted-then-cancelled outcome. PowerBullets source tests exercise
continuing Normal/Skill cycles and native replay. Existing scheduled admission
and owner barrier tests remain in their native suites. Daffodil Resonance
source tests cover interleaved two-owner inputs and Support with complete
projection-receipt replay.

[Source parry checks](../../crates/content/src/simulation/tests/source_parry.rs)
retain two paid Dodge cycles for three owners, implicit-switch identities,
Blackbird's one-use transformed permission, two Daffodil entry periods and
qualified Support with its1.667260s recovery. Invalid entry assumptions remain
failed native replays; duplicate/wrong-family/out-of-range references reject.

2026-11-06 clarification: an executed source request additionally retains `source_exhausted` when its lexical program ends. Under sequential admission this permits autonomous effects beyond a completed time boundary without promoting an unfinished requested input. The native `stop` policy and report `elapsed_us` preserve victory timing and safety-horizon failure; pending counts remain observations rather than an eligibility rule.

2026-11-06 input-time clarification: resolved `input_time_contexts` preserve source continuation spans, occurrence identity and UTF-16 ranges for exclusive time accounting. Action-window occupation can outlive ordinary recovery, so timestamps and packet metadata alone cannot reconstruct its classification. These annotations do not affect combat admission, scheduling or damage; native replay consumes them for identical time receipts. Inputs with changed command lists must update or remove these source annotations. Independent probe-invariance tests compare combat fields separately from authored time provenance; full replay still compares every report field.

2026-11-06 failure-boundary clarification: a timestamped command prefix cannot
represent a lexical evaluation/work/action failure or an unresolved source gate.
The executed request retains typed `source_halt` with the settled time,
failed/pending outcome, diagnostic and occurrence context. Replay settles that
prefix using the same native engine and executed seed, then halts without draining
later effects. This is terminal provenance, never resumable combat state or a
success override. Native admission rejection remains an actual rejected command.
Source program checks cover exact full-report replay, continuing Pact effects,
implicit-switch prefixes and invalid boundary rejection; CLI checks cover saved
compiled artifacts and native exported-request aggregation.

The same terminal provenance binds an event-budget cap when native execution
stops during an input or after lexical exhaustion. Replay reexecutes that cap
and reconciles the resulting boundary; it cannot inject a combat snapshot or
raise the caller's limit. Native program Wait annotations follow the same
reached branch/energy schedule shifts as queued commands, including final
waits with no following event. Exact horizon endpoints complete; later endpoints
remain pending without adding polling events.
