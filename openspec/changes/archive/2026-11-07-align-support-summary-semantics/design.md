# Design

## Context

See [proposal.md](proposal.md) for motivation and [the delta](specs/studio-application/spec.md) for observable behavior.

`crates/research/src/trials/studio/observe.rs` already validates completed execution and native state timelines, collects canonical healing receipts, and recursively totals explicit shield absorption. It currently assigns applied healing to `hps` and absorption per second to `sps`. `model.rs` and `mod.rs` expose these samples and their statistical aggregates; the same Rust collector serves native source execution and Studio through WASM/Worker.

Native `GameStateFrame` observations contain active actor identity and incremental actor replacements. Initial/final frames contain complete states. `GameActorShieldSummary` includes identity, remaining/max HP and exact expiry in microseconds. The recorder preserves equal-time settlement order; an expiry can be observed after its actual deadline during an idle gap. These observations are sufficient for this report calculation without adding a second shield event ledger.

The reference gcsim checkout was inspected at commit `5dacbb4f0e27944a1409743342a7ad17998eae53`. Its [healing collector](https://github.com/genshinsim/gcsim/blob/5dacbb4f0e27944a1409743342a7ad17998eae53/pkg/stats/heal/heal.go) records received healing before the overheal deduction; [overview aggregation](https://github.com/genshinsim/gcsim/blob/5dacbb4f0e27944a1409743342a7ad17998eae53/pkg/agg/overview/overview.go) divides healing by duration and time-weights normalized shield HP. Its [shield selection](https://github.com/genshinsim/gcsim/blob/5dacbb4f0e27944a1409743342a7ad17998eae53/pkg/stats/shield/effective.go) chooses the strongest shield against each damage type. This establishes the reference card meanings, not NTE stacking rules.

## Goals / Non-Goals

**Goals:** Compute both metrics once in Rust from authoritative observations, preserve deterministic real-time accounting, and deliver identical values through samples, aggregates, histograms and exports.

**Non-Goals:** Change combat mechanics, introduce incoming attacks, add Genshin damage-type multipliers, sum every shield grant as throughput, or add another set of support cards.

## Decisions

### 1. Gross healing defines HPS

Sum `healing_receipts.gross` across all recipients and divide by `elapsed_us / 1_000_000`. The canonical receipt invariant is gross = applied + excess. Autonomous healing has another causal representation and must not be counted twice. Preserve the existing `gross_healing`, `healing` (applied), and `excess_healing` aggregate keys; no additional effective-HPS field is needed for this change.

Alternative: keep applied HPS as the primary card. Rejected because it answers how much missing HP was restored, which remains available in the applied total but does not match the requested card.

### 2. SHP measures the active actor's available protection

For a living active actor, capacity at time t is the sum of remaining HP of its distinct unexpired shields. NTE currently settles shields sequentially with `ShieldOrder::OldestFirst` in `crates/core/src/kernel/state/mod.rs`; distinct shields therefore contribute additive absorbable capacity. Replacement by shield identity already occurs in `crates/content/src/simulation/state/actor.rs`. Read the resulting native state rather than reimplementing replacement.

Define `shp = integral(capacity(t) dt) / elapsed_us`, with capacity in HP and time in microseconds. Each actor's shields remain recipient-specific. Only the controlled actor contributes at a given instant; team-wide grants copied to several recipients do not multiply protection. Defeated actors contribute zero. This active-actor interpretation is the explicit reporting choice for adapting gcsim's party protection to NTE; it does not change shield ownership or settlement.

Alternatives: a strongest-shield rule would contradict current additive NTE settlement; summing every party member would multiply off-field copies; summing grants would measure throughput and ignore downtime. None matches available protection.

### 3. Integrate native boundary states with exact deadlines

Add a small Rust support-observation helper within the existing research collector. Start from the complete initial actor state. For every next frame at time t, integrate the previously settled state over the half-open interval [previous_time, t), then apply actor-slot replacements and the frame's active actor index. Equal-time frames have zero duration; the last settled state at that time governs the next positive interval.

Within an interval, each shield contributes `remaining_hp * max(0, min(next_time, expires_at_us, elapsed_us) - previous_time)`. Thus expiry needs no new scheduler event or synthetic state frame. Depletion, grant, replacement, defeat and switches arrive through native state changes; maximum HP is not a substitute for remaining HP. Clip at actual trial completion, not the requested horizon.

Use checked `u128` accumulation for HP-microseconds and convert only the final ratio to the public floating-point statistic. Validate required actor slots, unique shield identities, remaining/max HP consistency and bounded frame times. Propagate invalid observations as collector errors through the existing failure path. Empty support activity produces positive numeric zero, not a missing metric or fabricated fallback. Do not alter scheduler or callback order for a report calculation.

Alternative: infer capacity from shield-absorption receipts or a final-state snapshot. Neither establishes shield uptime, unused protection or intermediate replacement, so both are insufficient.

### 4. Replace the current contract atomically

`Sample.sps` and aggregate `metrics.sps` become `shp`; `hps` changes to gross semantics. Keep `shield_absorbed` as the separate actual-damage total, including redirected deliveries exactly once and excluding non-shield interception. Do not add legacy-input defaults or an `sps` alias.

Update `web/src/domain/contracts/trials.ts`, result selectors, `Summary.tsx`, `ResultsPage.tsx`, and `Histogram.tsx`, plus any discovered exports/fixtures. The six cards remain DPS, effective EPS, damage, HPS, SHP and duration. Use `Healing Per Second (HPS)` with HP/s and an explicit includes-overheal explanation; use `Effective Shield HP (SHP)` with HP and an explicit time-average/active-actor explanation. Applied/excess healing and absorbed shield damage remain secondary totals. The existing statistical renderer supplies mean, min, max, standard deviation and quartiles from each metric's per-trial distribution.

WASM and Worker transport carry Rust results unchanged. Resume verification, export serialization and histogram selectors must all consume the same new fields. No TypeScript reconstruction of support mechanics or rate-to-capacity conversion is permitted.

## Risks / Trade-offs

- Idle gaps can overstate uptime if observed removal times are used -> clip every shield contribution at its native expiry.
- Delta frames and same-time updates can double count or apply the wrong actor -> reconstruct by slot and integrate before applying each next boundary; add independent switch and equal-time examples.
- An active-actor SHP hides off-field reserves -> label the interpretation explicitly and retain the full native state in report/export views.
- Synthetic interval tests prove accounting, not game accuracy -> pair them with existing Iroi and Adler source fixtures; preserve native mechanics assumptions and uncertainty.
- Old saved payloads have different keys and HPS meaning -> update current Rust/TS contracts and fixtures together, with no backward-compatibility path.

## Migration Plan

Implement and verify Rust samples/aggregates first, then update current TS consumers and the Studio detailed reference in the same coherent change. Extend existing native support tests, actual WASM/Worker transport tests and report-control coverage; visually inspect desktop and narrow reports with positive healing/shields and zero incoming damage. Run focused affected checks, review the stable diff, then `python tools/checkpoint.py run`. Synchronize/archive the delta only after implementation and checks complete; validate the archived tree and run the final checkpoint before committing. No publication or deployment is part of this change. Rollback, if needed, reverts the coherent implementation and its accepted spec updates together.
