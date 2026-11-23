# Native critical-hit sampling

Responsibility: game eligibility and RNG/shared-roll rules, independently of reproducible technical streams.

Which packets can crit, where do fixed overrides apply, when is CR/CD captured, and are draws shared across packets/targets? Does a source-specific random selection use one roll per cast, hit or recipient, and what weights/ownership apply? Do not infer these answers from the damage kind or curve name alone.

Consequence: damage distributions, trial branches and delayed target outcomes change. Current [seeded randomness](seeded-randomness.md), [research crit policy](research-critical-hits.md) and [damage/stat zones](damage-and-stat-zones.md) establish their explicit modeled semantics only; individual known owner overrides remain in their dossiers.

Checked evidence: owner `FixedCritRate` fields, frozen descriptions/graphs and cited receipts. PCG golden words and native/WASM replay check deterministic execution, not NTE's underlying generator or shared-roll policy.

Return on applicable packet eligibility/capture evidence and a discriminating multi-hit/multi-target policy. Preserve explicit model IDs and source conflicts; no blanket crit assumption or guessed distribution.
