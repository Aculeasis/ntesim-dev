# Design

## Context

See proposal.md - Why.
Rollup metric cards render in `.rollup-grid` without a bottom margin, causing immediate abutment with the `.two-column.report-overview` panels below. Target cards in `Targets.tsx` stack target title above level and health statistics, inflating height when scenarios configure multiple targets. Furthermore, the `Target Information` panel title adds redundant height when 3+ targets are already identified by each target's own card header.

## Goals / Non-Goals

**Goals:**
- Provide clear vertical separation between rollup cards and overview panels.
- Display each target's name and primary stats on a single header line.
- Omit the `Target Information` panel heading when 3 or more targets are present.

**Non-Goals:**
- Modifying elemental resistance calculation or resistance icon layouts.
- Altering the distribution histogram chart on the right column.

## Decisions

### 1. Panel heading rendering
- In `Panel.tsx`, mark `title?: string` as optional.
- Render `div.panel-heading` conditionally on `Boolean(title || controls)`. When neither is provided, omit the heading container entirely so children start without unused top padding or margins.

### 2. Single-line target header
- In `Targets.tsx`, wrap the target label `<strong>Target {idx + 1}:</strong>` and the `.target-stats` container in `.target-header-line`.
- Use `display: flex; align-items: baseline; flex-wrap: wrap; gap: 6px 14px;` so target identification and primary statistics (Level, HP, DEF) flow seamlessly on one line on desktop.

### 3. Adaptive panel title for 3+ targets
- In `ResultsPage.tsx`, pass `title={request.targets.length >= 3 ? undefined : 'Target Information'}` to the target panel.
- Scenarios with 1 or 2 targets retain the section title; scenarios with 3 or more targets maximize vertical compaction.

### 4. Rollup grid vertical spacing
- In `studio-report.css`, assign `.simulation-report .rollup-grid { margin-bottom: 14px; }` to maintain clean separation above overview panels.

## Risks / Trade-offs

- *[Risk]*: Small viewports wrapping target header line.
  → *Mitigation*: `flex-wrap: wrap` allows stats to wrap below the target label on narrow screens without overflowing horizontally.
