# Spec Delta

## MODIFIED Requirements

### Requirement: Discord Reaction Interaction
When an active Discord reaction triggers on a living target that is in the Broken state, Lacrimosa's Breakthrough 2 passive (`GA_lacrimosa_Passive_1`, Tomato Banquet) SHALL deliver one additional Chaos damage packet (`AnHunZhouTwo_Damage`) scaling at 400% of ATK. This interaction SHALL NOT trigger from periodic Nightmare DoT ticks solely due to target Broken state, nor when Discord reactions do not occur.

#### Scenario: Nightmare interacts with Discord
- **GIVEN** a living dummy is in the Broken state and affected by Lacrimosa's Nightmare DoT
- **WHEN** periodic Nightmare DoT ticks land on the target without a Discord reaction triggering
- **THEN** standard Nightmare DoT damage is dealt and no `AnHunZhouTwo_Damage` is scheduled
