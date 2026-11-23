# Design

## Context

See `proposal.md` for motivation.

In the current implementation:
1. When an ally DoT tick lands, `on_dot_delivery` in `crates/content/src/simulation/reactions/replacement.rs` invokes `apply_scorch(ctx, ctx.state.current_time_us, target_index, snapshot)`.
2. `apply_scorch` in `scorch.rs` delegates to `apply_scorch_with_application(..., new_discord_application = true)`.
3. Because `new_discord_application` is unconditionally true, a brand-new `discord_generation` ID is minted on every DoT tick. When coexisting with Nova, `discord_pairs.insert((target_index, scorch, nova))` treats each tick as a distinct reaction event, emitting Discord 24 times in 120s rather than once per pair.
4. Concurrently, `presets/studio/zankou-blackbird.nte` contains an immediate `daffodil swap;` check directly preceding `zankou swap;`, causing Daffodill to enter the field for a 1.25s switch recovery without executing actions before being instantly replaced by Zankou.

## Goals / Non-Goals

**Goals:**
- Enforce the `combat.discord_break_gauge_v1` invariant: DoT-triggered Scorch stack additions and duration refreshes under Silent Sunset must reuse the existing `discord_generation` and not re-emit Discord callbacks against coexisting Nova statuses.
- Preserve legitimate Zankou and Lacrimosa synergies (Scorch Enhancement 3-stack cap, DoT scaling, and Nightmare maintenance).
- Eliminate idle switch recovery in `zankou-blackbird.nte`, improving rotation throughput and ensuring entry attack sequences complete as intended.
- Regenerate `web/src/generated/team-list.json` and related preset validation assets.

**Non-Goals:**
- Changing base Discord reaction damage, Break gauge deduction ratios, or elemental priority rules.
- Redesigning Zankou's or Lacrimosa's underlying kit abilities or stat allocations.
- Altering other unrelated team presets or historical control benchmarks (such as S5).

## Decisions

### Decision 1: Distinguish DoT-triggered Scorch replacement from initial switch application
In `replacement.rs`, DoT-driven Scorch updates represent an in-window stack increment and duration extension rather than a new Cycle switch.
- **Approach**: Call `apply_scorch_with_application(ctx, now_us, target_index, snapshot, false)` (or expose an explicit refresh helper) during `on_dot_delivery`.
- **Rationale**: Reusing `existing_discord_generation` allows the deduplication check `discord_pairs.insert(...)` in `maybe_trigger_discord` to recognize that the Scorch/Nova pair has already reacted, preventing runaway Discord cascades.
- **Alternatives considered**:
  - Adding an internal cooldown (ICD) to Discord: Unnecessary and arbitrary, as the existing pair-keyed deduplication set is already designed to prevent repeated callbacks for the same pair.
  - Suppressing Lacrimosa's passive callback: Would leave the Break gauge reduction bug intact; fixing deduplication at the reaction source resolves both passive triggers and Break shred correctly.

### Decision 2: Streamline `zankou-blackbird.nte` rotation
- **Approach**: Remove the redundant second Daffodill swap condition preceding Zankou's entry, and let Daffodill's entry actions execute when she enters the field cleanly.
- **Rationale**: Eliminating 14.8s of wasted switch recovery allows Zankou and Blackbird to align their damage windows within Sakiri's 20s ATK buff window and restores proper DPS parity.
- **Alternatives considered**:
  - Leaving the preset unchanged: Leaves a known ~15s dead window in the curated preset, skewing comparative analysis.

## Risks / Trade-offs

- **[Risk: Regression tests relying on replacement-triggered Discord]** → Mitigation: Inspect `zankou_scorch.rs` and `lacrimosa_public.rs` to ensure tests expecting single-pair Discord admission continue to pass while verifying deduplication on DoT delivery.
- **[Risk: Stale team list catalog hashes]** → Mitigation: Regenerate `web/src/generated/team-list.json` using the project's standard generation scripts and verify consistency across native and WASM targets.
