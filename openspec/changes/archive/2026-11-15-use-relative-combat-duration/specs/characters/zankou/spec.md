# Spec Delta

## MODIFIED Requirements

### Requirement: Skill & Heartwrench Execution

Skill SHALL select ordinary or enhanced Sanguine Dash in Reality and ordinary or enhanced Soulcross in Illusion from actual owner state and prerequisites. The selected variant SHALL own its packets, Heartwrench applications, Break, costs and world-clock cooldown. Failed admission or an interrupted transition SHALL NOT grant the completed form change.

#### Scenario: Demon Strike cast and Heartwrench application
- **GIVEN** Zankou has Skill off cooldown and the prerequisite for the Heartwrench-producing enhanced Sanguine Dash
- **WHEN** Skill input is committed and reaches its form-transition event
- **THEN** the selected packets settle against eligible living targets and apply their declared Heartwrench stacks
- **AND** the sourced cooldown starts at its actual trigger rather than an unconditional twelve-second shortcut

#### Scenario: Basic prerequisite is lost
- **GIVEN** a qualifying held attack, Normal stage or counter enables enhanced Sanguine Dash
- **WHEN** an unrelated admitted input invalidates that opportunity or its world-clock deadline expires
- **THEN** the next Skill cannot use that expired enhancement
- **AND** a rejected input cannot create a new prerequisite

#### Scenario: Enhanced Soulcross is earned
- **GIVEN** Zankou remains on field in Illusion and its late enhanced-Soulcross window has opened
- **WHEN** a legal Skill completes
- **THEN** enhanced Soulcross and Oblivion settle with Follow-up semantics and unlock Bloodfeast Reverie
- **AND** Zankou returns to Reality without resetting her continuing energy or DoTs


### Requirement: Burst Execution & Vile Ash Application

Burst SHALL prioritize earned Bloodfeast Reverie over ordinary Inferno Flamenco, then offer the earned, timed, one-use free enhanced Inferno Flamenco. Each variant SHALL retain its sourced packets, costs, cooldown admission and DoT producers. Vile Ash SHALL be applied only by qualifying producers, with an independent ten-stack cap and thirty-second, one-second-period carrier.

#### Scenario: Burst cast and Vile Ash application
- **GIVEN** Zankou has earned Bloodfeast Reverie and meets its actual resource/admission requirements
- **WHEN** Burst is committed and its application event settles
- **THEN** its sourced hits and five Vile Ash stacks reach each eligible living target
- **AND** the paid cast retains its actual energy/cooldown ledger and grants the one-use enhanced Inferno Flamenco opportunity

#### Scenario: Consecutive enhanced Ultimates
- **GIVEN** Bloodfeast Reverie has completed and its free-finisher opportunity is still active
- **WHEN** the next legal Burst executes without waiting for another paid energy refill
- **THEN** it selects enhanced Inferno Flamenco and consumes that opportunity exactly once with zero Ultimate Energy cost
- **AND** a third Burst cannot reuse the opportunity

#### Scenario: Free-finisher deadline
- **GIVEN** an unused free-finisher opportunity exists
- **WHEN** its exact world-clock deadline is reached before another Burst
- **THEN** the opportunity expires and readiness no longer treats that Burst as free
- **AND** swapping does not reset or extend its deadline


### Requirement: R6 Scorch defeat enhancement

The six-node resonance SHALL grant a sixty-second enhancement after a qualifying Scorch defeat, refreshing on subsequent qualifying defeats. During that enhancement, qualifying Zankou DoT infliction SHALL set the recipient's enhanced Scorch to three stacks; unrelated direct hits SHALL NOT fabricate the trigger.

#### Scenario: Refresh the defeat enhancement
- **GIVEN** the enhancement is active and another qualifying Scorch defeat occurs
- **WHEN** the defeat callback settles
- **THEN** its lifetime refreshes to sixty seconds

#### Scenario: Enhancement trigger and expiry
- **GIVEN** the resonance enhancement is active on a living Zankou
- **WHEN** she inflicts a qualifying DoT before the world-clock expiry
- **THEN** that target's enhanced Scorch reaches its three-stack cap without recursive additions
- **AND** at the exact expiry or after source defeat, a new application cannot use the expired enhancement


### Requirement: A0 form modifier lifecycle

A0 Reality and Illusion attacks SHALL grant their declared Hunt or Delusion contribution during the action. An adjacent legal Skill SHALL carry an earned effect into its sourced eight-unpaused-world-second retained window. A0 field exit and source defeat SHALL remove the source's form modifiers.

#### Scenario: Retained overlap and exact expiry
- **GIVEN** Zankou earns Hunt through Reality attacks, enters Illusion, earns Delusion and completes Soulcross
- **WHEN** both retained deadlines are still in the future
- **THEN** Hunt grants her twenty-five-percent damage bonus and Delusion grants twenty-five-percent team DoT critical damage, each once
- **AND** each contribution ends at its own exact deadline without a resource refill
- **AND** local contact pauses alone do not extend Hunt's eight seconds after entry or Delusion's eight seconds after successful Soulcross

#### Scenario: Field exit clears A0 form effects
- **GIVEN** A0 Hunt or Delusion is still active
- **WHEN** Zankou leaves the field or is defeated
- **THEN** that source's form modifier contributions end
- **AND** returning to the field does not restore the discarded effects or re-grant entry Cycle


### Requirement: Combat-time action clock

Action progression, contact pauses, queued follow-ups, cancellation and switch gates SHALL follow Zankou's sourced owner clock and shared global-pause policy. Cooldowns, form lifetimes, modifier expiry and free-cast windows SHALL use unpaused world time. Local contact pauses alone SHALL NOT freeze those world timers. A failed input SHALL preserve preceding legal state and identify its occurrence.

#### Scenario: Contact does not freeze cooldowns
- **GIVEN** a delivered packet causes an owner montage pause during Illusion without a global timer pause
- **WHEN** absolute time advances through that local contact pause
- **THEN** action progression follows the contact pause while form and cooldown deadlines continue on unpaused world time
- **AND** additional AoE recipients cannot duplicate one causal packet's pause or resource grant

#### Scenario: Global pause retains owner world deadlines
- **GIVEN** an Illusion lifetime, retained modifier, cooldown or free-finisher opportunity has two world seconds remaining
- **WHEN** an Ultimate cinematic or Hotori active Time Stop advances ten absolute seconds without a source-removal event
- **THEN** each ordinary owner deadline retains its two seconds and observations agree with later admission and expiry
- **AND** global pause does not duplicate contact pauses, stacks, resources or completed form transitions


### Requirement: Live form and readiness observations

Source queries and reports SHALL expose actual form, form time, relevant enhancement permissions and selected Skill/Burst readiness using owner admission state. Queries SHALL NOT create permissions, resource grants or future knowledge. Native replay and WASM/Worker execution SHALL preserve the same state and source identity.

#### Scenario: Ready means the currently selected variant
- **GIVEN** enhanced Soulcross or a free Inferno Flamenco has become available
- **WHEN** the source reads readiness and then submits the corresponding ordinary input
- **THEN** query and admission select the same variant and world-clock deadline
- **AND** replay reconstructs the same cost, packets and form transition
