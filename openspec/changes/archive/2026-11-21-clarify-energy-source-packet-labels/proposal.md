# Proposal

## Why

In Studio's "Energy Per Source" chart, characters with specialized energy mechanics (such as Riposte, Plunge, or Support packets) display duplicate rows labeled `"<Character> ordinary hit"`. This happens because:
1. `EnergyPanels.tsx` aggregates rows by `[source, kind]` rather than by resolved display label, splitting different `event_kind` values into distinct chart rows.
2. `energy-sources.ts` only recognizes a handful of direct packet event kinds, causing all other specialized energy grants (such as Riposte, Plunge, and Support packets) to fall back to the default `"<Character> ordinary hit"` label.

Deduplicating rows with identical display labels in the panel and explicitly labeling specialized energy events gives users accurate, informative insight into where team energy is generated.

## What Changes

- Register specialized energy packet event kinds in `web/src/domain/reports/energy-sources.ts`:
  - Riposte energy grants: `iroi_riposte_energy_grant`, `sakiri_riposte_energy_grant`, `mint_riposte_energy_grant`, `nanally_riposte_energy_grant`, `adler_riposte_energy_grant`, `aurelia_riposte_energy_grant`.
  - Support energy grants: `sakiri_support_packet_energy_grant`, `daffodil_support_energy`, `nanally_support_packet_energy_grant`, `zero_support_packet_energy_grant`, `blackbird_support_energy_grant`, `blackbird_evolved_support_energy_grant`, `chiz_support_energy`.
  - Plunge energy grants: `iroi_plunge_energy_grant`, `adler_plunge_energy_grant`.
- Update `web/src/features/results/EnergyPanels.tsx` to group chart rows by resolved display label `[source.source, details.label]`, ensuring rows with identical display labels never duplicate on the chart.
- Add focused tests verifying that energy source details resolve specialized packet labels correctly and that the panel produces unique rows.

## Capabilities

### Modified Capabilities
- `studio-application`: Clarify that Energy Per Source rows present distinct labeled energy sources without duplicated generic ordinary hit rows for specialized actions.

## Impact

- Web UI presentation: `web/src/features/results/EnergyPanels.tsx`, `web/src/domain/reports/energy-sources.ts`.
- Trace event localization: `web/src/domain/localization-events.ts` automatically inherits clearer labels for Riposte/Support/Plunge events.
- No simulator domain or Rust engine changes: contracts, receipts, and calculation remain unchanged.
