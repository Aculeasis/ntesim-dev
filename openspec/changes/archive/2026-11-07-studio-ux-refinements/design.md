# Design

## Architecture

This change improves Studio usability across three key areas:
1. **Tools Navigation**: Replaces the blocking modal dialog with a non-modal dropdown menu anchored directly to the Tools button in `.editor-actions`.
2. **Card and Editor Streamlining**: Removes extraneous script location links from `CharacterCard` and the static `Initial party resources` disclosure from `EditorPage`.
3. **Report Import Capacity**: Increases the file upload size ceiling from 2 MB to 50 MB in `files.ts` to accommodate multi-iteration simulation reports containing full execution traces.

## Implementation Details

1. **Tools Dropdown (`App.tsx`, `ToolsDialog.tsx`, CSS)**:
   - In `App.tsx`, render the Tools button with a dropdown caret indicator (`Tools ▾`) and `aria-haspopup="true"`.
   - Toggle the dropdown menu directly below the button. Clicking outside the menu or pressing Escape dismisses it.
   - Retain element ID contracts (`id="tools-btn"`, `id="tools-dialog"`, `id="import-btn"`, `id="export-script"`, `id="expand-config"`, `id="language-tool"`, `id="import-input"`, and `[role=alert]`) to preserve compatibility with existing tests and browser automation.
   - Remove the modal `<Dialog>` wrapper around Tools while preserving its functionality (import, export, inspect compiled model, language reference).

2. **Clean Character Cards and Editor (`CharacterCard.tsx`, `EditorPage.tsx`)**:
   - In `CharacterCard.tsx`, remove the `<button className="text-button" ...>Source · line {a.loc.line}</button>` element.
   - In `EditorPage.tsx`, remove the `<details className="panel"><summary>Initial party resources</summary>...</details>` block.

3. **Import Limit Increase (`files.ts`, `browser_acceptance.mjs`, docs)**:
   - In `web/src/infrastructure/storage/files.ts`, update the size threshold check from `2000000` (2 MB) to `50000000` (50 MB) with error message `'Import exceeds 50 MB.'`.
   - Update `tools/browser_acceptance.mjs` test buffer from 2 MB to 50 MB and verify rejection error pattern.
   - Synchronize references in `docs/` and `openspec/specs/studio-application/reference.md`.
