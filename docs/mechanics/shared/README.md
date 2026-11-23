# Shared mechanics

These cards own common rules and reusable technical models. A technical service or source projector is labeled as such in its card; it does not prove native gameplay or complete adapters. These cards distinguish missing evidence from pending integration.

| Responsibility | Card |
| --- | --- |
| Outgoing damage, primary stats, zones, crit, DEF/RES, cycle/break/heal/shield arithmetic | [Damage and stat zones](damage-and-stat-zones.md) |
| Exact rational expression evaluation and quantization service | [Exact arithmetic](exact-scaling-arithmetic.md) |
| Character base/growth/terminal breakthrough source selection | [Character progression](character-progression.md) |
| Weapon base/growth/terminal breakthrough source selection | [Weapon progression](weapon-progression.md) |
| Static equipment contributions, compatibility and supplied build constraints | [Equipment builds](equipment-builds.md) |
| Retired console packing boundary (outside simulator scope) | [Console packing](console-packing.md) |
| Raw native packet coefficients, metadata and explicit coordinates | [Native damage values](native-damage-values.md) |
| Raw refinement parameter/curve selection | [Weapon refinement values](weapon-refinement-values.md) |
| Static native build-to-run projection and provenance | [Native build projection](native-build-projection.md) |
| Purchased/effective skills and native action projection | [Native action projection](native-action-projection.md) |
| Permanent extracted parameters, source/graph reader and build-input schema | [Combat-data contract](../../../openspec/specs/runtime-content/reference.md) |
| Explicit structured proc parameter admission | [Configured gear procs](configured-gear-procs.md) |
| Explicit equipped research parameter projection | [Research gear projection](research-gear-projection.md) |
| Static team loadouts/target profiles | [Team loadouts](team-loadouts.md) |
| Explicit anonymous dummy stats and scenario assumptions | [Anonymous dummy targets](dummy-targets.md#explicit-target-inputs) |
| Atomic resource/cooldown/recovery admission service | [Action admission](action-admission.md) |
| Recipient efficiency/carry, capped grants and resource conservation | [Energy accounting](energy-accounting.md) |
| Recipient/stack/expiry/ICD/callback service | [Buffs and callbacks](buffs-and-callbacks.md) |
| Field ownership, phases, cancellation, death, refunds and forms service | [Actor lifecycle](actor-lifecycle.md) |
| Internal clocks/hitlag/deadline research service | [Clocks and action timing](clocks-and-action-timing.md) |
| Explicit-trigger reaction/break research service | [Reaction and break harness](reaction-and-break-harness.md) |
| Bounded content-runner target gauge and Broken-tag lifecycle | [Content-runner Break model](content-break-model.md) |
| Decoded cycle/Break fields and external model findings | [Cycle source findings](cycle-source-findings.md) |
| Scripted incoming damage, mitigation, shield/HP settlement service | [Incoming damage harness](incoming-damage-harness.md) |
| Seeded deterministic streams and bounded sampling service | [Seeded randomness](seeded-randomness.md) |
| Explicit per-target/hit research crit policy | [Research critical hits](research-critical-hits.md) |
| Action timing, animation and cancel policies | [Action timing](action-timing.md) |
| Build legality, constraints and candidate pools | [Build legality](build-legality.md) |
| Critical hit sampling and distributions | [Critical sampling](critical-sampling.md) |
| Cycle gauge and Break interaction questions | [Cycles and Break](cycles-and-break.md) |
| Defense, resistance and layered reduction policies | [Defense and resistance](defense-and-resistance.md) |
| Energy accounting, generation and gauge questions | [Energy and gauges](energy-and-gauges.md) |
| Frozen Global evidence applicability | [Evidence applicability](evidence-applicability.md) |
| Incoming damage dependencies and mitigation | [Incoming dependencies](incoming-dependencies.md) |
| Stat rounding and value quantization | [Quantization](quantization.md) |

Runtime scheduling/report interfaces remain in [execution](../../../openspec/specs/execution-safety/reference.md), [game runner](../../../openspec/specs/combat-execution/reference.md) and [DSL execution](../../../openspec/specs/script-execution/reference.md). Product all-in-range dummies and one real DPS-time axis remain requirements, not source discoveries or implemented native targeting/timing claims.
