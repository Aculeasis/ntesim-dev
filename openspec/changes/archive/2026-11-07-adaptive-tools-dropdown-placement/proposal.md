# Proposal

## Why

When the simulator controls panel is near the bottom of the viewport, the Tools dropdown opens downwards past the lower boundary of the screen, clipping its menu items. The dropdown must dynamically choose its placement (above or below the button) based on available viewport space to remain completely visible.

## What Changes

- Measure available viewport space above and below the Tools button on mount, window resize, and scroll.
- Position the dropdown above the button (`bottom: calc(100% + 4px)`) when remaining space below is insufficient and space above is greater; otherwise open downwards.
- Ensure the dropdown stays bounded to the viewport with scrolling if vertical space is constrained.
- Animate the button caret indicator to reflect open state and placement.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: adaptive placement for Tools dropdown menu preventing viewport overflow.

## Impact

Dropdown component (`ToolsDialog.tsx`), run controls stylesheet (`studio-run-controls.css`), and specification scenario.
