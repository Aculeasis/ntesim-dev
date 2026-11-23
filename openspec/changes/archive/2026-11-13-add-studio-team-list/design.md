# Design

## Context

See [proposal.md](proposal.md) for motivation and [the Studio delta](specs/studio-application/spec.md) for behavior.

`presets/studio/index.json` owns five selectable scripts and one non-selectable language reference. The 20-second S4 entry is excluded from the user-facing catalog; its unlisted source remains an input of existing native regression tests. `useConfig.setPreset` replaces the draft with the selected source. `Tabs` represents three actual views with tab keyboard handling. `StudioDialogs` and `Dialog` supply modal composition and focus restoration. `CharacterCard` includes interactive stat controls and a separate weapon block, unsuitable for a clickable catalog row without a compact presentation variant.

The current WASM client exposes Rust-owned configuration inspection, compilation and streaming trial collection (`startSourceTrials`, `stepSourceTrials`, `snapshotSourceTrials`). Existing Studio tests and Worker checks consume those APIs; JavaScript does not need to calculate DPS. The accepted source-identity contract keeps previous results bound to their executed source independently of a newly selected draft.

This crosses navigation, presentation, generated data and build freshness, so a design artifact is applicable.

## Goals / Non-Goals

**Goals:** Keep catalog opening immediate, reuse current artwork and source identity, and produce a small reproducible display projection from Rust output. Make generation incremental and keep computation outside browser interaction.

**Non-Goals:** No new Rust mechanics, public CLI command, WASM wire field or Worker operation. No backend, roster optimizer, personal team database or adjustment of existing scripts to make their conditions equal. Catalog DPS is the result of each authored scenario, not a controlled comparison of identical combat conditions.

## Decisions

### 1. Retain the script manifest as the single selection source

Extend the existing preset pipeline rather than create a second hand-maintained team catalog. Keep script IDs, labels, source and default `s5`; exclude `language-reference` from selection as today. The generated bundle will carry a versioned, typed Team list projection for each selectable script: ordered character display/build fields from Rust inspection, calculation conditions and a discriminated DPS result (`ready` with numeric mean, or `unavailable` with a reason).

Retain source hash and engine version/source hash with the projection. Build inputs and cache identity also include the WASM bytes hash and generator/projection version. Never store absolute workstation paths. No raw reports, sample arrays or generated verification receipts enter tracked assets; those stay under `cache/`. JSON here transports authored scripts and display output; it does not become a runtime mechanics catalog.

Alternative: manually author roster and DPS metadata in the manifest. Rejected because it can drift from the loaded script and duplicate Rust-owned inspection.

### 2. Precompute with the existing Rust trial session through WASM in Node

Add explicit team-catalog generation and freshness-check modes to the existing preset/web tooling. Use the current WASM build and bindings through the cached Node/tsx toolchain. Compile each exact selectable source, obtain its Rust inspection/options and run all authored iterations with the same consecutive seed sequence as Studio. Consume the final native mean DPS directly from `aggregate.metrics.dps.mean`; do not reimplement per-trial rates or statistical aggregation in TypeScript.

Preserve the authored duration/stop/crit/mode/seed/iteration settings, including the existing single-iteration Baicang scenario. S1-S3 and Baicang use a 20-second horizon while S5 uses 120 seconds. Retain conditions, target stats and native assumptions/quality in generated metadata and raw local calculation outcomes. Team list rows have no Details control or calculation report. The ordinary Results/export workflow retains its existing model-limit presentation. A reached unsupported dependency, failed or incomplete run, or native output without computed game DPS cannot supply a ranked number. Keep the scenario with unavailable status and its short diagnostic; valid zero remains `ready`.

Generate one scenario at a time under the existing four-CPU wrapper. Cache reusable outputs by exact source, engine/WASM identity and generator version. Verify that the WASM embedded engine source identity matches the current `tools/cargo.py` engine fingerprint before generation. A fresh checkout must reproduce assets without `cache/`; the cache only avoids repeated computation. Emit deterministic compact output, excluding wall-clock timings and timestamps.

Build/check integration validates selectable-entry coverage, schema, exact source identity and engine identity. Stale projections fail delivery with the documented regeneration command; they are never silently reused. Explicit regeneration recomputes invalidated entries and records unavailable outcomes when the engine cannot complete them. A current bundle remains usable without the local calculation cache. Ordinary UI opening, filtering and selection never call simulation APIs.

Alternatives: browser computation delays opening and consumes user resources; hardcoded DPS has no reproducible identity; a new native aggregate command expands a public API unnecessarily. The existing Rust session through WASM already produces the displayed Studio metric.

### 3. A dialog button in the navigation, with a dedicated compact row

Keep the three view tabs at the left edge. Use a three-column navigation grid with equal flexible outer columns, place the tablist in the left column and Team list in the center column. Team list is a normal dialog-opening button outside `tablist` semantics and the arrow-key view-switching loop. At narrow widths use a separate centered row to avoid collisions. It remains available without Results/Sample. Compose the dialog through `StudioDialogs` while keeping the current view selected underneath.

Use a dedicated compact character presentation in the team feature with existing `Artwork`, character metadata and equipment tooltip primitives. Share useful visual helpers with `CharacterCard` where appropriate without changing full Simulator cards or nesting their stat buttons inside a row action. Each row has a distinct scenario activation control and separately focusable equipment inspection, with no nested buttons. Display name, awakening, portrait, element, Arc/refinement and cartridge/count; omit stat expansion, skill lines, the separate weapon row and calculation Details. Four slots follow declared order; unfilled slots are inert. Start with roughly 90-100 pixel character-card height and tune against existing cards during browser review; do not scale down text with a CSS transform.

Use a desktop modal around `min(1120px, 94vw)` and at most `85dvh`. Keep its header outside the scrolling list. Place the scenario label and four compact cards to the left, DPS to the right. On narrow screens wrap controls and use a two-by-two slot grid within each scenario row. The exact spacing is a visual implementation detail, not a new view or gameplay concept.

Alternative: add a fourth page. Rejected because the requested popup should preserve the current screen and dismiss without navigation.

### 4. Combined presentation filters with independent pickers

Keep query and selected canonical character-ID sets in app-level session state so dialog unmount/reopen retains them; do not extend draft local-storage contracts. Build picker options from the current display character catalog, sorted by displayed name. Use portraits and names, selected styling, accessible pressed state and a per-picker Clear All. Include and Exclude have independent counts, including zero. Only one picker is open at a time; switching buttons changes the open picker. Click outside dismisses it; Escape closes the picker first, then the parent dialog on a later press.

Search trims/case-folds whitespace-separated terms and requires each term to occur in at least one actual member's display name; terms can match different members. Apply Include as all-selected membership and Exclude as any-selected rejection. A character in both sets produces no matches without silently modifying either set. Preserve query/filters in an empty state and show a concise no-results message. Sort ready entries by unrounded DPS descending, break ties by stable script ID and place unavailable entries last, also ordered by ID.

These are catalog presentation operations, not simulated mechanics. No DSL parsing is permitted in filters or card construction.

### 5. Scenario loading follows existing draft semantics

Row activation uses the existing preset-loading boundary, switches to Simulator, closes the dialog and scrolls to the editor view start. It replaces the draft just as Load configuration currently does and does not auto-run. Remove the old selector and obsolete toolbar styling if no consumer remains. Keep executed Results/Sample attached to the prior source; do not inject precomputed summaries as user-run results.

While either a trial run or sample execution is busy, allow browsing and inspection but disable scenario activation with a short explanation. Reuse the existing `busy` state instead of inventing cancellation or resume behavior. Dismissal restores focus to Team list. Keyboard focus stays within the modal; row activation, portrait toggling and equipment information must all work without a pointer.

## Risks / Trade-offs

- Different durations, builds, target conditions and sample sizes can make ranking look like a universal team tier list -> retain exact scripts and calculation metadata; keep the existing executed Results/export workflow available for detailed inspection.
- Engine or source changes invalidate precomputed numbers -> enforce identities in the build/check boundary and regenerate invalidated entries before delivery.
- Some presets may fail current engine checks -> preserve their roster/source and show unavailable DPS with the diagnostic instead of changing mechanics or dropping scenarios to manufacture success.
- Recalculation can be expensive -> reuse valid calculation cache entries, run scenarios sequentially and retain current CPU limits; browsing never pays that cost.
- Compact cards and overlapping pickers can become unreadable or inaccessible -> use dedicated card layout, contained scrolling, clear focus states and desktop plus 390-pixel browser acceptance.
- Native assumptions may be lengthy -> keep calculation provenance outside the compact list rows; do not add a Details expander to the catalog.

## Migration Plan

1. Extend the existing preset tooling and generate the five retained team projections against a current shared-engine build; retain raw outcomes in `cache/` and exclude S4 from catalog transport.
2. Introduce the dialog, compact rows and filters, wire selection to the existing draft-loading boundary, and remove Load configuration in the same change.
3. Update the Studio reference and development instructions with generation/freshness behavior and calculation cache usage. Retain existing import/export and executed-source semantics.
4. Validate focused presentation and generator cases, existing Worker parity and actual browser interaction, then review the stable diff and run the checkpoint before delivery. No publication or deployment is included.

The generated schema and UI consumers change together without compatibility shims. Reverting the coherent change restores the old selector and source-only bundle without migrating stored drafts.
