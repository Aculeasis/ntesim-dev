# Tasks

## 1. Make the preset bundle a generated local artifact

- [x] 1.1 Add the exact bundle path to `.gitignore` and remove `web/src/generated/script-presets.json` from the Git index. Verify `git ls-files` no longer lists it and `git check-ignore` recognizes the regenerated file.
- [x] 1.2 Verify automatic preparation with the generated file initially absent before each relevant command: `node tools/web.mjs check`, `format-check`, `test tests/team_catalog.test.mjs`, `teams-check`, and `build`, run through `python tools/check.py --`. Verify development-server startup and an edited preset refresh through the existing Vite integration. Reuse the current generator; repair any demonstrated preparation gap and extend an existing test boundary only where needed. Confirm exact preset text and metadata and that these checks do not change saved Team list DPS.
- [x] 1.3 Update `docs/DEVELOPMENT.md` to explain the maintained manifest/scripts and disposable, ignored JSON. Verify documented commands recreate a missing bundle and regenerated output introduces no extra Git changes beyond its intended removal. Check existing browser distribution fingerprinting and invalid-input error handling remain consistent with this lifecycle.

## 2. Integration verification and review

- [x] 2.1 Review the stable diff, run `python tools/openspec.py validate generate-script-presets-locally --strict --no-interactive`, then `python tools/checkpoint.py run`. Resolve affected failures and confirm that the only maintained source changes are those required for this cleanup, with preset content and saved DPS unchanged.

## Workflow follow-up

- After implementation and applicable checks pass, archive the change with spec synchronization and validate the archived result according to `docs/DEVELOPMENT.md`.
- Deliver the completed implementation as one descriptive commit after the final checkpoint. Publication and deployment are outside this change.
