# Tasks

## 1. Reproducible team projection and DPS generation

- [x] 1.1 Extend the existing preset tooling and typed bundle contract with Rust-inspected ordered roster/build fields, effective calculation conditions, source/engine identity and ready/unavailable DPS status; verify all five selectable manifest entries are represented, `s5` stays default and `language-reference` stays non-selectable.
- [x] 1.2 Add explicit generation through the existing Node/WASM trial-session boundary, preserving exact scripts, authored iterations and consecutive seeds, and consuming the native mean DPS; generate the five retained entries, retaining raw outcomes in `cache/`, and verify a generated numeric mean against a fresh same-source/same-engine session without implementing independent mechanics or statistics in JS.
- [x] 1.3 Add cache reuse and build/check freshness enforcement for source, generator and current engine identity, including detection of stale WASM before generation; extend the existing web test boundary with temporary inputs for source/engine invalidation, unavailable outcomes, valid zero and deterministic cache-free regeneration, and verify a build rejects a deliberately stale projection.
- [x] 1.4 Document generation, regeneration after engine changes, cache usage and heterogeneous scenario conditions in `docs/DEVELOPMENT.md` and `docs/WEB_ARCHITECTURE.md`; verify the documented commands produce current assets from a checkout without calculation cache and obey the four-CPU wrapper.

## 2. Dialog, compact cards and scenario loading

- [x] 2.1 Add Team list beside the actual view tabs, compose its bounded modal through the existing dialog boundary, render four compact ordered slots with Arc/set icons and DPS/status, and remove Load configuration and unused toolbar consumers; extend `web/tests/studio.test.tsx` for roster presentation, short rosters, awakening/equipment distinctions and unavailable status, then run `node tools/web.mjs test web/tests/studio.test.tsx`.
- [x] 2.2 Wire row activation to existing preset loading and Simulator navigation without auto-run; disable selection during simulation/sample execution and retain prior executed-source identity. Extend the existing browser acceptance flow to verify exact source loading, unchanged results on selection, busy protection and dismissal without changing the draft.
- [x] 2.3 Update `openspec/specs/studio-application/reference.md` to describe the replacement selector, compact catalog presentation, precomputed status and preserved draft/results semantics; verify references and removal of obsolete Load configuration instructions.

## 3. Search, roster pickers and responsive interaction

- [x] 3.1 Implement session-retained query/Include/Exclude state, case-insensitive all-term name search, all-Include/no-Exclude membership and descending unrounded DPS sorting with stable ID ties; extend existing Studio presentation tests for combined filters, conflicts, empty controls/results, clear-one-filter behavior and unavailable-last ordering, then rerun the focused Studio test.
- [x] 3.2 Add independent portrait pickers with counts, toggles, selected states, Clear All and picker-first Escape dismissal; extend `tools/browser_acceptance.mjs` for pointer/keyboard toggling, outside dismissal, reopen persistence, exact filter outcomes and focus restoration, and document those behaviors in the Studio reference.
- [x] 3.3 Finish persistent-header scrolling, contained picker placement and desktop/390-pixel layouts; verify actual browser screenshots and keyboard operation show readable compact cards, accessible equipment inspection, reachable DPS and close controls, no nested interactive controls and no page-level horizontal overflow.

## 4. Integration and stable-tree checks

- [x] 4.1 Run the affected web checks (`node tools/web.mjs check` and focused tests), then actual browser acceptance with `node tools/web.mjs verify-trials --browser` under `python tools/check.py --`; retain the browser receipt/screenshots in `cache/` and verify filtering/browsing creates no simulation work. Reuse valid existing Worker parity coverage rather than adding a second mechanics suite.
- [x] 4.2 Validate the change with `python tools/openspec.py validate add-studio-team-list --strict --no-interactive`, review the stable diff and run `python tools/checkpoint.py run`; resolve failures with focused checks and confirm the final receipt covers current code, generated assets, documentation and tests before delivery.

## 5. Review corrections

- [x] 5.1 Keep the three view tabs left aligned and center Team list independently, with browser geometry checks and a narrow layout without overlap.
- [x] 5.2 Remove row Details and exclude S4 from the manifest and generated catalog; update dependent replay/filter checks, documentation and specification expectations, then verify the five retained scenarios.
- [x] 5.3 Regenerate the catalog, run focused web and browser checks, review the stable diff and run python tools/checkpoint.py run before returning the updated preview for review.

## Workflow follow-up

- After implementation and applicable checks pass, sync and archive this change using the pinned OpenSpec wrapper, then validate the final archived tree and refresh its checkpoint before committing. Publication and deployment require separate explicit authorization.
