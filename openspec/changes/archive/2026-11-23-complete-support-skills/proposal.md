# Proposal

## Why

Support Skills (QTEs) are only partly implemented. Sakiri, Nanally and Zero can cast on ordinary switches and have an artificial 15-second QTE cooldown. Haniel, Hathor and Edgar can attack automatically on ordinary entry, and Daffodil's QTE can suppress her earned Phantom Step. Scripts currently have no independent purchased QTE level.

## What Changes

- Delete the legacy automatic Support dispatcher and implement Sakiri, Nanally and Zero in the same reaction-qualified flow as the rest of the supported roster. An eligible switch reaction enables one incoming QTE with no QTE cooldown.
- Implement Lacrimosa A6's earned reaction entry after Devilish Gift with its own readiness rules and zero outgoing Cycle cost.
- Remove automatic ordinary attacks from field entry. Keep distinct, earned owner mechanics and check how they compose with QTE. Daffodil automatically performs Phantom Step after the full QTE animation when both are available.
- Complete every supported character's QTE with its own timing, packets, healing, recipients, resource effects and progression rules.
- **BREAKING** Require an explicit purchased `qte=<level>` on every script character declaration. Apply any owner-specific Resonance 3 upgrade during effective-level projection. Missing or invalid values produce a source diagnostic.
- Set `qte=8` for every character in the Team list scripts, including S5. Migrate the other maintained scripts, affected fixtures and documentation, and verify that bundled rotations execute under the corrected combat rules.

## Capabilities

### New Capabilities

### Modified Capabilities

- **combat-execution** gains reaction-qualified Support dispatch and ordered automatic entry actions.
- **script-execution** gains required authored QTE investment.
- **script-inspection** exposes explicit QTE investment and diagnoses omissions on the character declaration.
- **studio-application** requires explicit level-eight QTE investment across the curated collection and its control.

## Impact

Native combat, owner kits, progression and script configuration change together. The native CLI, WASM/Worker, Studio character cards, script exports, report consumers and bundled presets use the same updated contract.
