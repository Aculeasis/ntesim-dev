# Explicit finite cycle checks

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`check-cycles-synthetic FIXTURE.json ROTATION.nte CHECKS.json` runs a synthetic
rotation and checks caller-declared conditions at labeled
[state observations](../state-observations/reference.md). Version:
`synthetic-finite-cycle-check-v1`. It retains the entire underlying rotation
report, including partial execution and source origins. It does not reset or
repair state, and passing finite checks is not a proof of infinite sustainability.

## Explicit request

Schema 1 requires label, clock, positive `interval_us`,
`max_phase_error_millionths`, exact `expected_observations` (2..10,001), and
`resources`. Each resource rule requires a nonnegative `minimum` and relation
`equal`, `non_decreasing` or `unconstrained`. No omitted tolerance, floor or
relation is inferred. Empty resource maps explicitly request time/count checks
only. At most 128 resource rules and a 1-MiB request are allowed. Labels follow
observation limits. Unknown clocks/resources, invalid fields/schema/relations
and malformed rotations fail before simulation with exit 1.

Selected commands are limited to a conservative upper bound of 1,000,000
causal events as in seeded batches. Simulation dependency checks still apply.
The fixture's original seed is used; no seed override or fresh per-cycle state
is introduced.

## Boundary and interval evaluation

Only observations with the exact requested label participate, in delivered
trace order. Each boundary records its event/source-command IDs, real time,
selected clock's `total_phase_millionths`, requested pool values and signed
changes from the previous boundary. The first boundary has no previous delta.
Full snapshots, actors/RNG/target state and iteration mapping remain in the
nested simulation trace.

Total phase is integer local microseconds * 1,000,000 + the scheduler's fractional
phase. Adjacent interval differences are compared to `interval_us * 1,000,000`;
their absolute error must not exceed the declared tolerance. Fractional error
is never rounded away. Actual/expected/error phases serialize as decimal u128
strings. Rates and pauses are actual snapshot readings, not asset/FPS assumptions.

Every selected resource must meet its floor at every boundary, including the
first. `equal` compares adjacent current values for equality; `non_decreasing`
requires the next value >= previous; `unconstrained` checks only the floor.
Signed changes serialize exactly as decimal integers. The checker does not
infer nondecrease merely from a large initial resource buffer. Capacity/action/
target/RNG recurrence is not part of these particular rules.

All violations are retained with boundary-index diagnostics: `RESOURCE_FLOOR`,
`RESOURCE_RELATION`, `CYCLE_INTERVAL`, `OBSERVATION_COUNT`, and
`SIMULATION_INCOMPLETE`. Missing labels produce zero observed boundaries and a
count failure, not an invented state sample. Extra boundaries also fail the
exact-count requirement. Failed simulation or any pending events prevents a
passing check, even if all requested snapshots were already captured.

`complete` means execution succeeded, no events remain and the exact requested
boundary count was observed. `passed` additionally requires every declared
invariant to hold. Complete execution with invariant violations is a failed
check. Exit 0 requires `passed`; violations/incomplete execution retain a JSON
report and exit 2. The nested simulation has its own success/diagnostics.

## Identity, example and limits

Input identity hashes the ASCII audit-version tag followed by fixture, rotation
source and checks bytes, each with an unsigned 64-bit big-endian length prefix.
`checks_sha256` hashes original request bytes; the nested rotation retains
original fixture/source/input identity. Changing a tolerance changes audit
identity without changing the underlying simulation.

The example asks for four ten-microsecond boundaries and nondecreasing energy.
Its simulation succeeds, but energy 3,2,1,0 produces three relation failures,
so the checker intentionally exits 2. Explicitly adding one energy grant per
actor action satisfies these finite resource conditions. That passing variant
still does not establish behavior beyond the observed execution. A 500,001-ppm
clock has a 20-millionth error per boundary interval in the example; tolerance
0 fails, 20 passes the time condition and 19 fails.

Current tests cover resource debt versus execution success,
explicit replenishment, fractional tolerance, floor/equality/count requirements,
failed/pending execution, validation and independent audit/rotation identities.
Native process verification checks exit codes 0/1/2, exact boundary
deltas/interval arithmetic, byte-identical replay, passing replenishment and
the independent three-input hash oracle. Local verification is saved in
`target/validation/cycles/verification.json`.

These checks concern only the declared conditions at a finite set of boundaries.
They do not prove conditions between observations, a recurring scheduler state,
infinite resource sustain, game clock bindings or validated game DPS. Full strict
adapters and game acceptance remain pending; the existing browser retains incomplete quality.
