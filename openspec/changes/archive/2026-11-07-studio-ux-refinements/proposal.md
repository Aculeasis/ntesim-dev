# Proposal

## Why

Studio currently presents Tools as a full-screen modal dialog, which disrupts the workflow compared to standard web dropdown menus. In addition, team cards display redundant script line pointers (`Source · line X`), the editor includes an obsolete `Initial party resources` disclosure panel, and importing large simulation reports fails because of a rigid 2 MB file size limit that rejects multi-iteration reports (e.g., 17 MB files).

## What Changes

- Replace the Tools modal dialog with a compact dropdown menu anchored beneath the Tools button, adding a standard chevron/arrow indicator (`▾`) to the button.
- Remove `Source · line X` location buttons from character cards.
- Remove the `Initial party resources` disclosure block from the editor.
- Increase the text script and JSON report import size limit from 2 MB to 50 MB to allow loading realistic simulation reports.
- Update tests, browser acceptance checks, and documentation references to reflect the 50 MB limit and dropdown interaction.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: streamlined Tools dropdown menu, clean team card presentation, and increased file import size limit.

## Impact

React editor and dialog components (`App.tsx`, `ToolsDialog.tsx`, `CharacterCard.tsx`, `EditorPage.tsx`), file storage utility (`files.ts`), browser acceptance test (`browser_acceptance.mjs`), and detailed reference documentation. Native Rust engine and Worker transport contracts are unchanged.
