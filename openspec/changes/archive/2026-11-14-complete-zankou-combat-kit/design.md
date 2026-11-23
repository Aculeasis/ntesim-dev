# Design

## Context

See [proposal.md](proposal.md) for scope. Planning inspection was read-only against the `main` baseline containing the completed Team list work.

Observed boundaries:

- `crates/content/src/characters/zankou/skill.rs` expands a Reality Normal chain and routes every admitted ordinary Skill through the same two-packet Heartwrench producer. `state.rs` has a small Normal/Hunt state, without an executable Illusion lifecycle. `burst.rs` consumes stored Kindling and applies Vile Ash without selecting the described Ultimate forms.
- `parameters.rs` fixes Skill recovery to 800000 microseconds and readiness to 12120000 microseconds. Neither those constants nor the historical montage tables establish a complete form-specific action clock. `ZankouKit` already owns cast/hit hooks, damage modifiers and exact packet Cycle grants; extend that owner rather than inventing a second simulator.
- `vile_ash.rs`, `dot_kind.rs`, `spread.rs`, `scorch.rs` and the existing Zankou tests provide separate source/target DoT trains, native coefficients and nonrecursive Scorch. Reuse the meaningful independent tests while correcting producers and form rules. Current source Kindling is unbounded, and the DoT critical bonus has only the permanent A1 binding.
- `openspec/specs/characters/zankou/spec.md` hardcodes one Skill, a paid Force Burst, and broad awakening statements. These are selected historical contracts rather than evidence of all current gameplay paths. The delta replaces the changed blocks and preserves unaffected scenarios.
- Sakiri's `skill.rs` gates the currently located flat-ATK producer on `has_effect(4)`. It computes `0.30 * static_stats.base_atk`; `callback.rs` applies it to other actors for twenty seconds. `sakiri_actions.rs` already independently checks source base 1000 giving +300 and excludes self. This establishes a bounded working A4 case, not complete correctness of the ordinary Burst buff, real activation timing or a continuing team rotation.
- The selected preset currently uses Sakiri `[6]`, skill-before-burst Iroi, and a timed loop of ordinary Zankou Burst/Skill/Normal. Daffodill remains `[3,4,5]`. Her existing owner supports two Phantom Step charges from a paid Burst and consumes them on legal ordinary returns; no Critical Dodge is necessary for that route.

References read for this plan:

- [Zankou mechanics dossier](../../../../docs/mechanics/characters/zankou/action-damage.md) and [Vile Ash coefficient card](../../../../docs/mechanics/characters/zankou/vile-ash-dot-coefficient.md): native names, values, recovered cue/form grants and unresolved executor policies. Their historical sections contain contradictory awakening labels and timing summaries; retain useful source findings and replace obsolete claims during implementation.
- [Sakiri dossier](../../../../docs/mechanics/characters/sakiri/action-damage.md) and [Daffodill dossier](../../../../docs/mechanics/characters/daffodil/action-damage.md): current recipient, control and charged-entry models.
- [Zankou guide](https://neverness.app) and [Sakiri guide](https://neverness.app), read November 13, 2026: published combat intent and the user-requested rotation. They do not establish microsecond clocks or replace the frozen Global inputs. The guide's short Skill interval and self-CI wording differ from current selected repository policies; do not silently choose whichever produces more damage.

## Goals / Non-Goals

**Goals:** A single Rust owner that admits ordinary source inputs, derives actual forms and earned enhancements, preserves real time and continuing resources, and supports every combat branch in the proposal. Independent owner checks must precede the curated-script rewrite. Sakiri's A4 audit separates exact arithmetic, ordinary-Burst interaction and aggregate performance.

**Non-Goals:** A new rotation optimizer, runtime JSON character catalog, generic form framework, UI redesign, changes to other authored team builds, overworld/life skills, enemy AI, geometry or fictional counter successes. Shared engine fixes legitimately invalidate other results without authorizing their authored-script rewrites.

## Decisions

### 1. Resolve concrete evidence gaps at the existing owner boundary

Use current installed-client `DT_SkillDamageData`, `DT_ZankouEffectFigure`, character ability/awakening configuration, montage notifies and the already located native/cue exports, supplemented by current checkable public sources. Follow the accepted [rolling data policy](../../../specs/product-scope/reference.md#rolling-data-policy): historical game-version or patch matching is not an admission gate. Start from `GA_Zankou_Shadow_Skill`, `MS_ZankouShadowSkill`, `ZanKouLogicObj`, `Buff_Zankou_GiveUltraForce`, `Buff_Zankou_GiveUltraMagic`, the negative-priority Force override and `Calc_ZankouDotStackCoef`. Raw inputs and optional receipts stay in `cache/`; do not execute game binaries or acquired scripts. Retain only consumed typed Rust values and durable source explanations. Parser layout/type checks establish a valid decode; source hashes identify bytes and do not impose a product version lock.

The initial binding work has specific outputs, rather than a generic instruction to research the kit:

| Existing gap | Required binding decision and verification |
| --- | --- |
| Ordinary/enhanced Skill routing | Native class/packet mapping, valid preceding Normal/held/counter inputs, invalidation, Illusion duration, late Soulcross window, transition commit and cancel deadlines; explain the old twelve-second model versus the guide's short window. |
| Shared Ultimate input | Priority of ordinary, Bloodfeast and free-finisher grants, actual energy and cooldown checks, cleanup/expiry, A4 unlock and per-class Kindling capture/consumption. Do not identify Force/Magic by a packet-name guess. |
| DoT producers and composition | Which delivered contacts apply/spread each carrier; class-specific stack coefficients, expansion bypass, refresh and tick phase, recipients and stale-generation cancellation. A five-stack Burst application cannot be copied to every variant by default. |
| Passive/awakening scope | A0/A1 Hunt/Delusion recipients and cleanup, A3 heal recipient/Max-HP basis and resistance, A5 eligibility/once ledger, three-/six-node resonance and defeat enhancement. Audit the CI recipient conflict against primary evidence. |
| Sakiri Burst/A4 | Native ordinary-Burst carrier, A4 additive versus replacement relationship, source base-ATK projection and the activation/lifetime/recipient path. |

Keep the accepted Crimson Reverie initialization and CI-aura contract until primary evidence establishes a correction; the published self-only wording is a recorded conflict, not authority to change shared team stats opportunistically. If the audit proves a recipient or other accepted-rule error, revise its complete owning delta and independent numeric expectations before implementation. Exact unresolved native playback may use an explicit bounded model supported by located inputs and independent checks; never label it measured/native-verified. An unresolved reached damage or permission dependency remains a blocking diagnostic and cannot count as completed combat support.

The current player configuration plus Passive2 carrier now establish the self recipient, consistent with the published description. The Zankou delta replaces the complete Crimson Reverie requirement and both entry/return scenarios before runtime changes. Zankou keeps CI+100; teammates receive none. The independent off-field Blackbird Nova expectation changes from 60600 to 53100 with Twinflight T3's own CI108. Current owner/carrier references and selected finite-clock, grant/capture and DoT-composition models are in the mechanics dossier. The DoT consumer is additive CoefModify on ListenBattle; both classes retain NeverReset phase and half-open expiry. These explicit bounded models replace the conflicting adapter shortcuts, without claiming recovered native executor bodies.

The November 14 apply audit recovered the four player Skill montage/packet mappings and the nominal five-second Skill row; see the [saved source findings](../../../../docs/mechanics/characters/zankou/action-damage.md#frozen-player-skill-audit-november-14-2026). It also found a scoped decoder error in the older player's `ZanKouLogicObj`. The user selected the current installed client and rolling evidence explicitly. Repair the current logic decode with current native metadata and use supported current findings to supersede older mechanics. Do not require the old executable/asset identity, and do not treat recovered direct player packets as a complete Skill including its shadow. Unresolved mechanics remain research questions; a mere version difference does not block them.

Alternative rejected: translate the guide literally into fixed waits and packet overrides. That would hide the missing owner state and would make future callers reproduce the same mistake.

### 2. Keep form, action progression and earned permissions separate

Extend the existing Zankou state with explicit Reality/Illusion, real form deadline/generation, per-form Normal progress, enhanced-entry prerequisite, late Soulcross permission, Bloodfeast unlock, free-finisher deadline/consumption and the bounded stored/captured Kindling lifecycle. Keep per-target A5 consumption separate from actor-wide form state. Use shared owner clock, event queue, cancellation and resource infrastructure, following the established Daffodill/Iroi/Nanally integration patterns only where applicable.

Ordinary `attack`, held input, `skill`, `burst`, legal player inputs and actual switches remain the authoring surface. Extend `ZankouKit` admission/selection/readiness/clock hooks and only the required existing dispatch points. No source option selects `enhanced=true`, injects a counter result or supplies a desired packet. Transitions commit at their sourced event, not at script parsing or a guessed animation end. Interruptions remove unborn events while preserving launched packets and already committed costs.

Montage progress and contact pause remain distinct from real cooldowns, form timers, DoT ticks and free-cast windows. Deduplicate each causal packet's contact pause and resource channels across AoE recipients. At equal timestamps use the existing event priorities and explicit half-open permission deadlines; verify the last legal microsecond and exact expiry. Field exit cancels Illusion, clears its pending opportunities according to source, and preserves independently living DoTs and real cooldowns.

Alternative rejected: add scattered booleans to the current fixed two-packet Skill or use a single timer for form, cooldown and montage. Both lose causality under cancellation and switching.

### 3. Route damage and buffs through real producers

Reuse the independent Heartwrench/Vile Ash trains with source, target, initiating command, metadata and causal generation. Correct application/spreading at each producer; preserve the ten-stack limits and sourced periodic rules. Mark actual Illusion attacks, Soulcross and Bloodfeast as Follow-up without changing Incantation scaling or converting every DoT/Reality packet. Bind live Hunt/Delusion modifiers, A1 persistence/spreading, A2/A4 shared enhancement, A3 healing/resistance, A5 Break and resonance at their actual recipient/event boundary.

Extend current `zankou_actions`, `zankou_skill`, `zankou_skill_aoe`, `zankou_dot`, `zankou_scorch`, `zankou_cycle` and `zankou_passives` checks where they own the behavior. Independent expected coefficients, stack counts, costs and timestamps must not be calculated by calling the production helper under test. Preserve positive-HP resource accounting, ordinary/Cycle separation, no AoE overgrant and Scorch non-recursion.

Three selected nodes are resonance, not the A3 node; six selected nodes are resonance, not A6 alone. Keep explicit awakening subsets throughout projection, inspection and tests. Replace obsolete test expectations rather than admitting old inputs through compatibility shims.

A6 obtains owner-managed foreign DoT stack counts through the existing `CharacterKit` registry interface. Lacrimosa supplies her live Nightmare decay count and Adler supplies his unexpired per-target carrier; the shared hit pipeline sums living owners without importing their implementations. BuffManager continues to supply the ordinary tagged DoT carriers, so each target's total is capped only at A6's final consumer.

### 4. Make readiness usable without script-owned mechanics

Bind explicit live names for actual form, remaining Illusion time, enhanced-entry/Soulcross/Bloodfeast/free-finisher availability, and relevant modifier time using the existing query infrastructure in `characters/queries.rs`. Preserve current public names where their meaning remains valid. `.skill.ready` and `.burst.ready` must use exactly the owner selector and cost/cooldown gates used by admission. Queries cannot mutate state or peek at RNG. Finalize the small vocabulary in the DSL/mechanics reference together with its code and source tests; no new general DSL syntax is needed.

Native scenario/source replay, inspection, WASM trials and actual Worker must retain input/occurrence identity, selected packets, metadata provenance, form transitions and resource receipts. Any transport adjustment belongs at the existing shared boundary. TypeScript only renders/validates structured observations; it does not choose a form or compute a buff.

### 5. Diagnose Sakiri before measuring a rotation

Extend the existing numeric A4 test boundary with identical `[4]` versus no-A4 selections, separating the source's own ordinary-Burst buff. Use exact source base 1000 to check +300, a weapon-base change to check the projected source basis, and console/recipient stat changes to reject the wrong basis. Check self exclusion, living allies on/off field, failed admission, source/recipient defeat, refresh without stacking and exact twenty-second expiry. Test a controlled ATK-scaling ally hit inside/outside the window, plus a non-ATK damage channel that must not gain a fictitious multiplier.

Use real source executions and managed-modifier observations for timing/continuing state. Use `[]` versus `[4]` to isolate A4, then `[6]` versus `[4]` to measure the actual preset substitution without confusing the lost A6 contribution with A4's own value. If an ordinary Burst contribution is absent or confused with A4, fix its real producer and extend the same owner tests. A4 does not mean +30% team DPS: its observed value depends on base ATK, other buffs, coverage and the share of affected damage. Document a correct small result with causal receipts instead of changing a scalar to obtain the expected ranking.

### 6. Rewrite only the selected source after owner verification

Keep `zankou-insight` and its label, roster, equipment, console additions, seed 42, 1000 iterations, 120 real seconds and common dummy. Preserve Zankou A0 and Daffodill `[3,4,5]`. Replace Sakiri's one selected node `[6]` with `[4]`; do not add a second node or change Sakiri in other presets.

The authored sequence is:

1. Start on Zankou; spend her real entry Cycle by switching to Daffodill for Scorch. Use available Daffodill Burst/Skill, Sakiri Burst then Skill, Iroi Burst then Skill. Use Daffodill's actually charged return for Phantom Step before Zankou's damage window. Respect available energy and native swap/cancel gates.
2. Use the required ordinary Normal setup if needed for a legal held attack, then enhanced Sanguine Dash. Attack in Illusion until the actual enhanced-Soulcross window; do not switch out or hardcode five seconds in place of readiness.
3. Complete enhanced Soulcross, then Bloodfeast Reverie and the earned free Inferno Flamenco at their earliest legal input boundaries. Start another held-entry sequence, use actual applications/spreading to build both DoTs, and take support handoffs only at safe form boundaries. A full stacking window must demonstrate the second DoT reaching ten stacks through lawful producers.
4. Use Daffodill's remaining real charge and the supported equipped Black Tome Arc when available; retain existing equipment rather than importing the guide's different Arc. Reapply Iroi/Sakiri buffs where needed and fund the next paid Ultimates with earned resources. Near the horizon use a bounded tail and explicit wait without admitting a doomed full window.

The guide supplies ordering intent, not permission to skip recoveries or assume every support Burst is ready each loop. Retain before/after exact-source comparisons under the same corrected engine and common seeds. Also isolate Sakiri no-A4 versus `[4]` on the final rotation. Keep direct grant, buff coverage and damage-category receipts with the aggregate comparison. No required percentage uplift is invented.

Alternative rejected: rewrite every Zankou preset or change their builds to recover old scores. All other authored preset bytes remain unchanged; the shared engine may legitimately change their damage. Update only test fixtures/consumers actually incompatible with corrected admission, without disguising a failed existing catalog run.

### 7. Deliver through the existing verification and generation pipeline

Run focused Rust owner/source checks first, then the corrected source to a complete diagnostic horizon and its 1000-seed ensemble. Rebuild native/WASM after owner changes. Regenerate `script-presets.json` and Team list via `python tools/check.py -- node tools/web.mjs teams`, then `teams-check`; all entries whose engine identity changed need current computations, not copied DPS. Verify source hashes for every other preset and the unchanged S5 control.

Use existing source/native replay, native/WASM parity and actual Worker smoke for the new derived variants and continuing windows. Extend the existing all-preset replay/catalog tests only for distinct transport/provenance/node-selection risks. Use actual Studio Sample for the corrected source, seeds 42 and 43 and scrubbing to the horizon, with a bounded browser check of exact source loading and console/page errors. Mechanics stay in Rust, not copied into UI test suites.

Update current mechanics and DSL references in place, removing contradictory obsolete descriptions and repairing their consumers. Do not edit historical archives to pretend they predicted the new model. Validate this change, review the stable diff, run `python tools/checkpoint.py run`, then synchronize/archive and refresh the final checkpoint before the coherent commit when implementation is authorized.

## Risks / Trade-offs

- Hidden native grant/cancel/DoT policies → resolve the named exports and class selectors first; retain explicit bounded assumptions and diagnose unsupported reached dependencies. Full combat completion cannot be inferred from one successful opener.
- Fixed-guide intervals differ from the frozen runtime → derive readiness from actual owner state and justify chosen timings in the mechanics reference; do not tune cooldowns for DPS.
- Current CI-aura scope differs from published wording → preserve accepted behavior pending primary evidence and revise its owning delta before any proven correction. Include the resulting affected-score invalidation in review.
- Existing Zankou research sources receive different semantics → update compatible regression expectations, preserve other authored team sources, and surface any remaining incompatible catalog entry instead of silently replacing it or keeping its old score.
- Small Sakiri A4 aggregate uplift → compare the exact isolated ATK grant and actual uptime first; total-DPS magnitude is not an acceptance threshold.
- More form events and DoT spread may increase runtime → reuse generation invalidation, exact event scheduling and causal deduplication; verify the required full ensemble under the existing four-CPU bound without adding a performance campaign.

## Migration Plan

Implement and verify the complete owner and Sakiri audit before changing the selected rotation. Migrate current contracts, fixtures and consumers together with no obsolete-input shim. Regenerate projections/results only after the owner state stabilizes, then finish transport/browser validation and the final checkpoint. A rollback restores the coherent commit and regenerated matching data together; deployment/publication is outside this change.

## Applied bounded A3/A5 fixture decisions

The recovered A3 dynamic armor coefficient is.5; native armor capacity/regeneration remains unlocated. The existing native interruption fixture gains optional normalized `armor_damage`, accepted only for Zankou: one armor unit per admitted action, exact incoming pressure multiplied by.5 during A3 Hunt (otherwise1), cancellation at accumulated1. Omission remains unconditional cancellation. This is a disclosed finite fixture model, without incoming-enemy inference or general DSL syntax. The new consumed `zankou.combat_passives_v1` description records that limit. A3 healing uses shared live-MaxHP/healing settlement on an armed real-second phase.

A5 uses its sourced657891 progress impulse and999999 gauge payload, once per nonsummoned living authored target per combat. Existing gauge and Break settlement determine the result. Native `zankou.story_mode_v1` explicitly selects story-mode boss exclusion; omitted mode models ordinary stationary dummies. The existing combat-transition fixture resets the actual per-target ledger on enter/leave. Team Break+300% is a living-source managed modifier.

## Applied Sakiri Burst finding

Current warning-free GA_Sagiri_UltraSkill graph has independent requests at446 (Level4_1, guarded by node4) and826 (ordinary QDamUp, unguarded). Both current calculators read GetAtkBase and independent.30/20s rows. The old A4-only ordinary producer was a demonstrated defect. Each admitted activation now requests the ordinary30% carrier and, with A4, a separate additional30% carrier; each self-excludes and refreshes cap1. Shared base-ATK projection already includes weapon base and excludes console additions, so no shared stat-formula change is needed. The existing snapshot/living-recipient policy remains explicit under sakiri.burst_atk_v1; source defeat preserves an earned snapshot, dead allies receive no new grant, and queries sample living carriers. The mechanics dossier records exact graph offsets, source rows and independent arithmetic.
