# Spec Delta: combat-execution

## ADDED Requirements

### Requirement: Normalized report provenance and tables

Combat reports SHALL store unique evidence items, packet metadata templates, buff definitions, and resolved model assumption descriptions in shared report catalogs rather than repeating static prose or identical metadata objects per command or hit.

#### Scenario: Repeated ability hits and talent projections
- **GIVEN** a combat scenario executes multiple actions and hits from the same ability and damage effect
- **WHEN** the native report is serialized
- **THEN** each unique evidence record, metadata template, and assumption description is stored once in its shared catalog and referenced by identifier or index
- **AND** identical static action ledger and talent projection entries group their matching `command_indices`

### Requirement: Delta state timeline frames

Recorded `state_timeline` observation frames SHALL retain complete `Initial` and `Final` boundary snapshots and emit incremental actor field deltas and buff diff operations on intermediate `Event` boundaries.

#### Scenario: Reconstructing timeline state across event boundaries
- **GIVEN** an intermediate event updates actor energy carry and adds or refreshes a buff stack
- **WHEN** the resulting `state_timeline` is validated and replayed sequentially from `Initial` to `Final`
- **THEN** intermediate `Event` frames record only changed actor fields and buff `upsert`/`remove` operations referencing `buff_catalog`
- **AND** the reconstructed final actor and buff states exactly match the `Final` snapshot and final ledgers

### Requirement: Data-only structured trace events

Trace entries SHALL record language-neutral structured event kinds, references, and numeric payloads without human-readable prose strings, and SHALL group simultaneous hit-driven energy grants sharing the same source basis.

#### Scenario: Multi-recipient hit energy fanout
- **GIVEN** one settled hit distributes ordinary energy across four party members at the same timestamp
- **WHEN** trace entries are emitted
- **THEN** the grant is recorded as one grouped fanout trace event containing all recipient ledgers without per-recipient prose sentences
- **AND** energy conservation and replay ledgers reconcile identically across all recipients

### Requirement: Canonical exact rational strings

Exact rational values in combat requests, reports, and wire payloads SHALL serialize and deserialize canonically as `"<numerator>/<denominator>"` decimal strings without object wrappers or loss of arbitrary integer precision.

#### Scenario: Serializing and parsing exact damage and rates
- **GIVEN** a scenario contains fractional multipliers and exact damage exceeding JavaScript safe-integer bounds
- **WHEN** the request is parsed and the report is serialized to JSON
- **THEN** exact rationals use canonical `"<numerator>/<denominator>"` strings retaining all numerator and denominator digits
- **AND** `ExactFraction`, `Ratio`, and `WideFraction` values reject legacy `{numerator, denominator}` object shapes
