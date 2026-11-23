# Proposal

## Why

The Tools menu currently combines script and report loading into a single action (`Import script / report with source`). When users load a simulation report, only the script source is extracted and placed in the editor, while the executed simulation metrics, aggregates, and samples are lost. Users need separate actions to either load a script into the editor or load a complete simulation report and immediately view its results.

## What Changes

- Split `Import script / report with source` into two separate Tools menu options: `Import script` and `Import report`.
- `Import script` imports `.txt`, `.nte`, or `.json` files containing source and updates the editor draft.
- `Import report` imports a `.json` report export, reconstructs the `TrialResult` and source, populates simulation results in `useSimulation`, and automatically navigates to the Results tab.
- Add test coverage for both script and full report import flows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: dedicated script import and full simulation report import actions with automatic Results navigation.

## Impact

File utilities (`files.ts`), simulation state hook (`useSimulation.ts`), Tools menu component (`ToolsDialog.tsx`), main application view routing (`App.tsx`), and web unit tests. Native Rust engine and Worker protocols remain unchanged.
