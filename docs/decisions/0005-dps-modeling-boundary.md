# 0005: Model the causal path to team DPS

- Status: accepted (retrospective)
- Recorded: 2026-10-16
- Original decision date: unknown
- Current owners: [product modeling boundary](../../openspec/specs/product-scope/reference.md#dps-relevance-and-modeling-boundary), [evidence and modeling](../DEVELOPMENT.md)

## Context and decision

The product compares reproducible team DPS under declared scenarios. Required fidelity follows direct and indirect damage dependencies, including resources, buffs, HP conditions, action availability and timing. Use the owning modeling policy to justify bounded omissions; retain unsupported dependencies and hypotheses visibly.

Accurate scenario-conditioned DPS is the product requirement. Minimal event/state representations and the all-in-range dummy scenario are scope/model choices. They do not claim to reproduce an entire spatial encounter or the whole game. Detailed scope stays in the product brief, not this record.

## Rationale

Full spatial simulation, enemy AI, and complex movement consume substantial development and runtime resources without improving reproducible DPS comparisons against stationary training targets. Conversely, treating non-damage actions as irrelevant drops critical causal dependencies like heal-triggered buffs, shields, or energy generation. Bounding the simulation to direct and indirect causal paths to team damage directs effort toward usable combat coverage without silently accepting inaccurate damage estimates.

## Consequences and assumptions

Relevance must be assessed for the declared scenario and reached owners. An omission valid for one team may fail for an HP-dependent team. Preserve applicability and return conditions in the existing mechanic owner/receipt and limitations in results; do not create a parallel mechanics or coverage registry. All-in-range scenarios exclude spatial coverage claims while retaining target caps, hit counts and delivery behavior.

Console geometry and specialization are outside the model: console stats are script-authored with `add stats`; only declared 2-/4-piece set effects apply automatically.

This choice assumes useful comparisons can be scoped explicitly. Unknown relevance remains unresolved; strict/model admission and independent checks still apply under their owners.

## Reconsider when

A new owner or scenario exposes a causal dependency omitted by the simplified representation. Extend that owner and its checks before advertising the scenario as complete. Broader spatial/survival product goals would require a scope decision; a single missing damage-relevant trigger need not force simulation of the entire game.

## Verification

Use independent numerical/logic checks and causal boundary cases in existing owner oracles, including resource/timing effects and the DPS denominator. Verify hypotheses, omissions and unsupported reached dependencies remain visible through reports/exports. This documentation adds no executable capability or evidence of baseline readiness.
