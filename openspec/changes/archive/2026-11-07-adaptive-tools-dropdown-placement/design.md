# Design

## Architecture

The `ToolsDropdown` component detects its bounding rectangle relative to the browser viewport (`window.innerHeight`). If the space below the button is less than the required dropdown height plus a margin, and the space above is greater than the space below, it assigns `data-placement="top"`. CSS styles position the menu above or below the anchor accordingly.

## Implementation Details

1. **Placement Measurement (`ToolsDialog.tsx`)**:
   - In `ToolsDropdown`, track `placement: 'bottom' | 'top'`.
   - On mount, window resize, and scroll, measure `rect = parent.getBoundingClientRect()` and `dropdownHeight = container.offsetHeight || 190`.
   - If `window.innerHeight - rect.bottom < dropdownHeight + 8 && rect.top > (window.innerHeight - rect.bottom)`, set `placement = 'top'`; else `'bottom'`.
   - Render `data-placement={placement}` on the `#tools-dialog` element.

2. **Styling (`studio-run-controls.css`)**:
   - Default: `top: calc(100% + 4px); bottom: auto;`.
   - `[data-placement="top"]`: `top: auto; bottom: calc(100% + 4px);`.
   - Add `max-height: calc(100vh - 32px); overflow-y: auto;` to prevent off-screen overflow in extreme height constraints.
   - Smooth caret rotation on `#tools-btn[aria-expanded="true"] .dropdown-caret`.
