# Proposal: Compact lossless simulation reports

## Why

Exporting a 120-second 4-character Studio simulation ([`s5.nte`](../../../../presets/studio/s5.nte)) currently produces an **18,089,015 B (17.25 MiB)** `nte-research-report.json` file despite single-run data costing far less. Almost half of the file comes from duplicating the full first sample under `selected_sample`, while inside [`GameScenarioReport`](../../../../crates/content/src/simulation/model/report.rs) static provenance/assumptions/talent tables repeat linearly per hit and command, `state_timeline` re-serializes unchanged actor fields and complete active-buff snapshots on every change, `trace` embeds English prose sentences (`detail`) alongside four-way ungrouped energy fanout entries, and every `ExactFraction` expands into a four-line JSON object. Removing those sources of duplication without compression or dropping pretty-printing reduces the same `indent=2` JSON export to **~2.64–3.19 MiB** (81–85% smaller) while separating pure report data from UI localization.

## What Changes

1. **Remove duplicate `selected_sample` from Studio export** — **BREAKING**
   - Stop emitting `selected_sample` alongside `run.result` in [`exportReport`](../../../../web/src/app/App.tsx) (`App.tsx` / `SamplePage.tsx`). `importReportFile` does not consume `selected_sample`, and any sample seed in `samples` can be deterministically reproduced from `compiled` + `seed` via `SOURCE_SAMPLE`.
2. **Normalize static provenance, assumptions, metadata, and talent receipts** — **BREAKING**
   - Intern repeated `NativeEvidence` objects into `report.evidence_catalog` and unique `ActionPacketMetadata` templates (excluding execution-local `energy_group_id`) into `report.metadata_catalog`.
   - Store `metadata_id` (and optional `energy_group_id`) plus `execution_model_assumption_ids` referencing `report.resolved_model_assumptions` on `hit_receipts`, `action_metadata_ledger`, and `player_input_receipts` instead of inline 1.5–2 KB descriptions and provenance copies.
   - Group `talent_projection_receipts` and `action_metadata_ledger` by unique `(payload sans command_index)` with `command_indices: Vec<usize>`, and update native/CSV/TypeScript validators and replay builders accordingly.
3. **Delta and catalog encoding for `state_timeline`** — **BREAKING**
   - Extract static buff definitions into `report.buff_catalog` (`BuffDefinition` referenced by `buff_def_id`), replace full `Vec<ActiveBuffInstance>` snapshots on intermediate `Event` frames with `buff_diff` (`upsert`/`remove` operations by explicit buff instance key), and emit sparse field-level `GameActorStateDelta` records (carrying `slot` plus changed fields only) on intermediate `Event` frames while retaining full `Initial` and `Final` boundary frames.
4. **Structured, prose-free `trace` events with grouped energy fanouts** — **BREAKING**
   - Remove free-form English `detail` strings from [`GameEventReceipt`](../../../../crates/content/src/simulation/model/report.rs) so reports remain language-neutral structured data: replace prose-embedded parameters (such as switch actor indices, buff/heal payloads, hit stack counts) with compact structured fields/references (`hit_index`, structured event data), and update [`Localization.event`](../../../../web/src/domain/localization.ts) in Studio to format purely from structured data without regex parsing English text.
   - Group simultaneous hit-triggered energy grants sharing `(time_us, phase, event_kind, active_actor, source, ability_id, hit_basis)` into one trace entry with `energy_fanout.recipients` instead of emitting N nearly identical events per hit.
5. **Compact `"<numerator>/<denominator>"` exact rational serialization** — **BREAKING**
   - Serialize [`ExactFraction`](../../../../crates/core/src/combat/exact/fraction.rs), [`Ratio`](../../../../crates/core/src/kernel/formula/mod.rs), and [`WideFraction`](../../../../crates/core/src/kernel/state/wide_pool.rs) uniformly as exact canonical `"<numerator>/<denominator>"` decimal strings across JSON requests, reports, `NTE wire v1`, TypeScript consumers, authored JSON scenarios, and verification scripts, preserving arbitrary integer precision beyond `2^53 - 1` without object verbosity or backward-compatibility shims.

## Capabilities

### New Capabilities
- None (all five changes refine existing combat report, WASM wire, and Studio application contracts).

### Modified Capabilities
- `combat-execution`: Require normalized report catalogs (`evidence_catalog`, `metadata_catalog`, `buff_catalog`), delta timeline observations, language-neutral structured trace events with grouped energy fanouts, and `"num/den"` exact rational serialization in native requests/reports/exports.
- `studio-application`: Omit redundant `selected_sample` from trial report exports, update report validation/playback/import consumers for normalized/delta receipts and `"num/den"` rationals, and render localized event labels/details exclusively from structured trace fields rather than English regex matching.
- `wasm-transport`: Preserve exact wide-integer rationals via `"num/den"` canonical strings and align WASM binary/JSON endpoints (`export_source_report`, `replay_source_report`, `step_source_trials`, parity baselines) with compact normalized report schemas.

## Impact

- **Rust engine & CLI (`crates/core`, `crates/content`, `crates/research`, `crates/cli`)**: Exact rational serde (`ExactFraction`, `Ratio`, `WideFraction`), report structs (`GameScenarioReport`, `GameStateFrame`, `GameEventReceipt`, `GameHitReceipt`, `ActionMetadataOccurrence`, `talents.rs`), timeline `Recorder`/`validate_state_timeline`, studio trial observer (`observe.rs`), CSV tables (`csv.rs`), and report-inspecting Rust tests (including `.detail` substring assertions and `{"numerator","denominator"}` assertions).
- **WASM transport (`crates/wasm`, `crates/wasm/js`)**: Unified compact report payloads through `NTE wire v1`, `source_reports.rs`, `source_trials.rs`, and Rust/JS parity tests.
- **Studio (`web/src`, `web/tests`)**: Export payload (`App.tsx`, `SamplePage.tsx`), lossless/exact rational helpers (`contracts/ratio.ts`, `validation-support.ts`, `statistics.ts`), report validators (`validation.ts`, `validation-metadata.ts`, `energy.ts`), playback state reconstruction (`replay/model.ts`, `replay/events.ts`), and pure data-driven localization (`localization.ts`, `localization.test.ts`, `EventLog.tsx`).
- **Fixtures & scripts (`examples/*.json`, `tools/verify_*.py`, `tools/*.mjs`, `openspec/specs/*/reference.md`)**: Update authored JSON rational literals and oracle scripts together with code changes (no backward-compatibility shims).
