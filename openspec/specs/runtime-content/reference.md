# Runtime content constants

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

The simulator stores required content constants in typed Rust structures. Every retained field needs a concrete simulation consumer and an independent numeric or causal check. Whole source exports, unused fields and speculative catalogs are forbidden in Git. JSON is available for requests, reports and exports; it is not the storage format for runtime constants.

Source-only combat projections and extracted pages were removed from Git. Optional acquisition inputs live outside Git and are not needed for executing the simulator. Tools must not repopulate committed JSON catalogs or generated source-table sections in mechanics cards.

## Schema and omissions

Typed content preserves distinctions required by its consumers, including absent versus explicit zero, exact units, selected refinement/skill coordinates and owner-specific recipients/clocks. Missing source details must not create invented defaults. Unused source fields are omitted rather than transported into runtime reports.

Targets are anonymous dummies with explicit script-defined stats; enemy catalogs, profiles, spawn arrays, skills and forms are forbidden. See [dummy targets](../../../docs/mechanics/shared/dummy-targets.md), [agent rules](../../../AGENTS.md) and [decision 0009](../../../docs/decisions/0009-anonymous-targets-and-typed-content.md). Mechanical rules and independent checks stay with their character/weapon/set owners.
