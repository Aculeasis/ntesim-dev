# Design

## Context

Scorch creates and refreshes its target carrier in one shared path. A paid Esper Cycle switch already distinguishes the outgoing actor that spends Cycle from the participant whose CI and damage attribution the reaction captures.

## Decisions

- Sample Temperance once per successful Scorch application or refresh with its own seeded RNG stream. Use source-ordered candidates, one refreshable timed target status per outcome, and the existing target status cleanup and world clock. Living B2 Adler may be off field when another actor applies Scorch.
- Consume the reductions in current resistance and Break calculations. Report authored dummy ATK with the live percent reduction against original ATK. Haniel's direct extraction ledger remains separate. Explicit incoming damage is still authored independently of dummy ATK.
- Use the outgoing Cycle spender for Zero's R6 trigger. The damage snapshot's higher-CI attribution remains independent. Use the frozen 12% and 20s as a base-ATK percentage for each living party member, including Zero, with a cap-one refresh and world expiry. The trigger and lifetime are declared interpretations of the frozen description.

## Risks / Trade-offs

Native outcome odds and the exact Zero trigger callback are unavailable in the recovered data. The model records uniform seeded Temperance selection and outgoing-spender attribution in the reached provenance.
