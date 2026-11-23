# Timing, clocks, pause/dilation, and Hotori contract

This reference separates the interval-based timing research service from live execution. Both use the checked primitives in `ntesim-core::kernel::timing`; `RuntimeModeClock` adds incremental, generation-owned pause lifecycles. The selected gameplay policy is `paused-world-v1`, under the `relative-combat-v1` clock contract. These identities describe a bounded model, not verified native countdown execution.

Live execution retains one absolute causal scheduler. [DSL](../../NTE_DSL.md) `duration`, `.duration` under `stop=time`, `.time` and whole-run DPS use relative combat time. Wait seconds, authored scenario timestamps, receipts and playback positions use absolute time. Both are simulated elapsed time, independent of measured host execution time. Reports retain `relative_duration_us`, `absolute_limit_us`, `relative_elapsed_us` and `absolute_elapsed_us`. See [execution integration](../../../openspec/specs/script-execution/reference.md#pending-live-input-and-scenario-integration).

For absolute elapsed time A and the union of executed timer pauses P, R = A - P: **relative + union pause = absolute**. The engine observes actual commitment, release, cancellation and defeat; it does not add a speculative full Ultimate duration to the horizon. Twenty paused seconds in a completed 120-relative-second run yield 140 absolute seconds. A pause clipped by failure or defeat contributes only its executed prefix. Caller work/event limits and the absolute ceiling remain independent.

Every supported Ultimate has a content-owned cinematic binding. A retrieved `Common.EndAbilitySkill` marker is preferred; an unavailable release, including Zero's unserialized component, uses the consumed owner action-completion fallback. Hotori hands setup to active Time Stop without a timer gap and releases at the actual finisher, expiry or defeat. Bindings retain provenance separately from the shared policy assumption.

## 1. Multi-clock hierarchy

Combat simulation maintains distinct, causal clock domains to avoid conflating elapsed animation time with challenge deadlines:

- **Real clock (`real_time_us`)**:
  - Monotonically increasing simulated real combat time in microseconds, independent of host runtime.
  - Never pauses, dilates, or rewinds. Serves as the global causal anchor for all events and inputs.
- **Game mode clock (`game_mode_horizon_us`)**:
  - The challenge countdown timer (e.g. 120-second domain/challenge time limit).
  - Ticks at 1.0x during active combat.
  - Pauses during supported Ultimate cinematics and Hotori's actual active Time Stop. Other camera/menu systems are outside the executable model.
- **Actor clocks (`actor_clocks_at_horizon_us`)**:
  - Per-character local clocks that govern action animation playback and phase deadlines.
  - Advance at 1.0x (1,000,000 PPM) under normal conditions.
  - Dilate (e.g. freeze at 0 PPM or slow down) during hitlag windows caused by weapon impact resistance or external time slows.
- **World clock**: ordinary cooldowns, proc ICDs, buffs/shields/debuffs, status and periodic phases, passive recovery and recording/refund windows retain exact remaining world time during both pause kinds. Local contact hitlag alone does not pause this clock.
- **Object lifetime clock**: deployed objects pause existence and activity in cinematics. Their existence advances in active Hotori Time Stop while ordinary autonomous activity pauses. Living Hotori B2 exempts Vita Bud attacks from active Time Stop, consuming normal lifetime and actual volley counts; another cinematic still dominates.
- **Action clocks**: only the current cinematic generation and permitted finite Hotori Katana/direct-replay contexts progress through their applicable pause. This is not permanent same-owner immunity. Source removal, defeat and replacement remain immediate.
- **Incoming dummy schedules** preserve their remaining delay through applicable pauses. Targets are stationary script-authored dummies; this declares no enemy AI or geometry.

World pause, object existence and finite action exemptions are selected policies. Conflicting Scorch reports motivate the conservative no-periodic-ticks/no-catch-up rule; valid immediate reactions remain separate. Continuing object lifetime during active Time Stop is a lower-confidence assumption. Charge grants no energy or delayed credit during active Time Stop; unrelated supported instantaneous grants retain their own rules.

Summary rates (DPS, EPS, granted EPS, HPS) and available shield HP (SHP) pair relative and absolute bases calculated per trial from settled totals over that trial's actual positive duration $T_b$:
- $\text{Rate}_b = \text{Total} / T_b$
- $\text{SHP}_{\text{absolute}} = \frac{\int C(a)\,da}{\text{absolute\_elapsed}}$
- $\text{SHP}_{\text{relative}} = \frac{\int C(a)\,dR(a)}{\text{relative\_elapsed}}$

Relative SHP strictly distinguishes clock weighting from denominator scaling: during a mode pause interval, $dR(a) = 0$, so shield capacity active only during paused windows accumulates zero relative HP-microseconds. Dividing the absolute capacity integral by relative duration is invalid and rejected. Field time and the absolute damage timeline panel retain absolute time denominators. Canonical team rankings, comparison eligibility and research evaluation remain strictly canonical relative DPS.

## 2. Mode pause interval union

When supported pause owners overlap, their intervals must not be added cumulatively:

- The engine computes the **disjoint union** of overlapping intervals $[[S_1, E_1], [S_2, E_2], \dots]$ via `merge_intervals`.
- For example, a pause from $[1.0\text{s}, 4.0\text{s}]$ and an overlapping pause from $[2.0\text{s}, 5.0\text{s}]$ merge into a single continuous paused interval $[1.0\text{s}, 5.0\text{s}]$ (total 4.0s paused, not 6.0s).
- Total paused time $P(t)$ up to real time $t$ is the sum of overlaps with the merged intervals.
- The game mode time $M(t)$ is strictly:
  \[
  M(t) = t - P(t)
  \]
- `mode_to_real_time(M)` provides the earliest inverse for a known research interval set. The live clock projects a future relative deadline only while running; during an open pause it returns no finite projection and the scheduler advances to actual release work or the independent absolute ceiling.

Independent arithmetic at absolute 7s: `[1,4)` contributes 3s, `[2,5)` contributes 3s and their intersection contributes 2s. The union is 3 + 3 - 2 = 4s, so relative time is 7 - 4 = 3s. Ending one owner at 4s cannot resume the clock while the other remains active until 5s. Duplicate or stale release is idempotent; observation clips open pauses without naturally ending them.

## 3. Hitlag and time dilation

Attacks connecting with targets generate hitlag on the attacker, the target, or both:

- `HitlagWindow { actor_index, start_real_us, duration_real_us, dilated_rate_ppm }`.
- `dilated_rate_ppm = 0`: full animation freeze (0.0x playback speed).
- `dilated_rate_ppm = 500_000`: 50% slow-motion playback.
- During hitlag windows, the actor's local clock advances at $(dt \times \text{rate\_ppm}) / 1\,000\,000$.

Overlapping windows use the minimum active PPM rate (strongest slowdown), independent of insertion order. This is an explicit probe policy, not evidence of NTE overlap behavior. The integral retains fractional PPM across all segments before quantization. The inverse returns the earliest integer real timestamp reaching the requested actor offset; it never floors the inverse into a premature deadline. Rates above normal speed, reversed observation intervals and unrepresentable durations/deadlines reject explicitly. Pause insertion is checked before mutation, and merged intervals are read-only.

### Action deadline resolution

`HitlagCurveWindow` adds typed piecewise-linear rate knots to the same actor
clock. Curves retain constant endpoint rates and clip at their declared real
duration. The integrator splits at knot/window boundaries and exact crossings
between overlapping rates, then integrates the strongest slowdown as a rational
area. It retains fractional progress until final observation. Deadline inversion
plans the effective linear pieces once and solves the reached piece with bounded
integer bisection; it schedules no polling or per-microsecond events. Existing
constant-window clocks retain their original path.

Independent checks include a clipped four-knot window with1637.5us progress over
130000us (128362.5us lost), two crossing ramps with2.5us total progress over10us,
noninteger crossing12/7, constant-freeze composition, fractional carry, actor
isolation, insertion-order independence and checked `u64` boundary deadlines.
Fadia's ordinary clock consumes these curves and tests interruption within an
unfinished ramp; replacing the curve by one flat freeze gives a different next
hit and is not the selected model.

An action starting at $t_0$ specifies:

- `startup_us`: time to hit delivery.
- `hit_offsets_us`: hit connection offsets.
- `recovery_us`: endlag duration after the active phase.
- `earliest_switch_offset_us`: earliest point where character switch is permitted.
- `total_duration_us`: $t_{\text{startup}} + t_{\text{recovery}}$.

`resolve_action_deadlines` converts actor local offsets into real-time timestamps by integrating through active hitlag windows:

- Hits occurring before hitlag deliver at their base offset.
- Recovery and switch windows occurring after hitlag are delayed in real time by the freeze duration.
- Maps all resolved deadlines to both real time and game mode time.

## 4. Hotori clock and cooldown branch model

The owner-specific resource scalars, cooldown branch and Time Stop evidence are in [Hotori's card](../characters/hotori/time-stop-resource-and-cooldown.md); unanswered progress/pause rules remain in its matching unknown card and [shared action timing](action-timing.md). This timing service evaluates the explicitly configured branch; it does not establish full native Time Stop execution.

## 5. Separate real-time and game-timer receipts

The standalone timing research service retains its existing probe receipt format. It accepts explicitly authored intervals rather than deriving gameplay cinematic bindings. Its separate receipts are:

- `real_time_report`:
  - `horizon_us`: total wall-clock microseconds.
  - `horizon_s`: exact rational seconds (`Ratio { numerator, denominator }`).
- `game_mode_report`:
  - `horizon_us`: total challenge countdown microseconds elapsed.
  - `horizon_s`: exact rational challenge seconds.
- These research outputs do not provide an absolute-duration execution option or a second live DPS metric. Gameplay DPS uses actual relative elapsed time; zero relative duration yields an unavailable rate even when combat completed successfully.

Sample reports intern native `timer_catalog` definitions (owner or shared account, key and clock basis). Existing state frames carry changed `timers` snapshots with deadlines in that basis. Cooldowns, observed proc guards and the stamina recovery gate use world coordinates; deployed fields use object-lifetime coordinates. Seeking computes exact remaining time from the declared coordinate, so an absolute cursor delta cannot consume a frozen remainder. Buff definitions likewise retain world basis, and shields expose their declared basis with cached absolute projections at the observation boundary. These are observations on existing state boundaries, never a second timer event stream or scheduler polling. Compact batches omit this catalog and keep the two elapsed scalars.

## 6. CLI execution and independent verification

The native CLI exposes:
```bash
ntesim timing-stats_DIR REQUEST.json
```
Produces a structured JSON receipt with real horizon, mode horizon, merged pause intervals, actor local clocks, resolved action hit/switch/recovery deadlines, and Hotori cooldown probe results.

### Verification oracle
Independent verification is enforced by `tools/verify_timing_cli.py`:

- Independently merges overlapping pause intervals in Python.
- Simulates piecewise actor hitlag dilation and action deadline resolution.
- Verifies Hotori 0s real player vs 35s clone cooldown.
- Rejects 12 invalid configurations (zero team size, team size > 4, zero horizon, unsupported package, out of bounds hitlag/action actor, unresolved Hotori player identity, corrupt mode pauses, invalid hitlag rates, missing action fields, unauthorized top-level field).
- Validates byte-identical replay and relocation invariance.
- Checks frozen package file SHA-256 hashes and caches proof in `target/verification/timing-state.json`.

## Consumed Ultimate release bindings

The retrieved ability pause components cover all 23 supported owners. The table separates concrete notify offsets from consumed owner-completion fallbacks. Offsets describe montage progress; the runtime composes finite action progression and local delay before releasing an actual pause. Continuing effect lifetime never supplies a cinematic endpoint.

| Owner | Selected endpoint |
| --- | --- |
| [Adler](../characters/adler/action-damage.md#relative-combat-timer-binding) | `Adler_UltralSkill` Common.EndAbilitySkill at 3.025935 montage seconds |
| [Aurelia](../characters/aurelia/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Baicang](../characters/baicang/action-damage.md#relative-combat-timer-binding) | `Cang_UltralSkill` Common.EndAbilitySkill at 2.854959 montage seconds |
| [Blackbird](../characters/blackbird/action-damage.md#relative-combat-timer-binding) | Evolved: `Blackbird_Evolve_UltraSkill` Common.EndAbilitySkill at 5.021767 montage seconds; ordinary transformation: owner action-completion fallback |
| [Chaos](../characters/chaos/action-damage.md#relative-combat-timer-binding) | `char_Chaos_skill_ultraskill_Montage` Common.EndAbilitySkill at 4.802493 montage seconds |
| [Chiz](../characters/chiz/action-damage.md#relative-combat-timer-binding) | `char_rabbit_ultraskill_1_Montage` Common.EndAbilitySkill at 4.250937 montage seconds |
| [Daffodil](../characters/daffodil/action-damage.md#relative-combat-timer-binding) | `Daffodill_UltraSkill` Common.EndAbilitySkill at 5.099006 montage seconds |
| [Edgar](../characters/edgar/action-damage.md#relative-combat-timer-binding) | `Edgar_UltraSkill` Common.EndAbilitySkill at 3.402224 montage seconds |
| [Fadia](../characters/fadia/action-damage.md#relative-combat-timer-binding) | `Fadia_UltraSkill` Common.EndAbilitySkill at 4.417366 montage seconds |
| [Haniel](../characters/haniel/action-damage.md#relative-combat-timer-binding) | `char_Haniel_skill_ultraskill_Montage` Common.EndAbilitySkill at 4.237614 montage seconds |
| [Hathor](../characters/hathor/action-damage.md#relative-combat-timer-binding) | `Hathor_UltraSkill` Common.EndAbilitySkill at 4.518597 montage seconds |
| [Hotori](../characters/hotori/action-damage.md#relative-combat-timer-binding) | World's Tide commitment through setup, EnterTimeStop and actual finisher, expiry or defeat |
| [Iroi](../characters/iroi/action-damage.md#relative-combat-timer-binding) | `Oneiroi_UltraSkill` Common.EndAbilitySkill at 4.004747 montage seconds |
| [Jiuyuan](../characters/jiuyuan/action-damage.md#relative-combat-timer-binding) | `Kuhara_UltraSkill` Common.EndAbilitySkill at 5.599441 montage seconds |
| [Lacrimosa](../characters/lacrimosa/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Linko](../characters/linko/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Mint](../characters/mint/action-damage.md#relative-combat-timer-binding) | `Mint_UltraSkill` Common.EndAbilitySkill at 3.903489 montage seconds |
| [Nanally](../characters/nanally/action-damage.md#relative-combat-timer-binding) | `Nanally_UltralSkill` Common.EndAbilitySkill at 3.584608 montage seconds |
| [Sakiri](../characters/sakiri/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Shinku](../characters/shinku/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Skia](../characters/skia/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Zankou](../characters/zankou/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
| [Zero](../characters/zero/action-damage.md#relative-combat-timer-binding) | Consumed owner Ultimate action-completion boundary |
