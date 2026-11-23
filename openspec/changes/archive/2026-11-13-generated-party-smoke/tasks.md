# Tasks

## 1. Fix observed equipped-party failures

- [x] 1.1 Reproduce S5 with Lost Radiance on Zero, implement shared exact DEF-ignore saturation for direct and Break damage, extend independent numeric coverage and update the defense reference; verify the S5/native replay and boundary checks pass.
- [x] 1.2 Exclude exact canonical engine declarations from the 32 custom-entry allowance, extend existing validation coverage for canonical/custom/mismatched bounds and repair the metadata reference; verify the focused metadata test passes.

## 2. Retain focused regression coverage

- [x] 2.1 Remove the generated party sweep and its module registration, development documentation and proposed workflow requirements; verify no active source or documentation references remain.

## 3. Integration

- [x] 3.1 Review the final diff, pass focused Rust checks and strict OpenSpec validation, then run `python tools/checkpoint.py run --wasm-smoke` on the final implementation.

## Workflow follow-up

- Archive the completed change with synchronized specs after checks pass.
- Validate the archived result and check the final stable tree. Leave all changes uncommitted on their separate branch; do not merge or publish.
