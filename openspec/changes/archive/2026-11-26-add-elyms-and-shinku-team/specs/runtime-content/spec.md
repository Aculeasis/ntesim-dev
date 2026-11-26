## MODIFIED Requirements

### Requirement: Released Character Registry Includes Akane Rin

`ntesim-data`, `ntesim-content`, `ntesim-research`, and Studio contracts SHALL include Akane Rin (`character:1057`, slug `akane`, script alias `Akane Rin`) within the released roster (`25` characters, `12650` four-member combinations), supporting `start_string_energy`, live `weapon:fork_prism` pairing, `'akane'` portrait mapping, `akane_hit` energy attribution, and regenerated catalog/name assets while preserving `recommended-gear.csv` unchanged.

#### Scenario: Expanded released roster, research combinations, and DSL catalog
- **WHEN** callers query released characters, four-character team combinations, resonance parity fixtures, or script catalogs
- **THEN** `25` released characters and `12650` team combinations are exposed including `character:1057` (`akane` / `Akane Rin`)
- **AND** `presets/research/recommended-gear.csv` remains unchanged

#### Scenario: Far Side of the Moon character pairing and Studio bundle parity
- **WHEN** `Akane Rin` equips `weapon:fork_prism` (*Far Side of the Moon*) or Studio loads localization and portrait assets
- **THEN** `Synced Resonance` (`RedirectSkill`) and `Perfect Pitch` (`Category E`) interoperate with `weapon:fork_prism` triggers
- **AND** `'akane'` resolves to `characters_akane-rin.webp` and `akane_hit` resolves in energy attribution reports alongside regenerated Studio catalog and name bundles

## ADDED Requirements

### Requirement: Elyms runtime content and identity parity

`ntesim-data`, `ntesim-content`, `ntesim-research`, and Studio contracts SHALL include Elyms (`character:1090`, slug `elyms`, script alias `Elyms`) as the 25th released character (`12650` four-member combinations), supporting live `weapon:fork_elyms` pairing, `Aim` binding, `FOLLOW_UP_DAMAGE_TAG` on `Remembrance` packets, `elyms_hit` energy attribution, and regenerated `catalog.json`/`names.json` assets while preserving `recommended-gear.csv` unchanged.

#### Scenario: Enumerate 25 released characters and 12650 combinations including Elyms
- **WHEN** `ntesim-data`, `ntesim-research`, `ntesim-wasm`, and Studio catalog/localization tests query released characters and four-character combinations
- **THEN** `25` released characters and `12650` team combinations are exposed including `character:1090` (`elyms` / `Elyms`)
- **AND** `presets/research/recommended-gear.csv` remains unmodified

#### Scenario: Resolve Elyms trace energy and signature weapon follow-up parity
- **WHEN** Elyms casts `On the Line` / `Remembrance` (`XieTongSkillA1`/`B1`) while wielding `weapon:fork_elyms` or emits `elyms_hit` energy receipts
- **THEN** `Remembrance` carries `State.Damage.Followup` under `GA_Elyms_UltraSkill` so `weapon:fork_elyms` (`Echoes on Hold`) activates its post-Ultimate follow-up procs
- **AND** Studio maps `elyms_hit` through `energy-sources.ts` cleanly without unknown-source errors

### Requirement: Non-Ultimate cinematic pause and shared passive simulation hooks

The simulation runtime SHALL support non-Ultimate `ModePauseKind::Cinematic` admission on per-packet/per-ability rules (admitting Elyms `Skill2` and `Skill3` for `1_719_655 us` while leaving `Skill1` unpaused), an active-actor Ultimate Energy grant observer across combat energy grants, and a non-retriggering `Remora` source flag that suppresses recursive `Remora Enhancement` scheduling.

#### Scenario: Shared runtime hooks maintain pause, energy observer, and Remora recursion isolation
- **WHEN** Elyms casts `Skill2` or `Skill3`, an active ally gains Ultimate Energy in combat, or Elyms's `7.5s` delayed `Remora` applies to a target
- **THEN** `ModePauseKind::Cinematic` pauses relative clocks for `Skill2`/`Skill3` (`1_719_655 us`) without affecting `Skill1` or existing characters' cooldown-preserving bypasses
- **AND** active-actor energy grants notify Passive 1 `Charge Enhancement`, while the `Remora` source flag prevents `Remora Enhancement` from re-scheduling itself
