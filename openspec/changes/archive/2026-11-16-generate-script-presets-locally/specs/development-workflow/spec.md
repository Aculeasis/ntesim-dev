## ADDED Requirements

### Requirement: Single maintained Studio preset source

Studio preset definitions SHALL be maintained in `presets/studio/index.json` and its referenced `.nte` files. The derived browser bundle `web/src/generated/script-presets.json` SHALL be ignored by Git and absent from tracked repository inputs. Generating that bundle SHALL NOT require editing another copy of a preset.

#### Scenario: Track a preset edit
- **GIVEN** a developer changes a maintained Studio `.nte` file or its manifest metadata
- **WHEN** the browser preset bundle is regenerated
- **THEN** its entries contain the current manifest metadata and exact script text
- **AND** the generated JSON does not appear among untracked or modified Git files

#### Scenario: Fresh checkout contains maintained sources only
- **WHEN** the repository is checked out without local generated artifacts
- **THEN** the Studio preset manifest and referenced `.nte` files are present
- **AND** no tracked copy of `web/src/generated/script-presets.json` is required

### Requirement: Automatic browser preset preparation

Supported Studio build, development and web validation commands SHALL prepare the browser preset bundle from maintained sources before any consumer reads it. A missing bundle SHALL require no manual preparation. Preparation SHALL preserve preset identity, ordering, labels, selectability, default selection and exact script text, without triggering DPS recalculation.

#### Scenario: Build without a preexisting bundle
- **GIVEN** a checkout has documented build prerequisites and no generated browser preset bundle
- **WHEN** the ordinary Studio build runs
- **THEN** it generates the bundle before compiling its consumers
- **AND** the built site loads the exact maintained source of each selected preset without access to the repository filesystem

#### Scenario: Validate without a preexisting bundle
- **GIVEN** a checkout has documented validation prerequisites and no generated browser preset bundle
- **WHEN** a supported web type check, format check, test or Team list validation command runs
- **THEN** the command prepares the bundle before a consumer needs it
- **AND** it does not fail solely because that generated file was initially absent
- **AND** it does not recalculate saved Team list DPS

#### Scenario: Develop and edit a preset
- **GIVEN** a checkout has documented development prerequisites and no generated browser preset bundle
- **WHEN** the Studio development server starts and a maintained preset is subsequently edited
- **THEN** initial loading succeeds and the served preset source updates to the edited text
- **AND** neither operation requires committing generated JSON or recalculating DPS

#### Scenario: Invalid maintained input
- **GIVEN** the preset manifest is invalid or a referenced script is missing or empty
- **WHEN** browser preset preparation runs
- **THEN** the command or development server reports the input error
- **AND** it does not silently serve an old generated bundle as the current preset collection
