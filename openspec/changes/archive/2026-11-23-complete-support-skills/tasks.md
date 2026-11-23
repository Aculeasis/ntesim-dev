# Tasks

## 1. Explicit QTE progression and source contract

- [x] 1.1 Add an independent purchased QTE field to native actor configuration and bind it to the Support progression slot. Extend existing Rust projection coverage to verify a level different from Skill, legal purchase limits, the effective cap, and a Resonance 3 owner that upgrades QTE alongside one that leaves it at the purchased level.
- [x] 1.2 Require `qte=<level>` in script declarations and carry it through export, script inspection and the native/WASM/Worker and Studio contracts. Verify a nonmatching Skill and QTE level survives the compiled request and inspection. Check missing, duplicate and invalid fields for source diagnostics and confirm inspection returns no partial party. Update the grammar, field reference and maintained examples in `docs/NTE_DSL.md`.

## 2. Reaction admission and automatic switch actions

- [x] 2.1 Use a single switch-reaction qualification for actual switches and read-only Support readiness. Delete the legacy `characters/support_dispatch.rs` fallback and its cooldown models. Implement fresh Sakiri, Nanally and Zero owner bindings in the common path. Replace cooldown-based fixtures with two funded qualified entries into each owner less than 15 seconds apart, and verify each pays Cycle once, uses the actual target and admits one QTE. Cover ordinary, same-character and input-driven switches through existing switch and readiness cases. Update the shared actor lifecycle and Cycle mechanics references.
- [x] 2.2 Remove automatic ordinary entry attacks from Haniel, Hathor and Edgar and audit the remaining owner entry selectors for the same bug. Verify ordinary switches into affected owners create no attack, attack-triggered effects or attack recovery. Retain the sourced conditions for earned entry actions and update affected owner documentation.
- [x] 2.3 Schedule Daffodil's regular Phantom Step at the QTE animation end using her owner clock. Update existing Daffodil switch and parry coverage for QTE alone, Step alone, both on one swap, recovery at the handoff, one Step charge, and cancellation or stale follow-up. Verify the full QTE runs before Step, including its reached hit pause. Document the observed order and the numeric timing basis in Daffodil's mechanics reference.

## 3. Complete Support effects across the supported roster

- [x] 3.1 Reconcile the registered playable roster against the owner dossiers and typed Support rows. Bind missing QTEs and complete partial ones with the owner's timing, packet sequence, damage and resource settlement, recipients, conditions and follow-ups. Verify every registered owner reaches its intended Support through a legal reaction switch. Use existing owner tests for distinct packet, timing and resource risks and update each affected dossier with the consumed values and modeled limits.
- [x] 3.2 Complete and verify QTE healing and secondary effects using the existing native heal, resource and observer paths. Check Iroi's separate damage and healing delivery with unequal purchased Skill and QTE levels, Nanally's surviving Cat using captured Burst talent, and reached owner/equipment cast observers and Hotori replay retaining one causal cast with no duplicate Cycle spend. Record selectors and timing in the relevant owner and equipment references.
- [x] 3.3 Bind Lacrimosa A6's Devilish Gift reaction window to free Nova and Scorch entry, including qualifying Discord, and document its source and clock. Verify the zero-Cycle route, consumption, expiry, and that the normal paid path can still admit her QTE outside that window.

## 4. Maintained scripts and Team list

- [x] 4.1 Set `qte=8` in every Team list actor declaration, including S5, and migrate other maintained scripts, inline examples and test fixtures to explicit legal values. Update the preset build profile documentation and verify exact source loading with the existing preset generator and `python tools/check.py -- node tools/web.mjs teams-check`.
- [x] 4.2 Execute every selectable script through its authored stopping condition with the current engine using bounded trials. Repair affected waits and reaction handoffs from actual recovery and resource state. Verify successful runs finish without admission or unsupported-effect diagnostics and report the old calculation identity for any retained last-calculated catalog DPS.

## 5. Integration and checkpoint

- [x] 5.1 Run the affected native owner and script tests, then verify representative QTE casts, missing-level diagnostics and automatic entry ordering through the current WASM/Worker and replay boundaries. Use the repository's source, report and transport checks for the changed surfaces.
- [x] 5.2 Review the stable diff and run `python tools/openspec.py validate complete-support-skills --strict` and `python tools/checkpoint.py run` before committing. Resolve failures and rerun affected checks.

## Workflow follow-up

- Archive the completed change and synchronize the accepted specifications after implementation checks pass.
