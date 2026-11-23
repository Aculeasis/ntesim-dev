# Design

## Context

`tools/checkpoint.py` ensures the worktree is validated before commits are allowed via pre-commit hooks and `receipt.json`. However, during active development, checkpoint's monolithic `rust-tests` hashing and unconditional builds result in 3.5 to 4 minute runtimes even when only a single downstream test or file was modified. See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Skip executing tests for Rust packages whose sources and upstream dependency closures have not changed and have already passed.
- Cut package execution time under the 4-CPU limiter by testing remaining dirty packages sequentially.
- Cache `native-build` and `wasm-build` against their source closures and output binary artifacts.
- Preserve full compatibility with `covers_check`, `test_agent_checkpoint.py`, and pre-commit verification.

**Non-Goals:**
- Caching sub-module test cases within a single crate's test binary (e.g. running only a subset of tests inside `ntesim-content`).
- Bypassing `--fresh` (which must continue to clear all caches and run all checks fresh).

## Decisions

### Decision 1: Transitive upstream dependency closure for package fingerprinting
For each Rust package, we inspect the workspace `Cargo.toml` manifests to compute its upstream dependency closure (the package itself plus all dependencies in `dependencies`, `dev-dependencies`, and `build-dependencies`, transitively).

The input files for a package's test key include:
- All files under `crates/<dir>/` for each crate in the transitive upstream closure.
- Root toolchain files: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `tools/cargo.py`, `tools/checkpoint.py`, `tools/cpu_limit.py`, `tools/tool_output.py`.
- Shared tabular inputs:
  - `presets/research/` for packages whose closure includes `ntesim-data`, `ntesim-content`, `ntesim-research`, `ntesim-cli`, or `ntesim-wasm`.
  - `fixtures/`, `research/`, `reviews/` for packages whose closure includes `ntesim-research` or `ntesim-cli`.
  - `examples/`, `presets/` for packages whose closure includes `ntesim-cli`.

*Alternative considered*: Hashing all of `crates/` for every package. Rejected because changes in `crates/cli` would invalidate `ntesim-content`.

### Decision 2: Granular package execution and caching inside `rust-tests`
`check_plan` retains the top-level check `rust-tests` with the list of required packages:
`["python", "tools/cargo.py", "test", "--locked", "--offline", "-p", pkg1, "-p", pkg2, ...]`.
This preserves exact compatibility with existing planning tests, `covers_check`, and `receipt.json`.

In `run_checkpoint`:
- First, if the full `rust-tests` key matches `successes.json`, the entire check is reused (0.00s).
- If not, `run_checkpoint` computes each required package's individual fingerprint against `successes.json`.
- Packages whose individual fingerprint is already marked successful are skipped.
- If all required packages are already in `successes.json`, `rust-tests` completes immediately with `reused: True`.
- Any remaining dirty packages are executed sequentially:
  `["python", "tools/cargo.py", "test", "--locked", "--offline", "-p", pkg]`
- When an individual package test succeeds, its individual success key is recorded immediately. If a subsequent package fails, the successes of earlier packages survive in `successes.json` for subsequent runs.
- Once all required packages succeed, the overall `rust-tests` key is also recorded.

*Alternative considered*: Splitting `rust-tests` into `rust-test:<pkg>` checks in `check_plan`. Rejected because multiple existing tests in `test_agent_checkpoint.py` assert on `plan` containing `"rust-tests"` with all `-p` arguments, and `covers_check` expects `"rust-tests"`. Keeping the contract at `rust-tests` while executing and caching granularly inside `run_checkpoint` satisfies both the contract and the performance goal.

### Decision 3: Caching build steps (`native-build`, `wasm-build`)
Instead of returning `None` in `check_inputs` for `native-build` and `wasm-build`:
- `native-build` inputs: the transitive upstream closure of `ntesim-cli`, plus `target/debug/ntesim.exe` (or `ntesim`) in `artifact_inputs`.
- `wasm-build` inputs: the transitive upstream closure of `ntesim-wasm`, plus `target/wasm-bindgen/ntesim_wasm_bg.wasm` and `target/wasm-bindgen/bindings.js` in `artifact_inputs`.
When the source closure, environment, toolchain, and binary artifact SHA256 match a previous success, the build step is reused without running Cargo and `wasm-bindgen`.

## Risks / Trade-offs

- **[Risk]** A deleted build artifact causes a false positive reuse.
  → **Mitigation**: `artifact_inputs` hashes the output binary files. If an artifact is deleted or modified, its hash becomes `None` (or changes), invalidating the cache key and forcing a rebuild.
- **[Risk]** Inter-crate circular or unmodeled dependencies.
  → **Mitigation**: Rust strictly disallows circular dependencies, and workspace manifests declare all inter-crate dependencies. Transitive traversal guarantees all upstream dependencies are included in the hash.
