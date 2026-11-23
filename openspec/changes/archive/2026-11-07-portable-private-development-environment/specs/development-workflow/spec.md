# Spec Delta

## ADDED Requirements

### Requirement: Optional private development configuration

Shared instructions SHALL identify one optional ignored location for machine-specific supplements without replacing or duplicating shared instructions. Private paths, permission grants, proxy endpoints and credentials SHALL remain outside versioned project inputs. Ordinary development checks SHALL work without private configuration.

#### Scenario: Checkout without local supplements
- **GIVEN** a supported machine has the documented prerequisites and no private configuration
- **WHEN** a developer follows shared instructions
- **THEN** ordinary setup and checks do not require the author's directories, credentials or permission grants

#### Scenario: Local instructions supplement shared rules
- **GIVEN** optional local instructions exist in the documented ignored location
- **WHEN** an agent starts work in this checkout
- **THEN** it reads those supplements alongside the shared guide without requiring an override copy of that guide

### Requirement: Explicit external acquisition inputs

Tools requiring a local client installation SHALL resolve its binary directory from an explicit environment setting and SHALL fail with an actionable configuration error before reading client files when that setting is missing or invalid. Explicitly selected environment files SHALL be parsed as data without shell execution or implicit variable expansion; inherited environment values SHALL take precedence.

#### Scenario: Missing client location
- **WHEN** native acquisition is requested without a configured binary directory
- **THEN** the tool names the required setting and does not probe an assumed installation

#### Scenario: Environment file contains shell syntax
- **GIVEN** an explicitly selected environment file contains a value with shell metacharacters
- **WHEN** the command runner loads that file
- **THEN** the value is passed literally to the child and no shell expression is executed

### Requirement: Repository portability and privacy validation

Checkpoint SHALL reject commit-eligible private configuration, machine-specific absolute paths and recognizable credential literals, reporting file, line and category without echoing the private value. Synthetic path-rejection fixtures and public attribution SHALL remain valid. Evidence metadata SHALL retain source identity without recording the developer's filesystem layout.

#### Scenario: Forced addition of ignored private configuration
- **GIVEN** a local environment or permission file is forcibly added to Git
- **WHEN** checkpoint validates repository inputs
- **THEN** validation fails even though the file normally matches an ignore rule

#### Scenario: Machine path in evidence metadata
- **WHEN** source metadata contains a developer's absolute filesystem path
- **THEN** checkpoint rejects it without printing that path's value
