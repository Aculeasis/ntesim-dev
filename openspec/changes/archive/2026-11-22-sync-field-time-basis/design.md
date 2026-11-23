# Design

## Context

The Rust Studio collector currently obtains absolute field time by summing gaps between switch receipts and adding the final interval. Later seeds use compact execution reports that omit detailed pause spans. The runtime already knows the active actor and both clock deltas for every time advance. Results already keeps the DPS selection by run ID.

## Goals / Non-Goals

Use one Rust accounting path for full and compact trials and one Results selection for both buttons. The scope covers on-field duration, trial aggregation, transport, report restoration and presentation. Combat admission, clock rules, damage attribution and Sample playback are outside this change.

## Decisions

1. **Accumulate integer microseconds in the runtime.** During each time advance, credit the absolute and relative deltas to the current active actor. Reuse the existing relative clock delta to account for overlapping pauses and ownership changes at their actual boundaries. Credit the active slot using the current field-time ownership rules, including idle time and the final interval. Switches at the same timestamp accrue zero between them. Expose the per-actor pairs in both full and compact native reports. This avoids carrying detailed pause spans in compact reports and avoids assigning paused time to the wrong character by scaling absolute durations with the run-wide elapsed ratio.
2. **Validate and aggregate the two durations in Rust.** Carry both non-null durations per actor through compact observations and check actor count, clock ordering and the sums against the trial's elapsed clocks. Use integer microseconds through the observation boundary. At aggregation, convert to seconds and summarize each basis over the same completed seeds. A zero duration is a valid observation and participates in the mean. The updated Studio actor aggregate exposes `field.relative` and `field.absolute` as required statistics. Aggregate DPS availability does not determine field-time availability.
3. **Reuse the existing Results state and pie chart.** Supply both card buttons from the run's existing selection and use one button presentation with card-specific accessible names. Read the selected mean seconds for each actor and use the existing chart's share calculation, legend, seconds tooltips and empty state for a zero total. Live Worker chunks, resumed runs and imported reports deliver the same aggregate contract. Button clicks only update presentation state.

## Risks / Trade-offs

- Incorrect switch or pause attribution can preserve an elapsed total while distorting individual shares. Exercise a two-character paused trial with independently derived values and compare detailed and compact observations.
- A valid aggregate can have all-zero relative field time while Absolute remains usable. Exercise zero-relative seeds separately from ordinary rate availability and keep them in mixed-seed means.
- An existing browser report-export check still locates a duration dropdown. Update that assertion to use the current DPS button and the new Field Time Distribution button at both desktop and 390-pixel widths.

## Migration Plan

Update native report producers, WASM/Worker consumers, the Studio aggregate type and portable report validation together. Validate the presence of both field-time bases before replacing the displayed run. A saved report with the former absolute-only field needs to be regenerated from its source script.
