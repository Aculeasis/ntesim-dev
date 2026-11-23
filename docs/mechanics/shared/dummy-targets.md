# Anonymous stationary dummy targets

## Explicit target inputs

Targets are anonymous stationary dummies. Authors set HP, maximum HP, DEF, common/elemental resistance and any needed level/Boss assumptions in the same shareable simulation script. For example:

```nte
target lvl=80 hp=100000000 def=1080 resist=0.2 boss=true;
```

`boss` is an explicit eligibility assumption for character/gear effects. It never resolves an enemy identity. Multiple target declarations create multiple dummies; all are in range, subject only to each owner's recipient cap/distribution rule. Coordinates do not participate in this model: `pos` and structured `position_mm` inputs reject, and the UI displays no position map. Target level remains an explicit mechanic input, including Zero's level-gated execute condition. There are no concrete enemy catalogs, native profile or pack lookups, spawn/level arrays, enemy forms, enemy skills, scripted boss phases or automatic target presets. `type` and `pack` fields reject, including in legacy imported requests.

## Break and incoming damage

`original_atk` optionally supplies a nonnegative integer initial dummy ATK for
[Haniel's B2 Nova extraction](../characters/haniel/action-damage.md#selected-complete-passive-specification).
There is no default. Reached extraction rejects omission, while explicit zero
is legal and funds no grant. Exact runtime ATK and source-specific transfer/
rollback receipts are reported separately from HP damage. This assumption
never generates attacks or determines an incoming-damage event's amount.

Adler's Temperance can reduce reported live ATK by 20% of the authored original ATK for its status lifetime. This percentage reduction is independent of Haniel's capped extraction ledger.

An explicit complete `break_gauge`, `max_break_gauge`, `broken_duration` triple enables the existing [Break state machine](content-break-model.md). Omission supplies no gauge. The incoming gauge multiplier starts at one; damage-relevant dynamic modifiers must belong to a checked mechanic. Never derive stats, a gauge, timing or Boss status from game mob tables.

Explicit incoming-damage/critical/parry assumptions use the shared [incoming settlement contract](incoming-damage-harness.md) and retain their costs, windows and callbacks. They do not simulate an enemy AI or encounter.

## Runtime data policy

Only values consumed by simulation mechanics belong in runtime content. Store them in typed Rust constants with a concrete consumer and independent checks. Raw exports, unused fields and speculative catalogs stay outside Git; JSON is transport/export, not runtime constant storage. The [agent guide](../../../AGENTS.md) and [decision](../../decisions/0009-anonymous-targets-and-typed-content.md) enforce this boundary.
