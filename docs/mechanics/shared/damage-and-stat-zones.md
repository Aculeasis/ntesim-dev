# Combat formulas and stat zones


Read this before changing stats, gear, buffs or damage. It owns shared arithmetic for the frozen Global `global-2026-10-04` model; effect owners determine eligibility, recipients, stacking and snapshots. A formula does not establish native execution. [The formula review](../../../reviews/combat-formula-audit/review.md) records evidence, disagreements and the pasted-formula audit.

## Units and owners

Use fractions: `20% = 0.20`, a `154.2% ATK` coefficient is `1.542`, and `150% CRIT DMG` means an **additional** `1.50` (a critical hit deals `2.50×`). CI and BI are points, not percentages. Native names alone do not establish units.

| Owner | Read for |
| --- | --- |
| [Equipment](equipment-builds.md), [build binding](native-build-projection.md) | Base constants, additive packs, mains/substats and exact static projection |
| [Weapons](weapon-progression.md), [passive values](weapon-refinement-values.md), [gear procs](configured-gear-procs.md) | Refinement parameters, activation and live effects |
| [Damage values](native-damage-values.md), [action binding](native-action-projection.md) | Coefficients, packet types, fixed crit and skill coordinates |
| [Lifecycle](actor-lifecycle.md), [RNG](seeded-randomness.md) | Stacks, expiry, capture/read time and critical sampling |
| [Reactions](reaction-and-break-harness.md), [enemies](incoming-damage-harness.md) | Explicit reaction/break harness and incoming damage; harness values are not native defaults |
| [Exact arithmetic](exact-scaling-arithmetic.md) | Rational evaluation and declared final quantization |

## 1. Add within a zone; multiply between zones

Resolve eligible active contributions under their stacking policy first, then sum each stat/zone. Reapplication does not automatically create another contribution. Do not infer an independent multiplier from the words "increases" or "%".

In the static active-buff harness, `ReplaceBySource` replaces a repeated ID/stat, `ReplaceByTarget` replaces modifiers on that stat, and `AddStacks` caps repeated ID/stat applications. Distinct carriers require distinct IDs. `AtCast` reads original source stats; `AtApplication` reads stats after earlier applications. `Dynamic` requires live lifecycle state and is rejected by the static evaluator.

| Quantity | Combine eligible contributions | Apply |
| --- | --- | --- |
| ATK%, HP%, DEF% | Sum in the corresponding percentage stat | `base × (1 + sum) + flat_sum` |
| Flat ATK/HP/DEF | Sum in the corresponding flat stat | Add after percentage scaling |
| CRIT Rate / additional CRIT DMG | Sum separately, including character defaults once | Apply packet critical rule |
| General + matching damage type + matching action + other damage bonus | Sum **together** | One `M_bonus = 1 + bonus_sum` |
| DEF-ignore / DEF-shred | Sum each stat separately in the current model; see §3 | Two remaining-defense factors |
| RES-ignore / RES-shred | Subtract applicable percentage points from original RES | One effective RES, then its curve |
| Final Damage bonuses | Sum effects explicitly assigned to this zone | One `M_final = 1 + final_bonus_sum` |
| Stain | One applicable status factor, separate from damage bonuses | `M_stain` in §4 |
| Independent special effects | Only evidence-bound independent factors | `M_special = product(M_j)` |
| Outgoing / incoming healing bonuses | Sum within each source/recipient zone | Multiply the two zones |
| Break intensity + Break Damage bonuses | `BI/300 + break_bonus_sum` | One `1 + BI/300 + break_bonus_sum` |

`Final Damage` is not a catch-all for an unclassified effect. A coefficient addition changes the base term; an ordinary damage bonus changes `M_bonus`; they must not be substituted for one another.

Example: general20% + matching-element30% + skill40% gives **`M_bonus=1.90`**, not `1.20 × 1.30 × 1.40=2.184`. A separately evidenced final25% zone then multiplies that result by `1.25`.

## 2. Primary stats and packet base

```text
base_atk = character_base_atk + weapon_base_atk
ATK = base_atk × (1 + sum(AtkUp)) + sum(AtkAdd)
HP  = character_base_hp × (1 + sum(HPMaxUp)) + sum(HPMaxAdd)
DEF = character_base_def × (1 + sum(DefUp)) + sum(DefAdd)
B = sum(stat_i × coefficient_i) + flat_damage
```

Weapon ATK is base ATK; module flat ATK is not. A support buff calculated from the source's base ATK enters the recipient's flat layer after its explicit cap, if any. A conversion uses its explicitly named source stat and capture time; do not guess base versus final.

A compiled static ATK already includes gear: a live `p` ATK% buff adds `base_atk × p`, so `ATK_live = ATK_static + base_atk × p + live_flat`. Never apply `ATK_static × (1 + p)` or apply static gear again. Analogous percentage rules hold for HP and DEF where the runtime adapter supports them.

Equipment oracle: `base_atk = 620 + 474 = 1094`, `AtkUp = 0.45 + 0.375 = 0.825`, `AtkAdd = 420`; **ATK = 2416.55**. A new 20% ATK buff gives **2635.35**, not 2899.86. Six authored ATK% additions of 9% give **54%**, not `1.09^6 − 1`.

Coefficient additions are `k = k_native + sum(delta_k)`; an explicitly relative coefficient boost is `k = k_native × (1 + sum(relative_boosts))`. Require owner evidence for the choice; retain multi-stat and flat terms rather than silently discarding them.

The native runtime can transport exact mixed ATK/DEF/HP weights inside one packet. It reads each used live stat from its own base/flat zones, sums the weighted terms before common damage factors, samples one critical outcome per target packet and quantizes once. Legacy single-stat requests keep their existing representation. Mixed HP terms retain the owner's existing HP eligibility and modifier limits. A coefficient addition without an owner-selected term diagnoses instead of being applied to every term. Independent Rust checks cover `200/3 + 150*2/7`, live stat zones, and the rounding counterexample `1/3 + 1/3 -> 1`.

## 3. Direct damage, crit, DEF and RES

For an eligible ordinary packet:

```text
D_raw = B × M_bonus × C × F_def × F_res × M_stain × M_final × M_special
M_bonus = 1 + general + matching_type + matching_action + other − dealt_reduction
C = 1                       on a noncritical hit
C = 1 + CD                  on a critical hit
E[C] = 1 + p × CD           before quantization
```

`p = clamp(CR, 0, 1)` for normal crit; `none`, `fixed50`, `fixed100` use `0`, `0.5`, `1`. CR/CD start at 0.05/0.50 for the ordinary character model and gear/buffs add once. Fixed crit replaces CR, not CD. At CR=0.8 and CD=1.5, `E[C]=2.2`. An expected raw value is not expected rounded damage or a sampled combat result.

| Packet kind | DEF | RES | Bonus, final and crit |
| --- | --- | --- | --- |
| Elemental / ordinary physical | Apply | Apply matching channel | Apply eligible modifiers |
| Mental | Bypass (`F_def=1`) | Apply Mental channel | Apply eligible modifiers; Mental is not Psyche |
| True | Bypass | Bypass | Bypass; current core policy is `D_raw=B × M_special` |

All six native elements (Anima/NATURE, Cosmos, Chaos, Incantation, Lakshana, Psyche) are separate from Mental and ordinary physical damage. Damage kind alone does not grant crit eligibility or make a reaction eligible for ordinary bonuses.

### DEF-ignore is a defense stat, not a damage bonus

The community candidate for a defender with base, percentage and flat layers is:

```text
A = 600 + 6 × attacker_level
DEF_effective = [DEF_base × (1 + DEF_boost − DEF_shred) + DEF_flat] × (1 − DEF_ignore)
F_def = A / (A + DEF_effective)                   for nonnegative DEF_effective
```

Current shared core accepts nonnegative target DEF and separate shred/ignore in `[0,1]`. For the supported unbuffed-enemy case:

```text
DEF_effective = target_defense × (1 − shred) × (1 − ignore)
```

**Sum ignore sources into `ignore`; sum shred sources into `shred`; do not sum ignore and shred together.** This aggregation is the current tested model, not independently established native multi-source stacking/cap behavior. The dynamic-stat harness clamps each aggregate to `[0,1]`; the exact evaluator rejects an out-of-range aggregate. Owners must define cap/replacement behavior before passing a value.

With Lv80 attacking DEF1080: no reductions gives `F_def=1/2`; ignore25% gives `4/7` (14.2857% more damage); shred20% plus ignore25% gives `5/8`, since 60% of DEF remains. Two distinct eligible ignore sources of 25% and 15% give ignore40% and `5/8` under the additive model. Multiplying their remaining fractions would be a different model. Ignore100% gives `F_def=1`; ignore adds no benefit to Mental/True or zero-DEF targets. Ignore is attacker/packet-scoped; shred is target-scoped even where a harness transports it in attacker stats.

Content target debuffs now preserve an optional authored `base_def` separately from total `def`. The shared target-stat reader computes `max(0, total_def - base_def * sum(def_percent_reduction))` before packet DEF-ignore and the shared DEF factor. Omitting `base_def` explicitly assumes total authored DEF is also base. Sakiri Impish Trick supplies 10% on successful Suppress: base 1000 and total 1500 become 1400 rather than 1350. Direct packets, Break, Blossom and Scorch query this same delivery-time stat with world-time expiry. Fractions remain exact and negative results clamp at zero under the declared model. Native negative-DEF rules remain unverified.

Use explicit dummy DEF assumptions. `600 + 6 × target_level` is an optional community candidate, not an automatic target rule.

### Resistance

```text
R = original_resistance − applicable_ignore − applicable_shred
F_res = max(0, 1 − R)                    when R >= 0
F_res = 1 − R/(1 − R)                    when R < 0: standard_rational
F_res = 1 − R/1.10                       when R < 0: linear_scaled
```

The negative branch is **unresolved**: `standard_rational` is the current gameplay candidate; pinned Drive Calculator evidence uses `linear_scaled`. At `R=−0.20` they give `7/6` and `13/11`. Keep the selected model explicit. RES-ignore20% subtracts 0.20 points, not 20% of existing RES. Obtain original/weakness RES from the target profile; do not assume all enemies have 20%.

## 4. Esper Cycle effects

```text
Q(CI) = 1 + CI/600
E(CI) = 1 + 0.2 × CI/(CI + 180)           CI >= 0
M_stain = stain_base_multiplier × E(CI)  when Stain applies; otherwise 1
D_cycle = B_reaction(level) × Q(CI) × C × F_def × F_res × eligible_reaction_factors
```

Stain applies to **Lakshana/Psyche**, not Cosmos/Psyche. Its baseline multiplier is **1.2**; an evidenced enhancement may replace that base (Blackbird's frozen tooltip specifies 30%, hence a candidate base of 1.3). At baseline CI108, `M_stain=1.29`; the 7.5% intensity tooltip is relative to the 1.2 base, not total damage gain. Keep it out of `M_bonus` and do not apply it twice. Its CI owner/capture policy remains a reviewed hypothesis (see the review), not an automatic highest-team-CI rule.

Baseline reaction rules: Blossom and Nova cannot crit; Scorch uses fixed50 and the owner's CD; Nova is Mental and bypasses DEF. Base damage is reaction- and level-specific, not ordinary ATK scaling. Select element, owner, CI, defense/resistance ignore and other factors under an explicit reaction policy. Do not automatically reuse ordinary DMG/elemental/Mental bonuses: the published reaction guide excludes console Mental bonus from Nova. Enhancement/final/special applicability needs owner evidence.

The research helpers expose `hexed_fraction = 1.2 × E(CI) − 1`; treating it as a fraction of the triggering hit is a candidate, not native follow-up execution. Hexed needs its own packet/eligibility/rounding policy. Remora/Charge/Discord are not generic level-scaled damage packets. [Reaction harness](reaction-and-break-harness.md) defaults and caller-selected triggers do not establish these game mechanics.

The floating CI/Stain/Hexed helpers reject negative or nonfinite CI and return checked results; an invalid intensity cannot become a neutral multiplier. Stain base multipliers must also be finite and nonnegative, and an overflowing factor is rejected.

## 5. Break, healing and shields

```text
D_break_i = B_break(level_i) × (1 + BI_i/300 + break_bonus_i)
            × (original_break_cap/3) × F_def_i × F_res_i × M_special_i
D_break_team = sum(eligible living actors' contributions)
gauge_shred = base_shred × (1 + break_efficiency) × enemy_break_resist_efficiency

H_raw = [sum(stat_i × heal_coefficient_i) + heal_flat]
        × (1 + outgoing_bonus_sum) × (1 + recipient_incoming_bonus_sum)
H_applied = min(quantize(H_raw), recipient_missing_hp)
overheal = quantize(H_raw) − H_applied
shield_raw = [sum(stat_i × shield_coefficient_i) + shield_flat] × (1 + shield_bonus_sum)
```

Break Damage, gauge depletion, BI and Break Efficiency differ. Baseline break has no ordinary damage bonus or crit factor; a broken target has no automatic vulnerability. `original_break_cap` is not remaining gauge. Base break and resistance-channel/aggregation policies require the scenario/owner profile; the generic four-actor harness does not prove native dead-actor eligibility. Existing reference arithmetic: two Lv1 contributions with base91, cap45, DEF654, RES0.20/0.16 sum to about 1076.66 before rounding.

Healing uses the healer's outgoing and the recipient's incoming bonuses. A healer's received-healing stat cannot affect another recipient. Settlement requires valid current/max HP and does not implicitly revive. Shield stacking/absorption belong to lifecycle/settlement. Live healing remains the tested `combat.live_healing_modifiers_v1` hypothesis where supported.

The formula harness requires explicit `healing_target: {current_hp, max_hp}` with positive living HP within capacity. Healing packets settle in order against continuing HP through shared `VitalState`/`Pool`; omitted state cannot use demonstration vitals.

## 6. Implementation and verification boundaries

Preserve exact intermediates; quantize at the declared packet/settlement boundary. Pasted research reports floor for displayed damage/stats. The floating harness accepts explicit rounding; exact gameplay/incoming packets use nearest integer, ties upward. **Display rounding is not proof of HP settlement rounding.** No universal damage cap or per-stage truncation is validated by these helpers.

Floating `combat::evaluate_direct_damage` also offers a generic harness vulnerability factor `1 + attacker.vulnerability + target_vulnerability`; it is not a native Stain adapter. The exact evaluator has no dedicated Stain/vulnerability or flat/multi-stat base fields; content supplies a single stat/ratio and explicit final/special factors. Extending a consumer must preserve zone identities and reject reached missing behavior, not silently set a required factor to one. Formula correctness alone does not establish action, reaction or full-team gameplay support.

Shared implementations: [stat layers](../../../crates/core/src/combat/stats.rs), [floating zones](../../../crates/core/src/combat/damage.rs), [DEF/RES/crit/CI](../../../crates/core/src/combat/arithmetic.rs), [exact zones](../../../crates/core/src/combat/exact/mod.rs), [live delivery](../../../crates/content/src/simulation/hit/calculation.rs). `combat-stats REQUEST.json` is a formula harness; quality/readiness follows the actual adapter.

An owner may also have a source-established rounding step inside its calculation. [Lacrimosa's A5 ceiling](../characters/lacrimosa/nightmare-damage-and-duration.md#a5-maxhp-reduction-and-healing-source-chain) belongs to that returned additive-heal expression, not to a general replacement for final HP settlement. Preserve such stages when binding their owners and keep remaining native settlement questions explicit.

Use independent numbers distinguishing addition from multiplication, base from final stats, and ignore from shred. Existing [owning Rust checks](../../../crates/content/src/simulation/tests) and [owning Rust checks](../../../crates/content/src/simulation/tests) check raw-export/Fraction/Decimal arithmetic, replay and invalid requests. Core exact/floating tests check zones and damage-kind parity. These checks validate implemented arithmetic, not unresolved game hypotheses.
