# Tasks

## 1. Report presentation

- [x] 1.1 Recompose the report with compact summaries, full-width timelines, contribution pies, paired actions/field time and full-width grouped energy; verify layout against the saved baseline and reference in desktop/mobile screenshots.
- [x] 1.2 Render native timeline statistics with distinct series, readable time axes and pointer/keyboard values; verify quartiles, variability and zero/empty handling in the existing Studio rendering test.
- [x] 1.3 Preserve metric/source/recipient identity in ranked bars and grouped energy accounting; extend existing Studio coverage for source-kind grouping, recipient slots, empty pies and native-value rendering, then run focused Studio/report checks.

## 2. Integration and delivery

- [x] 2.1 Exercise real browser filters, energy modes, chart focus, partial/stopped state and responsive layout at 1440 and 390 pixels; save screenshots and a bounded receipt under cache/report-redesign.
- [x] 2.2 Review the stable diff, validate OpenSpec and run python tools/checkpoint.py run with affected checks passing.

## Workflow follow-up

- Synchronize the delta and archive the completed change.
- Run strict archived validation and checkpoint on the final tree, then commit the coherent change.
