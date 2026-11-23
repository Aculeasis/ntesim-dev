# Bounded synthetic search

## Purpose

Defines exact ranking over an explicitly declared finite candidate set and common seeds. The [detailed reference](reference.md) owns formats, bounds and identities.

## Requirements

### Requirement: Explicit bounded candidates

Synthetic search SHALL evaluate only the declared finite candidate list and common seed list, rejecting invalid inputs or an excessive combined work estimate before simulation.

#### Scenario: Combined event budget exceeded
- **GIVEN** candidate and seed counts imply more than the allowed total events
- **WHEN** the search is preflighted
- **THEN** it rejects the request rather than evaluating a partial candidate set

### Requirement: Complete exact ranking

Ranking SHALL compare exact objective sums with stable tie order and SHALL be withheld if any requested candidate or seed is incomplete.

#### Scenario: One pending candidate
- **GIVEN** one candidate finishes with a delayed hit still pending
- **WHEN** the search is reported
- **THEN** all candidate receipts remain visible while ranking and best candidate IDs are null
