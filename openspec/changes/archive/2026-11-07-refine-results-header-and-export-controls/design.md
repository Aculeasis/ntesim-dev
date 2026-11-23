# Design

## Context

See proposal.md - Why.
Studio views currently place `.view-heading`, `.intro`, and `.editor-actions` sequentially in the document flow. On non-config views (`data-view !== 'config'`), `.editor-actions` renders as a static block aligned to the right, causing an empty row to appear between the intro description and the simulation progress / results.

## Goals / Non-Goals

**Goals:**
- Replace `▷ Run` with a standard `Export report` button in the top action bar when `view === 'results'`.
- Eliminate the dead vertical gap above simulation progress by pairing view title/intro and action controls in a shared flex header.
- Eliminate the redundant `results-toolbar` row below the party cards in `ResultsPage`.

**Non-Goals:**
- Changing simulation execution or worker lifecycle.
- Modifying the floating fixed action bar on the `Simulator` (`config`) view.

## Decisions

### 1. Flex layout container `.view-header`
- Wrap title/intro in `.view-header-title` alongside `.editor-actions` inside `.view-header`.
- When `data-view="config"`, existing styles position `.editor-actions` fixed at the bottom of the viewport; `.view-header-title` expands naturally.
- When `data-view !== 'config'`, `.editor-actions` aligns horizontally with the heading and intro, avoiding extra line breaks and empty space.

### 2. Standard button for Export report on Results tab
- In `App.tsx`, condition the primary button on `view === 'results'`:
  - `view === 'results'`: renders `<button id="export-report-btn" disabled={!run.result} onClick={exportReport}>Export report</button>`.
  - other views: renders the existing `▷ Run` button (`#run-btn` with `run-button` class).
- `Export report` does not carry the `run-button` class, rendering with default button aesthetics matching `Settings`.

### 3. Removal of redundant `results-toolbar` from `ResultsPage`
- Remove `<div className="results-toolbar">` containing `Completed · X / Y completed`, `Export report`, `Edit config`, and `▶ View sample`.
- Progress and iterations are cleanly shown in `RunProgress` immediately above `TeamCards`.
- Tab navigation (`Simulator`, `Results`, `Sample`) and the top `Export report` action provide access to all three operations.

## Risks / Trade-offs

- *[Risk]*: Responsive narrow screen overflow.
  → *Mitigation*: Under 900px, `.view-header` stacks vertically with column direction and full width controls.
