# 0009: Anonymous targets and consumed typed content

- Status: accepted
- Decision date: 2026-10-21
- Authority: explicit human instruction
- Owners: [agent guide](../../AGENTS.md), [dummy targets](../mechanics/shared/dummy-targets.md), [OpenSpec changes](../../openspec/README.md)
- Clarifies [0005](0005-dps-modeling-boundary.md); supersedes the JSON runtime-catalog requirement in [0008](0008-architecture-consolidation-and-hygiene.md).

## Decision

Targets are anonymous stationary dummies with stats and assumptions authored in the single GCSim-style script. Concrete enemies, monster stat/profile catalogs, native spawn/level selection, enemy skills/forms and boss-phase simulation are forbidden. Required Boss eligibility and incoming-damage/Break conditions are explicit scenario inputs.

Commit only constants required and actually consumed by simulation mechanics. Runtime content belongs in typed Rust structs and tables, containing only consumed fields. Raw source exports, unused parameters and speculative catalogs stay outside Git in `cache/`. Embedding an untrimmed JSON payload in Rust does not satisfy this rule. JSON remains available for request/report/export transport.

## Rationale and consequences

Native target catalogs and generic JSON source projectors expanded the repository without helping script-defined dummy DPS. Source availability is not a reason to add data or execution branches. Keep a concrete Rust consumer and independent numeric/causal check for each retained runtime field. This decision also removes duplicate JSON hypothesis metadata: typed Rust identifiers/descriptions already supply the consumed assumptions.

Mechanics-first coverage, one Rust engine, explicit hypotheses, real combat time, the four-CPU ceiling and native/WASM/Worker parity remain required. Research evidence can support mechanics without being committed or loaded at runtime.

**2026-10-23 clarification:** Generated native research backlogs are cache-only source observations. Package
audits verify retained inputs and explicit incomplete readiness; they do not
require a source-candidate catalog or infer executable coverage from its rows.
The `gaps` command exposes audited package scope declarations and optional
canonical owner identity. Reached mechanics, assumptions and failures belong
to simulation reports and Rust checks, while development follows the active
selected OpenSpec change.

## Verification

The language and structured request reject enemy/profile selectors. Repository checks reject enemy catalogs, even small ones. Explicit dummy stats, elemental resistance, Boss eligibility, Break cycles and incoming-damage callbacks retain independent Rust and native/WASM/Worker checks. Data migration must preserve those values and behavior while removing unused fields and JSON runtime catalogs.

### Explicit attack-start and proximity observations

The structured request may declare a finite sequence of anonymous-dummy attack-start
observations directed at a party member and proximity eligibility changes. These
are authored mechanic predicates, without enemy identities, skill definitions,
autonomous decisions or damage generation. Content owns the reached mark/reward
lifecycle. Separate incoming declarations retain their own HP/shield settlement;
observing an attack start never manufactures a contact or damage amount. This
extends the existing explicit-assumption boundary for Hathor Gaze; source lowering
remains part of the native language phase.

Incoming declarations may explicitly bind an event index to an anonymous dummy
instigator for contact-sensitive owner mechanics. The binding preserves the existing
externally mitigated HP/shield settlement and does not imply an attack start,
mitigation rule or additional damage. Zero/prevented contact and paid HP costs
cannot satisfy a positive incoming-hit predicate merely by naming a dummy.

An event may explicitly select the checked before-DEF fixture stage using the same
declared dummy source/level. Its nominal amount excludes only DEF mitigation;
the engine samples the original recipient's live DEF once, preserves a typed basis
receipt and then uses the existing conserved incoming/share/shield/HP path. External
final amounts retain their original semantics. This supports damage-relevant own
DEF carriers without introducing enemy attacks or pretending every incoming kind
uses the fixture formula.
