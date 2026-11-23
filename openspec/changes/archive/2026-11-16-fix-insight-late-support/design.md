# Design

## Context

See proposal.md. Insight's forty-five-second reserve predates relative clocks. The runner also treats an action recovery deadline or ordinary wait beyond stop=time as incomplete, forcing scripts to avoid natural partial final windows.

The gcsim architecture reference separates the action loop from the simulation stop: skipUntilCanQueue and advanceFrames both consult nextFrame/stopCheck, which can end an action or wait successfully at duration. Reviewed upstream source: https://github.com/genshinsim/gcsim/blob/main/pkg/simulation/run.go (architecture only; no copied code or frame-based implementation).

## Goals / Non-Goals

Keep the absolute event scheduler, relative countdown, lawful owner admission and uninterrupted form windows. A successful time-limited run need not finish its final command or rotation. Failed admissions, explicit unresolved energy gates, victory policy and safety budgets keep their distinct diagnostics. S5 remains the historical control.

## Decisions

The shared native stop treats ordinary pending actions/effects at the relative endpoint as expected truncation rather than missing execution. Remove the ordinary-wait-overrun rejection. Keep energy-gate diagnostics and absolute/event ceilings. The source runner checks the completed relative timer before advancing its interpreter, when leaving an implicit switch, and inside action-window advancement. It must not admit another command or evaluate an invalid suffix at the completed boundary.

All twenty curated rotations repeat normally, with no duration lookahead or special carry-only terminal tail. Keep .time-based support cadence, form conditions, Black Tome's World-clock guard, equipment and earned resources. The existing source-baseline boundary executes the actual presets, with an Insight late-support regression; the source-clock and native live-program checks cover clipping, suffix suppression and replay. Existing conditional test assertions use explicit arithmetic failure instead of overlong waits, since ordinary wait clipping is now successful. Shared-engine integration verifies native/WASM/Worker results.

## Risks / Trade-offs

Additional iterations can expose cooldown or form-admission defects hidden by old tails. Validate every edited composition and the authored Insight seed range. Damage after the endpoint stays pending and contributes zero; equal-time causal damage retains the existing endpoint policy. Retained scores are historical until an explicit catalog refresh; no optimality claim is made.
