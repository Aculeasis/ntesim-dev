# Proposal: Shinku Burst Cooldown and Charge Rotation Fix

## Why

Shinku's curated preset `presets/studio/shinku-charge.nte` suffers from abnormally long rotation intervals (~35–43s), massive party-wide energy overflow (>530 energy over 120s), and depressed DPS. Investigation identified a compounding 33.5s Ultimate cooldown bug in `crates/content/src/registry/kits.rs` and `skill.rs`: when the Surging Crimson stance concludes with the Dragonflame Verdict finisher at ~13.5s, the engine assigns a fresh 20-second cooldown rather than tracking the remaining duration of the 20-second commitment window that began on initial Burst cast. Furthermore, the curated rotation in `shinku-charge.nte` contains an idle 1.5s wait on Zero (`wait(.zero.skill.cd)`) and holds Shinku on-field spamming unbuffed basic attacks waiting for the bloated burst cooldown.

Fixing the burst cooldown accounting and smoothing the support sequence in `shinku-charge.nte` aligns Shinku's combat cadence with the 20–25s standard cycle, halves party energy overflow, and raises curated team performance to expected levels.

## What Changes

- **Preserve Burst Commitment Cooldown**: Store the original 20-second Burst deadline (`paid_burst_until_us`) upon initial Crimson Fury activation. When Dragonflame Verdict completes, set the remaining cooldown to `paid_burst_until_us.saturating_sub(now)` rather than starting a new 20-second penalty, ensuring Shinku's Burst is ready when the party's support window finishes.
- **Update Runtime State**: Track `paid_burst_until_us` in Shinku's `State` across continuing cycles and stance transitions.
- **Refine Curated Charge Scenario**: Update `presets/studio/shinku-charge.nte` to eliminate the 1.5s idle wait on Zero and leverage Hathor's held skill (`skill[hold=1.5s]`) to generate hits, unbalance, and clean cooldown synchronization between Zero and Hathor.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `characters/shinku`: Clarify that Ultimate cooldown (20.0s) commences at initial Crimson Fury commitment and that execution of Dragonflame Verdict preserves the remaining commitment cooldown rather than adding a subsequent 20-second cooldown.

## Impact

- `crates/content/src/characters/shinku/awakenings.rs`: Add `paid_burst_until_us` field to `State`.
- `crates/content/src/characters/shinku/skill.rs`: Set `paid_burst_until_us` on initial burst cast and restore remaining cooldown on stance conclusion.
- `crates/content/src/registry/kits.rs`: Calculate Dragonflame Verdict's cooldown as `state.paid_burst_until_us.saturating_sub(*timestamp_us)`.
- `presets/studio/shinku-charge.nte`: Refined curated rotation and sync.
- Verification tests in `crates/content/src/simulation/tests/shinku_actions.rs` and script export benchmarks.
