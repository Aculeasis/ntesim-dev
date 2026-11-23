# Explicit research equipped parameter projection

Responsibility: optional `gear_projection` compilation in `run-game-built`/`batch-game-built`, preserving caller and native source receipts. The [build projector](native-build-projection.md) validates static loadouts; the [refinement reader](weapon-refinement-values.md) resolves native parameters.

Policies `native_mind_royale_curve_v1` and `native_mind_royale_fireflies_solo_v1` require a separate nonempty hypothesis. Their supported owner/scope, values, triggers, units and lifetime models are owned solely by [Mind Royale](../weapons/mind-royale/passive.md) and [Fireflies](../sets/fireflies/passive.md). The restricted research scope is distinct from ordinary native Studio/DSL equipped execution. Resolving source values does not validate a native consumer or release availability; strict rejects these research projections.

## Compilation and receipts

Check actor/build identity, weapon group, selected refinement, equipped set/shape conditions, level and initial vitals before projection. Resolve exactly the policy's named native parameters with their declared percent hints. Additional unknown parameters, duplicate proc configuration, unsupported wearer/scope and mismatched equipped identities fail. Parameter unit conversion must be exact: resource units integral, seconds converted to positive integral microseconds, and all values/deadlines representable. Never round unresolved units into supported inputs.

Replace an existing matching proc at its original position or insert it once; retain `replaced_proc:null` for insertion. Preserve actor slot, native identity/refinement, old/new configuration, complete native buff/curve/key evidence and `runtime_execution_validated:false`. Other caller proc hypotheses and static/action projection assumptions retain separate identities. Compilation never mutates caller inputs.

`batch-game-built` compiles the same receipts once, hashes the canonical scenario including projected proc values and executes the requested seed/timing rows without a pilot. Failure/statistics follow the [built batch contract](../../../openspec/specs/source-bound-batches/reference.md). Every output retains incomplete quality and `game_dps_computed:false`.

## Consumers and verification

[Mind Royale example](../../../examples/native-gear-built-game-research.json) and [restricted Fireflies example](../../../examples/native-fireflies-built-game-research.json) deliberately replace caller parameters. `tools/verify_native_gear_projection.py` and `tools/verify_native_fireflies_projection.py` use hash-checked raw exports, independent Fraction/PCG/resource/expiry calculations, insertion/replacement, boundary failures, replay/batches and scoped WASM parity. Their independent source and input checks are maintained in those verifier scripts; source mutations and unsupported scope guards live in content/CLI tests.

Unanswered native consumers/applicability belong to the matching unknown owner cards and [shared applicability](evidence-applicability.md). Existing typed producer integration belongs to the implementation queue.
