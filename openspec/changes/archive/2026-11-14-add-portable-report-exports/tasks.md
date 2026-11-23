# Tasks

## 1. Runtime dependencies and asset resolution

- [x] 1.1 Add exact runtime dependencies `@bokuweb/zstd-wasm` 0.0.27 and `html-to-image` 1.11.13, update the lockfile and install through the cached workflow. Resolve their modules and local codec WASM through Vite; verify installation, a production build, local Worker/WASM URLs and complete runtime/embedded-library attribution.

## 2. Portable report storage and bounded decoding

- [x] 2.1 Separate the 50,000,000-byte script limit from the 100,000,000-byte report file and decoded-JSON limits; retain pretty JSON and add compact exact serialization and format-specific filenames/binary downloads. Extend `web/tests/file_exports.test.mjs` and the existing report tests to verify UTF-8 byte accounting, extensions, executed-source identity and unchanged exact digits.
- [x] 2.2 Implement the v1 `.nter` container, Zstd-9 encoding, complete frame preflight and bounded decoding in a dedicated module Worker with transfer, task isolation and cleanup. Extend existing file/report coverage for byte-exact round trips, hash/version/flags failures, huge 64-bit sizes/windows, unknown sizes, dictionaries, extra frames/trailing bytes, forged small sizes and actual output-capacity failure; verify untrusted oversized declarations are rejected before codec output allocation and that a valid operation works after errors.
- [x] 2.3 Implement bounded PNG chunk reading/writing for exactly one `ntEr` attachment containing the complete `.nter` container. Extend existing file/report tests to verify ordinary PNG structure/CRCs, attachment byte identity, no pixel decoding on import, absent/duplicate/corrupt/truncated chunks and complete-file size enforcement.
- [x] 2.4 Route JSON, `.nter` and PNG through one validated lossless report-import path with atomic UI loading and distinct errors. Extend `web/tests/report_controls.test.mjs` and the Studio round-trip boundary for source/request/aggregate/sample/playback restoration, wide values, stopped/failed context, invalid report structure and unchanged draft/results on failure; verify real 100,000,000/100,000,001-byte file and decoded boundaries, plus reduced-cap bomb fixtures.
- [x] 2.5 Document the container layout, PNG chunk, byte units, supported formats and separate file/decoded limits in `openspec/specs/studio-application/reference.md`; verify the documented fields and error cases agree with the focused tests and the design.

## 3. Export menu and faithful PNG capture

- [x] 3.1 Replace the immediate Export report action with the three-item adaptive dropdown, accessible keyboard focus/dismissal and visible preparation/error state. Update the browser `exported` helper and affected consumers together; extend existing browser acceptance to verify that opening the menu does not download, selecting JSON still downloads directly, all three entries work, and missing-result/busy PNG availability is explained.
- [x] 3.2 Add the Results capture boundary from team cards through the final character-energy panel, leaving comparison/details outside. Implement font/artwork/layout readiness, current control/expansion fidelity, CSS-pixel raster scale, dimension/area guards and cleanup. Extend existing browser acceptance and inspect exported desktop/390-pixel rasters against the equivalent live region, verifying artwork, SVG/text, off-screen height and both capture boundaries.
- [x] 3.3 Bind the PNG snapshot and embedded report to one stable executed run; inject its container, check final file size and download only complete output. Extend browser acceptance for PNG re-import and ordinary image display, absent attachment errors, stale-run rejection, no automatic simulation, unchanged live selections and recovery after failed capture/import.
- [x] 3.4 Update relevant Studio/user documentation for the menu, `.nter`, original-PNG sharing and readable failures; verify there are no remaining claims that clicking Export report immediately downloads JSON or that report imports are limited to 50 MB.

## 4. Integration and delivery checks

- [x] 4.1 Run `node tools/web.mjs test tests/file_exports.test.mjs tests/report_controls.test.mjs tests/studio.test.tsx` and `node tools/web.mjs check`, then run the existing affected Worker/replay checks selected by checkpoint. Fix affected consumers and verify all report representations preserve the same delivered execution without introducing Rust mechanics changes.
- [x] 4.2 Run the existing production browser acceptance through `node tools/web.mjs verify-trials --browser`; verify all formats, visual capture, local subpath Worker/WASM loading, offline lazy imports and license inventory. Retain browser/visual evidence under `cache/` and confirm no Team list DPS recalculation or publication occurred.
- [x] 4.3 Review the stable diff and run `python tools/openspec.py validate add-portable-report-exports --strict --no-interactive` and `python tools/checkpoint.py run` serially, fixing issues and reusing valid receipts where applicable. Verify code, tests, reference and change artifacts agree before marking implementation complete.

## Workflow follow-up

- After implementation tasks and applicable checks pass, synchronize and archive with `python tools/openspec.py archive add-portable-report-exports --yes`.
- Run strict archived validation and checkpoint on the final tree before a coherent commit. Publication/deployment requires separate explicit authorization.
