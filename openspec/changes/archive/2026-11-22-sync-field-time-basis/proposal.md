# Proposal

## Why

Field Time Distribution currently uses absolute on-field time even when the report shows relative DPS. A second copy of the time-basis button lets users switch the report from the field-time card without scrolling back to DPS.

## What Changes

- Show each character's share and seconds on field on the selected time basis. Relative time excludes intervals during which the combat timer is paused. Absolute time includes them.
- Add the same `Relative / Absolute` button to Field Time Distribution. Both buttons reflect and update the existing report selection, which defaults to Relative for each new or imported run.
- Deliver both native field-time summaries for completed trials so switching and restoring a report can use recorded results immediately.
- **BREAKING** Portable reports that contain only the previous absolute field-time statistic will require regeneration.

## Capabilities

### New Capabilities

### Modified Capabilities

- `studio-application` gains synchronized time-basis buttons, dual-basis field-time rendering and restoration of both field-time summaries.

## Impact

Rust trial observation and aggregation, compact WASM/Worker observations, the Studio report contract, portable report validation and the Results controls are affected. Canonical comparison eligibility and the absolute damage timeline retain their existing time bases.
