# Complete Far Side of the Moon

## Why

Far Side of the Moon equips correctly, but its passive still treats Redirect Skill damage as an unconditional bonus. Model the released Gravitide rotation so the result reflects qualifying Basic Attack hits and the overlap between its two effects.

## What Changes

- Grant bounded Redirect Skill damage stacks from the wearer's damaging Basic Attack packets, with a five-world-second duration and a 300 ms acquisition gate.
- Retain the wearer's 15-world-second Basic Attack damage window from admitted Redirect Skill casts and require both effects for the Lakshana bonus.
- Keep permanent Crit Rate, the documented refinement values and the legal equipment group in the shared CLI and Studio projection.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `runtime-content` gains the Far Side of the Moon passive contract.

## Impact

Rust equipment dispatch, build projection, combat observations, the Studio proc contract and generated build catalog, and existing native and Worker parity coverage.
