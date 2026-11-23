# Specification: Lacrimosa (Character 1004)

## Purpose
Black-box observable behavioral contract for Lacrimosa in `ntesim`.
Technical scaling tables, internal curves, and engine asset paths belong in [Lacrimosa Mechanics Dossier](../../../../docs/mechanics/characters/lacrimosa/action-damage.md).

All outgoing damage is Chaos (`DamageElement::Chaos`), scaling with live ATK.

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## Requirements

### Requirement: Stance Switching & Attack Chains
Lacrimosa SHALL operate in two distinct combat stances: Mode A (Primary / Parasol) and Mode B (Dual-Blade).
Switching stances SHALL emit a Stance Switch damage packet (`SwitchModB_Damage`) and reconfigure the normal attack chain.

#### Scenario: Stance toggle
- **GIVEN** Lacrimosa is in Mode A
- **WHEN** Stance switch is executed
- **THEN** Mode B becomes active
- **AND** a `SwitchModB_Damage` packet is emitted
- **AND** subsequent normal attack inputs execute Mode B attack stages.

### Requirement: Skill "Devilish Gift" ("Sleep In") Charges & Cooldown
Skill SHALL possess 2 base charges with a 12.0s recharge timer per charge.
Casting Skill SHALL consume 1 charge, deal 1 direct explosion packet (`Skill1_Damage`), and apply 5 stacks of Nightmare DoT to hit targets.

#### Scenario: Skill cast with available charges
- **GIVEN** Lacrimosa has 2 Skill charges and 0 active recharge timers
- **WHEN** Skill input is committed
- **THEN** charges decrease to 1
- **AND** `Skill1_Damage` packet is scheduled
- **AND** 5 stacks of Nightmare DoT are applied to target
- **AND** a 12.0s cooldown timer starts for charge 1.

#### Scenario: Skill cast with 0 charges
- **GIVEN** Lacrimosa has 0 Skill charges
- **WHEN** Skill input is attempted
- **THEN** action admission fails with an `UnmetCooldown` diagnostic
- **AND** no charges or cooldowns are mutated.

### Requirement: Nightmare DoT Application & Stacking
Nightmare DoT SHALL deal periodic Chaos damage ticks (`Blood_Damage` or `Blood_Damage_LV6`) every 1.0s up to a maximum duration of 10.0s.
Applying new stacks SHALL refresh the duration to 10.0s and increase stack count up to the 10-stack cap.

#### Scenario: Nightmare DoT stack refresh
- **GIVEN** target enemy has 3 stacks of Nightmare DoT with 3.0s duration remaining
- **WHEN** Skill applies 5 additional stacks
- **THEN** target has 8 stacks of Nightmare DoT
- **AND** duration is refreshed to 10.0s.

### Requirement: Burst Execution by Stance
Casting Burst in Mode A SHALL consume 120 energy and deliver 4 direct flurries followed by a slam (`UltraSkill2_Damage`) applying 5 Nightmare stacks.
Casting Burst in Mode B SHALL deliver a flurry and finisher with independent coefficients.

#### Scenario: Mode A Burst commit
- **GIVEN** Lacrimosa is active in Mode A with 120 energy
- **WHEN** Burst input is committed
- **THEN** 120 energy is consumed
- **AND** 4x `UltraSkill1_Damage` followed by 1x `UltraSkill2_Damage` packets are settled
- **AND** 5 stacks of Nightmare DoT are applied to target.

### Requirement: Discord Reaction Interaction
When an active Discord reaction triggers on a living target that is in the Broken state, Lacrimosa's Breakthrough 2 passive (`GA_lacrimosa_Passive_1`, Tomato Banquet) SHALL deliver one additional Chaos damage packet (`AnHunZhouTwo_Damage`) scaling at 400% of ATK. This interaction SHALL NOT trigger from periodic Nightmare DoT ticks solely due to target Broken state, nor when Discord reactions do not occur.

#### Scenario: Nightmare interacts with Discord
- **GIVEN** a living dummy is in the Broken state and affected by Lacrimosa's Nightmare DoT
- **WHEN** periodic Nightmare DoT ticks land on the target without a Discord reaction triggering
- **THEN** standard Nightmare DoT damage is dealt and no `AnHunZhouTwo_Damage` is scheduled

### Requirement: Continuing charges and effects

Skill charges, partial cooldowns and target DoT timers SHALL persist across continuing cycles without resetting.

#### Scenario: Second cycle starts
- **GIVEN** a rotation finishes one cycle with a partial Skill recharge and an active DoT
- **WHEN** the next cycle begins
- **THEN** the existing charge count, recharge progress and DoT deadline continue

### Requirement: DEF ignore cap

Combined DEF-ignore bonuses SHALL NOT exceed the system cap of 100% ignored defense.

#### Scenario: Combined bonuses exceed the cap
- **GIVEN** selected bonuses would sum to more than 100% DEF ignore
- **WHEN** defense is evaluated for a hit
- **THEN** effective ignored defense is capped at 100%

### Requirement: DoT defeat callbacks

A target defeated by a DoT tick SHALL trigger defeat callbacks and clear its active debuffs.

#### Scenario: Nightmare defeats a dummy
- **GIVEN** a living dummy has Nightmare and insufficient HP to survive its next tick
- **WHEN** that tick settles
- **THEN** the target defeat is observed and active debuffs are cleared
