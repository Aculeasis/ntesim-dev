# Web architecture

The studio uses React, strict TypeScript and Vite over one Rust/WASM engine. UI availability does not establish complete combat models or validated DPS.

## Delivery stage

Studio uses the current shared Rust source executor. [Check selection](DEVELOPMENT.md#choosing-checks) distinguishes native mechanics, shared API smoke and full browser integration checks.

Rust [script inspection](../openspec/specs/script-inspection/reference.md) supplies the whole party, canonical IDs, exact compiled static stats, initial resources, build contributions and source locations. Team cards render these values without calculating derived stats or embedding gameplay validation. Missing display metadata shows the canonical ID.

The editor uses `compile_source`, retaining party and typed effective options when rotation preflight fails. Invalid drafts remain editable and block Run. Diagnostic and character-declaration controls select Rust UTF-16 ranges, mapping imported CRLF/CR to the textarea's normalized LF representation without changing authored source. Settings use Rust configuration-only source edits, including stop policy and optional victory duration; unchanged expressions and unrelated text survive. Font/theme/wrap changes do not recompile the source. Inspection is configuration support. Worker execution uses prepared Rust live source sessions; Rust owns trial aggregation and quality. Results, selected seed execution, reports and source navigation retain the executed artifact and actual per-seed requests independently of draft edits.

The [browser decision](decisions/0002-react-static-browser.md) records the confirmed reasons for React and avoiding a simulation backend. [One engine](decisions/0001-rust-product-engine.md) and the [rotation-language decision](decisions/0006-sequential-rotation-language.md) distinguish shared semantics from the transitional browser compiler. This document remains the current web boundary owner.

| Directory | Responsibility |
| --- | --- |
| `web/src/app` | Entry point, tabs and feature composition |
| `web/src/features/editor` | Config editing, diagnostics and name search |
| `web/src/features/team` | Compiled team, equipment and stat cards |
| `web/src/features/results` | Completed-trial charts and research metadata |
| `web/src/features/playback` | Sample controls, event log and recorded state |
| `web/src/features/simulation` | Run status, streaming snapshots and cancellation |
| `web/src/features/settings`, `import-export` | Settings and file dialogs |
| `web/src/components` | Reusable presentation components and charts |
| `web/src/domain` | Presentation contracts and existing report/replay adapters; no DSL or stat compiler |
| `web/src/infrastructure` | generated WASM bindings, binary transport, Worker protocol and local persistence |
| `web/src/generated` | Display catalogs, audited native build inputs and fixture projections as JSON |
| `web/src/styles` | Small CSS modules preserving the existing cascade |

Team list is a modal launched from the center of the navigation, independently
of the three actual view tabs grouped at the left. Its compact
cards, name search and independent Include/Exclude portrait pickers consume
`web/src/generated/team-list.json`; they never parse scripts or run simulations.
The existing script manifest remains the selection source, with S4 excluded.
Rows have no calculation Details control. Selecting a row uses
the current preset draft-loading boundary, switches to Simulator and waits for
Run, preserving the executed identity of any previous Results/Sample. Active
execution permits browsing but disables team loading. Filters survive reopening
within the app session and do not extend stored-draft contracts.

The offline `tools/studio_teams.mts` generator consumes Rust inspection and native
streaming trial aggregates through the existing WASM client. TypeScript only
projects display metadata; Rust owns per-trial rates and mean DPS. The build/check
boundary validates exact source, current engine/WASM and generator identity;
stale values require regeneration. Raw outcomes and reusable calculation cache
stay in `cache/studio-teams/`. See
[Team list generation](DEVELOPMENT.md#studio-team-list-generation) for commands
and the intentionally different authored scenario conditions.

Domain code cannot import React, features or infrastructure. Infrastructure cannot import features or components. Components receive data and callbacks; they do not run the engine. Engine requests/reports use explicit contracts; dynamic expressions and unconsumed engine metadata use `unknown` with guards. The architecture regression rejects application JavaScript, untyped escape hatches, HTML injection and handwritten modules/styles longer than 350 lines. Generated JSON and universal WASM glue are outside that limit. Prefer one responsibility per file; the limit is a guard, not a target.

### Architecture ownership and boundary invariants

1. **Rust owns all domain logic**: DSL lexing, parsing, AST validation, parameter and stat projection, calculations, event simulation, timeline management, and result aggregation live exclusively in Rust.
2. **TypeScript is strictly presentation and integration**: The studio collects user inputs, passes scripts/options to WASM/Worker, receives structured diagnostics or simulation results, and renders UI components (editor, charts, tables, catalogs).
3. **No duplicate domain algorithms or test suites**: TypeScript must never implement shadow math, stat evaluators, or rotation interpreters. The duplicate language frontend and combat/mechanics suite have been removed. `content::dsl` owns source compilation and diagnostics; the WASM client only transports typed requests/results. Settings numeric input and source-preserving option edits also use Rust.
4. **Testing boundary**: JS/TS test suites cover only React component rendering, user interactions, and minimal WASM boundary smoke checks (Worker lifecycle, serialization schema, error propagation). Domain mechanics are tested exclusively in Rust (`cargo test`).

The shared Rust source compiler follows [public DSL syntax](NTE_DSL.md) and the [execution contract](../openspec/specs/script-execution/reference.md). It owns progression/additive stats, equipment binding, action lowering and authored source locations for CLI, editor/imports and Worker. Rust owns primary packet selection, action cost/recovery/cooldown/impact inputs, switch/input advances, character input eligibility and model assumptions. Builds are projected before rotation lowering, so explicit energy waits validate against the owner capacity. Generated catalogs provide display metadata; runtime policy comes from Rust, without fixture-based packet fallbacks. Public scripts expose no engine IDs, timing profiles, snapshots or objects.

The generated build catalog contains display descriptors and source hashes. Runtime action/equipment policy and per-slot research critical-roll configuration come from Rust projection. Full native action graphs, progression tables and diagnostic evidence stay outside Git; [stored-data policy](STORED_DATA_POLICY.md) owns the projection and budget. Growth arithmetic remains in the shared Rust projector and its independent native build oracle.

Editor and Worker use the native live compiler without static expansion. The Worker transports contiguous seed chunks to a Rust session and streams completed sample deltas plus Rust-owned observations. Failed executions never enter statistics. Stop destroys the Worker and retains delivered samples; resume reconstructs and verifies the prefix in Rust before continuing in a fresh instance. Retry starts a new run. Selected seeds rerun the executed live source, preserving their own action requests and occurrences. Explicit structured/scheduled APIs remain available for scenario consumers. Awakening uses sorted index arrays. Rust option edits preserve comments, spacing, loops and swaps; font/theme/wrap remain UI preferences. Bounded [browser lifecycle/performance acceptance](BROWSER_PERFORMANCE.md) records actual production measurements and application flows.

Rust owns exact parsing, configuration checking and per-trial live branch/query execution. Owner-aware authoring covers hold seconds, typed critical/parry flags, action targets, Support Skills/resources/states and shared Fons/scenario inputs. Sample playback applies native sparse state frames and their exclusive receipt prefixes on the single combat-time axis, including equal-time boundaries and rewinds. Trace/hit displays retain separate identities rather than matching diagnostic prose. HP, energy, cycles, resources, shields, targets, buffs and stamina come from recorded observations; expiry deadlines are displayed without advancing an inferred readiness/form model. Event causes and explicit occurrence controls select ranges in the executed source, including waits.

Exact sample reports use the native source export and source/native reconciliation. Rust emits JSON text and the same named CSV tables as the CLI; wide integer tokens never pass through JavaScript serialization. Reconstructed presentation observations must match the selected execution. Saved native reports can be verified separately with the native 64 MiB limit. Failed trials preserve their actual request/source/diagnostic/state and exact export without entering statistics. Paired comparisons use the native shared-seed/context/budget policy and retain incomplete pairs and raw reports. Actor/source/target DPS distributions use Rust per-trial elapsed-time rates, including victory stops. Generic Worker `INVOKE` also transports native bounded source search with exact checkpoint text and fresh-Worker continuation; no TypeScript search/ranking implementation is added. [Calibration](BROWSER_CALIBRATION.md) and [lifecycle/performance](BROWSER_PERFORMANCE.md) are accepted; final distribution remains in the [OpenSpec changes](../openspec/README.md).

Drafts use only `ntesim_config_v3`. Historical storage migrations are outside scope. Invalid drafts remain editable; inaccessible storage does not prevent execution. Import script validates text scripts and JSON containing source with a 50 MB limit. Import report restores JSON, compressed `.nter` and original report-bearing PNG files with separate 100 MB complete-file and decoded UTF-8 JSON limits (MB = 1,000,000 bytes). Bare engine requests cannot recover authored rotations/additive builds and are rejected for script import; the structured API remains separate. The Export report menu offers JSON, Compressed report and PNG with report. Exports retain the executed source/request, aggregates/samples, one raw playback report and incomplete/failure markers. PNG captures the current Results layout through the final energy panel and embeds the same complete report; share the original file because image processing may strip metadata. Imports validate before changing the draft/results and never simulate. Missing attachments, damage, unsupported formats and size failures are visible. The [Studio reference](../openspec/specs/studio-application/reference.md#portable-report-container-v1) defines the binary contract and allocation bounds.

Compiled model assumptions preserve stable IDs/descriptions through Worker/WASM and exports; they are not public scenario declarations. Execution details display the returned map. Before aggregation the Rust collector rejects failed/incomplete executions, mismatched engine/run/model identities and invalid native damage/state/energy observations. These checks also protect replay ownership. The explicit all-hit critical hypothesis uses wildcard effect eligibility so later child/off-field packets do not need an editor entity list; manually scoped effect lists remain available. Native stat projection does not establish full kit or native equipment-layout legality.

## Toolchain and build

Use Node 24 LTS and npm. From the repository root:

```sh
node tools/web.mjs install
node tools/web.mjs check
node tools/web.mjs format-check
node tools/web.mjs test
node tools/web.mjs dev
```

`install` copies the committed manifest/lock into `cache/web` and runs `npm ci --ignore-scripts`. Integrity hashes and versions come from `web/package-lock.json`. Dependencies and npm/Vite caches stay in `cache/`; the launcher resolves them without another `node_modules` directory. The runtime test configuration resolves React's JavaScript exports; strict checking resolves its declarations. `dev` serves localhost:5173. `test` first makes a fresh production build for the offline manifest regression.

```sh
node tools/web.mjs build
python -m http.server 5174 --bind 127.0.0.1 --directory target/web
node tools/web.mjs verify-trials --out target/browser-trials.json
```

Deploy `target/web`, not the source directory. Vite emits hashed entry, Worker, CSS and WASM assets. The offline plugin stages exact runtime licenses and build identities, hashes the complete output, and generates `sw.js` from `web/offline-worker.template.js`. Its manifest includes every build asset; activation retires only earlier ntesim caches. Production offline caching works on localhost or HTTPS; development does not register a Service Worker. Relative asset paths support directory-prefix deployment. Reload an already open page after the new worker activates.

The editor and Worker share the pointer-free client and NTE wire v1 decoder described in the [WASM contract](../openspec/specs/wasm-transport/reference.md#2-typed-bindings-and-binary-transport). Generated glue owns WASM memory; full batch reports preserve exact integers and provenance.

Serve `.wasm` with `Content-Type: application/wasm`. The editor and Worker instantiate the fetched Response with `WebAssembly.instantiateStreaming`, preserving the asset URL for browser debuggers. Hosts with other MIME types or runtimes without streaming fall back to bytes; those loads may retain Firefox's synthetic WASM URL and its source-map warning. HTTP and compilation failures remain visible.

`tools/build_web_names.mjs` and `tools/build_studio_catalog.mjs` produce display JSON from the frozen package. Do not regenerate frozen evidence to change display code. Builds/tests require no game installation, upstream checkout or acquisition tools. [Portable distribution](BROWSER_DISTRIBUTION.md) ships identity illustrations and original SVG icons; source-site descriptions and acquisition evidence remain excluded. The typed UI artwork map resolves canonical display IDs to bundled files without reconstructing mechanics.

The pinned gcsim v2.47.2 tree (`caf738dc807a646e2eee7e7b572f225dd6deb510`) was read as architectural evidence. Its tagged license is MIT; this refactor copies no upstream implementation. Dependency licensing remains separate from game artwork ownership; see `NOTICE` and the generated production `licenses/runtime.txt`.

## Verification boundaries

The DPS timeline covers the full declared horizon with positive-width intervals. Runs longer than 240 seconds use 240 bins; shorter runs use one-second bins with a final partial interval where needed. Bucket arithmetic must not add a zero-width trailing bin or divide by zero during collection.

Regression suites exercise readable partial S1–S5 scripts and structured engine fixtures, actual fixed/random WASM reports, streaming/Stop/retry, source-preserving option edits, packet critical policies, empirical quantiles, energy conservation, replay ownership/rewinds, React escaping, original SVG icons, persistence and generated offline inventory. Results retain the actual native quality/strict/game-DPS flags. Scheduled partial fixtures retain their incomplete flags; successful source executions disclose their bounded model hypotheses. These checks establish interface compatibility, not validated game DPS, complete adapters or legal build compilation.
