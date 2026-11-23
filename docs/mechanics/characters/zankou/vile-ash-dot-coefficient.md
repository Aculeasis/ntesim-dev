# Zankou Vile Ash DoT coefficient

**Responsibility:** Vile Ash DoT class identity and Zankou's decoded stack-coefficient calculation. Current research follows the [rolling data policy](../../../../openspec/specs/product-scope/reference.md#rolling-data-policy); historical package identities below are source references.

## Current owner composition, November 14, 2026

The current read-only client extraction locates the missing consumer in **Buff_Zankou_ListenBattle**: additive **CoefModify**, coefficient one, custom magnitude **Calc_ZankouDotStackCoef**, no extra application requirement. The native generic damage executor is inherited through `GE_Zankou_DamageBase` and `GE_DamageBase`; the latter names `DamageExecutionCalculation`. The source helper bypasses its stack read for positive class-default `fExpandMult`; otherwise it reads the exact current damage class on the hit target. Its current curves are **ZankouDotPerStackCoef=1** and **ZankouDotMaxCoef=10**.

The selected bounded composition is `native_coefficient * (1 + clamp((stacks-1)*1,0,10))`: one/five/ten ordinary stacks give one/five/ten times the native packet, never another multiplication by stacks. A positive-expansion copy bypasses that additional stack factor. The native executor body is not recovered; this algebra is an explicit model supported by the additive destination, the one-stack offset, the published stacking description and independent arithmetic. It is not certified native body execution.

Both carrier exports declare AggregateBySource, cap ten, thirty-second duration, one-second period, no immediate tick and NeverReset phase. The current selected model preserves phase on live refresh, uses half-open expiry and invalidates stale generations. Heartwrench and Vile Ash remain independent; Bloodfeast's configured five-stack Vile Ash producer is not copied onto ordinary Inferno. See the [current producer and grant contract](action-damage.md#current-grants-modifiers-and-dot-composition). Raw exports and the checked helper statements remain in .

The executable adapter now follows that composition. Five native level-one Vile Ash stacks at ATK1000/DEF0 give75 per tick and2175 across29 half-open ticks. A second paid Bloodfeast at20.5s applies at24.716667s: the first phase stays5.216667s+n seconds, with20 ticks at75 followed by30 at150. Expired stacks reset before a new application. Source defeat removes pending deliveries and opaque stack carriers; field exit preserves them. Target defeat removes only that target's trains.

Actual montage spread notifications copy up to four captured attached carriers to all eligible living dummies, including the donor. Heartwrench, Vile Ash, Nightmare, Scorch and Adler's attached Skill DoT keep their original owner, native coefficients, caps and existing phase/refresh policy. The finite selection order is Heartwrench/Vile Ash, Nightmare, Scorch, then Adler if a slot remains; this discloses the bounded choice when more than four classes coexist. Recipients are applied from pre-copy snapshots, so one recipient cannot feed another during the same spread. World area attacks are not attached carriers. Adler's one-stack carrier refreshes its ten-second deadline and keeps its queued native phase. DoT delivery itself never emits a spread notification. Source/native checks exercise four simultaneous carriers, Adler refresh beyond its original end, separate ten-stack Zankou trains, zero-HP contacts, living recipients at impact, off-field equipment callbacks and actual source/target defeat.

## Current evidence

- Vile Ash uses a distinct ultimate-tagged damage class. `Calc_ZankouDotStackCoef` checks exact classes and positive expansion, then computes a clamped additional coefficient.
- Local GE and calculation assets are hash-listed in the frozen source manifest. The bounded local model is an interpreter finding; it does not bind the `CoefModify` destination or native executor ownership.

## Provenance and limits

- Local asset summary: `LOCAL_ASSET_RESEARCH.md`; frozen source identities: [source provenance policy](../../../SOURCES.md).
- Current Zankou action/DoT slice: [`zankou-action-binding-model.json`](../../../../reviews/q10-game-action-binding/zankou-action-binding-model.json) and [`zankou-dot-producer-model.json`](../../../../reviews/q10-game-action-binding/zankou-dot-producer-model.json).
- Keep Vile Ash's exact class check distinct from the ordinary DoT producer. Do not apply the additional coefficient twice.

## Reached class filter and stack offset, October 18

The expansion's `Calc_ZankouDotStackCoef#/1/Properties/Gameplay Effect Arr` explicitly contains **`GE_Player_Zankou_DotDamage` and `GE_Player_Zankou_DotUltraDamage`**. `CustomCalculateBaseMagnitude` (`#/2`, statements 262..1130) takes the target ASC's **CurrentDamageGameplayEffect**, gets its exact class, checks that array, and checks the class-default `fExpandMult > 0` branch before the stack read. It obtains the active count for that exact class on HitActor and computes **clamp((count - 1) * ZankouDotPerStackCoef, 0, ZankouDotMaxCoef)** on the permitted read path. The subtracted one matters: a supplied single stack contributes no additional coefficient; it must not become one full extra stack contribution.

The initial bundle's ordinary `GE_Player_Zankou_DotDamage#/2` separately declares AggregateBySource, limit **10**, NeverReset period and **no immediate periodic execution**, with duration/period curves `ZankouDotDamageDur`/`ZankouDotDamagePeriod`; its tags include State.Damage.Dot and State.Damage.Melee. Do not transfer the other class's ultimate tags or treat a coefficient cap as this carrier's stack limit. These objects are warning-free. The class and stack-offset source decisions are located; native current-damage capture, expansion bypass application, coefficient destination/composition and tick scheduling remain open.

## DoT critical-damage source and party lookup, October 18

The initial bundle's `Buff_Zankou_MagicDotDamCritUp#/2/Properties/CustomApplicationModifierInfos` supplies additive **CritDamageBase**, requiring source tag **State.Damage.Dot**, through `ZankouMagicCritDamageUp`. The calculator's `#/1` selects `ZankouMagicCritDamageUpLv1=0.5` when `IsZanKouHaveLevel1Buff` succeeds, otherwise `ZankouMagicCritDamageUp=0.25` (`DT_ZankouEffectFigure#/rows`, coordinate 0). These are additional critical-damage points for eligible packets, not guaranteed critical chance or general damage bonus.

The helper (`#/2`) first checks whether the instigator has an active `ZanKouLogicObj` and, if so, tests its own `Buff_Zankou_Level1`. Otherwise it iterates **GetEquippedPlayers()**, looking for a valid member with that logic object and A1 marker; it returns true on a match and false when the loop exhausts. This is an explicit party lookup rather than an inference from the global carrier name. Effective source/recipient routing and native critical eligibility remain open: the graph does not make every teammate's DoT crit automatically.

## Delusion A3 healing gate recovered, October 18

Initial `Buff_Zankou_MagicDotDamCritUp#/2/Properties/Executions` names **MaxHPRecoverHPExecution** with a Source/snapshot additive SkillCoefficientMult custom magnitude from `Calc_ZanKouLv3Treat`. The warning-free helper (`#/2`) requires OwnerActor and HitActor to cast to players, a `ZanKouLogicObj` on **HitActor**, a living **OwnerActor**, and OwnerActor ASC holding **both Buff_Zankou_Level3 and Buff_Zankou_MagicDotDamCritUp**. It then returns `ZankouRecoverMultLv3=0.04` at coordinate 0. The carrier's period scalar is 1 and duration policy Infinite; its serialized 120 magnitude does not establish expiry after 120 seconds.

This locates a conditional four-percent magnitude and distinguishes the source tested for liveness/markers from the hit actor checked for the logic object. The native executor still decides MaxHP source/recipient, healing modifiers/clamp and delivered period/clock. Do not infer a team-wide heal or multiply by arbitrary target MaxHP solely from the global critical-damage carrier name.

## Unknown execution policies

- The additive `CoefModify` destination and selected bounded composition are located above. Existing owner checks cover the producer, expansion bypass, carrier copying and delivery without applying the stack factor twice.
- The native executor body and dynamically expanded copies beyond the declared model remain unverified; do not label model checks as measured native execution.

## Prior findings

See [source provenance policy](../../../SOURCES.md), [source provenance policy](../../../SOURCES.md), and [`zankou-dot-producer-model.json`](../../../../reviews/q10-game-action-binding/zankou-dot-producer-model.json). Revisit the bounded policies when a current executor path or controlled trace supplies additional evidence; no historical game-version match is required.
