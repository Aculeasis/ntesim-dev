# Proposal

## Why

`duration=120` currently limits elapsed absolute combat time, including Ultimate cinematics and Hotori's extended Time Stop, so it cannot express a 120-second challenge timer. Players need the timer-based limit and DPS while retaining the real spacing of actions during stopped time and an inspectable absolute duration.

## What Changes

- **BREAKING:** Make authored `duration`, live `.time` and the primary DPS denominator use relative combat time: absolute elapsed time minus the union of actual mode-timer pauses. Keep event/scenario timestamps and `wait(seconds)` on absolute elapsed combat time; project action delays from their declared progressing or paused clock domains onto that absolute scheduler.
- Integrate a runtime mode-timer clock for **all supported characters' Ultimates**, with explicit content-owned cinematic start/release lifecycles. Hotori additionally keeps the timer paused through actual Time Stop, including early finisher, expiry, interruption and death. Keep timer pauses distinct from enemy, actor and effect clock policies; do not equate the full duration of every Ultimate's buffs or summons with its cinematic pause.
- **BREAKING:** Pause ordinary cooldowns, internal cooldowns, timed buffs/debuffs, DoT phase/expiry, passive recovery and recording/refund windows during cinematics and Hotori Time Stop. Preserve exact remaining time and fractional progress, including timers created during a pause. Declare the selected defaults and exceptions rather than treating all friendly systems as absolute-time systems.
- Keep the active cinematic action, Hotori's Katana/resource drain and explicitly replayed direct actions progressing on their action clocks. Select separate lifespan/activity policies for deployed fields and Vita Buds; B2 permits bud attacks during Hotori's active Time Stop while consuming their normal lifetime/count, without granting immunity to ordinary cinematic pauses. Freeze periodic Scorch by default even when applied during Time Stop; retain immediate reactions from valid applications separately from periodic ticks.
- Record linked source observations, conflicting reports, confidence limits and the chosen rule for each unresolved timer class in the design, then transfer durable evidence and rationale into the owning mechanics references during implementation. Missing measurements covered by these disclosed defaults do not block a complete bounded-model run or become verified native facts.
- Retain relative and absolute elapsed durations from one execution. Normalize team, actor, source and target DPS consistently, including native experiments, comparisons, per-seed statistics and exact exports. Keep zero-relative-duration rates explicitly unavailable.
- Add an upper-right `Relative` / `Absolute` selector to **Sim Duration (Dur)**, defaulting to Relative and switching that card's complete statistical summary without changing the executed run or other metrics.
- Preserve an absolute Sample axis, playback speed, event order and seek cursor; show both clocks and shade recorded timer-pause spans. Keep aggregate damage timelines on an explicitly absolute axis that includes damage beyond the relative limit's numeric value.
- Store compact pause intervals and both elapsed totals without duplicating event streams or adding per-frame timer samples. Preserve compact batch observations and regenerate selected detailed samples through the existing path.
- Keep independent absolute execution, event and work limits, readable partial-failure diagnostics and existing report-file limits. Measure added clock bookkeeping separately from the extra combat performed to reach a relative limit.

## Capabilities

### New Capabilities

None; the existing execution, reporting and Studio boundaries own this behavior.

### Modified Capabilities

- `script-execution`: Relative duration and `.time`, absolute waits, paused live cooldown/modifier reads and relative time-limit completion.
- `combat-execution`: Dual-duration reports, mode/world/exception clock policies, exact pause/resume conservation, relative DPS and compact observations.
- `runtime-content`: Sakiri's twenty-second buff carriers use the shared unpaused world-timer policy instead of unconditional absolute lifetime.
- `characters/zankou`: Ordinary cooldowns, form/modifier lifetimes and DoTs observe global time stops while preserving the distinct local contact-pause behavior.
- `execution-safety`: Independent absolute safety limits while the relative clock can pause, with explicit stopping causes.
- `studio-application`: Duration-card selector, dual-clock Sample and timer inspection, absolute timeline extent, relative Team list calculation contracts and faithful report restoration.
- `wasm-transport`: Native/WASM/Worker agreement on the revised clocks, pauses, statistics, replay and resume contracts.

## Impact

- Rust core timing primitives; content scheduler/source advancement, stop policy, live observations, timed effects/resources/cooldowns, report validation, every supported Ultimate's cinematic lifecycle and Hotori's extended Time Stop; research damage analysis, Studio aggregation and source experiment/comparison consumers.
- Native request/report and source export/replay contracts, WASM sample/batch endpoints, Worker streaming/resume and report import/export validation. Update current contracts together without legacy-input shims.
- Studio summary cards, aggregate damage charts and Sample controls/lanes/tooltips. EPS, HPS, SHP and field-time accounting keep their existing absolute-time definitions and expose their time basis clearly.
- Update code **and** affected accepted requirements, capability references, `docs/NTE_DSL.md`, shared clock/buff/reaction/energy references and character timing cards. Remove or rewrite contradictory statements about uninterrupted real-time cooldown/buff/DoT progression; preserve historical evidence as history without forwarding stubs. Include Team list calculation identity and documentation examples. Existing timing research and retrieved pause components do not by themselves establish native execution; selected fallbacks remain explicit hypotheses.
- Extend existing Rust, transport, browser and report tests; retain bounded performance and serialization receipts under `cache/`. No new dependency, deployment, enemy catalog or simulation of enemy AI is required.
