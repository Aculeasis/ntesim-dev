# Zankou combat delta

## MODIFIED Requirements

### Requirement: Crimson Reverie Entry & Aura

At combat entry, living Zankou SHALL have her Cycle gauge set to 100 exactly once. Crimson Reverie SHALL add 100 Cycle Intensity to Zankou herself while she is a team member, including off field. It SHALL NOT grant Cycle Intensity to other actors or refill Ultimate Energy. Field returns SHALL preserve continuing Cycle rather than repeating initialization.

#### Scenario: Combat entry Cycle initialization
- **GIVEN** a scenario commences with living Zankou in the party
- **WHEN** combat starts at 0.0 seconds
- **THEN** her Cycle gauge is set to 100 and her Cycle Intensity gains exactly 100 points
- **AND** other actors' Cycle Intensity and every actor's Ultimate Energy remain unchanged

#### Scenario: Off-field source and repeated returns
- **GIVEN** Zankou has spent her entry Cycle and left the field
- **WHEN** she returns after one or more switches or thirty real seconds
- **THEN** her current Cycle balance is preserved without another grant
- **AND** her own 100-point CI contribution persists without becoming a team aura or stacking another copy

### Requirement: Skill & Heartwrench Execution

Skill SHALL select ordinary or enhanced Sanguine Dash in Reality and ordinary or enhanced Soulcross in Illusion from actual owner state and prerequisites. The selected variant SHALL own its packets, Heartwrench applications, Break, costs and real cooldown. Failed admission or an interrupted transition SHALL NOT grant the completed form change.

#### Scenario: Demon Strike cast and Heartwrench application
- **GIVEN** Zankou has Skill off cooldown and the prerequisite for the Heartwrench-producing enhanced Sanguine Dash
- **WHEN** Skill input is committed and reaches its form-transition event
- **THEN** the selected packets settle against eligible living targets and apply their declared Heartwrench stacks
- **AND** the sourced cooldown starts at its actual trigger rather than an unconditional twelve-second shortcut

#### Scenario: Basic prerequisite is lost
- **GIVEN** a qualifying held attack, Normal stage or counter enables enhanced Sanguine Dash
- **WHEN** an unrelated admitted input invalidates that opportunity or its real deadline expires
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
- **WHEN** its exact real-time deadline is reached before another Burst
- **THEN** the opportunity expires and readiness no longer treats that Burst as free
- **AND** swapping does not reset or extend its deadline

### Requirement: A1 damage bonuses

Selected A1 SHALL keep Hunt and Delusion active while Zankou lives, including off field and across form changes, with the forty-percent Hunt damage bonus and fifty-percent team DoT critical-damage bonus. Its additional DoT-spreading producers SHALL require the actual selected node and eligible delivered attack.

#### Scenario: A1 is selected
- **GIVEN** Zankou has A1 selected
- **WHEN** her selected passive bonuses are applied
- **THEN** her damage bonus and the team DoT critical-damage bonus are each applied once
- **AND** form changes and field exit do not remove them while she remains alive

#### Scenario: Exact node and source lifetime
- **GIVEN** A1 is absent or Zankou is defeated
- **WHEN** an otherwise eligible A1-only spreading attack or permanent modifier would be evaluated
- **THEN** the absent or expired A1 effect contributes nothing
- **AND** selecting unrelated nodes cannot stand in for A1

### Requirement: A2 Kindling

With A2 selected, qualifying Scorch or Hexed Cycle participation SHALL grant Kindling up to one stored stack. The next qualifying Inferno Flamenco SHALL receive the additional 150% final damage and consume the earned enhancement under its actual lifecycle. Selected A4 SHALL share that enhancement with the associated Bloodfeast Reverie without granting another stored stack.

#### Scenario: Consume Kindling
- **GIVEN** Zankou holds a Kindling stack from qualifying Cycle participation
- **WHEN** her next qualifying Burst consumes the enhancement
- **THEN** that Burst receives the declared final multiplier
- **AND** later unqualified Bursts cannot reuse it

#### Scenario: Cap and A4 sharing
- **GIVEN** one stored Kindling and repeated qualifying Cycle participation
- **WHEN** Bloodfeast Reverie and Inferno Flamenco execute with A2 and A4 selected
- **THEN** the stored count never exceeds one and both eligible casts receive the sourced shared enhancement
- **AND** without A4, Bloodfeast Reverie cannot inherit Inferno Flamenco's Kindling bonus

### Requirement: A6 DoT scaling

Selected A6 SHALL increase only Oblivion's final hit by twelve percent per qualifying DoT stack on that hit's target, capped at 240% for twenty stacks. Other attacks SHALL NOT inherit this bonus.

#### Scenario: A6 reaches its cap
- **GIVEN** a target has twenty qualifying DoT stacks
- **WHEN** Oblivion's final hit is evaluated with A6 selected
- **THEN** the stack-dependent bonus is +240% and further stacks cannot increase it

#### Scenario: Multiple targets retain independent counts
- **GIVEN** two eligible targets carry different DoT stack totals
- **WHEN** the same Oblivion final hit reaches both
- **THEN** each target receives its own capped bonus
- **AND** preceding Oblivion packets and ordinary attacks receive no A6 bonus

### Requirement: R6 Scorch defeat enhancement

The six-node resonance SHALL grant a sixty-second enhancement after a qualifying Scorch defeat, refreshing on subsequent qualifying defeats. During that enhancement, qualifying Zankou DoT infliction SHALL set the recipient's enhanced Scorch to three stacks; unrelated direct hits SHALL NOT fabricate the trigger.

#### Scenario: Refresh the defeat enhancement
- **GIVEN** the enhancement is active and another qualifying Scorch defeat occurs
- **WHEN** the defeat callback settles
- **THEN** its lifetime refreshes to sixty seconds

#### Scenario: Enhancement trigger and expiry
- **GIVEN** the resonance enhancement is active on a living Zankou
- **WHEN** she inflicts a qualifying DoT before the real expiry
- **THEN** that target's enhanced Scorch reaches its three-stack cap without recursive additions
- **AND** at the exact expiry or after source defeat, a new application cannot use the expired enhancement

## ADDED Requirements

### Requirement: A0 form modifier lifecycle

A0 Reality and Illusion attacks SHALL grant their declared Hunt or Delusion contribution during the action. An adjacent legal Skill SHALL carry an earned effect into its sourced eight-real-second retained window. A0 field exit and source defeat SHALL remove the source's form modifiers.

#### Scenario: Retained overlap and exact expiry
- **GIVEN** Zankou earns Hunt through Reality attacks, enters Illusion, earns Delusion and completes Soulcross
- **WHEN** both retained deadlines are still in the future
- **THEN** Hunt grants her twenty-five-percent damage bonus and Delusion grants twenty-five-percent team DoT critical damage, each once
- **AND** each contribution ends at its own exact deadline without a resource refill
- **AND** action pauses do not extend Hunt's eight seconds after entry or Delusion's eight seconds after successful Soulcross

#### Scenario: Field exit clears A0 form effects
- **GIVEN** A0 Hunt or Delusion is still active
- **WHEN** Zankou leaves the field or is defeated
- **THEN** that source's form modifier contributions end
- **AND** returning to the field does not restore the discarded effects or re-grant entry Cycle

### Requirement: Reality and Illusion lifecycle

Zankou SHALL start in Reality. Completed Sanguine Dash SHALL enter timed Illusion. Completed Soulcross, Bloodfeast Reverie, Illusion expiry or field exit SHALL end Illusion. A later return SHALL NOT restore discarded form time. Already delivered persistent effects SHALL retain their own lifetimes.

#### Scenario: Early swap loses Illusion
- **GIVEN** Zankou is in Illusion and has not completed Soulcross
- **WHEN** she leaves the field and later returns
- **THEN** she returns in Reality without the discarded Illusion or its enhanced-Soulcross permission
- **AND** existing Heartwrench and Vile Ash continue according to their independent source lifetimes

#### Scenario: Interrupted transformation
- **GIVEN** a form-changing Skill has been admitted but its completion event has not settled
- **WHEN** a supported interruption cancels that event
- **THEN** no completed transition or late-form reward is granted
- **AND** committed costs, reached cooldown triggers and already launched damage follow the cancellation contract

### Requirement: Form-specific combat inputs

Ordinary player inputs SHALL select Zankou's five-stage Reality Normal chain, four-stage Illusion chain, form-specific held attacks, sourced Support, plunge and earned counter. Illusion Normal, held attacks, Soulcross and Bloodfeast Reverie SHALL carry their actual Follow-up semantics. A caller SHALL NOT select a damage packet to bypass owner permissions.

#### Scenario: Held entry and both chains
- **WHEN** legal Normal and held inputs execute across a completed Reality-to-Illusion transition
- **THEN** the delivered chains, Heartwrench applications and enhanced-entry permissions match the current form
- **AND** each input uses its native talent category and continuing resource channels

#### Scenario: Counter and plunge require their inputs
- **WHEN** the source requests a counter or airborne attack without its earned prerequisite
- **THEN** it cannot receive counter damage or plunge enhancement by choosing a packet or waiting
- **AND** legally earned counterparts execute through the same source/native admission boundary

### Requirement: Combat-time action clock

Action progression, contact pauses, queued follow-ups, cancellation and switch gates SHALL follow Zankou's sourced owner clock. Cooldowns, form lifetimes, modifier expiry and free-cast windows SHALL progress on real combat time. A failed input SHALL preserve the preceding legal state and identify the actual rejected occurrence.

#### Scenario: Contact does not freeze cooldowns
- **GIVEN** a delivered packet causes an owner montage pause during Illusion
- **WHEN** real time advances through that pause
- **THEN** action progression follows the pause while form and cooldown deadlines continue in real time
- **AND** additional AoE recipients cannot duplicate one causal packet's pause or resource grant

### Requirement: Hunt and Delusion lifetimes

Without A1, Hunt and Delusion SHALL follow their actual form/action activation and delayed-removal rules, including the sourced overlap after a qualifying form transition. Their damage and DoT-critical modifiers SHALL use their declared recipients and SHALL NOT become permanent through an unrelated attack or field return.

#### Scenario: Overlap ends without a fresh grant
- **GIVEN** a qualifying completed form change retains the previous form's modifier for its overlap duration
- **WHEN** that duration expires or a sourced cleanup condition occurs
- **THEN** only independently active modifiers remain
- **AND** source queries and delivered damage agree on the resulting state

### Requirement: DoT spreading and producer ownership

Heartwrench and Vile Ash applications, spreading and refresh SHALL retain source/target ownership, independent caps, native coefficient composition and tick phases. Only eligible producers SHALL spread the allowed effects. Spreading SHALL NOT multiply one producer's resource gain or let a copied carrier recursively spread itself.

#### Scenario: Spreading retains separate carriers
- **GIVEN** eligible living dummies carry differently stacked Heartwrench and Vile Ash
- **WHEN** a qualifying Illusion attack spreads their allowed effects
- **THEN** each recipient retains independently capped owner-bound carriers and the sourced refresh/phase behavior
- **AND** dead or ineligible targets receive no new application

### Requirement: Remaining awakening and resonance behavior

Selected A3 SHALL apply Hunt's additional interruption resistance and Delusion's sourced periodic self-heal. Selected A4 SHALL unlock Bloodfeast Reverie through its additional prerequisite. Selected A5 SHALL supply its Break effects with actual per-target, per-combat and mode eligibility. Three-/six-node resonance SHALL depend on selected-node count separately from A3/A6 identity.

#### Scenario: A3 passive uses the actual modifier
- **GIVEN** A3 is selected and the relevant Hunt or Delusion modifier is active
- **WHEN** a qualifying interruption-resistance evaluation or one-second healing event occurs
- **THEN** the corresponding fifty-percent resistance or four-percent-Max-HP self-heal applies
- **AND** source defeat removes living-source effects without healing unrelated teammates

#### Scenario: A4 unlock and A5 once-only Break
- **WHEN** A4's sourced Ultimate-ready condition is reached or an A5 enhanced-entry attack hits an eligible target
- **THEN** the corresponding earned Ultimate permission or once-per-target forced Break occurs
- **AND** mode-excluded bosses and repeat A5 contacts cannot bypass the actual eligibility ledger
- **AND** A5's team Break-damage contribution uses its sourced recipient scope

#### Scenario: Resonance does not substitute an awakening
- **GIVEN** three selected nodes omit A3, or fewer selected nodes include A3
- **WHEN** talents, Ultimate ratios and A3 passive effects are evaluated
- **THEN** the three-node talent/ratio bonus and the A3 passive each follow their own prerequisites
- **AND** six-node timed ATK and Scorch-defeat effects cannot activate from A6 alone

### Requirement: Live form and readiness observations

Source queries and reports SHALL expose actual form, form time, relevant enhancement permissions and selected Skill/Burst readiness using owner admission state. Queries SHALL NOT create permissions, resource grants or future knowledge. Native replay and WASM/Worker execution SHALL preserve the same state and source identity.

#### Scenario: Ready means the currently selected variant
- **GIVEN** enhanced Soulcross or a free Inferno Flamenco has become available
- **WHEN** the source reads readiness and then submits the corresponding ordinary input
- **THEN** query and admission select the same variant and real deadline
- **AND** replay reconstructs the same cost, packets and form transition
