# Design

## Context

The frozen Arc has five refinement inputs and a single cast-driven carrier. Global release descriptions add a separate Basic Attack hit effect. The same Rust projection feeds authored builds, the CLI and Studio WASM.

## Decisions

- Divide the frozen four-stack Redirect Skill damage curve by four with exact rational arithmetic. Use the release values for the hit effect's duration, acquisition gate and cap, independent of refinement.
- Route damaging category-A packets through the existing positive HP loss hook. Place the per-wearer acquisition deadline on the shared world timer path and apply the stack in the post-hit callback phase. Same-time recipients share the gate.
- Model the five-second period as one refreshable stack group, with qualified hits at cap refreshing its expiry. The release description gives one duration and does not describe independently expiring stacks. Preserve this interpretation, world clocks and ordering in the reached hypothesis.
- Keep the existing cast carrier and read both carriers at delivery. Each damage zone adds once. The cast carrier stores the Basic Attack amount, and the Lakshana amount applies only while the stack carrier and cast carrier coexist.
- Exercise real authored Daffodil packets for all five refinements and use explicit synthetic Lakshana packets for overlap arithmetic. The current supported roster has no native Lakshana wearer in this Arc group.

## Risks and Limits

- The release description and older client strings diverge. Document the evidence and preserve the frozen parameter rows as source data. Exact stack refresh behavior and timer ordering remain modeled.
- Akane Rin's character kit is outside this equipment change.
