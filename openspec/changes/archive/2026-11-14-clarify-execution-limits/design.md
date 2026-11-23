# Design

## Context

See [proposal](proposal.md) for the reproduced failure. `CompileLimits` supplies the shared native/WASM per-trial work budget. The live evaluator counts expression nodes and control/action operations cumulatively. Runtime event and victory diagnostics already flow unchanged through Worker errors and failed reports.

## Goals / Non-Goals

Provide readable numeric causes at the Rust owner so all existing consumers receive the same explanation. Preserve admission, deterministic replay, typed codes and partial reports. No new settings UI, JSON fields, budget replenishment or changes to damage and event limits.

## Decisions

- Set the existing default operation budget to 60,000; preserve explicit overrides and the existing upper bound. Increasing the ceiling is the requested bounded change; resetting work after combat advancement would change safety semantics.
- Format operation and event messages where those limits are enforced. Distinguish expression nesting depth from operation exhaustion because they currently share a misleading message. Include numeric live input/action ceilings in their existing diagnostic paths as well.
- Share the victory time-limit message between event advancement and source advancement. Finalization without a reached horizon must report source/effect exhaustion, including the configured horizon for context. Time-stop successful completion remains successful.
- Extend current evaluator, live-loop, source-clock, source-export/replay and event-limit tests. Native CLI/WASM smoke verify transport of the existing diagnostic fields; Worker and UI need no schema or string parsing changes.

## Risks / Trade-offs

- Runaway scripts can execute three times as many operations before stopping. Independent command, event and combat-time ceilings remain enforced; memory is not preallocated for the additional budget.
- Changed human-readable messages affect replay equality and assertions. Update current consumers and replay checks together; engine identities already bind the implementation.
