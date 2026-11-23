# Formula report verification, 2026-10-12

Input: user-pasted read-only audit dated 2026-10-12, SHA-256 `be344a55fec4cc7d87a7662ba9d7f731a327693b1f006befb5ff00dd1a509758`.
Scope: the reported paths and their current native/browser consumers, against the frozen Global contracts and saved tables. No game code was executed and no frozen evidence was changed.

## Six reported likely bugs

| Report item | Finding and disposition |
| --- | --- |
| 1. Be Happy stale Max HP | Confirmed. Capture `effective_hp_max` at the credited defeat before queuing healing. Preserve that captured base if buffs change before the callback; bonus sampling and missing-HP cap remain callback-time policies. |
| 2. CLI charge efficiency | Confirmed. Native build compilation now replaces a caller efficiency proc with exact `ChargeGetEfficiencyBase`, removes a unit override, rejects duplicates, and retains the native attribute and replaced values in the receipt. Runtime numeric generation already supports exact efficiency/carry. |
| 3. Edgar HP percent identity | Confirmed report-contract defect; settled HP was already correct. Studio and built scenarios now export the complete static HPMaxUp pool including A5. Runtime A5 provenance validates that pool without adding A5 again. Explicit breakdown receipt fields named `hp_percent_excluding_edgar_a5` remain breakdowns. |
| 4. Zankou packet comparisons | Confirmed identity error, with a more precise consequence than the report claims. `GA_Zankou_UltraSkill` also made the ordinary-Ultimate predicate overly broad: a magic packet under that ability could receive Kindling before A4, and the ordinary predicate could already grant its ratio addition. Both ordinary/force and magic variants now use damage-effect IDs. Magic Kindling requires A4; both variants receive the modeled resonance_3 ratio addition from awakening 3. This does not supply a missing Kindling producer or complete magic-Ultimate execution. |
| 5. Exact vulnerability | Capability difference, not a reached arithmetic bug. [Damage and stat zones](../../docs/mechanics/shared/damage-and-stat-zones.md) §6 explicitly scopes floating vulnerability to a generic harness and describes the exact/content boundary. There is no live vulnerability producer to migrate. Adding an unevidenced zone would not implement a native mechanic. |
| 6. Element/Mental damage bonus pooling | Rejected. [Damage and stat zones](../../docs/mechanics/shared/damage-and-stat-zones.md) §1 explicitly sums general, matching type, matching action and other ordinary bonuses into one zone. Separate stat identities do not imply separate multipliers: 20% general + 30% matching element + 40% skill is 1.90, not 2.184. Existing typed eligibility remains intact. |

## Remaining observations

| Observation | Verification / remaining limit |
| --- | --- |
| DEF-ignore sum, cap and exact rejection | Explicit current model in combat contract §3. Ignore and shred are distinct remaining-defense factors; multi-source native cap/stacking is unresolved. No evidenced replacement policy was found. |
| RES reduction sum versus strongest | Current additive model; no evidence in this report establishes MAX/replacement. Stack multiplication is dormant for the sole current producer's single-stack policy. |
| Zankou 1.50 final addition | Frozen `1036AwakenEffect` Effect2 says Final DMG increases by 150%; the matching curve is `ZankouFinalUltraDamUpLv2=1.50`. This is a +1.50 bonus, hence a 2.50 factor, not +0.50. Effect4 explicitly extends Kindling to Bloodfeast Reverie; resonance_3 explicitly increases both Ultimate variants' ratios by 20%. |
| Skia 0.08 versus 0.04 | Frozen `1008AwakenEffect` Effect3 describes +4% against a locked target and another +4% when only one locked target is on field. The two bonuses are supported, but inspection confirmed a separate selector bug: code counted all targets instead of living targets locked by Skia. The bonus now queries the existing owner-status count; an additional unmarked target no longer removes the second 4%. |
| Linko field 0.30/A6 | Frozen `1072AwakenEffect` resonance_6 explicitly grants all allies +30% Anima/Incantation DMG and extends the field by 5s. Both constant and awakening gate have a native text citation. |
| Nanally unconditional 0.15 | Frozen `1010AwakenEffect` resonance_6 says damage dealt increases by 15%, without a distance condition. `Nanally_LvExtraFarTarget` naming does not override the frozen wording. |
| CRIT Rate cap | Combat contract §3 explicitly clamps ordinary CR to [0,1]; fixed critical rules replace CR. No change. |
| Reset/Extend per-stack value replacement | Existing shared lifecycle policy. No supplied live producer counterexample establishes independent captured values for these policies; `Independent` is the distinct per-instance policy. Native replacement versus retained values remains owner-specific. |
| Fractional carry ordering | Carry is already generated post-efficiency energy, so new generation is scaled and then added to carry. Scaling carry again would repeatedly amplify previously generated energy. Exact grants and carry conservation remain tested model policies. |
| Shared team-weapon scopes | Explicit owner hypotheses, including same-name refresh/replacement. Native multi-wearer stacking is not established by this report; changing all scopes would invent a rule. |
| First-wins efficiency | Public runner validation rejects duplicate proc kinds before runtime construction. The compiler also rejects duplicate manual efficiency procs before replacing one. The reported silent public-path drop is not reproducible. |
| DEF shred / special factor / exact vulnerability | No reached producers; bounded missing capabilities, not confirmed damage errors. Future owner adapters must preserve zones and reject unsupported reached behavior. |
| Hathor GE comparisons | Redundant dead alternatives alongside GA predicates; no reported reachable packet loses its bonus through them. |
| Floating RES reduction | Separate harness interface; native delivery supplies target reduction to the exact path. No demonstrated native arithmetic failure. |
| Two ATK% contribution channels | Both aggregate additively; the report supplies no producer that registers the same contribution twice. No speculative rewrite. |

## Counterexamples and checks

- Be Happy: base HP 1000, static +10%, ten +4% live stacks yields live Max HP 1500. T1 healing is 180, not the stale-static 132. Starting at HP50 settles to HP230; the captured 180 persists if stacks expire before the queued callback. Browser/native checks also include flat HP contributions.
- Charge: Speedy Hedgehog contributes +0.12 to the native 1.00 multiplier. Sakiri Skill's frozen ChargeAdd values 3.902 and 5.398 yield `(3.902 + 5.398) × 1.12 = 10.416`, hence 10 whole energy and exact carry 52/125. A forged manual 1.80 multiplier is replaced, not multiplied again.
- Edgar: base1000, +10% equipment, +20% A5, flat50 gives HP1350 with exported `hp_percent=3/10`; multiplying settled equipment HP by 1.20 would give HP1380. A fractional base1001/+1/3/+1/5/flat1/3 case settles once to HP1535 with the full percent pool 8/15.
- Zankou: table-driven ordinary/force/magic packets, A2/A3/A4 and active/inactive Kindling distinguish the A4 magic gate and the separate ratio/final zones.
- Skia: the native A3 Skill locks target zero; adding an unmarked second target must preserve every exact damage packet and leave that other target untouched.

Passed verification: `python tools/cargo.py test --locked --offline -p ntesim-content --lib`, `python tools/cargo.py clippy --workspace --all-targets --locked --offline -- -D warnings`, native CLI/WASM builds, `python tools/check.py -- python tools/verify_build_cli.py --out target/verification/formula-audit-static-build.json`, `node tools/web.mjs check`, and selected `native_build`, `formula_audit`, Edgar/Zankou/Skia/Be Happy browser suites with `NTESIM_VERIFY_NATIVE=1`. Current browser suites include actual Worker execution and complete native/WASM report comparison. Changed browser tests also pass formatting checks.

The older `tools/verify_game_build.py` oracle still expects an obsolete static-stat schema and its imported research oracle expects three pre-chain hits. It was not weakened or used to certify this change. Current independent numeric assertions live in the Rust regressions and `web/tests/formula_audit.test.mjs`. Suit10 4pc DSL binding is still unsupported; the Be Happy/live-Max-HP combination uses an explicit structured research proc, while the existing Be Happy refinement scripts continue through the actual Worker. Full kits, native multi-source stacking and unimplemented producers remain outside these fixes.
