# Proposal

## Why

A valid victory rotation against a 50 million HP dummy exhausts the default script operation budget before the one-hour combat horizon. Existing operation, event and horizon messages do not consistently identify the numeric limit or explain why the run stopped.

## What Changes

- Raise the default per-trial script operation budget from 20,000 to 60,000.
- Report exhausted operation and event budgets with their configured values and clear descriptions.
- Explain victory horizon failures with the configured combat-time limit in seconds, retaining incomplete results and typed diagnostics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `execution-safety`: Define the default script operation budget and readable numeric limit diagnostics.

## Impact

Shared Rust DSL evaluation and combat execution diagnostics, existing replay tests, and execution-safety documentation. Native CLI and WASM/Worker inherit the messages without transport or UI schema changes. No damage formulas, stop policies or other safety limits change.
