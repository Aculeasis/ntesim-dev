# Tasks

## 1. Build identity and Studio header

- [x] 1.1 Add shared build-time identity resolution and Vite injection; capture one UTC build-start instant, use local HEAD/branch rules, validate explicit deployment context, and forward matching metadata into browser staging. Extend existing tooling/distribution coverage for main, dirty HEAD, branch, detached/local-tag and explicit-tag cases; verify staged metadata matches the supplied build context and new consumed helpers participate in source/checkpoint fingerprints.
- [x] 1.2 Replace the two static lines in `web/src/app/App.tsx` with an accessible GitHub icon, separate ref link and fixed viewer-local timestamp, using adjacent bordered source and timestamp pills; extend the existing Studio rendering boundary for exact destinations, slash/reserved-character tags, zero padding, multiple time zones and a daylight-saving/date boundary. Verify `node tools/web.mjs check` and the focused existing rendering checks pass.
- [x] 1.3 Document local HEAD semantics, detached builds and timestamp behavior in the existing Studio/development reference; verify Markdown links and that documentation distinguishes the build instant from commit date, page-open time and simulation time.

## 2. GitHub deployment build CPU exception

- [x] 2.1 Add the explicit GitHub-only build opt-in to `tools/cpu_limit.py` and ensure nested Cargo, browser staging and Team list build processes inherit it. Extend `tests/test_cpu_limit.py` to verify a simulated runner with more than four available CPUs is not capped, smaller caller limits survive, and GitHub context or opt-in alone does not disable local defaults; run the focused CPU/tooling tests under the normal local cap.
- [x] 2.2 Update shared CPU guidance in `AGENTS.md`, `docs/DEVELOPMENT.md` and `openspec/config.yaml` to describe the deployment-only exception. Verify local validator guidance still requires one validator at a time under the shared four-CPU limit.

## 3. Source selection and one-job Pages workflow

- [x] 3.1 Implement exact tag/main source resolution with fetched history, annotated-tag peeling, ancestry validation and frozen SHA checkout. Extend the existing distribution/tooling test boundary with isolated Git fixtures for a historical main tag, annotated tag, blank manual input, unknown tag, tag outside main, branch/revision input masquerading as a tag and non-main manual dispatch; verify invalid selections stop without fallback or deployment.
- [x] 3.2 Add `.github/workflows/deploy-pages.yml` with tag-push/manual triggers, optional tag input, exactly one build/deploy job, official Pages actions, permissions, environment, concurrency and the CPU opt-in. Install pinned build prerequisites and invoke browser `build`; upload `target/web` and deploy only after successful build/upload. Verify parsed workflow structure and triggers locally, and confirm no test, checkpoint or distribution `verify` commands are invoked by the job.
- [x] 3.3 Classify the workflow and any new build helper in `tools/checkpoint.py`, extending `tests/test_agent_checkpoint.py` for affected check selection and fingerprint invalidation. Verify the new workflow is accepted as a known input and its change selects the appropriate tooling validation without adding a remote test job.
- [x] 3.4 Add the GitHub Pages Studio link to README and document manual/tag examples, main-history eligibility, prerequisites, GitHub Actions as the Pages source and environment rules permitting main plus tag events. Check links and keep setup instructions durable.
- [x] 3.5 Keep GitHub Pages deployment free of Team list DPS recalculation, reuse committed outcomes, and update the deployment/ranking contracts and documentation. Verify the parsed workflow excludes `--release` and Team list calculation commands while existing explicit release preparation still recalculates.

## 4. Integration and delivery checks

- [x] 4.1 Run focused existing distribution/CPU/checkpoint tests, web type/render checks and required current WASM/Worker checks selected for the stable implementation diff. Build a release artifact and use the existing browser smoke boundary at `/ntesim-dev/` to verify header links, desktop/390 pixel layout, assets, both WASM modules, Worker execution and service-worker scope; retain results under `cache/`.
- [x] 4.2 Review the complete stable diff, run `python tools/openspec.py validate add-studio-build-identity-and-pages-deploy --strict --no-interactive`, then `python tools/checkpoint.py run` with one validator at a time. Fix failures and rerun affected checks; report behavior and actual checks without pushing tags, triggering Actions or publishing.

## Workflow follow-up

- After implementation tasks and checks pass, synchronize the deltas and archive this change using the project's OpenSpec archive workflow.
