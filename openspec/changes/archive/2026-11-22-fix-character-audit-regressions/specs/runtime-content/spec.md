## ADDED Requirements

### Requirement: Public character effect projection

Authored scenarios SHALL select supported owner effects with their canonical model provenance and native progression gates. Entry effects SHALL retain actual outgoing state, switch eligibility and qualified Support rules.

#### Scenario: Owner passives without an authored owner cast
- **GIVEN** a public scenario includes Adler with Righteous Heart unlocked or Aurelia with Harmonics unlocked
- **WHEN** Adler's starting DEF is evaluated or an eligible Nova expires with Aurelia in the party
- **THEN** the corresponding owner passive is available without a manual model declaration or prior owner cast

#### Scenario: Lacrimosa Evade into Baicang entry
- **GIVEN** Lacrimosa is in ordinary Evade recovery and the incoming Baicang qualifies for his ETD-4 entry effect
- **WHEN** `dash[window=swap]` yields at the existing Evade exit gate and the script admits the switch to Baicang before Evade recovery ends
- **THEN** the qualified entry uses the existing Evade exit and JumpZ models with their ordinary costs and deadlines

### Requirement: Character effect activation and thresholds

Reached character effects SHALL use their own cast markers, progression gates and exact target predicates across supported damage and action paths.

#### Scenario: Ultimate and Skill effect producers
- **WHEN** an authored Adler Burst reaches its second volley marker, Aurelia casts Burst, or Blackbird casts the evolved Finisher
- **THEN** Adler receives each selected Burst resource callback once at the marker
- **AND** Aurelia's Burst leaves Cadenza unchanged and Blackbird's Finisher leaves the transformation Burst's team ATK grant unchanged

#### Scenario: Target damage and resistance conditions
- **WHEN** Chaos hits a target marked by Warrant at Breakthrough 4, Jiuyuan binds a target at Resonance 6, or Mint with node 6 hits a target at exactly 40 percent HP
- **THEN** Chaos receives the 30 percent Warrant damage bonus, Pact reduces the bound target's Anima resistance by 10 percentage points, and Mint receives her low-HP damage bonus respectively
- **AND** selecting Jiuyuan's node 6 alone does not grant the resonance reduction
