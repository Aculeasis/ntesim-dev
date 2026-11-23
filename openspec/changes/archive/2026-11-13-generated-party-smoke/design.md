# Design

## Context

See proposal.md for motivation. In S5, Zero execution supplies 75% DEF ignore, Lost Radiance adds 25% and applicable Iroi support adds 5%. Passing 105% to the exact formula rejects the supported composition. Separately, automatic owner model declarations can exceed the former combined 32-entry metadata limit for legal four-member parties.

## Goals / Non-Goals

**Goals:** fix these two observed failures with exact arithmetic and targeted regressions at existing Rust boundaries.

**Non-Goals:** generated composition sweeps, exhaustive equipment combinations, new public fields, browser features or newly claimed game evidence for multi-source stacking.

## Decisions

### Bound effective content DEF ignore

Use one small shared content helper after applicable packet/equipment/team contributions are added. Saturate at exactly 100% in direct and Break evaluation; preserve the exact sum below the bound and checked arithmetic. Individual exact-formula inputs above 100% still fail validation. The existing defense reference already documents this bounded model; native multi-source semantics remain unverified.

The S5 regression changes only Zero to Lost Radiance and uses one iteration, asserting complete execution, a later execution packet and complete native replay. Existing shared contract and Break tests cover independent values below, at and above the cap.

### Separate canonical metadata from custom entries

Exact canonical ID/description pairs are bounded by the finite engine registry and do not consume the existing 32-entry custom allowance. Unknown identities and mismatched descriptions still consume that allowance; per-entry byte limits, strict matching and reached-conflict checks remain enforced. Extend the existing metadata validation test, including the exact boundary and a spoofed canonical description.

### Keep verification focused

The broad generated party test is removed at the users request. Existing full-roster/refinement/set tests retain their roles. The focused S5 regression takes approximately 0.3 seconds locally. No runtime or transport shape changes are needed; the admission change outside the API directories nevertheless selects the existing WASM/Worker smoke boundary explicitly.

## Risks / Trade-offs

- The 100% cap is the documented bounded simulator model, not new native stacking evidence.
- Canonical exemptions must match both identity and description to preserve the custom bound.
- Focused tests cover the observed regressions without claiming arbitrary party or passive coverage.

## Migration Plan

Retain changes uncommitted on `codex/generated-party-smoke`. Run focused checks and `python tools/checkpoint.py run --wasm-smoke`, then synchronize/archive the completed OpenSpec artifacts and check the final tree. Do not merge, publish or deploy. Current input shapes need no migration.
