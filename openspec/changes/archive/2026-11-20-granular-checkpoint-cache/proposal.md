# Proposal

## Why

During local development, running `python tools/checkpoint.py run` frequently takes upwards of 3.5 to 4 minutes on every invocation because `rust-tests` hashes the entire `crates/` directory into a single monolithic check. Modifying a downstream crate (e.g. `ntesim-research`, `ntesim-cli`) or an individual test forces all 2,573 tests in `ntesim-content` (~85-100s) and other unaffected crates to execute repeatedly. In addition, `native-build` and `wasm-build` are never cached, and multi-package `cargo test` invocations face execution/linking contention under the 4-CPU limiter.

Introducing granular, per-package test caching based on upstream crate dependency closures and caching unchanged builds allows checkpoint to skip tests and builds that have already passed, dramatically reducing cycle times from minutes down to seconds when working on focused changes.

## What Changes

- **Granular Package-Level Rust Test Caching**: Compute transitive upstream dependency closures for each Rust crate. Track and cache successful test execution per package in `successes.json`.
- **Skip Unchanged Passing Packages in `rust-tests`**: When executing `rust-tests`, verify each required package against its cached fingerprint; execute `cargo test` only for packages whose inputs changed or have not yet passed.
- **Sequential Package Test Execution**: Execute dirty packages sequentially rather than in a single combined `-p ... -p ...` invocation, reducing linking/runtime contention under the 4-CPU limiter and recording intermediate package successes even if a subsequent package fails.
- **Cached Build Steps (`native-build`, `wasm-build`)**: Bind build steps to their package source closures and generated binary artifacts (`target/debug/ntesim.exe`, `target/wasm-bindgen/...`), skipping redundant compiler invocations when sources and outputs are unchanged.
- **Receipt Integrity**: Preserve full pre-commit verification invariants in `receipt.json` and `verify_checkpoint`, ensuring all required checks and packages are fully validated before commit.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `development-workflow`: Update `Focused checks follow maintained inputs` to specify that test caching tracks package-level upstream dependency closures, skipping already passed packages when their sources and dependencies are unchanged.

## Impact

- `tools/checkpoint.py`: Checkpoint planning, input fingerprinting, test execution, and success cache logic.
- `tests/test_agent_checkpoint.py`: Tests verifying granular package reuse, build caching, and invalidation rules.
- `docs/DEVELOPMENT.md`: Documentation of checkpoint caching behavior and package-level test reuse.
