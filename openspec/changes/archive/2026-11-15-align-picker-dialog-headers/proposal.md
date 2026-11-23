# Proposal

## Why

Team list search is visibly above the other header controls. Reference pickers waste a separate row on a title and text Close button.

## What Changes

- Center Team list search vertically with its neighboring controls.
- Put search and an icon close button in one row across all Name Search categories, removing their visible category heading.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: Compact, aligned picker headers with accessible dialog names.

## Impact

React dialog presentation and CSS only. Browser layout verification covers desktop and narrow screens; Rust, native, WASM and Worker contracts are unchanged.
