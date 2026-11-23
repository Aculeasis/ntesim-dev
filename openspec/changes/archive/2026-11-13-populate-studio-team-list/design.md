# Design

## Context

See [proposal.md](proposal.md) for motivation and [the Studio delta](specs/studio-application/spec.md) for the collection contract.

`presets/studio/index.json` currently selects S1, S2, S3, S5 and Baicang, with S5 as the default and `language-reference` non-selectable. Several Rust, CLI and WASM tests directly read S1-S4 files; removing a menu entry does not authorize deleting those fixtures. `tools/studio_presets.mjs` bundles exact sources; `tools/studio_teams.mts` inspects builds and computes mean DPS using the existing Rust/WASM trial session, one script at a time. Its cache includes source, engine, WASM and generator identities. Failed runs already project unavailable DPS.

The Team list implementation and its completed OpenSpec change remain uncommitted. Its pending `Team list entry and scenario selection` requirement hardcodes five entries, while the accepted main specification does not yet contain that requirement. This is a known specification dependency, not a reason to overwrite the existing implementation or silently introduce conflicting accepted requirements.

Console geometry and roll legality are outside scope. [Authored equipment stats](../../../../docs/mechanics/shared/equipment-builds.md) distinguish console additions from automatically applied character, weapon and set stats. [Build legality](../../../../docs/mechanics/shared/build-legality.md) explicitly limits claims about obtainable allocations. The user confirmed a comparable console-stat budget allocated by role, then requested reuse of the original comparison profile recovered from Git: 15 useful substat lines per character, level-80 characters/weapons, skill level 8 and rarity/acquisition-specific awakening investment.

The DSL supports explicit awakening subsets: `awakening=3` means `[1,2,3]`; the required Daffodill build is `awakening=[3,4,5]`. A compact card's A3 label counts selected nodes and must not obscure their actual identity in the script/reference.

## Goals / Non-Goals

**Goals:** Use existing source scripts, native execution and catalog generation to deliver a defensible team collection. Make research resumable through the ordinary change tasks and authored scripts, while keeping raw evidence out of tracked assets. Measure full continuing rotations with consistent conditions and disclose differences in investment.

**Non-Goals:** No automatic optimizer, new benchmark service, console-layout generator, enemy database, mechanics implementation in TypeScript, rework of dialog interactions, or publication. A guide's tier ranking is a research lead, not simulator validation or proof of optimality.

## Decisions

### 1. Curate twenty new roster compositions, then retain S5 separately

Use twenty new teams plus S5 as the working target. Count distinct canonical four-character rosters, not reorderings, equipment variants or duplicate scripts. Favor several credible archetypes and damage dealers over filling slots with weak permutations. A small justified count adjustment is compatible with “approximately twenty”; never fill the collection with unsupported or empty placeholders.

The initial research directions below organize selection, not a second progress ledger. Only the two required rosters are fixed. Other candidates may change after sourced build research and execution.

| Direction | Grounded starting point | Further selection question |
| --- | --- | --- |
| Blossom | Nanally / Hotori / Zero / Jiuyuan, mandatory | Compare accessible replacements such as Mint and support variants without losing the desired Cycle chain. |
| Hexed / follow-up | Nanally / Jiuyuan / Sakiri / Adler from the Nanally guide | Evaluate Linko/Iroi variants and their actual follow-up recipients and field-time needs. |
| Zankou mixed Scorch/Hexed | Zankou / Iroi / Sakiri / Daffodill, mandatory | Separate direct/DoT versus Break investment and selected awakening assumptions. |
| Break | Zankou / Daffodill with Adler and Sakiri or Baicang | Check whether selected A5 effects, BI and the Great Thief are necessary and worth their investment; do not assume them at A0. |
| Quickswap / Nova | Lacrimosa / Haniel / Sakiri / Daffodill | Validate supported swap boundaries, stored attacks and Phantom Step routes without fabricated parries. |
| Off-field Scorch | Lacrimosa / Zankou / Sakiri / Haniel | Measure persistent DoTs against the competing on-field rotation. |
| Baicang hyper | Baicang / Adler / Sakiri / Daffodill | Validate Power Words, Burst and charged-action routes beyond the current partial domain preset. |
| Charge | Shinku / Hathor / Zero / Nanally | Verify actual continuing energy income and complete enhanced-form windows. |
| Remora / Stain | Chaos / Zero / Hathor / Haniel | Compare a Charge alternative using available Blossom enablers. |
| Other current tier-list leads | Blackbird Nova/Stain, Chiz, Aurelia and Mint | Resolve available exact rosters and builds; exclude Elyms/Akane Rin and their weapons before choosing variants. |

Research starting references, inspected on 2026-11-13:

- [Reference guide team tier list](https://neverness.app), page updated 2026-11-11: discover current archetypes, then inspect actual team guides rather than importing tier labels as scores.
- [Nanally](https://neverness.app) and [Hotori](https://neverness.app): the required Blossom core has complementary Bud and replay mechanics. Start by testing replay of Zero's Cycle-enabling Skill, teammate setup and a real Nanally field window. Hotori's Lost Radiance is a promising alternative when Zero holds Speedy Hedgehog.
- [Zankou](https://neverness.app): a published rotation covers the exact mandatory roster. Use Scorch setup, Daffodill's Insight, Sakiri/Iroi buffs, then Zankou's form/DoT window as a starting hypothesis. The final rotation must follow actual engine resource and recovery state.
- [Daffodill](https://neverness.app): selected Insight/Break awakenings make BI materially different from a conventional crit build. Evaluate a BI-focused `[3,4,5]` build rather than copying A0 damage stats.
- [Lacrimosa](https://neverness.app), [Baicang](https://neverness.app), [Shinku](https://neverness.app) and [Chaos](https://neverness.app) contain concrete starting rosters/rotations for the corresponding rows above.

These are attributed guide recommendations, not verified primary mechanics. For reached mechanics, consult current owner code/tests and `docs/mechanics/`; corroborate disputed values with source evidence. Record guide dates, links, interpretation and model limits in a concise `presets/studio/teams.md` reference during implementation. Do not duplicate whole guides. Store raw acquisitions in `cache/team-curation/`.

Alternative: generate twenty permutations automatically. Rejected because roster count alone does not establish synergy or useful diversity.

### 2. Establish the shared profile before tuning damage

Reuse the comparison conditions in the [historical calculation contract at commit c1fcb411](https://github.com/Aculeasis/ntesim-dev/blob/c1fcb41119b963dec26cf6aac68e6acce1f35281/docs/ACCEPTANCE_DELIVERABLES_CONTRACT.md#L30). Its explicit 15-line/skill-8 profile also survives in the [Mint build context](../../../../docs/mechanics/characters/mint/build-candidate-context.md). That context preserves one candidate, not proof that its allocation or resulting totals fit every character. The historical document was later removed with the obsolete research pipeline; recover its comparison assumptions without reinstating console geometry, old acceptance quotas or fixed caller-delay rules.

For new final scripts, use the following baseline rather than selecting a new investment budget:

| Input | Collection baseline |
| --- | --- |
| Final trials | `iteration=1000 duration=120 seed=42 mode=research crit=random`; 120 real combat seconds |
| Character and weapon progression | Level 80/80 |
| Purchased talents | `talent=8,8,8`; effective levels still include applicable resonance bonuses |
| Useful substats | Exactly 15 useful lines per character across the entire console, allocated by role |
| Limited S awakenings | A0 |
| Standard S and Zero awakenings | A1 |
| A-rank characters and Chiz awakenings | A6 |
| Weapon refinement | T1 by default; this is the collection's retained refinement choice, not an extra claim from the recovered historical section |

Awakening labels count selected effects: choose and record the actual node indices instead of assuming that A1 means node 1. Distinguish rarity from limited acquisition, and preserve all-six selections for baseline A6. The mandatory Daffodill `[3,4,5]` selection is an explicit exception to the standard S baseline. The user permits small justified deviations, particularly accessible non-limited A1-A2 or T1-T3 choices; record the exact change and rationale rather than automatically adding investment. Do not lower the A6 baseline for A-rank characters/Chiz to A1-A2 or mutate S5 to match this profile.

Use a common level-80 stationary dummy with DEF 1080 and base resistance 0.2 for new baseline entries. Start at full HP, with current/max HP high enough that all final seeds retain the complete 120-second measurement. Choose and document that HP once before comparisons, then keep it identical; change it for the whole affected comparison group if necessary. Break-cap, boss eligibility, initial resources and special conditions must also be explicit and shared where relevant. Specialized multi-target or other-condition variants must be identified as such and not treated as controlled baseline comparisons. Full initial ordinary energy follows the current script default and is disclosed; owner-specific special resources use their actual declared defaults, not invented full meters.

The budget is already fixed at 15 useful substat lines per character; implementation must recover and document the source-backed per-line values and relevant item categories/tiers used to translate those lines into numeric stats, rather than inventing a new budget or taking arbitrary endgame totals from guides. Start from the historical profile's preserved source/receipt evidence and corroborate uncertain values before using them. Keep the same line-value convention across the collection, with role-specific allocation toward crit/ATK, CI, BI or support needs. Document line counts, exact per-line values, non-substat console contributions and final additions in `teams.md`, so each build can be checked against the fixed budget. Count neither console main-stat/specialization contributions nor automatic character/weapon/set bonuses as useful substat lines; keep their contributions separate and avoid double-counting. Console additions remain authored through `add stats`, without packing or allocation code in the engine. Existing S1/S5 totals are partial fixtures, not universal stat targets. Do not claim that an authored comparison budget certifies obtainable rolls; disclose any justified exception explicitly.

Designate one Speedy Hedgehog wearer per new team and check that their actual Burst cadence supports the buff. Review other shared non-stacking buffs and equipment trigger requirements individually. Repeated personal-damage sets can be appropriate; a blanket unique-set rule is unjustified. S5 retains its existing duplicate support sets as part of the control.

Alternative: copy “recommended endgame stats” into every build. Rejected because those totals include different Arcs, investment and automatic bonuses.

### 3. Validate the real source and improve through bounded comparisons

Draft complete `.nte` sources and use existing native `compile-script`/`run-script` inspection and the WASM source-trial APIs already consumed by `calculateTeam`. Small seed batches can diagnose admission and timing quickly; they never substitute for final 1,000-trial/120-second results. Keep the opener, repeatable phase and horizon tail distinct. Use supported live state/resource queries and explicit waits; never reset state between rounds or schedule illegal inputs just because a guide lists a combo.

Inspect per-owner damage, Cycle/Break ownership, buff windows, energy income/spend/overflow, waits, failed admissions and final elapsed combat time. In particular, guide descriptions of game-mode timers or Time Stop must not change the simulator's real combat-time denominator. Do not assume incoming attacks, successful parries, target defeat or geometry in a stationary dummy profile. Use separate disclosed scenarios only when those assumptions are specifically justified.

For each retained roster, compare a plausible initial script against a meaningful alternative suggested by the receipts: action ordering/cancellation, support refresh cadence, energy investment, damage-zone allocation, or equipment. Use identical target/profile and seed sequence for a rotation comparison and disclose changed investment for build comparisons. Compare native aggregate means and per-seed outcomes where available; report modest or noisy differences honestly. Once a sustainable candidate is chosen, validate its exact final source for all 1,000 seeds at 120 seconds.

Raw evidence in `cache/team-curation/` identifies exact source hashes, engine/WASM identity, conditions, seeds, completion counts, diagnostics and before/after metrics. Keep durable build/rotation rationale, final assumptions and the reference budget near the scripts; do not add a separate readiness registry or hand-maintained DPS catalog. Reuse current generation caching for final catalog values.

If a reached path exposes a bug, minimize it, distinguish script misuse from incorrect or missing engine behavior, fix the Rust owner, extend its existing meaningful regression boundary, and rerun affected candidates. Update relevant mechanics and delta specs when behavior changes. A broad unrelated feature campaign is outside scope; unresolved failures remain explicit and cannot count toward the successful collection. Mandatory teams remain required even if their validation needs an owner correction.

Alternative: optimize only an opening segment or select the highest partial run. Rejected because resources, form uptime and damage rankings change over a continuing two-minute fight.

### 4. Preserve the source manifest and replace the catalog only after validation

Keep `.nte` sources plus `index.json` as the authoring boundary. After final candidates pass, expose the selected twenty, remove S1/S2/S3/Baicang from selection, retain S5 and the language reference, and generate both JSON bundles. Preserve original regression files still read by native/Worker checks; they need not be selectable manifest entries. Do not delete S4 or unrelated current edits.

The inspected S5 source SHA-256 is `be3c86075fd56ff430909abdaf398c824cdfde96535bd0d9fadd18d712e9dd8a`; verify its bytes and default identity before and after replacement. Engine fixes legitimately invalidate its cached DPS and require recomputation without source edits.

Update `presets/studio/README.md`, the generation section of `docs/DEVELOPMENT.md` and `openspec/specs/studio-application/reference.md`, which currently describe the five-entry collection and mixed run conditions. Keep model limits and any scenario-specific exceptions visible in the reference. Do not add per-row report controls or change source/result identity.

Extend existing `web/tests/team_catalog.test.mjs` to validate collection invariants through actual Rust inspection/completion metadata, including the required node subset, baseline level/talents/awakening selections or documented exceptions, excluded content, final conditions and absence of duplicate Speedy Hedgehog. Check each authored 15-line allocation and its numeric translation against the documented source-backed values as part of build review; runtime inspection alone cannot reconstruct a line count from summed `add stats`. Avoid executing every 1,000-trial team again in routine UI tests; final generation supplies full-run evidence. Replace five-entry and positional-first-S1 assumptions in `team-list-cases.tsx`, replay/browser checks and catalog parity with appropriate collection assertions or explicit small fixtures. Preserve independent partial-roster rendering coverage using a local test fixture rather than retaining partial menu entries.

Rust owns any combat corrections and their mechanics tests. Native CLI runs can diagnose scripts; the current WASM build supplies the catalog values; existing Worker/browser checks verify transport, exact source loading, filters and scrolling for the larger collection. The required flat ci/bi script additions now bind existing native Cycle/Break Intensity fields through shared build projection; the previous Cycle projection hardcoded zero. This adds source/inspection vocabulary without changing the report wire shape. Existing Rust numeric tests and native/WASM smoke cover exact summed values and independent stat zones. Rebuild WASM after an engine fix, regenerate every invalidated projection and use the existing CPU-limited sequential pipeline.

Alternative: hardcode actors and DPS for the new teams. Rejected because it duplicates the source and loses engine identity and stale-data protection.

Two owner omissions found by causal trial inspection are corrected before scoring: Zero's admitted Skill now fills its Cycle pool once, and Lacrimosa's connected packets consume their frozen `HeterochromeAdd` values through the existing packet-accrual policy. Native numeric/causal tests own both corrections; typed Lacrimosa values follow the frozen source rather than the older sampled mechanics table. All downstream scoring is invalidated after either engine correction.

Sample playback additionally validates each selectable team's actual source execution. Lacrimosa's existing Broken-target Shatter callback must carry `lacrimosa.broken_shatter_v1` provenance on both its content-owned semantic tag and owner-child binding; its selected damage, timing and trigger remain unchanged. Nanally's removal emits `utility_ability` with an ordinary admission cost receipt, so the energy validator applies the same ownership, occurrence, timing and pool reconciliation checks as for `invoke_ability`. Break caused by persistent Support damage retains its initiating switch even when the Support has no damaging cast occurrence; existing command/participant validation still checks that origin. Missing Shatter provenance, missing switch origins and costs outside either admission kind remain rejected. Aggregate success alone does not establish replay validity.

Offline final curation can use four isolated WASM instances inside one validator under the shared four-CPU affinity. Each instance validates one exact source and writes its own receipt; state and RNG are never shared. The shipped catalog generator remains sequential and reuses only exact source/engine/WASM/generator matches. Normalize Rust fingerprint inputs to the repository's LF convention before the final build, including Git-clean legacy CRLF files, so checked-in source and generated identity agree after checkout.

The generator writes its final display JSON through the installed repository formatter and configuration. Generation must pass the existing web format gate without a manual post-generation rewrite; this changes artifact serialization only, leaving the shared-engine calculation and raw receipts intact.

### 5. Reconcile the prerequisite specification explicitly

Before applying catalog changes, synchronize the completed `add-studio-team-list` delta through its normal workflow after checking its existing completion evidence. Do not discard or rewrite its implementation. Then re-read accepted `studio-application/spec.md` and add a full `MODIFIED` block for `Team list entry and scenario selection` to this change: replace its fixed-five-entry wording and its matching scenario precondition with the curated collection, preserving all other navigation, exact-source, explicit-Run and previous-result scenarios. The new collection requirements in this delta then add constraints without leaving contradictory accepted catalog definitions. Keep archival of the predecessor separate from the current change's eventual completion.

## Risks / Trade-offs

- Current guides recommend unavailable characters or newer mechanics → filter both characters and signature Arcs before selection; match canonical available IDs and retain dated source references.
- Source-backed values coexist with selected execution hypotheses → preserve model assumptions and use causal receipts; avoid claiming game-measured accuracy.
- Full-run validation or bug fixes can be expensive → diagnose with small runs, run one validator at a time under four logical CPUs, cache exact unchanged results and rerun invalidated cases only.
- A currently unsupported mandatory interaction may need more than a script adjustment → keep the required roster, implement the smallest supported owner correction with evidence, and keep any remaining blocker explicit.
- Fixed five-entry assumptions exist in the predecessor and tests → reconcile the accepted requirement before applying, then check source selection, ranking, filters and long-list scrolling with the actual collection.
- Different investment or target assumptions can mislead rankings → share the baseline, disclose exceptions, retain native run conditions, and describe DPS as each authored scenario's outcome.

## Migration Plan

1. Resolve the predecessor spec dependency without changing its unrelated working-tree edits.
2. Research and validate candidates while retaining the existing menu; record only concise durable rationale and normal OpenSpec task progress.
3. Replace selectable partial entries, preserve S5 and required regression files, regenerate current source/DPS assets, and repair affected documentation and checks.
4. Run focused native/web/Worker and browser checks appropriate to the final diff, strict OpenSpec validation, stable diff review and `python tools/checkpoint.py run` before committing. Archive this change only after its implementation tasks and checks pass.
5. If integration fails, keep the catalog replacement pending while correcting the affected script or generator; do not discard unrelated edits or publish a partially validated collection. Deployment requires separate authorization.
