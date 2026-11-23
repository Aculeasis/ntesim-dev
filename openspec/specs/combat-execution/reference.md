# Partial game runner contract

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

CLI: run-game SCENARIO.json. Model/validation/execution live in crates/content/src/simulation/. [Example](../../../examples/game-scenario.json), [checkpoint scope](../../README.md).

The runner executes configured casts, switches and waits with exact damage/resource receipts. Native live-source admission and its exported replay assess the selected model from declared and reached assumptions. Canonical implemented hypotheses and source-bound static build projection yield `bounded_hypothesis`; unknown/mismatched assumptions or caller metadata overrides yield `unbounded_hypothesis`. Scheduled research fixtures retain `incomplete` quality. A completed bounded source execution with positive relative elapsed time exposes scenario-conditioned DPS (`game_dps_computed=true`); failed/incomplete runs do not.

## Request boundary

Schema 1 and the initial package identity are supported with required `clock_contract="relative-combat-v1"` and `timer_policy="paused-world-v1"`. Requests declare seed, `relative_duration_us`, independent `absolute_limit_us`, active actor, actors, targets, actions and optional explicit research critical policy. `stop` defaults to `time`, completing at the relative target or all-target defeat; `victory` ignores the authored target and ends at all-target defeat or an incomplete safety stop. Reports retain both bounds and actual `relative_elapsed_us` / `absolute_elapsed_us`. Missing or obsolete clock contracts reject; no adapter reconstructs missing pauses.

The absolute scheduler observes actual generation-owned pauses and recomputes its provisional endpoint when a pause opens, releases or changes kind. At observation A, R + P = A, where P is the clipped union of half-open pause intervals. Independently, `[1,4)` and `[2,5)` have union `[1,5)` of length four; at A=7, R=3. A 120-relative-second run with twenty seconds of actual pause ends at absolute 140 seconds. Already scheduled endpoint work and equal-time children remain included; new endpoint input cannot open another pause. Later damage remains pending.

Detailed sample `timer_pauses` retain owner/generation, kind and absolute start/end; `timer_bindings` interns each reached owner's release provenance. Compact batches retain elapsed scalars without interval history. Ordinary world timers, object lifetime and finite action contexts compose the declared policy in the [clock reference](../../../docs/mechanics/shared/clocks-and-action-timing.md), separately from local hitlag. Executed live source exports retain `source_exhausted` with `after_settled_events` admission to distinguish valid autonomous continuation from truncated requested input. Pending-event counts alone do not define completion.

Typed `action_admission_order` distinguishes ordinary `scheduled` inputs
(the default) from `after_settled_events` inputs exported by native live source.
The latter use the shared Observation boundary so due hits/callbacks/death
settle before an authored same-time input. This field is retained for native
replay; no timestamp offset, retry or second combat engine is introduced.
See [sequential admission replay](../../../docs/decisions/0017-sequential-admission-replay.md).

Executed source exports retain optional typed `source_halt` for evaluation,
work/action-budget failures and unresolved source waits. It binds the real
settled time, failed/pending classification, diagnostic range and occurrence
context. Native replay executes the prefix on the same seed, then stops without
draining later combat effects or inventing another input. Invalid boundaries,
exhausted-source contradictions and mixed live-program wrappers reject.
Ordinary admission failures remain native rejected commands. An `event_limit`
halt additionally binds the actual event cap. Replay reruns that cap and checks
the resulting limit, time and diagnostic; it never copies a reported combat
state. A stricter caller limit cannot be bypassed. The cap can interrupt an
input or the autonomous drain after lexical exhaustion.

Actors supply canonical character/weapon/set identities, level/awakening/skill fields, static stats, initial/max HP, initial/capacity energy and supported explicit gear procs. These fields are not complete native legal builds or chosen-effect awakening contracts.

Targets supply explicit level, current/max HP, defense, common/elemental resistance and required mechanic assumptions, without position or enemy lookups. Actions include InvokeAbility, SwitchActor, Wait and typed player inputs. Ability commands currently supply ability/effect identity, curve array index, ATK ratio, cost, recovery/cooldown/hit delay and timestamp. Native content owners expand packet trains, resources, passives and child events; source-action integration remains tracked separately in native workflow.

Preflight checks schema/mode/package, 1–4 actors and at least one anonymous target (target count bounded by `max_events`, not party size), active/switch/actor indexes, monotonic command times, horizon/event limits, capacities and research policy validity. Research crit requires research mode, nonempty hypothesis/eligibility, matching configured actor streams and representable rate. It never establishes actual game eligibility/shared rolls.

Optional `dummy_combat_events` declare anonymous-dummy `attack_start` observations
(timestamp, target and party-recipient indices) and `proximity` eligibility changes
(timestamp, target and boolean). They require a supported content observer and
ordered in-horizon declarations, never schedule damage, and preserve declaration
indices in trace/replay. Expiry precedes an equal-time attack start; proximity
changes precede accumulation completion. See the [shared observation boundary](../../../docs/mechanics/shared/incoming-damage-harness.md#explicit-dummy-combat-observations)
and Hathor's selected Gaze models. Optional `incoming_dummy_sources` binds incoming
event indices to anonymous dummy instigators; actual HP loss/shield contact can
consume the reached owner's enhanced mark without creating an attack-start or
another damage settlement. `incoming_damage_basis` optionally selects the explicit
`before_defense` fixture stage for an event, requiring its dummy source/level and
`combat.incoming_defense_v1`; omission or `externally_mitigated` preserves the final
declared amount. The root retains pre-DEF/sample provenance in `defense_mitigation`
while `delivered_damage` remains the conserved amount passed to shares/shields/HP.
Public source lowering uses supported owner profiles and diagnoses unsupported reached inputs.

### Required dummy target semantics

The [product target model](../product-scope/reference.md#dps-relevance-and-modeling-boundary) is one or more stationary dummies with combat stats, all in range and inside attack areas. Uncapped AoE hits all eligible dummies; capped attacks select at most their owner-defined recipient limit per hit, and single-target attacks retain one recipient. Owner selection, damage distribution, target-count effects and delivery timing remain required. Native radii, shapes, placement and movement are outside scope. Unknown recipient limits or selection/distribution rules must remain visible rather than silently granting every dummy damage. Structured `position_mm` and source `pos` inputs reject under the [implemented dummy boundary](../../../docs/mechanics/shared/dummy-targets.md); source aim-target selection and manual Break/gauge integration remain in Q4-L3.

## Execution and checked arithmetic

Queue order/time/sequence is deterministic. Configured field ownership, energy costs, locks and cooldowns govern admission; reached partial callbacks/hits update exact damage and ledgers. Invariants are checked initially and after causal transitions. These are implementation checks, not proof of continuous native state between events.

The current runner pops the next queued event and assigns its timestamp directly to simulation time; it does not iterate through empty frames or microseconds. Explicit resource/state waits retain this event-driven approach; ordinary failed action admission halts without automatic retries; [advancement rules](../script-execution/reference.md#event-driven-time-advancement) own wake-up and sparse-gap acceptance checks.

Shared capped spending/grants, recipient efficiency/carry and conservation are owned by [energy accounting](../../../docs/mechanics/shared/energy-accounting.md). This interface records their settlement; missing native producer policies stay explicit.

Grant/restoration trace events include either `energy_change` (for single-actor/direct grants and admitted casts) or `energy_fanout` (for multi-actor hit-driven team grants): typed source, source actor, recipient actor(s), exact unscaled `charge_basis` for efficiency-scaled grants, energy before/after, fractional carry before/after, gross whole pool units, retained units and per-event overflow. Grouped `energy_fanout` hoists shared `source`, `source_actor_index`, and optional `base` out of per-recipient `recipients[]` (each retaining its `recipient_actor_index`, optional `distribution`, pool/carry transitions, and `charge_basis`), while single-actor hit grants keep `energy_change` with optional `hit_basis`. Flat restorations omit `charge_basis` and preserve carry; Rust checks their source-specific amounts against the [Adler](../../../docs/mechanics/characters/adler/action-damage.md) and [Sakiri](../../../docs/mechanics/characters/sakiri/action-damage.md) owner rules. Rust also enforces fixed charge bases and awakening gates from the [Zero](../../../docs/mechanics/characters/zero/action-damage.md), [Nanally](../../../docs/mechanics/characters/nanally/action-damage.md) and [Mint](../../../docs/mechanics/characters/mint/action-damage.md) owners. Zero carry may be omitted and means exact `0/1`. Whole units follow the existing efficiency/fractional carry model; starting energy is excluded. Mind Royale, Mint A6, Zero A2, Nanally A1, Sakiri A5, Adler A5, Rishi Eclipse and supported ordinary-hit team sources use these shared receipts; supported ordinary-hit and explicitly modeled direct-packet team grants carry exact `base`/`distribution` (`hit_basis` or `energy_fanout.base` + `recipient.distribution`) provenance, which remains available alongside `charge_basis`. Rust settles each charge basis against recipient efficiency and prior carry; browser aggregation, replay and JSON exports consume the recorded recipient(s) and carry, even off-field, and reconcile recorded transitions with each final pool without recomputing owner rules. Trace events are pure language-neutral data (`hit_index`, `from_actor`, `to_actor`, typed `data` payloads without `detail` prose or duplicate `damage_dealt` on `hit_damage`), and UI localization formats presentation strings strictly from structured receipts. Missing/corrupt receipts reject observations before aggregation. Admitted costs retain typed admission receipts and preserve carry. Remaining energy-source eligibility stays with owners; implemented team generation is checked per owner. [Ledger and parity checks](../../../crates/content/src/simulation/tests/energy_observations.rs), [direct-source provenance rejection](../../../crates/content/src/simulation/tests/direct_packet_energy.rs) and [direct-source policy checks](../../../crates/content/src/simulation/tests/adler_energy.rs) cover the public receipts and malformed-ledger boundaries.

Preflight/runtime deadline additions are checked for action recovery/cooldown/hit delays, switches, Mind Royale cooldown, Fireflies delivery-plus-expiry and reached Mint/Sakiri/Iroi buff hooks. Exact u64 endpoints are representable; overflow and invalid indexes return errors. Unreached hooks do not impose blanket buff-duration constraints.

Configured effects are owned by character/weapon/set modules through typed content events. No implicit gameplay semantics may be inferred from a working callback. Crit defaults to non-critical settlement; explicit research policy alone consumes its declared RNG streams.

## Explicit HP consumption

Research scenarios may declare `hp_cost_events: [{timestamp_us, actor_index, source, amount}]` under accepted `combat.hp_cost_fixture_v1`. The integer amount is an explicit scenario input, separate from incoming damage and ability admission. Events must be ordered, inside the horizon, within the event budget, and reference a configured actor and a nonempty source of at most 256 bytes. They do not shift with live action waits. No DSL syntax or native ability cost is inferred.

Each event consumes at most current HP minus one; zero/dead/clamped-zero payments grant nothing. Consumption bypasses shields, incoming-damage absorption and damage-received callbacks. At equal time, prequeued incoming damage settles first, followed by HP costs in authored order, then newly queued admitted hit deliveries. Positive payments dispatch the [Silent Garden](../../../docs/mechanics/weapons/jingmotingyuan/passive.md) consumption callback before those hits. This ordering and nonlethal floor are explicit fixture hypotheses; owner adapters must separately supply native cost, affordability and timing policies.

Reports retain `hp_cost_receipts` with event index, combat time, actor, source, requested HP, HP before, actual HP loss and HP after, plus `actor_hp_cost` trace entries. Native, WASM binary/JSON and Worker consumers preserve these fields; aggregation validates their declaration identity. The supported cost callback does not promote whole-kit or full weapon readiness.

## Waiting and idle diagnostics

The shared executor owns waiting/idle classification and diagnostics; the DSL supplies player inputs and explicit waits, not warning commands or idle thresholds. Native reports expose `input_time`: exclusive half-open intervals and exact totals by actor/cause, reconciling to `absolute_elapsed_us`. Native owner recovery (including entry recovery and changing hitlag/interruption deadlines) takes precedence over authored waits; sparse gaps add no scheduler events. The ledger distinguishes action/switch recovery, `wait`, `wait_until`, energy gates, gaps before scheduled inputs and script exhaustion. Resolved `input_time_contexts` retain actual source continuation boundaries, occurrence identities and complete source ranges in exported native requests; these annotations never schedule combat work or grant readiness. Energy intervals retain the threshold and settled available energy, splitting when that state changes. Native replay retains terminal source failures and unresolved waits; complete Worker presentation/parity follows in browser integration.

Native scheduled/program waits are clipped at the relative time endpoint. A
final ordinary wait or selected branch wait extending beyond it still completes
the time-limited run successfully; later source inputs are not evaluated. Program suffix timestamps are authored after the
`wait_if_below_us` baseline; reached branch deltas and actual energy-wait delays
shift both queued inputs and Wait annotations. The time ledger records those
resolved waits, preserving native recovery precedence. All in-horizon effects
settle before a pending gate reports the horizon as elapsed time. Failed
admission and event-budget stops retain their actual earlier settled prefix;
victory keeps its early all-target-defeat boundary.

- Collect real combat-time intervals for authored `wait`, `wait_until`, explicit resource/state gates and script exhaustion. Failed ordinary admission produces a failure receipt at that timestamp, not an automatically inserted interval. A change of blocking cause splits the interval; overlapping requirements never double-count elapsed time. A false `if ...ready` branch does not itself create a blocked-input warning.
- Reports retain actor, awaited action, source command/location where available, start/end/duration, reason and the relevant missing-resource/cooldown/state information. Aggregate lost input time by cause and actor without replacing exact intervals. No-input time can include continuing off-field damage; it is not zero combat activity.
- `input_time.warnings` classifies a contiguous same-actor/same-cause no-input episode lasting at least 5,000,000us as prolonged. Source occurrence or energy snapshot boundaries do not restart that clock. Coalesce qualifying episodes into one warning per actor/cause, retaining episode count, total/longest duration and first start. Four actors and five idle causes bound output to 20 warnings regardless of scheduler work. Deliberate waits and exhaustion remain distinct classifications, without an optimality judgment. Action/switch recovery and channels never produce these warnings; failed admission remains an instantaneous receipt.
- A switch prohibited during Hotori Time Stop fails immediately. Authored waits can allow eligible continuing effects and an early finisher exit; preserve their elapsed time. A horizon reached with an unresolved explicit gate is incomplete execution and cannot enter successful aggregates.
- Source diagnostics distinguish typed admission, arithmetic, work/input/event limits, unresolved energy gates and independent safety stops; `failure_context` retains the actual dynamic occurrence and rejected native command, including implicit switches. Failure consumes no invented retry time. Native/CLI/WASM exports retain these receipts; complete Worker presentation/parity follows in browser integration. Checks distinguish empty-energy Ultimate failure, cooldown failure, state-prohibited switch, deliberate wait, script exhaustion and actual long idle from a long cast/channel/recovery.

## Reports, failures and horizon

Reports expose execution success/completion separately from quality: hit receipts, exact total damage, event counts/trace, final actor/target state, pending events, halt reason, invariants, seed/RNG policy/state and engine identity.

The live-program API accepts a separate typed wrapper around a stable scenario request. Each live HP or active-actor-energy branch reports an Observation-barrier receipt with its observed value, threshold, selected and baseline waits, action boundary and source location. Ordered live energy waits report their caller threshold, source-selected actor, settled energy at start/resume, action boundary and source location; an unresolved wait makes execution incomplete. The selected branch wait or actual accumulated energy delay shifts only the later action schedule; action indices and packet provenance remain stable for rejection, damage and energy validation. Energy threshold is not evidence of native Burst cost or admission readiness. `run-game-program` accepts this wrapper through the same content runtime; it does not parse NTE source.

Browser validation settles each packet's exact damage under the [shared quantization policy](../../../docs/mechanics/shared/damage-and-stat-zones.md#6-implementation-and-verification-boundaries), caps applied HP loss to the target's remaining HP, and reconciles every final target HP/capacity with the receipt ledger. Unsafe-sized packet integers retain their lossless string representation. This validates implemented settlement consistency, not native rounding evidence.

Structural MaxHP changes serialize in `structural_hp_changes`, with source actor, actor/target recipient, cause, time and before/after MaxHP/currentHP. Membership, base-stat changes, extraction and rollback are not hit/heal receipts. CurrentHP clamps contribute no damage, resource grant or combat-loss callback; reversal restores the stat without restoring HP.

Every primary and owner child hit retains its real origin, including delayed callbacks. Authored inputs retain `command_index`; autonomous owner activations instead retain `autonomous_event_index` into `autonomous_events`, with exactly one origin on newly produced hits and controls. The ledger records owner, active recipient, time, resolved ability or entry-effect class in `ability_id`, and the actual initial/field-entry/action/HP/MaxHP/deadline cause; coordinated activations retain both their controller cast and an earlier positive applied-HP hit by the active ally; it never inserts a synthetic input. Children and Break contributions retain this origin independently of damage attribution. An autonomous healing settlement records captured recipient, exact live-stat basis, gross/applied/excess amounts and HP before/after. Old authored JSON remains unchanged. See [ADR 0014](../../../docs/decisions/0014-autonomous-owner-event-identity.md). Character-specific child packet eligibility and hypothesis rules belong to [character dossiers](../../../docs/mechanics/characters/README.md), including [Zero](../../../docs/mechanics/characters/zero/action-damage.md). Reached model assumptions retain exact identity and description in reports; a linked saved receipt is not a present validation claim.

Normalized provenance and receipt catalogs deduplicate static metadata across hits and commands: `evidence_catalog` stores unique `NativeEvidence` entries, `metadata_catalog` stores unique normalized `(action, packet, binding)` metadata templates (referencing evidence by `evidence_id` and hypotheses by `hypothesis_id` into `resolved_model_assumptions`), `hit_receipts` and `action_metadata_ledger` reference `metadata_id`, optional execution-local `energy_group_id`, and `execution_model_assumption_ids`, `player_input_receipts` retain `execution_model_assumption_ids`, and identical static entries in `action_metadata_ledger` and `talent_projection_receipts` group their initiating commands into `command_indices`.

Exact fractional owner resources serialize separately as `fractional_resources`, keyed by canonical resource ID. Each ledger exposes exact capacity, initial/current/granted/spent/overflow and conservation; it never rounds owner units or substitutes Charge/Cycle carry. Empty maps are omitted. Rust performs resource arithmetic and validation; Studio presentation follows browser integration.

`state_timeline` retains complete `Initial`/`Final` boundary snapshots and sparse changed-field `GameActorStateDelta` records (omitting unchanged keys, using explicit JSON `null` for cleared `charge_remainder`/`cycle` and empty `[]`/`{}` for cleared shield/resource collections), changed target slots, stamina changes, and `buff_diff` (`upsert`/`remove` over positive monotonic `instance_id` references into `buff_catalog`) on intermediate `Event` frames. Equal-time event ordinals
and exclusive `hits_end`, `healing_end`, `trace_end` join observations with their
typed settlement receipts. Buff/status/shield expiry fields remain actual
deadlines even when removal is observed after an idle gap. `validate_state_timeline` folds the entire catalog/delta sequence from `Initial` and checks final/idle-gap reconciliation. Frames do not expose
every transient internal callback state and are not resumable scheduler state. `trace` records pure language-neutral `GameEventReceipt` entries (`time_us`, `phase`, `event_kind`, `active_actor`, optional `hit_index`, `from_actor`, `to_actor`, `energy_change` or `energy_fanout`, and typed `data: GameEventPayload`), omitting localized/prose `detail` strings and duplicate hit damage.
`healing_receipts` preserve exact bases/bonuses, source/recipient and gross,
applied/excess HP, including zero-application settlements. Each hit captures
`active_actor_index` at contact; delayed damage keeps its original owner.
Rust `damage_analysis()` groups actual actor/target/ability/effect/field damage,
reconciling checked integer sums of per-packet rounded damage, applied HP loss
and overkill. Primary Damage includes overkill, while applied HP loss is capped
by the target's remaining HP. The exact total is an integer rational `N/1`;
raw per-hit formula fractions remain diagnostic. Two 100.4 hits give 200 Damage,
two 100.5 hits give 202, and a 100 hit against 30 HP gives 100 Damage, 30 HP loss
and 70 overkill. Contributions, bins and both DPS bases use this common total;
DPS itself remains fractional.
Exact canonical DPS uses actual `relative_elapsed_us` only for assessed completed runs; zero relative duration leaves the rate unavailable. Partial
contributions remain visible without DPS. See [ADR 0019](../../../docs/decisions/0019-causal-state-observations.md).

For report summaries, native rates pair relative and absolute bases calculated per trial from settled totals over that trial's positive elapsed duration $T_b$:
- $\text{DPS}_b = \text{settled damage} / T_b$
- $\text{EPS}_b = \text{retained energy} / T_b$
- $\text{granted EPS}_b = \text{granted energy} / T_b$
- $\text{HPS}_b = \text{gross healing} / T_b$

Native available shield HP (SHP) pairs relative and absolute averages by integrating remaining absorbable shield capacity $C(a)$ on the active living actor:
- $\text{SHP}_{\text{absolute}} = \frac{\int C(a)\,da}{\text{absolute\_elapsed}}$
- $\text{SHP}_{\text{relative}} = \frac{\int C(a)\,dR(a)}{\text{relative\_elapsed}}$

Relative SHP strictly distinguishes clock weighting from denominator scaling: during a timer pause interval, the relative clock delta $dR(a) = 0$, so shield capacity active only during pauses accumulates zero relative HP-microseconds. Dividing the absolute capacity integral by relative duration is mathematically invalid and rejected. When a duration is zero, the corresponding rate or SHP value is unavailable (`null`); a defined absolute rate does not restore relative comparison eligibility or make unsupported damage valid. Canonical team rankings, comparison eligibility and research evaluation remain strictly canonical relative DPS.

An owner-captured full damage-base replacement appears on its hit as optional
`damage_base_override`. It replaces the entire stat/coefficient product before
live damage, DEF/RES and critical zones. `scaling_stat`/`scaling_rate` and native
metadata still identify the source fallback and must not be multiplied into
that replacement again. Ordinary hits omit the field. Chiz's admitted Coin1
is the current resource-derived consumer; Rust captures and floors the amount,
and clients retain the receipt without reconstructing its financial rules.

Compounding resource profiles may use the bounded wide kernel ledger. Its `WideFraction` quantities—alongside `Ratio` (`u64`) and `ExactFraction` (`u128`) in requests, buffs, stamina, damage/energy receipts, and reports—serialize and deserialize canonically as reduced `"<numerator>/<denominator>"` decimal strings (rejecting `{numerator, denominator}` objects and invalid or zero denominators), retaining arbitrary numerator/denominator digits without rounding. External raw-export `Scalar` fields (`character-stats`, `weapon-stats`, `build` raw projection inputs, `unmodeled_attributes`) and research comparison deltas (`SignedFraction`) retain their `{"numerator": "<str>", "denominator": "<str>"}` object representation. `ResourcePoolSummary` preserves exact strings through native JSON, WASM binary/JSON, and replay. Clients must parse and retain string components via `BigInt` before presentation formatting; no floating-point resource arithmetic or UI reconstruction is authorized by this transport extension.

An owner may expose separate conserved ledgers for savings and drawn debt.
For admitted Chiz loans, `chiz.grain.spent` counts own Grain consumption;
`chiz.loan_debt.granted` counts principal actually drawn and `.spent` counts
principal repaid. Debt capacity is the explicit credit limit, and current is
outstanding principal. These are separate from Ultimate Energy, Cycle, stocks
and Fons; clients must not infer income or clear debt at a period boundary.

`hold_inputs` may include optional `reserved_stocks` only under an owner that
explicitly supports that choice. Chiz accepts2/3 on a positive Skill hold;
three requires the selected held-Full model. Other owners reject the option.
Omission retains the existing owner default and wire representation. This is
a paid reservation, not a caller-selected damage stage or free extra cast.
A later actual Chiz Skill command may instead reserve the third stock through
its owner gates; all delayed Full packets retain that later command origin.

Admission/invariant/event-limit halts retain available structured reports; preflight or arithmetic Result errors may have no returned partial report. Consumers must retain the actual diagnostic and withhold successful aggregates. Events after horizon are reported pending; remaining boundary semantics are tracked in the [OpenSpec changes](../../README.md).

A queued event after the horizon retains `SCENARIO_HORIZON_PENDING` with its timestamp and the horizon boundary, unless a more specific halt reason already exists. A live energy gate starting after the horizon retains `LIVE_ENERGY_WAIT_PENDING`; an already active energy wait also keeps its own pending-wait reason when another queued event exceeds the horizon. These reports remain incomplete and preserve their settled prefix.

Checks live beside content simulation and CLI tests. `tools/verify_game_runner_cli.py` independently checks fixture fractions/state/replay; `tools/verify_energy_ledger.py` reconciles typed ordinary-hit, cast and weapon grants with capped balances, overflow and replay. Independent Mint grant amounts remain in Rust owner/energy observation tests; the CLI check must include ordinary-hit grants rather than treating the A6 cast grant as the entire Skill income. Strict scenario closure and complete shared gameplay remain in the [OpenSpec changes](../../README.md).
