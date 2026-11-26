# Studio scripts

Edit the complete public scripts in `*.nte`. `index.json` owns their IDs, labels,
menu order, visibility and the default preset. Add a script file and a catalog
entry to expose another preset; no TypeScript changes are required.

`node tools/web.mjs dev`, `build`, `check` and `test` automatically serialize the
catalog and exact source texts into `web/src/generated/script-presets.json`.
Development also watches these files for changes. The generated JSON is a
bundling artifact; edit the `.nte` files and catalog rather than its contents.
The editor, language reference, default draft and Worker tests use this bundle.
Saved user drafts remain their own source artifacts.

Team list contains twenty-two complete four-character compositions and the unchanged
S5 historical control, which remains the default. Older partial S1/S2/S3 and
Baicang examples remain non-selectable regression sources. The language reference
is also outside the menu. Native request fixtures in `examples/*game-scenario.json`
do not generate these scripts or define the menu.

The curated scripts use 1,000 iterations, 120 relative seconds, level-80 characters
and Arcs, purchased talents 8/8/8 and fifteen useful console substat lines per
character. [Builds, rotations, investment exceptions and model limits](teams.md)
explain their exact conditions and dated sources. Each `.nte` file contains the
complete build and continuing rotation; loading it never starts a simulation.

Generate current display values with `python tools/check.py -- node tools/web.mjs
teams`, then run `teams-check` through the same wrapper. See the
[generation workflow](../../docs/DEVELOPMENT.md#studio-team-list-generation) for
the required current WASM build and cache behavior.
