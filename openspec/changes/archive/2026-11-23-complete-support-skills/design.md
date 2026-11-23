# Design

## Context

The switch engine already owns Cycle reaction qualification and settlement. The current owner entry hook can select one attack, one Support, or a field effect with optional Support. A separate fallback in `characters/support_dispatch.rs` bypasses reaction qualification for three owners. Support coefficient projection reads `purchased_skill_level`, which script compilation fills from the Skill coordinate of the talent tuple.

## Goals

- Share one switch qualification and action admission across native requests, script execution and the WASM Worker.
- Capture each owner's complete Support action and show its effect, source and level in the existing reporting paths.
- Keep authored inputs, automatic actions and later hit or heal delivery on the same combat-time axis.

## Decisions

### Use the switch's reaction for Support admission

Resolve the locked target and qualifying reaction against settled pre-switch state. Keep this decision with the admitted switch so Cycle spending and field callbacks cannot cause the same qualification to be evaluated on a different state. Commit the switch and its reaction once. Use the resulting qualification for the incoming owner's Support action, including any owner admission guards. Reuse the same read-only qualification and guards for `.support.ready` on its applicable target. Keep the reaction and switch independent of an ordinary incoming action rejection, and expose reached unsupported effects through the existing incomplete-result diagnostics. A failed Support admission leaves no partial Support packets, resource effects or observers.

Delete the legacy fallback dispatcher, its live cooldown entries and the runtime use of its model. Build new qualified owner bindings for Sakiri, Nanally and Zero on the same path as the other characters. Reuse current typed packet and effect data where verified against the owner dossier. Update any provenance attached to the superseded dispatch assumptions.

Represent Lacrimosa A6 as an owner-specific alternative to the outgoing Cycle payment on a qualifying Nova or Scorch switch. Arm the five-second window at each committed Devilish Gift. On the earned entry, consume it and start the separate five-second A6 internal cooldown. A return can use the free entry only while the cast window is open and the cooldown has expired. Standard reactions retain their usual qualification during that cooldown. Both forms of qualification share the same target, reaction, Discord and Support settlement. Use the settled world clock for both A6 timers.

### Compose automatic actions using their actual boundaries

Represent an earned owner entry effect and eligible Support as parts of the same switch. Remove unconditional field-entry Normal actions, including the Haniel, Hathor and Edgar fallbacks. Keep the existing conditions on earned entry mechanics such as Baicang's JumpZ. Check owner selectors individually for other conflicts.

For Daffodil, capture the pending regular Phantom Step when both actions are available and admit it at the QTE animation-completion transition. The user observed QTE play to the end with Phantom Step starting immediately afterward. Use the owner action clock and established hit-pause rules for the completion marker. A first hit, generic input window or guessed fixed delay would place the handoff on the wrong boundary. When a numeric completion boundary needs a modeled value, record its evidence and assumption in the owner dossier.

Link the pending follow-up to the actor and originating action so defeat, stale generations and interrupted actions cannot deliver it. Apply the Step's normal admission and charge rules at the handoff. Check the existing explicitly authored parry path against its own charge and finisher rules. Preserve one settlement per charge and record the regular Step independently of any QTE parry finisher. Keep subsequent authored commands subject to the real recovery of the automatic actions.

### Keep the QTE level independent throughout projection

Parse and validate the required `qte` field with the existing character declaration validator. Carry it as a distinct purchased level through the shared actor configuration, native scenario export and import, script inspection and Studio transport. A raw native request needs explicit level data when it reaches an automatic QTE. A missing value yields a diagnostic at the relevant boundary.

Use the owner's Support slot and progression table to derive the effective level from the purchased value, selected awakening count, the owner's Resonance 3 mapping and the supported level cap. Keep purchase validation separate from the effective-level cap. Reuse this projection for each QTE-scaled damage and heal component. Independently scaled effects keep their own inputs. Nanally's surviving Underboss follow-up uses its captured Burst talent, while Iroi's QTE damage and heal use her QTE talent at their distinct owner events.

### Complete the QTE payload per owner

Use the registered playable roster as the scope. Bind each QTE to its complete owner action rather than using a generic damage-only projection. Reuse owner action clocks, exact packet rows, healing and resource services, targeting, proc hooks and category metadata. Review maturity against the existing character dossiers and frozen sources. Keep added typed values consumed by Rust and document any timing or delivery assumptions where the source is incomplete.

Preserve the existing semantics for effects that observe actual casts, including Edgar's allied-QTE keys, supported weapon procs and Hotori's recorded-action replay. A replayed QTE keeps the original owner's action and build and uses its virtual cast context without a new switch reaction or outgoing Cycle payment.

## Risks / Trade-offs

- Script timing and resource assumptions can change when a formerly free QTE disappears, a newly qualified QTE adds recovery or Phantom Step begins later. Run bundled scripts against the current engine and fix waits and switch choices using the actual owner windows and live state.
- Packet arrays alone establish neither complete execution nor a heal or its recipients. Check the owner clock and effect routes independently, with exact tests for damage, healing, resources and any reached secondary effects.
- Studio's Team list can carry a prior calculation identity after scripts migrate. Keep the recorded identities truthful under the existing explicit refresh policy.

## Migration Plan

1. Land the Rust configuration and combat changes with the current native and WASM contracts, TypeScript consumers and maintained scripts migrated together.
2. Set every Team list actor to `qte=8`, including S5. Put explicit legal levels in other maintained scripts and fixtures and update examples in the DSL, mechanics and preset documentation.
3. Regenerate the normal preset bundle through the existing tooling and verify exact loading, executable bundled rotations and representative native/Worker parity.
