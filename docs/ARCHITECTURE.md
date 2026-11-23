# Architecture

This document owns current structural boundaries. The [decision records](decisions/README.md) preserve reasons, trade-offs, provenance and reconsideration conditions; they do not establish runtime readiness. Relevant records: [Rust and one engine](decisions/0001-rust-product-engine.md), [browser choices](decisions/0002-react-static-browser.md), [domain ownership](decisions/0003-shared-services-and-content-owners.md), [event-driven time](decisions/0004-event-driven-simulation.md), [modeling scope](decisions/0005-dps-modeling-boundary.md) and [rotation language](decisions/0006-sequential-rotation-language.md).

## One engine

Changes follow the selected OpenSpec capability and the affected-boundary [check selection](DEVELOPMENT.md#choosing-checks). Native and browser consumers share the Rust engine.

Rust is the only product engine and owns ALL domain logic: DSL parsing, AST, validation, parameter/stat derivation, calculations, event simulation, timeline management, and result aggregation. Python supplies acquisition, deterministic preparation and independent audits. Frozen partial Python models are evidence, not a second maintained implementation or a reason to preserve obsolete research code. Routine product tests/data audits must work after moving this directory; acquisition takes `--legacy-root` explicitly. The browser/TS layer must not implement a secondary domain engine.

Five layers: contracts/data; event kernel; NTE adapters; rotation/research execution; native CLI and WASM adapters. Scalar asset configurations are declarative data. Model fidelity follows the [DPS relevance policy](../openspec/specs/product-scope/reference.md#dps-relevance-and-modeling-boundary); use lightweight event/state models when unrelated detail is costly. Complex character transitions are typed Rust adapters; no runtime execution of Unreal bytecode. Numerical reference probes are retained as partial evidence.

The workspace enforces these boundaries with `ntesim-contracts`, `ntesim-core`, `ntesim-data`, `ntesim-content`, and `ntesim-research`. Core owns shared clock, queue, combat, actor, modifier and target services and cannot depend on content or research. Data owns immutable package audits, scalar bindings, console legality and build compilation. Content owns per-character, weapon and set behavior and scenario execution over core services. Research owns published-team comparisons, hypotheses, trials, search, verification and ambient-time profiling. CLI and WASM compose these same crates.

Runtime modules, types, commands and output fields use domain names rather than source-site brands. Historical source locations are isolated in `ntesim-data::package` metadata, with source URLs retained in evidence/comments. The frozen package is not renamed or regenerated. Per-character `preview.rs` files preserve historical manual inputs and never establish executable completeness or strict readiness.

## Content ownership

Implemented character transitions live under `content/src/characters/<character>/`; equipment transitions belong to their named modules under `content/src/weapons/` and `content/src/sets/`. A small equipment owner may be one `<owner>.rs` file. Promote it to `<owner>/mod.rs` with focused state, trigger, callback and projection modules when those responsibilities need independent changes; do not create empty folders or one-function files merely for symmetry. Native scalar tables and legality stay in data. Do not add empty executable adapters for catalog-only items: absence remains explicit.

The content registry dispatches by stable native IDs. Typed character and equipment state belongs to its adapter; the shared actor does not acquire per-kit fields. Content schedules typed `ContentEvent` callbacks through contexts carrying simulation time, queue and sequence, never through ambient time or IO. Scenario execution owns admission and hit settlement separately and retains exact packet receipts and quality boundaries.

Equipment owners keep runtime hooks, configuration/deadline validation and native projection together. `gear/validation.rs` routes typed proc variants and checks shared uniqueness/deadline bounds; owner-specific legality stays in the owner. `build/gear.rs` registers native weapon projectors once for both build compilation and Studio catalog generation. Studio obtains character primary packets through the content registry. Keep the explicit causal order of equipment hooks.

`simulation/hit/` separates delivery eligibility, speculative statuses, exact calculation, RNG and committed effects. Character-specific target restrictions dispatch through registry hooks. Metadata separates public types, native resolution, owner binding and research declarations; request, energy and report models have separate modules. A pre-hit status contributes speculatively and commits only when damage connects; target defeat and resource callbacks retain their delivery order.

Split shared services by responsibility, with adjacent tests and narrow facade exports. File size alone is not a reason to invent a new abstraction. New content must have an obvious owner without adding character-specific branches to core. `tests/test_architecture.py` protects Cargo dependency direction, portable core capabilities and source-neutral identifiers. Behavior-preserving migrations must verify full deterministic reports, event order, resource conservation and RNG, plus native/WASM parity when the browser artifact is rebuilt.

Character-specific hit rules dispatch through generic typed registry hooks. Ability provenance, packet legality and equipment effects remain in their content owners; shared hit calculation and settlement do not hardcode character identities.

## Determinism and time

Integer simulation time, stable event ordering and explicit clock domains. Asset coordinates are a distinct unit and cannot enter runtime schedules without a binding. Event priorities express tested settlement policies, not guessed game defaults. Use a pinned portable seeded RNG; each iteration has its own seed/state, and aggregation is ordered by iteration. Native and WASM must share gameplay code.

Execution is event-driven: advance directly to the next queued event, readiness/transition deadline, wait endpoint or horizon. Timestamp precision does not imply per-frame/per-microsecond iteration, and authored resource/state waits must not introduce fixed-rate polling. Preserve real periodic mechanics as events; [the execution contract](../openspec/specs/script-execution/reference.md#event-driven-time-advancement) owns the detailed wake-up and sparse-gap checks.

The live source integration exposes one real elapsed combat-time axis for DSL waits/horizons, scenario timestamps, reports and DPS, distinct from host execution time. Content owns Hotori Time Stop and other mode-specific process progress/pause policies, remaining lifetimes and resource/combo transitions. Core's independently tested local-clock services remain internal tools, not additional public clocks.

## Interfaces

Versioned JSON requests/configuration/results. Stable native IDs plus display aliases; never use localized names as primary keys. Runtime loading receives bytes in memory. The filesystem and network belong to tools/adapters. Dependency diagnostics identify the mechanic and evidence gap, including non-numeric policies.

[Script inspection](../openspec/specs/script-inspection/reference.md) returns canonical IDs, exact static party stats and initial resources from configuration compilation without executing the rotation. Rust source diagnostics retain typed codes and editor-compatible ranges. CLI and the thin WASM adapter serialize the same typed results; presentation must not rebuild stats or validate combat rules.

native combat mechanics (B3/Q2) own permitted tap/hold/channel continuation/cancellation after switching and automatic Support Skills on qualifying entry, including owner readiness/resource checks. These shared engine rules apply independently of the scripting language; no universal cancellation or 250 ms lock is assumed. Q4 integrates ordinary scripted inputs with those mechanics rather than implementing them in the interpreter. See the [combat lifecycle contract](mechanics/shared/actor-lifecycle.md#combat-switching-mechanics).

Rotation language follows [NTE DSL](NTE_DSL.md) and its [execution contract](../openspec/specs/script-execution/reference.md): sequential actions, switches, waits, exact numeric variables/arithmetic, live read-only readiness/resource/state/modifier queries, conditions and bounded loops. User functions, priority lists and raw engine property access are excluded. Rust interprets branches per trial. Native owners define typed hold/target/boolean choices and next-action transition windows; admission separates actor recovery, field control and pending effects. Shared party Fons and scenario delivered damage/critical/parry assumptions retain source/provenance. The interpreter halts immediately on failed ordinary action admission; only authored waits suspend execution; [executor diagnostics](../openspec/specs/combat-execution/reference.md#waiting-and-idle-diagnostics) own idle/block receipts outside DSL syntax. Studio calls the shared live Rust source executor. No access to future random draws or invented player inputs. Interpreter steps/events have explicit budgets.

## Research and migration

Keep source bytes immutable. Bundle the required evidence for portable audits; do not ship client binaries, whole archives, recordings or arbitrary external scripts in the runtime package. Preserve source attribution and upstream license metadata separately from original product code.

Research follows the [rolling data policy](../openspec/specs/product-scope/reference.md#rolling-data-policy); source hashes, frozen packages and exact patch certification are not research gates. Current native/WASM adapters consume typed content and maintained CSV presets directly; historical raw-package inspection/import commands are retired. Actual release availability is separate from datamined presence. Catalog presence never authorizes execution.

## Browser boundary

The existing React/TypeScript studio is strictly a presentation and integration layer. It receives user scripts, delegates parsing/validation/execution to Rust via WASM/Worker, and visualizes output (editor diagnostics, catalogs, timelines, charts, and simulation statistics). Keep browser dependencies outside core. Ordinary native mechanics changes use Rust/native checks; source/report/adapter boundary changes and phase exits retain bounded WASM/Worker smoke checks. Full native/WASM parity and actual Worker/replay/aggregation checks return when changing the web layer and for final acceptance.

The studio must not implement or duplicate combat calculations, DSL compilers, or stat evaluators. The source frontend now lives in `content::dsl`; Studio and Worker call its WASM API for compilation, diagnostics and source-preserving option edits. The duplicate TypeScript frontend and combat test suite have been removed. Model assumptions, unsupported mechanics and execution failures remain explicit; interface availability alone does not establish combat coverage. No simulation service, accounts or server database.
