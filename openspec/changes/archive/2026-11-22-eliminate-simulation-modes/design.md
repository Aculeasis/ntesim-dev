# Design

## Context

NTESIM originally envisioned two simulation modes: `strict` (which disallowed all unverified model hypotheses and random RNG rolls) and `research` (which permitted hypotheses and stochastic evaluations). Over time, all actual gameplay features—including character talent projection, weapon and set effect procs, and stochastic crit evaluation—were implemented only for the `research` pipeline. `strict` mode remained as dead code that caused build compilation and scenario validation to fail, while reports permanently included `strict_ready: false`.

See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Completely remove the `mode` concept from contracts, runtime requests, DSL compilation, and validation.
- Remove the `strict_ready` boolean flag from all simulation report structs, receipts, and Web UI validators.
- Rename fields with `research_*` prefixes (`research_crit` -> `stochastic_crit`, `research_action_metadata` -> `action_metadata`) and CLI command `batch-game-research` -> `batch-game`.
- Update all consumers, unit tests, and fixtures in lockstep without backward-compatibility shims.

**Non-Goals:**
- Renaming the `ntesim-research` workspace crate or altering the internal structure of batch analysis routines.
- Changing combat simulation logic, damage formulas, or RNG distribution mechanics.

## Decisions

### Decision 1: Clean Removal Over Shims
- **Choice**: Remove `mode` and `strict_ready` completely from structs, parsers, and interfaces.
- **Alternatives considered**:
  - Defaulting `mode` to `research` silently when omitted: Rejected because keeping the field preserves dead mental overhead and contradicts the `AGENTS.md` mandate (*"No backward compatibility: update current contracts, consumers and tests together; do not add obsolete-input shims"*).
- **Impact**: Any DSL script specifying `mode=...` will fail to parse as an unsupported option; all JSON fixtures and examples must drop `"mode"`.

### Decision 2: Report Readiness Contract Cleanup
- **Choice**: Delete `strict_ready` from all Rust report models (`ntesim_core`, `ntesim_content`, `ntesim_data`, `ntesim_research`, `ntesim_cli`) and TypeScript report types (`web/src/domain/contracts/`).
- **Rationale**: `strict_ready` was always hardcoded to `false`. Removing it simplifies the report wire format and schema validation in `report-validation.ts`. Reports continue to track `status`, `quality` (`incomplete`, `bounded_hypothesis`, etc.), and `game_dps_computed`.

### Decision 3: Renaming Research-Prefixed Fields and CLI Subcommands
- `GameScenarioRequest::research_crit` -> `stochastic_crit`: Highlights the actual behavior (stochastic RNG crit evaluation and per-actor roll sequences) rather than an obsolete mode name.
- `GameScenarioRequest::research_action_metadata` -> `action_metadata`: Represents custom caller-supplied action metadata.
- CLI subcommand `batch-game-research` -> `batch-game`: Standardizes the batch evaluation command.

### Decision 4: Hypothesis Recording Simplification
- In `ntesim_content::runtime_api::hypotheses::record_hypothesis`, remove the check requiring hypotheses to be pre-declared in `model_assumptions`. Encountered hypotheses are automatically recorded in `execution_assumptions` for report provenance, while conflicting reached hypotheses continue to produce an error.
- Remove `validate_strict_model_assumptions` and `require_strict_hypothesis`.

## Risks / Trade-offs

- **[Risk] Serialized fixtures and examples breaking due to unknown fields**:
  `serde(deny_unknown_fields)` on `GameScenarioRequest` will reject files containing `"mode": ...`.
  $\rightarrow$ **Mitigation**: Update all `examples/*.json`, `reviews/**/*.json`, and test fixtures to remove `"mode"` and `"strict_ready"`. Run `python tools/checkpoint.py run` to ensure all checks pass.

- **[Risk] Web UI / Worker transport desynchronization**:
  If WASM report outputs drop `strict_ready` while Web UI storage validation expects it, reports will fail to load.
  $\rightarrow$ **Mitigation**: Update `web/src/infrastructure/storage/report-validation.ts` and test suites (`web/tests/report-import-cases.mjs`, etc.) to verify reports without `strict_ready`.
