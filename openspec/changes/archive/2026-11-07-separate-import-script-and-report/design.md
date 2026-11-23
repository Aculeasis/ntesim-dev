# Design

## Architecture

This change provides two distinct import paths in Studio:
1. **Script Import**: Parses plain scripts (`.nte`, `.txt`) or source-containing JSON, normalizes the script syntax using the WASM engine, and updates the editor configuration state.
2. **Report Import**: Parses complete exported JSON reports containing executed source and simulation results (`aggregate`, `samples`, `report`, etc.), reconstructs the canonical `TrialResult` model, updates both editor source and simulation state, and switches the active tab to Results.

## Implementation Details

1. **Storage and Parsing Utilities (`files.ts`)**:
   - Retain `importFile` for script loading (`.nte`, `.txt`, `.json`).
   - Implement `importReportFile(file)` returning `{ source: string; result: TrialResult; failure?: string | null; cancelled?: boolean }`.
   - Validate that report files are `.json`, contain a source script, and include report aggregates. Reconstruct any missing derived fields (`party`, `compiled`, `request`) via the WASM engine.
   - Enforce the 50 MB file limit across both import functions.

2. **Simulation State (`useSimulation.ts`)**:
   - Add `loadReport(source: string, result: TrialResult, failure?: string | null, failedTrial?: FailedSourceTrial | null)` to `useSimulation`.
   - Set status to `'completed'` (or `'failed'` / `'stopped'`), progress to 100%, and update stage description with completed iteration counts.

3. **Tools Menu and App Integration (`ToolsDialog.tsx`, `App.tsx`)**:
   - In `ToolsDialog.tsx`, provide two distinct buttons: `Import script` (`#import-btn`) and `Import report` (`#import-report-btn`).
   - Wire hidden file inputs (`#import-input` for script, `#import-report-input` for report).
   - In `App.tsx`, upon successful report import, call `config.setSource(source)`, `run.loadReport(source, result, ...)`, and switch `setView('results')`.
   - Upon script import, update `config.setSource(source)` and ensure view is `'config'`.
