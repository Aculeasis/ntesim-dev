# Spec Delta

## MODIFIED Requirements

### Requirement: Studio tools and script import

Studio SHALL provide a non-modal Tools dropdown menu with adaptive viewport placement and an indicator arrow, support importing script and JSON export files up to 50 MB, and display team cards and editor presets without redundant script line pointers or initial resource panels.

#### Scenario: Tools dropdown menu
- **WHEN** the Tools button is clicked
- **THEN** a dropdown menu appears adaptively positioned above or below the button to remain within the visible viewport
- **AND** selecting an action or clicking outside dismisses the menu without opening a full-page modal

#### Scenario: Large report import
- **WHEN** a user imports a script or JSON report export with source up to 50 MB
- **THEN** the file is accepted and normalized for execution
- **AND** files exceeding 50 MB are rejected with an explicit error
