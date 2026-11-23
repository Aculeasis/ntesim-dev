# Tasks

## 1. Component and Markup Adjustments

- [x] 1.1 In `web/src/components/ui/Panel.tsx`, make `title` optional and omit `panel-heading` when both title and controls are absent. Verify via web tests.
- [x] 1.2 In `web/src/features/results/Targets.tsx`, group target name and stats on a single `.target-header-line`. Verify markup structure.
- [x] 1.3 In `web/src/features/results/ResultsPage.tsx`, omit the target panel title when `request.targets.length >= 3`. Verify via render test.
- [x] 1.4 In `web/src/styles/studio-report.css` and `studio-targets.css`, add `margin-bottom` to `.rollup-grid` and single-line flex styling for `.target-header-line`.

## 2. Verification and Delivery

- [x] 2.1 Run web test suite and confirm target layout and panel heading behaviors pass.
- [x] 2.2 Run `python tools/checkpoint.py run` and confirm all receipts pass.

## Workflow follow-up

- Archive the change after checks pass and sync deltas to the main spec.
