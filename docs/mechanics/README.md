# Mechanics dossiers

These references retain mechanical source facts, selected models and unresolved questions under the [rolling data policy](../../openspec/specs/product-scope/reference.md#rolling-data-policy). [Product requirements](../../openspec/specs/product-scope/reference.md#dps-relevance-and-modeling-boundary) determine relevance; the selected [OpenSpec change](../../openspec/README.md) defines current work. Neither table presence nor a tested hypothesis establishes a complete kit.

Existing cards' Global/cutoff labels and hash-bound references describe the evidence behind each model. They do not restrict new findings to that version or require another receipt pipeline. Update the owning rule when current supported evidence supersedes it, retaining material conflicts and a source link/file reference. Prefer current verifier/test sources over generated run receipts; keep authored models and independent expected inputs only while they have a current consumer.

| Owner | Mechanics dossier |
| --- | --- |
| Released characters | [Characters](characters/README.md) |
| Weapon candidates | [Weapons](weapons/README.md) |
| Set candidates | [Sets](sets/README.md) |
| Shared combat and builds | [Shared rules](shared/README.md) |

The acquisition record identifies the initial traversal, category supplement and subsequent immutable research bundles. Start targeted follow-up from the named owner: [native target sources](shared/dummy-targets.md#explicit-target-inputs), [Chiz Grain/market findings](characters/chiz/grain-and-market-source-findings.md), [Fadia threshold branches](characters/fadia/action-damage.md#recovered-threshold-and-damaged-hp-branches), or [cycle/Break source findings](shared/cycle-source-findings.md). Recovered configuration belongs in the owner card even when native execution remains open; cache-only supplementary evidence is not automatically part of the committed compact package.

The [compact contract](../../openspec/specs/runtime-content/reference.md#schema-and-omissions) identifies which selected component parameters are now portable. Follow each owner card for native behavior and open questions; a repaired layout or portable parameter does not establish execution.

## Responsibility and authority

A filename identifies one responsibility, not one number. A whole damage calculation, action train, resource lifecycle or passive can be one card when its rules must be understood together. Split a card when its responsibilities have different owners, evidence or independently closable questions; do not create one file per multiplier or an undifferentiated `misc.md`.

Each card contains:
- **Source facts** and explicitly labeled **tested hypotheses / bounded models**.
- **Open questions** and **return conditions** specifying what cannot yet be inferred from client assets.

Current mechanical prose has one owner. When migrating it, remove the old prose and rewrite its consumers to links. Delete an emptied document rather than keeping a forwarding stub. API, architecture, product and change documents can describe their own interfaces/requirements and link to mechanical owners; they must not maintain another copy of an owner's values, triggers or stacking rules.

Frozen source bytes and saved `reviews/` receipts remain historical evidence. Their claims apply to their recorded inputs/tree, not automatically to current code. Link them from cards with this boundary; do not modify receipts to simulate a new validation or retain them as a separately updated current specification. If a card and an old receipt differ, document the reason and applicable evidence.

## Research and implementation

Start from the selected OpenSpec change and relevant behavior capability, then read only the needed owner and shared references. Targeted research answers concrete evidence gaps. Research history retains useful source leads and rationale, not mandatory campaign steps or completion counters. The [data package reference](../../openspec/specs/data-packages/reference.md) describes current readers and audits. Markdown is not a runtime constant storage format.
