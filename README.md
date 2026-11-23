> **Disclaimer:** This simulator is rough, incomplete, and probably wrong in plenty of places. Using it to compare real in-game DPS is about as reliable as measuring the weather on Mars from your window. Treat the numbers as experiments, not facts.

# ntesim — Neverness to Everness Simulator

An open-source combat and DPS simulator for **Neverness to Everness (NTE)**. A single Rust engine powers both the native CLI and the React/TypeScript browser studio via WebAssembly in a Web Worker, providing deterministic and reproducible team DPS calculations under declared combat scenarios.

[Open Studio on GitHub Pages](https://aculeasis.github.io/ntesim-dev/)

---

## Development

The native CLI and React Studio share one Rust engine. Supported mechanics execute under explicit scenario assumptions; a feature's presence is not a claim of complete native game fidelity. Current source/tests and model limits define executable support.

Work is organized with [OpenSpec](openspec/README.md): accepted capability requirements, focused changes and verifiable tasks. Use `python tools/openspec.py view` for the terminal dashboard; [development](docs/DEVELOPMENT.md#openspec-workflow) explains setup and continuation.

## Core Simulation Principles

- **Stationary Training Dummy Model**: Targets are stationary dummies with explicitly authored combat stats and scenario assumptions (boss eligibility, incoming damage, parry/dodge windows). No spatial geometry, enemy pathfinding, or enemy AI simulation is needed.
- **Event-Driven Combat Timeline**: A single real combat-time axis driven by discrete event advancement rather than polling. Hit delays, animation windows, cast recoveries, character switching, and Hotori Time Stop are modeled with causal fidelity.
- **DPS Relevance Boundary**: Strict modeling applies to all factors directly or indirectly affecting team damage (multipliers, stat zones, Fons/energy, reactions, buff durations, action availability). Non-DPS elements are simplified or omitted with explicit, documented hypotheses.
- **GCSim-Style NTE DSL**: Team configurations, character builds, target stats, and combat action sequences are defined in a human-readable, shareable `.nte` script.

---

## Getting Started

### Prerequisites

- **Python 3.11+**
- **Rust 1.98.1** (pinned via rustup)
- **Node.js 22.12+ / npm** (for the web studio)

All build outputs go to `target/`, and reusable caches are kept in `cache/`. No installed game client is required.

### Building & Running Native CLI

```sh
# Build the native CLI
python tools/cargo.py build -p ntesim-cli --locked --offline

# Run workspace unit and integration tests
python tools/cargo.py test --workspace --locked --offline

# Run the automated checkpoint suite (verifies diffs and boundary checks)
python tools/checkpoint.py run
```

> **Note**: The `--offline` flag requires pre-cached dependencies. Remove `--offline` if dependency fetching is necessary.

On Windows, the binary is compiled to `target/debug/ntesim.exe`; on Linux/macOS, it is `target/debug/ntesim`.

### Running the Web Studio

```sh
# Install frontend dependencies
node tools/web.mjs install

# Start the Vite development server
node tools/web.mjs dev

# Run frontend type-check, tests, and production build
node tools/web.mjs check
node tools/web.mjs test
node tools/web.mjs build
```

The browser executes the shared Rust model locally in a Web Worker. Native/WASM/Worker checks compare current artifacts; model limitations remain explicit.

---

## CLI Usage Examples

```sh
# Compile or validate an NTE DSL script
target/debug/ntesim compile-script examples/kite-marked-cycles.nte

# Execute a game scenario JSON
target/debug/ntesim run-game examples/game-scenario.json

# Execute a synthetic rotation timeline
target/debug/ntesim run-rotation-synthetic examples/synthetic-rotation.json examples/synthetic-rotation.nte

# Enumerate maintained research variants without a raw package
target/debug/ntesim compositions
```

Detailed CLI commands, WASM benchmarks, and offline reproduction steps are documented in the [Runbook](docs/RUNBOOK.md).

---

## Project Structure

```
ntesim/
├── crates/
│   ├── contracts/   # Domain contracts, schema definitions, and typed HypothesisId catalog
│   ├── core/        # Event engine, combat timeline clock, damage formulas, stats, reactions
│   ├── content/     # Kits for 23 characters, 51 weapons (R1-R5), 12 sets, combat interactions
│   ├── data/        # Data auditing, package loaders, static build projections
│   ├── research/    # Parameter search, synthetic models, multi-seed batch runners
│   ├── cli/         # Native CLI binary (`ntesim`)
│   └── wasm/        # WASM interface binding the engine to the web worker
├── web/             # React 19 / TypeScript studio, DSL editor, timeline visualization
├── tools/           # Bounded-CPU runners (cpu_limit.py), checkpoint.py, cargo/web wrappers
├── examples/        # Example .nte scripts and scenario JSONs
├── openspec/        # Accepted behavior, active changes and archived changes
└── docs/            # Mechanics evidence, guides and architecture decisions
```

---

## Documentation

- **[Documentation Map](docs/README.md)**: Router for mechanics cards, domain contracts, and development guides.
- **[OpenSpec capabilities and changes](openspec/README.md)**: Accepted requirements, selected work and task dashboard.
- **[Product Requirements & Modeling Policy](openspec/specs/product-scope/reference.md)**: DPS relevance rules, dummy target model, and scope boundary.
- **[NTE DSL Specification](docs/NTE_DSL.md)**: Grammar, declarations, and action timeline statements.
- **[Runbook](docs/RUNBOOK.md)**: Complete development commands, quality checks, and WASM benchmarks.
- **[Rust Architecture](docs/ARCHITECTURE.md)** & **[Web Architecture](docs/WEB_ARCHITECTURE.md)**: Domain designs and boundary contracts.

---

## License

Original simulator code is licensed under the [MIT License](LICENSE). Third-party attributions and dependency notices are listed in [NOTICE](NOTICE).

*Neverness to Everness* game assets, character designs, and mechanics data remain the property of their respective copyright holders.
