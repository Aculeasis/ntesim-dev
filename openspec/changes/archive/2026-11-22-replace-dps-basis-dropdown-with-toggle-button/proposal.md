# Proposal

## Why

The Relative / Absolute time basis switcher in the Studio DPS card is currently rendered as an HTML `<select>` dropdown. A compact toggle button displaying the active mode ("Relative" / "Absolute") that switches state on click provides a more immediate, single-click interaction while preserving the compact card header footprint.

## What Changes

- Replace the `<select>` dropdown in `web/src/features/results/Summary.tsx` with a compact `<button>` displaying the current time basis ("Relative" or "Absolute").
- Clicking the button toggles between `'relative'` and `'absolute'` time bases via `onTimeBasisChange`.
- Add compact styling for `.rollup-heading button` in `web/src/styles/studio-report.css` to match the compact dimensions and visual rhythm of the card header.
- Update web component tests in `web/tests/report_controls.test.mjs` to assert the toggle button rendering and ensure no `<select>` dropdown remains.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
<!-- None: visual and interaction refinement consistent with existing studio-application spec requirements -->

## Impact

- `web/src/features/results/Summary.tsx`: Use a toggle `<button>` instead of `<select>`.
- `web/src/styles/studio-report.css`: Style `.rollup-heading button` compactly.
- `web/tests/report_controls.test.mjs`: Update assertions for the button and absence of select.
