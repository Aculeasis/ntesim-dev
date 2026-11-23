# Event and clock kernel

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

The scheduler uses integer microseconds as a technical representation. This does not claim measurements have microsecond accuracy. Every clock has an explicit nonnegative rate expressed in millionths; rate zero pauses future local deadlines. Fractional local phase is retained across advances and rate changes. No FPS, hitstop duration, clock binding or animation multiplier is inferred from asset coordinates.

Each clock owns a deadline heap. Global monotonically assigned event IDs order events at equal real timestamps, including different clocks and local deadlines that round to the same real tick. A delivered event can schedule an immediate causal child; earlier queued events keep their order. Future, canceled and unknown events cannot be cited as delivered causes.

`pop_before(horizon)` advances all clocks to the earliest eligible event, or to the explicit horizon if none is due. Backward horizons fail. Cancellation removes pending eligibility without emitting an event. Rates can change while tasks retain their local deadlines. Checked arithmetic rejects overflow before partial time advancement. There is no ambient system time, sleep or IO in the kernel.

`clock_readings()` returns all clock rates, integer local readings and retained fractional phases without mutation. Fraction denominators are 1,000,000; overflow fails instead of truncating. Synthetic [state observations](../state-observations/reference.md) bind these readings to current actor/resource/target/RNG state in a causal effect trace.

`schedule_batch_after` enqueues a same-clock/same-deadline batch with contiguous IDs after preflighting its clock, delivered cause, deadline and ID capacity. Any error leaves IDs, tasks and clocks intact; even empty batches validate their references. Synthetic actor invocation uses this batch to create causal leaf effects atomically with admission/costs. Checks cover parity with individual scheduling at a fractional clock phase and atomic errors.

Generated property checks compare delivery against independently sorted real deadlines. Focused examples cover fractional phase, pauses, cancellation, immediate causal children, same-tick rounding and atomic errors. These validate scheduler semantics only. Game-specific mode/actor/effect clock bindings and death cancellation rules must come from separate reviewed mechanics.

The initial reference implementation scans pending tasks to resolve same-real-tick buckets after clock changes. Performance optimization can retain these semantics once benchmarks justify it. Game-specific clock policies belong to their native content owners and mechanics references. Exact arithmetic is described in [the formula contract](../../../docs/mechanics/shared/exact-scaling-arithmetic.md); the synthetic text timeline compiler is described in [the rotation contract](../synthetic-rotations/reference.md).

# Resource and health settlement

Pools store exact integer quanta with explicit capacity and initial value. The caller supplies the unit/quantization rule; no NTE rounding rule is asserted. Grants report both applied and excess amounts. Resource spending rejects shortages atomically. Capacity reductions return the amount removed instead of silently losing it.

Health settlement receives damage after a separately validated formula. Shields have explicit source IDs and a caller-selected absorption order. Oldest/newest policies are technical options for tests, not confirmed NTE stacking rules. Damage reports per-shield absorption, HP loss, overkill and death separately. Healing reports applied/excess amounts and cannot implicitly revive a dead recipient. Revival and event cancellation on death need independent policies and are not implemented by this primitive.

State tests cover conservation/resource bounds, overflow, explicit shield ordering and independent targets across continuous synthetic cycles. This primitive does not bind game-specific shields, healing multipliers or death policies; supported content adapters such as Edgar healing supply their own checked models. Apply the [DPS relevance policy](../product-scope/reference.md#dps-relevance-and-modeling-boundary) to further integration.

Local-clock recovery/cooldown/resource admission uses a separate [action contract](../../../docs/mechanics/shared/action-admission.md). Its multi-pool spending is atomic; it does not authorize unknown game action execution or imply a whole multi-effect action transaction.
