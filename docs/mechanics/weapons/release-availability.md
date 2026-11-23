# Weapon release availability

- Responsibility: whether frozen package weapon candidates were actually released in Global by the cutoff.
- Current finding: native table identity, recommendation rows, or a passive receipt establish that an item exists in the frozen data, not that it was released or obtainable. The package has 51 frozen native weapon candidates; this is not a released-weapon count. The reviewed Q1 weapon receipts and Q10 passive-value verifier explicitly leave release availability unverified.
- DPS impact: do not present candidate gear as an available build choice until its release status is established.
- Return condition: source evidence tied to Global release state at the frozen cutoff; preserve the source identity and applicability with the finding.
- Evidence: [weapon passive value contract](../shared/weapon-refinement-values.md), [build domain](../../../openspec/specs/runtime-content/spec.md), [owning Rust checks](../../../crates/content/src/simulation/tests/weapon_critical_cycles.rs).
