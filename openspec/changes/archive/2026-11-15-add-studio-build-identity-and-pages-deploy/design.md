# Design

## Context

See [proposal.md](proposal.md) for motivation. `web/src/app/App.tsx` currently renders the two literal header lines. `web/vite.config.ts` uses `base: './'`, emits the offline service worker, and calls `tools/browser_distribution.py stage` to retain dependency notices and `build-info.json`. `tools/browser_distribution.py build --release` already compiles WASM, runs `teams --fresh`, builds the site and packages it. These are build/preparation operations; its separate `verify` command invokes browser tests.

There is no `.github/` workflow yet. `tools/cargo.py`, browser distribution and Team list subprocesses call `tools/cpu_limit.py`. `tools/checkpoint.py` currently rejects unclassified workflow paths. Existing regression boundaries include `web/tests/studio.test.tsx`, `web/tests/offline.test.mjs`, `tests/test_cpu_limit.py`, `tests/test_native_distribution.py` and `tests/test_agent_checkpoint.py`.

## Goals / Non-Goals

**Goals:** Bind the displayed identity and timestamp to the actual built source; resolve deployment refs before compiling; reuse the current browser release pipeline and project-subpath support; keep the GitHub build CPU exception explicit and inherited by build children.

**Non-Goals:** Changing Rust mechanics, native CLI behavior, WASM wire contracts, Worker messages, report identities or game-data cutoff policy. No new test matrix or general release-management framework.

## Decisions

### 1. Resolve one immutable build identity

Add a small build-time metadata helper consumed by Vite and the browser distribution entry point. Resolve Git from the checkout root using argument arrays: full HEAD SHA, a short SHA with at least seven characters, and the symbolic branch when attached. Ordinary local invocations always use HEAD, even with uncommitted edits or tags; do not use `git describe` or HEAD's parent. Detached local builds use the hash without a branch prefix. Keep the repository URL fixed to the user's public destination, independent of clone remotes or private local settings.

An explicit deployment tag/commit context overrides branch detection only after ref validation and HEAD equality checks. A tagless deployment supplies logical branch `main` even if checkout uses a detached SHA. Use hash tree URLs for hash labels and unambiguous `tree/refs/tags/<tag>` URLs for tag labels; encode ref characters without losing slash-containing tag identity.

Capture a UTC instant once at the outer browser build's start, or at Vite startup when invoked directly or in development. Forward the same metadata into Vite's compile-time constant and its staging subprocess so `build-info.json` describes the same source and instant as the header. Store generated scratch only in `cache/` if needed; do not create tracked generated metadata. Include any new consumed helper in distribution source fingerprints and checkpoint dependency inputs. Missing Git context must produce an honest unavailable identity or an actionable build error, never a plausible fabricated ref; deployment requires a valid identity.

This avoids a runtime GitHub request, local tag guessing and timestamps that change on every page visit. Byte-for-byte rebuilds require the same supplied build instant; the ordinary timestamp intentionally identifies each new build rather than guaranteeing identical bytes across different times.

### 2. Keep localization and layout in the UI

Render a dedicated small header component from `App.tsx`, using an inline GitHub SVG and normal anchors with accessible names. Present the icon/ref and timestamp in adjacent rounded, bordered pills of equal height, with a 12 pixel gap. Keep this pair together when the header wraps on narrow screens. Use the browser's local date/time components to format fixed-width `dd.mm.yy · hh:mm` with a 24-hour clock; do not let locale formatting select separators or field order. Retain the UTC value in a semantic `time` element. Use scoped header styles, wrapping or truncation with the full identifier inspectable, to keep long refs usable at 390 pixels. The build timestamp is unrelated to simulation time or engine identity.

### 3. Resolve the selected source against fetched main

Create `.github/workflows/deploy-pages.yml` with `push.tags: ['**']` and `workflow_dispatch` containing one optional string input, `tag`. Do not add branch-push or PR triggers. Keep one Ubuntu job for source selection, build, artifact upload and deployment.

Fetch full history and tags. Manual dispatch requires `github.ref == refs/heads/main`; automatic tag events are valid even though their event ref is not a branch. Resolve tags through exact `refs/tags/` names, peel annotated tags to commits, and check `git merge-base --is-ancestor <commit> origin/main`. The user selected any commit in main's history, not just its tip. Without an input tag, resolve and freeze the fetched `origin/main` tip. Checkout the resolved full SHA, verify HEAD, and pass explicit version context to the build. A missing/ineligible ref fails before build/upload/deploy, without fallback. Use event/input values through environment variables and argument arrays or quoted shell variables, never interpolated as executable shell code.

Branch trigger filtering alone cannot prove tag ancestry. Looking up tags attached to HEAD also cannot distinguish manual hash builds from intentional version deployments. The explicit selection context covers both risks.

### 4. Build and deploy the existing release payload in one job

Install supported Python, a pinned supported Node version satisfying the current Vite package, pinned Rust from `rust-toolchain.toml`, the WASM target and the exact wasm-bindgen CLI version consumed by `tools/wasm_bindings.py` into `cache/wasm-bindgen`. Install npm dependencies through `node tools/web.mjs install` using the committed lockfile. Use online dependency acquisition on a fresh runner, preserving the project's local Cargo cache layout.

Run `python tools/browser_distribution.py build --output target/ntesim-browser-pages.zip`, then upload `target/web` as the Pages site. The ZIP is a local packaging byproduct, not the Pages artifact layout. This builds current engine glue, notices and service-worker assets while reusing the committed Team list without DPS recalculation. Explicit `teams` refresh and `build --release` retain their separate calculation behavior. Actions runs no `verify`, checkpoint or test suites. Keep `base: './'` unless the project-subpath smoke check demonstrates a necessary change.

Use the official configure/upload/deploy Pages actions with compatible maintained versions selected at implementation. Grant `contents: read`, `pages: write` and `id-token: write`; use the `github-pages` environment and deployment URL output. Serialize Pages deployments with a shared concurrency group and do not cancel an in-progress deployment. Build/upload must succeed before the final deploy step in the same job.

The [official custom workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages) describes the Pages permissions, artifact and environment requirements. GitHub's [static Pages workflow](https://github.com/actions/starter-workflows/blob/main/pages/static.yml) demonstrates the single-job arrangement. Environment ref rules must admit main dispatches and intended tag events; a branch-only main environment rule would reject tag deployments despite their valid ancestry.

### 5. Explicit GitHub build CPU exception

Extend the existing shared limiter with a narrow opt-in, for example `NTESIM_BUILD_CPUS=all`, honored only when `GITHUB_ACTIONS=true`. Set it only in the deployment job. In that mode do not narrow the runner's allowed affinity or impose four-thread defaults; retain caller-specified smaller limits and the runner's own affinity. Nested Cargo, browser staging and Team list children inherit the same context. Do not bypass the Cargo wrapper, which also supplies engine fingerprints and WASM glue.

Local calls retain the current cap, including when the opt-in is accidentally present without GitHub context. Local validators remain bounded; ordinary CI without the deployment opt-in also remains bounded. This is preferable to an automatic global CI exemption or duplicating the build pipeline with raw Cargo commands. Update shared CPU documentation and project context to state this authorized deployment-only exception.

### 6. Verify at existing boundaries

Extend current rendering/metadata checks for main, branch, detached/local-tag and explicit-tag identities, reserved ref characters, fixed timestamps, local time-zone offsets and a daylight-saving/date boundary. Use isolated Git fixtures for source selection and main ancestry, including annotated tags and historical main commits. Extend the existing distribution/tooling tests rather than creating a new verifier command.

Extend `tests/test_cpu_limit.py` for the opted-in GitHub build and unchanged local defaults. Teach checkpoint to classify the workflow and select relevant tooling checks, with regression coverage in `tests/test_agent_checkpoint.py`. Validate YAML and the one-job/no-test trigger contract locally. Run an existing browser smoke against the built site mounted at `/ntesim-dev/`, checking header links/layout and asset/Worker/WASM/service-worker loading. This local verification stays outside Actions.

## Risks / Trade-offs

- [Pages environment ref rules can reject eligible tags] -> Document GitHub Actions as the Pages source and environment rules permitting main and intended tag refs.
- [An old main-history tag predates required build tools/workflow] -> Build exactly that tagged source and fail honestly if it lacks required prerequisites; never substitute current main while displaying the old tag.
- [Full Team list calculations delay deployment] -> Reuse the committed catalog in Pages builds; perform recalculation only by explicit refresh or explicit release preparation. Recorded calculation identities stay attached to saved outcomes.
- [A dirty local tree differs from the linked commit] -> Honor the requested HEAD label without a dirty suffix; document that local hashes identify the last committed base. Existing engine/input fingerprints continue to identify actual simulator sources.
- [Timestamp metadata changes offline cache bytes] -> Inject it before staging/cache hashing so each new build gets a consistent offline payload. Reloading an old cached build continues to show that build's identity and instant.

## Migration Plan

Implement metadata/header, source selection/workflow, CPU exception and documentation together. README links Studio. Pages uses the GitHub Actions source and environment rules permitting main plus eligible tag refs. Deployments run on tag pushes or manual dispatch from main.

For rollback, manually redeploy an earlier buildable tag in main's history. No simulator contract or persisted-user-data migration is required.
