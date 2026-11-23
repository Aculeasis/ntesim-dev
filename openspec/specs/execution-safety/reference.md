# Execution limits, overflow, causal ordering, invariants, and partial-failure replay contract

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

This contract defines execution limits, overflow/error safeguards, stable causal ordering, state invariants, transactional atomicity, and partial-failure replay in `ntesim-core::execution`. It is a standalone harness; gameplay changes are tracked in [OpenSpec changes](../../README.md).

## 1. Execution limits and bounded simulation

To prevent infinite loops, unbounded recursion, and memory exhaustion from runaway procs, callbacks, or recasts, simulation is constrained by explicit bounded limits (`ExecutionLimits`):

- **Maximum event count (`max_events`)**: Default `100,000`. Limits total simulation events; halts execution if exceeded. The same work bound limits configured dummy count at preflight; target count has no four-actor party cap. Script inspection uses the default work bound. Owner-specific selection caps still apply independently.
- **Maximum script operations (`CompileLimits.maxSteps`)**: Default `60,000` per trial. Counts evaluated expression nodes and live control/action operations cumulatively, independently of simulated combat time. Exhaustion reports the configured operation count. Expression nesting has a separate limit of 128 levels.
- **Maximum script inputs/commands (`CompileLimits.maxActions`)**: Default `10,000`. Bounds yielded actions/waits and native commands, including implicit switches. Exhaustion identifies which budget was reached and its configured count.
- **Maximum horizon (`max_horizon_us`)**: Default `3,600,000,000` $\mu\text{s}$ (60 minutes). This is the independent absolute scheduler ceiling for combat execution. Relative targets are validated independently; a legal target can remain incomplete at this ceiling when pauses prevent reaching it. Never enlarge the caller ceiling to fit pauses.
- **Maximum proc recursion depth (`max_proc_depth`)**: Default `4`. Procs triggering procs cannot recurse beyond depth 4.
- **Maximum resource capacity (`max_pool_capacity`)**: Default `1,000,000,000`. Prevents arithmetic overflow in resource grants.
- **Maximum active shields (`max_active_shields`)**: Default `32`. Bounded per-actor shield buffers.

If any limit is exceeded, an explicit `ExecutionError::LimitExceeded` is generated, preventing panics, infinite loops, or uncontained allocation.

Source execution retains typed `work_limit`, `action_limit`, `event_limit` and `horizon` diagnostics with source coordinates and partial reports. Event exhaustion reports the configured event count; victory time-limit failures report the configured combat duration in seconds (normally 3,600). A victory rotation that ends with no pending effects before the horizon reports early rotation exhaustion instead of claiming the time limit elapsed. Successful `stop=time` completion remains successful.

## 2. Stable causal ordering and tie-breaking

All events are scheduled on a discrete microsecond timeline and sorted using a deterministic lexicographical key:
\[
(\text{time\_us}, \text{priority}, \text{sequence\_id})
\]

Microseconds define timestamp resolution, not a simulation iteration step. The scheduler advances directly to the next eligible event or horizon, without visiting empty frames/microseconds. The future live interpreter must preserve this property while adding readiness deadlines and wait endpoints; see [event-driven advancement](../script-execution/reference.md#event-driven-time-advancement).

### Causal priority hierarchy (`CausalPriority`)

When multiple events share the exact same microsecond timestamp, they execute in strictly ordered causal phases:
1. `mode_pause` (0): Game-mode challenge clock freezes/resumptions.
2. `action_admission` (1): Command checks, lockouts, cooldown checks, and atomic state transitions.
3. `resource_cost` (2): Resource deductions and grants.
4. `buff_tick_or_expiry` (3): Buff expirations, periodic ticks, and ICD resets.
5. `hit_delivery` (4): Attack hit arrival, direct damage, and healing.
6. `reaction_or_break` (5): Elemental reaction triggers, break gauge depletion, and break damage.
7. `proc_callback` (6): Secondary event listeners and passive activations.
8. `settlement_and_death` (7): Vital checks, shield removal, and active actor field transfer on death.
9. `observation` (8): Causal state snapshots and diagnostic probes.

### Monotonic sequence tie-breaking

Simultaneous events with identical `time_us` and identical `priority` are resolved in strict FIFO scheduling order using a monotonic 64-bit `sequence_id` counter. No two events ever share an ambiguous execution slot.

## 3. Transactional atomicity and partial-failure replay

- **Atomic action admission**:
  - Before an action mutates state (deducting resources, setting lockouts, or scheduling hits), all admission prerequisites must be satisfied: actor alive, actor on field, actor not in recovery lock, actor not on cooldown, and sufficient resource balances.
  - If any admission check fails, the transaction is rolled back completely (`state = pre_step_state`), leaving all pools, actor vitals, and cooldowns unmutated.
- **Partial-failure diagnostic**:
  - Halts execution cleanly and populates `halt_diagnostic`:
    - `failed_step_index`: Index of the offending request step.
    - `time_us`: Timestamp when failure occurred.
    - `code`: Structured diagnostic code (e.g. `"RESOURCE_SHORTAGE"`, `"ACTOR_LOCKED"`, `"LIMIT_EXCEEDED"`).
    - `message`: Human-readable error description.
- **Replay invariance**:
  - Partial traces and final state snapshots up to the failure point are fully preserved.
  - Rerunning the scenario produces byte-identical traces, byte-identical state snapshots, and identical `state_sha256` / `trace_sha256` hashes.

## 4. State invariants verification

At initial state, after every step, and upon simulation settlement (including partial failure), the engine verifies six fundamental state invariants:

1. **Resource conservation**:
   For every resource pool:
   \[
   \text{initial\_amount} + \text{total\_granted} = \text{current} + \text{total\_spent} + \text{total\_overflow}
   \]
   No energy, stamina, or resonance can be created or destroyed unaccounted.
2. **Actor vital bounds**:
   \[
   0 \le \text{current\_hp} \le \text{max\_hp}
   \]
   $\text{is\_alive} = \text{true} \iff \text{current\_hp} > 0$.
3. **Target vital bounds**:
   \[
   0 \le \text{current\_hp} \le \text{max\_hp}
   \]
4. **Shield non-negativity**:
   \[
   0 \le \text{remaining\_hp} \le \text{max\_hp}
   \]
5. **Active actor validity**:
   Unless all actors in the team are dead (`party_wiped: true`), exactly one actor has field ownership (`active_actor < team_size` and `actors[active_actor].is_alive = true`).
6. **Clock monotonicity**:
   Audit event timestamps $t_k$ are non-decreasing: $t_k \ge t_{k-1}$.

If any invariant is violated, an `InvariantViolation` error is raised immediately, halting simulation and marking `invariants_passed: false`.

## 5. Core engine purity

The core library `ntesim-core` strictly enforces zero external system side effects:
- **Zero filesystem access**: No `std::fs` calls in library code.
- **Zero network calls**: No network sockets or HTTP requests.
- **Zero DOM / browser references**: Pure decoupled Rust logic ready for WASM compilation.
- **Zero system clock dependencies**: No `std::time::SystemTime::now()` or `Instant::now()`. All clocks are driven deterministically by explicit input parameters.

## 6. CLI execution and independent verification

The native CLI exposes:
```bash
ntesim execution-stats data/global-2026-10-04 examples/execution-scenario.json
```

Output contract:
- `status`: `"ok"` or `"failed"`
- `content_id`: `"global-2026-10-04"`
- `success`: boolean
- `execution_completed`: boolean
- `halt_diagnostic`: optional diagnostic object if halted early
- `horizon_us`: simulation horizon in microseconds
- `final_time_us`: timestamp of last executed event
- `final_active_actor`: index of active actor at settlement
- `party_wiped`: boolean
- `total_events_processed`: total executed event count
- `max_proc_depth_reached`: peak proc recursion depth observed
- `invariants_checked`: total number of invariant validations performed
- `invariants_passed`: boolean
- `state_sha256`: SHA-256 hash of final state
- `trace_sha256`: SHA-256 hash of audit event trace
- `actors`: final actor vitals, shields, and damage/heal stats
- `targets`: final target vitals, shields, and damage stats
- `resource_pools`: map of pool states and conservation audit receipts
- `events`: complete chronological audit log
- `quality`: `"incomplete"`
- `combat_adapter_complete`: `false`
- `game_dps_computed`: `false`

Verification oracle:
- `tools/verify_execution_cli.py`:
  - Independent Python state machine matching all 10 scenario events.
  - Verification of exact resource conservation and target HP endpoints.
  - Rejection of 12 invalid configurations (zero team size, team size > 4, actor count mismatch, active actor out of bounds, bad package ID, zero horizon, zero max HP, current HP > max HP, time reversal, horizon limit exceeded, duplicate pool ID, extra schema fields).
  - Transactional rollback on insufficient resource, verifying invariant preservation.
  - Byte-identical replay and relocation invariance check.
  - Cached in `target/verification/execution-state.json`.

Requests enforce schema version, ordered actor identities and initial resource capacity; oversized initial amounts are rejected rather than clamped. Positive costs/grants require their named pools. Elemental damage requires an explicit source actor for attacker-level defense mitigation; no demonstration level is substituted. Resource operations use core Pool and shield/HP absorption uses core VitalState; overflow in retained counters fails explicitly. State/trace serialization errors cannot be hashed as empty receipts.

Actor healing and shield admission use shared VitalState. Every shield producer checks duplicate identity; healing, overheal, observation and sequence counters reject overflow instead of saturation. Invalid transitions retain the pre-step state in the incomplete execution report.

Relative combat execution recomputes its absolute stop projection at real pause transitions. It jumps to the next eligible event, release, explicit wait endpoint or safety bound without polling frozen clocks. Diagnostics retain the absolute elapsed/ceiling and relative elapsed/target distinction. Operation, action and event budgets remain independent of elapsed time.
