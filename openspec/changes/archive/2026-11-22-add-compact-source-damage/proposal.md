# Proposal

## Why

The source charts show which effects contributed damage, but reading the total for an ability such as Wildfire requires adding its combo hits, charged attacks and periodic effects by hand.

## What Changes

- Add a compact `full` / `comp` button immediately after the Source DPS or Source Damage Instances title. `full` is the default and shows the effect breakdown.
- In `comp`, show one row per owner and ability, with the ability name and the sum of its effect means for the selected metric. Apply the character filter, ranking and folding to the resulting rows.
- Use the delivered Studio results so changing modes is immediate and requires no simulation or sample regeneration.

## Capabilities

### New Capabilities

### Modified Capabilities

- `studio-application` gains a compact source view for both DPS and damage instances alongside the detailed effect view.

## Impact

Studio's results UI, source names, and report checks are affected. The change uses the existing Rust aggregates and Worker payloads and requires no new dependencies.
