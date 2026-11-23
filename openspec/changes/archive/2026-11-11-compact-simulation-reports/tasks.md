# Tasks: Compact lossless simulation reports

## 1. Canonical exact rational `"numerator/denominator"` format

- [x] 1.1 Update `ExactFraction`, `Ratio`, and `WideFraction` serde in `crates/core` to serialize/deserialize canonical `"<numerator>/<denominator>"` decimal strings (rejecting `{numerator, denominator}` objects and invalid/zero denominators), update `crates/wasm/src/wire.rs` & wire tests, and verify with `python tools/cargo.py test -p ntesim-core -p ntesim-wasm --locked --offline`.
- [x] 1.2 Migrate `examples/*.json`, Rust scenario tests, and Python/Node oracle helpers (`tools/verification_inputs.py`, `tools/verify_*.py`, `tools/*.mjs`, `tools/benchmark_native_wasm.py`) to `"numerator/denominator"` strings, verifying `python tools/cargo.py test -p ntesim-content -p ntesim-cli --locked --offline`.
- [x] 1.3 Update Studio rational contracts and parsers (`web/src/domain/contracts/ratio.ts`, `validation-support.ts`, `statistics.ts`, `team/stat-rows.ts`, `CharacterStats.tsx`) plus `openspec/specs/combat-execution/reference.md` and `openspec/specs/wasm-transport/reference.md`, verifying `node tools/web.mjs types` and `lossless_json.test.mjs` / `wasm_wire.test.mjs`.

## 2. Normalized provenance, metadata, assumptions, and talent receipts

- [x] 2.1 Implement `evidence_catalog`, `metadata_catalog`, `execution_model_assumption_ids`, and `command_indices` deduplication for `hit_receipts`, `action_metadata_ledger`, `player_input_receipts`, and `talent_projection_receipts` across `crates/content` (`model/`, `talents.rs`, `runtime/report.rs`, `export/csv.rs`), updating `observe.rs`, CLI JSON/replay tests, and `openspec/specs/combat-execution/reference.md`; verify `python tools/cargo.py test -p ntesim-content -p ntesim-research -p ntesim-cli --locked --offline`.
- [x] 2.2 Update TypeScript contracts, `validateObservations`, `createMetadataValidator`, `replayEvents`, `ResultsPage`, and web fixture assertions (`web/tests/fixtures/*.mjs`) to resolve normalized metadata, evidence, assumption IDs, and `command_indices`; verify `node tools/web.mjs types` and metadata/replay web tests.

## 3. Delta and catalog encoding for `state_timeline`

- [x] 3.1 Add deterministic buff `instance_id` tracking and `buff_catalog` extraction in `crates/core`/`crates/content`, update `Recorder` and `validate_state_timeline` (`crates/content/src/simulation/timeline.rs`, `export/csv.rs`) to emit and fold `buff_diff` (`upsert`/`remove`) plus sparse `GameActorStateDelta` records on `Event` frames while preserving complete `Initial`/`Final` frames, and extend timeline tamper tests; verify `python tools/cargo.py test -p ntesim-content -p ntesim-research --locked --offline`.
- [x] 3.2 Update TypeScript `StateFrame`/`ScenarioReport` contracts, `createReplay` state folding, `ReplayStage`/`StatePanel`, `openspec/specs/combat-execution/reference.md` + `openspec/specs/studio-application/reference.md`, and `web/tests/replay.test.mjs`; verify `node tools/web.mjs test`.

## 4. Structured data-only `trace` events and grouped energy fanouts

- [x] 4.1 Remove `detail` prose (and duplicate `damage_dealt` on `hit_damage`) from `GameEventReceipt`, emit structured event refs/payloads (`hit_index`, `from_actor`, `to_actor`, `data`) plus grouped `energy_fanout` across `crates/content`, update `observe.rs`, `export/csv.rs`, and Rust simulation trace assertions, and verify `python tools/cargo.py test --workspace --locked --offline`.
- [x] 4.2 Update [`Localization.event`](../../../../web/src/domain/localization.ts), `replay/events.ts`, `EventLog.tsx`, `validateEnergy`, `web/tests/localization.test.ts`, and `openspec/specs/studio-application/reference.md` to format localized UI text purely from structured event/receipt fields without English regex matching; verify localization, report controls, and Studio tests via `node tools/web.mjs test`.

## 5. Studio export deduplication and full integration verification

- [x] 5.1 Remove `selected_sample`, `sampleReport`, and `onReport` wiring from `web/src/app/App.tsx` and `web/src/features/playback/SamplePage.tsx`, update `openspec/specs/studio-application/reference.md`, and verify report export/import round-trip in `web/tests/studio.test.tsx`.
- [x] 5.2 Rebuild native CLI, WASM artifacts, and generated web data, run `presets/studio/s5.nte` (`indent=2` JSON export confirmed <= 3.5 MiB vs 17.25 MiB baseline: measured 2,530,056 B / 2.41 MiB), verify `tools/verify_wasm_parity.py`, run `python tools/openspec.py validate --all --strict --no-interactive --concurrency 4`, and pass `python tools/checkpoint.py run --full`.

## Workflow follow-up

- Run `python tools/openspec.py archive compact-simulation-reports --yes` after all tasks and checkpoint verification pass.
- Re-run `python tools/openspec.py validate --archived --strict --no-interactive --concurrency 4` and final `python tools/checkpoint.py run` before squashing/merging branch `compact-simulation-reports` into `main`.
