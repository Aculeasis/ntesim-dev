# Tasks

## 1. Engine Reaction Counters

- [x] 1.1 Define `ReactionKind` indexing in `crates/content/src/simulation/reactions/` and add zero-allocation `reaction_counts: [u32; 8]` to `ReactionRuntime`.
- [x] 1.2 Wire counter increments for qualified switch admissions (Blossom, Hexed, Scorch, Nova, Stain, Remora), character kit refreshes (Baicang B2 Scorch refresh, Blackbird A6 Nova reapplication), and ternary triggers (Charge energy grant, Discord gauge loss).
- [x] 1.3 Expose `reaction_counts` in `GameScenarioReport` and verify correct increment counts in `crates/content/src/simulation/tests/reactions.rs`.

## 2. Research Observation & Aggregation

- [x] 2.1 Update `Observation` in `crates/research/src/trials/studio/` to record per-trial reaction counts from `GameScenarioReport`.
- [x] 2.2 Implement available reaction discovery from `request.actors` elements in `crates/research/src/trials/studio/mod.rs` (pairs for binary reactions, trios for Charge and Discord).
- [x] 2.3 Compute `Statistics` (`mean`, `min`, `max`, `std`, percentiles) for every available reaction across completed trials (preserving 0 counts for untriggered reactions) and sort descending by mean in `Aggregate.reactions`.
- [x] 2.4 Verify multi-trial reaction aggregation and empty results for mono-element squads in `crates/research/src/trials/studio/tests.rs`.

## 3. Character Reaction Passives Audit & Implementation

- [x] 3.1 Implement Zero (MC) Charge healing passive (`Reaction_GuangLingXiang1_Recover_1051`, 50% base ATK heal on Charge grant) in `crates/content/src/characters/zero/` and verify with a dedicated test.
- [x] 3.2 Audit and implement any remaining unmodeled character reaction passives (Jiuyuan Blossom bud limit, Sakiri Scorch extension, Shinku Reaction AoE) in `crates/content/src/registry/dispatch.rs` and verify each with tests.

## 4. Studio UI & Responsive Presentation

- [x] 4.1 Update TypeScript contracts in `web/src/domain/contracts/trials.ts` to include `ReactionAggregate` in `TrialAggregate`.
- [x] 4.2 Update `web/src/features/results/EnergyPanels.tsx` to render the `Esper Cycles` panel using `BarChart` with error bars and descending sort when reactions exist, and completely hide it (rendering 2 columns) when 0 reactions are available.
- [x] 4.3 Add responsive styling for `.three-column` grid layout in `web/src/styles/` ensuring clean collapse to a single column on narrow viewports.

## 5. Verification & Delivery

- [x] 5.1 Run focused Rust tests with `python tools/cargo.py test -p ntesim-content -p ntesim-research`.
- [x] 5.2 Run web build and test suite with `node tools/web.mjs test`.
- [x] 5.3 Run `python tools/checkpoint.py run` to ensure all checks pass.
