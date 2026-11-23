# 0008: Architecture consolidation, boundary hygiene and tooling refactoring

- Status: accepted
- Verification schedule: superseded by [Check selection](../DEVELOPMENT.md) on 2026-10-26; other consolidation choices remain accepted
- Recorded: 2026-10-20
- Decision date: 2026-10-20
- Current owners: [architecture](../ARCHITECTURE.md#one-engine), [requirements](../../openspec/specs/product-scope/reference.md), [OpenSpec changes](../../openspec/README.md), [agent guide](../../AGENTS.md)
- Related records: [0001](0001-rust-product-engine.md), [0003](0003-shared-services-and-content-owners.md), [0006](0006-sequential-rotation-language.md)

## Context and decision

Following the comprehensive project review and explicit human authorization on 2026-10-20, the project establishes a mandatory consolidation and hygiene milestone prior to resuming full-scale gameplay mechanics implementation. The review revealed accumulated architectural debt:
1. Dual domain logic: A shadow compiler, parser, AST, runtime, and stat calculators in TypeScript (`web/src/domain/language/`), accompanied by 92 redundant JavaScript combat tests in `web/tests/combat/`.
2. Binary bloat: Multi-sentence English hypothesis texts embedded directly as string constants inside Rust source files (`crates/content/src/characters/*/skill.rs`, weapons).
3. Dead code: Obsolete 3D coordinate and radius geometry in `crates/core/src/targets/enemy/geometry.rs`, violating the 0D dummy all-in-range product contract.
4. Tooling sprawl: Over 70 individual `verify_*.py` and `verify_*.mjs` scripts in `tools/` with copy-pasted verification boilerplate.
5. Checkpoint friction: An artificial 1000-line test file limit forcing arbitrary module splitting and baseline exceptions.

### Approved Decisions
1. **Single Source of Truth (Rust DSL & Math):** Purge `web/src/domain/language/` and `web/tests/combat/*.mjs`. Rust exclusively owns DSL parsing, AST, validation, stat projection, and combat simulation. The Web Studio becomes a pure presentation layer interacting through WASM.
2. **Hypothesis Text Extraction:** Extract all multi-sentence hypothesis strings out of Rust source code into structured metadata (`data/combat/hypotheses.json`). Rust files reference hypotheses strictly by typed identifiers (`HypothesisId`).
3. **Dead Code Elimination & Crate Hygiene:** Purge 3D geometry from `ntesim-core`. Keep `core` strictly as a generic event-driven simulation engine.
4. **Tooling Consolidation with Strict CPU Safety:** Consolidate `tools/verify_*.py` scripts into a unified CLI / test harness. Remove the artificial 1000-line test module ceiling. **Preserve the strict load limit of <= 4 logical cores across all tools and child processes** to prevent workstation slowdown during autonomous agent runs.
5. **WASM Transport Modernization:** Migrate high-frequency batch simulation and timeline event transport to typed bindings and binary/zero-copy serialization.

### Explicitly Rejected Proposals (Preserved Requirements)
- **Vertical Slices Rejected:** Incremental vertical slices across a single team were explicitly rejected by human decision. The project will implement full mechanics across all 23 characters, 51 weapons, and 12 equipment sets first, followed by rotation execution, tuning, and calibration.
- **DSL Profile Splitting Rejected:** Splitting DSL into separate GUI configuration profiles (JSON/TOML) was explicitly rejected. The monolithic, all-in-one GCSim-style DSL (containing build options, target stats, and action sequences in one shareable script) is an intentional core feature.

## Rationale

- Eliminating TypeScript domain duplication restores single responsibility, cuts maintenance in half, and eliminates divergence bugs.
- Removing hardcoded hypothesis prose cuts WASM/binary footprint and cleanly separates documentation from executable logic.
- Eliminating dead 3D geometry prevents confusion regarding native spatial radii.
- Consolidating 70+ scripts into a unified CLI simplifies continuous integration and maintenance.
- Keeping the CPU limit <= 4 cores ensures development and background agent runs do not freeze the developer's computer.
- Retaining all-in-one GCSim-style DSL preserves script portability, shareability, and version control simplicity.

## Consequences and assumptions

- Phase 1 (Q0.1 - Q0.5) must be completed before new character, weapon, or set mechanics are added to `crates/content`.
- Web Studio components must be updated to consume WASM diagnostics and compilation results.
- Full hypothesis audit trails remain fully preserved in `data/combat/hypotheses.json` and `docs/mechanics/`.

## Reconsider when

- A proposed change reintroduces domain logic into TypeScript or splits the monolithic DSL script into separate files without human authorization.
- System CPU limits exceed 4 logical cores during automated runs.

## Verification

- `crates/wasm` exports validated by web editor smoke tests and Vitest WASM runner.
- Zero duplicate compiler/stat files in `web/src/domain/language/`.
- Zero 3D geometry types in `crates/core`.
- Zero multi-sentence hypothesis constants in `crates/content/src/`.
- All verification tools pass under `tools/cpu_limit.py` (<= 4 CPU cores).

## Implementation clarification (2026-10-20)

Q0.5 uses version-matched `wasm-bindgen` 0.2.128 and an interned self-describing binary report format, with generated declarations and one shared browser/Node client. Generated glue returns owned Uint8Array bytes and releases Rust storage; one copy remains at this ownership boundary. Pre-serialized static provenance retains pooled raw JSON; timeline and observation records use binary tags. This replaces the former no-generator/manual-C-ABI mechanism in the [WASM contract](../../openspec/specs/wasm-transport/reference.md#2-typed-bindings-and-binary-transport), without changing report schemas, exact arithmetic or the monolithic DSL.

## Current workflow

[OpenSpec](../../openspec/README.md) owns selected changes and accepted capabilities. [Development](../DEVELOPMENT.md#choosing-checks) selects current native, API and browser checks. Mechanics tests belong to Rust; Python checks retain distinct CLI, package and transport integration risks.
