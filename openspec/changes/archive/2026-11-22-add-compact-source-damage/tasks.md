# Tasks

## 1. Source chart rows and controls

- [x] 1.1 Build compact rows from the delivered source means and keep the detailed view as the default. Extend the existing report controls coverage to check both DPS bases and damage instance counts, owner separation, effect-free ability labels, the shared `Reactions` label, and sums that include effects outside the detailed chart's visible slice.
- [x] 1.2 Add the title-adjacent button, run-scoped mode selection, and folding based on the displayed row count. Verify keyboard toggling and a 390-pixel viewport, retention across Type, Character, DPS basis and result chunk changes, the default on a new or imported run, and restoration of the detailed rows when switching back. Update the Studio capability reference with the new source chart mode and its mean-only display.

## 2. Integration checks

- [x] 2.1 Run `node tools/web.mjs test tests/report_controls.test.mjs tests/studio.test.tsx tests/source_labels.test.ts` and `python tools/openspec.py validate add-compact-source-damage --strict --no-interactive`. Review the diff and run `python tools/checkpoint.py run` before delivery.
