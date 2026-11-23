# Spec Delta

## ADDED Requirements

### Requirement: Studio build source navigation

Studio SHALL replace the upper-right static game title with a small GitHub icon linking to `https://github.com/Aculeasis/ntesim-dev` and a separately linked build identifier opening that repository at the build's commit or explicit deployment tag. Both links SHALL have accessible names and keyboard focus. The header SHALL remain usable without horizontal overflow on narrow screens.

#### Scenario: Local main build
- **GIVEN** main has checked-out HEAD `bb887a5` and may contain uncommitted edits
- **WHEN** Studio is built or its development server starts locally
- **THEN** the identifier is `bb887a5` and links to `https://github.com/Aculeasis/ntesim-dev/tree/bb887a5`
- **AND** it identifies HEAD rather than HEAD's parent, a guessed future commit or a tag discovered locally

#### Scenario: Local branch build
- **GIVEN** the checked-out branch is `some/branch` at HEAD `bb887a5`
- **WHEN** Studio is built or its development server starts locally
- **THEN** the identifier is `some/branch/bb887a5`
- **AND** its link opens the commit tree rather than the moving branch tip

#### Scenario: Local tag checkout
- **GIVEN** local HEAD is detached at a commit bearing tag `v1.2.3`
- **WHEN** Studio starts locally without an explicit deployment identity
- **THEN** it displays the commit's short hash without a fabricated branch prefix or tag label

#### Scenario: Explicit deployment tag
- **GIVEN** a deployment builds the validated tag `v1.2.3`
- **WHEN** its Studio header renders
- **THEN** the identifier is `v1.2.3` without a branch or hash suffix
- **AND** its link opens `https://github.com/Aculeasis/ntesim-dev/tree/refs/tags/v1.2.3`
- **AND** tag names containing slashes or reserved URL characters retain their exact ref meaning in the link

#### Scenario: Keyboard navigation and narrow header
- **WHEN** the header is navigated by keyboard or viewed at a 390 pixel viewport width
- **THEN** the repository icon and version link are separately reachable with visible focus
- **AND** a long branch or tag name remains inspectable without causing page-level horizontal overflow
- **AND** the icon/ref and timestamp occupy adjacent rounded, bordered pills of equal height

### Requirement: Viewer-local build timestamp

Studio SHALL replace `Global · 04 Oct 2026` with the build-start instant rendered in the viewer's current time zone as zero-padded `dd.mm.yy · hh:mm` using a 24-hour clock. That instant SHALL be fixed for the served build or development-server session, independently of the viewer's page-open time, selected UI locale or simulation state.

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

## MODIFIED Requirements

### Requirement: Precomputed team DPS ranking

Team list SHALL display its last complete supported precomputation, sorted by numeric DPS descending with stable identity for ties. Browsing SHALL NOT simulate. DPS refresh SHALL occur only on explicit request or during explicit release preparation, not on local edits, rebuilds or routine checks. Failed, incomplete and unsupported calculated results SHALL remain unavailable. Recorded source/engine identities SHALL remain truthful.

#### Scenario: Reproduce a displayed DPS value
- **WHEN** a team has a complete supported precomputation
- **THEN** its displayed DPS equals the shared engine's mean of completed per-trial damage rates using actual elapsed combat time
- **AND** the calculation retains the source and engine identity needed to reproduce it

#### Scenario: Reject stale or unsuccessful rankings
- **GIVEN** the script or engine has changed, or a calculation fails, remains incomplete or reaches an unsupported damage dependency
- **WHEN** rankings are explicitly refreshed for release preparation
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

#### Scenario: Deploy Pages with saved rankings
- **GIVEN** the checkout contains a structurally valid committed Team list
- **WHEN** the GitHub Pages workflow builds and deploys Studio
- **THEN** it reuses the committed DPS outcomes and their recorded calculation identities without recalculation
- **AND** current WASM compilation does not trigger a catalog refresh

### Requirement: Calculation cache follows actual inputs

An explicitly requested calculation SHALL reuse cached results only for matching scenario inputs, options, engine, WASM and generator identities. Explicit release preparation SHALL recalculate delivered entries without relying on prior calculation cache. Input changes alone SHALL NOT schedule recalculation.

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
- **WHEN** explicit release preparation refreshes the catalog
- **THEN** every delivered selectable entry is freshly calculated with the current engine and inputs
- **AND** prior cached success cannot conceal a failed, incomplete or unsupported fresh result
