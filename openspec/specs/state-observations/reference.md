# Causal state observations

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

Synthetic `observe_state` effects with an explicit `label` capture current
state at their effect execution. Labels must be nonblank and at most 128 UTF-8
bytes; repeated labels are allowed for cycle boundaries. Selected observations
are validated before simulation, including actor leaf effects. The observation
version is `synthetic-state-observation-v1`.

The effect creates no hit, resource transition, action admission, RNG draw or
clock-rate change. It is an ordinary scheduled command/effect (or actor leaf),
with stable event/cause IDs, source command/iteration mapping and optional actor
origin. Inserting observations changes trace IDs and pending counts; it preserves
the relative ordering of existing combat transitions. No implicit action wait,
flush, queue priority or boundary synchronization is added.

## Snapshot contents and timing

`changes.observation` contains label/version, real time, every clock's integer
local time, fractional phase and rate, current pending-event count, target HP/
capacity/shields/order, pool values/capacities and actor gates/actions/readiness.
Configured streams add the current RNG audit state; with no streams `rng` is
omitted. All maps serialize in deterministic sorted order. Snapshots contain
only current state, never future rolls or simulated lookahead.

`clock_readings()` exposes the scheduler's retained fraction as
`phase_millionths`, with denominator 1,000,000 and range 0..999,999. Together
with `local_time_us`, this preserves its exact fixed-point phase. `rate_ppm`
is the current declared rate; zero identifies a paused clock. Overflow of the
integer reading fails without truncation or mutation. No FPS or game animation
binding is inferred.

Pending count is taken after the observation event was popped, and includes
all remaining commands/effects/hits, including future cycle commands. There
is no promise that earlier same-timestamp commands have completed all causal
children: global event IDs still govern ties. An actor leaf observation after
a random damage leaf can see the spent cost/consumed roll before its delayed
hit lands. Wait explicitly to observe a desired settled boundary.

Snapshots are audit records, not a complete scheduler checkpoint or restoration
interface: pending task contents and delivered/canceled ID sets are not embedded.
They do not alone certify periodic execution or infinite sustainability. The
engine also does not reset HP, resources, cooldowns or RNG at observations.

## Continuous-cycle example and verification

`synthetic-cycles.json` / `synthetic-cycles.nte` observes local boundaries
0, 10, 20 and 30, with actor actions at 1, 11 and 21 and delayed hits two local
microseconds later. The 20-event trace shows HP 200,175,150,140; energy 3,2,1,0;
and RNG raw counts 0,1,2,3. Skill ready deadlines after actions are 11,21,31.
The target, pool, cooldown and random stream remain continuous across repeats.
A fourth invocation fails its explicit cost instead of obtaining a fresh pool.
These values are synthetic technical checks, not observed game measurements.

Two scheduler tests cover fractional readings, rate changes, pauses, repeat
reads and overflow. Four runner tests cover complete boundary state, parity
with an unobserved run, fractional/paused clocks, label validation/RNG omission
and actor origin/current-state ordering. One CLI test joins repeated observations
to source iterations. Native process checks verify byte-identical
replay, boundary states, a failed fourth cycle, and the 500,001-ppm fractional
clock oracle; results are saved locally at
`target/validation/observations/verification.json`.

The [finite-cycle checker](../cycle-evaluation/reference.md) now uses these observations for
explicit resource floors/relations, clock intervals/tolerances and counts. Full
sustainability proofs, game clock/action bindings and strict gameplay coverage
remain pending; finite examples do not establish strict gameplay readiness.
