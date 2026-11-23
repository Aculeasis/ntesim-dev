# Design

## Context

See proposal.md. In the simulation engine, energy granted through team hit fanout uses `GameEnergyFanoutReceipt` where the source is the owner's ordinary hit enum (`GameEnergySource::<Char>Hit`), while the trace event carries a specific `event_kind` representing the originating combat action (`<char>_ordinary_hit_energy_grant`, `<char>_riposte_energy_grant`, `<char>_support_packet_energy_grant`, `<char>_plunge_energy_grant`, etc.).

The trial collector in Rust (`observe.rs`) aggregates energy by `(recipient_actor_index, grant.source, event.event_kind)`.
On the web client:
- `EnergyPanels.tsx` aggregates rows into a `Map` keyed by `JSON.stringify([source.source, source.kind])`.
- `energy-sources.ts` resolves the row's label via `energySourceDetails(source.source, source.kind)`.
- If `source.kind` is missing from `directPacketSources`, `energySourceDetails` falls back to `sources[source].label` (`"<Character> ordinary hit"`).

## Goals / Non-Goals

**Goals:**
- Eliminate duplicated chart rows with identical labels in the Energy Per Source panel by grouping on `[source.source, details.label]`.
- Map specialized energy event kinds (Riposte, Support, Plunge) to descriptive labels in `directPacketSources`.
- Ensure trace event logs (`localization-events.ts`) receive clear, accurate labels for those events as well.

**Non-Goals:**
- No changes to the Rust engine, simulation runtime, or calculation of energy receipts.
- No changes to wire serialization or `TrialResult` contracts.

## Decisions

### 1. Key chart rows by `[source.source, details.label]`
- **Decision:** In `EnergyPanels.tsx`, resolve `energySourceDetails` first, then key the accumulation map with `JSON.stringify([source.source, details.label])`.
- **Rationale:** Ensures that even if two distinct event kinds share a label (e.g. if an unmapped event falls back to a default), their values combine into a single bar on the chart, eliminating ambiguous duplicates.
- **Alternatives considered:**
  - *Keep `[source.source, source.kind]` and only add dictionary entries*: Fragile; any newly added unmapped event in the engine would immediately recreate duplicate rows in the UI.
  - *Key solely by `source.source`*: Loses the ability to distinguish Skill/Riposte/Support packets from normal attack ordinary hits.

### 2. Expand `directPacketSources` with Riposte, Support, and Plunge events
- **Decision:** Explicitly add the following event kinds to `directPacketSources`:
  - Ripostes: `iroi_riposte_energy_grant`, `sakiri_riposte_energy_grant`, `mint_riposte_energy_grant`, `nanally_riposte_energy_grant`, `adler_riposte_energy_grant`, `aurelia_riposte_energy_grant`.
  - Supports: `sakiri_support_packet_energy_grant`, `daffodil_support_energy`, `nanally_support_packet_energy_grant`, `zero_support_packet_energy_grant`, `blackbird_support_energy_grant`, `blackbird_evolved_support_energy_grant`, `chiz_support_energy`.
  - Plunges: `iroi_plunge_energy_grant`, `adler_plunge_energy_grant`.
- **Rationale:** Aligns with how `Zankou Skill packet` and `Daffodil direct packet` are handled, giving complete transparency to energy generation in rotations.

## Risks / Trade-offs

- [Risk] Mismatched `source` field in `directPacketSources` throws in `energySourceDetails`.
  $\rightarrow$ Mitigation: Ensure every mapped `event_kind` strictly points to its owner's `source` (e.g. `iroi_riposte_energy_grant` $\rightarrow$ `iroi_hit`). Add automated assertions in web tests to verify all entries in `directPacketSources` match their declared owner sources.
