# Spec Delta

## Purpose

Observable behavioral contract for Blackbird (`character:1042`) in `ntesim`, governing her transformation Burst, zero-cost finisher, ordinary Skill bird marks, and continuing energy state.

## ADDED Requirements

### Requirement: Transformation Burst Activation
Activating `GA_BlackBird_UltraSkill` SHALL require 100 Ultimate Energy, start a 20.0s World cooldown timer, grant a team-wide base ATK bonus of 20% for 15.0s, and activate Blackbird's transformation form for 21.5s on-field World lifetime.

#### Scenario: Successful transformation cast
- **GIVEN** Blackbird is active on field with at least 100 Ultimate Energy and no active Burst cooldown
- **WHEN** `GA_BlackBird_UltraSkill` is invoked
- **THEN** 100 Ultimate Energy is spent
- **AND** a 20.0s cooldown starts for `GA_BlackBird_UltraSkill`
- **AND** transformation form is activated
- **AND** team-wide base ATK bonus is applied.

### Requirement: Zero-Cost Finishing Burst
While in transformation form, invoking Blackbird's Burst input SHALL cast `GA_BlackBird_Evolve_UltraSkill` (Finisher). The finisher SHALL require zero Ultimate Energy, have no separate cooldown, deliver 7 Psyche damage packets, and terminate the transformation form.

#### Scenario: Successful finisher execution
- **GIVEN** Blackbird is in active transformation form
- **WHEN** `GA_BlackBird_Evolve_UltraSkill` is invoked
- **THEN** zero Ultimate Energy is spent
- **AND** 7 damage packets are delivered
- **AND** the transformation form ends immediately.

### Requirement: Energy Accumulation During Transformation Form
Blackbird SHALL continue to accumulate Ultimate Energy from connected hits and off-field team energy generation while in transformation form.

#### Scenario: Off-field energy generation while transformed
- **GIVEN** Blackbird is in transformation form paused off-field
- **WHEN** active teammates deliver combat damage
- **THEN** Blackbird receives elemental and team energy grants according to her Charge Efficiency up to the 100 energy cap.

### Requirement: Ordinary Skill Execution and Coordinated Birds
While not in transformation form, `GA_BlackBird_Skill` SHALL have a 16.0s cooldown, fill the Cycle gauge to 100 on activation, deliver 4 Psyche damage hits, and arm the marked-target bird controller with up to 5 coordinated attacks triggered by active ally hits over 15.0s.

#### Scenario: Ordinary Skill cast
- **GIVEN** Blackbird is active and not transformed with Skill off cooldown
- **WHEN** `GA_BlackBird_Skill` is committed
- **THEN** Cycle gauge fills to 100
- **AND** target dummy receives 4 Skill hits and a 15.0s mark for coordinated bird attacks.
