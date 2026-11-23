# Proposal

## Why

The current Load configuration selector hides team composition and offers no DPS ranking or roster filtering. A visual Team list will make the bundled scenarios discoverable and loadable directly from the main navigation.

## What Changes

- Keep Simulator, Results and Sample as left-aligned view tabs and place an independent Team list dialog button at the center of the navigation.
- Show four compact character slots per row, approximately half the current card size, with Arc and cartridge icons on each character card and team DPS at the right. Preserve incomplete rosters with empty slots.
- Replace Load configuration with this list, retaining S1, S2, S3, S5 and Baicang and their exact scripts; remove the 20-second S4 entry from the user-facing catalog.
- Add a centered character-name search and Include/Exclude portrait pickers with selection counts, toggles and Clear All. Require every included character and reject any excluded character.
- Precompute mean DPS using the shared Rust engine before delivery; sort valid results descending and place unavailable results last with explicit status. Retain calculation provenance in generated metadata; omit per-row Details controls and calculation reports.
- Selecting a row closes the dialog, loads its scenario into Simulator and leaves execution to Run.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: Visual bundled team selection, combined roster filters, compact team rows, source loading and reproducible precomputed DPS ranking.

## Impact

- React navigation/dialog composition, editor preset controls, team presentation and responsive styles.
- Existing preset generation in `tools/studio_presets.mjs`, generated Studio assets and web build/check integration; offline generation consumes the shared Rust engine through the current WASM binding.
- Existing Studio rendering, Worker integration and browser acceptance coverage; current documentation for the preset workflow and Studio source identity.
- No combat mechanics, native CLI contract, WASM wire schema, external service or additional artwork dependency is required. Raw calculation outputs remain in `cache/`; only the compact UI projection is bundled.

Scope excludes user-authored team publishing, personal saved-team management, automatic team optimization and normalizing the existing scenarios to common combat conditions.
