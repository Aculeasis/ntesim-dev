# Specification: Shinku (Character 1076)

## Purpose
Black-box observable behavioral contract for Shinku in `ntesim`.
Technical scaling tables, internal curves, and engine asset paths belong in [Shinku Mechanics Dossier](../../../../docs/mechanics/characters/shinku/action-damage.md).

All outgoing damage is Cosmos (`DamageElement::Cosmos`), scaling with live ATK.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## Requirements

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
Once all Crimson Judgment stacks are consumed or upon Surging Crimson expiration, a subsequent Ultimate input SHALL execute Dragonflame Verdict (`UltraSkill_Rage_Damage` 10 hits followed by `UltraSkill2_Rage_Damage` finisher). Ultimate ability cooldown commences at initial Crimson Fury commitment (20.0s duration); Dragonflame Verdict execution SHALL preserve the remaining commitment cooldown rather than imposing an additional subsequent full cooldown penalty.

#### Scenario: Dragonflame Verdict execution
- **GIVEN** Shinku is in Surging Crimson with 0 Crimson Judgment stacks
- **WHEN** Ultimate input is committed
- **THEN** Dragonflame Verdict executes dealing 10 rapid strikes and the 400% ATK finisher nuke
- **AND** Surging Crimson stance concludes
- **AND** Shinku's Ultimate ability cooldown retains the remaining duration from the initial Crimson Fury commitment.

### Requirement: Skill & Judgment Execution
In ordinary stance, casting Skill SHALL deliver 3 progressive slashes (`Skill1_1` through `Skill1_3_Damage`) with a 12-second nominal cooldown. In Surging Crimson stance, the skill is replaced by Scarlet Descent (`Skill1_Rage_Damage` + `Skill2_Rage_Damage`), which has no standard cooldown, consumes 12 Fading Reason, and can be cast up to 5 times per Surging Crimson window.

#### Scenario: Rage Skill cast
- **GIVEN** Shinku is in Surging Crimson stance with 12 Fading Reason and fewer than 5 Scarlet Descent casts
- **WHEN** Skill input is committed
- **THEN** 12 Fading Reason is consumed
- **AND** `Skill1_Rage_Damage` and `Skill2_Rage_Damage` are settled
- **AND** 1 Crimson Judgment stack is granted (capped at 3)
- **AND** the Scarlet Descent cast count increments.

### Requirement: Menacing Gaze Aura & ATK Stacking
Shinku SHALL project a Menacing Gaze aura in active combat, pulsing every 2.0s to apply pressure and stack self ATK bonus up to the maximum stack cap.

#### Scenario: Menacing Gaze pulse
- **GIVEN** Shinku is active in combat with 2 stacks of Menacing Gaze
- **WHEN** 2.0s pulse timer expires
- **THEN** Menacing Gaze stack count increases to 3
- **AND** live ATK bonus is recalculated accordingly.

### Requirement: Continuing Rage and aura
Surging Crimson remaining duration, Fading Reason, Crimson Judgment stacks, Menacing Gaze stacks, and stance timers SHALL persist across continuing cycles without resetting.

#### Scenario: Continue with active stance
- **GIVEN** a cycle ends with an active stance, Fading Reason, and aura stacks
- **WHEN** the next cycle begins
- **THEN** the same state and deadlines continue.
