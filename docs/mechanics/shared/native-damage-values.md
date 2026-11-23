# Native damage values

`damage-values REQUEST.json` binds
explicitly selected native damage rows and an explicit array coordinate to
audited evidence. It does not schedule hits or compute game damage/DPS.
This supplies scalar evidence for the [S1 review](../characters/mint/build-candidate-context.md) and Q1;
all gameplay execution claims remain INCOMPLETE.

## Request and source binding

The schema-1 request requires `content_id: global-2026-10-04`, a
`character:native_id`, one configured `ability_id`, 1..64 distinct explicit
`effect_ids`, and a nonnegative integer `array_index`. Unknown fields fail.
The coordinate is **zero-based native array indexing**, not a purchased or
effective skill level. There is no implicit level conversion, awakening
bonus, interpolation, or default coordinate.

The CLI audits the portable package before resolving `typed_native_damage`.
The 378 consumed rows are typed Rust constants, with exact native numeric
units stored as integer millionths. The original `DT_SkillDamageData` export
and 575 unconsumed candidates remain cache-only.
Consumed action identities and purchase prerequisites come from typed Rust
`character_actions`; the original configuration export is cache-only.
Typed character definitions and action
ownership must match. Damage IDs use the existing checked ASCII
native lookup, and duplicate canonical selections fail. Selected rows must
belong to that ability through `GAName`; selection order is retained.

`GAName` alone does not prove an owner branch is executable. Retained rows
serve existing dispatch, Support or explicit packet/callback boundaries;
an unsupported branch still diagnoses before execution. Source-only test
and unconsumed candidates are omitted rather than offered as runtime content.
Effect-class asset paths and mapping indices are omitted: no simulation
consumer used them. Their original export and source identity remain cache-only.
Damage row identity and `GAName` ownership still bind each selected coefficient.

## Values and unknowns

Each selected row retains native damage type/category and exact nonnegative
fractions for `ChargeAdd`, `UnbalValue` and `FixedCritRate`.
`HeterochromeAdd`, `StroyBlanceGERate` and destruction/impulse fields are
source-only observations without a simulation consumer; reports omit them.
Every value of the selected `AtkRateBaseArray`, `DefRateBaseArray` and
`HPRateBaseArray` is checked before selecting the coordinate. Missing,
malformed, negative or overflowing values fail. Nonempty arrays require an
in-range coordinate; no fixed array length is assumed. A declared empty
array reports `declared_empty` and a null selected value, not zero damage.

The unbound `GACost`/`GAStrengthCost` source observation is omitted: no
simulation consumer used it. The original table and source identity are
cache-only. Executable action costs remain owned by the existing owner and
scenario rules; omitting a source observation does not make an action free.

At array coordinate 7, the bundled Mint skill row
`GE_Player_Mint_Skill1_Damage_New` has `AtkRateBaseArray = 771/500 = 1.542`,
`ChargeAdd = 49/20 = 2.45`. The cache-only source observation
`HeterochromeAdd = 41/10 = 4.1` is independently checked without entering runtime.
`GE_Player_Mint_UtraSkill1_Damage_New` has `AtkRateBaseArray = 2273/1000`.
The cache-only cost evidence records ultimate cost 100 and strength cost 0;
the skill has no cost row. These historical source fields are omitted from
the runtime package and reports.

The earlier `DT_GlobalValueMintData.Mint_Skill_1_1` curve gives 127 at
coordinate 8. That is a different serialized curve from the damage array.
Neither this difference nor the similar coordinate numbers establish patch
conflict, percentage units, runtime level dispatch or formula application.
Do not substitute the display/global curve for the damage row or infer that
purchased skill level 8 necessarily selects array coordinate 7.

Reports preserve typed source IDs/pointers, original request hash, audited
metadata hashes, engine version and source fingerprint. `reached_by_action`,
`hit_count` and resource recipients remain null.
`quality: incomplete`, `status: INCOMPLETE`,
`combat_adapter_complete: false` and `game_dps_computed: false` are explicit.
Release/hotfix/decoder applicability, delivery, geometry, capture/crit,
rounding and clocks remain separate evidence requirements. Exit 0 confirms
report generation; invalid requests or evidence exit 1.

## Verification and reproduction

From the portable product directory:

```powershell
python tools/cargo.py build --locked --offline -p ntesim-cli
python tools/cargo.py run --locked --offline -p ntesim-cli -- damage-values examples/mint-skill-values.json
python tools/check.py -- python tools/verify_damage_values_cli.py --out target/verification/damage-values.json
```

On non-Windows systems pass `--cli target/debug/ntesim` to the verifier.
The [owning Rust checks](../../../crates/content/src/simulation/tests/damage_bonuses.rs) and
current Rust tests are the maintained checks.
they are not product inputs.

An independent original-export Decimal/Fraction oracle checks 39 effect
coordinates: five selected melee rows, one skill row and seven ultimate
rows at coordinates 0, 7 and 12. It checks scalar/hash receipts,
11 invalid requests, audited-input mutation rejection, byte-identical
replay and relocation, and unchanged source inputs. Current scalar/CLI tests cover the selected binding and rejection boundaries. These checks establish partial arithmetic and source binding only.
