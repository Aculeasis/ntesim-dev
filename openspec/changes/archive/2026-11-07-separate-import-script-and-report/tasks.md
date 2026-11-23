# Tasks

## 1. Separate script and report import

- [x] 1.1 Implement `importReportFile` in `files.ts` to parse exported reports and reconstruct `TrialResult`.
- [x] 1.2 Add `loadReport` method in `useSimulation.ts` to set imported results and run status.
- [x] 1.3 Add `Import report` option and input to `ToolsDialog.tsx` alongside `Import script`.
- [x] 1.4 Handle report import in `App.tsx`, updating both script draft and simulation results and navigating to Results.

## 2. Verification and delivery

- [x] 2.1 Add unit tests for `importReportFile` and `importFile` in `studio.test.tsx` and verify web tests pass.
- [x] 2.2 Synchronize specification reference and delta, validate OpenSpec, run checkpoint, and archive change.
