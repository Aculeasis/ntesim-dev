# Tasks

## 1. Rust field-time accounting

- [x] 1.1 Record exact absolute and relative field time per actor in full and compact reports. Extend the existing Studio collector tests with independently calculated results for two characters, a timer pause and a switch, and verify both sets of times reconcile with the trial elapsed totals.
- [x] 1.2 Carry both durations through compact observations and expose both actor statistics. Extend detailed/compact parity and zero-relative coverage to check aggregation over mixed seeds and rejection of malformed field-time pairs at the observation boundary. Run `python tools/cargo.py test -p ntesim-research --locked trials::studio`.

## 2. Studio controls and portable reports

- [x] 2.1 Connect both buttons to the existing run selection and render the selected field-time seconds and shares. Extend the existing report controls coverage for both bases and the zero-total chart. Update the Studio report reference to describe the shared buttons, native seconds and share calculation.
- [x] 2.2 Update the TypeScript aggregate contract and portable report validation for both required statistics. Extend report import coverage to restore both distributions and reject a previous absolute-only field without replacing the open report.
- [x] 2.3 Update the existing browser report-export acceptance check to exercise switching from each button by pointer and keyboard at desktop and 390px widths. Verify that both controls, DPS and field time follow the choice, that Sample navigation retains it, and that switches issue no simulation or replay request. Verify that the image reflects the choice and imported data defaults to Relative with both bases available.

## 3. Integration checks

- [x] 3.1 Run the focused Web tests for report controls, portable report import and Worker streaming against the current WASM build. Run the existing browser acceptance boundary through `node tools/web.mjs verify-trials --browser` for the updated report-export scenario.
- [x] 3.2 Review the stable diff, validate the OpenSpec change with `python tools/openspec.py validate sync-field-time-basis --strict` and run `python tools/checkpoint.py run` before delivery.

## Workflow follow-up

- Archive the change after implementation and applicable checks pass.
