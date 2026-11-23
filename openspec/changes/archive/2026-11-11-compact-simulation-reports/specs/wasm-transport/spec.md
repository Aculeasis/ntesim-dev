# Spec Delta: wasm-transport

## ADDED Requirements

### Requirement: Compact exact rational wire encoding

The WASM binary transport and JSON report endpoints SHALL represent exact rationals as canonical `"<numerator>/<denominator>"` decimal strings interned by `NTE wire v1` without safe-integer precision loss.

#### Scenario: Crossing the WASM boundary with compact rationals
- **GIVEN** a combat scenario produces exact damage and carry rationals including numerators or denominators larger than 2^53 - 1
- **WHEN** the report is encoded via `NTE wire v1` or serialized by `export_source_report` and verified by `replay_source_report`
- **THEN** each rational arrives as an exact `"<numerator>/<denominator>"` decimal string retaining full integer precision

### Requirement: Normalized compact report parity

Native CLI, WASM endpoints, and Worker transport SHALL produce identical compact normalized reports, structured trace events, and delta timeline frames with zero schema drift or backward-compatibility shims.

#### Scenario: Paired native and WASM parity comparison
- **WHEN** identical scenario inputs execute on both native CLI and WASM Workers under parity checks
- **THEN** decoded catalogs, delta timelines, grouped energy fanouts, and structured trace events match with zero epsilon differences
