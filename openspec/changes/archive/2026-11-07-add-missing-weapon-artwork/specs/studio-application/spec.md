# Spec Delta

## ADDED Requirements

### Requirement: Catalog weapon artwork coverage

Studio SHALL provide bundled WebP artwork assets for all catalog weapons (Arcs), mapping each weapon identifier to its valid WebP asset without fallback placeholders.

#### Scenario: Equipping catalog weapons
- **WHEN** an actor in the party equips any of the 51 catalog weapons
- **THEN** the character card and weapon inspector render the corresponding bundled WebP artwork
- **AND** the fallback placeholder is not displayed
