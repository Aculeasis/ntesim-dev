## ADDED Requirements

### Requirement: Adler Temperance on Scorch

Each successful Scorch application or refresh SHALL draw one Temperance outcome for its target when a living Breakthrough 2 Adler is in the party. Each outcome SHALL refresh one independent 15-world-second target status. The timed status SHALL survive source field exit or defeat and end on expiry or target defeat.

#### Scenario: Exact target changes from one seeded outcome
- **WHEN** a qualifying Scorch application draws ATK, resistance or Break acceptance
- **THEN** it reduces the target's authored ATK by 20% of original ATK, reduces Physical, Esper and Mental resistance by 10 percentage points, or adds 10 percentage points to incoming Break efficiency respectively
- **AND** subsequent ticks do not resample the outcome

#### Scenario: Refresh and replay
- **GIVEN** an existing Temperance status and another successful Scorch application on that target
- **WHEN** the same outcome is drawn again or a different outcome is drawn
- **THEN** the same outcome refreshes without stacking, or the distinct outcomes coexist with independent deadlines respectively
- **AND** repeating the scenario with the same seed reproduces the draws and outcomes

### Requirement: Zero Resonance 6 Cosmos Cycle ATK grant

When living Zero with six selected awakening nodes pays for a qualified Cosmos Esper Cycle, living party members SHALL receive a refreshable +12% base-ATK-scaled bonus for 20 world seconds.

#### Scenario: Trigger ownership and effect lifetime
- **WHEN** Zero funds a qualified Cosmos switch, including one whose damage is attributed to the higher-CI incoming actor
- **THEN** each living party member gains that carrier once and a later qualifying activation refreshes it without stacking
- **AND** an unpaid switch, a switch paid by another character, or fewer than six selected nodes does not grant the carrier
