# Spec Delta

## MODIFIED Requirements

### Requirement: Viewer-local build timestamp

Studio SHALL replace `Global · 04 Oct 2026` with the build-start instant rendered in the viewer's current time zone as zero-padded `dd.mm.yy · hh:mm` using a 24-hour clock. When loaded, that instant SHALL reflect the compiled WebAssembly engine's embedded build instant extracted by the client, falling back to the web distribution instant beforehand. That instant SHALL remain fixed for the loaded engine, independently of the viewer's page-open time, selected UI locale or simulation state.

#### Scenario: Same build in different time zones
- **GIVEN** a build started at `2026-11-14T10:20:30Z`
- **WHEN** it is viewed in Europe/Istanbul and UTC
- **THEN** the respective timestamps are `14.11.26 · 13:20` and `14.11.26 · 10:20`
- **AND** both represent the same build instant with daylight-saving and date boundaries determined by the viewer's zone

#### Scenario: Reload an existing build
- **GIVEN** a built Studio artifact or development server is already available
- **WHEN** a viewer reloads it later without a new build or server restart
- **THEN** the timestamp still represents that artifact's or session's original build-start instant
- **AND** it does not become a ticking clock or the commit date

#### Scenario: Engine build instant extracted from WebAssembly
- **GIVEN** an active Studio session loads a WebAssembly engine binary compiled at `2026-11-21T02:00:00Z` while the static web bundle was served at `2026-11-21T04:00:00Z`
- **WHEN** the engine finishes loading
- **THEN** Studio's header displays the engine's embedded build instant `21.11.26 · 02:00` in UTC
- **AND** the semantic time element reflects the WebAssembly engine's embedded build instant
