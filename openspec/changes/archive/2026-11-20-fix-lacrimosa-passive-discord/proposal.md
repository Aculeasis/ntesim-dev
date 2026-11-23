# Proposal

## Why

Lacrimosa's Breakthrough 2 passive (`GA_lacrimosa_Passive_1` / Tomato Banquet) is currently implemented in `crates/content/src/characters/lacrimosa/skill.rs` to trigger a 400% ATK packet (`GE_Player_Lacrimosa_AnHunZhouTwo_Damage`) on every periodic Nightmare DoT tick whenever a target is Broken, erroneously treating the target's Broken state as a Discord reaction. This awards massive phantom damage in non-Discord teams (such as `Blackbird · Stain / Hathor`, where ~685k out of 1.22M damage came from this false trigger), severely distorting character DPS comparisons. Furthermore, `GA_lacrimosa_Passive_1` has no localized display name in the catalog, rendering as raw engine IDs (`GA_lacrimosa_Passive_1 · GE_Player_Lacrimosa_AnHunZhouTwo_Damage`) in Studio charts and tooltips.

## What Changes

- **Fix Passive Trigger Condition**: Update `GA_lacrimosa_Passive_1` to trigger exclusively when a Discord reaction occurs on an already Broken target (via the `on_discord` hook), and remove the erroneous trigger inside `on_hit` for periodic Nightmare DoT ticks during Break.
- **Add Human-Readable Ability Title**: Register `GA_lacrimosa_Passive_1` as `"Tomato Banquet"` in `crates/data/src/ability_titles.rs` and update generated name catalogs so Studio displays a clear, polished skill name.
- **Update Specifications and Tests**: Update `characters/lacrimosa` specification to reflect the true Discord reaction requirement on Broken targets, and align existing simulator tests and preset receipts with mechanics reality.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `characters/lacrimosa`: Update Requirement "Discord Reaction Interaction" to specify that Tomato Banquet (`AnHunZhouTwo_Damage`) triggers only upon Discord reaction execution against a Broken target, rather than on periodic DoT ticks during Break.

## Impact

- `crates/content/src/characters/lacrimosa/skill.rs`: Remove erroneous DoT `is_broken` trigger and ensure `on_discord` verifies the Broken target condition before scheduling `AnHunZhouTwo_Damage`.
- `crates/content/src/simulation/tests/`: Update `lacrimosa_public.rs` and related fixtures to verify the corrected trigger semantics.
- `crates/data/src/ability_titles.rs` & `web/src/generated/names.json`: Add `"Tomato Banquet"` for `GA_lacrimosa_Passive_1`.
- Studio results: Fix charts and tooltips showing raw `GA_lacrimosa_Passive_1 · GE_Player_Lacrimosa_AnHunZhouTwo_Damage`, and correct DPS rankings in presets such as `blackbird-hathor.nte`.
