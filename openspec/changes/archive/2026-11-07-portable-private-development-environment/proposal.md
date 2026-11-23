# Proposal

## Why

Tracked tooling, instructions and evidence metadata contain paths and permissions tied to one developer's computer. A fresh checkout must work on supported platforms without exposing that computer's layout or requiring its private configuration.

## What Changes

- Keep optional machine instructions and permission rules under ignored `.local/`, referenced by the shared agent guide without duplicating it.
- Accept private tool inputs through environment variables, with an explicit ignored environment-file option on the existing command runner.
- **BREAKING**: native acquisition tools require an explicitly configured client binary directory instead of assuming an installation location.
- Remove machine paths and session-specific setup narratives from tracked documentation and source metadata while retaining evidence identities.
- Extend checkpoint's existing static boundary to reject private local configuration, machine paths and recognizable credential literals in tracked or commit-eligible inputs.
- Verify execution from a relocated checkout with spaces and no private configuration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `development-workflow`: optional private instructions and configuration, explicit external tool inputs, and a repository portability/privacy check.
- `distribution`: fresh-checkout reproduction must be independent of the author's paths and private configuration.

## Impact

Shared agent/development documentation, local Git exclusions, checkpoint static checks and regression coverage, the existing command runner, native acquisition consumers, and frozen research source metadata. No mechanics, engine contracts or publication changes. Existing platform support remains unchanged; Git history and author metadata are outside this change.
