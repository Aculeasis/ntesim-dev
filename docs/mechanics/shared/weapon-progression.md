# Native weapon constant binding v1

The [combat formula contract](damage-and-stat-zones.md) owns stat units and zone composition. Weapon base ATK joins character base ATK; additive secondary stats join their corresponding stat sums. Refinement parameters do not become unconditional or independent multipliers.

`weapon-stats REQUEST.json` compiles constant weapon properties and selected refinement identity from consumed typed Rust constants for the audited frozen `global-2026-10-04` package. Explicit in-memory overlays keep independent schema/source validation; filesystem loading and evidence auditing belong to the native CLI. This is the supported scalar portion of plan A3, not a full weapon/passive adapter.

## Input and interpretation

All fields in [the request example](../../../examples/weapon-constants.json) are mandatory: `schema_version: 1`, the exact package ID, `weapon_id: weapon:NATIVE_ID`, integer `level` (1..80), `breakthrough` (0..6) and `refinement` (1..5). Unknown fields, booleans, invalid ranges, unknown identities and unsupported packages fail. Native ASCII row references use case-insensitive lookup and return the canonical frozen ID; ambiguous case aliases or non-ASCII table keys fail rather than guessing FName semantics.

The compiler resolves typed definition/progression/stat/refinement bindings:

1. [Typed weapon definitions](../../../crates/data/src/weapon_definitions/mod.rs): 51 identities, display aliases, rarity, compatibility groups, progression pack references/limits and explicit exclusive-owner lists. Public metadata and all owner checks share this validated transport. Inventory/acquisition fields, source-only localization and artwork/model paths from cache-only `DT_ForkItemData` are omitted. Evidence identifies `typed_weapon_definitions`; absent data in explicit legacy overlays does not use the baseline.
2. [Typed weapon upgrade levels](../../../crates/data/src/weapon_upgrade_levels.rs): the chosen level's modifier pack. Ten referenced groups expose 800 exact bindings for levels 1..80. The 800 unreferenced source rows and `NeedExp` are omitted; the original `DT_ForkUpgradeData` export stays cache-only. Evidence identifies `typed_weapon_upgrade_levels`. Explicit legacy overlays retain validation without filling missing rows from the baseline.
3. [Typed weapon breakthrough](../../../crates/data/src/weapon_breakthrough.rs): the chosen terminal breakthrough's modifier pack and maximum level. All 51 referenced groups retain seven exact coordinates (357 bindings), with caps 20/30/40/50/60/70/80. Currency/material costs from the cache-only `DT_ForkBreakthroughData` export are omitted. Evidence identifies `typed_weapon_breakthrough`; explicit legacy overlays keep independent validation without filling absent data.
4. [Typed weapon modifiers](../../../crates/data/src/weapon_modifiers/mod.rs): unconditional additive native-property scalars from those two selected packs. Ten exact 80-level integer attack arrays and 51 seven-stage attack/secondary arrays retain 1,157 consumed packs. Secondary channels use exact ten-thousandths of their native units (BI remains points), and explicit zero attack stays present. The 800 unreferenced level packs and `SortKey` fields from cache-only `DT_ForkModifyData` are omitted. Evidence identifies `typed_weapon_modifiers`; explicit overlays still reject unsupported operations, conditions and malformed scalars.
5. [Typed weapon refinements](../../../crates/data/src/weapon_refinements/mod.rs): the chosen refinement row's identity/provenance. Ordered parameter names/percent hints and English descriptions serve their separate passive/owner consumers. Unused buff-class references/flags, costs, titles and source localization from cache-only `DT_ForkUpgradeStarDataTable` are omitted. Evidence identifies `typed_weapon_refinements`.

Add the level pack to the selected terminal breakthrough pack once each. Historical breakthrough packs are prior states and are never all summed. Refinement changes the selected row and passive parameters, while these constant stats stay equal. This interpretation was independently checked against published level-80 endpoints in the preceding acquisition research; the portable verification retains explicit known endpoints and separately compares the Rust process against original export rows using Python `Decimal`/`Fraction`.

Unknown/missing values, conditional modifier packs, unsupported operations, negative constants and checked arithmetic overflow fail explicitly. Values remain in native property units, with no display percentage conversion or rounding. Numerator/denominator strings preserve exact sums over the normalized JSON number representation parsed by serde_json. This does not claim to recover a native binary float's original precision. The frozen selected values fit this contract; future schemas need separate reviewed support.

The level cannot exceed the selected breakthrough's maximum or the item's declared limits. Full progression legality, character compatibility and release availability still need the complete build/release contracts. A stat preview does not certify that every lower-level/higher-breakthrough request is obtainable in game.

## Output, provenance and limits

Reports have `kind: weapon-constants-v1`, `schema_version: 1`, canonical weapon identity, explicit progression selection, exact `attributes`, the two selected modifier steps, the selected refinement row and six row evidence receipts. Each receipt supplies source ID, table, canonical row and escaped JSON pointer. Source-only buff-class arrays and their static build/team report echoes are removed; gameplay buffs remain owned by Rust adapters. Static reports keep explicit unvalidated-passive status, including with empty listed parameters.

The CLI audits package sources, remaining retained rows, inventory and partial fixtures before compilation. It hashes the exact request and four package metadata files; snapshots taken before audit must still match afterwards. Reports also bind engine version/source fingerprint for the typed constants and the ordinary audit summary. Filesystem locations and ambient time are absent, so unchanged input and engine replay after relocation byte-for-byte.

Quality stays `incomplete`, release stays `unverified`, and `passive_adapter_complete` and `full_build_legality_checked` stay false. Frozen coverage inputs are not rewritten. This command cannot authorize a strict game run, calculate DPS, select optimal gear or establish weapon release from table presence.

Exit 0 means the requested constant-data compilation succeeded, while its gameplay quality remains incomplete. Invalid schema/requests, integrity failures or unsupported constant rules exit 1 before a report is printed. Strict scenario validation retains its independent rejection of absent adapters.

## Reproduction and checks

From the product directory:

```powershell
python tools/cargo.py run --locked --offline -p ntesim-cli -- weapon-stats examples/weapon-constants.json
python tools/cargo.py build --locked --offline -p ntesim-cli
python tools/check.py -- python tools/verify_weapon_cli.py --out target/verification/weapon-constants.json
```

On another platform, pass `--cli target/debug/ntesim` to the verification tool. Independent numeric checks verify the item, upgrade, breakthrough, modifier and refinement parameters; CLI execution and relocation need only the runtime package. Original evidence files are never mutated.

The Last Rose at level 80/breakthrough 6 has `AtkBase = 416 + 154 = 570` and `CritBase = 6/25` (24% in display units). Eternal Waltz retains `HPMaxUp = 33/80` (41.25%) and `AtkBase = 424`; Contemplative Cat retains `AtkBase = 512` and `CritDamageBase = 11/25`. These are constant-stat endpoints, not complete precombat stats.

Rust tests cover terminal-only addition, case lookup, refinement separation, exact decimals/scientific notation/overflow, malformed/ambiguous/conditional/unsupported data and native CLI/hash/quality behavior. The portable process verifier covers all 51 candidate identities at level 80/T1, representative lower/intermediate levels and terminal stages, all Rose refinements, independent original-export fraction arithmetic, known endpoints, evidence/hash receipts, replay/relocation, invalid requests and corrupted normalized data. Candidate count does not imply 51 released weapons.

The upgrade migration independently checks all 800 retained coordinates against
cached source bindings and exact rational stat sums. Breakthrough checks compare
all 357 selected terminal packs/caps with the original source, reject levels above
each nonterminal cap and preserve exact rational stat sums. Actual Worker checks
compile all 51 weapons across all seven breakthrough stages in native/WASM and
compare execution, aggregation and replay. Those scripts declare `start_fons=0` explicitly for reached Fons-dependent
passive reads; they do not infer funds or alter Chiz's combat resources.
