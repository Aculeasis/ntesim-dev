# Native incoming-damage dependencies

Responsibility: source-bound incoming profiles and survival/evasion/settlement rules only when causally needed for DPS.

Which enemy/profile/mitigation channels apply to a declared hurt event? Which owner shield/heal/HP-loss triggers count absorption, positive overheal, exact depletion, source survival and death? What are the native cancellation/invulnerability/Critical Dodge/Riposte windows for an eligible assumed-success input?

Consequence: reached damage-dependent gear/buffs/resources must not be silently disabled. Full enemy AI/movement/survival fidelity is outside product scope; an explicit scenario input can implement a bounded needed trigger without claiming native AI.

Known configured mitigation/absorption/HP settlement services are in [incoming damage harness](incoming-damage-harness.md); damage/heal/shield formula zones are in [damage/stat zones](damage-and-stat-zones.md#5-break-healing-and-shields). Owner-specific hypotheses remain in the corresponding dossier.

Checked leads: native monster/skill/figure tables, source descriptions and owner receipts. The deterministic scripted enemy harness's movement/phases/auto field transfer are technical fixture policies, not automatically gameplay rules. Explicit current runner hurt is separate from the harness's enemy-defense model.

Return on a reached profile/window/trigger rule or reviewed bounded scenario model with independent absorption/HP-loss/expiry/callback checks. Implementation of accepted scenario hurt and assumed-success syntax remains in Q4-L5.
