# Browser performance and application acceptance

F1-F3/H4-H6 browser acceptance uses the production Studio and its actual bundled
Worker/WASM at a directory prefix. [Lifecycle](BROWSER_LIFECYCLE.md) owns Stop,
delivered prefixes and continuation; the [application contract](../openspec/specs/studio-application/reference.md)
owns input, accessibility and offline requirements. Accepted portable
distribution/release has its own [artifact workflow](BROWSER_DISTRIBUTION.md).

## Reproduce

Use installed Playwright and Chrome. Playwright is a development measurement
dependency, outside the shipped application; the receipt records both versions.
Set `NTESIM_PLAYWRIGHT_MODULE` in your environment to the installed Playwright
module's entry point (or use the explicit environment-file runner documented in
[local configuration](DEVELOPMENT.md#local-configuration)). Then build current artifacts:

```powershell
python tools/cargo.py build -p ntesim-wasm --release --target wasm32-unknown-unknown --locked --offline
python tools/check.py -- node tools/web.mjs build
node tools/web.mjs verify-trials --browser
```

The launcher enforces the shared four-CPU affinity through `tools/cpu_limit.py`.
`NTESIM_BROWSER_CHANNEL` optionally selects another installed Chromium channel;
Chrome is the recorded default. `NTESIM_BROWSER_OUTPUT` optionally selects an output directory.
The tool replaces the summary with a nonpassing running receipt before loading the browser, records
failures, and closes its browser and local server. No public deployment occurs.

The receipt binds the current Rust source identity, WASM bytes, every production
asset, measurement tools, source workloads, browser/runtime versions and CPU
affinity. Stale WASM, changed build assets, failed/incomplete workloads and budget
failures reject. Screenshots and raw measurements stay outside Git. The existing
`verify-trials` mode without `--browser` remains the Node Worker calibration check.

Include actual browser acceptance in a checkpoint by passing
`--extra-checks` with this literal command array:

```json
[["node", "tools/web.mjs", "verify-trials", "--browser"]]
```

Then run `python tools/checkpoint.py run --extra-checks browser-checks.json`
with `NTESIM_PLAYWRIGHT_MODULE` set. The ordinary checkpoint still retains the
complete current-artifact native/WASM/Worker parity and web regression boundary.

## Measurement boundaries and budgets

Ordinary workloads include real UI Run, Worker loading/execution, streamed
messages and completed rendering. The paired comparison also includes opening
its exact native JSON export. The bounded native search runs through `INVOKE`
in the actual production Worker: two authored refinements, two seeds, a two-trial
checkpoint and continuation in a fresh Worker. Rust owns eligibility, budgets,
ranking and checkpoint verification. This transport does not add a search UI.

Browser CPU is the change in Chrome's cumulative CPU time across its own process
tree. Retained memory is measured after explicit V8 collection in the page and
Worker: used JS heap, backing stores and queried live `WebAssembly.Memory`
buffers. Their conservative sum is not process RSS or peak residency; it does
not measure every native browser allocation. Worker JSON bytes and encoding time
come from an extra measured encoding pass on received messages; they do not
isolate structured-clone cost. Main-to-Worker synchronous `postMessage` time is
recorded separately. Compact UI retention bytes and exact native report/checkpoint
text bytes have separate fields.

Each measured workload must finish within 30 seconds wall and browser CPU,
retain at most 512 MiB of measured JS/WASM memory, and serialize at most 64 MiB.
Ordinary workloads process at most 100,000 events. Search additionally retains
the native authored 10,000 events/run, 100,000 total events and 32 MiB receipt
budgets. These are explicit practical host budgets, not automatically calibrated
thresholds or guarantees for arbitrary iterations, source, hardware or horizons.
Stop latency is measured from the available Stop control to the visible stopped
state. Resume active time includes reconstruction; stopped idle time is excluded.

## Recorded production run

Measured 2026-11-07 on Windows 11 build 26300, AMD Ryzen 5 9600X, four inherited
logical CPUs (0-3), Node 24.19.0, Playwright 1.62.1 and Chrome 156.0.8078.81.
WASM uses the pinned Rust release build. The retained receipt and three mobile
screenshots are recorded in the local acceptance workspace.

- Engine source: `932fba6d6bfc3d4f1fa5f876af4dfa7f6106dd951991d26fd0e79ff395a7e452`.
- WASM: `5f50683574d8ca9c753e0bf8b54a00d3972f58d3648ea03826e54d1b434ef250`.
- Production index: `7d26805e171fcfb58494dddffcddaca015c0652cd150ffd73ea3813cb1a52cd7`.

| Workload | Trials | Events | Wall ms | CPU s | Retained MiB | Serialized MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Short random live source | 1000 | 5000 | 1289 | 2.39 | 21.9 | 0.24 |
| [Mint](../examples/mint-native-cycles.nte), two continuing periods/seed | 3 | 1509 | 304 | 0.57 | 54.1 | 5.94 |
| [Daffodil/Zero](../examples/baselines/daffodil.nte), two continuing periods/two dummies | 3 | 3738 | 812 | 1.69 | 114.2 | 16.50 |
| [Incoming shields](../examples/incoming-shield-cycles.nte), named off-field delivery | 3 | 174 | 95 | 0.24 | 75.2 | 0.46 |
| Mint paired comparison and exact export | 4 | 2012 | 5239 | 6.15 | 270.3 | 22.89 |
| Native refinement search with fresh Worker continuation | 4 | 2012 | 1095 | 1.39 | 344.0 | 22.88 |

The search checkpoint is 12,331,182 bytes; its full native JSON report is
23,986,358 bytes. The larger retained search footprint includes the currently
displayed Studio result/comparison and exact checkpoint/report strings. The
1000-trial source delivered 14 Worker messages, 770,855 encoded bytes and 1.8 ms
of additional JSON encoding. Daffodil delivered 26,131,290 bytes across five
messages, with 51.7 ms encoding; repeating the first report has a material
transport cost. No constant-memory or arbitrary-size claim follows.

Stop became visible in 22.8 ms, retaining one delivered trial. Resume completed
the same 1000 samples/statistics in 1378 ms end-to-end, with 1170.6 ms active
Worker work including reconstruction. The edited draft did not change the
executed source, and incompatible artifacts remained failed rather than stopped.

## Exact-throughput startup and execution review

The original measurements below use the former fractional-hit-sum statistics.
Current primary Damage sums each hit's existing rounded damage as checked
integers, includes overkill, and keeps applied HP loss separate. Contributions
and time bins use the same basis; DPS remains total damage divided by elapsed
seconds, without rounding the rate. Detailed formula fractions remain available.
The throughput change retains original, quality-repaired, rounded-policy-only
and optimized references separately so intended score changes cannot be counted
as exact parity or hidden inside a performance claim.

The synchronized fractional reference at `e31e3eed` is retained separately
under `cache/exact-throughput/synced-main-artifacts/`. Cold, repeat, comment
edit and Worker replacement produced first renders at 1,323, 600, 718 and
879 ms and completed rendering in 31.13, 30.16, 30.10 and 30.48 s, respectively.
The full 1,000-seed rounded-only reference matches across native, sequential
WASM and four compute Workers, including reactions and both field-time bases.
The per-seed mechanics digests match the synchronized fractional reference.
Independent sums of the original rounded hits account for the mean relative
DPS change from 37,840.72294094617 to 37,840.583050000045.

Successful batch trials accumulate observations at event boundaries. They keep
the hit ledger used by parent-dependent callbacks and the live state used by
admission, while the collector receives contributions, energy, field time,
healing and shield statistics. Replay and failed-seed inspection use detailed
reports. Final medians across three full repetitions were 37.28 s native and
54.74 s sequential WASM, from 67.46 and 74.17 s on synchronized main. Each
route matched the rounded reference. Median sequential WASM process peak
was 182.8 MiB, including Node runtime and raw receipt output.

The final production browser runs retained about 199–201 MiB across the
four lifecycles and completed rendering in 19.46–20.41 s. First render took
1,078 ms cold, 539 ms repeated, 613 ms after a comment edit and 742 ms after
Worker replacement. The repeated compile round trip fell from 36.3 to 6.2 ms
with validated preparation reuse. Ordered collection overlapped computation.
Warm-cache integrity, replaced Worker loading and resumed samples are covered
by the source-trial, WASM loading and Worker integration checks.

A repeat on the final transport modules matched all 1,000 samples and all
aggregate statistics in each lifecycle. It completed rendering in 19.67–20.75 s
with a 19.75 s median, and rendered the cold first sample in 1,095 ms.

In the matched Node harness, one, two and four compute Workers took 55.19,
33.05 and 21.01 s, with one additional aggregator. Affinity selects four
logical CPUs across two physical SMT cores. From one to four compute Workers,
wall improved 2.63× while process CPU rose from 57.20 to 79.79 s. The final
four-worker finish spread was 88–151 ms. Each compute instance retained about
44.7 MiB of WASM memory. The harness sends all observations together to the
collector and includes raw output in its process peak. Browser transport
commits incrementally and has a separate memory measurement.

The original Zankou Insight workload (unchanged maintained source, seeds
42–1041, 120 relative seconds) is retained under
`cache/exact-throughput/original/`, including production assets and raw browser
receipts. Engine `570bc632…3303` and WASM `254bbfe5…5184` use the original
fractional damage-total policy. Chrome 157.0.8103.51 shared logical CPUs 0–3.
Its default pool contained eight Workers, including one aggregator.

| Run | First received sample / ms | First rendered sample / ms | Completion / s | Browser CPU / s | Retained JS/WASM / MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cold | 1,068.0 | 1,195.0 | 30.449 | 119.39 | 233.55 |
| Repeated | 531.9 | 627.9 | 29.553 | 116.24 | 233.71 |
| Comment edit | 627.4 | 733.8 | 29.743 | 116.55 | 233.97 |
| Replaced pool | 881.0 | 990.3 | 30.187 | 117.60 | 232.61 |

This explicit long benchmark processes 3,073,000 events, outside ordinary
acceptance's unchanged event/time budgets. It does not certify those checks.
First progress appeared within 3.2–5.2 ms. Whole compile-request round trips
were 33.5–96.3 ms; they are not parser timings. Native diagnostic spans instead
measured two parses at 0.390 ms combined, eight build projections at 31.649 ms
and 54 eager descriptor attempts at 40.880 ms across compile/reconstruction
and one sample (nested regions). Retaining validated preparation and closing
warm-cache integrity gaps are the first startup candidates. Independent export
JSON and collector construction were below 0.1 ms each on native.

The separate Worker-adapter receipt measures WASM compilation at 14.9–55.8 ms
per instance and instantiation at 0.35–0.59 ms. Diagnostic Chrome traces exposed
JavaScript module compilation, not a separate WASM phase; no full-compilation
count is inferred from a UI label. Compare actual default-pool behavior
separately from matched one/two/four compute Workers plus an aggregator.
The [development measurement commands](DEVELOPMENT.md#opt-in-throughput-measurements)
retain all raw repetitions, artifact identities and process-resource limits.

Build measurements are separate: a matching native benchmark no-op took
0.49 s, current WASM compilation 177.08 s, wasm-bindgen 0.636 s and web
bundling/staging 6.044 s. wasm-opt was unavailable. Cargo's final native
binary unit includes LTO/backend/link work and cannot establish linker-only
time. These costs do not count toward simulation speedups.

## Application checks

The same production run verifies keyboard tab navigation, focus restoration and
Escape with a nonempty search field, search insertion choices, focus tooltips,
keyboard playback seeking, visible focus outlines, reduced motion and 390px
Simulator/Results/Sample layouts. Primary control/status text contrast is at
least 6.89:1 in the measured dark theme. This bounded flow check does not claim
a complete accessibility certification across every palette or assistive tool.

Oversized script imports reject at the 50 MB boundary; report imports have independent 100 MB file and decoded-JSON limits. Invalid JSON shapes remain
visible errors, supported source JSON imports compile, and invalid drafts survive
reload with Run blocked. An inaccessible storage getter permits editing and real
execution. Corrupt WASM produces visible diagnostics and prevents Run.

A real Service Worker installs 93 cached URLs under `/studio/`, including the
Worker, WASM, CSS, artwork and runtime notices. A controlled cache-version update
waits for activation, preserves an unrelated cache and retires the old ntesim
cache. Offline cold reload, fetching every cached asset and actual Worker execution
then succeed without network. Current portable archives and release inventories
are accepted separately in [browser distribution](BROWSER_DISTRIBUTION.md), which
excludes the artwork present in this earlier measured build. Publication still
requires explicit authorization.


The relative-combat-clock acceptance also uses `node tools/browser_acceptance.mjs --clock-performance` through the shared check runner. It measures a no-Ultimate control, ordinary multi-character Ultimates and continuing Hotori, with eight trials each. Each receipt retains source hashes, actual packet/trace mix through the matching absolute120-second prefix and the complete endpoint, raw/compact/compressed report sizes, timer/buff metadata counts and30 absolute seeks. This separates additional or suppressed combat work from elapsed-clock bookkeeping rather than interpreting every timing difference as overhead.

For the before measurement, retain the existing production site, matching WASM and hash-bearing artifact manifest before rebuilding. `--clock-baseline` runs those archived production assets and records their original engine identity and report fields only inside the measurement boundary. It does not import obsolete reports into the current application. Raw measurements and screenshots stay ignored; the ordinary current browser acceptance remains required for delivery.

## Relative-clock acceptance, 2026-11-15

The paired receipts are recorded in the local measurement workspace. Each records its exact
engine, WASM, production assets, tools, browser and workload source hashes,
alongside wall/CPU time, retained memory, raw/compact/compressed report bytes,
packet/trace mix and thirty absolute seeks. The accepted receipt binds the final
checked implementation, including absolute declaration bounds. Both use the
same four-CPU Windows/Chrome measurement boundary described above.

Each workload completes eight trials at exactly 120 relative seconds. The
selected sample preserves the following work and state-boundary observations:

| Workload | Events before / after | Absolute endpoint after | Hits through absolute 120s, before / after | Sample boundaries before / after |
| --- | ---: | ---: | ---: | ---: |
| No Ultimate | 216 / 216 | 120.000s | 17 / 17 | 29 / 29 |
| Four ordinary Ultimates | 352 / 392 | 131.306s | 25 / 25 | 46 / 46 |
| Continuing Hotori | 1296 / 1304 | 143.670s | 81 / 81 | 162 / 162 |

Hotori's absolute 120-second prefix has one Charge energy callback rather than
five, and 19 Hathor Gaze callbacks rather than 24. Suppressed Charge grants and
frozen World progress change work independently of the longer endpoint. The
ordinary workload adds lifecycle events. The no-Ultimate control preserves its
event mix, allowing a separate view of clock bookkeeping. These bounded
measurements include timing noise and do not establish a universal overhead
percentage; the receipts retain the actual costs of each workload.

The ordinary sample interns seven timer definitions. Sample boundary counts
remain unchanged, without a timer-only frame series; compact batches omit
detailed timer catalogs and pause histories. All workloads meet the existing
30-second wall/CPU, 512-MiB retained-memory, 64-MiB serialization and 100,000-event
budgets. Full acceptance also verifies JSON/compressed/PNG restoration, both
duration distributions, Absolute Dur raster capture, keyboard navigation/seek,
pause shading and desktop/390px layouts. These checks verify the selected model
and host budgets; the mechanics references retain native-fidelity limits for
the disclosed pause assumptions.

## Paired report summary basis acceptance, 2026-11-15

The post-change performance measurement repeats the three retained workloads under the identical four-CPU Windows/Chrome measurement runner (`node tools/browser_acceptance.mjs --clock-performance`), comparing against the measurement baseline.

| Workload | Trials | Events (base / post) | Absolute endpoint | Relative endpoint | Hits / Damage (base / post) | Compact JSON bytes (base / post) | Compressed report bytes (base / post) | Wall ms (base / post) | CPU s (base / post) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| No Ultimate | 8 | 216 / 216 | 120.000s | 120.000s | 17 / 2,180 | 164,354 / 167,772 (+2.0%) | 13,915 / 13,996 (+0.6%) | 264.1 / 269.0 | 0.74 / 0.75 |
| Four ordinary Ultimates | 8 | 392 / 392 | 131.306s | 120.000s | 25 / 18,935 | 216,960 / 223,640 (+3.0%) | 20,453 / 21,096 (+3.1%) | 157.1 / 167.5 | 0.42 / 0.42 |
| Continuing Hotori | 8 | 1304 / 1304 | 143.670s | 120.000s | 81 / 78,886 | 374,799 / 382,293 (+2.0%) | 30,247 / 30,857 (+2.0%) | 171.6 / 181.6 | 0.45 / 0.46 |

Combat totals, clock endpoints, event/hit/frame counts, and absolute timeline bounds remain identical across all seeds. Growth is strictly limited to paired summary scalars and statistics (`TimeBasisPair<T>`) for DPS, EPS, HPS, SHP, and actor/source/target contribution DPS; no additional stream, event, frame, or timeline was introduced. All workloads remain well within the 30-second wall/CPU, 512-MiB memory, and 64-MiB serialization limits.
