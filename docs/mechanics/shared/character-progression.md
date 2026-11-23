# Typed character constants and progression

`character-stats REQUEST.json` compiles the 23 released characters' primary stats and energy capacities from [typed Rust records](../../../crates/data/src/constants/character/values.rs). The remaining `DT_Character` input supplies display, element and equipment-group metadata. Runtime values do not load base-stat or level-modifier JSON tables. Unreleased aliases have no retained stat record and diagnose.

## Retained values and consumers

Each record has only native identity, base HP, ATK, DEF and energy capacity. Identity selects the record. HP/ATK/DEF feed exact static build projection, equipment zones and live percentage buffs; capacity feeds actor energy limits. Capacity does not imply initial energy. Shared ordinary defaults are CRIT Rate `1/20`, additional CRIT DMG `1/2`, charge efficiency `1`, Break efficiency `1` and zero in the stat domains read by the current combat adapter.

Satiety, traction mass, resistance/immunity arrays, recovery/world/cost fields and other unconsumed native properties are absent. The old `DT_PlayerPackData` and `DT_PlayerModifyPackData` exports and character metadata's three stat-pack references are removed. Original evidence remains optional cache material outside Git. Weapon, module and live-stat contributions keep their separate owners and zones.

## Exact growth rule

For base primary stat `B`, selected character level `L` and selected breakthrough stage `S`:

```text
HP/DEF growth  = round_positive(B * 8 * (L - 1) / 100)
ATK growth     = round_positive(B * 5 * (L - 1) / 100)
HP/DEF bonus   = round_positive(B * 8 * S / 10)
ATK bonus      = round_positive(B * 5 * S / 10)
total          = B + growth + bonus
```

`round_positive` rounds to the nearest integer, positive halves upward, using integer arithmetic. Round growth and the terminal bonus separately **before** addition. For base ATK81 at level11/stage1, both terms are 40.5 and round to41: `81 + 41 + 41 = 163`, rather than rounding their sum to162. Historical breakthrough stages are not accumulated. Overflow diagnoses.

Before deleting the exports, these rules were compared independently with all **5,520 growth values and 414 terminal bonus values** for the 23 released characters, with no mismatches. Every retained source pack was unconditional and additive and contained only HP/ATK/DEF. This is the exact frozen Global model over supported levels, not a claim about future versions or obtaining a build in game.

The [shared Rust caps](../../../crates/data/src/constants/character/progression.rs) are `[20, 30, 40, 50, 60, 70, 80]` for stages0..6. Levels must be1..80 and no higher than the selected cap. Character stats and talent progression share these caps. Stage zero contributes no breakthrough bonus and preserves the explicit historical hypothesis that absent unascended packs add nothing. Higher-stage/lower-level inputs preview constants without certifying acquisition prerequisites.

The request requires schema1, package `global-2026-10-04`, canonical `character:NATIVE_ID`, integer level and breakthrough. Unknown fields, missing fields, booleans, negative/fractional/out-of-range selections and unsupported identities fail. The [example](../../../examples/character-constants.json) selects Sakiri at level80/stage6.

## Output and provenance

Reports retain `kind: character-constants-v1`, exact attribute fractions, three typed calculation terms and two evidence entries: the remaining character metadata row and a typed-stat receipt with base, growth, terminal bonus and energy capacity. Rust constants are bound by the engine source fingerprint; no receipt pretends that removed JSON rows remain inputs. The CLI additionally hashes the exact request, remaining package metadata and `DT_Character` input, rejects changed audited inputs and replays identically after relocation.

Only independently released coverage identities receive released status. Quality remains `incomplete`; this stat service does not execute skills/passives, choose awakenings or establish full build legality. Successful constant compilation alone cannot authorize a strict combat run. Content-owned simulation adapters supply those checks separately.

## Verification

```powershell
python tools/cargo.py test --locked --offline -p ntesim-data constants::character
python tools/check.py -- python tools/verify_character_cli.py --out target/verification/character-constants.json
python tools/checkpoint.py run
```

Rust checks all 23 independent published CN level80 endpoints, separate half rounding, caps, stage selection, unsupported identities, request validation and omitted unused fields. CLI checks those published endpoints plus five hand coordinates, typed receipt channels, file/process boundaries, stage-zero assumptions, replay/relocation, invalid requests and corrupted remaining input rejection. Equipment consumers reuse Rust character output and independently verify their equipment arithmetic; they do not maintain another growth engine. Native/WASM and actual Worker aggregation/replay remain required.

Sakiri at level80/stage6: base `(HP1360, ATK78, DEF75)`, growth `(8595, 308, 474)`, terminal `(6528, 234, 360)`, total **HP16483, ATK620, DEF909**, capacity120.
