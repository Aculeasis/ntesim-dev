# Browser application contract

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

[Web architecture](../../../docs/WEB_ARCHITECTURE.md) owns module/build boundaries; [NTE DSL](../../../docs/NTE_DSL.md) owns syntax. Studio runs the shared live Rust source executor in a Worker. Rust owns trial observations and quality; display catalogs and charts cannot upgrade the engine's evidence assessment.

Team cards and editor diagnostics consume the shared [Rust script API](../script-inspection/reference.md); localized display metadata is separate from canonical party/stat output. [Development](../../../docs/DEVELOPMENT.md#choosing-checks) defines the applicable browser checks.

## Inputs and views

- Configuration is the single source of truth. Settings edits effective source declarations, preserves unrelated text and verifies the resulting values. Syntax/compile errors carry line/column and block Run; no hidden actor fixture defaults.
- Simulator shows the draft; Results and Sample bind the executed source/request and delivered observations. Editing the draft never alters an executed result. Team cards expose recorded total/additive stats; snapshots cannot fabricate equipment-only decomposition.
- Native elements, names and packet labels are presentation metadata. English fallback labels and readable event templates retain raw identifiers/receipts in exports. Unknown observations are unavailable, not zero.
- Keyboard/focus/hover card details, editor search, responsive layout and reduced-motion behavior remain usable across Simulator/Results/Sample.

## Worker and trial lifecycle

Commands/responses use discriminated unions in web/src/infrastructure/worker/protocol.ts, with task IDs and sample offsets. Obsolete messages are ignored; invalid offsets and callback failures become visible failures.

Positive exact iteration counts have no application upper cap; default 1000. Initial/final seed arithmetic must remain a JavaScript safe integer. The default controller has no elapsed-time cutoff; explicit API timeouts are opt-in. Stop terminates the Worker and retains delivered work. Resume reconstructs and verifies that prefix through Rust in a fresh Worker before running the remaining seeds; it uses the executed source even when the editor draft has changed. Retry starts a new run. Only typed user cancellation is labelled Stopped; errors retain their actual cause. Messages and fatal errors from replaced Workers cannot affect the current task. See [browser lifecycle](../../../docs/BROWSER_LIFECYCLE.md).

Trial collection checks completion/invariants before aggregation. Seeds are contiguous; fixed mode is non-critical deterministic settlement. Random mode requires explicit research eligibility/roll/actor-stream hypotheses, including emitted child packets. More fixed seeds cannot manufacture variance.

Updates represent completed work, normally after the first sample, about 250ms of work and the final sample; only SUCCESS completes a task. Samples/observations stay in memory. No arbitrary-size or constant-memory guarantee is claimed.

## Observations and playback

Damage/accepted actions/field time and actor/source/target/time contributions derive from actual receipts. Damage belongs to the hit owner even off field. Empirical frequencies/mean/quantiles/box plots preserve discrete or constant outcomes; no fitted uncertainty distribution is fabricated.

Primary Damage and every damage contribution/bin sum each hit's already-rounded
damage with checked integer accumulation before conversion for statistics.
Damage includes overkill; HP loss remains separately capped by remaining target
HP. Both DPS bases divide this common total by actual elapsed seconds without
integer rounding of the rate. Exact sample diagnostics retain raw formula
fractions alongside the rounded hit and applied loss. Current exports and replay
use the same primary total, including amounts above JavaScript's safe integers.

Whole-run team, actor, source and target DPS use actual relative elapsed seconds by default, or actual absolute elapsed seconds when Absolute is selected. Native statistics summarize per-seed rates before aggregation; a seed with zero relative duration makes the full set's relative DPS unavailable, independently of raw combat completion. Damage and both duration distributions remain inspectable. The shared Relative / Absolute buttons in Damage Per Second (DPS) and Field Time Distribution select the summary basis for Dur, DPS, EPS, HPS, SHP, applicable histograms, actor/source/target DPS contributions and field time together. Selection is run-scoped, defaults to Relative on fresh/imported runs, persists across tab navigation and chunk delivery, and operates without engine requests or explanatory captions/tooltips. Canonical exact sample DPS, comparison eligibility and Team list rankings remain strictly relative.

Results show compact DPS, retained EPS, damage, gross HPS, effective shield HP (SHP) and duration summaries. Rust collects gross, applied and excess healing once from canonical healing receipts, including autonomous heals. Gross HPS divides gross healing (including overheal) by each trial's actual elapsed seconds in the selected basis (`relative_elapsed_us` or `absolute_elapsed_us / 1_000_000`) in HP/s. SHP averages remaining absorbable shield capacity on the active living actor over the completed trial, weighted by the selected clock's advancing time ($\int C(a)\,da$ vs $\int C(a)\,dR(a)$), in HP; distinct concurrent shields are additive following NTE settlement, replacement uses updated instance HP, switches restrict capacity to the currently active actor, defeated actors contribute zero, and idle intervals clip strictly at exact native expiry times. Shield absorption (`shield_absorbed`) is retained as a separate actual-damage total summing explicit shield absorption on incoming deliveries and redirected children, excluding immunity, other interception and unused shield capacity; it does not determine SHP. Zero activity with positive duration produces zero metrics; zero denominator leaves relative rates/SHP unavailable. HPS and SHP histograms and exports consume the same native per-trial values (`hps`, `shp`).

The timeline panel is titled `Damage Timeline (absolute time)` with options `Damage per second` and `Cumulative Damage`. Both span the report width on an explicitly absolute combat-seconds axis, using the largest completed native absolute endpoint for either stop policy, independently of the DPS-card basis selector. Damage at absolute 135 remains at 135 in a 120-relative-second run. Native aligned buckets start at one second and double width as needed to stay within 240 buckets; merging chunks promotes to the coarsest delivered dyadic grid without moving tail damage into the requested relative horizon. Shorter seeds contribute zero later bucket damage and retain their completed cumulative totals. Local bucket rates are damage per absolute second, separately labeled from whole-run DPS, with native statistics available on hover/focus. Damage shows mean/min/max and a mean ± standard-deviation band clipped to the observed range; cumulative damage shows min/max and p25/p50/p75 curves. Contribution pies precede ranked character/source bars, paired actions/field time and energy panels. Field Time Distribution displays each character's mean seconds as the active character across completed trials and their percentage share of the sum of those means on the selected basis, formatting native seconds and shares in inspection tooltips and legend rows and rendering an empty chart state when total field time on the selected basis is zero. Character colors follow actor slots and Source DPS shows a legend matching its owner filter. Native Scorch DoT, Blossom pistils, Nova and Hexed follow-ups have distinct source labels; summon damage retains its owner and effect identity. Character-element grouping describes character ownership rather than the element of every hit. The Source DPS and Source Damage Instances panel features a title-adjacent mode toggle between detailed (`full`, default) and compact (`comp`) views. In `comp`, rows group by owner slot and ability ID into effect-free ability totals (with shared reaction damage labeled `Reactions`), displaying only the mean for the selected metric and basis without interval bounds, error bars or effect suffixes. Source mode selection is run-scoped, defaults to `full` on new or imported runs, and is retained across Type, Character, DPS basis and result chunk updates. Folding applies based on the displayed row count (top 15 when 20 or more grouped or detailed rows). The NTE palette is Anima green, Psyche blue, Incantation red/pink, Chaos purple, Lakshana yellow and Cosmos pale white. Narrow screens stack chart panels. Run/Settings/Tools controls remain in document flow outside the editor.

Target cards show level, current/maximum HP, DEF and six named element-icon resistance entries. Explicit element resistance overrides replace the base value for that element; omitted elements use base resistance. Desktop overview panels fill the same row, with compact target contents and outcome histogram. Targets remain anonymous stationary dummies without position geometry.

Energy separates gross, retained and overflow, reconciling final ledgers. Initial energy is not generated energy; final energy is independent of the selected source view. Undeclared/zero intensity rows are omitted; declared display values cannot imply native intensity execution.

Energy-source rows group the same native source and event kind, with stacked mean grants by recipient. Gross/retained/overflow selections retain their units and reconcile to the displayed totals; grouped means do not claim combined uncertainty bounds. Detailed individual trial events and per-seed exports remain available in the Sample view.

Playback validates `buff_catalog`, `buff_diff`, sparse `ActorStateDelta` folding, grouped `energy_fanout` grants, and final/idle-gap reconciliation via `validateStateTimeline` and `validateEnergy`, and applies recorded native state frames at microsecond timestamps and equal-time settlement boundaries, with exclusive hit/healing/trace prefixes and reversible event stepping (`StatePanel` resolves `ActiveBuffRef` entries against `buff_catalog` for inspection). Event titles, actors, and tooltips in `Localization.event` and `EventLog` are formatted purely from structured trace refs, payloads, and receipt catalogs without parsing trace prose. State is atomic within each recorded frame; individual trace rows do not establish intermediate callback state. Portrait cues illustrate events, not measured native animations/geometry. Missing observations require engine receipts before display. Recorded deadlines do not imply an inferred cooldown/readiness model. Applicable simplifications/omissions from the [DPS relevance policy](../product-scope/reference.md#dps-relevance-and-modeling-boundary) remain visible in reports and exports.

## Persistence, files and quality

Only ntesim_config_v3 stores drafts/presets; no historical key migration. Invalid drafts remain editable; storage failure does not prevent execution. Import script accepts text scripts and JSON containing source, up to 50,000,000 bytes. Import report accepts JSON, `.nter` and PNG carrying a report, up to 100,000,000 bytes for the entire file and independently for decoded UTF-8 JSON. MB means 1,000,000 bytes; exactly each limit is allowed. Reports validate structure and source compilation before atomically replacing the draft and results, without starting a simulation. Exports preserve executed source/request, aggregate/samples, single raw report (`report`), and failure/cancellation context without duplicating a `selected_sample` copy.

Export report opens an adaptive keyboard-operable menu: JSON, Compressed report and PNG with report. Selecting a format downloads a timestamped `YYYY-MM-DD_HH-mm-ss-report.json`, `.nter` or `.png`. JSON remains pretty printed; compressed forms encode the same bundle as compact UTF-8 JSON, preserving exact strings and wide integer digits through the lossless importer. Later draft edits do not change execution identity. PNG requires stable delivered results; active simulation/sample work explains its disabled entry. Preparation errors remain visible and do not download incomplete files.

The PNG captures the current Results region from team cards through **Character Energy at End of Simulation**, inclusive, at the displayed responsive width and one raster pixel per CSS pixel. It retains controls and expansion states and includes off-screen height. Comparison and Run details remain outside. Capture waits for fonts, artwork and stable layout; dimensions must be positive safe integers, each edge at most 32,767 pixels, and area at most 32,000,000 pixels. A changed run or an unsupported/failed canvas aborts capture. The final image plus attachment must satisfy the file limit. Share the original PNG as a file: image editors and sharing services may strip its report metadata.

### Portable report container v1

All multi-byte container integers are unsigned little-endian. The standalone `.nter` and PNG attachment contain identical container bytes:

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 8 | ASCII `NTESIMRP` |
| 8 | 1 | Version `1` |
| 9 | 1 | Codec `1`: Zstandard, encoded at level 9 |
| 10 | 2 | Reserved flags, zero |
| 12 | 4 | Decoded UTF-8 JSON byte length, nonzero and at most 100,000,000 |
| 16 | 32 | SHA-256 of decoded JSON bytes (integrity, not authenticity) |
| 48 | Remaining | Exactly one ordinary Zstandard frame |

The complete frame is preflighted before initializing the codec: known content size must equal the container length; content size and advertised window must each be at most 100,000,000 bytes. Wide fields are checked as `BigInt`. Dictionary encodings, reserved encodings, unknown sizes, skippable/concatenated frames, malformed blocks/checksum framing and trailing bytes are rejected. The pinned decoder allocates the checked content size and passes that fixed destination capacity to decompression; false small declarations fail within it. There is no unknown-size fallback or growth/retry loop. Actual decoded length, SHA-256, strict UTF-8 and the current report structure must then pass. A dedicated module Worker serializes codec operations, transfers buffers and terminates after completion, error or cancellation; simulation Workers are separate. These byte limits are not a total tab-memory guarantee.

PNG uses exactly one private ancillary, safe-to-copy **`ntEr`** chunk immediately before `IEND`. Its payload is the complete container, without Base64 or another compression layer. Standard PNG big-endian chunk lengths and CRC-32 over chunk type plus payload apply. Import scans all bounded chunk views and validates signature, structure, CRCs, one attachment and final `IEND` without trailing bytes. It never decodes image pixels or unrelated compressed metadata. Absent attachment, duplicate/corrupt/truncated chunks, unsupported container versions/codecs/flags, damaged frames/hash/UTF-8/JSON, missing report data and file/decoded limits have distinct readable failures. Failures preserve the current draft/results and allow retrying the same file.

Team list replaces the editor's Load configuration selector and opens from the
center of the navigation without changing the underlying view. Simulator,
Results and Sample remain separate view tabs grouped at the left. Each row
shows the scenario label, four ordered compact slots (including empty slots for
short rosters), awakening, Arc/refinement and cartridge/count icons, and native
mean scenario DPS. Keyboard-focusable equipment inspection exposes names and
equipment levels. Rows have no Details expander or calculation report. The
selectable collection contains twenty-two complete four-character teams plus the
unchanged default S5 control. Partial S1/S2/S3/Baicang examples remain outside
selection. The [curation reference](../../../presets/studio/teams.md) owns dated
sources, the shared 1,000-iteration/120-second profile, exact fifteen-line console
allocations, investments, rotations and model limits. Required rosters include Nanally/Hotori/Zero/Jiuyuan,
Zankou/Iroi/Sakiri/Daffodill with Daffodill nodes `[3,4,5]`, and Shinku/Akane Rin/Elyms/Iroi.
Failed or incomplete calculations show unavailable DPS with a
reason after the ranked numeric entries; valid zero stays numeric zero.

The centered search requires every case-insensitive name term to match an actual
team member. Include requires every selected character; Exclude rejects any
selected character. Both portrait pickers have independent counters, selection
toggles and Clear All. Search and filters combine and preserve DPS order; a
character selected in both sets produces an explicit no-results state. Query and
selections survive dialog reopening during the current app session. Outside
click dismisses a picker; Escape closes a picker first and the dialog on a later
press. The dialog keeps its header visible while rows scroll, adapts to 390-pixel
screens and restores focus to its opener when dismissed.

Selecting a row loads its complete bundled script into Simulator without
auto-running or replacing existing executed results. While a simulation or sample
execution is active, rows remain inspectable and loading is disabled. Catalog
opening, filtering and loading do not execute simulations. DPS is precomputed
with the shared Rust engine using each unchanged script's authored conditions;
build checks reject stale source, engine or generator identity. The
[generation workflow](../../../docs/DEVELOPMENT.md#studio-team-list-generation)
owns regeneration and local cache details.

Incomplete/hypothetical status survives every chart/view/export. Native errors cannot become successful trials. Failed source trials retain their actual requests, diagnostics, state and exact report exports even when no seed completes. Exact sample CSV/JSON and shared-seed comparisons use the native CLI owners; actor/source/target DPS statistics use each seed's actual relative elapsed time. Final [browser calibration](../../../docs/BROWSER_CALIBRATION.md) and bounded [lifecycle/performance acceptance](../../../docs/BROWSER_PERFORMANCE.md) are accepted; distribution remains in the [OpenSpec changes](../../README.md).

Production offline inventory includes app, Worker, CSS, WASM, artwork and runtime licenses. Activation touches only ntesim caches; reload after a new build activates. Development does not register a Service Worker. Existing parity, trial, lifecycle/replay and offline tests establish model/UI compatibility, not strict game correctness.

The Sim Duration (Dur) card has an accessible Relative / Absolute selector. Each new run defaults to Relative; selection remains local to that run across view navigation and switches mean, min, max, standard deviation and all three quartiles together without another engine request. Other metrics retain their native denominators. A seed set containing an unavailable relative rate displays unavailable whole-run DPS and contribution rates rather than selecting only the defined seeds.

Sample retains absolute geometry, speed, seek and event stepping, including distinct events during a relative plateau. It displays labeled current/final absolute and relative times, shades native pause intervals and indexes their union for arbitrary seeks. Buff definitions declare their native clock basis; inspection subtracts the corresponding clock coordinate from their semantic expiry. Damage Timeline uses native aggregate absolute extent for both stop policies and labels bucket rates per absolute second separately from relative whole-run DPS. Embedded report data retains both duration distributions regardless of the selected card display.
