# Design

## Context

See `proposal.md` for motivation and `specs/development-workflow/spec.md` for the required behavior.

`tools/studio_presets.mjs` already reads `presets/studio/index.json` and its referenced scripts, validates them, and writes the derived JSON only when its content changes. `tools/web.mjs` prepares it before supported commands. The Vite plugin also prepares it at build start and watches maintained presets during development. `web/src/domain/presets.ts` imports the JSON, while browser acceptance checks and Node-based web tests consume the same sources through their existing paths.

## Goals / Non-Goals

**Goals:** Keep the existing bundle format and synchronous preset API, with automatic preparation working when the derived file is absent. Cover both the command wrapper and Vite integration because they are separate preparation entry points.

**Non-Goals:** Changing preset content, adding HTTP loading, changing Rust/native/WASM mechanics or Worker transport, or changing the saved Team list DPS lifecycle.

## Decisions

### Keep the existing generated JSON and preparation hooks

Ignore the exact `web/src/generated/script-presets.json` path and remove it from the Git index. Keep other generated files tracked according to their current lifecycle. Reuse the current generator and import path; adjust preparation only if an existing supported entry point cannot regenerate the missing bundle before consumption.

Importing raw scripts through a new Vite loader or serving them separately would add loader or network behavior without helping this small cleanup. The existing JSON is a suitable browser build input; its presence in Git is the duplication being removed.

### Preserve maintained inputs and error handling

The manifest continues to own preset metadata and `.nte` files continue to own script text. Generation must retain their ordering, default and exact text. Invalid maintained inputs must surface their existing errors rather than silently use stale output. Document that the generated file is disposable and automatically recreated by supported commands.

### Keep verification proportional

Exercise existing type, test, build and development boundaries with the generated file initially absent. Use existing Team list/browser checks to verify exact source loading. No new mechanics tests or generator framework are needed. Any necessary regression coverage belongs in the existing tooling or Team list boundary.

## Risks / Trade-offs

- A consumer may have relied on the committed file being present. Verify preparation from an absent-file state before each relevant entry point; repair the existing preparation boundary if needed.
- Ignoring a tracked file alone does not remove it from Git. Verify both that the path is no longer tracked and that regeneration leaves Git status free of that path.
- Browser distribution source identity currently scans files under `web/`, including this bundle. Check its existing build/stage/current-verification flow for dependence on a local generated file, and ensure the supported workflow prepares inputs before fingerprinting or consuming them.
- Checking generation must not launch expensive DPS refresh. Retain saved `team-list.json` and use ordinary commands without `teams` recalculation or explicit release preparation.

## Migration Plan

Remove the generated JSON from the index, add the exact ignore rule, and update the development documentation. Existing local copies remain usable and replaceable; a fresh checkout regenerates the file automatically. Deliver the implementation as one coherent commit after applicable checks and checkpoint. No publication or deployment is part of this change.
