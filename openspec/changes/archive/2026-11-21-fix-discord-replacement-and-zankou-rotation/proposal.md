# Proposal

## Why

In `lacrimosa-scorch`, DoT-triggered Scorch stack refreshes under Zankou's Silent Sunset pass `new_discord_application = true`, minting a new Discord generation ID on every 1.0s Nightmare DoT tick and re-triggering Discord 24 times rather than once per Scorch/Nova pair. Concurrently, the flagship `zankou-blackbird` preset contains a redundant immediate switch to Daffodill right before Zankou that wastes over 14 seconds in idle switch recovery. Resolving the Discord deduplication defect and streamlining the Zankou-Blackbird rotation restores accurate mechanics and brings team DPS in line with established tier rankings.

## What Changes

- **Fix Discord deduplication on Scorch replacement**: Ensure that Scorch stack additions and duration refreshes triggered by ally DoT deliveries (Silent Sunset) maintain the existing `discord_generation` (`new_discord_application = false`), emitting Discord callbacks only upon genuine initial Scorch/Nova coexistence rather than on every periodic DoT tick.
- **Streamline `zankou-blackbird.nte` preset rotation**: Remove the redundant zero-action switch to Daffodill right before Zankou, allowing Daffodill's entry sequence to progress properly and eliminating ~15 seconds of dead switch recovery across the 120s combat timeline.
- **Update curated team catalog measurements**: Recompute and update `web/src/generated/team-list.json` and associated preset metrics to reflect the corrected mechanics and rotation.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `characters/zankou`: DoT-triggered Silent Sunset Scorch replacements and stack refreshes preserve existing Discord generation deduplication and do not re-emit Discord callbacks against coexisting Nova statuses.
- `studio-application`: The curated `zankou-blackbird.nte` preset rotation eliminates redundant immediate switches, and curated team catalog results reflect corrected Discord reaction deduplication and rotation execution.

## Impact

- **Affected code**: `crates/content/src/simulation/reactions/replacement.rs`, `presets/studio/zankou-blackbird.nte`, `web/src/generated/team-list.json`.
- **APIs and contracts**: Preserves the `combat.discord_break_gauge_v1` specification that periodic ticks and refreshes of existing Scorch/Nova pairs emit no redundant Discord callbacks.
- **Replay and tests**: Affected tests in `zankou_scorch.rs` and team list validation fixtures will reflect corrected reaction counts and accurate DPS figures.
