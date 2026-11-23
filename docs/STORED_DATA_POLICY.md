# Stored runtime data

New review summaries are Markdown. Generated machine receipts and captures
belong outside Git; verifier sources and Rust tests are the current
checks. Keep tracked review inputs only when they are authored models, consumed
requests, source evidence needed for development, or independent expected
fixtures. Unused inputs and generated receipts may be removed with their links
and readers migrated or deleted. Under the Rolling Data Policy, verifier
sources and Rust tests provide direct behavioral guarantees without
hash file gatekeeping.

Commit only data consumed by simulation mechanics. Runtime constants belong in typed Rust structs/tables with only required fields, a concrete consumer and independent checks. Unused parameters, whole source exports and speculative catalogs are forbidden. JSON remains request/report/export transport. The [agent guide](../AGENTS.md) and [decision 0009](decisions/0009-anonymous-targets-and-typed-content.md) own this rule.

Targets are anonymous stationary dummies with explicit script-defined stats. Enemy catalogs, native profiles, spawn/level arrays, forms, skills and boss-phase simulation are forbidden. [Repository checks](../tools/repository_data.py) reject enemy catalogs and source-only data directories even when small; .gitignore also excludes those directories.

Raw evidence and source-only projections live outside Git. They support optional research, never ordinary simulation execution. Do not copy them back into runtime catalogs or mechanics-card data dumps. The deleted build-stats.json reference catalog had no application consumer; runtime build/action policies come from Rust.

The retained legacy table adapters are being replaced with consumed typed constants. Preserve exact values, absence/zero distinctions, units, refinement/skill selection and native/WASM/Worker behavior while doing so; moving untrimmed payloads from JSON to Rust is forbidden. Authored scenario requests, actual application UI inputs and compact independent test fixtures are distinct from source dumps.

Character `EffectFigure` source exports are forbidden in the runtime package. Consumed owner parameters belong in Rust; source rows for unimplemented branches remain cache-only. Repository checks cover future owner names as well as the retired baseline exports, and embedded-table checks prevent reintroducing those exports through a frozen adapter.

Ability tooltip exports are cache-only. Typed Rust action titles supply Studio
localization through the catalog export; descriptions, icons, unused passive
labels and unsupported-owner titles do not enter the runtime package. Tooltip
packet rationale remains in owner cards and independent combat tests.

Native action categories use consumed typed action tags. Their transport hashes
cover only the declared tag fields, with `typed:ability_tags` provenance. Absent
rows stay absent; activation assets and relive flags from the retired source
export are not runtime inputs. Explicit test/research tag overlays remain JSON
transport and never silently merge with the baseline.

Gameplay-effect class mappings and unique indices are cache-only source
observations. Damage compilation consumes exact GE identity, action ownership
and numeric fields; it does not load game classes or resolve asset paths.

Typed character actions retain only the 23 supported owners' proactive action
identities, native indices and consumed purchase prerequisites. Their transport
uses `table:typed_character_actions` provenance and hashes only those fields.
Item/currency costs, noncombat groups and unsupported owner rows stay cache-only.
Purchased/effective skill interpretation remains in the shared Rust compiler.

Resonance level additions retain 66 consumed skill identities for 23 owners in
typed Rust. Chiz explicitly declares no level additions; its source buff payload
is omitted. Full awakening exports, descriptions, assets and unrelated effects
stay cache-only; implemented awakening mechanics remain owned by character
modules. `table:typed_resonance_levels` evidence points into the consumed
transport, and explicit owner overlays keep their own schema/field validation.

Weapon level bindings retain only the ten upgrade groups referenced by the 51
supported weapon definitions. Typed coordinate rules preserve their exact
level-to-modifier identities for 1..80. Unreferenced upgrade groups and `NeedExp`
stay cache-only. Stat values, terminal breakthrough addition and refinement
selection keep their existing consumers; experience costs do not enter combat.

Weapon breakthrough bindings retain 51 typed groups and seven shared level caps,
preserving all 357 terminal modifier identities. Rust weapon compilation, static
build stage selection and Studio growth consume only the cap and modifier fields;
currency/material requirements stay in the cache-only original export. Typed
evidence and explicit legacy overlays preserve source and missing-data checks.

Weapon definitions retain 51 typed identities, public titles, rarity, compatibility
groups, progression references/limits and explicit exclusive-owner lists. Rust
constant compilation, equipment legality, passive owners, aliases and Studio
consume these fields. Eleven nonempty exclusive lists and forty empty lists keep
their separate meanings; group compatibility does not infer exclusive semantics.
Game inventory, acquisition, artwork/model paths and untranslated source-only
fields stay cache-only. Owner guards also retain twelve explicit empty exclusive
descriptions and Rose's English cosmetic-skin text, which distinguishes its
cosmetic list from mechanical exclusivity. Native catalog export supplies the UI; definition evidence
uses `typed_weapon_definitions`, while explicit legacy overlays validate separately.

Weapon static modifiers retain only the 800 referenced level packs and 357
terminal packs. Typed integer attack arrays and exact decimal secondary channels
preserve their native units, including BI points and explicit zero attack. The
800 unreferenced level packs and every source-only `SortKey` are omitted. The
shared additive compiler still validates operation/condition metadata in explicit
overlays; typed evidence identifies the consumed values, not the cached export.

Weapon refinement curves retain 156 typed Constant curves and their 618 original
sparse keys: 154 listed parameter curves and two consumed Nest Bird mechanic
curves. Signed exact decimals preserve the negative reduction curve for its
owner's explicit magnitude conversion. Fifty-one unused curves, default-value
sentinels and unused extrapolation fields stay cache-only. Nest Bird's observed
but unapplied CD2 curve is absent from runtime data and reports; it does not
establish a cooldown. Typed curve evidence and explicit overlays keep provenance,
key indices, Constant left-key selection and invalid-input diagnostics.

Weapon refinement definitions retain 51 typed groups and 255 supported rows:
154 ordered parameter names/percent hints, English descriptions and the native
description keys checked by owners. Shared descriptions are stored once per
group; Nest Bird's five distinct descriptions/keys preserve their exact ranks.
Unused buff-class paths/flags, currency costs, titles, source-language echoes and
localization tables stay cache-only. Reports expose the selected row, consumed
parameters and provenance; raw buff-class observations are removed. The static
build report keeps an explicit unvalidated-passive flag, including when the
listed parameter array is empty. Actual effects remain owned by Rust adapters.

Native damage definitions retain 378 packet rows for 23 supported owners:
exact ATK/DEF/HP arrays, ChargeAdd, UnbalValue, FixedCritRate, GAName and native
damage type/source category. Integer millionths preserve all source digits;
empty arrays and explicit zeros remain distinct. Common packet entry serves
the seven reached Support rows and eight explicit Jiuyuan callback exclusions
as well as packets named by owner dispatch. The other 575 source rows stay
cache-only, including three Silent Garden names whose live implementation
uses weapon curves and synthesized metadata rather than damage rows.
Destruction/impulse fields and HeterochromeAdd/StroyBlanceGERate have no live
consumer and are omitted. The original export remains available to independent
source checks. Baseline transport is `typed_native_damage`; explicit legacy
overlays retain missing-row/field validation and never merge defaults.
The portable package contains no JSON runtime constant tables; engine identity
binds typed constants while package hashes bind retained metadata/presets.

Static set bonuses retain only referenced pack IDs, consumed attributes and
exact values in Rust. Their JSON transport declares unconditional addition;
unused rookie packs and source-only sort metadata are omitted. Set counts still
gate the original 2-/4-piece bonuses, and authors must not repeat them in stats.

Set definitions retain only supported IDs, English UI titles/descriptions,
thresholds, modifier pack IDs and owner-checked buff paths in typed Rust.
`typed_set_definitions` transport omits localization keys/source strings, game
icons, geometry maps, relive flags and redundant class names. Null buff paths
remain null, including the static four-piece shield/healing sets. Explicit
overlays replace the baseline and retain dependency checks.
Diabolos and Street Boxer retain an explicit unbound buff marker; no runtime
consumer checks their source asset paths, so those paths stay cache-only.

Implemented four-piece set effects use typed single-key parameters. Retain only
values consumed by owner projectors; their units and triggers remain owner
contracts. Unused curve rows and default/infinity metadata stay cache-only.
Explicit parameter overlays replace the baseline and keep coordinate, mode,
value and missing-row validation.

The generated `mechanics.json` source-candidate inventory is cache-only. It is
neither runtime metadata nor executable coverage. Audits retain explicit false
package readiness and source integrity checks for retained inputs; `gaps`
reports package scope declarations, while simulation reports own reached
assumptions and diagnostics.

Console/module geometry, packing, automatic console stats and specialization bonuses are out of scope. Authors supply console stats through `add stats`; only declared 2-/4-piece set effects apply automatically, including their static bonuses. Do not retain or port console-geometry or specialization catalogs to Rust.
