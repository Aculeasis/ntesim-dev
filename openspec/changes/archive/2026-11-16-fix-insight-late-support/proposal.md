# Proposal

## Why

Curated scripts stop refreshing support well before their 120-relative-second endpoint because they reserve entire rotations and switch to carry-only tails. Insight seed 42 ends support casts at absolute 119.292479 seconds (relative about 70). Removing those reserves exposes a runner defect: an admitted action extending beyond the time limit is classified as incomplete.

## What Changes

- Make stop=time finish successfully at the exact relative duration while an admitted action or ordinary authored wait is in progress; retain settled damage and pending later effects.
- Stop source evaluation at that endpoint, including during implicit switches, before another input or expression can run.
- Remove end-of-run reserves and carry-only filler tails from the twenty curated compositions, retaining normal state, readiness and support scheduling.
- Preserve failed admissions, unresolved explicit energy gates, victory failures and independent safety limits as unsuccessful or incomplete outcomes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `script-execution`: successful time completion during actions and ordinary waits, without evaluating the suffix.
- `studio-application`: continuing curated rotations without early support shutdown or artificial final tails.

## Impact

Shared Rust source/native execution and existing owner/source checks; all twenty curated .nte scripts and their reference. WASM, Worker and Sample inherit shared-engine behavior. The S5 historical control retains its authored source. No automatic readiness retries or frame polling are introduced. Catalog recalculation remains an explicit operation under the development guide.
