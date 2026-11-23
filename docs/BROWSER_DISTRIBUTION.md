# Portable Studio distribution

The local browser artifact contains static Studio, its actual Worker and Rust/WASM
engine, identity illustrations, original SVG icons, notices and exact runtime dependency license texts.
It needs no game installation, research package or backend. The
[release contract](../openspec/specs/distribution/reference.md) governs publication separately.

## Build and exercise

Use the pinned Rust/toolchain and WASM generator from the [runbook](RUNBOOK.md),
Python 3.11+, Node 22.12+ and npm. Install the web lockfile with
`node tools/web.mjs install`, then:

```sh
python tools/browser_distribution.py build --output target/distribution/studio.zip
python tools/browser_distribution.py verify target/distribution/studio.zip --current
```

Destinations must be new; `--offline` on build requires cached Cargo inputs.
Verification requires installed Chrome and Playwright with
`NTESIM_PLAYWRIGHT_MODULE` set to its absolute `index.mjs`.
Local build/stage/verification descendants inherit the shared four-CPU limit. Only
trusted local artifacts are executed; hashes are integrity, not authentication.

The builder performs a locked current WASM build and Vite production build.
Every production build stages complete notices before constructing its offline
inventory. `build-info.json` records engine/source identities, Cargo/npm lock
hashes, Rust/Node/generator versions, WASM hash and runtime dependency closures.
Its `studio_build` field binds the same HEAD/ref links and UTC build-start instant
as the header; see [build identity](DEVELOPMENT.md#studio-build-identity).
Cargo resolves normal/build dependencies from `ntesim-wasm` for
`wasm32-unknown-unknown`; npm traverses only the installed locked runtime closure
(React, React DOM, Scheduler). Missing attribution, versions or license texts
reject the build. Development/build dependencies are not shipped runtime code.
The pinned Rust standard-library copyright document and MIT/Apache/LLVM texts
cover linked standard-library components; PCG retains its license and notice.

`manifest.json` binds every archive payload's size/SHA-256 and build identity.
The adjacent `.zip.sha256` binds the whole ZIP. Inventory and paths are checked
before extraction. `--current` rejects changed source or locks. Fixed archive
timestamps/order/modes give equal archives for equal payloads; cross-machine
compiler/path byte equality is not promised. Engine identities include raw line
endings; replay retains each actual identity.

## Artifact review and usage

Studio bundles the locally retained character, weapon, set and element WebP
illustrations used by cards, search and replay, alongside original SVG icons and
favicon. Third-party illustrations are sourced for identity display;
they belong to their respective owners and are not covered by the application
MIT license. No redistribution license for them is asserted; `site/NOTICE`
records this distinction. Copied source-site descriptions, acquisition evidence,
raw exports and research packages remain excluded from version control.
Numerical runtime constants remain consumed typed Rust content. Vite serves and
copies `web/public/assets/artwork/`; the generated offline inventory includes
every illustration. Unavailable artwork uses an accessible SVG placeholder.

Extract and serve `site/`, for example:
`python -m http.server 8080 --directory site --bind 127.0.0.1`.
Open [localhost Studio](http://127.0.0.1:8080/). HTTPS static hosting may use a
directory prefix. Preserve relative paths and JavaScript/WASM MIME types;
`file://` is unsupported. Initial loading needs the files. Subsequent controlled
loads support offline execution. A new build is fully cached before a top banner
offers **Update & reload**; activation waits for that click. Studio checks for
updates on return to the tab, reconnect and every five minutes while visible.
The button waits for an active operation to finish. Reload preserves the locally
saved script/editor settings and clears in-memory results; other open tabs offer
their own reload after activation. First installation needs no update prompt.
Development mode uses Vite updates directly and does not install the offline cache.
The archive README contains executable usage/reproduction commands and model,
storage, export and comparison limits.

Verification relocates the payload to a new space-containing directory and
executes existing [browser acceptance](BROWSER_PERFORMANCE.md) against that copy
under `/ntesim-dev/`: header source links and responsive layout, actual Worker workloads, exact exports, comparison/search
continuation, keyboard/mobile/errors, Service Worker upgrade and offline cold
reload/execution. This reuses the browser integration boundary; it adds no combat
formula tests. Native/WASM/Worker parity remains a separate final checkpoint.

## GitHub Pages deployment

[Open Studio on GitHub Pages](https://aculeasis.github.io/ntesim-dev/).

In repository **Settings → Pages**, select **GitHub Actions** as the source.
Allow the `github-pages` environment to deploy from main and intended tag refs;
a main-only branch rule would block tag events even after ancestry validation.
The workflow grants `contents: read`, `pages: write` and `id-token: write`, and
serializes deployments without cancelling one already running. See GitHub's
[custom Pages workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
and [single-job static workflow](https://github.com/actions/starter-workflows/blob/main/pages/static.yml).

Only tag pushes and manual runs trigger `.github/workflows/deploy-pages.yml`.
For manual deployment, open **Actions → Deploy Studio to Pages → Run workflow**,
select **main**, and either leave `tag` blank to build the freshly fetched main
tip or enter an exact tag such as `v1.2.3` or `release/v1.2.3`. Pushing an intended
tag such as `git push origin refs/tags/v1.2.3` triggers the same pipeline.
Ordinary branch pushes do not deploy.

Lightweight and annotated tags must peel to a commit anywhere in fetched main
history; they need not point to its tip. Unknown tags, branch names, revision
expressions, non-commit tags, tags outside main and non-main manual dispatch fail
before build/upload/deployment without fallback. Selection fetches full main/tag
history, freezes the full SHA and checks out that exact commit. Tagged builds
display the explicit tag; blank manual runs display main's short hash even if
the commit already has tags. An old tag missing required build tools fails
honestly instead of building newer sources under an older label.

One Ubuntu job installs Python 3.13.9, Node 24.19.0, Rust pinned by the selected
`rust-toolchain.toml`, the WASM target and the exact wasm-bindgen CLI version in
`tools/wasm_bindings.py`, plus locked npm dependencies. Rustup must be available
on the runner; Cargo acquisition uses the local Cargo cache.
The job runs `python tools/browser_distribution.py build --output
target/ntesim-browser-pages.zip`, including current WASM, the committed Team list, notices
and offline assets, then uploads **`target/web`**, not the ZIP, using official
Pages actions. Any required step failure prevents deployment. No test suite,
distribution `verify` or checkpoint runs in that job.
Deployment does not recalculate team DPS. Refresh the committed catalog explicitly
with `node tools/web.mjs teams` or use `build --release` for fresh release preparation.

The build explicitly sets `NTESIM_BUILD_CPUS=all`; only with GitHub's
`GITHUB_ACTIONS=true` does the shared limiter retain all available runner CPUs
without imposing thread defaults. Smaller caller limits still survive in build
children. Ordinary local builds/validators retain the shared four-CPU policy and
one validator at a time. Team calculations still process one scenario at a time.
The generated relative URLs support the `/ntesim-dev/` project subpath, including
both WASM modules, Workers and the scoped offline cache.

## Acceptance

Accepted locally on 2026-11-07: Windows x64, Chrome 156.0.8078.81, Playwright
1.62.1, Node 24.19.0, pinned Rust 1.98.1 and wasm-bindgen 0.2.128. Final
`python tools/checkpoint.py run --full` retains the complete workspace/package,
native/WASM/Worker parity and web regression boundary, with extra checks for
current native archive reproduction and relocated current Studio acceptance.
The literal extra-check array is:

```json
[
  ["python", "tools/native_distribution.py", "verify", "target/distribution/ntesim-native-phase4.zip", "--current"],
  ["python", "tools/browser_distribution.py", "verify", "target/distribution/ntesim-studio-phase4-final.zip", "--current"]
]
```

Set `NTESIM_PLAYWRIGHT_MODULE` when reproducing the browser command. The retained
checkpoint uses a local wrapper to set that measurement-only variable in the
browser child, preserving cache reuse for unchanged unrelated checks.

Current artifacts are `target/distribution/ntesim-native-phase4.zip` and
`target/distribution/ntesim-studio-phase4-final.zip`, each with its adjacent
SHA-256 file. Studio's archive SHA-256 is
`0e6b5d2834eec1d36f8dcbf4309cdb1da704a3bda21da20cea4c078359c24035`;
the native archive's is
`952a02e6abe977fe433c3c12741a34a21fc094ab39182f0938cffdcb202cf2aa`.
Current browser measurements and screenshots remain local.

Fresh reproduction uses a new local clone of `dd64dc2` with the final Cargo
gateway patch copied onto that checkout, its own downloaded Cargo/npm caches,
installed pinned compiler/WASM target and new build directory. Only the
version-checked wasm-bindgen generator is reused as an installed prerequisite;
no old engine artifacts or Cargo build/dependency caches are copied. npm installs
with lifecycle scripts disabled. Dependency downloads use the caller's network
environment, preserving explicit Cargo settings; see
[local configuration](DEVELOPMENT.md#local-configuration). No private network
endpoint is part of the reproduction instructions.

The fresh archive passes the same relocated actual browser scenario.
Its engine identity is
`55e77d025e37fc0ff3fa75aa1bdcf667d7a53d462178d85db8bdeb30abba5972`,
versus current checkout
`932fba6d6bfc3d4f1fa5f876af4dfa7f6106dd951991d26fd0e79ff395a7e452`.
Comparing every engine input found only Git CRLF/LF normalization differences.
Each artifact executes/reconciles under its own identity; cross-identity replay
or binary byte equality is not claimed.

Model and calibration details remain in
[native baselines](NATIVE_BASELINES.md) and [browser calibration](BROWSER_CALIBRATION.md).
