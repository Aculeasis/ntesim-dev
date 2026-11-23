# Proposal

## Why

The vertical margin beneath the rollup summary cards was set to 14px, which exceeds the consistent 8px spacing standard used across all sections and cards in the simulation report.

## What Changes

- Adjust the bottom margin of `.simulation-report .rollup-grid` from 14px to 8px to match the consistent 8px report layout spacing.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
<!-- None: visual spacing refinement consistent with existing spec requirements -->

## Impact

- `web/src/styles/studio-report.css`: Update `.simulation-report .rollup-grid` margin-bottom to 8px.
