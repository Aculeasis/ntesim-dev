# Design

## Context

See proposal.md - Why.
All major cards, panels, and grids in the Studio simulation report use 8px margins and gaps (`.result-team`, `.metadata`, `.panel`, and grid gaps). Aligning `.rollup-grid` to 8px ensures unified visual rhythm.

## Goals / Non-Goals

**Goals:**
- Align `.simulation-report .rollup-grid` bottom margin to 8px.

**Non-Goals:**
- Modifying chart structures or panel layouts.

## Decisions

- Change `.simulation-report .rollup-grid` margin-bottom from 14px to 8px.

## Risks / Trade-offs

- None.
