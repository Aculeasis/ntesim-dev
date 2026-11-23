# Design: Compact lossless simulation reports

## Context

See [proposal.md](proposal.md) for the size measurements (`18,089,015 B` / `17.25 MiB` -> `~2.64–3.19 MiB` pretty `indent=2` JSON on [`s5.nte`](../../../../presets/studio/s5.nte)) and capability summary.

The codebase enforces strict exactness, deterministic causality, zero backward-compatibility shims ([`AGENTS.md`](../../../../AGENTS.md)), shared Rust core (`crates/core`), content runtime (`crates/content`), studio research (`crates/research`), CLI (`crates/cli`), WebAssembly/Worker bridge (`crates/wasm`), and React/TypeScript Studio (`web/`). Measurements showed four structural causes of JSON bloat alongside a separate Studio UI bug:
1. Studio `exportReport` attaches `selected_sample` alongside `...run.result`, duplicating the full 8.6 MiB sample execution on disk even though `importReportFile` never reads it.
2. Static `NativeEvidence`, `ActionPacketMetadata`, `execution_model_assumptions` prose, `action_metadata_ledger`, and `talent_projection_receipts` are denormalized into every hit and action receipt.
3. `state_timeline` re-emits full `Vec<ActiveBuffInstance>` snapshots and complete 16-field `GameActorStateSummary` structs whenever any charge remainder or buff expires.
4. `GameEventReceipt` embeds English prose in `detail` (which Studio regex-parses in `Localization.event`), while ordinary-hit energy grants emit 1..4 near-identical events instead of one grouped fanout receipt.
5. `ExactFraction`, `Ratio`, and `WideFraction` serialize as four-line `{ "numerator", "denominator" }` objects across reports, requests, fixtures, and `NTE wire v1`.

## Goals / Non-Goals

**Goals:**
- Reduce exported pretty-printed (`indent=2`) report size by ~81–85% purely via structural normalization, delta timelines, structured data-only trace events, and compact exact rational strings — with **zero** numeric precision loss, **zero** compression, and **zero** loss of pretty formatting.
- Separate pure simulation data from UI text: eliminate English prose (`detail`) from Rust combat reports and move all display formatting/localization into Studio over typed report data.
- Keep `SourceReportExport` native replay, `damage_analysis()`, CSV exports, WASM `NTE wire v1` transport, and zero-epsilon native/WASM parity exact without adding legacy fallback parsers.

**Non-Goals:**
- Minifying JSON (`indent=0`), gzip/zstd binary report containers, or lossy floating-point approximation of exact rationals.
- Retaining legacy compatibility shims for `{numerator, denominator}` objects, `selected_sample`, or inline denormalized metadata.
- Altering combat mechanics, RNG draws, admission order, or statistical aggregation values.

## Decisions

### Decision 1: Remove `selected_sample` and unused callback wiring from Studio export (`App.tsx` / `SamplePage.tsx`)
- **Choice**: Remove `sampleReport` state from [`App.tsx`](../../../../web/src/app/App.tsx) and the `onReport` callback prop from [`SamplePage`](../../../../web/src/features/playback/SamplePage.tsx); stop emitting `selected_sample` from `exportReport`.
- **Rationale**: `SamplePage` mounts immediately while hidden and initializes `execution` from `run.result`, causing `exportReport` to duplicate ~8.56 MB identically. Meanwhile, [`importReportFile`](../../../../web/src/infrastructure/storage/files.ts) never reads `selected_sample`, and any sample seed in `samples` can be deterministically recomputed from `compiled` + `seed` via `SOURCE_SAMPLE`.
- **Alternatives considered**: Conditionally omitting `selected_sample` only when `selected_sample.report.seed === run.result.report.seed`. Rejected because `importReportFile` does not consume `selected_sample` for alternate seeds either, and keeping a conditionally present 8.5 MB blob retains dead schema complexity.

### Decision 2: Intern static provenance, metadata, assumptions, and talent receipts
- **Choice**:
  - Add `evidence_catalog: Vec<NativeEvidence>` and `metadata_catalog: Vec<NormalizedActionPacketMetadata>` to `GameScenarioReport` (where `evidence_id: usize` replaces inline `NativeEvidence` on `source_evidence`, `identity_evidence`, and `Native` provenance, and `hypothesis_id: String` references `resolved_model_assumptions` in `AcceptedModelHypothesis` provenance).
  - Store `metadata_id: usize`, optional `energy_group_id: Option<String>`, and `execution_model_assumption_ids: Vec<String>` (keys into `report.resolved_model_assumptions`) on `GameHitReceipt` and `ActionMetadataOccurrence` (and `execution_model_assumption_ids` on `PlayerInputReceipt`).
  - Group identical entries (excluding `command_index`) in `action_metadata_ledger` and `talent_projection_receipts` using `command_indices: Vec<usize>`. Update `PreparedSourceScenario` / [`BorrowedGameScenarioReport`](../../../../crates/content/src/simulation/runtime/report.rs) so `talents` pre-encodes deduplicated entries once at compilation/projection time.
  - Update [`validateObservations`](../../../../web/src/domain/reports/validation.ts), [`createMetadataValidator`](../../../../web/src/domain/reports/validation-metadata.ts), `observe.rs`, `replay/events.ts`, and `SourceReportExport::csv_tables()` to resolve `metadata_id`, `evidence_id`, `command_indices`, and `execution_model_assumption_ids` via the shared report catalogs.
- **Alternatives considered**: Keeping inline `metadata` and `execution_model_assumptions` on `HitReceipt` for `damage.csv`'s `receipt_json` column. Rejected because CSV consumers receive `report.json` in the same `SourceReportExport` bundle and normalizing hits shrinks both JSON reports and `damage.csv`.

### Decision 3: Delta + catalog `state_timeline` (`buff_catalog`, `buff_diff`, `GameActorStateDelta`)
- **Choice**:
  - Add `buff_catalog: Vec<BuffDefinition>` to `GameScenarioReport` containing immutable definition attributes (`id`, `source`, `owner_actor`, `recipient`, `refresh_policy`, `stacking_scope`, `source_lifetime`, `semantic_tags`, `max_stacks`, `decay_policy`, `stat_modifiers`).
  - Assign a monotonic deterministic `instance_id: u64` in core/content buff state tracking so concurrent stacks of the same buff (such as Fireflies stacks) have unambiguous 1-to-1 identity across timeline deltas.
  - Keep `Initial` (`frame[0]`) and `Final` (`frame.last()`) frames complete, emitting compact `ActiveBuffRef { instance_id, buff_def_id, stacks, duration }` lists in `Initial`/`Final` and `final_buff_state`. Intermediate `Event` frames emit `buff_diff: Option<BuffTimelineDiff>` (`{ upsert: Vec<ActiveBuffRef>, remove: Vec<u64> }`) and sparse `actors: Vec<GameActorStateDelta>` (`slot` + only fields modified since the previous frame).
  - Extend [`validate_state_timeline`](../../../../crates/content/src/simulation/timeline.rs) and [`createReplay`](../../../../web/src/domain/replay/model.ts) to fold deltas sequentially from `Initial` through every `Event` frame and verify exact equality against `Final`, `final_actor_states`, and `final_buff_state`.
- **Rationale**: In `s5.nte`, active buff re-snapshots (1,658 instances vs 516 actual diff ops) and 16-field actor snapshots triggered 951 times by `charge_remainder` account for ~1.63 MB of `state_timeline`.

### Decision 4: Prose-free structured `trace` events and grouped energy fanouts
- **Choice**:
  - Remove `detail: String` from [`GameEventReceipt`](../../../../crates/content/src/simulation/model/report.rs) across all runtime emitters (`crates/content`), eliminating English prose from report data.
  - Replace prose-embedded information with typed compact fields/payloads on `GameEventReceipt`: e.g. `hit_damage` carries `{ hit_index, gear_stacks? }` (without repeating `damage_dealt` or effect/ability prose), `switch_actor` carries `{ from_actor, to_actor }`, and heal/buff/proc/stack/control events carry a compact optional `data: Option<GameEventData>` struct or tagged variant (`skip_serializing_if = "Option::is_none"`) with structured fields (`actor`, `target`, `recipient`, `ability_id`, `effect_id`, `buff_id`, `stacks`, `max_stacks`, `duration_us`, `expires_at_us`, `gross`, `applied`, `overflow`, `excess`, `hp_before`, `hp_after`, `value`, `reason`, etc.).
  - Group simultaneous energy grants sharing `(time_us, phase, event_kind, active_actor, source, ability_id, hit_basis)` at the same causal event boundary into `energy_fanout: Option<GameEnergyFanoutReceipt>` (`source`, `source_actor_index`, `ability_id?`, `hit_basis?`, `recipients: Vec<GameEnergyRecipientReceipt>`), retaining `energy_spend` / single-recipient `energy_change` for spends and standalone grants.
  - Update [`Localization.event`](../../../../web/src/domain/localization.ts), [`replayEvents`](../../../../web/src/domain/replay/events.ts), [`EventLog.tsx`](../../../../web/src/features/playback/EventLog.tsx), [`validateEnergy`](../../../../web/src/domain/reports/energy.ts), [`observe.rs`](../../../../crates/research/src/trials/studio/observe.rs), [`energy.csv`](../../../../crates/content/src/simulation/export/csv.rs), and Rust tests (`event_kind` + structured `data`/`energy_fanout` assertions) so Studio formats all English/localized UI text solely from structured report data without regex parsing.
- **Alternatives considered**: Removing `detail` only from `energy_change` trace events while keeping regex-parsed English `detail` on buffs/heals/hits. Rejected per architectural clarity: reports are machine-readable data, not localized strings, and UI localization must consume structured fields directly.

### Decision 5: Uniform `"<numerator>/<denominator>"` exact rational serialization
- **Choice**:
  - Implement canonical `"<numerator>/<denominator>"` string serialization and strict string deserialization on `ExactFraction` (`u128`), `Ratio` (`u64`), and `WideFraction` (`num_bigint`), rejecting `{numerator, denominator}` object shapes and zero/invalid denominators.
  - Across `NTE wire v1` (`crates/wasm/src/wire.rs`), each rational now traverses binary transport as an interned UTF-8 string (`STRING`/`REFERENCE` tags 4/5) instead of a 2-key object container, unifying compact JSON and exact wide-integer (> `2^53 - 1`) transport cleanly.
  - Update TypeScript `ExactFraction` / `Ratio` / `WideFraction` contracts, `parseExactFraction` / `exactFraction` / `fraction`, `lossless-json` boundaries if needed, `examples/*.json`, Rust unit/integration tests, and `tools/verify_*.py`/`.mjs`.
- **Alternatives considered**: Keeping `{numerator, denominator}` on `GameScenarioRequest` and changing only `GameScenarioReport`. Rejected because shared types in requests and exports would split into two conflicting wire formats in the same `TrialResult` bundle (`request` vs `report`) and violate the zero-shims policy in `AGENTS.md`.

## Risks / Trade-offs

- **[Risk] Broad mechanical touch surface in Rust tests and example JSON files** -> ~105 Rust files construct `GameEventReceipt`, and simulation tests currently assert on `event["detail"].contains(...)` and `{"numerator": ..., "denominator": ...}`.
  - **Mitigation**: Update [`GameEventReceipt`](../../../../crates/content/src/simulation/model/report.rs) and `ExactFraction`/`Ratio` constructors with clean Rust builders/helpers and migrate tests and `examples/*.json` in focused crates/batches using exact structured field assertions (running `cargo test` per crate incrementally).
- **[Risk] Buff delta identity ambiguity when duplicate buff IDs coexist (e.g., independent-stack Fireflies)** -> Keying diffs by `(id, owner_actor, source)` alone fails when multiple independent instances exist.
  - **Mitigation**: Include a monotonic deterministic `instance_id: u64` on active buff instances inside `BuffManager` so `buff_diff.upsert` and `buff_diff.remove` operate by exact `instance_id` while `validate_state_timeline` verifies step-by-step equality against `Final`.
- **[Risk] Drift between native JSON, CSV exports, WASM wire decoding, and Studio UI** -> Schema changes can desynchronize offline workers, parity baselines, or `export-script`.
  - **Mitigation**: Update `crates/wasm`, `crates/content/src/simulation/export/csv.rs`, `web/src/domain/`, and `tools/verify_*.py` in the same change and verify through `python tools/checkpoint.py run --full`.

## Migration Plan

1. Migrate exact rational serde (`"num/den"`) across `crates/core`, `crates/content`, `crates/wasm`, `examples/*.json`, TS contracts, and verification scripts/tests.
2. Normalize static catalogs (`evidence_catalog`, `metadata_catalog`, `execution_model_assumption_ids`, `command_indices` grouping) and update Rust/TS validators and CSV export.
3. Implement `buff_catalog` (`instance_id`), `state_timeline` buff diffs, and `GameActorStateDelta` field diffs in `Recorder`, `validate_state_timeline`, and Studio playback (`createReplay`).
4. Replace `GameEventReceipt.detail` prose with structured fields/data and grouped `energy_fanout`, migrating Rust tests and `Localization.event` / `EventLog.tsx`.
5. Remove `selected_sample` duplication from `App.tsx` / `SamplePage.tsx`, regenerate generated UI JSON/WASM artifacts, update capability `reference.md` docs, and run focused + full checkpoint suites.
