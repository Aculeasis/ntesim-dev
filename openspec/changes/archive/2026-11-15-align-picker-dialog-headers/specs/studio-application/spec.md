## ADDED Requirements

### Requirement: Compact picker dialog headers

Name Search dialogs SHALL display search and an icon close button on one horizontally aligned row, with close immediately to the right and no visible category heading. Each dialog and close button SHALL retain an accessible name. Team list search SHALL be vertically centered with its neighboring header controls on desktop.

#### Scenario: Open a reference picker
- **WHEN** any Name Search category is opened on desktop or at 390 pixel viewport width
- **THEN** search and the close icon appear on one row without horizontal overflow or a visible category heading
- **AND** search, insertion and dismissal remain usable

#### Scenario: Open Team list on desktop
- **WHEN** Team list is opened at desktop width
- **THEN** the title, search, include/exclude buttons and close icon share a vertical center
