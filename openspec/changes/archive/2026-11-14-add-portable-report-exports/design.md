# Design

## Context

See [proposal.md](proposal.md) for motivation and [the delta](specs/studio-application/spec.md) for behavior. `App.tsx` currently downloads `serializeReportBundle(run)` directly. `files.ts` accepts only `.json` reports, uses `parseLosslessJson`, and checks the same 50,000,000-byte limit for script and report imports. The executed run is already distinct from the editor draft.

`ResultsPage` renders character cards, summaries, target/distribution and damage panels, and `EnergyPanels`, followed by `ComparisonPanel` and Run details. Charts use SVG or HTML; character artwork and CSS are bundled. The existing Tools dropdown supplies placement and dismissal behavior. Existing file/report tests and `tools/browser_acceptance.mjs` cover the relevant storage and browser boundaries; its `exported` helper assumes the current immediate-download button and must change with that contract.

The production build uses cached packages under `cache/web/node_modules`, Vite, a generated offline inventory and runtime-license staging. The new assets must work from an arbitrary checkout and deployment base path, including offline operation.

## Goals / Non-Goals

**Goals:** one report representation across three file forms; bounded binary decoding; faithful capture of the current Results presentation; atomic import; asynchronous codec work and explicit error cleanup.

**Non-Goals:** changes to combat mechanics, engine report schemas, native CLI exports or existing simulation Worker messages; JPEG/WebP/PDF export; a new report viewer; recovery of data stripped from an image; new compression settings or a user-visible codec selector.

## Decisions

### 1. Keep the report bundle and use Zstd-9

Retain the existing pretty-printed JSON download. Compressed and PNG exports serialize the same bundle without indentation and encode it as UTF-8. Never parse and reserialize JSON through ordinary JavaScript numbers to compact it. The lossless importer remains the JSON entry point, and exact rationals, wide values, source identity, assumptions and partial/failure fields stay intact.

Use pinned `@bokuweb/zstd-wasm` 0.0.27, the package measured during exploration, with compression level 9. For the 1,000-trial S5 and Zankou reports, compact JSON compressed to approximately 124 KB and 167 KB. Zankou's warm compression/decompression medians were 19.9/1.1 ms in that package under Node/V8; Brotli-11 yielded 141 KB but required 2.82 seconds to compress. These exploratory measurements guide the choice, not a browser performance guarantee. A native gzip-6 baseline produced 238 KB for compact Zankou JSON. No benchmark suite or persistent performance ledger is introduced.

Alternatives: gzip does not meet the chosen size/speed balance; Brotli-11 prioritizes a small additional size saving over interactive speed; Zstd-19 costs substantially more time and memory. A binary replacement for JSON would introduce unnecessary schema and exact-value conversion risks.

### 2. A small, explicit standalone container

Use `.nter` for the compressed report, avoiding collision with `.nte` scripts, with timestamped names ending in `-report.nter`. JSON retains `-report.json`; PNG uses `-report.png`. The v1 binary layout is:

| Offset | Bytes | Meaning |
|---|---:|---|
| 0 | 8 | ASCII signature `NTESIMRP` |
| 8 | 1 | Container version, `1` |
| 9 | 1 | Codec, `1` = Zstandard |
| 10 | 2 | Reserved flags, zero |
| 12 | 4 | Decoded UTF-8 JSON length, unsigned little-endian |
| 16 | 32 | SHA-256 of the decoded JSON bytes |
| 48 | remainder | Exactly one ordinary Zstd frame |

Require a known frame content size matching the container length, no external dictionary, no concatenated/skippable frames and no trailing bytes. The hash detects damage; it does not establish authenticity. Use the same complete container bytes as the PNG attachment. Header/version checks, frame validation and errors belong to transport helpers, not Rust combat logic. Document this public format in the existing Studio detailed reference during implementation.

### 3. Enforce two 100 MB limits before trusting metadata

Use named report constants of 100,000,000 bytes for both the entire input/output file and decoded UTF-8 JSON; keep the script-import constant at 50,000,000. The PNG file limit includes image data, attachment and chunk overhead. Exactly the limit is allowed; one byte beyond it is rejected. The JSON limit counts bytes, not UTF-16 string length, and applies to the JSON representation actually stored in that format.

The decode path is:

1. Check `File.size` before `text()`/`arrayBuffer()`. Identify the supported file signature and validate actual loaded byte length. A filename alone never establishes a binary format.
2. For PNG, walk chunk boundaries using checked arithmetic and bounded views, validate structural framing/CRCs, and extract exactly one report chunk. Do not decompress PNG pixels or unrelated compressed metadata. Reject truncated lengths, duplicate attachments, invalid end markers and trailing data.
3. Check the container's version, flags, codec and decoded length before initializing the codec. Zero, oversized or inconsistent lengths are invalid.
4. Preflight the complete single Zstd frame with checked offsets and `BigInt` for 64-bit metadata. Require an explicitly known content size equal to the checked container length. Reject external dictionaries, reserved encodings, an excessive advertised window, additional frames and incomplete block/checksum framing. The maximum admitted window is 100,000,000 bytes; do not convert a wide size to a JavaScript number before bounding it.
5. Only then call the pinned decoder. Its simple decoder allocates output according to the frame content size and passes that capacity to `ZSTD_decompress`. Because that size has already been validated and matched, the destination cannot exceed the declared length or 100 MB. A false small declaration fails in the decoder when output would exceed capacity. Never use the unknown-size fallback, automatic buffer-growth retries or post-decompression size checks as the protection mechanism.
6. Verify actual decoded length, SHA-256, strict UTF-8 and the supported report structure before normalizing source and loading state. Output is never exposed to the UI until all stages succeed.

This is bounded one-shot decompression rather than streaming: the current encoder always writes a known content size, and a fixed validated destination gives the same hard output bound without another decoder API. The [Zstd API](https://github.com/facebook/zstd/blob/v1.5.7/lib/zstd.h) defines destination capacity as an upper bound. The [package's decoder](https://github.com/bokuweb/zstd-wasm/blob/master/lib/simple/decompress.ts) otherwise trusts the frame size, which is why an application-level preflight is mandatory. Confirm the installed locked package matches this allocation behavior in the focused tests.

The 100 MB values bound file/report bytes, not total tab memory. UTF-8 decoding, JSON objects, source compilation and image capture add overhead. Avoid unnecessary copies and allow allocation failures to become readable errors.

### 4. Isolate codec work and commit imports atomically

Add a dedicated report-codec module Worker with a small typed encode/decode request/response boundary. Lazily load the codec and transfer owned buffers instead of copying them through messages. Serialize operations; terminate the Worker after completion, failure or cancellation so its grown WASM heap is released. Ignore stale task replies. This Worker does not occupy or replace a simulation pool Worker and does not change its protocol.

After binary decoding, route all formats through one report-bundle validator and the existing lossless JSON/source normalization path. Validate required bundle structure before calling the existing UI load handlers. Keep failure and cancellation data; do not reconstruct successful aggregates from absent fields. Import script remains a separate action. Failed imports must not call `config.setSource` or `run.loadReport`.

Use distinct readable errors for file size, decoded size, unsupported format/version, damaged container/frame/checksum, absent PNG attachment and missing/invalid simulation report data. Clear the file input after every attempt so the same file can be retried. Permit subsequent valid imports after failures.

### 5. Capture one stable Results region

Wrap the character cards through `EnergyPanels` in a capture boundary while retaining the existing `.simulation-report` styling context. Keep `ComparisonPanel` and Run details outside it. Read current DOM width and full region height; do not substitute a standard desktop layout or expand folded content. Current distribution/timeline/source/energy selections and card/source-list expansion remain visible in the image; the embedded report still contains the full data.

Use pinned `html-to-image` 1.11.13, loaded only for image export, to clone the DOM/CSS and embed artwork into SVG. Its deep SVG cloning skips computed styling on SVG descendants, so decorate those descendants in the detached SVG markup before rasterization. Encode the raster through canvas Blob output, then clear its dimensions and image source in `finally`; the live DOM is never restyled. Capture at one raster pixel per CSS pixel to avoid device-dependent quadrupling of memory. Preserve computed background, inherited CSS variables, pseudo-elements, SVG charts, control values and bundled artwork. Await fonts, image decoding and layout stabilization before capture. Close the menu and omit transient chart/card tooltips from the snapshot. The [library](https://github.com/bubkoo/html-to-image) clones DOM/CSS into SVG, so browser visual verification is required.

Image export is available for stable completed, stopped or failed delivered results. While simulation/sample execution is active, keep the PNG entry visible but disabled with a reason; JSON and compressed exports may still snapshot the delivered prefix. During capture, snapshot the executed bundle once, prevent overlapping export operations and discard the result if its run identity changes. Do not mutate the live cards or chart selections.

Before allocating the canvas, check nonzero safe integer dimensions, a maximum edge of 32,767 pixels and maximum area of 32,000,000 pixels. If unsupported by the browser or these bounds, fail explicitly without silently truncating or downscaling. Release canvas, temporary nodes, image resources and object URLs after every outcome. Verify final PNG plus attachment is within 100 MB before download.

### 6. Embed binary report data in PNG

Add exactly one private ancillary `ntEr` chunk before `IEND`, containing the standalone container without Base64 or another compression layer. The lowercase final letter marks the attachment as safe to copy because report data is independent of image pixels. Write and verify the PNG chunk CRC, scan by chunk type rather than assuming a fixed position, and retain valid ordinary PNG structure. The [PNG specification](https://www.w3.org/TR/png/#5Chunk-layout) defines this extension mechanism.

PNG import only needs the chunk parser and codec; it never invokes the DOM capture library or rasterizes the image. An image whose metadata was stripped cannot recover its report and must report that specific condition. JPEG EXIF and text-chunk Base64 were rejected because they add conversion/size costs without improving restoration.

### 7. Integrate the menu, dependencies and existing checks together

Reuse the Tools placement, arrow and dismissal pattern with shared behavior where practical. Add explicit focus movement/restoration and accessible menu state for Export report. Generalize the download helper for string and Blob/binary output. Update Import report's file picker and the browser `exported` helper together; a click on the opener alone no longer downloads JSON.

Add both runtime packages and exact lock entries through the cached installation workflow during apply. Resolve their JS and the codec WASM through Vite's existing checkout-relative cached-module setup, following the codec's documented optimize-dependency exclusion where required. Emit a local WASM URL that works in development, production, module Workers and subpath deployments; do not fetch libraries from a CDN at runtime. Include lazy modules/WASM in the generated offline inventory and stage the npm packages' and embedded codec's applicable licenses.

Extend existing `file_exports`, `report_controls` and Studio round-trip tests for codec/container/PNG data handling, exact values and failure status. Exercise true output-capacity failure as well as header rejection: forged small content size, oversized 64-bit content size/window, unknown size, concatenated frames and truncated/duplicate PNG chunks. Use small reduced-cap fixtures for repeated hostile cases, plus real 100,000,000/100,000,001-byte boundary checks. Verify rejection happens before unchecked decoder allocation and that failed attempts release their Worker/resources.

Extend existing browser acceptance for menu behavior, all downloads/imports, desktop and 390-pixel PNG output, current filter/expansion fidelity, a valid image without report data, stale-run cleanup and offline lazy-codec/capture loading. Compare the exported raster with the equivalent live Results region and inspect artwork/SVG/text and the final panel boundary. Retain existing native/WASM replay checks as integration evidence; do not add duplicate mechanics tests or recalculate Team list DPS.

## Risks / Trade-offs

- A JSON report within the byte cap can require substantially more object memory -> run codec work separately, minimize copies, clean up promptly and show allocation failures rather than claiming a 100 MB total-memory guarantee.
- Zstd metadata can advertise much larger allocations than the actual report -> preflight full frame structure and bounds before the library, then rely on an explicitly bounded destination rather than trusting the header or checking output afterward.
- DOM-to-image may differ for CSS, fonts or SVG in a browser -> verify actual exported rasters at both supported layout boundaries; missing assets or incomplete captures fail visibly.
- PNG raster memory scales with area independently of compressed report size -> fixed CSS-pixel scale, explicit dimensions/area guardrails and cleanup, with no silent crop.
- Image editors or services can remove the attachment -> document sharing the original PNG as a file; import gives a specific absent-report error.
- Lazy dependency paths can break in production/offline -> validate emitted Worker/WASM URLs, subpath operation, inventory and attribution in the existing package/browser boundaries.

## Migration Plan

Implement UI, file readers/writers, tests and detailed-reference changes together. Keep the existing JSON report schema, `.nte` script behavior and simulator contracts. The new v1 container is the only supported compressed format; no gzip, archive or obsolete-input compatibility adapters are introduced. Archive and synchronize this change after implementation checks. Publication is outside this change; reverting the implementation removes the new formats while previously saved JSON remains usable.
