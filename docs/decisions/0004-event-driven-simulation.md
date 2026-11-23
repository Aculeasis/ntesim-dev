# 0004: Event-driven deterministic simulation

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision date: unknown
- Current owners: [architecture](../ARCHITECTURE.md#determinism-and-time), [DSL execution](../../openspec/specs/script-execution/reference.md#event-driven-time-advancement)

## Context and decision

Rotations include sparse waits, pending hits, resource grants and timed state changes. Advance to the next relevant event, readiness deadline, wait endpoint or horizon. Use integer simulation time, stable equal-time ordering and explicit seeded RNG; do not retry readiness on arbitrary frame or microsecond ticks.

The requirements are causal timing, reproducibility and the real elapsed combat-time denominator. Event-driven scheduling is the chosen mechanism. Timestamp precision does not require iteration at that precision, and a temporary compiler limitation is not a scheduler policy.

## Rationale

Fixed-tick polling makes execution cost scale with empty duration and the chosen tick rate, and can either miss exact readiness boundaries or introduce artificial delays. Event- and deadline-driven advancement advances time directly to the next causal state change, keeping scheduler performance independent of sparse wait durations. Integer timestamps and deterministic seeded RNG guarantee bit-for-bit reproducible results across CLI, WASM, and Web Worker execution.

## Consequences and assumptions

Owners must expose relevant deadlines and grants; equal-time settlement needs deliberate checks. Real periodic hits and resource/proc ticks remain events. Event-driven execution must not omit periodic mechanics or invent a future wake-up for impossible waits. It assumes the relevant modeled state changes can be represented by events, deadlines or causal elapsed-time calculations.

The single public real combat-time integration has its own pending boundaries in the linked contracts. Internal local clocks and Hotori process policies do not authorize another public DPS denominator; this record does not declare their product integration complete.

### Analytic stamina integration

Combat stamina uses the analytic elapsed-time option above. One typed optional
scenario profile supplies the account pool and rates; the shared content
service advances exact recovery at existing events and final observation time,
after the current spending-pause deadline. It creates no periodic refill events
and makes no hidden account-state choice. Admission checks before spending;
owner actions reuse the same account pool and record actual command ownership.
The sparse-horizon stamina check preserves event count while reaching capacity.
This extends the selected mechanism without replacing owner action clocks or
certifying the native recovery carrier's universal applicability.

### Caster progress and real producer clocks

Baicang's explicit caster-clock model consumes the existing exact PPM hitlag
integrator. Positive contact schedules a meaningful delayed pause event only
when the owner specifies a delay; immediate pauses settle in the causal hit
callback. Reproject only unreleased producer coordinates, recovery and legal
cancel markers. Released projectiles and activated field/Word timers retain
real deadlines. A pause can outlive an action or field exit without freezing
another actor, cooldowns or the DPS denominator. The owner retains current
action anchors and only the unexpired windows needed by the next action;
there is no periodic actor-clock poll or global hitstop catalog.

### Owner resource decisions at a recovery boundary

2026-10-28 clarification: variable source hitpause rates use the same analytic
deadline mechanism. Typed linear rate knots and overlapping constant windows are
split at exact rate crossings; rational integration preserves partial progress
through interruptions. Deadline inversion uses bounded bisection within the
reached effective linear piece. It is mathematical clock inversion, not a loop
that polls game readiness, and introduces no periodic simulation events.

Chaos Verdict uses one two-phase completion event: the reached boundary first
defers only equal-time queued ordinary admissions/switches to Observation,
then commits its owner decision in ProcCallback after due HP/resource contacts
and immediate death settlement. This lets the decision extend recovery before
another authored input, while a same-time fatal incoming hit prevents spending
or granting new immunity. The shared queue helper preserves timestamps and
sequence identity; it neither retries admission nor inserts elapsed time.
Auto ticks remain ordinary ModePause events, so the decision sees an equal-time
tick regardless of insertion order, without polling or duplicate settlement.
This is an owner-requested causal barrier, not a global change to all action
admission ordering. Checks cover the tick/resource, HP/death and new-action
boundaries plus ordinary interruption immediately before/at completion.

### Readiness and a bounded owner continuation lifetime

Chaos full Verdict exposes an AnySkill readiness gate before its later reached
EndAbilitySkill marker. Retain that finite source window for its actual hold
predicate and native Jump gate, while ordinary next actions remain available
from AnySkill. The existing per-input recovery override now supplies its exact
owner deadline, including a later gate for that specific input; it does not
change another input's readiness or insert a wait. Earlier bindings already
select their own clamped deadlines and retain their behavior. A failed early
hold/Jump returns the typed recovery deadline after general readiness as well.
Interruption/exit/defeat or another admitted action consumes the source; expiry
is an analytic owner deadline, without retaining unused pose-tail length or
polling. Full-stage immunity retains its separately declared real cap/recovery
policy. Checks separate ordinary readiness, strict continuation admission and
exclusive source end, with continuing native resource/replay coverage.

## Reconsider when

Explicit action-indexed Dash direction now selects the existing owner clock
profile at commit. Chaos and Blackbird capture their native Back recovery and
cancel gates, including Blackbird's live form. The profile records movement
intent without geometry; shared stamina and dodge cooldown remain independent.
Front preserves prior reports, while the reached Back receipt hypothesis retains
the selected intent. No compiler-side owner-state reconstruction or hidden wait
is introduced; public source lowering stays with native workflow.

A verified damage-relevant mechanic requires continuous evolution that the existing event/threshold model cannot represent accurately or economically. First evaluate an owner-scoped analytic calculation or meaningful threshold events. Any alternative must preserve causal order, exact readiness and the declared denominator; a convenient fixed-tick loop is not evidence by itself.

## Verification

Follow the execution owner's sparse-gap, intervening grant/expiry, exact deadline and equal-time checks. Empty gap length must not multiply scheduler work for an unchanged event structure. Verify deterministic native/CLI/WASM/Worker behavior when affected; acceptance expectations do not assert that every pending live-language path already passes.
