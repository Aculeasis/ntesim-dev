# Restore character mechanics in public scenarios

## Why

The character audit found discrepancies between the ordinary scenario path and the repository's source-backed mechanics. Public scenario setup omits some implemented owner models, and several cast and damage conditions disagree with their accepted timing or progression rules.

## What Changes

- Select supported Adler, Aurelia and Lacrimosa/Baicang models through public scenario projection.
- Correct Cadenza, Adler Burst callback timing, Blackbird's team ATK grant, Chaos's Warrant bonus and Mint's HP threshold.
- Apply Jiuyuan's existing Pact resistance reduction consistently at Resonance 6.
- Reconcile inaccurate example comments and mechanics references with executable behavior and frozen source rows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `runtime-content` covers character progression, existing supported owner models and exact effect conditions in authored scenarios.

## Impact

The shared Rust simulator, DSL, Studio projection and mechanics checks are affected. Live source occurrences can report the `swap` action window through the existing Worker transport.
