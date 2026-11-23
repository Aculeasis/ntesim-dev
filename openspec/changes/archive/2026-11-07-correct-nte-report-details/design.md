# Design

## Context

See proposal.md. Rust Studio Collector currently computes damage/energy/action statistics only. Canonical healing receipts already record gross/applied/excess for both ordinary and autonomous healing. Incoming damage receipts expose shield absorption and redirected children independently of immunity and non-shield interception. Source damage includes NTE Esper Cycle packets under `ntesim.reaction`, currently rendered as indistinguishable custom abilities.

The screenshot and bundled element artwork agree on the six palette identities. Targets already carry base and element-specific resistance ratios. The overview's `align-items: start` leaves an external blank gap below its short target panel.

## Goals / Non-Goals

**Goals:** Fix the target/legend/palette regressions and surface existing native support observations accurately across streamed batches, histograms, stopped results and exports.

**Non-Goals:** Add or alter combat mechanics, infer target geometry, import Genshin reaction semantics, classify unknown packets through string heuristics or infer shield generation/uptime from final capacity.

## Decisions

- Resolve target resistance display with the same override/base rule as Rust. Render existing element artwork, name and percentage together in a wrapping grid. Stretch desktop overview panels and constrain histogram prose density; keep mobile stacking.
- Centralize correct colors in the existing element palette. Add a reusable visible chart legend, populated from the same actor slots and filter as Source DPS. Keep owner colors independent from element colors.
- Name known native Esper Cycle effect IDs explicitly in the display label helper; unknown sources preserve their original identity rather than becoming generic custom abilities. Existing Iroi summon effect names remain intact.
- Extend the Rust Studio sample with HPS and shield-absorption rate, and the native aggregate with gross/effective/excess healing and shield-absorbed totals. Observe canonical healing receipts once; recurse through redirected incoming receipts to sum explicit shield absorption only. TypeScript consumes these values without recomputing mechanics or batch statistics. Update current contracts without legacy defaults.
- Keep six cards: DPS, retained EPS, damage, effective HPS, shield absorption per second and duration. Keep hit/event counts in the execution summary. Show healing totals and a precise shield metric note; add HPS/shield-rate histogram choices.

## Risks / Trade-offs

- Full HP means effective HPS is zero even with healing casts → show gross and excess healing alongside the effective rate.
- Unused shields produce no absorption → label the metric as absorption and explain that it needs incoming damage.
- Autonomous heals also have causal-event records → use canonical healing receipts only to avoid double counting.
- Payload changes cross native/WASM/Worker resume/export boundaries → rebuild current WASM, run the existing native/Worker checks and browser controls, then checkpoint.

## Migration Plan

Update Rust collection and TS consumers together, validate focused native/UI checks and screenshots, run checkpoint, synchronize and archive, then verify the final tree and commit locally.
