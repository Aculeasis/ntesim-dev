# Design

## Context

Studio displays a build timestamp in `.build-identity time`. Previously this came exclusively from `__STUDIO_BUILD__.build_started_at` resolved by `tools/studio_build.py` when Vite was started or bundled. The simulation engine `ntesim-wasm` is built separately through Cargo (`tools/cargo.py`). In development or when web and wasm are rebuilt independently, the displayed timestamp indicated the web GUI start/build instant rather than the simulation engine build instant. See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Embed a UTC ISO 8601 build timestamp directly into `ntesim-wasm` binary at compilation time.
- Expose the embedded timestamp via `WasmVersionInfo` and `engine.buildStartedAt()`.
- Extract and display the WASM build timestamp in Studio's header, falling back to the bundled web instant while the engine is loading.
- Coordinate `tools/browser_distribution.py` so packaged distribution builds pass the shared instant to both the Cargo WASM build and Vite staging.

**Non-Goals:**
- Changing repository URL, branch, or commit hash resolution in `tools/studio_build.py`.
- Altering user-local time zone formatting (`localBuildTime`).
- Introducing new external crate dependencies.

## Decisions

### 1. Embed build instant via `crates/wasm/build.rs`
- Add `build.rs` to `crates/wasm`.
- Check `env::var("NTESIM_BUILD_STARTED_AT")` and emit `cargo:rerun-if-env-changed=NTESIM_BUILD_STARTED_AT`.
- If the environment variable is absent or empty, calculate the current UTC ISO 8601 string (`YYYY-MM-DDTHH:MM:SS.mmmZ`) using `std::time::SystemTime::now()` and pure integer civil date math.
- Emit `cargo:rustc-env=NTESIM_WASM_BUILD_TIME=<instant>`.
- In `crates/wasm/src/model.rs`, read `env!("NTESIM_WASM_BUILD_TIME")`.
- *Alternatives considered:* Relying on Python to write generated source files into `src/` (rejected: would dirty the git worktree and complicate standalone Cargo builds).

### 2. Transport and client extraction
- Add `build_started_at: String` to `WasmVersionInfo`.
- Support `"build_started_at"` action in `dispatch_transport` and `dispatch_invoke`.
- Add `buildStartedAt()` to `crates/wasm/js/runtime.js` and `web/src/infrastructure/wasm/client.ts`.
- *Alternatives considered:* Separate binary custom section in WASM (rejected: requires wasm-level parsing tools, whereas transport/version already provides clean, tested ownership-safe communication).

### 3. Studio UI integration
- Update `web/src/app/BuildIdentity.tsx` to accept an optional `wasmBuildTime?: string`.
- Inside `BuildIdentity`, if `wasmBuildTime` is not provided, resolve it asynchronously from `loadEngine().then(e => e.buildStartedAt())`.
- In `web/src/features/editor/useConfig.ts`, track and return `wasmBuildTime` from engine initialization and pass it to `App.tsx`.
- Fall back to `build.build_started_at` while the WASM engine is loading or if loading fails.
- *Alternatives considered:* Blank or loading spinner in header (rejected: causes jarring layout shift and breaks existing SSR/static markup snapshots).

### 4. Distribution consistency
- In `tools/browser_distribution.py:build`, set `os.environ[INSTANT] = instant` prior to invoking `cargo(...)`.
- *Rationale:* Ensures packaged releases have matching timestamps across `build-info.json` and the bundled `ntesim-wasm` binary.

## Risks / Trade-offs

- **[Incremental compilation freshness]** → When `NTESIM_BUILD_STARTED_AT` is not set, Cargo re-runs `build.rs` only when crate sources or dependencies change. Rebuilding an unchanged worktree does not rebuild `ntesim-wasm`, preserving caching.
- **[Brief initial fallback timestamp]** → The web GUI displays `build.build_started_at` for milliseconds until WASM finishes compiling/streaming. Once `loadEngine()` resolves, it seamlessly updates to the WASM timestamp without layout shift.
