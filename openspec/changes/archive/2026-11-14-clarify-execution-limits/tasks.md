# Tasks

## 1. Shared execution limits and diagnostics

- [x] 1.1 Raise default operations to 60,000, clarify operation/depth/input/action messages and extend the existing evaluator/live-loop tests; document the ceilings in the execution-safety reference.
- [x] 1.2 Add numeric event and victory horizon messages, distinguish early rotation exhaustion, and extend existing source-clock, event-limit and source-export replay assertions.

## 2. Integration verification

- [x] 2.1 Run focused Rust tests and reproduce successful completion of the supplied 50 million HP seed-42 rotation with default limits.
- [x] 2.2 Review the stable diff, run strict OpenSpec validation and `python tools/checkpoint.py run` including the selected native/WASM smoke boundary.

## Workflow follow-up

- Archive the completed change with spec synchronization, validate archived artifacts and run checkpoint on the final tree before committing.
