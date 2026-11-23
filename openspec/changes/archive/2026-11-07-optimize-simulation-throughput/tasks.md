# Tasks: Optimize Simulation Throughput

## 1. Action Descriptor Preparation Caching

- [x] 1.1 Pre-resolve action packet metadata and command parameters during scenario preparation to avoid full scenario request cloning, dual validation, and talent JSON serialization inside `admit_descriptor`, verified by `python tools/cargo.py test -p ntesim-content -- source_scheduler`.
- [x] 1.2 Verify that static and dynamic rotation action admissions, packet indices, and diagnostic locations match existing fixtures, verified by `python tools/cargo.py test -p ntesim-content -- actions`.

## 2. Lean Batch Trial Execution (Sample vs Batch Separation)

- [x] 2.1 Accumulate active shield time-weighted HP directly in combat state clock advancement and expose the scalar in scenario reports, verified by a parity unit test asserting exact equality with timeline-derived shield HP across shielded characters.
- [x] 2.2 Introduce a lightweight execution path for batch trials (trials 1..N) that bypasses per-event `timeline.observe()` frame creation, diffing, and redundant trace event allocations, verified by `python tools/cargo.py test -p ntesim-research -- studio`.
- [x] 2.3 Verify that sample trial 0 preserves full timeline frames, receipts, and validation integrity for the Studio inspector UI, verified by `python tools/cargo.py test -p ntesim-wasm`.

## 3. Browser Pipeline and Single-Core Benchmarking

- [x] 3.1 Audit `web/src/infrastructure/worker/runner.ts` chunking, timer resolution, and snapshot serialization across the WASM boundary, verified by `node tools/web.mjs test`.
- [x] 3.2 Benchmark native and WASM single-core throughput on S5 Mint/Sakiri/Zero/Iroi (`iteration=1000 duration=120`), recording before/after single-thread execution latency and verifying zero numerical divergence in aggregate results.

## 4. Verification and Checkpoint

- [x] 4.1 Run full native and WASM test suites to verify zero regressions across character kits and CLI tools: `python tools/cargo.py test --release`.
- [x] 4.2 Run repository checkpoint: `python tools/checkpoint.py run`.
