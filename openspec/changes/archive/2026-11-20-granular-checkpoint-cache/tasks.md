# Tasks

## 1. Rust Package Dependency Resolution and Fingerprinting

- [x] 1.1 Implement upstream crate closure and package input resolution in `tools/checkpoint.py`, and verify unit tests in `tests/test_agent_checkpoint.py` confirm correct dependency scopes per package
- [x] 1.2 Implement granular package test caching and sequential execution within `rust-tests` in `tools/checkpoint.py`, and verify unchanged packages are skipped while dirty packages run

## 2. Build Step Caching and Verification

- [x] 2.1 Enable artifact and source-backed caching for `native-build` and `wasm-build` in `tools/checkpoint.py`, and verify build steps reuse prior successes when binaries and sources match
- [x] 2.2 Update `tests/test_agent_checkpoint.py` with tests for package-level test reuse, downstream modification isolation, and build artifact reuse, verifying all tests pass

## 3. Documentation and Integration Verification

- [x] 3.1 Update `docs/DEVELOPMENT.md` to document package-level test caching and build reuse behavior
- [x] 3.2 Run OpenSpec validation and complete checkpoint verification (`python tools/openspec.py validate --all --strict --no-interactive --concurrency 4` and `python tools/checkpoint.py run`) to confirm performance improvements and compliance
