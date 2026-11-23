# Spec Delta: studio-application

## ADDED Requirements

### Requirement: Deduplicated trial report exports

Studio report exports SHALL serialize the executed source, request, aggregate, samples, and sample report once without duplicating the delivered sample execution under a second `selected_sample` payload.

#### Scenario: Exporting a completed trial run
- **WHEN** a user exports the trial results bundle from Studio
- **THEN** the exported JSON omits the redundant `selected_sample` duplicate while preserving the complete executed run bundle
- **AND** re-importing the exported file restores the executed source, aggregate statistics, samples, and sample playback without data loss

### Requirement: UI-owned event localization

Studio playback and event inspection SHALL localize actor labels, action names, and event details exclusively from structured report receipts and trace data without parsing prose text from trace events.

#### Scenario: Rendering localized combat events
- **GIVEN** a combat report with structured trace events, hit receipts, healing receipts, and grouped energy fanouts
- **WHEN** Studio builds playback events for display and search
- **THEN** event actor, title, and detail text are formatted in UI localization helpers directly from structured fields and catalog templates

### Requirement: Normalized report and rational consumption

Studio report validators, energy and target auditors, and timeline playback SHALL validate and reconstruct normalized report catalogs, delta state frames, grouped energy fanout entries, and `"<numerator>/<denominator>"` exact rationals.

#### Scenario: Validating and scrubbing a compact report
- **GIVEN** a report using cataloged provenance, delta timeline frames, grouped trace fanouts, and `"numerator/denominator"` exact rationals
- **WHEN** Studio imports or runs the scenario and scrubs through playback frames
- **THEN** observation validation enforces catalog and assumption invariants, reconstructs every actor and buff snapshot cleanly, and parses wide rationals with exact `BigInt` precision
