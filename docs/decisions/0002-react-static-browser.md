# 0002: React studio with static browser execution

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision dates: unknown
- Current owners: [web architecture](../WEB_ARCHITECTURE.md), [browser contract](../../openspec/specs/studio-application/reference.md), [requirements](../../openspec/specs/product-scope/reference.md)

## Context and decision

Use the existing React/TypeScript studio to configure, run and inspect simulations. A Worker invokes the shared Rust/WASM engine; deployment is a static application with local persistence and import/export. The current scope excludes a simulation service, accounts and a server database.

React is a framework choice. Static local execution is a separate product/deployment boundary; the confirmed reason for React does not explain every hosting or toolchain choice.

## Rationale

Static browser execution eliminates the operational overhead, hosting complexity, and network latency of a dedicated backend simulation service. Running the simulation client-side via WebAssembly provides instant feedback and offline capability. React and TypeScript provide a component-driven architecture for complex UI configuration (build editors, timeline views, damage breakdowns), while executing simulations in dedicated Web Workers prevents long-running calculations from blocking the user interface.

## Consequences and assumptions

React does not own combat semantics; it configures and visualizes simulation state. Browser target limits, Worker communication, local storage failures and offline asset updates still need explicit handling. Static deployment assumes the required workflow can execute on the user's device; shared accounts and server persistence are outside current scope.

## Reconsider when

Concrete UI maintenance or accessibility problems justify evaluating a framework change, or required workload measurements show browser execution is insufficient. A server-dependent product feature would first require an explicit scope revision. Similarity to gcsim alone is insufficient reason to introduce one of its features or dependencies.

## Verification

Use the [web verification boundaries](../WEB_ARCHITECTURE.md#verification-boundaries), browser contract and [WASM parity contract](../../openspec/specs/wasm-transport/reference.md). A replacement must preserve authored inputs, report quality markers, Worker behavior and applicable offline/import/export workflows. Passing UI checks does not validate the combat model.
