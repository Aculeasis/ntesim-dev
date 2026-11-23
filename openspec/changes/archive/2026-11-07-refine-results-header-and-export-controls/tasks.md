# Tasks

## 1. Studio Header and Results Controls

- [x] 1.1 In `web/src/app/App.tsx`, wrap the heading/intro and action bar in `.view-header`, and render a standard `Export report` button in place of `▷ Run` when `view === 'results'`. Verify via web tests.
- [x] 1.2 In `web/src/features/results/ResultsPage.tsx`, remove the redundant `results-toolbar` row and verify module remains under 350 lines.
- [x] 1.3 In `web/src/styles/studio-report.css`, style `.view-header`, `.view-header-title`, and `.editor-actions:not([data-view='config'])` to sit inline without empty vertical gaps.

## 2. Verification and Delivery

- [x] 2.1 Run web test suite and architecture checks, confirming module limits and report controls behave as expected.
- [x] 2.2 Run `python tools/checkpoint.py run` and confirm all receipts pass.

## Workflow follow-up

- Archive the change after checks pass and sync deltas to the main spec.
