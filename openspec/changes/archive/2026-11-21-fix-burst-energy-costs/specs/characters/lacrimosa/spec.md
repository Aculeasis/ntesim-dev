# Spec Delta

## MODIFIED Requirements

### Requirement: Burst Execution by Stance
Casting Burst in Mode A SHALL consume 120 energy and deliver 4 direct flurries followed by a slam (`UltraSkill2_Damage`) applying 5 Nightmare stacks.
Casting Burst in Mode B SHALL deliver a flurry and finisher with independent coefficients.

#### Scenario: Mode A Burst commit
- **GIVEN** Lacrimosa is active in Mode A with 120 energy
- **WHEN** Burst input is committed
- **THEN** 120 energy is consumed
- **AND** 4x `UltraSkill1_Damage` followed by 1x `UltraSkill2_Damage` packets are settled
- **AND** 5 stacks of Nightmare DoT are applied to target.
