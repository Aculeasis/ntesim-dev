# Native action and process timing


Responsibility: native admission-to-hit/recovery/cancel timing and progress/pause policy for reached processes.

Questions: how do asset montage/notify offsets map to delivered packet timestamps and ordinary recovery/earliest switch? Which clocks govern each cooldown, buff, resource, summon/DoT lifetime and ICD? What progress pauses during Hotori Time Stop, and how is remaining lifetime restored? Which owner casts continue after switching or input cancellation?

Consequence: timing changes legal actions, delayed/off-field modifiers/resources and the relative combat-time DPS denominator. Coincident modeled packets must not be promoted to verified native spacing.

The accepted product uses one public real simulated combat-time axis; this is a requirement in [DSL execution](../../../openspec/specs/script-execution/reference.md), not another research question. Internal explicit research clocks/deadline arithmetic live in [clocks and action timing](clocks-and-action-timing.md). Live executor implementation remains queue work.

Checked leads: local asset findings, [calculator montage-inference limits](../../DRIVE_CALCULATOR_RESEARCH.md), public trace applicability, owner packet receipts. Asset sequence length, overlays and cinematic duration do not independently prove native playback or cancel rules. Different recovery and earliest-switch constraints remain distinct.

The selected action/notify/parent-class chains and additional table/curve dependencies provide timing parameters. Its notify index preserves coordinates, cancellation labels and field pointers, with conditions retained in the original exports; native enum verification confirms cancellation labels. Notify wrappers such as `BP_CanBreakSkill`, `BP_CantChangePlayerNS` and `BP_NanallyCatFuncNS` inherit native classes, so their serialized presence does not supply the omitted implementation. Continue from the saved reached consumers rather than repeating broad extraction.

The [permanent tables](../../../openspec/specs/runtime-content/reference.md) supply 45 configured cooldown-curve bindings, 43 of them linked to a seconds tooltip, plus cancel curves. These numbers are available implementation inputs; countdown start/progress/modifiers and special owner overrides remain the research questions.

Return on source-bound playback/dispatch/rate/cancel/process policy, or a reviewed bounded timing model with a discriminating legal-rotation check. Do not ask for gameplay measurements, repeat exhausted acquisition, or replace unanswered intervals with invisible zero time.

## Reached shared montage caller

The October 17 Mint investigation follows the initial bundle's `GA_MeleeBase -> GA_MontageBase -> GA_HTAbilityBase` and `GA_SkillBase -> GA_MontageBase` chains. In `GA_MontageBase:ExecuteUbergraph_GA_MontageBase`, statement 1603 calls native `GetMontagePlayRate`; statement 1660 passes its result to `AbilityTask_PlayMontageAndWaitForEvent:PlayMontageAndWaitForEvent`. The separate literal 1.0 argument is not a replacement for that returned play rate. `OnReceiveEvent` forwards the received tag/data to `ApplyEffectContainer(..., -1)` and `K2_SpawnAbilityActorsEffect`; this binds the ability's event map to a reached caller without recovering native notify frequency or damage delivery. The OnInterrupted and OnBlendOut delegates reach `K2_EndAbility`; downstream cancellation, pending hits and cleanup still require the native task/ability consumers.

`BP_SendGamePlayEvent`, `BP_CanBreakSkill` and `BP_CantChangePlayerNS` are thin wrappers over native `NotifyState_SendGamepPlayEvent`, `NotifyState_CanBreakCurSkill` and `NotifyState_CantChangePlayer`. They supply no Blueprint implementation that would turn an asset interval into a measured next-input or switch deadline. Native `CheckNeedUseReleaseTask`, `GetHoldTime`, `WaitInputReleaseTask` and `AbilityTask_JumpSection` also remain on the hold/release path. [Mint](../characters/mint/action-damage.md#skill-and-burst-timing-investigation) owns its concrete event coordinates and stop/return conditions. These shared findings require no re-export.

## Damage-trigger actor configuration recovered, October 18

The saved expansion's warning-free **DamageTriggerActorDataTable#/0/Rows** contains **329** rows of **DamageTriggerActorStaticData**. A targeted join of actual roster **DamageTriggerConfigID** fields finds **82** serialized caller entries and **59** literal IDs; owner cards retain their event/container/config pairs. One caller spelling, `playerLacrimosaMelee3`, differs only in case from the table's `PlayerLacrimosaMelee3`; it is recorded as a lookup variant rather than silently inventing another row. A matching row name without an actual caller is not a bound action train.

These rows expose separate **DelaySpawnTime, DelayTriggerTime, Period, duration, TriggerOnSpawn and MaxTriggerCount** inputs. Their time-valued numbers are source configuration, not measured real elapsed seconds. Preserve all inputs: `TriggerOnSpawn=true` is not automatically an extra `1 + MaxTriggerCount` hit, a positive duration is not recovery, and `Period=0/-1` or `MaxTriggerCount=0/-1` is not automatically zero damage, an infinite attack train or a chosen frequency. Native sentinel handling, cap scope, first/last tick, pauses, gameplay-scale propagation and pending cancellation remain open. Multiple spawn entries can have different admission conditions and must not be summed before those predicates are applied; HitBack/Float and empty effect lists are distinct from damage classes.

**BP_DamageTriggerActorBase** inherits native **DamageTriggerActor** and supplies no Blueprint function body for its scheduler. The reached common montage caller above binds event maps to spawning, but actual **AbilitySpawnDamageTriggerActor**, **ProjectileSpawnDamageActorsEffect** and DamageTriggerActor initialization/dispatch still own successful spawn, repeat, target and clock behavior. A manually registered timer in a subclass is a separate source route, not another copy of the table's periodic packet. The shared protected-native-body barrier remains; locating these inputs does not replace the real combat-time denominator with an asset clock.

## Reached trigger-ability dispatcher recovered, October 18

Warning-free initial **GA_TriggerGEBase:K2_ActivateAbility#/3 → ExecuteUbergraph#/2**, 1282..1409, calls the GameplayAbility parent and **K2_CommitAbility**; a failed commit skips the dispatch path. On success, **DelayExecContainer <= 0** iterates TriggerEventTags directly. With a positive value it instead uses a latent loop: **FirstExecute=true** dispatches the first tag immediately; false waits DelayExecContainer before that first tag. It then waits DelayExecContainer after **each** dispatched tag, including the last, before checking completion. Thus positive DelayExecContainer does not by itself delay the first packet, nor does DelayEndAbility alone describe the complete configured lifecycle. The CDO **#/5** supplies FirstExecute=true, Input_Pressed and ServerInitiated; a child override must be checked separately. These are source inputs, not proof of a particular authority/clock outcome.

Each tag reaches **CustomEvent#/1 → /2**, 2237..2271, calls native **K2_CanApplyAbilityEffect**, and only if allowed constructs event data with avatar as Instigator, null Target and empty target data. It requests **ApplyEffectContainer(tag,data,-1)** and **K2_SpawnAbilityActorsEffect(tag,data)**. A positive **DelayEndAbility** creates a WaitDelay then ends on its OnFinish callback; a nonpositive value ends directly. Native commit/cost settlement, CanApply predicate, latent cancellation, task clocks, actual application and actor initialization remain required. Same-tag direct and spawned containers are both reached requests, not automatically two independent delivered packets.

Actual inherited consumers in the saved sources include **GA_Nanally_TriggerSpawnProjectile** and **GA_MintActorWind_Skill**, plus **GA_Oneiroi_TriggerGEBase** and its Branch/Awaken2 children; class bindings are retained in `trigger-subclass-headers.txt`. The Oneiroi wrapper independently waits **FirstDelay > 0** before invoking this parent, otherwise invokes it directly. Its WaitDelay and the parent's DelayExecContainer/DelayEndAbility are separate layers, not duplicate or interchangeable first-hit offsets. Owner cards retain their literal values and packet identities.

## Time-function parameter metadata

Function **parameter declarations** establish input signatures independently of protected implementation bodies:

| Function | Ordered input declarations | Consequence for reached callers |
| --- | --- | --- |
| AbilitySystemBlueprintLibrary.RefreshActiveGameplayEffectDuration | TargetASC, GameplayEffectClass, **AddStackCount**, OptionalInstigator, NewInstigator, NewEffectCauser, NewSourceObject; separate ReturnValue | Zankou passes an integer stack-add request, rather than a seconds input. OptionalInstigator and replacement attribution slots are distinct. |
| AbilityCharacter.K2_SetGameplayEffectTimerPauseOrNot | GameplayEffect, bPause, **PausedType**; separate UnderlyingType descriptor | Mint/Blackbird pass pause type 2; no extra fourth runtime argument or recovered universal pause follows. |
| Four-input SetCustomTimeDilation overload | **NewDilation, DilationType, bReplicate, bUsedParticle** | The common player-tenacity cue requests 0.01/1 with type5, false replication input and true particle input. The separate one-input overload declares only Dilation. |
| AbilitySystemComponent.K2_ModifyActiveEffectStartTime | GameplayEffect, **StartTimeDiff, SourceAbilityComponent, LimitFinalDuration, bLimitToEffectDuration** | Linko's 0, Aurelia's 2 and shared Stain's 8 are start-time-difference inputs. The -1/false arguments belong to duration limiting, rather than stack count or source replacement. |

The four-input dilation overload's DilationType enum corresponds to **Character.TemporaryCustomDilationMap_Key**; its type is **ECustomTimeDilationType**, binding **5=TimeDilationArmorBroken** (and 3=TimeDilationBuffEffect). The pause function's PausedType uses **EPausedGEType** (0=None, 2=Func, 4=SkiaUltraSkillActor); retain the call's numeric type 2 rather than asserting an unverified enum linkage.

This establishes parameter identity/order/type inputs, not successful mutation, selection of every matching effect, cap/overflow, null-source behavior, sign/limit arithmetic, snapshot replacement, clock advancement, nested-pause arbitration or timer restoration.

## Release-task and latent-delay declarations

The same read-only metadata method recovers **WaitInputRelease(OwningAbility, AutoTriggerTime, bTestAlreadyReleased)** with a separate task ReturnValue, and **Delay(WorldContextObject, Duration, AcceptTimeDilation, LatentInfo)**. These distinguish a task's automatic-trigger input from a Blueprint comparison of reported TimeHeld, and a requested dilation-aware latent delay from a generic fixed real-time wait. They do not recover the automatic/release callback arbitration, activation clock, cancellation, dilation composition or actual elapsed time.

The reached **GA_Shinku_Evade** passes **AutoTriggerTime=0.25, bTestAlreadyReleased=false**, then its callback explicitly requires bAuto=true before the already located FrontEvade/state/carrier checks. **GC_BlackBird_UltraSkill_Transform_test** instead passes **Duration=0.1, AcceptTimeDilation=true** before its paused transformation-carrier requests. Neither declaration establishes a .25/.1 real-second combat deadline. The selected Mint reference search finds only a separate Support dissolve helper and Dark Weapon candidate for this pair of functions, not a new ordinary Mint hold/release consumer; preserve the existing ordinary hold-task boundary.

The Shinku/Blackbird calls are warning-free; the separate Dark Weapon candidate has scoped read errors and is not ordinary-kit evidence. No protected native body was recovered or game code executed.

## Shared dodge, reaction and parry clock requests

Warning-free shared cue/carrier joins resolve four additional clock consumers. These are explicit actor-dilation requests, separate from the named Mint Skill-end timer pause and from natural resource/GE clocks. All calls below pass **bReplicate=false, bUsedParticle=true** after the type; no explicit authority predicate appears in these bodies.

| Carrier → cue | Reached request and cleanup | Distinct serialized timing inputs |
| --- | --- | --- |
| Buff_ExtremeAtk_TimeDilation#/2 → GC_CommonDisplay_ExtremeTime | OnWhile independently casts **MyTarget and EffectContext.EffectCauser** to ability characters and requests **0.1, type3** on each; OnRemove requests **1,type3** on each. The execution-flow continuation attempts EffectCauser even when MyTarget's cast fails. | Carrier HasDuration **0.27**. A separate Boss07 carrier shares this cue; that does not make its scene-specific admission ordinary-kit evidence. |
| buff_reaction_display6#/1 → GC_CommonDisplay_Reaction6 | OnWhile casts **MyTarget** and requests **0.12,type3**; OnRemove requests **1,type3**. BeginPlay adds no mutation. | Carrier HasDuration **0.36**. The legacy Six suffix alone does not identify a current reaction's causal producer. |
| Buff_Parry_Common_MonsterTimeDilation#/1 and _NoneDis#/1 → GC_Parry_MonsterTimeDilation | OnWhile casts MyTarget, waits **Kismet Delay 0.3**, then starts a timeline. Its Update requests **curve output,type5**; Finished returns without a separate reset. OnRemove requests **1,type5** after its own MyTarget cast. | Ordinary carrier HasDuration **0.8**, NoneDis **0.25**; timeline length **0.5**, cubic keys **(0,0.05),(0.5,1)**. These are not a flat .05 multiplier for .8 real seconds. |
| Buff_Parry_Common_PlayerTimeDilation#/1 and _NoneDis#/1 → GC_Parry_PlayerTimeDilation | OnWhile casts MyTarget and starts a timeline; Update requests **curve output,type5**. Finished and a successful OnRemove cast both reach the reset **1,type5** using the saved OnRemove-cast variable. | Ordinary carrier HasDuration **0.5**, NoneDis **0.25**; timeline length **0.65**, one serialized cubic key **(0,0.01)**. Do not invent the missing terminal curve key or infer a .65 real-second player freeze. |

Type 3 is **TimeDilationBuffEffect**, type5 **TimeDilationArmorBroken**, under the native enum/type join above. The Player Finished path reaches the saved **K2Node_DynamicCast_AsAbility_Character_1** assigned on removal, rather than the OnWhile player variable; its successful receiver before OnRemove is not established by that callback alone. Neither parry OnRemove body explicitly stops the timeline or cancels the monster's latent Delay. The NoneDis monster carrier's .25 input is less than the .3 delay input, and the ordinary player carrier's .5 is less than its .65 timeline length: native duration domains, cue destruction/reuse and latent/tick cancellation must decide whether a pending slow request survives removal. Adding .3+.5 or rescaling a carrier by its dilation input does not recover that settlement. These exact differences matter to the real combat-time denominator and later legal input/cooldown/resource progression.

The eight actual carrier→cue edges in this selected six-cue pass include [Reaction3](cycle-source-findings.md#reaction3-removal-tail-and-unbound-scorch-start-candidate-checked-october-18) and the scene-specific Boss07 edge; **GC_Reaction_ZhouAn_Start has no saved carrier edge**. No global “all buffs pause” rule, parry reward or Hotori Time Stop timing is established by these requests.

## Action timing specification

- **Single real combat-time axis**: Time in the simulation advances monotonically along a continuous timeline  \in [0, T]$ (expressed in milliseconds or seconds). Discrete frame approximations are converted to exact decimal timestamps.
- **Action progression stages**: CastStart checks admission and reserves resources; HitDelivery settles each discrete packet and its callbacks; EarliestCancel opens the selected input gate; RecoveryEnd returns the finite action to idle. These timestamps stay absolute while the selected action context determines progression.
- **Switch lock and cooldown**: The public switch has a250000us World cooldown. Owner montage switch gates are separate finite action deadlines. Persistent constructs and periodic carriers use their declared World/activity/ObjectLifetime clocks under the shared pause policy.

## Time-domain rules and hit cadence

- **Event-driven advancement**: The simulation engine advances directly between causal events (hit delivery, buff expiration, resource tick, cooldown reset) rather than busy-polling.
- **Equal-timestamp causal ordering**: mode transitions; scheduled action admission; resource costs; buff ticks/expiry; hit delivery; reactions/Break; proc callbacks; settlement/death and endpoint teardown; observation. Source replay uses AfterSettledEvents admission so reached same-time causal work finishes before the next authored input. Sequence resolves remaining ties.
- **Hitstop modeling**: consume the reached owner's explicit source duration, dilation rate and overlap rule. Local hitstop changes finite action progress while World timers continue whenever no global timer pause is active. There is no universal added delay for every heavy hit.

## Boundary conditions and interruptions

- **Pre-hit interruption** cancels unissued finite work under the owner's actual cancellation policy and retains committed costs. At equal timestamps, a delivery preceding settlement/interrupt priority remains delivered.
- **Hotori Time Stop** freezes ordinary World timers and autonomous activity. Only permitted finite Hotori/replay contexts and the living B2 Vita Bud attack exception progress; another cinematic dominates those exemptions. Deployed-object lifetime follows its separate declared policy.
- **Multi-target simultaneous hits** preserve deterministic dummy-index order and source-specific recipient caps.

## Complete action timing handoff

The [core causal priorities](../../../crates/core/src/execution/limits.rs),
[content execution loop](../../../crates/content/src/simulation/runtime/execution.rs)
and [timer policy](../../../crates/content/src/simulation/timer_policy.rs) consume
these rules. Existing owner tests verify cancellation before a later contact,
retention of already settled packets and separate switch gates. Shared clock
tests independently verify pause union and relative/absolute conservation.
