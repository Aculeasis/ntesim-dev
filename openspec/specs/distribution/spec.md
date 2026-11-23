# Distribution requirements

## Purpose

Defines reproducible native and browser distribution, artifact-specific attribution and explicit publication authorization.

The [detailed reference](reference.md) retains interface formats, limits and supporting technical detail.

## Requirements

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

### Requirement: Explicit publication

Publication, deployment and release tagging SHALL require explicit user authorization beyond local build and validation.

#### Scenario: Locally validated artifact
- **GIVEN** an artifact passes local checks but publication has not been authorized
- **WHEN** local work completes
- **THEN** the artifact remains available for review without being published

### Requirement: Main-history Pages deployment selection

Studio Pages deployment SHALL run automatically on tag pushes and support manual dispatch from main with an optional exact tag name. A selected tag MUST resolve to a commit reachable from fetched main. Dispatch without a tag SHALL build the latest fetched main commit. Invalid selections SHALL stop before publication. Ordinary branch pushes SHALL NOT trigger deployment.

#### Scenario: Push a main-history tag
- **GIVEN** a lightweight or annotated tag resolves to a commit in main's history
- **WHEN** that tag is pushed
- **THEN** the deployment builds exactly its resolved commit and supplies the tag as Studio's identifier
- **AND** the commit need not be the current main tip

#### Scenario: Dispatch a tagged version
- **GIVEN** manual dispatch selects main and names an existing tag whose commit belongs to main's history
- **WHEN** deployment resolves its source
- **THEN** it builds that tag's exact commit and supplies that tag as Studio's identifier
- **AND** the input is resolved as a tag rather than as an arbitrary branch or revision expression

#### Scenario: Dispatch the latest main
- **GIVEN** manual dispatch selects main and leaves the tag input blank
- **WHEN** deployment fetches and resolves main
- **THEN** it builds that fetched tip and displays its short commit hash without a branch prefix
- **AND** a tag already attached to the commit does not replace the hash

#### Scenario: Reject an ineligible selection
- **WHEN** a pushed or requested tag is missing, does not resolve to a commit, or points outside main's history, or manual dispatch selects a branch other than main
- **THEN** the run reports the ineligible selection and does not publish a Pages artifact
- **AND** it does not fall back silently to main or another version

#### Scenario: Push an ordinary branch commit
- **WHEN** main or another branch receives an ordinary commit push without a tag push
- **THEN** this deployment workflow is not triggered automatically

### Requirement: Single-job Studio Pages delivery

The Pages workflow SHALL build and deploy Studio in one job without running test suites, checkpoint or team DPS recalculation. Publication SHALL retain current WASM, committed Team list outcomes with their recorded calculation identities, required notices and offline assets. Delivered URLs SHALL work beneath `/ntesim-dev/`. A build failure SHALL prevent deployment of replacement contents.

#### Scenario: Prepare and publish a version
- **WHEN** a valid tag or manual deployment runs
- **THEN** one job installs documented pinned build prerequisites, builds the browser site, uploads its contents and deploys them to GitHub Pages
- **AND** it does not invoke native, frontend, transport or browser test suites
- **AND** it reuses committed Team list outcomes without recalculating DPS
- **AND** saved failed, incomplete or unsupported outcomes remain unavailable according to the existing Studio requirements

#### Scenario: Serve the project subpath
- **GIVEN** the generated site is hosted at `https://aculeasis.github.io/ntesim-dev/`
- **WHEN** a viewer loads or reloads Studio
- **THEN** application assets, engine WASM, codec WASM, Worker and offline update resources resolve beneath that site path

#### Scenario: Build fails before deployment
- **WHEN** source resolution or a required build/preparation/upload step fails
- **THEN** no deployment step publishes incomplete replacement site contents

### Requirement: GitHub deployment build CPU policy

GitHub-hosted Studio deployment builds SHALL be able to use all CPUs available to their runner without the local four-CPU cap. Ordinary local tools and validation SHALL retain the shared four-CPU limit. The deployment exception SHALL NOT become the default for local invocations.

#### Scenario: Build on GitHub runner
- **GIVEN** a GitHub-hosted deployment runner exposes more than four CPUs
- **WHEN** the deployment job compiles WASM and prepares the site and Team list
- **THEN** build tools and children can use the available CPUs without project-imposed four-CPU affinity or default worker caps

#### Scenario: Build and validate locally
- **WHEN** ordinary build tools or validators run locally
- **THEN** the existing shared four-CPU cap and any smaller caller-provided limits remain enforced

### Requirement: Studio hosting documentation

README SHALL link to `https://aculeasis.github.io/ntesim-dev/`. Deployment documentation SHALL describe main/tag selection, manual inputs and required Pages settings.

#### Scenario: Configure Pages deployment
- **WHEN** a maintainer follows the deployment documentation
- **THEN** it identifies GitHub Actions as the Pages source and explains main/tag selection
- **AND** it describes environment rules admitting main dispatches and intended tag events
