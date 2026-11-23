# 0001: Rust product engine shared by native and browser

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision date: unknown
- Current owners: [architecture](../ARCHITECTURE.md#one-engine), [requirements](../../openspec/specs/product-scope/reference.md), [WASM parity](../../openspec/specs/wasm-transport/reference.md)

## Context and decision

The simulator needs reproducible combat behavior through CLI and browser. Rust is the product engine for both, composed through native and WASM adapters. Python remains available for acquisition, preparation and independent audits; partial Python models are evidence rather than another maintained product engine.

Consistent combat semantics is the requirement. Rust and shared compilation are architectural choices used to meet it; neither the language nor compilation alone establishes accurate game mechanics.

## Rationale

Sharing gameplay code avoids maintaining two implementations whose timing, resource and damage behavior could diverge. Rust was chosen as the product engine because it compiles directly to native binaries for high-performance CLI/batch workloads and to WebAssembly (WASM) for client-side browser execution. Rust's strict type system and ownership model prevent runtime memory corruption in complex event-driven simulation. Python remains available for offline data acquisition, asset preparation, and independent verification audits.

## Consequences and assumptions

Rust's compiler checks help constrain implementation, while simulation logic still needs independent numerical and causal verification. Ownership/type complexity and the native/WASM toolchain are real maintenance costs. Python audits should remain independent rather than merely repeat engine code.

This choice assumes the shared engine can support the required browser execution and practical performance across both targets. Browser convenience does not justify a second stat, timing or damage implementation.

## Reconsider when

Measured compiler/toolchain friction, unsupported target capabilities or sustained implementation difficulty materially obstruct the usable baseline. Compare a concrete alternative and migration cost before proposing a language change; preserve reproducibility and native/browser semantic parity. A preference for a familiar language alone does not establish a project benefit.

## Verification

Use the existing [architecture checks](../../tests/test_architecture.py) for dependency boundaries and [native/WASM verification contract](../../openspec/specs/wasm-transport/reference.md) for shared behavior, including actual browser consumers when affected. Compiler success cannot replace independent model checks or establish full kit coverage.
