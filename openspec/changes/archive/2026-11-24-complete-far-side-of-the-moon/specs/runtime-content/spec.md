## ADDED Requirements

### Requirement: Far Side of the Moon Gravitide

Far Side of the Moon SHALL add its refinement Crit Rate to the wearer's base Crit Rate. Delivered Basic Attack packets with positive HP loss SHALL grant one Redirect Skill stack for five world seconds, up to four stacks with a 300ms acquisition gate. Qualified hits SHALL refresh the shared deadline. Admitted wearer Redirect Skills SHALL refresh a 15-world-second Basic Attack window before hit delivery. During overlap of both effects, the wearer SHALL gain the refinement Lakshana bonus.

#### Scenario: Accumulate bounded stacks from damage
- **GIVEN** a wearer with no active stacks
- **WHEN** Basic Attack packets damage one or several targets in the same instant and then land again at eligible times
- **THEN** the first instant grants one stack and each eligible later instant increases it up to four, and a qualified hit at cap renews the shared lifetime
- **AND** non-Basic packets, zero HP loss and another actor's hits do not grant stacks

#### Scenario: Combine independent effects on delivery
- **WHEN** a Redirect Skill is admitted while stacks are active and subsequent damage lands inside and outside either effect
- **THEN** Redirect Skill packets gain the current per-stack bonus and Basic Attack packets gain the active cast-window bonus
- **AND** Lakshana packets gain the element bonus only during overlap, additively with any matching category bonus
- **AND** expiry precedes equal-time delivery, switching preserves both effects, and rejected casts do not open or refresh the cast window

#### Scenario: Use the same refinement and timing in CLI and Studio
- **WHEN** a script equips the Arc on a compatible wearer and executes continuing actions in native or WASM
- **THEN** both paths report the same refinement values, stack and window transitions, and settled damage
- **AND** world-time pauses extend live effect and acquisition deadlines without granting extra stacks
