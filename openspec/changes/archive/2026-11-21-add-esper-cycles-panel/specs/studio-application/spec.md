# Spec Delta

## ADDED Requirements

### Requirement: Esper Cycles reaction statistics panel
Studio SHALL present an "Esper Cycles" statistics panel displaying all Esper Cycle reactions available to the deployed team composition and their trigger distribution across completed simulation trials. The panel SHALL adaptively display when at least one reaction is available and hide completely when no reactions are available.

#### Scenario: Team with available reactions renders 3-column overview
- **WHEN** a simulation run completes for a party that contains at least one available Esper Cycle reaction
- **THEN** the report overview row displays a 3-column layout containing "Actions", "Field Time Distribution", and "Esper Cycles" in that order
- **AND** the "Esper Cycles" panel renders a horizontal bar chart displaying every available reaction in the team
- **AND** reactions are sorted in descending order of mean trigger count
- **AND** each reaction displays an error bar (whiskers) representing its trigger distribution (min, max, std) across trials
- **AND** hovering or focusing a reaction bar exposes its mean, min, max, and standard deviation in a tooltip

#### Scenario: Team without available reactions hides Esper Cycles panel
- **WHEN** a simulation run completes for a party with zero available reactions (such as a mono-element team or non-adjacent elements)
- **THEN** the "Esper Cycles" panel is completely hidden from the report
- **AND** the report overview row displays the 2-column layout containing only "Actions" and "Field Time Distribution"

#### Scenario: Available reactions with zero triggers display zero
- **WHEN** an available reaction in the party triggers zero times throughout all simulation trials
- **THEN** the reaction is displayed in the "Esper Cycles" panel with a trigger value of 0 and an empty error bar
- **AND** the tooltip identifies the mean, min, max, and std as 0

#### Scenario: Reaction triggers from character passives are counted
- **WHEN** character skills or passives reapply or refresh an Esper Cycle reaction (such as Baicang B2 Scorch refresh or Blackbird A6 Nova reapplication)
- **THEN** those events increment the total trigger count for the respective reaction alongside qualified switch admissions

#### Scenario: Ternary reaction triggers
- **WHEN** Blossom pistils hit targets affected by active Remora (triggering Charge) or Scorch and Nova overlap on the same living target (triggering Discord)
- **THEN** each occurrence increments the trigger count for Charge and Discord respectively
