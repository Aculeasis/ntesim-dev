# Combat formula review

Reviewed October 12, 2026 against current Rust core/content and frozen `global-2026-10-04` tables. Scope: the user's twelve-section formula sheet, stat zones and DEF-ignore. No frozen source or runtime behavior changes. The [shared contract](../../docs/mechanics/shared/damage-and-stat-zones.md) owns arithmetic; this receipt owns evidence and open questions.

## Evidence and applicability

| Evidence | Inspected | Limits |
| --- | --- | --- |
| User's `Pasted text.txt` | All supplied equations/links; SHA256 in local verification receipt | Supplied synthesis, not an independent experiment |
| [Reddit translation](https://www.reddit.com/r/NevernessToEverness/comments/1tlwa1w/translated_info_from_china_on_damage_formulas/) | Article and correction comments via web browsing | Translation of CN research; incorrect reaction elements; Global version unspecified |
| [666bj original](https://shimo.im/docs/16q8xbK17oUOrjq7) | Original and `/read` URLs attempted | Both inaccessible; no direct original verification claimed |
| [Esper Cycle article](https://www.ntebuild.com/news/nte-esper-cycle-damage-explained) | Equations, ownership and eligibility | Secondary account of Raddish's tests; no raw measurements/version binding obtained |
| [Raddish Part 1](https://www.youtube.com/watch?v=B6TrwDEAlSs) | Link resolved from article; opening attempted | Video unavailable through web tool; no transcript/measurement claim |
| [Drive Calculator audit](../../docs/DRIVE_CALCULATOR_RESEARCH.md), saved source audit (historical raw capture retired; see [source policy](../../docs/SOURCES.md)) | Pinned September 27 findings | Primary evidence of that calculator's model, not game proof; GitHub/raw pages inaccessible on this pass |
| [Hotori author analysis](https://www.taptap.cn/moment/801733694359342092) | Author proposes additive ignore stacking | Expressly tentative, not a controlled multi-source test |
| [Equipment](../../docs/mechanics/shared/equipment-builds.md), [owning Rust checks](../../crates/content/src/simulation/tests), [live calculation](../../crates/content/src/simulation/hit/calculation.rs) | Additive native pack interpretation and static/live arithmetic | Current model consistency, not certification of unresolved native consumers |
| Frozen reaction pairs (optional cached source DT_ReactionData), `DT_GameplayAbilityTipsData` (cache-only source) | `/rows/REACTION_RESULT_TYPE_5/CharacterElementType1` PSYCHE and `CharacterElementType2` LAKSHANA; `/rows/GA_BlackBird_Passive_1/AbilityDescription/0/Description/LocalizedString` Stain enhancement | Source pairs and 30% enhancement description do not bind native formula execution |

Search snippets and reposts were discovery leads only. No applicable raw controlled measurements or primary native damage implementation were acquired. Accessible translations/guides corroborate candidates; agreement between copies is not independent confirmation. Website currency does not establish frozen-cutoff applicability. These are retrieval findings, not new immutable raw-page snapshots or replacements for package evidence.

## Pasted-sheet verdict

| Section | Verdict / qualification |
| --- | --- |
| 1, 12: direct product | Consistent for eligible ordinary damage. Stain is separate; generic harness vulnerability is an additional explicit model factor. True has its own zone policy |
| 2: base and ATK | Correct layered model. Sum percentages once; flat ATK is outside percentage scaling. Current direct APIs have narrower single-stat representations |
| 3: damage bonuses | Correct additive zone after eligibility/stack resolution. Dealt reduction is a signed contribution, not DEF mitigation |
| 4: crit | Correct raw expectation with additional CD and packet-specific probability. Fixed50 replaces CR; raw expectation differs from rounded/sample results |
| 5: DEF | General layered equation and unbuffed-enemy simplification agree. Core implements the simplified nonnegative target model. Level-derived enemy DEF is a candidate profile |
| 6: RES | Positive branch needs the core's nonnegative clamp. Rational negative branch is unsettled; retain `linear_scaled` alternative |
| 7: Stain | Curve agrees at baseline. Lakshana/Psyche is correct; translated Cosmos/Psyche is erroneous. Base1.2 is not universal under enhancements (Blackbird tooltip: 30%). CI owner/read time needs a hypothesis |
| 8: final | Sum effects assigned to this zone; a supplied harness multiplier does not establish native classification |
| 9: special | Multiply only factors established as independent, never automatically per item |
| 10: Cycle | Require reaction-specific base/level, crit and eligible factors. Nova bypasses DEF; ordinary typed bonuses must not automatically reach reactions |
| 11: Break | Consistent per-contributor equation. BI and bonus add together. Team eligibility, resistance channel and rounding need binding; gauge loss is separate |

## DEF-ignore decision

Retain `A/(A + DEF × (1−shred) × (1−ignore))`, `A=600+6×attacker_level`. Ignore reduces the rational denominator; ignore25% is not damage+25%.

Current `DynamicStats::apply_modifier` and live ability/team-ignore addition sum distinct eligible ignore sources; shred sources sum separately. The two remaining-defense factors then multiply. This is the current tested model. Tooltips and a tentative author build sum do not experimentally settle native multi-source stacking/caps. Dynamic clamp versus exact rejection is an API boundary, not two native rules.

General defender shred reduces the base percentage layer before adding flat DEF. Multiplying precombined buffed/flat DEF by `1−shred` is not equivalent; require a layer-aware adapter. Ignore belongs to the actor/packet, shred to the target. Mental/True/zero-DEF targets gain nothing from ignore.

## Conflicts and return conditions

| Question | Current handling | Reopen with |
| --- | --- | --- |
| Negative RES | Rational gameplay candidate and explicit linear alternative; −0.20 gives 7/6 versus 13/11 | Applicable raw tests at multiple negative values or bound formula evidence |
| Stain/Hexed CI owner | Explicit owner/capture policy; Reddit's highest-participant statement conflicts with the newer article's dealer policy | Tests varying only support CI, then dealer CI, or bound consumer |
| Damaging reaction owner | Both guides maximize level-base × CI-zone; the translated separate highest-CI statement can contradict that. Treat owner and CI capture together as a hypothesis | Unequal-level/unequal-CI measurements or native binding |
| Nova bonuses | Preserve the guide's console Mental-bonus exclusion; reject automatic ordinary-bonus reuse | Controlled typed/general-bonus tests or bound reaction packet policy |
| Stain/Hexed scope | Correct elements; helpers alone do not establish follow-up eligibility/snapshot/retrigger | Bound status/packet lifecycle or controlled measurements |
| Ignore stacking/cap | Additive current model after eligibility/stack resolution; helper clipping does not prove game cap | Controlled two-source/saturation tests or native aggregation/consumer |
| Display/settlement rounding | Reported display floor is distinct from current exact nearest/ties-up | Independent HP deltas for fractional packets, not displayed integers alone |

Bounded source pass reused existing findings. Stop: no engine reclassification based solely on translated guides, inaccessible originals or another model's code. Reopen only under the conditions above; unrelated implementation can continue.

## Verification

Current core combat tests and existing CLI build/combat independent oracles check implementation arithmetic. Independent `Fraction`/`Decimal` checks cover the contract's additive ATK, live base-scaled ATK, bonus zone, ignore alone/with shred/two sources, both negative RES curves, crit, CI/Stain, Break, healing and shields. Check local links in changed documents and staged whitespace. Outputs stay in `target/verification` and `target/tool-logs`; native uncertainty remains as listed above.

```text
python tools/cargo.py test --locked --offline -p ntesim-core combat::
python tools/cargo.py build --locked --offline -p ntesim-cli
python tools/check.py -- python tools/verify_combat_cli.py
python tools/check.py -- python tools/verify_build_cli.py --out target/verification/combat-formula-build.json
```

The local supplemental `reviews/combat-formula-audit/check_contract.py` checks document links, the displayed exact examples, frozen Stain pair/Blackbird tooltip, and eight `combat-stats` cases including two ignore carriers and live ATK plus the shared damage-bonus zone. Its receipt is `target/verification/combat-formula-contract.json`, binding the pasted input and changed document hashes. It is a review scratch check, not another native mechanics specification.

The build oracle initially stopped at an obsolete optimizer rejection: a one-stat allowed list was treated as insufficient for the removed fixed allocation heuristic. The current equipment contract/compiler require a distinct nonempty list and return unsupported optimization. The negative case now checks an empty allowed list, preserving the actual input guard; no production behavior changed. Static additive-stat comparisons had already passed before this stale expectation failed.
