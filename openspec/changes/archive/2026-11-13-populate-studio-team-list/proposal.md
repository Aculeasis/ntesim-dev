# Proposal

## Why

The new Team list still contains partial research scenarios rather than a useful selection of complete teams. Populate it with about twenty researched, equipped four-character compositions and tested continuing rotations, while retaining S5 as an unchanged control.

## What Changes

- Author twenty new complete team scripts as the working target, spanning several supported damage dealers and Cycle strategies; retain S5 in addition. Select teams for credible synergy and measured performance, not merely different labels or equipment on the same roster.
- Include Nanally / Hotori / Zero / Jiuyuan and Zankou / Iroi / Sakiri / Daffodill. In the latter, Daffodill has exactly three selected awakenings, `[3,4,5]`, not the prefix `[1,2,3]`.
- Exclude Elyms, Akane Rin and their signature weapons from all new builds and candidates, even when current internet guides recommend them.
- Research team roles, equipment and rotations online, starting with the supplied [Reference guide team tier list](https://neverness.app), and reconcile advice with the current engine and mechanics references. Allow different builds for a character in different teams; avoid redundant non-stacking support sets such as two Speedy Hedgehog wearers in a new team.
- Restore the historical comparison profile: 15 useful substat lines per character across the whole console, character and weapon level 80, and purchased talents 8/8/8. Baseline awakenings are A0 for limited S, A1 for standard S and Zero, and A6 for A-rank characters and Chiz; select the actual nodes explicitly. Use 1,000 iterations over 120 real combat seconds, with role-specific allocation of that fixed stat budget. Keep T1 as the default weapon refinement and document justified small exceptions, including accessible non-limited A1-A2/T1-T3 options and the mandatory Daffodill `[3,4,5]` build. S5 remains unchanged. Never silently inflate investment to win a ranking.
- Test initial scripts, inspect resource/admission and damage receipts, compare sensible rotation/build alternatives, and retain reproducible evidence for the final choices. Fix encountered simulator defects at their owning boundary, then return to validating the affected teams.
- Replace selectable S1, S2, S3 and Baicang research entries only after the new collection passes its checks. Preserve S5's exact source, identity and default selection, and keep required non-selectable regression/language fixtures outside the Team list.
- Regenerate source and Team list assets through the existing shared-engine pipeline and update catalog-dependent checks and documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: A curated collection of complete, equipped and validated bundled teams, mandatory roster/build constraints, documented run conditions, and replacement of partial selectable scenarios while preserving the S5 control.
- `runtime-content`: Preserve authored flat Cycle and Break Intensity through shared build projection, restore Zero's missing Skill Cycle fill and Lacrimosa's frozen connected-packet Cycle accrual, required for actual reaction routing.

## Impact

- Authored scripts and manifest in `presets/studio/`; generated source and Team list bundles in `web/src/generated/`; current catalog/browser tests and Studio preset documentation.
- Existing generation in `tools/studio_presets.mjs` and `tools/studio_teams.mts`, Rust/WASM source trials, inspection and report exports. Extend tooling only where necessary for the concrete collection workflow; no new simulator or public optimization API is planned.
- A concise curated-team rationale/reference near the scripts, with source links and limitations. Raw web acquisitions, trial outputs and comparison receipts remain in `cache/`.
- Targeted Rust content/kernel fixes and corresponding mechanics/spec corrections only if concrete script validation exposes a defect. An unsupported reached dependency must remain explicit until resolved; it cannot be bypassed to obtain a successful score.

This change builds on the implemented, still-unarchived `add-studio-team-list` change and preserves its unrelated working-tree edits. It does not redesign the Team list dialog, implement console packing, execute game code, publish a site, or claim global optimality or game-measured DPS.
