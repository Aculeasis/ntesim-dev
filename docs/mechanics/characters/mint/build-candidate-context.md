# Mint reviewed A0 build context

**Responsibility:** static values for one saved Mint build candidate. This is build provenance, not a character mechanic or an optimized-build claim.

The candidate is Mint A0 at level 80 / breakthrough 6, purchased Skill level 8, Mind Royale T5, Fireflies, a 20-cell console packing, and 15 explicit substat lines. Reviewed static values: character/weapon base ATK 596/555 (combined 1151); ATK 46383/20; HP 17720; DEF 727; Crit Rate 57/100; additional Crit DMG 47/50; Anima bonus 1/10.

These exclude unresolved runtime effects. Coordinate-8 scalar probes are source-bound but do not establish effective skill level or hit count. Candidate evidence: [`intent.json`](../../../../reviews/s1-mint-a0/intent.json), [`build.json`](../../../../reviews/s1-mint-a0/build.json), [`review.json`](../../../../reviews/s1-mint-a0/review.json). Current static-build assertions are checked by [owning Rust checks](../../../../crates/content/src/simulation/tests/mint_actions.rs) and [the Rust projection test](../../../../crates/content/src/build/tests/missing_receipts_and_nonzero_unsupported_typed_bonus_fail.rs). The candidate is incomplete and not optimized.

**Open questions:** [matching unknown responsibility card](action-damage.md#unknown-execution-policies).
