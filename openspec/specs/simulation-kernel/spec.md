# Event and resource kernel

## Purpose

Defines deterministic event delivery, exact clock advancement and conservative resource and health settlement.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

### Requirement: Deterministic event order

Events SHALL use exact integer time and stable identity ordering for equal real timestamps, including immediate causal children.

#### Scenario: Equal-time child delivery
- **GIVEN** an event is delivered with another event already queued at the same timestamp
- **WHEN** the delivered event schedules an immediate child
- **THEN** the earlier queued event retains its order before the new child

### Requirement: Paused local deadlines

Clock advancement SHALL retain fractional local phase and pending local deadlines when rates change or pause.

#### Scenario: Resume a paused clock
- **GIVEN** a clock has a pending local deadline
- **WHEN** its rate becomes zero and later becomes positive
- **THEN** its deadline pauses and resumes without losing retained phase

### Requirement: Conservative settlement

Resource and health settlement SHALL distinguish applied, excess, absorbed and lost amounts and reject invalid spending atomically.

#### Scenario: Resource grant overflow
- **GIVEN** a resource pool contains 8 units with capacity 10
- **WHEN** 5 units are granted
- **THEN** 2 units are retained, 3 are reported as excess and the pool becomes 10
