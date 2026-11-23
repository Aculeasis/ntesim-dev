# Authored console stats and equipped set effects

`build-stats REQUEST.json` compiles character and weapon constants, authored console stats and declared set effects. Filesystem loading and package auditing belong to the native CLI; Rust owns stat composition. Its static report does not by itself establish complete passive or gameplay coverage.

The public build artifact is the shared NTE script. Authors enter console stats with `add stats`. Console layouts, packing, module mains/substats and specialization are outside the model; do not infer or add them automatically. See the [data policy](../../STORED_DATA_POLICY.md).

`cartridge count=2/4` activates the corresponding set effects, including static stat bonuses. Do not also include those set bonuses in authored additions. Dynamic effects remain owned by their [set mechanic](../sets/README.md).

## Internal static-build transport

`build-stats` and equipped research adapters use a strict request with `schema_version`, `content_id`, `character_id`, `weapon_id`, `weapon_tier`, `set_id`, `set_count` (0..4) and optional `additive_attributes` of exact rational receipts. Character/weapon level and breakthrough may be declared explicitly; supported defaults are level 80, breakthrough 6.

The compiler combines typed character constants, weapon constants, explicit additions and reached 2-/4-piece set packs. Shared [damage/stat zones](damage-and-stat-zones.md) own arithmetic: weapon ATK is base ATK, console flat ATK is additive, percentages sum within their zones. For example, base ATK 1094 plus authored ATK +50% and flat ATK +100 gives 1741, not a new base ATK.

The twelve [typed set definitions](../../../crates/data/src/set_definitions.rs)
preserve 2-/4-piece thresholds, static pack references, null buffs and the exact
buff paths checked by owner projectors. English titles/descriptions serve
Studio aliases and tooltips through its catalog export. Source localization
metadata, game icons, geometry and relive flags stay cache-only; they do not
participate in build compilation. Definition evidence uses
`table:typed_set_definitions`; explicit test/research overlays remain separate.
Diabolos and Street Boxer four-piece effects use their owner projectors and
runtime handlers. Street Boxer's unconditional `Lakshana_4_1` crit fraction is
also compiled once into static `CritBase` at four pieces, with its exact typed
parameter evidence; its runtime reaction window applies only the difference
between the selected replacement and that already compiled baseline. Raw
legacy requests without a compiled baseline retain their original runtime
addition. Unused source asset paths remain omitted.

Static set packs come from [consumed typed Rust modifiers](../../../crates/data/src/set_modifiers.rs).
Exact hundredths preserve fractional bonuses without binary-float conversion.
Only packs referenced by supported sets are retained; unused rookie packs and
the original `DT_EquipmentModifyData` export remain cache-only. Receipts identify
`typed_set_modifiers`; explicit modifier overlays retain separate provenance
and validation, including zero, missing rows and conditional-pack rejection.

Implemented dynamic set parameters come from [typed single-key values](../../../crates/data/src/set_parameters.rs).
Their provenance is `typed_set_parameters`; source `CT_Equipmentbuff` is
cache-only. The common projector preserves exact coordinate-one values while
each owner interprets ratios, stack counts and seconds and enforces its unit
bounds. Explicit overlays do not fill their missing rows from the baseline.

Unknown stats, retired layout/allocation fields, invalid identities/progression and set counts above four diagnose. Authored console additions cannot replace character/weapon base stats. Additive set packs must explicitly have empty conditions and supported operations. A four-piece proc below count four is rejected; an unequipped four-piece effect is not installed.

## Checks and evidence

Rust tests check explicit additions, empty additions, exact arithmetic, thresholds 0..4, unsupported inputs and continuing owner effects. CLI/WASM/Worker checks cover shared results, replay and aggregation. Input hashes and typed/source evidence describe the values consumed; no console geometry or allocation legality is claimed.
