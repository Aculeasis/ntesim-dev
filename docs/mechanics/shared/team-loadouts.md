# Native team loadout and scenario contract v1

`team-stats REQUEST.json` compiles source-bound multi-actor team loadouts and scenario target definitions against the audited frozen `global-2026-10-04` package. The Rust engine operates over in-memory normalized tables; filesystem IO and integrity verification belong to the native CLI. This supplies static B1 inputs under supported constraints. Full native build legality, compatibility/release, resource/awakening/passive execution and strict game coverage remain incomplete.

The example uses Watch Your Heads! (`fork_Kite`) T1 for Skia under native group compatibility; it is not a complete recommended combat build. Independent raw-export checks verify actor base-ATK sums and incompatible-group rejection.

Each actor preserves each actor's native weapon group binding and selected-refinement row/buff array with source pointers. `selected_weapon_buffs`, `weapon_refinement_binding.buffs` and `unresolved.weapon_passive_execution` retain the same native array; no weapon passive is executed by this static report.

## Request and interpretation

The [example request](../../../examples/team-loadout.json) specifies a complete 4-character team and combat scenario:

- Top-level fields:
  - `schema_version`: integer 1.
  - `content_id`: exact `"global-2026-10-04"`.
  - `actors`: array of 1 to 4 actor loadout specifications.
  - `targets`: array of 1 or more anonymous, script-authored stationary dummies.
  - `duration_us`: simulation horizon in microseconds (positive integer).
  - `active_actor_slot`: integer (0..`actors.len() - 1`), designating the initial on-field character (defaults to 0).
  - `mode`: optional string (`"strict"` or `"research"`, defaults to `"strict"`).
- Actor loadout specification:
  - `slot`: integer (0..3). Each actor on the team must occupy a unique slot.
  - `build`: static build request as specified in [the equipment contract](equipment-builds.md) (`character_id`, `weapon_id`, `weapon_tier`, `set_id`, `set_count`, and optional explicit `additive_attributes`).
  - `awakening`: integer (0..6), specifying the awakening tier/choice.
  - `initial_energy`: optional finite nonnegative float representing starting energy; capacity and execution remain unresolved in this static report.
  - `initial_hp_fraction`: optional float in range `[0.0, 1.0]` (defaults to 1.0).
- Target profile specification:
  - `id`: unique string identifier (e.g., `"target_boss"`).
  - `level`: target level (1..120).
  - `base_hp`: base hit points (> 0).
  - `base_def`: base defense value.
  - `resistances`: dictionary mapping elemental identifiers (`"INCANTATION"`, `"CHAOS"`, etc.) to decimal resistance scalars.
  - `weakness`: optional elemental weakness identifier.
  - `break_gauge`: optional break gauge capacity.

### Validation and team legality rules

1. **Distinct characters:** all actors on the team must specify distinct native character identities. Duplicate characters are strictly rejected.
2. **Team size bounds:** a team consists of 1 to 4 actors. Empty actor arrays and teams exceeding four members fail validation.
3. **Slot uniqueness and active slot bounds:** each actor must have a unique slot index within `0..actors.len()`. `active_actor_slot` must be within valid team indices.
4. **Target constraints:** at least one target must be provided. Target IDs must be unique. Target `base_hp` must be positive.
5. **Static build validation:** character/weapon identity, group, progression, exact authored additions and set count (0..4). Only reached 2-/4-piece effects are added automatically. Console geometry, module allocation and specialization are outside the model.
6. **Awakening bounds:** `awakening` must be an integer between 0 and 6.

## Output and provenance

Reports use `kind: static-team-scenario-v1` and provide:

Actor, weapon and set identities come from the shared compiler's canonical native rows, including case-insensitive input aliases. Unknown modes and native element enums reject; missing element metadata cannot become `Unknown`. Explicit zero starting energy and HP fractions are retained.

- `team_size`: number of actors on the team.
- `team_elements`: array of elemental affiliations represented across the team.
- `team_base_atk`: exact fraction representing the sum of all actors' combined base attack (`character_base_atk` + `weapon_base_atk`).
- `active_actor_slot` and `active_character_id`: starting on-field character.
- `total_target_hp`: string decimal representation of combined enemy HP.
- `actors`: array of compiled actor reports, each containing:
  - Slot, character ID, level, breakthrough, element, awakening tier.
  - Weapon ID, level, refinement tier, set ID and declared set count.
  - Release status and evidence pointer from `coverage.json` and `legacy:characters`.
  - Base attack fraction, derived stats (`hp`, `atk`, `def`, `charge_efficiency`, `crit_rate`, `crit_damage`), and known attributes.
  - Unresolved set buffs and mechanic backlog references.
- `targets`: echoing the validated target profiles.
- Provenance hashes: SHA-256 digests for consumed tables and package metadata files, request hash, and engine source fingerprint.
- Quality boundaries: `quality: "incomplete"` and `status: "INCOMPLETE"`. Combat execution, dynamic buffs, and real-time skill dispatch remain unmodeled.

## Verification and reproduction

From the product directory:

```powershell
python tools/cargo.py build --locked --offline -p ntesim-cli
python tools/cargo.py run --locked --offline -p ntesim-cli -- team-stats examples/team-loadout.json
python tools/check.py -- python tools/verify_team_cli.py --out target/verification/team-scenario.json
```

On non-Windows platforms, pass `--cli target/debug/ntesim` to the verification tool.

The verification tool executes independent Python `Decimal`/`Fraction` oracles over raw table exports and asserts:

- 4-character baseline team endpoints:
  - Character 1: Sakiri (`1003`) + Good Boy (`fork_mofeikesi`) -> `base_atk = 1094`, `atk = 2416.55`, `hp = 22083`.
  - Character 2: Lacrimosa (`1004`) + The Last Rose (`fork_Rose`) -> `base_atk = 1206`.
  - Character 3: Skia (`1008`) + Watch Your Heads! (`fork_Kite`) -> `base_atk = 1032`.
  - Character 4: Nanally (`1010`) + Gold Record (`fork_GoldRecord`) -> `base_atk = 1206`.
  - Total team base ATK: exact fraction `4538/1` (1094 + 1206 + 1032 + 1206 = 4538).
  - Team elements: `["CHAOS", "INCANTATION", "LAKSHANA", "NATURE"]`.
- Rejection of 13 invalid requests, including the former incompatible Skia/Arachne pair, alongside identity/slot/target/horizon/awakening/HP/schema guards.
- Byte-for-byte replay and relocation invariance, and rejection of mutated normalized tables.
