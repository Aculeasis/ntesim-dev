# Tasks

## 1. Native support observations and contracts

- [x] 1.1 Change HPS to canonical gross healing in the existing Rust trial collector; extend its existing Iroi coverage with full-health overheal and actual-duration assertions, retaining once-only autonomous accounting and gross/applied/excess totals. Verify the focused native support tests pass.
- [x] 1.2 Implement checked integration of active-actor remaining shield HP from native delta frames and exact expiry deadlines. Extend the existing support-test boundary with independently calculated 5000 HP half-uptime and 3000 HP idle-expiry examples, actor switches, additive/replaced shields, depletion/defeat, equal-time boundaries, early completion, zero activity and invalid observations. Verify focused native tests pass without changing combat settlement.
- [x] 1.3 Replace `sps` with `shp` in Rust samples and aggregates, retaining `shield_absorbed` as the separate recursively reconciled total. Verify native sample/aggregate statistical consistency, explicit immunity exclusion, and a positive Adler SHP result with no incoming events using the existing source fixture boundary.
- [x] 1.4 Update `openspec/specs/studio-application/reference.md` during implementation with formulas, active-actor/additive NTE interpretation, units and current contract names; verify it agrees with Rust observations and this delta and removes obsolete effective-HPS/shield-rate descriptions.

## 2. Studio presentation and transport

- [x] 2.1 Update current TS trial contracts and every histogram, result selector, sample/export consumer and fixture from `sps` to `shp`; change HPS/SHP card labels, units and explanatory totals as designed. Extend existing `web/tests/report_controls.test.mjs` coverage to verify gross HPS, available SHP and matching histogram values; verify web types/format and focused report-control checks pass.
- [x] 2.2 Extend existing actual WASM/Worker trial coverage, including `web/tests/streaming_trials.test.mjs`, to verify positive healing/shields without incoming damage, zero support, exported field identity and unchanged delivered-prefix resume verification. Verify Rust/native and WASM results agree and no TS support-mechanics calculation or legacy `sps` field remains.
- [x] 2.3 Inspect desktop and 390 pixel reports using actual Iroi/Adler runs, with screenshots under `cache/`; verify six readable cards, nonzero HPS/SHP where support is produced without incoming attacks, correct HP/s versus HP labels and no horizontal overflow.

## 3. Integrated acceptance

- [x] 3.1 Run `python tools/openspec.py validate align-support-summary-semantics --strict --no-interactive`, review the complete stable diff for contract/reference consistency, and run `python tools/checkpoint.py run` after focused checks; resolve failures and verify all selected integration checks pass before implementation delivery.

## Workflow follow-up

- After implementation tasks and checks pass, synchronize and archive with `python tools/openspec.py archive align-support-summary-semantics --yes`.
- Validate archived specs with `python tools/openspec.py validate --archived --strict --no-interactive --concurrency 4`, run the final checkpoint on the stable archived tree and commit the coherent change. Do not publish or deploy.
