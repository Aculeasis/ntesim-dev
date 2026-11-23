# Spec Delta

## MODIFIED Requirements

### Requirement: Reproducible artifacts

Distributable artifacts SHALL retain executable reproduction instructions, current engine and input identities, required notices and supported target limits. Reproduction on a supported machine SHALL use checkout-relative paths and documented prerequisites without depending on the author's private configuration or filesystem layout.

#### Scenario: Fresh portable checkout
- **GIVEN** a supported target has the documented pinned tools and product inputs
- **WHEN** the documented artifact workflow runs in a fresh checkout
- **THEN** it can reproduce the artifact and its required identities and notices

#### Scenario: Relocated checkout without author configuration
- **GIVEN** a fresh checkout is placed in a different directory containing spaces and has no local supplements or private environment file
- **WHEN** the documented setup and focused tooling checks run with their prerequisites available
- **THEN** paths resolve within that checkout and no original installation directory, proxy endpoint or credential literal is required
