# Proposal

## Why

The simulation report uses oversized summary cards and narrow side-by-side time charts, making it harder to read the rotation and its contributions than the supplied gcsim reference. Studio needs a compact statistical report adapted to Neverness to Everness observations.

## What Changes

- Arrange compact metadata and six statistical cards above target information and the outcome histogram.
- Give damage and cumulative timelines separate full-width panels with readable axes and distinct statistical series.
- Keep three contribution distributions together, followed by ranked character and source bars, paired actions/field time, and full-width energy panels.
- Use consistent character colors, compact chart styling, keyboard inspection and a responsive single-column mobile layout.
- Retain native metric meanings, completed-trial statistics, execution identity and failure/partial status; show character-element grouping explicitly. Do not invent aura, reaction, healing or shield aggregates or target geometry.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: readable statistical report composition and accessible native-observation charts across desktop and mobile.

## Impact

React results components, existing chart components, report styles and focused Studio/browser coverage. Rust, native CLI, WASM/Worker contracts and simulation mechanics remain unchanged. gcsim is studied as a visual/layout reference only; no upstream code or dependencies are imported.
