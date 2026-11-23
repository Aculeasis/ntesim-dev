# Native distribution and reproduction

The native artifact is a host-target CLI ZIP, built from the pinned Rust
toolchain and Cargo.lock. It contains the executable, all 23 legal continuing A0
source recipes, original MIT license, PCG notices/license and complete license
texts for the native dependency closure, plus the pinned toolchain's Rust
standard-library copyright/attribution document and license texts. Cargo metadata is filtered for the
host and traversed from `ntesim-cli` through normal/build edges; unrelated
browser/WASM and development dependencies are excluded. Registry versions and
checksums remain attached to each dependency's exact license files.

## Build and verify

Install Python 3.11+, rustup and the native platform linker/SDK. The ordinary
Cargo helper resolves Rust 1.98.1, obtains dependencies when online, and confines
its toolchain/cache/build outputs to the checkout. No game installation,
acquisition root, browser toolchain or data package is needed for this artifact.

```sh
python tools/native_distribution.py build --output target/distribution/ntesim-native.zip
python tools/native_distribution.py verify target/distribution/ntesim-native.zip --current
```

Destinations must be new. Add `--offline` to `build` only when the pinned compiler
and dependencies are already installed/cached. Both commands and all descendants
share the existing four-CPU limit. Verification executes only a trusted locally
built artifact; checksums are integrity checks, not authentication.

The build always invokes Cargo's locked release build for its explicit host
target. Windows MSVC artifacts select static CRT linking; caller Rust flags are
replaced by the recorded distribution flags. Windows 10 or newer is required.
POSIX artifacts depend on that host's system loader/libc; no older-glibc or
cross-target compatibility is claimed. Target/OS acceptance is recorded below;
an untested target is not certified by the portable Python builder.

The adjacent `.zip.sha256` binds the complete archive. `manifest.json` binds each
payload's byte length/SHA-256, engine source, Cargo.lock, packager, compiler,
target, dependency registry checksums and license texts. Archive ordering,
timestamps and modes are deterministic for identical payloads. Compilers may
embed source paths or platform details, so cross-machine binary byte equality
is not promised. Compare identities and reconcile the saved source/native result.
`verify --current` additionally rejects stale engine/lock/packager identities.
It also checks the current authored recipes and original license/notice payload.
Engine identity hashes raw source bytes, including line endings. A fresh Git
checkout can normalize CRLF to LF and therefore acquire another engine identity
despite identical code. Keep that identity on its own reports; a result from a
different identity requires its matching executable rather than cross-identity
replay. Moving the built ZIP preserves its executable identity exactly.

## Artifact-specific content review

- Original executable/authoring code and example scripts use the repository MIT
  license. Runtime constants are the consumed typed Rust mechanics; no source
  exports or independent game assets are added by packaging.
- PCG32's adapted transition/seeding retains its Apache-2.0 license and notice.
  Native dependency licenses include compound obligations such as
  `unicode-ident`'s Unicode license as well as its MIT/Apache texts. Missing
  license metadata, texts or locked registry checksums reject the build.
- Artwork, browser assets, datamined exports, source-site payloads, frozen data
  packages, acquisition evidence and development caches are excluded. The native
  source workflow embeds its compiled content constants and does not load those
  files. The project claims no redistribution license for excluded third-party
  material. `NOTICE` distinguishes the native artifact from browser/research
  material retained elsewhere in the repository.

## Executable workflow and acceptance

The extracted README contains direct run/inspect/compile/export/replay commands,
exit-code semantics, source/model limits and the build commands above. All target
stats, equipment, initial resources and assumptions remain authored in the
included `.nte` recipes. The recipes are integration baselines, not optimized
rotations or game DPS measurements. [Native baselines](NATIVE_BASELINES.md) own
their mechanics/calibration limits; [native performance](NATIVE_PERFORMANCE.md)
owns cancellation/resume, bounded search and measured resource budgets.

Verification checks the exact ZIP inventory and every payload hash before
extraction. It launches the extracted executable in a new space-containing
directory with no data/source/build cache and with inherited engine/Cargo/Rust
environment overrides removed. It executes all 23 continuing recipes, checks the
binary's engine identity, compares complete source/compiled responses, exports
a result and reconciles it after deleting the original source. This tests
distribution/IO binding, not another set of combat formulas.

Accepted native target: `x86_64-pc-windows-msvc`, Rust 1.98.1, Windows x64.
Fresh-checkout reproduction uses a new local clone, Cargo dependency cache and
target directory. The current packager and NOTICE patch are copied onto the
unchanged committed engine; the locked online build downloads its own dependencies
and does not copy old Cargo/build caches. Both archives pass the relocation
workflow above. Comparing the original/fresh engine inputs found only CRLF/LF
differences after Git normalization; source/native reconciliation runs within
each artifact's actual identity. Windows PE imports contain only system
`kernel32`, `ntdll` and the core synchronization API set, with no external MSVC
runtime DLL. Dependency downloads use the caller's network environment; see
[local configuration](DEVELOPMENT.md#local-configuration) for private settings.

The selected current-artifact checkpoint includes `--wasm-smoke` and native
distribution verification, covering the adapter/actual Worker boundary.
Other OS/CPU targets have no acceptance claim. Full [browser distribution](BROWSER_DISTRIBUTION.md),
artwork exclusion and browser parity are covered by the browser distribution boundary. No tags, publication or
deployment are performed by the builder or verifier.
