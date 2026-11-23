# Nanally selected combat behavior

## Purpose

Defines the selected Nanally execution model against stationary dummies. [Detailed model](reference.md) retains exact timings, accounting, sources and limits; these are revisable model policies, not frame-verified game facts.

## Requirements

### Requirement: Ordinary chains and energy

Nanally SHALL retain the selected five-stage packet chain, contact-dependent owner timing and one ordinary-energy grant per qualifying packet group, independent of recipient multiplicity.

#### Scenario: Multiple recipients share one grant
- **GIVEN** an ordinary packet group damages more than one living dummy
- **WHEN** the positive HP-loss contacts settle
- **THEN** the group grants ordinary energy once and does not duplicate its contact pause per recipient

### Requirement: Authority and Skill

Nanally SHALL apply the selected Authority lifecycle and Skill rules with owner-defined early-end, recast and state cleanup.

#### Scenario: Authority ends early
- **GIVEN** Authority is active and its supported early-end condition occurs
- **WHEN** the owner settles the end transition
- **THEN** Authority ends and the selected recast or cleanup state follows the detailed model

### Requirement: Underboss lifetime

Underboss follow-up groups SHALL retain the creating Burst talent and dispatch conditions; delivery SHALL require that the owning summon is alive and unreplaced at its half-open lifetime boundary.

#### Scenario: Replaced summon
- **GIVEN** a follow-up group belongs to an earlier Underboss instance
- **WHEN** that instance is replaced before delivery
- **THEN** the stale group does not deliver as the new summon

### Requirement: Plunge assumptions

A supported plunge SHALL consume positive declared real flight time and apply its declared height bonus only to the owner landing coefficient, preserving separately owned Cat behavior.

#### Scenario: Plunge lands with Underboss
- **GIVEN** a valid plunge has a height bonus and a live Underboss
- **WHEN** the landing packet group executes
- **THEN** the owner landing uses its declared height multiplier and the Cat landing retains its own unmultiplied coefficient

### Requirement: Continuing ownership

Held actions, Dodge continuation, summons, resources and off-field damage SHALL preserve selected owner state and causal identities across continuing cycles.

#### Scenario: Off-field follow-up damage
- **GIVEN** Nanally has dispatched an independent owned follow-up
- **WHEN** field control switches before that follow-up lands
- **THEN** the follow-up retains its actual owner and supported delivery rules
