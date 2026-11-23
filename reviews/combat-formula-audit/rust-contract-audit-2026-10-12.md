# Rust combat contract audit, 2026-10-12

Authority: [Damage and stat zones](../../docs/mechanics/shared/damage-and-stat-zones.md), with equipment/build, weapon, damage-value, exact arithmetic, RNG, lifecycle, enemy and reaction owner contracts.

Three `gpt-6-luna` agents at high reasoning reviewed shared arithmetic, build/equipment projection and live delivery. The primary agent independently checked their counterexamples, report semantics and owner scope, implemented the confirmed fixes and reviewed the final diff. No game code or external downloaded code was executed; frozen evidence was preserved.

## Confirmed fixes

| Boundary | Defect | Resolution and independent counterexample |
| --- | --- | --- |
| Static buff projection | DEF-ignore/shred clamped after each modifier, losing contributions and making signed modifiers order-dependent. | Sum active contributions before clamping each affected channel. The same projection computes `AtApplication` snapshots. Initial 0.9 plus 0.3 minus 0.2 resolves to 1.0, rather than 0.8; Lv80 vs DEF1080 has factor 1 rather than 5/6. Initial 0.1 minus 0.3 plus 0.2 resolves to 0 rather than 0.2; factor 1/2 rather than 5/9. Both channels and application orders are tested. Unmodified invalid input channels still reach evaluator validation. |
| Floating Mental damage | `hit.element` selected the bonus channel even for `kind=mental`, allowing Psyche or Physical bonuses on a Mental packet. | Mental packets read the `Mental` bonus channel. Base1000, Psyche+50%, Mental+20%, zero RES and no crit resolve to 1200, rather than 1500. General/action bonuses remain additive; True still bypasses them. Ordinary physical remains represented as an ordinary packet with the Physical channel. |
| CI/Stain/Hexed helpers | Negative CI silently became a neutral factor; nonfinite values were not rejected consistently. | Helpers return checked `Result` values and reject negative/nonfinite CI, invalid Stain bases and factor overflow. At CI108, Q=1.18, E=1.075, baseline Stain=1.29 and candidate Hexed fraction=0.29. Zero remains valid. No native reaction execution is inferred from these helpers. |

The bare `Chaos` stat modifier now joins the other five native element aliases; the existing `elemental:Chaos` route already worked. This is consistency repair, rather than a separate damage-formula defect.

## Findings rejected or bounded

- Live `GameHitReceipt.crit_rate` retains the raw summed stat, including values above one; the existing Demon Blade test requires 29/25. Sampling already treats such values as guaranteed critical without consuming RNG words. The research-crit contract now states that distinction explicitly; receipt shape and runtime behavior are preserved.
- Break `floor()` occurs in the explicit-trigger floating research harness. The exact gameplay/incoming rounding rule does not establish a quantization policy for that harness, so changing it would exceed the evidence.
- Good Boy projects the ordinary Ultimate team-ATK effect only. Frozen refinement text also names unconditional Charge Efficiency (T1 0.18) and extra team ATK when the Ultimate controls enemies (T1 0.06). These remain coverage gaps in the explicitly partial owner adapter, not a demonstrated error in shared stat-zone arithmetic. This audit does not certify complete Good Boy execution.
- Public exact resistance fractions with zero denominators reject through checked arithmetic; comparison overflow also rejects. No panic or silent fallback was reproduced.

## Coverage and limits

Reviewed shared floating/exact zones, stat layers, static buff stacking/snapshots, healing/shield settlement, CI helpers, equipment derivation, content build/stat projections, weapon/set modifier placement, native packet delivery, critical rules, research profile consumers and CLI report boundaries. The static equipment oracle remains base ATK1094, percent0.825, flat420, resulting ATK2416.55; live ATK percentages scale from base ATK. DEF-ignore and shred remain separate factors, matching damage bonuses share one additive zone, and healing separates source outgoing from explicit recipient incoming bonuses.

Character and weapon reviews were targeted by shared modifier and packet-binding paths, not an exhaustive reconstruction of every frozen effect for all 23 kits. Native stacking/capture hypotheses, negative RES alternatives, partial passive coverage and reaction admission remain their documented owner limitations. No new readiness claim is made.

## Verification

- `python tools/cargo.py test -p ntesim-core --locked --offline`
- `python tools/cargo.py clippy --workspace --all-targets --locked --offline -- -D warnings`
- Native CLI and WASM artifact builds with locked offline dependencies.
- `python tools/check.py -- python tools/verify_combat_cli.py`: existing Decimal/Fraction endpoints, invalid requests, replay, relocation and frozen-input preservation, extended with four signed DEF channel cases and one Mental channel case.
- Final formatting and staged whitespace checks.

The changed floating formula harness is not exported by the WASM runtime. Gameplay report shape and exact live damage paths did not change; browser Worker behavior was not modified.
