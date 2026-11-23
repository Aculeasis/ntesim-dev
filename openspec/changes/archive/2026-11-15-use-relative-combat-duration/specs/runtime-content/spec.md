# Spec Delta

## MODIFIED Requirements

### Requirement: Sakiri A4 source-based ATK grant

Selected Sakiri A4 SHALL grant eligible living teammates, excluding Sakiri, flat ATK equal to thirty percent of her source base ATK for twenty unpaused world seconds after the sourced Burst activation event. Source base ATK SHALL include equipped weapon base ATK and exclude console flat ATK, ATK-percent buffs and recipient stats. Only actual node four SHALL enable this contribution.

#### Scenario: Independent flat-ATK arithmetic
- **GIVEN** Sakiri has source base ATK 1000 and exactly node four selected
- **WHEN** an admitted Burst reaches the buff activation event
- **THEN** each eligible teammate receives exactly 300 additional ATK from A4 and Sakiri receives none
- **AND** changing Sakiri's console ATK or a recipient's base ATK does not change that A4 amount
- **AND** a larger equipped weapon base ATK changes the grant by exactly thirty percent of that base difference

#### Scenario: Refresh and actual expiry
- **GIVEN** a teammate carries Sakiri's active A4 grant
- **WHEN** another legally funded Burst refreshes it or its exact deadline is reached
- **THEN** the same source refreshes its twenty-second lifetime without stacking duplicate grants, or removes the expired contribution respectively
- **AND** field exit does not invent a second application or extend the timer
- **AND** source/recipient defeat follows the declared living-carrier lifetime

#### Scenario: Node identity and rejected casts
- **WHEN** Burst admission fails or Sakiri has only unrelated awakening nodes selected
- **THEN** no A4 grant is created or refreshed
- **AND** selecting four nodes without node four cannot substitute for that node

### Requirement: Sakiri ordinary and awakened Burst contributions

Sakiri's ordinary Burst SHALL grant other eligible living teammates thirty percent of her source base ATK for twenty unpaused world seconds. Selected A4 SHALL add a separate thirty-percent contribution with the same duration and source basis. Each carrier SHALL refresh without stacking itself. Damage and live buff observations SHALL use the actual effective grant, recipients and actual world-clock lifetime rather than a synthetic expected team-DPS percentage.

#### Scenario: Compare an identical Burst with and without A4
- **GIVEN** otherwise identical builds, resources, recipients and legal actions with A4 absent or selected
- **WHEN** Burst and ally damage settle inside and outside the buff window
- **THEN** the ordinary Burst contribution remains correct in both cases and the isolated A4 difference equals its sourced flat-ATK contribution
- **AND** non-ATK-scaling damage does not gain a fictional thirty-percent multiplier
- **AND** an observed small total-DPS difference is explained using actual buff uptime and affected damage rather than relabeled as a failed buff

#### Scenario: Sakiri grant survives stopped world time
- **GIVEN** the ordinary and selected A4 grants each have two world seconds remaining
- **WHEN** an Ultimate cinematic or Hotori Time Stop lasts ten absolute seconds without source removal
- **THEN** both grants retain exactly two seconds and their separate source-based ATK contributions throughout the pause
- **AND** neither grant stacks, refreshes or expires solely from that absolute elapsed time
