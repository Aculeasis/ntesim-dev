# Actor lifecycle, action state machines, and field ownership contract

This contract defines field ownership, action phase state machines, cancel windows, interruption, death, refund policies, forms, and recast windows in `ntesim-core::lifecycle`. These checked harness services support work in the [OpenSpec changes](../../../openspec/README.md); their tests do not certify complete gameplay integration.

## 1. Field ownership

A combat team consists of 1 to 4 distinct actors (`team_size`).

- **Active on-field actor**: at any moment, while at least one actor is alive, exactly one living actor is designated as the on-field active actor (`active_actor_index`).
- **Off-field actors**: all remaining living actors are in the off-field state (`FieldStatus::OffField`).
- **Field constraints**:
  - Actions marked with `require_on_field: true` (such as normal attack chains and standard on-field skills) can only be initiated by the active on-field actor. Initiating an on-field action with an off-field actor returns `LifecycleError::RequiresOnField`.
  - Actions with `require_on_field: false` (such as deployable off-field skills or coordinated attacks) may be admitted by off-field actors while other actors are on-field.
- **Character switching**:
  - `switch_to(target_index, now_us)`:
    - Preflight checks: `target_index < team_size`, `target_index != active_actor`, actor is alive (`!is_dead`), and `now_us >= switch_locked_until_us`.
    - Active actor check: the current active actor must be either in `Idle` or in a phase where `CancelType::Switch` is permitted and `now_us >= earliest_cancel_us`. If the active actor is locked in an uncancelable phase (e.g. `Startup`), switching is rejected with `ActiveActorLockedInPhase`.
    - If cancelable via `Switch`, the active actor's action is canceled, field ownership transfers to `target_index`, and `switch_locked_until_us` advances to `now_us + switch_cooldown_us`.
    - Records an auditable `SwitchRecord { from_index, to_index, timestamp_us }`.

## 2. Action phase state machine

Each action executed by an actor progresses through three distinct, deterministic sequential phases:

1. **Startup (Windup)**:
   - Duration: `startup_us`.
   - Character winds up the weapon or charges the cast before the commitment / hit point.
   - Typically uncancelable by standard player cancel inputs (unless explicitly specified in `startup_cancels`).
2. **Active (Execution / Hit frames)**:
   - Duration: `active_us`.
   - The attack executes and damage / hit events occur.
   - Uncancelable by standard inputs to guarantee hit delivery.
3. **Recovery (Endlag)**:
   - Duration: `recovery_us`.
   - Character completes endlag or sheath animation.
   - This phase permits animation cancels (Dash, Jump, Switch, Skill) once `now_us >= recovery_start + earliest_offset_us`.
4. **Idle**:
   - The actor is ready to begin a new action.

Time advancement (`advance_time(now_us)`) causally transitions actors through `Startup -> Active -> Recovery -> Idle` as phase deadlines expire, emitting `PhaseTransition` audit records.

Team commands commit clock advancement, phase changes, pool settlement and field ownership atomically. Rejected commands leave all of these unchanged; a failed admission is not compensated by a grant that can alter resource accounting.

## 3. Cancel windows

Cancels are governed by explicit per-phase `CancelWindow { cancel_type, earliest_offset_us }`:

- Supported cancel types:
  - `Dash`: dash cancel.
  - `Jump`: jump cancel.
  - `Switch`: character switch cancel.
  - `Skill`: combo / skill chain cancel.
- A cancel request is valid if:
  1. The actor's current phase allows that `cancel_type`.
  2. `now_us >= phase_started_at_us + window.earliest_offset_us`.
- If the cancel type is not permitted in that phase, it returns `LifecycleError::CancelNotAllowed`.
- If attempted before the earliest offset, it returns `LifecycleError::CancelTooEarly { earliest_us, now_us }`.
- On success, the action phase transitions immediately to `Idle`, emitting a `CancelReceipt`.

## 4. Interruption, death, and refunds

### Interruption
Actors may be interrupted by external combat causes:

- `InterruptionCause`: `Stagger`, `Knockdown`, `Death`, `ForcedAbort`.
- **Refund policies**:
  - `RefundPolicy::FullRefund`: 100% of resource costs spent at action start are refunded back to the pool.
  - `RefundPolicy::PartialRefund { ratio }`: an exact fraction (`Ratio::numerator / Ratio::denominator`) of spent costs is refunded using integer floor arithmetic.
    The denominator must be positive and the ratio at most one; invalid policies fail before mutation through both direct action and team entry points.
  - `RefundPolicy::NoRefund`: no resources are refunded.
- **Phase sensitivity**:
  - Interruption during `Startup` (before commitment) applies the specified refund policy to restore resources to pools.
  - Interruption during `Active` or `Recovery` (after commitment) **never** refunds resources regardless of the policy.
  - In all cases, the action immediately aborts to `Idle`, emitting an `InterruptionReceipt`.
  - The team receipt separates actual `refunded_costs` from capacity-clipped `refund_excess`, using the shared pool grant settlement. A missing refund pool rejects the entire transition.

### Death
When an actor dies (`kill_actor(actor_index, now_us)`):

- The actor's vitality becomes `ActorVitality::Dead`.
- Any active action is immediately aborted with `InterruptionCause::Death` and `RefundPolicy::NoRefund`.
- If the deceased actor was the active on-field actor, field ownership is automatically transferred to the next living actor in slot order (0..team_size).
- If all team members are dead, field ownership is unset (`None`), representing a team wipe.
- Dead actors cannot be switched to (`LifecycleError::ActorDead`) and cannot start actions.

## 5. Forms and recast windows

### Forms / Stances
Characters with stance shifts or enhanced states (e.g., weapon transformation, elemental infusion mode):

- `enter_form(form_id, duration_us, now_us)`: enters the specified form with an optional duration.
- `active_form(now_us)`: returns the active form if `now_us < expires_at_us`.
- Automatically expires when time advances past the duration.

### Recast windows
Skills with follow-up / recast windows (e.g. Skill Stage 2 / multi-cast abilities):

- `open_recast(base_action, recast_action, duration_us, max_uses, now_us)`: opens a window lasting `duration_us` allowing up to `max_uses` casts of `recast_action`.
- `consume_recast(recast_action, now_us)`: verifies the recast action matches, checks that the window has not expired, decrements remaining uses, and closes the window when uses reach zero.
- Attempting to cast with an expired window returns `LifecycleError::RecastExpired`.
- Attempting a non-matching action returns `LifecycleError::RecastActionMismatch`.

## 6. CLI execution and independent verification

The native CLI exposes:
```bash
ntesim lifecycle-stats_DIR REQUEST.json
```
Evaluating a scenario produces an auditable JSON receipt with initial state, step execution receipts, final actor states, and remaining resource pools.

### Verification oracle
The independent Python verification oracle (`tools/verify_lifecycle_cli.py`):

1. Simulates the complete 9-step scenario independently using Python state machine logic, verifying final active actor (2), final energy (60), stamina (200), form expiration, and recast exhaustion.
2. Executes `ntesim-cli lifecycle-stats` on `ntesim/examples/lifecycle-scenario.json` and verifies that Rust output matches the Python simulation.
3. Verifies rejection of 12 invalid configurations:
   - Zero team size.
   - Team size > 4.
   - Initial active actor out of bounds.
   - Mismatched characters count.
   - Unexpected package ID (`cn-beta`).
   - Switching to self.
   - Switching on cooldown.
   - Off-field actor initiating on-field action.
   - Canceling too early.
   - Recast action mismatch.
   - Insufficient pool resource.
   - Extra unrecognized top-level fields.
4. Verifies byte-identical replay and relocation invariance with unchanged frozen input hashes.
5. Saves receipt to `target/verification/lifecycle-state.json`.

## 7. Quality boundaries

The lifecycle module adheres to explicit quality boundaries:

- `lifecycle_model_complete: true`
- `combat_adapter_complete: false`
- `game_dps_computed: false`
- Real game DPS and full kit adapters remain pending.

## Runtime admission and snapshots

Team runtime state exposes immutable views, so roster length, actor identity and clock ownership cannot bypass admission through public field mutation. Team commands are atomic. Forms/recasts require living actors; death clears action, form and recast state in this harness (no revival model). This rule does not cancel independent content buff snapshots.

Action state stores its last committed timestamp across phases and serialization. Start, advance, cancel and interruption reject reversed time. Restored phase intervals, costs, cancel windows and timestamps are validated; unknown snapshot/definition fields are rejected. A snapshot without its clock is incomplete and must not be reconstructed from an guessed phase start. Zero-duration phases are skipped, and receipts name the actual reached phase. An expired phase must be advanced before interruption rather than refunded as a stale startup.

Forms require a nonempty identity and positive timed duration. Recasts require nonempty action identities, positive duration and positive use count. Invalid opening leaves previous state intact. Expired recasts are absent from summaries and cannot be consumed.

## Combat switching mechanics

Cast continuation/cancellation after switching and automatic eligible Support Skills are required **native combat mechanics**. B3 supplies shared action/field services; Q2 character adapters own the reached rules. They must work through direct Rust/native CLI APIs before Q4 integrates scripted rotations. Full WASM/Worker acceptance follows in browser integration, with bounded API/milestone smoke checks during native development under [Check selection](../../DEVELOPMENT.md). This pending content integration does not change the implemented standalone harness behavior or establish complete native gameplay support.

- **Outgoing cast:** keep field control/switch lock, actor recovery and pending effects separate. On a legal switch, apply that owner's tap/hold/channel/recast continuation, cancellation and refund rules; preserve source attribution and the correct pending hits, costs, cooldowns and live/snapshotted effects. A switch is not a universal cancellation or continuation instruction. The accepted held Sakiri Skill continuation case executes independently from tap and other-owner cases: pending hits continue off field with live Sakiri stats and owner attribution, and source defeat cancels pending hits via `hit_owner_unavailable` (`crates/content/src/simulation/tests/sakiri_actions.rs::sakiri_held_skill_continues_after_switch_and_cancels_on_source_death`).
- **Incoming Support Skill:** the engine checks the incoming owner's qualifying readiness, resources and other conditions at the admitted switch and automatically dispatches the eligible Support Skill once. Not every switch qualifies. Same-character swaps are no-ops and cannot retrigger Support; Critical Riposte is a separate owner transition. Explicit and implicit switches must use this same combat path without duplicate Support packets or costs.
  - Support is admitted from the reaction qualification captured for that switch and has no independent cooldown. Each owner uses its purchased QTE level with applicable Resonance 3 bonuses. Explicit and implicit switches route through the same engine admission. Owner recovery still gates subsequent inputs, and reached model errors remain visible in the result.

These requirements add no DSL command or continuation/automatic-Support flag. The language executor only submits ordinary inputs, exposes read-only queries such as `.support.ready`, and verifies that engine-owned results, continuing effects and receipts survive source execution unchanged. Mechanics checks belong to B3/Q2; Q4 checks integration/parity against those already implemented rules.
