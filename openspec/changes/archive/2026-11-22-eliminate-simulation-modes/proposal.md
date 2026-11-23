# Proposal

## Why

The division between `strict` and `research` simulation modes is a legacy artifact from early development. Modern NTESIM execution (build compilation, talent projection, gear proc emulation, and stochastic RNG critical evaluation) requires hypotheses and only functions under what was formerly called `research` mode; `strict` mode cannot even compile builds and only survives in obsolete unit tests. Furthermore, reports hardcode `strict_ready: false` across all engines and transports.

Eliminating this artificial split simplifies the architecture, unifies the simulation pipeline into a single canonical execution model, removes dead validation code, and cleans up obsolete `mode` and `strict_ready` parameters from the DSL, contracts, transports, CLI, and Web UI.

## What Changes

- **BREAKING**: Remove the `mode` option from the DSL options syntax (`options mode=...;`). The DSL compiler rejects `mode` as an unsupported option.
- **BREAKING**: Remove `mode` field and `Mode` enum (`strict` / `research`) from `ntesim-contracts`, `Scenario`, `GameScenarioRequest`, `CompiledOptions`, and related transport schemas.
- **BREAKING**: Remove `strict_ready` from simulation reports, result contracts, and Web UI validation schemas. Reports retain quality (`incomplete`, `bounded_hypothesis`, etc.) and `game_dps_computed` without an obsolete readiness stub.
- Remove strict-mode validation branches: `validate_strict_model_assumptions`, `require_strict_hypothesis`, and strict-mode rejections for talent levels, gear procs, stochastic crit, and custom action metadata.
- Rename internal structs, fields, and CLI commands bearing the `research` prefix to generic simulation terms:
  - `research_crit` -> `stochastic_crit` (or unified `crit_policy`).
  - `research_action_metadata` -> `action_metadata` (or `custom_action_metadata`).
  - `batch-game-research` CLI command -> `batch-game`.
- Update all sample files, fixtures, tests, and Web UI types/validators to the unified contract without backward-compatibility shims.

## Capabilities

### Modified Capabilities

- `script-execution`: Update the scenario options requirement to remove `mode=research` defaulting and legacy mode scenarios, reflecting that the script compiler no longer accepts or requires a `mode` option.

## Impact

- **Rust crates**:
  - `ntesim-contracts`: Remove `Mode` enum and `mode` field from `Scenario`.
  - `ntesim-content`: Remove `mode` from `GameScenarioRequest`, `CompiledOptions`, options parser, build compiler, validation, and runtime hypothesis recording. Rename `research_crit` and `research_action_metadata` fields.
  - `ntesim-cli`: Update CLI commands (rename `batch-game-research`), remove `strict_ready` from report outputs.
  - `ntesim-data`: Remove `mode` from static team scenarios and loadout compilers.
  - `ntesim-research`: Remove `mode` checks in batch runner and scenario generators.
- **Web / TypeScript**:
  - `web/src/domain/contracts/`: Remove `mode` from `scenario.ts` and `script.ts`; remove `strict_ready` from `report.ts`, `source-report.ts`, and `trials.ts`.
  - `web/src/infrastructure/storage/report-validation.ts`: Remove `strict_ready` check.
  - `web/src/generated/team-list.json` and `scenarios.json`: Remove `mode` and `strict_ready`.
- **Fixtures & Tests**:
  - Remove obsolete tests verifying strict-mode hypothesis rejections.
  - Update all JSON fixtures, examples, and test helpers to omit `mode` and `strict_ready`.
