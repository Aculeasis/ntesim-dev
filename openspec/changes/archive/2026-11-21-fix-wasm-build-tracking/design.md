# Design: Fix WASM Build Freshness Tracking

## Context

See `proposal.md` for motivation. Currently, `crates/wasm/build.rs` emits only `cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT`. Under Cargo rules, once any `rerun-if` directive is output, Cargo disables default file watching for that build script. Consequently, when developers modify Rust source files and invoke `python tools/cargo.py build -p ntesim-wasm ...`, Cargo sees that `NTESIM_BUILD_STARTED_AT` is unchanged, marks `build.rs` as fresh, and reuses the cached `NTESIM_WASM_BUILD_TIME` value from previous compilations.

## Goals / Non-Goals

**Goals:**
- Ensure `crates/wasm/build.rs` re-runs whenever any engine sources change under `tools/cargo.py` or when local crate files change under standard Cargo.
- Keep the build script fresh whenever `NTESIM_BUILD_STARTED_AT` is explicitly supplied (preserving distribution / CI deterministic behavior).
- Preserve build cache when engine sources and environment have not changed.

**Non-Goals:**
- Unconditionally forcing `build.rs` to run on every build invocation regardless of source changes (which would destroy Cargo's incremental compilation caching).
- Altering the contract of `NTESIM_BUILD_STARTED_AT` or `engine.buildStartedAt()`.

## Decisions

### 1. Track `NTESIM_ENGINE_SOURCE_SHA256` and local sources in `crates/wasm/build.rs`
- Emit `cargo:rerun-if-env-changed=NTESIM_ENGINE_SOURCE_SHA256`.
  - *Rationale*: `tools/cargo.py` already computes `NTESIM_ENGINE_SOURCE_SHA256` covering all workspace runtime sources (`crates/**/*.rs`), `Cargo.lock`, `Cargo.toml`, and preset datasets. Whenever any engine code changes across the workspace, this environment variable changes, triggering `build.rs` to compute a new UTC timestamp.
- Emit `cargo:rerun-if-changed=src` and `cargo:rerun-if-changed=Cargo.toml`.
  - *Rationale*: For direct cargo invocations outside `tools/cargo.py` where `NTESIM_ENGINE_SOURCE_SHA256` is not present, changes to `ntesim-wasm` itself still invalidate the build script cache.
- Retain `cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT`.
  - *Rationale*: Maintains compatibility with `tools/browser_distribution.py` and CI pipelines that provide explicit timestamps.
- *Alternatives considered:* Emitting no rerun directives at all (rejected: would default to watching only `crates/wasm/`, missing changes in dependency crates like `crates/content/` or `crates/core/`).

## Risks / Trade-offs

- **[Timestamp updates on engine source edits]** → Whenever any runtime Rust file is modified, the WASM build timestamp is updated. When no code is modified, Cargo cache is preserved and the timestamp remains stable.
