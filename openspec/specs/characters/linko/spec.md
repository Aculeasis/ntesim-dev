# Specification: Linko (Character 1072)

## Purpose
Black-box observable behavioral contract for Linko in `ntesim`.
Technical scaling tables, internal curves, and engine asset paths belong in [Linko Mechanics Dossier](../../../../docs/mechanics/characters/linko/action-damage.md).

All outgoing damage is Nature / Anima (`DamageElement::Nature`), scaling with live ATK.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## Requirements

### Requirement: Xiaozhen Pet Lifecycle & Energy Accrual
Linko SHALL deploy or coordinate with the pet Xiaozhen during combat.
Xiaozhen operates on an independent pulse schedule and accrues energy from Linko's Normal attacks.

#### Scenario: Pet deployment via Skill
- **GIVEN** Linko has Skill ready
- **WHEN** Skill input is committed
- **THEN** 4 bombardment rocket packets (`Skill1..4_Damage`) are emitted
- **AND** Xiaozhen throw strike (`Skill1_ThrowXiaozhen_Damage`) and blast are settled
- **AND** Xiaozhen enters active field state with autonomous pulse scheduling.

### Requirement: Burst & LTE (Link Time Event) Multi-Stage Execution
Casting Burst SHALL cost 100 energy and initiate Oversync mode.
While Oversync is active, Link Time Event (LTE) becomes available, emitting Synced Strike packets (`LTE_Damage` and `LTE_Xiaozhen_Damage`).

#### Scenario: Burst commit and LTE transition
- **GIVEN** Linko has 100 energy
- **WHEN** Burst input is committed
- **THEN** 100 energy is consumed
- **AND** Oversync aura becomes active for 15.0s
- **AND** LTE Synced Strike packets are emitted.

### Requirement: Hexed Reaction Application
Linko's Burst and special abilities SHALL apply the Hexed status.
While Hexed is active on an enemy, incoming elemental damage triggers Hexed reaction follow-up damage according to the active Hexed coefficient.

#### Scenario: Hexed reaction trigger
- **GIVEN** target enemy has Hexed status
- **WHEN** any teammate lands an elemental damage packet on the target
- **THEN** Hexed reaction bonus packet is computed and delivered.

### Requirement: Off-field pet retention

Switching Linko off field SHALL preserve pet state and timers unless the pet is explicitly consumed by Support.

#### Scenario: Ordinary field exit
- **GIVEN** Linko has an active pet and no Support consumption occurs
- **WHEN** another actor takes field control
- **THEN** the existing pet state and deadlines persist

### Requirement: Continuing owner state

Oversync state, active pet cooldowns and accumulated Xiaozhen energy SHALL persist across continuing cycles without resetting.

#### Scenario: Second cycle retains energy
- **GIVEN** a completed cycle leaves Xiaozhen energy and a pet cooldown
- **WHEN** the next cycle begins
- **THEN** the stored energy and remaining cooldown continue
