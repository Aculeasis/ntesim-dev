# Spec Delta

## MODIFIED Requirements

### Requirement: Dual-clock execution agreement

Native CLI, WASM and Worker execution SHALL agree on relative limits, absolute timestamps, pause kinds, timer-policy identity, retained timer state, both elapsed durations, canonical relative DPS and both report summary bases. Sample, compact batch, replay and resume SHALL preserve this agreement and stopping causes without deriving gameplay pause rules or shield-capacity integrals in TypeScript.

#### Scenario: Paused execution across transport boundaries
- **GIVEN** identical source, seed and limits for rotations using ordinary Ultimate cinematic pauses and Hotori's extended Time Stop
- **WHEN** native, WASM and Worker consumers execute it
- **THEN** both elapsed totals, exact damage, pause boundaries, pending effects and relative DPS agree
- **AND** ordinary cooldown/buff remainders, paused DoT phase, deployed-object expiry and B2 bud exceptions agree under the selected timer policy
- **AND** loading that seed as a detailed sample agrees with the durations previously delivered in its compact observation
- **AND** relative/absolute DPS, retained/granted EPS, HPS and correctly weighted SHP agree with their native per-trial and aggregate values

#### Scenario: Resume verifies both elapsed durations
- **GIVEN** a stopped multi-seed run has a delivered prefix containing timer pauses
- **WHEN** a fresh Worker reconstructs the prefix before continuing
- **THEN** Rust verifies both duration values and their relative-DPS relationship for every retained seed
- **AND** the timer-policy identity and recomputed paused-state effects belong to the same retained execution contract
- **AND** altered duration data fails resume verification without becoming a successful continuation
- **AND** both summary bases, including relative SHP, participate in retained-prefix verification

#### Scenario: Current clock contract is required
- **WHEN** source artifacts, requests or reports use an obsolete single-clock contract or internally inconsistent dual-clock data
- **THEN** the affected import or replay boundary rejects them with a readable unsupported-contract or invalid-data diagnostic
- **AND** no compatibility shim guesses a relative duration or reconstructs missing native pauses from UI action names
- **AND** current portable result contracts require both summary bases rather than adapting obsolete single-basis summary fields

#### Scenario: Exact interval and elapsed validation
- **WHEN** clock data crosses native JSON, binary WASM transport or Worker report restoration
- **THEN** integer microseconds and existing exact rational damage retain their precision and validate against the same clock invariants
- **AND** invalid interval bounds, union totals or zero-relative-duration rate encodings cannot silently become finite relative DPS
- **AND** zero-denominator relative EPS, HPS and SHP remain unavailable through the same boundaries

#### Scenario: Basis selection uses delivered values
- **GIVEN** a restored or newly completed run has valid native data for both bases
- **WHEN** the user changes the Results time basis
- **THEN** presentation selects the corresponding delivered values without a simulation, replay or sample regeneration request
- **AND** damage, resource totals, canonical comparison eligibility and absolute timeline data remain unchanged
