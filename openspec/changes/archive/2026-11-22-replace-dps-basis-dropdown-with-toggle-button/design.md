# Design

## Context

In the simulation results view, the Damage Per Second (DPS) metric card provides a control to switch between relative and absolute time basis for report summaries. This control was implemented as a `<select>` element with `<option>` tags.

## Goals / Non-Goals

**Goals:**
- Replace the `<select>` dropdown with a compact `<button>` displaying "Relative" or "Absolute" based on the current `timeBasis`.
- Toggle between `'relative'` and `'absolute'` upon clicking the button.
- Retain the exact same accessibility contract (`aria-label="Damage per second time basis"`).
- Maintain compact height, font size, and spacing consistent with the surrounding rollup card heading.

**Non-Goals:**
- Altering the time basis logic, underlying statistics, or downstream consumers.
- Adding captions, tooltips, or extra helper text to the button.

## Decisions

1. **Toggle Button Interaction**:
   The button renders the current active basis name capitalized: `{timeBasis === 'relative' ? 'Relative' : 'Absolute'}`. Clicking it calls `onTimeBasisChange?.(timeBasis === 'relative' ? 'absolute' : 'relative')`.
   This allows a user to toggle the mode in a single click rather than opening a dropdown menu.

2. **Styling in `.rollup-heading`**:
   Add `.rollup-heading button` styling with `font-size: 11px`, `padding: 2px 8px`, `line-height: 1.4`, `border-radius: 4px`, and `flex-shrink: 0`. This preserves the compact footprint and prevents awkward squashing or wrapping.

## Risks / Trade-offs

- None: The change is purely a presentation refinement for the DPS card time basis control.
