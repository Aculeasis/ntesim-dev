# Proposal

## Why

Zankou's executable adapter selects one ordinary Skill and one paid Burst instead of her Reality/Illusion combat kit, so the bundled Insight team cannot perform its intended long rotation. Complete her combat behavior before rewriting that preset, and verify Sakiri's A4 contribution rather than interpreting a small team-DPS difference as proof of a defect.

## What Changes

- Complete Zankou's combat inputs against the existing stationary dummies: both Normal chains, held attacks, supported dodge counter and plunge, Support, ordinary/enhanced Sanguine Dash and Soulcross, both Ultimates and the earned free finisher. Bind forms, prerequisite windows, actual clocks, cancellation, resource costs, damage categories and continuing state in Rust.
- Bind Hunt/Delusion, both independently stacking DoTs and spreading, both passives, selected awakening effects A1-A6 and the three-/six-node resonance bonuses. Reconcile contradictory historical mechanics descriptions with current checkable evidence, including Crimson Reverie's CI recipients, Kindling's one-stack limit and the class-specific A4 sharing rules.
- Follow the accepted rolling data policy: use the installed client and current public sources without requiring a historical game build or patch match. Preserve source references, material conflicts and explicit model assumptions; game-version certification and multi-version support are outside this change.
- **BREAKING**: replace the fixed Skill/Burst contracts and obsolete executable shortcuts with current state-derived selection. Source commands remain ordinary player inputs; callers cannot select a desired enhanced packet or invent a form, charge, counter or timer. Update current consumers and independent tests together.
- Audit Sakiri's A4 flat-ATK grant, its relationship to the unawakened Burst buff, recipients, source base ATK, activation time and refresh/expiry. Fix a demonstrated discrepancy at its owner; preserve correct behavior if the small DPS effect is explained by actual uptime or damage composition.
- After the owner checks pass, rewrite only `presets/studio/zankou-insight.nte` as a curated rotation: support setup, held attack into enhanced entry, uninterrupted Illusion attacks, enhanced Soulcross, Bloodfeast Reverie immediately followed by the free Inferno Flamenco, then lawful repeat/support windows and a complete tail. Preserve Daffodill `[3,4,5]`; replace Sakiri `[6]` with `[4]`, retaining one selected node.
- Preserve the preset identity, four-character roster, equipment and comparison budget unless the user later requests another build. Use existing Daffodill charged-entry Phantom Steps and equipped Arc where lawful; do not import a different recommended weapon or fabricate incoming attacks.
- Rebuild native/WASM, regenerate the source/catalog projections and every invalidated score, and verify the exact corrected source through native replay, Worker and Studio Sample. Other presets keep their authored bytes; shared engine corrections may change their computed results.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `characters/zankou`: Complete combat input selection, forms, clocks, DoTs, passives, awakenings and resonance, replacing the fixed Skill/Burst model.
- `runtime-content`: Source-owned Sakiri A4 ATK contribution and independent verification against the ordinary Burst buff.
- `studio-application`: The corrected Insight preset, exact awakening selections and lawful complete continuing rotation under the existing curated profile.

## Impact

- Zankou's existing Rust owner, typed native packet data, admission/clock and callback bindings, live queries, metadata/provenance and existing Zankou mechanics tests. Reuse shared scheduling, resource and input infrastructure.
- Sakiri's owner/build projection and existing `sakiri_actions`/managed-modifier checks, only where the audit demonstrates a defect.
- The selected Insight `.nte`, its rationale in `presets/studio/teams.md`, generated source/catalog files and affected catalog/replay checks. Keep all other authored presets and S5 unchanged; minimally update regression consumers whose old expectations conflict with corrected mechanics.
- The Zankou capability, current DSL/mechanics references and existing native/WASM/Worker verification boundaries. Raw acquisitions, independent calculations and comparisons remain under `cache/`.

Full combat scope excludes overworld invisibility, city/life skills, enemy AI, geometry and console packing. Missing native details remain explicit research questions with concrete resolution tasks; they cannot be hidden by declaring only the A0 opener complete. This proposal authorizes planning only; implementation starts with a later apply request.
