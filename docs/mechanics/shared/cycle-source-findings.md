# Serialized cycle and Break findings

Responsibility: decoded/source numeric and lifecycle findings for cycles/Break, independently of the explicit runtime harness and missing native dispatch.

| Source finding | Status and limit |
| --- | --- |
| Heartwrench serializes `AggregateBySource`, cap 10, duration 30s, period 1s, no immediate tick and `NeverReset` period | Decoded fields; native stack refresh/overflow/expiry and ownership remain unresolved |
| Matching local Scorch assets/reaction curve give period 0.99s and duration 15s | Serialized values; they do not prove native scheduler/clock cadence or application trigger |
| Pinned public calculator uses break cap divided by reset rate on its effective clock; `45/9=5s` supports a candidate recovery interval | External project model; under [ADR 0009](../../decisions/0009-anonymous-targets-and-typed-content.md), native monster catalogs are retired and [dummy targets](dummy-targets.md#break-and-incoming-damage) use an explicitly authored Break profile |

These facts do not authorize automatic reaction admission, universal fixture bases or arbitrary layer coefficients. Formula composition belongs to [damage/stat zones](damage-and-stat-zones.md); explicit fixture behavior belongs to [reaction/Break harness](reaction-and-break-harness.md). Missing decisions/conflicts belong to [cycle/Break questions](cycles-and-break.md).

## Hexed and Stain calculation helpers recovered, October 18

The expansion bundle contains warning-free Blueprint helper bodies, without requiring native execution. `Cau_Reaction2_new:CustomCalculateBaseMagnitude` (`#/2`, statements 131..958) evaluates `DT_ReactionEffectFigure:LingZhouCopyCoef` at coordinate **1**, giving saved base **b=0.2**. It obtains `e=GetLingZhouReactionExpendOneDamageRatio(ASC)`, `m=ASC.GetFinalReactionDamageUpCoefficient(2)`, and `f=ASC.GetFinalDamageUpFinalCoefficient(HitActor ASC)`. Its successful return expression is:

```text
((b + e) * m + clamp(m - 1, 0, 100)) * f
```

For supplied `e=0.13`, `m=1.2`, `f=1.1`, the expression is **0.6556**, not 0.4356 from omitting the separate clamped term. At m=f=1 it is 0.33. This does not prove that Linko's stored 0.13 is the actual native getter return, explain the public 30% wording, identify an accumulated damage base, or establish final packet/application/double-bonus policy. Invalid ASC and explicit failed-curve branches can return **1.0** in this helper; do not replace all missing paths with zero or use that fallback as successful gameplay evidence. The owner lookup also attempts `OwnerActor.GetOwner()` when direct ASC resolution fails.

`Cau_Reaction5_new_DamUp:CustomCalculateBaseMagnitude` (`#/2`, statements 0..744) checks the owner ASC's native execution-damage-type getter against values **5 or 6**. On its successful curve path it returns `(1 + Reaction_HunXiang_DamageUP + GetHunXiangReactionExpendOneDamageRatio(OwnerActor)) * GetFinalReactionDamageUpCoefficient(5)`; the saved base row is **0.2** at coordinate zero. Getter capture, ordinal meaning, application destination and clocks remain separate from this algebra. At supplied extension 0.1 and reaction coefficient 1.2, the expression is **1.56**. Asset suffixes and helper arguments are not interchangeable with the verified `EReactionResultType` mapping below.

Both formulas are source-expression findings. They do not supersede the existing [formula models](damage-and-stat-zones.md#4-esper-cycle-effects) without a selected native consumer and an explicit evidence/model comparison. Preserve reached getter outputs as dependencies rather than inventing them from table presence.

## Global Break-base curve source

The frozen global-common table (optional cached source `DT_GlobalCommonData`), original export SHA-256 `d15fecf599b4a51567972c3c759b63ce5ae1ec4d54c60cc5f8a9a5cef19c4386`, supplies `/rows/UnbaldamagePara`. Its 80 keys use constant interpolation/endpoints. Coordinates **1 / 20 / 40 / 60 / 80** contain **91 / 140 / 462 / 1,440 / 3,603**. The level-one sample agrees with the existing Break arithmetic example; agreement at that point does not validate a native consumer or lookup coordinate.

This is a source-bound base-curve candidate, not native Break admission, contributor capture, BI/efficiency settlement or a recovery rule. With an explicitly selected base 3,603, cap 45, BI/bonus zero, attacker level 80, DEF 1,080 and RES 0.20, the existing formula gives **21,618** for one declared contribution. This independent arithmetic remains conditional on those inputs and the formula model. Acquisition preserves the source; [native cycle/Break questions](cycles-and-break.md) retain its application boundaries. No new raw table or runtime reader is needed to access this already committed curve.

## Reaction-result classification

**Source facts for the inspected PC_140/logic-566459 client.** Native enum verification confirms all seven non-sentinel `EReactionResultType` labels. The preserved `DT_ReactionData` export, SHA-256 `cff5109a224e53a1cd26b00bb80fac8354ca7485b074fe31f396675f69498fdd`, binds `/0/Rows/REACTION_RESULT_TYPE_<n>` as follows:

| Result | First element | Second element | Configured `DefaultDamageGE` |
| --- | --- | --- | --- |
| 1 | COSMOS | NATURE | `GE_ActorReaction_1_Damage` |
| 2 | NATURE | INCANTATION | `None` |
| 3 | INCANTATION | CHAOS | `Buff_Reaction_5_new` |
| 4 | CHAOS | PSYCHE | `Buff_Reaction_4_new` |
| 5 | PSYCHE | LAKSHANA | `None` |
| 6 | LAKSHANA | COSMOS | `None` |

The table exposes no result-0 row. Keep the serialized first/second element fields without inferring symmetry, application order, aura consumption or dispatch. `None` is not proof of no effect: some reactions have separate native consumers. Legacy asset numbering also differs from result numbering; `Buff_Reaction_5_new` is explicitly selected by result **3**, not result 5.

`Con_Equipment_Psyche_4` and `Con_Kite` each declare their getter result as a byte with `Enum=EReactionResultType`. Their exact target-tag/source-getter predicates belong to [Devil's Blood](../sets/devils-blood/passive.md) and [Watch Your Heads!](../weapons/kite/passive.md). Enum labels and table links resolve the numeric classification, not what the native getter captures or when modifiers evaluate. Cutoff applicability and those native consumers remain open.

## Nova period, expiry and refresh boundary

**Serialized source facts, not a native timeline.** Result 4 selects `Buff_Reaction_4_new`, export SHA-256 `422990b4fb0e01298e5c9de5fd92b39ec6ca3be25e6b1dcf22430bf9b5d01dc8`. The component `/5/Properties/InheritableGrantedTagsContainer` supplies `Ability.Player.Buff.Reaction.AnHun`; English locale SHA-256 `e2fcf52ad95ae25f7cf1432800c54ad2b3f1fab18a78f5e4a3f67e49ee1869e1`, `/0/Properties/ST_ReactionDes/Reaction_anhun`, names Nova. The adjacent `_des` text describes a five-second state followed by Mental damage.

The CDO's `/3/Properties` retains two distinct scalable bindings:

| Field | Explicit scalar | Curve row | Coordinate-1 curve value | Independent literal product |
| --- | ---: | --- | ---: | ---: |
| `Period` | 1 | `Reaction_AnHun_BuffTime` | 5 | 5 |
| `DurationMagnitude/ScalableFloatMagnitude` | 1.02 | `Reaction_AnHun_BuffTime` | 5 | 5.1 |

The curve is `/0/Rows/Reaction_AnHun_BuffTime` in `DT_ReactionEffectFigure`, export SHA-256 `4cac2af2b8fc1164cd229f57dffbbdc4030d887bb01f67bf25ef9b9e33a094d1`; it has one key and constant endpoints. The carrier declares `HasDuration`, `bExecutePeriodicEffectOnApplication=false`, target aggregation and limit 1, and `Executions[0].CalculationClass=DamageExecutionCalculation`. FINISH requests `buff_reaction_display4_end`. These fields do not support silently replacing the period with the duration, or treating a presentation-end request as the damage producer.

**Independent threshold counterexample:** at elapsed 5.05, the configured period product has been reached but the duration product has not. This separates two possible deadlines under a model that consumes those products; it does not prove that a native packet is delivered at either one. The arithmetic uses saved JSON scalars, not a claim about native binary-float rounding. The literal 1.02 scalar is present and must not disappear merely because the text rounds to five seconds.

`AnHunReactionSeventhGEComp` at object 0 and `ExecuteOnRefreshAddGEComponent` at object 4 have no serialized Properties; they retain native execution/refresh decisions. Object 1's asset tags explicitly include `Event.Common.OnlyServer`. Native initial scheduling, refresh execution, snapshot/stack reset, damage delivery versus expiry, and progress/pause remain open. The cache acquisition preserves these objects, enum data and exact source pointers. Do not promote a one-tick-at-five-seconds or damage-only-on-expiry model to native fact without that consumer evidence.

## Recovered owner extension payloads

### Reaction3 removal tail and unbound Scorch-start candidate checked, October 18

The warning-free **Buff_Reaction_3_new#/2 → GC_Reaction_3_new** join has a reached **OnRemove#/1** body. After a successful MyTarget ability-character cast it clears material desaturation, then requests **buff_reaction_display3_end at statement 477**, before checking **GetIsDead at 619**. Death only skips the later audio path; it does not guard that already reached tail-carrier request. The tail (`#/1`) is HasDuration 1 and names **GameplayCue.Display.Reaction.3.end**; its reached Properties supply no explicit damage execution or resource mutation. WhileActive changes desaturation/overlay material and does not deliver a separate reaction packet. Native removal reason, cue lifetime, tail admission and the actual reaction executor remain open; the visual tail must not become an extra damage tick.

The separate warning-free **GC_Reaction_ZhouAn_Start** has tag **GameplayCue.Display.Reaction.3.Start**, but no configured carrier match in the saved combined join. Its **OnWhileActiveEvent#/2 → graph#/1:10** casts MyTarget to ability character and EffectContext.EffectCauser to player, then at 407 requests **ordinary Buff_Reaction_5_new** on MyTarget with that player in SourceObj, BufferData level1/stack0 and null pTriggerAbilityComp. It does not request the Zankou-specific 1036 class. The CDO's three Adler-class Target Array entries and Random Stream are unused by this reached body; do not infer a three-choice spawn or three Scorch applications. Until an actual producer/current-applicability edge is recovered, this remains an unbound candidate rather than the universal Scorch initialization route. The tag's legacy 3 does not repair the payload into Reaction3.

The [shared clock requests](action-timing.md#shared-dodge-reaction-and-parry-clock-requests) separately preserve the legacy reaction-display dilation consumer.

### Common Nova removal graph anomaly recovered, October 18

Both **Buff_Reaction_4_new#/3** and **Buff_Reaction_4_new_Display#/1** name `GameplayCue.Display.Reaction.4.new`, joining warning-free **GC_Reaction_4_new**. Its **OnRemoveEvent#/2 → graph#/1:778** caches incoming EffectCauser as player and attempts to obtain its player state; a valid state, or the separate IsSummonPlayerCharacter branch, enters an equipped-player loop. The loop compares **the integer array index to 10088**, not a player CharacterID or a party actor. A match sets **Has1008=true** and stops the loop. The subsequent Has1008 gate requests **Buff_Reaction_4_new_1004 on cue MyTarget attributed to the cached player**. The WhileActive override only returns its local result; BeginPlay only calls the Actor parent.

The 10088 index and 1004 payload are preserved as a source anomaly. With explicitly supplied initial Has1008=false and array length 4, visited indices 0..3 cannot set the flag and no request follows; length 10089 would make index 10088 reachable. Has1008 is omitted from this CDO and has no explicit false/reset in the reached loop, so constructor/reused-instance initialization remains open. Do not repair the index into Skia 1008 or Lacrimosa 1004, infer a confirmed owner passive from those names, or silently admit the conditional packet as every Nova's native output. The main Nova execution/period and owner-specific enhancement listeners remain separate. Native source/cue lifetime, event delivery and current applicability remain unresolved.

### Stain time-change cue consumer recovered, October 18

The shared supplement joins **Buff_Reaction_6_new#/2 → GameplayCue.Display.Reaction.6.display → GC_Reaction_6_new_display**. Its warning-free **OnWhileActiveEvent#/3 → graph#/1:403** requires **IsAuthorityRole(MyTarget)**, obtains incoming EffectContext **EffectCauser**, requires native **CanHunXiangReactionExpendOneExtend(EffectCauser)**, then caches that actor after its player cast. On valid MyTarget ASC and successful **Reaction_HunXiang1_Time_1042=8** lookup at coordinate0, it calls **K2_ModifyActiveEffectStartTime** for Buff_Reaction_6_new with arguments **8, null, -1, false**. The [native parameter declarations](action-timing.md#time-function-parameter-metadata) identify **StartTimeDiff=8, SourceAbilityComponent=null, LimitFinalDuration=-1, bLimitToEffectDuration=false**. Only afterward does it subscribe **AsyncTaskGameplayEffectTimeChangeEx** using the CDO query **EffectDefinition=Buff_Reaction_6_new** (no explicit source filter). CDO also declares AcceptCustomDilation=false and bSupportOnlyOwnGE=true; the native meaning of that latter field is not a recovered substitute for query filtering.

The time-change callback **#/2 → 10** casts the stored OnWhile MyTarget to ability character, requests removal of **all Buff_Reaction_6_new**, then requests that same class on it attributed to the cached player, level1/stack0. It does not inspect the changed effect's new duration/start-time values, rerun the extension-eligibility gate or explicitly limit itself to one refresh. No task handle/EndTask or OnRemove override is reached. Native time-change emission, cue reuse/destruction, event re-entry, source attribution and successful removal/reapplication must therefore determine recursion and final duration. Do not flatten this into a guaranteed one-time eight-second extension or infer a 20-second total from description and row arithmetic.

The reapplied carrier (`#/2`) binds the previously recovered **Cau_Reaction5_new_DamUp** to additive **HunXiangInitialDamageUp**; this is the located custom-modifier destination, not proof of how the executor consumes it. The native extension-eligibility predicate is not replaced by a direct Blackbird/1042 lookup merely because the row name contains 1042. The [Blackbird owner](../characters/blackbird/action-damage.md#recovered-stain-and-nova-extension-parameters) retains its specific extension inputs and description conflicts. No game code was executed.

The four-struct repair supplies a warning-free current read of `DT_ReactionExtendDataTable`. [Blackbird](../characters/blackbird/action-damage.md#recovered-stain-and-nova-extension-parameters), [Aurelia](../characters/aurelia/action-damage.md#recovered-nova-extension-source) and [Linko](../characters/linko/action-damage.md#recovered-hexed-extension-inputs) own the recovered parameters, predicates, source/description differences and remaining native consumers. Those source-location/layout gaps are resolved; native activation, settlement and clocks remain open. The candidate-only Akane row is not an additional released owner.

Provenance: portable [source provenance policy](../../SOURCES.md), frozen `DT_ReactionData`/`DT_ReactionExtendDataTable`/figure and referenced local GE records; local acquisition applicability, [pinned calculator audit](../../DRIVE_CALCULATOR_RESEARCH.md). Selected graph fragments preserve their exact source identities; broad acquisition artifacts are not guaranteed present in this checkout.
