# Spec Delta

## MODIFIED Requirements

### Requirement: Precomputed team DPS ranking

Team list SHALL display its last complete supported precomputation, sorted by numeric DPS descending with stable identity for ties. Browsing SHALL NOT simulate. DPS refresh SHALL occur only on explicit request or during release/publication preparation, not on local edits, rebuilds or routine checks. Failed, incomplete and unsupported calculated results SHALL remain unavailable. Recorded source/engine identities SHALL remain truthful.

#### Scenario: Reproduce a displayed DPS value
- **WHEN** a team has a complete supported precomputation
- **THEN** its displayed DPS equals the shared engine's mean of completed per-trial damage rates using actual elapsed combat time
- **AND** the calculation retains the source and engine identity needed to reproduce it

#### Scenario: Reject stale or unsuccessful rankings
- **GIVEN** the script or engine has changed, or a calculation fails, remains incomplete or reaches an unsupported damage dependency
- **WHEN** rankings are prepared for release or publication
- **THEN** all delivered selectable scripts are recalculated with the current engine and inputs before delivery
- **AND** an old or partial DPS value cannot substitute for a failed, incomplete or unsupported fresh result
- **AND** the scenario remains discoverable and loadable with unavailable DPS and its reason when a fresh result is unsuccessful
- **AND** a valid zero-DPS calculation remains numeric zero rather than unavailable

#### Scenario: Continue ordinary development with last-calculated DPS
- **GIVEN** a structurally valid catalog has a successful precomputation from an earlier source or engine
- **WHEN** local sources change, WASM is rebuilt, or ordinary development/build/check commands run
- **THEN** the last-calculated numeric DPS may remain visible with its recorded calculation identity
- **AND** those operations do not recalculate DPS or fail solely because calculation identities differ from current sources

#### Scenario: Explicit recalculation
- **GIVEN** a user explicitly requests DPS refresh
- **WHEN** the requested calculation completes against the current engine and inputs
- **THEN** the catalog is updated with the new outcomes and their actual calculation identities
- **AND** failed, incomplete and unsupported outcomes remain unavailable instead of retaining a previous successful number for that refreshed entry

## ADDED Requirements

### Requirement: Calculation cache follows actual inputs

An explicitly requested calculation SHALL reuse cached results only for matching scenario inputs, options, engine, WASM and generator identities. Release/publication preparation SHALL recalculate delivered entries without relying on prior calculation cache. Input changes alone SHALL NOT schedule recalculation.

#### Scenario: Reuse identical requested calculation
- **GIVEN** a cached result matches all calculation inputs and identities
- **WHEN** an ordinary explicit refresh requests that calculation without forcing a new run
- **THEN** the matching cached result may be reused

#### Scenario: Refresh after an engine or generator change
- **GIVEN** a previous result was calculated with different engine, WASM, generator or scenario inputs
- **WHEN** a new explicit refresh requests that calculation
- **THEN** the previous result cannot satisfy the request as a matching cached calculation
- **AND** merely changing the inputs before that request did not launch recalculation

#### Scenario: Prepare release using a warm cache
- **GIVEN** earlier calculation cache entries exist
- **WHEN** release/publication preparation refreshes the catalog
- **THEN** every delivered selectable entry is freshly calculated with the current engine and inputs
- **AND** prior cached success cannot conceal a failed, incomplete or unsupported fresh result
