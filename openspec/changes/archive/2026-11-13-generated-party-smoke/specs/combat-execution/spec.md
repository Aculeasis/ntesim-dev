# Spec Delta

## ADDED Requirements

### Requirement: Bounded aggregate defense ignore

Supported packet, equipment and team DEF-ignore contributions SHALL combine additively with an effective maximum of 100% before native direct or Break damage evaluation. This bounded model SHALL preserve exact arithmetic, source applicability and existing unsupported-dependency diagnostics; it SHALL NOT relax individual formula-request validation.

#### Scenario: Zero execution with Lost Radiance and Iroi support
- **GIVEN** Zero's execution packet supplies 75% DEF ignore, Lost Radiance supplies 25% and an applicable Iroi team buff supplies another 5%
- **WHEN** the packet is evaluated against an authored dummy
- **THEN** effective DEF ignore is exactly 100% and the supported composition completes without an out-of-range arithmetic failure
- **AND** the packet does not gain a defense multiplier greater than one

#### Scenario: Aggregate below the bound
- **GIVEN** applicable DEF-ignore sources total less than 100%
- **WHEN** direct or Break damage is evaluated
- **THEN** the exact summed contribution is retained without rounding or rescaling

#### Scenario: Invalid individual formula request
- **WHEN** an individual exact damage-formula request specifies DEF ignore greater than 100%
- **THEN** formula validation continues to reject that request

### Requirement: Bounded custom model declarations

Canonical engine hypothesis declarations SHALL be admitted independently of the 32-entry allowance for custom declarations. Unknown identities and mismatched canonical descriptions SHALL consume the custom allowance. Existing per-entry size bounds, strict-mode acceptance and conflicting reached-hypothesis diagnostics SHALL remain enforced.

#### Scenario: Four-member party uses many canonical models
- **GIVEN** automatic content projection for a legal party produces more than 32 canonical hypothesis declarations
- **WHEN** its request is validated
- **THEN** the canonical declarations do not cause a metadata count rejection and remain in the execution provenance

#### Scenario: Custom declarations remain bounded
- **GIVEN** a request contains canonical declarations and 32 custom declarations
- **WHEN** one more unknown or mismatched declaration is added
- **THEN** validation rejects the excess custom declaration
