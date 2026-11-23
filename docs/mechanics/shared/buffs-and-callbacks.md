# Buff durations, ICD, and callback dispatch contract

Ordinary buff/shield/debuff lifetimes, stack decay and proc ICDs use the shared paused-world clock. Both cinematic and active-stop pauses preserve exact remaining duration, even for effects applied inside a pause. Admission, live queries and expiry delivery use that same basis. Explicit source removal is immediate; no frozen interval farms an ICD or queues catch-up applications. Absolute receipt timestamps still identify actual causality. See [clock composition](clocks-and-action-timing.md).


This contract defines the reusable buff and callback services in `ntesim-core::dispatch`. Their harness checks do not establish complete gameplay integration tracked in the [OpenSpec changes](../../../openspec/README.md).

## 1. Buff recipients and targeting

Buffs specify an explicit recipient model determining which entity benefits from the modifier:

- `BuffRecipient::SelfActor`: applies only to the buff's owner character (`actor_index == owner_actor`).
- `BuffRecipient::ActiveActor`: applies dynamically to whichever character is currently on-field (`actor_index == active_actor`). When an on-field switch occurs, the buff automatically transfers its stat bonuses to the incoming character without re-application or timer reset.
- `BuffRecipient::TeamAll`: applies simultaneously to all actors currently in the team roster.
- `BuffRecipient::SpecificActor(usize)`: targets a designated actor slot.
- `BuffRecipient::SpecificTarget(String)`: debuff applied to an external combat target entity (e.g. boss or minion).

Stat modifiers across all active buffs are aggregated on demand using exact rational arithmetic (`Ratio { numerator, denominator }`) with greatest common divisor (`gcd_u128`) canonical reduction to eliminate numerical drift.

## 2. Duration, refresh policies, and stack caps

### Durations

- `BuffDuration::Permanent`: remains active indefinitely until explicitly cleared.
- `BuffDuration::Timed { duration_us, expires_at_us }`: expires deterministically when game time `now_us >= expires_at_us`.

### Refresh policies
When a buff is re-applied while an instance is already active:

- `RefreshPolicy::Reset`: increments stacks up to `max_stacks` and resets the timer to `now_us + duration_us`.
- `RefreshPolicy::Extend { max_duration_us }`: increments stacks and extends `expires_at_us` by `duration_us`, capped by optional `now_us + max_duration_us`.
- `RefreshPolicy::Independent`: maintains independent stack timers.

Each independent stack retains its own expiry; a capped application does not refresh existing stacks. One-by-one decay advances from the scheduled expiry, rather than the caller's polling timestamp, and settles all elapsed expiries in a sparse time advance.

- `RefreshPolicy::Disallow`: denies re-application or timer extension while active.

### Identity and application snapshots

`BuffStackingScope::PerSource` (the serialization default) keys applications by ID, recipient, source and actor. `Shared` keys by ID and recipient across carriers. Reset and Extend capture the latest application values and source ownership; Disallow retains the original values. Reapplications sharing a key must agree on scope, refresh/decay policy, stack cap, permanent/timed mode and modifier channels; one-by-one decay also requires the same decay interval. Invalid definitions or deadline overflow leave the complete manager unchanged, including expiry state. Timed durations must be positive.

Content team-buff and target-status producers use this manager with explicit actor/target recipients. Their owner adapters supply amounts and eligibility; source-specific marker queries stay in their character owner. Harness correctness does not certify missing owner triggers or full game integration.

### Decay policies
When a timed buff expires:

- `DecayPolicy::AllAtOnce`: all stacks immediately expire and the buff is removed.
- `DecayPolicy::OneByOne`: if stacks > 1, decrements stacks by 1 and extends duration for the next stack, preserving gradual ramp-down mechanics.

## 3. Internal Cooldown (ICD) rules and scopes

To prevent multi-hit abilities from triggering secondary effects on every damage instance, effects specify an ICD entry.

### ICD rules

- `IcdRule::None`: no cooldown restriction; triggers on every valid matching event.
- `IcdRule::TimerOnly { window_us }`: enforces a minimum time interval between triggers (`now_us >= last_trigger_us + window_us`).
- `IcdRule::Standard3Hit2_5s`: standard action RPG ICD:
  - First hit triggers and initializes a 2.5s window (`window_start_us = now_us`).
  - Subsequent hits within the window are suppressed until either:
    1. Hit count reaches 3 (`hits_since_trigger >= 3`), or
    2. Elapsed time reaches 2.5s (`now_us >= window_start_us + 2_500_000`).
  - Upon triggering, the window and hit counter reset.
- `IcdRule::HitCountOnly { hit_interval }`: triggers strictly once every $N$ hits.

### ICD scopes

Timer eligibility uses checked elapsed time, so a deadline beyond representable time cannot fire at the maximum timestamp. ICD keys escape arbitrary callback/target/group identities; shared groups must register compatible rules.

Every matching event increments its ICD hit counter, including suppressed events. Eligibility and trigger commitment are separate; counting only eligible events prevents hit-count ICD from ever progressing.
Read-only eligibility predicts the next matching event by evaluating the same transition on a copy; it cannot use a separate pre-increment threshold. A zero hit interval or reversed entry time returns an explicit error without mutation, including direct entry/tracker calls. Runtime ICD state cannot be forged through public fields or deserialization.

- `IcdScope::Global`: ICD is tracked globally across all combat participants and targets.
- `IcdScope::PerTarget(target_id)`: ICD is tracked independently per enemy target ID for hit, damage-received and target-death events, allowing different targets to proc independently.
- `IcdScope::SharedGroup(group_id)`: multiple abilities sharing a group ID share a single cooldown counter.

## 4. Callback / proc dispatch and recursion guards

An extra-attack `multiplier_percent` scales the triggering damage receipt using integer floor arithmetic. A hit or damage-received basis is required; the percentage cannot become an absolute damage value. Energy grants require a registered pool and report applied energy and overflow separately. Missing pool state is an error.

Switch commands and emitted `ActorSwitched` events share one dispatch transition: the declared source must match the current field actor, field ownership changes before callbacks, and a failed callback restores ownership/time/resources. Construction validates team and active actor identity in the runtime, including direct API callers. Runtime fields expose read-only views; callbacks enter only through checked registration. Input definitions reject unknown fields rather than discarding misspelled mechanics.

### Combat events
Combat interactions emit structured `CombatEvent` variants:

- `ActionStarted { actor_index, action_id }`
- `HitDealt { source_actor, target_id, action_id, element, is_crit, damage }`
- `DamageReceived { target_id, source_actor, damage }`
- `HealDealt { source_actor, target_actor, amount }`
- `ShieldApplied { source_actor, target_actor, amount }`
- `ActorSwitched { from_index, to_index }`
- `TargetDied { target_id }`

### Proc conditions
Callbacks register filtering conditions:

- `AnyEvent`: matches all events.
- `RequiresCrit`: matches only events where `is_crit == true`.
- `ElementMatches { element }`: matches specific damage element.
- `ActionMatches { action_id }`: matches specific action identifier.
- `RequiresOnField { on_field }`: matches based on whether the owner is currently on-field.
- `TargetMatches { target_id }`: matches specific target identifier.

### Payloads and child events
Callbacks execute atomic payloads:

- `GrantPoolResource { pool, amount }`: adds resources to named team pools.
- `ApplyBuff { buff }`: applies or refreshes a buff definition.
- `TriggerBonusDamage { element, multiplier_percent, target_id }`: emits a secondary child `HitDealt` event.
- `TriggerHeal { amount, target_actor }`: emits a secondary child `HealDealt` event.

### Bounded execution guard
Secondary child events can trigger downstream callbacks (e.g. Bonus Damage triggering Energy Grant). To prevent infinite cascades and cyclic callback loops, the engine enforces:

- `DEFAULT_MAX_DISPATCH_DEPTH = 4`.
- If dispatch depth exceeds `max_depth`, execution aborts immediately with `DispatchError::RecursionDepthExceeded { max_depth }`.

## 5. CLI execution and independent verification

The native CLI exposes:
```bash
ntesim dispatch-stats_DIR REQUEST.json
```
Evaluating a dispatch scenario produces an auditable JSON receipt containing initial state, step receipts, final actor states, active buffs, rational stat modifiers, and resource pools.

### Verification oracle
Independent verification is enforced by `tools/verify_dispatch_cli.py`:

- Replays `ntesim/examples/dispatch-scenario.json` in an independent Python state machine.
- Rejects 12 invalid configurations (zero team size, team size > 4, initial active out of bounds, unsupported package, out of bounds switch, time reversal in advance, time reversal in event, pool current > capacity, infinite recursion loop, invalid step kind, invalid ICD rule, unauthorized top-level field).
- Validates byte-identical replay and relocation invariance.
- Checks frozen package file SHA-256 hashes and caches proof in `target/verification/dispatch-state.json`.

## Native task parameter declarations recovered, October 18

These are research findings about the client's native factories, separate from the executable harness policies above. The read-only function-reflection method in [action timing](action-timing.md#time-function-parameter-metadata) recovers eleven exact factory signatures. Joining those declarations to the repaired cue supplements yields **33 exact call sites**, all in assets without object-read warnings. This selected join is not complete native event coverage; it includes an Iroi Dark Weapon candidate whose factory identity does not admit that mode into the ordinary kit. The projection retains every ordered argument, object ordinal and statement in the metadata projection.

| Native factory | Ordered inputs, excluding ReturnValue and nested descriptors |
| --- | --- |
| AsyncTaskCharacterAttacked | WorldContextObject, InPlayer, **bGlobal, bAutoEnd** |
| AsyncTaskAbilityCharacterAttacked | WorldContextObject, InCharacter, **bAutoEnd**; no bGlobal input |
| AsyncTaskWaitApplyAbilityDamage | WorldContextObject, InCharacter, SpecifiedAbility, SpecifiedAbilitySectionNames, **bTriggerOncePerTargetPerSection, bTriggerOncePerTargetPerAbility, CustomFilter, bDebug, bAutoEnd** |
| AsyncTaskWaitSlayToActor | WorldContextObject, InPlayer |
| AsyncTaskGameplayEffectTimeChangeEx | WorldContextObject, AbilitySystemComponent, GameplayEffectQuery |
| AsyncTaskGameplayEffectTimeChange / AsyncTaskGameplayEffectStackChange / AsyncTaskGameplayEffectGEAddOrRemove | WorldContextObject, AbilitySystemComponent, GameplayEffectClass, InEffectGameplayTag |
| AsyncTaskSetTimer | WorldContextObject, InCharacter, Time, **bLooping, bMaxOncePerFrame, InFirstDelay, bInPauseTimerWhenGamePause** |
| AsyncTaskWaitAbilityInput | WorldContextObject, RelatePlayer, **bRequirePlayerPossessed, SpecifiedInputID, bTriggerOnce** |
| AsyncTaskPostPlayAbilityMontage | WorldContextObject, AbilitySystemComponent |

The saved CharacterAttacked calls request **bGlobal=true / bAutoEnd=true** for Nanally WalkOnWall, Haniel ExtraAtk and Zankou's shared Scorch-passive cue; **true/false** for Baicang's field-execute cue; **false/true** for Aurelia Passive2, Skia ShadowPas and Zankou A5/R6. These are explicit subscription inputs. A global request does not eliminate the callback's own instigator/marker/target tests; a non-global request is not proof of a native same-instigator predicate. bAutoEnd=true is a requested lifetime policy, not evidence of exactly one event or a known teardown boundary. Do not infer an immortal listener solely because its Blueprint has no EndTask call.

Shinku's three Rage timers and the separate HitBroken presentation timer request **bLooping=false, bMaxOncePerFrame=false, InFirstDelay=-1, bInPauseTimerWhenGamePause=true**. The last argument explicitly requests a game-pause policy; it is not an automatic-ability flag. The native -1 sentinel, timer age/start, which game-pause domains activate it, overlapping dilation and restoration remain unresolved. Hotori Time Stop must not silently be equated with this flag, nor must the timer inputs become real wall-clock deadlines.

Lacrimosa MeleeTotal requests **both per-target once flags false**, no custom-filter delegate, debug false and auto-end true; its own exact damage-class map still filters callbacks. Skia SneekBase requests **bRequirePlayerPossessed=true and bTriggerOnce=true** with its configured input ID; the separate event **bPress==ShouldPress** check remains necessary. Native task activation/delivery, prediction/deduplication, child-hit re-entry, source routing and actual cleanup are still required before these factory settings can become complete gameplay policies. No runtime or harness behavior changes with this research.

### Effect-request and count-query declarations

Read-only reflection binds **AbilityCharacter.K2_ActivateGameEffectsWithClassByActor** inputs to **WorldContextObject, SourceObj, GameplayEffect, ModifyData, pTriggerAbilityComp**, with a separate boolean ReturnValue. The caller requests application on the method receiver. SourceObj is an object input receiving the caller's actor; the final ASC argument is named **pTriggerAbilityComp**, not a formal EffectSource or instigator filter. A Blueprint caller passing its player/ASC in these slots supplies those exact inputs, but the declaration does not recover how native application builds Instigator/EffectCauser/SourceObject, chooses a source ASC, or settles BufferData zero values and captures. Owner summaries describing an actor as “attributed to” must retain this request-versus-settled-context boundary. An explicitly source-filtered GameplayEffectQuery or count query remains distinct from this application argument.

The separate **AbilitySystemComponent.GetGameplayEffectCount** inputs are **SourceGameplayEffect, OptionalInstigatorFilterComponent, bEnforceOnGoingCheck**, with integer ReturnValue. Skia's Shadow-Evade magnitude query passes its ShadowBase class, null instigator-filter component and true ongoing-check input. That true is not a documented “sum stacks” toggle. Native count units, ongoing eligibility and source resolution remain unresolved; keep active-handle count, stack count and a query's returned number distinct.

**CharacterAbilityUpgradeStatic.K2_GetCharacterAbilityLevelWithNetID** declares **PlayerCharacter, InUniqueId, AbilityId, OutSkillLevel, OutAwakenSkillLevel**, with a separate integer ReturnValue. Both named level outputs are out parameters. Skia's Shadow callback explicitly adds the two outputs; its ignored ReturnValue is not substituted for that expression. Other callers may use different outputs, so the metadata is not permission to reinterpret every saved level query as the same effective level.

These include the already located **Execution_Skia_ShadowEvade#/2**. These are argument/type declarations joined to actual calls, not recovered native effect application/count/upgrade bodies.
