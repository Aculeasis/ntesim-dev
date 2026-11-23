# Spec Delta

## ADDED Requirements

### Requirement: Explicit purchased Support Skill level

Every script character declaration SHALL require one explicit `qte=<integer>` purchased Support Skill level within the legal per-character progression range. The existing `talent` tuple SHALL continue to declare Normal, Skill and Burst. Compilation SHALL reject missing, duplicate or invalid QTE values with a source diagnostic.

#### Scenario: Author a distinct QTE level
- **GIVEN** a character declaration has legal `talent=8,8,8 qte=6`
- **WHEN** the script is compiled and executed or exported
- **THEN** the native scenario retains purchased QTE level 6 separately from the three purchased talent levels
- **AND** the effective QTE uses that level and the applicable owner progression bonuses

#### Scenario: Omit the QTE level
- **GIVEN** a character declares `talent=8,8,8` and omits `qte`
- **WHEN** the script is compiled
- **THEN** compilation fails with a diagnostic identifying the character declaration and missing field

#### Scenario: Duplicate or illegal QTE level
- **WHEN** a character declaration repeats `qte` or supplies a malformed or out-of-range QTE level
- **THEN** compilation fails with a diagnostic identifying the invalid declaration
