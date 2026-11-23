# Tasks

## 1. Contracts and Content Engine

- [x] 1.1 Remove `Mode` enum and `mode` field from `ntesim-contracts` (`Scenario`), update validation, and verify with `python tools/cargo.py test -p ntesim-contracts`
- [x] 1.2 Remove `mode` parsing from `ntesim-content` DSL options (`crates/content/src/dsl/options.rs`) so `mode` is rejected as an unsupported option, and verify with `python tools/cargo.py test -p ntesim-content -- options`
- [x] 1.3 Remove `mode` from `GameScenarioRequest`, `CompiledOptions`, build compiler, and simulation validation in `ntesim-content`; remove strict-mode validation functions (`validate_strict_model_assumptions`, `require_strict_hypothesis`), update `record_hypothesis`, rename `research_crit` -> `stochastic_crit` and `research_action_metadata` -> `action_metadata`, and verify with `python tools/cargo.py test -p ntesim-content`

## 2. CLI, Data, and Research Crates

- [x] 2.1 Remove `strict_ready` from report structs and receipts across `ntesim-core`, `ntesim-data`, `ntesim-content`, `ntesim-research`, and `ntesim-cli`, and verify with `python tools/cargo.py test -p ntesim-core`
- [x] 2.2 Update `ntesim-data` and `ntesim-research` to eliminate `mode` from team scenarios and batch runners, rename CLI subcommand `batch-game-research` to `batch-game`, and verify with `python tools/cargo.py test -p ntesim-data -p ntesim-research -p ntesim-cli`

## 3. Web UI, Transport, and Fixtures

- [x] 3.1 Update TypeScript contracts (`web/src/domain/contracts/`), remove `strict_ready` from `web/src/infrastructure/storage/report-validation.ts`, update `web/src/generated/team-list.json` and `scenarios.json`, and verify with `node tools/web.mjs test`
- [x] 3.2 Update `examples/*.json`, `reviews/**/*.json`, and test fixture files to remove `mode` and `strict_ready`, and verify with `python tools/cargo.py test`

## 4. End-to-End Verification

- [x] 4.1 Run the full checkpoint suite via `python tools/checkpoint.py run` and verify all native, WASM, web tests, and repository hygiene checks pass
