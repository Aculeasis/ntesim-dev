# Spec Delta

## ADDED Requirements

### Requirement: Released Character Registry Includes Akane Rin
`ntesim-data`, `ntesim-content`, `ntesim-research`, and Studio contracts SHALL include Akane Rin (`character:1057`, slug `akane`, script alias `Akane Rin`) as the 24th released character (`10626` four-member combinations), supporting `start_string_energy`, live `weapon:fork_prism` pairing, `'akane'` portrait mapping, `akane_hit` energy attribution, and regenerated catalog/name assets while preserving the 21-team comparison collection and `recommended-gear.csv` unchanged.

#### Scenario: Expanded released roster, research combinations, and DSL catalog
- **WHEN** callers query released characters, four-character team combinations, resonance parity fixtures, or script catalogs
- **THEN** `24` released characters and `10626` team combinations are exposed including `character:1057` (`akane` / `Akane Rin`)
- **AND** the curated 21-team Studio comparison list and `presets/research/recommended-gear.csv` remain unchanged

#### Scenario: Far Side of the Moon character pairing and Studio bundle parity
- **WHEN** `Akane Rin` equips `weapon:fork_prism` (*Far Side of the Moon*) or Studio loads localization and portrait assets
- **THEN** `Synced Resonance` (`RedirectSkill`) and `Perfect Pitch` (`Category E`) interoperate with `weapon:fork_prism` triggers
- **AND** `'akane'` resolves to `characters_akane-rin.webp` and `akane_hit` resolves in energy attribution reports alongside regenerated Studio catalog and name bundles

### Requirement: Shared Remora Reapplication and Pre-Overwrite Duration Hooks
Shared `Remora` (`Cosmos + Lakshana`) reaction handling SHALL provide an owner-sourced periodic `Remora` application path honoring active team `Remora` enhancements, and SHALL sample the target's existing active `Remora` remaining duration (`existing_remaining_us`) BEFORE overwriting or extending its `Remora` timer when notifying reaction listeners alongside existing `Hathor` B2 and `Chaos` B2 hooks.

#### Scenario: Existing Remora remainder delivered before timer refresh
- **WHEN** a character applies or periodically ticks `Remora` onto a target that currently has an active `Remora` status with `existing_remaining_us > 0`
- **THEN** reaction listeners receive `existing_remaining_us` captured strictly before the target's `Remora` expiration is refreshed
- **AND** initial application onto a target without active `Remora` reports no pre-existing remainder

#### Scenario: Owner-sourced periodic Remora ticks preserve Cycle reaction accounting
- **WHEN** Akane Rin's `Overtone` or `Reverb Field` applies a `3.0s` periodic `Remora` tick onto a target
- **THEN** the tick honors active `Hathor` B2 duration enhancement and notifies `Chaos` B2 and `Perfect Pitch` listeners
- **AND** Esper Cycle switch reaction counters continue distinguishing Cycle-triggered reactions from passive periodic applications
