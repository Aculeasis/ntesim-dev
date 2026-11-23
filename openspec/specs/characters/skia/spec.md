# Specification: Skia (Character 1008)

## Purpose
Black-box observable behavioral contract for Skia in `ntesim`.
Technical scaling tables, internal curves, and engine asset paths belong in [Skia Mechanics Dossier](../../../../docs/mechanics/characters/skia/action-damage.md).

All outgoing damage is Lakshana (`DamageElement::Lakshana`), scaling with live ATK.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## Requirements

### Requirement: Shadow Hound & State 38 Entry Window
Casting Skill SHALL execute the 6-contact Shadow Hound Chase combo, apply ShadowControl, and grant the 5.0s State 38 entry window.
While State 38 is active, switching to a teammate grants an entry damage bonus to the incoming ally.

#### Scenario: Skill cast and State 38 entry window
- **GIVEN** Skia has Skill off cooldown
- **WHEN** Skill input is committed
- **THEN** 6 Shadow Hound Chase packets (`Skill1..3_Damage`) are emitted
- **AND** a 12.0s Skill cooldown starts
- **AND** State 38 buff window is granted for 5.0s.

### Requirement: Fang Thrust Mark & Consumption
Fang Thrust hits SHALL mark target enemies.
Subsequent attacks by Skia or the Shadow Hound SHALL consume the mark, dealing secondary burst damage (`SkillShadowAtk_Damage`).

#### Scenario: Fang Thrust mark consumption
- **GIVEN** target enemy is afflicted with Fang Thrust mark
- **WHEN** Shadow Hound lands an attack packet
- **THEN** the mark is consumed
- **AND** `SkillShadowAtk_Damage` packet is settled against the target.

### Requirement: Burst "The Pack" Execution
Casting Burst SHALL consume 100 energy and deliver the initial spread strike (`UltraSkill1_Damage`) followed by 3 converge bites (`UltraSkill2_Damage`).

#### Scenario: Burst commit
- **GIVEN** Skia has 100 energy
- **WHEN** Burst input is committed
- **THEN** 100 energy is consumed
- **AND** `UltraSkill1_Damage` and 3x `UltraSkill2_Damage` packets are settled.

### Requirement: Unclaimed outro expiry

State 38 SHALL expire after five seconds without a teammate switching in, leaving no penalty or residual buffs.

#### Scenario: No teammate enters
- **GIVEN** State 38 has started
- **WHEN** five seconds elapse without a qualifying teammate switch
- **THEN** State 38 expires without a remaining buff

### Requirement: Continuing Hound state

The State 38 timer, active Hound generation and Skill cooldown SHALL carry across continuing cycles without resetting.

#### Scenario: Cycle boundary with active Hound
- **GIVEN** a cycle ends with an active Hound and Skill cooldown
- **WHEN** the next cycle begins
- **THEN** Hound generation and cooldown continue from existing state
