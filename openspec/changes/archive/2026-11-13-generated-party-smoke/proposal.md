# Proposal

## Why

Equipping Lost Radiance on Zero in Studio S5 reproducibly fails on a later Skill with `defense shred/ignore must be in [0,1]`. Some legal equipped parties also exceed the combined 32-entry hypothesis limit using canonical declarations automatically added by the engine.

## What Changes

- Bound supported additive DEF-ignore contributions at 100% for direct and Break damage while preserving individual formula validation.
- Retain the 32-entry custom hypothesis allowance independently of exact declarations from the finite engine registry.
- Add a focused S5 Zero/Lost Radiance native replay regression and extend the existing independent arithmetic, Break and metadata validation boundaries.
- Keep the implementation on its separate branch without a commit, as requested. Do not add a generated party sweep.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `combat-execution`: bounded aggregate DEF ignore and bounded custom model declarations without rejecting canonical engine provenance.

## Impact

Shared Rust content aggregation and validation, existing content tests and mechanics references. No new dependency, serialized field or UI feature. Native and WASM use the same correction; the admission change is checked through the existing WASM/Worker smoke boundary.
