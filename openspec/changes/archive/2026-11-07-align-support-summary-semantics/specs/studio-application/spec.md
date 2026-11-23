# Spec Delta

## MODIFIED Requirements

### Requirement: Native support summaries

Completed-trial statistics SHALL include gross, applied and excess healing, gross HPS in HP/s, time-averaged available shield HP (SHP), and actual absorbed shield damage as a separate total. HPS and SHP SHALL use actual elapsed combat time. Cards, histograms and exports SHALL share native per-trial values and explicit units. Zero activity SHALL produce zero statistics; failed/incomplete trials SHALL remain excluded from successful aggregates.

#### Scenario: Effective healing and overhealing
- **WHEN** a completed 10 second trial records 1000 gross healing with 600 applied and 400 excess
- **THEN** its HPS is 100 and the report retains all three healing totals
- **AND** ordinary and autonomous healing are counted once from their canonical healing receipts

#### Scenario: Healing at full health
- **WHEN** a completed 10 second trial records 1000 gross healing entirely as excess
- **THEN** HPS is 100 even though applied healing is zero

#### Scenario: Shield absorption
- **WHEN** a completed 10 second trial records 300 shield absorption and 200 damage prevented by a separate immunity effect
- **THEN** the absorbed shield damage total is 300 and SHP is calculated independently from available shield capacity over time
- **AND** neither unused shield capacity nor non-shield prevention is counted as absorbed shield damage

#### Scenario: Redirected incoming damage
- **WHEN** an incoming event contains redirected child deliveries
- **THEN** each delivery's explicit shield absorption contributes once to the team total

#### Scenario: Shield without incoming damage
- **WHEN** a completed 10 second trial has 10000 HP of available shield capacity on its active actor for exactly 5 seconds and no shields for the other 5 seconds
- **THEN** SHP is 5000 HP even if incoming damage and absorbed shield damage are zero
- **AND** shield downtime remains part of the averaging duration

#### Scenario: Distribution and units
- **WHEN** a user inspects support cards, selects their histograms or exports trial results
- **THEN** HPS is identified as healing including overheal in HP/s and SHP as mean available shield capacity in HP
- **AND** card means, minima, maxima, standard deviations and quartiles are derived from the same native per-trial values as the histograms and exports

## ADDED Requirements

### Requirement: NTE available shield accounting

SHP SHALL average remaining absorbable shield capacity on the active living actor over the completed trial. It SHALL respect native shield stacking, recipients, replacement, depletion, defeat, switches and exact expiry times. Off-field copies SHALL NOT inflate this value. Immunity and non-shield interception SHALL NOT contribute. Missing or invalid observations SHALL NOT become successful zero-valued summaries.

#### Scenario: Actor switch changes available protection
- **WHEN** actor A has a 10000 HP shield and is active for the first 4 seconds of a 10 second trial, then unshielded actor B is active for the remaining 6 seconds
- **THEN** SHP is 4000 HP
- **AND** actor A's off-field shield does not contribute after the switch

#### Scenario: Additive NTE shields and replacement
- **WHEN** the active actor has distinct concurrently absorbable shields with 3000 and 2000 HP remaining
- **THEN** available capacity is 5000 HP for that interval
- **AND** replacing the first shield with a 4000 HP instance changes capacity to 6000 HP rather than counting the replaced instance again

#### Scenario: Expiry during an idle interval
- **WHEN** a 10000 HP shield starts at 2 seconds, expires at 5 seconds, and the next observed event is at 9 seconds in a completed 10 second trial
- **THEN** the shield contributes exactly 3000 HP to SHP
- **AND** the later observation time does not extend its lifetime

#### Scenario: Depletion and defeat
- **WHEN** incoming damage reduces a shield's remaining HP or defeats its recipient
- **THEN** subsequent positive-duration intervals use the reduced remaining capacity or zero capacity for the defeated actor
- **AND** same-time settlements introduce no artificial duration

#### Scenario: Early completion and invalid observations
- **WHEN** a trial completes early
- **THEN** SHP uses its actual elapsed combat time and ignores shield lifetime beyond completion
- **AND** missing or invalid required shield observations prevent successful aggregation rather than supplying a guessed value
