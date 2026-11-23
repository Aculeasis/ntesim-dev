# Proposal

## Why

Studio's static game title and Global cutoff in the upper-right header do not identify the code or build being viewed. A small source identity and local build timestamp, together with an automatic Pages deployment workflow, will make published versions traceable and the browser Studio easy to find.

## What Changes

- Replace those two header lines with adjacent rounded, bordered pills: a linked GitHub icon and separate build identifier, plus the build-start instant formatted as `dd.mm.yy · hh:mm` in the viewer's time zone.
- Use the checked-out HEAD's short hash for ordinary local builds, including dirty builds and checkouts at a tag. Prefix a non-main branch as `some/branch/bb887a5`; main displays only the hash. Explicit tag deployments display the tag instead.
- Add one GitHub Actions job that builds and publishes Studio on tag pushes or manual dispatch. Tags must resolve to commits in main's history, as confirmed by the user. Manual dispatch runs from main, accepts an optional tag, and otherwise builds the latest fetched main commit.
- Build current WASM and Studio with the committed Team list, staged notices and offline assets, without recalculating DPS or running test suites in the deployment job. Keep fresh Team list calculations explicit. Permit that GitHub-hosted build to use all runner CPUs while preserving local four-CPU limits.
- Link Studio at `https://aculeasis.github.io/ntesim-dev/` in README and document deployment inputs and Pages setup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `studio-application`: Observable header source identity, repository/ref navigation and viewer-local build timestamp.
- `distribution`: Main-history-only tag/manual Pages publication, one build/deploy job, preserved browser release contents and a GitHub build CPU exception.

## Impact

- React header markup/styles, Vite build metadata and adjacent Studio rendering coverage.
- Browser distribution metadata/staging, shared CPU-limiting helper, existing tooling tests and checkpoint classification for the new workflow file.
- New `.github/workflows/deploy-pages.yml`, README and development/distribution documentation.
- Existing Rust engine, native CLI semantics, WASM API and Worker messages remain outside the behavioral scope. The workflow still compiles and packages the shared WASM engine.
