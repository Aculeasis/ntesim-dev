# Spec Delta: Shinku (Character 1076)

## ADDED Requirements

### Requirement: Defiant Spirit & Surging Crimson Stance Entry
Shinku SHALL maintain a Defiant Spirit gauge with a maximum capacity of 60. Normal attacks and skills in ordinary stance generate Defiant Spirit on hit. Committing the Ultimate ability (Crimson Fury) consumes 60 Defiant Spirit, deals 9 initial burst hits (`UltraSkill1` through `UltraSkill5`), and activates the 13-second Surging Crimson stance.

#### Scenario: Ultimate cast enters Surging Crimson
- **GIVEN** Shinku is in ordinary stance with 60 Defiant Spirit
- **WHEN** Ultimate (Crimson Fury) is committed
- **THEN** 60 Defiant Spirit is consumed
- **AND** 9 initial burst packets are scheduled
- **AND** Surging Crimson stance is activated for 13 seconds
- **AND** live damage bonus is increased by +30% (+60% with Awakening 3 Resonance).

### Requirement: Tidal Assault & Fading Reason Generation
While active in Surging Crimson stance, Shinku's normal attacks SHALL execute Tidal Assault (`Melee1_Rage` through `Melee5_Rage`), generating Fading Reason up to a maximum capacity of 12.

#### Scenario: Tidal Assault generates Fading Reason
- **GIVEN** Shinku is active in Surging Crimson stance with 0 Fading Reason
- **WHEN** Tidal Assault attack lands on target
- **THEN** Fading Reason increases toward the cap of 12.

### Requirement: Crimson Judgment Dashes
During Surging Crimson, each cast of Scarlet Descent SHALL grant 1 stack of Crimson Judgment up to a maximum of 3 stacks. Starting at 9.0 seconds into Surging Crimson, the Ultimate input SHALL trigger Crimson Judgment dashes (`UltraSkillPre_Rage_Damage`, 4 hits per dash), consuming 1 stack per dash.

#### Scenario: Crimson Judgment dash execution
- **GIVEN** Shinku is at 9.5s of Surging Crimson with 3 Crimson Judgment stacks
- **WHEN** Ultimate input is committed
- **THEN** Crimson Judgment dash executes dealing 4 hits
- **AND** 1 Crimson Judgment stack is consumed.

### Requirement: Dragonflame Verdict Finisher
Once all Crimson Judgment stacks are consumed or upon Surging Crimson expiration, a subsequent Ultimate input SHALL execute Dragonflame Verdict (`UltraSkill_Rage_Damage` 10 hits followed by `UltraSkill2_Rage_Damage` finisher).

#### Scenario: Dragonflame Verdict execution
- **GIVEN** Shinku is in Surging Crimson with 0 Crimson Judgment stacks
- **WHEN** Ultimate input is committed
- **THEN** Dragonflame Verdict executes dealing 10 rapid strikes and the 400% ATK finisher nuke
- **AND** Surging Crimson stance concludes.

## MODIFIED Requirements

### Requirement: Skill & Judgment Execution
In ordinary stance, casting Skill SHALL deliver 3 progressive slashes (`Skill1_1` through `Skill1_3_Damage`) with a 12-second nominal cooldown. In Surging Crimson stance, the skill is replaced by Scarlet Descent (`Skill1_Rage_Damage` + `Skill2_Rage_Damage`), which has no standard cooldown, consumes 12 Fading Reason, and can be cast up to 5 times per Surging Crimson window.

#### Scenario: Rage Skill cast
- **GIVEN** Shinku is in Surging Crimson stance with 12 Fading Reason and fewer than 5 Scarlet Descent casts
- **WHEN** Skill input is committed
- **THEN** 12 Fading Reason is consumed
- **AND** `Skill1_Rage_Damage` and `Skill2_Rage_Damage` are settled
- **AND** 1 Crimson Judgment stack is granted (capped at 3)
- **AND** the Scarlet Descent cast count increments.

### Requirement: Continuing Rage and aura
Surging Crimson remaining duration, Fading Reason, Crimson Judgment stacks, Menacing Gaze stacks, and stance timers SHALL persist across continuing cycles without resetting.

#### Scenario: Continue with active stance
- **GIVEN** a cycle ends with an active stance, Fading Reason, and aura stacks
- **WHEN** the next cycle begins
- **THEN** the same state and deadlines continue.

## REMOVED Requirements

### Requirement: Rage Meter & Threshold Transformation
**Reason**: Replaced by authentic Defiant Spirit resource and Crimson Fury Ultimate entry into Surging Crimson stance.
**Migration**: Use Defiant Spirit and Ultimate activation to enter Surging Crimson.

### Requirement: Rage decay
**Reason**: Obsolete placeholder mechanism; Surging Crimson operates on a fixed 13-second duration rather than continuous rage decay.
**Migration**: Model Surging Crimson with a 13-second active duration deadline.
