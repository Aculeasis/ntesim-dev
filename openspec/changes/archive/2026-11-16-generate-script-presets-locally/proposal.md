# Proposal

## Why

`web/src/generated/script-presets.json` duplicates the maintained `.nte` scripts and manifest in Git even though Studio already regenerates it automatically. Treating this JSON as a local build artifact removes duplicate diffs and synchronization work while preserving the browser's bundled preset sources.

## What Changes

- Stop tracking the generated script preset bundle and ignore its exact path.
- Retain automatic generation before supported Studio build, development and validation consumers, including a checkout without the generated file.
- Keep the preset manifest and `.nte` scripts as the maintained inputs, with exact text included in the browser build.
- Document the generated bundle's lifecycle and verify existing entry points without a preexisting bundle.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `development-workflow`: Require an untracked, automatically prepared Studio preset bundle from maintained script inputs.

## Impact

The expected implementation touches `.gitignore`, removal of the tracked generated JSON, and `docs/DEVELOPMENT.md`. Existing generation in `tools/studio_presets.mjs`, `tools/web.mjs` and `web/vite.config.ts` should be reused; change those consumers only if verification reveals a preparation gap. Browser tests also read the bundle and must remain usable through the supported commands. This introduces no dependencies, Rust mechanics, Worker protocol or UI changes. Saved `team-list.json` calculations retain their existing committed lifecycle.
