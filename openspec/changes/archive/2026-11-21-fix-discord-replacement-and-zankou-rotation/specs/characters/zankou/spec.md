# Spec Delta

## MODIFIED Requirements

### Requirement: Silent Sunset & Scorch Replacement
With a living Zankou in the party, qualified Scorch applications SHALL install Zankou's 3-stack enhanced carrier. Ally DoT deliveries SHALL add stacks and refresh duration without advancing tick phase or re-emitting Discord against active Nova. Scorch ticks SHALL NOT trigger additions.

#### Scenario: Ally DoT adds Scorch stack under Silent Sunset
- **GIVEN** target enemy has 1 stack of Zankou's enhanced Scorch
- **WHEN** an ally inflicts a DoT damage tick on the target
- **THEN** the target's Scorch stacks increase to 2
- **AND** the 15.0s duration is refreshed to 15.0s
- **AND** the next tick deadline phase is preserved.

#### Scenario: Ally DoT refresh does not re-emit Discord
- **GIVEN** a target carries active Nova and Zankou's enhanced Scorch with Discord already emitted for that pair
- **WHEN** an ally DoT tick refreshes Scorch duration or adds a stack under Silent Sunset
- **THEN** Scorch duration is refreshed and stacks are updated
- **AND** no additional Discord reaction or dependent Discord passives are triggered
