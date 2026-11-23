# Reactions, elemental auras, break, and spatial geometry contract

In integrated combat, ordinary Scorch, Nightmare, Heartwrench, Vile Ash and other periodic damage/healing retain phase and lifetime on paused world time through cinematics and active Hotori Time Stop. A quarter-second remaining phase resumes a quarter second after release; a carrier applied inside the pause starts its full phase there. Refresh keeps its owner's existing stack/generation rules, and there is no catch-up. Immediate supported Discord/reapplication callbacks retain causal settlement and are not periodic ticks. The isolated harness below advances its supplied clock; it is not evidence that absolute time advances these carriers through global pauses. See [clock composition and evidence limits](clocks-and-action-timing.md).



This contract describes the explicit-trigger research harness in `ntesim-core::reaction`, including spatial targets, break state and resource conservation. It does not implement native aura-driven reaction admission or complete the gameplay reaction work tracked in the [OpenSpec changes](../../../openspec/README.md).

The content simulation owns the reviewed switch model (`combat.esper_cycle_reactions_v1`) for Blossom, Scorch and Nova, plus the explicit Stain, Remora/Charge and Discord extensions (`combat.stain_switch_v1`, `combat.remora_charge_v1`, `combat.discord_break_gauge_v1`), per [cycles and Break](cycles-and-break.md). Daffodil's selected Discord cap reduction is separately bound by `daffodil.discord_cap_reduction_v1`. These are typed content-simulation models over authored stationary dummies; they do not add automatic native target profiles or establish hidden game defaults. Hexed and other owner extensions remain pending there. The selected stationary-dummy models in that card supersede historical movement/harness examples below.

The [combat formula contract](damage-and-stat-zones.md) owns shared zones, CI/BI scaling and game-formula hypotheses. Fixed bases, stacking, Remora damage and four-actor contributions below are explicit harness behavior, not evidence overriding the game-formula contract.

## 1. Spatial target geometry and splash AoE

Combat targets are situated in 3D Euclidean space:

- `Point3D { x_mm: i64, y_mm: i64, z_mm: i64 }` in integer millimeters ($1\,000\text{ mm} = 1.0\text{ m}$).
- Exact integer distance squared calculation eliminates floating-point drift:
  \[
  \text{dist}^2 = \Delta x^2 + \Delta y^2 + \Delta z^2 \le r^2
  \]
- **Targeting rules**:
  - `SingleTarget { target_id }`: impacts the targeted enemy only.
  - `AllTargets`: global effect impacting every active combat target.
  - `SplashRadius { center_target, radius_mm }`: impacts all enemies located within `radius_mm` of the central target.
- **AoE scaling boundary**:
  - **Four targets are never an automatic $4\times$ scalar multiplier.**
  - Targets outside splash radius (e.g. spread-out enemies) are not hit. Multi-target damage is strictly the sum of verified individual target hits.

## 2. Break accumulation, states, and team break damage

### Break gauge and shred

- Targets specify `break_cap: u64` and `current_break_gauge: u64`.
- Attacks with `break_shred > 0` deduct gauge points on hit.
- Target states:
  - `BreakState::Unbroken`: normal state.
  - `BreakState::Broken { broken_at_us, duration_us }`: break state (default duration 5.0 seconds = `5_000_000` $\mu\text{s}$).

### 4-character team break damage
When an enemy's break gauge is fully depleted:

- The enemy transitions to `Broken`.
- **All four team members simultaneously deal break damage** against the broken target.
- For each actor $i \in \{0, 1, 2, 3\}$, break damage is computed via:
  \[
  \text{BreakDamage} = \text{base\_break} \times \left(1 + \frac{\text{intensity}}{300}\right) \times \left(\frac{\text{break\_cap}}{3}\right) \times \text{def\_factor} \times \text{res\_factor}
  \]
  - Verified against the published Zero+Mint Lv1 break endpoint ($525\text{ damage/actor}$, $2\,100\text{ total team break damage}$).
- After `duration_us` (5.0s), the target automatically unbreaks and restores its full break gauge.

### Separate no-break scenarios

- `break_enabled: bool`:
  - When `break_enabled == false`: break shred does not deplete gauge, targets never enter the Broken state, and break damage is completely suppressed.
  - Provides no-break scenario variants for explicit comparisons; the legacy Reference guide preset is optional.

## 3. Elemental reactions

The caller explicitly selects a reaction; the listed fixed damage values are harness parameters, not source-bound native reaction scaling or proof of elemental prerequisites:

- `ReactionKind::Blossom`: instant reaction dealing $9\,000$ base damage.
- `ReactionKind::Scorch`: periodic damage over time (DoT):
  - Base damage: $2\,700$ per stack.
  - Tick interval: strictly $0.99\text{ seconds}$ (`990_000` $\mu\text{s}$).
  - Duration: $15.0\text{ seconds}$ (`15_000_000` $\mu\text{s}$).
  - Max stacks: $3$.
  - Stacking refreshes duration; ticks occur on schedule without immediate tick on application.
- `ReactionKind::Nova`: explosive reaction dealing $45\,000$ base damage, detonating immediately on refresh.
- `ReactionKind::Remora`: charging remora bomb dealing $10\,000$ base damage.

Scorch retains the latest applying actor as its owner. Advancing time settles every scheduled tick before expiry, even when the caller jumps across the entire duration; no tick occurs at the expiry boundary. Attack and explicit-reaction calls return the periodic receipts produced while advancing to their timestamp. Polling frequency cannot change damage or attribution.

## 4. Energy mechanics and per-event bounds

Energy management maintains atomic conservation and strict event scope:

- Each actor possesses an exact integer `Pool` with capacity (typically 100).
- Ultimate abilities atomically spend energy.
- **Per-event energy grant policy**:
  - Energy gained from attacks (e.g. +25 energy) is awarded **per event**, strictly **not multiplied by the number of targets hit in AoE**.
  - Hitting 3 targets with a splash skill grants the base energy amount once, preventing exponential energy generation in multi-target combat.

## 5. CLI execution and independent verification

The native CLI exposes:
```bash
ntesim reaction-stats_DIR REQUEST.json
```
Produces a structured JSON receipt with total hits, break events with 4-actor contribution breakdowns, triggered reactions, final target states, and team energy pools.

### Verification oracle
Independent verification is enforced by `tools/verify_reaction_cli.py`:

- Replays spatial splash geometry and verified hit counts.
- Verifies 4-actor simultaneous break damage ($2\,100$ total).
- Verifies Scorch DoT tick progression and stack capping.
- Rejects 12 invalid configurations (zero team size, team size > 4, actor count mismatch, empty targets, bad package ID, nonexistent target for reaction, energy > capacity, negative team size, invalid targeting type, invalid reaction kind, missing target position, unauthorized top-level field).
- Validates byte-identical replay and relocation invariance.
- Checks frozen package file SHA-256 hashes and caches proof in `target/verification/reaction-state.json`.

Direct ReactionEngine construction shares actor-slot, target-identity, ratio and break-state validation with the scenario adapter. Attack, reaction and time transitions commit atomically: missing break resistance, overflow or any later target failure restores resources, time, target states and pending Scorch. Fixed-base explicit reactions remain a research harness, not automatic game reaction/cycle support.
