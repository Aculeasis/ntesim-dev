# Proposal

## Why

Studio currently downloads only large JSON reports. Users need a fast, smaller portable report and a shareable image of the displayed results that also carries the complete restorable report, with explicit protection against oversized files and decompression bombs.

## What Changes

- Replace the Results view's immediate Export report action with a Tools-style dropdown offering JSON, Compressed report and PNG with report.
- Keep the current JSON bundle and add lossless Zstandard level-9 compression of its compact UTF-8 representation in a versioned report container.
- Capture the current Results presentation from the character cards through Character Energy at End of Simulation, inclusive, and embed the same compressed report container in PNG metadata.
- Import JSON, compressed reports and report-bearing PNGs through Import report, restoring the executed source, delivered results, playback data and failure/partial status.
- Raise the report file limit from 50 MB to 100 MB (100,000,000 bytes), with an independent 100 MB decoded-JSON limit enforced before and during decompression. Keep the script-import limit at 50 MB.
- Report missing embedded data, corrupt or unsupported formats and size-limit violations without replacing the current draft or results.

## Capabilities

### New Capabilities

None; the formats extend the existing Studio report workflow.

### Modified Capabilities

- `studio-application`: report export menu, portable compressed and PNG report round trips, faithful Results image capture, and bounded report import.

## Impact

- Presentation and file handling: `web/src/app/App.tsx`, the import/export feature, `ResultsPage`, storage helpers and a dedicated report-codec Worker.
- Dependencies and packaging: a pinned Zstd WASM codec and DOM-to-image library, cached dependency installation, Vite asset handling, offline asset inventory and third-party licenses.
- Validation: extend existing report/file tests and browser acceptance for the new dropdown, PNG rendering, downloads, imports and hostile inputs. Update the Studio detailed reference and relevant user documentation during implementation.
- Rust combat logic, native CLI report commands and the simulation WASM/Worker protocol do not change. Exact report values, executed-source identity and evidence/partial status retain their current contracts.
