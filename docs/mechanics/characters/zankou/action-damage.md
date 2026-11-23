# Zankou action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1036`.

**Responsibility:** Zankou outgoing action packets, modeled action sequencing, and owner-specific damage effects. Current work follows the [rolling data policy](../../../../openspec/specs/product-scope/reference.md#rolling-data-policy); dated packages below identify historical inputs.

**Model state:** the current executable owner covers the combat inputs, forms, passive effects, selected awakenings and resonance described below. Native coordinates and scalar inputs remain distinct from the explicitly bounded scheduling, recipient and armor models; passing checks establish their named scenarios, not measured live-client equivalence.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Implemented form and Skill state

`zankou.forms_v1` replaces the former Normal/Hunt stance flag with Reality/Illusion. A successful entry commits at496507us montage progress and arms an eight-world-second, half-open Illusion deadline. Soulcross commits Reality at2039354us progress. The selected Skill is captured at admission: an enhanced Soulcross admitted one microsecond before natural expiry still completes afterwards; an input exactly at expiry selects Reality entry. Expiry generations prevent an old timer from ending a later form.

The canonical Skill input retains its identity while the owner derives all five entry or seven return packets. Both entry variants have998950us base recovery, with27000us lost progress from the sourced final contact pause. The paused-world five-second cooldown begins at accepted input and survives interruption. Enhanced Soulcross becomes eligible at3500000us remaining Illusion time. Enhanced entry belongs only to the current eligible Reality section, from its Skill gate through its final recovery boundary; an idle microsecond or a replacement action discards it. There is no script-controlled form setter.

Field exit cancels unborn montage contacts and notifications, returns to Reality and discards form-specific permissions. Returning to the field restores none of that discarded state. Committed cooldowns and launched DoT trains continue. An interruption before the entry notify prevents transformation; at the exact notify timestamp causal ordering commits the transition first. Existing `zankou_skill` tests verify these boundaries through source-generated commands and native replay. The existing owner tests cover both Normal/held chains, secondary inputs, Burst selection, modifiers and continuing native replay.

## Frozen player Skill audit, November 14, 2026

The executable Normal adapter now consumes the current five-stage Reality chain (2/1/3/5/6 direct contacts) and four-stage Illusion chain (1/3/4/3), preserving native A talent and Incantation scaling. Illusion Normal/held contacts carry the current `Ability.Player.Nanally.XieTongDamage` Follow-up tag and each schedules one Heartwrench application. Reality and Illusion held attacks contain four and five direct contacts respectively. Author `zankou attack; zankou attack[hold=200ms]; zankou skill;` for an earned Reality held-enhanced entry. Held input requires at least200ms and the current form's Normal1/2 predecessor; an unearned hold fails without a replacement cast.

The finite owner clock retains the sourced contact spacing, explicit positive contact-pause inputs and distinct Skill/Jump/movement gates. Missing pause duration supplies no inferred pause. Pause is integrated as lost actor progress once per causal packet across all recipients; world form/cooldown/DoT deadlines continue through this local contact hitlag, but freeze during global cinematic and active-stop pauses. The conservative ordinary continuation uses the sourced AnySkill recovery and a maximum two-second idle gap measured after that recovery. Independent level-one direct sums at ATK1000/DEF0 are6259 for all seventeen Reality contacts and4990 for all eleven Illusion contacts. Illusion Normal Cycle is27.701 before the6.3 entry contribution; recipient count cannot multiply these grants.

Secondary authoring uses existing inputs. `dash` costs the shared15 stamina and requires declared combat stamina; Front/Back use their sourced owner gates. Only an explicit earned Critical Dodge replaces the next `attack` with the166172us Riposte contact (387993us base recovery plus297000us contact lost progress). The research `dash[critical=true]` harness is testable but is not needed by the ordinary team rotation. `jump; plunge[airtime=500ms,height_bonus=1/2];` declares a new flight and landing: the contact is83928us after landing with281249us landing recovery, and height scales its native ATK coefficient by1+height within0..1. A plunge without a new Jump fails. Qualified switching alone dispatches Support at791367us with1034234us base recovery, a separate1015890us switch gate and320000us contact lost progress; an ordinary unqualified return cannot manufacture it. Source/native replay verifies earned Riposte, plunge and actual Cycle-qualified Support, paid stamina and rejected unearned landing.

## Implemented canonical Ultimate variants

The ordinary `burst` input remains `GA_Zankou_UltraSkill` in source and native replay. Selection prioritizes earned Bloodfeast, then a live free enhanced Inferno, then ordinary Inferno. Enhanced Soulcross earns Bloodfeast at1860000us progress. Paid Inferno/Bloodfeast cost120 energy and start twenty paused-world seconds of cooldown; only Bloodfeast applies five Vile Ash at4216667us. Its5675956us notify grants one free enhanced Inferno for five half-open World seconds, costing zero and bypassing the continuing paid deadline. Source selection records actual cost0/cooldown0 for that one command. Expiry, consumption, interruption or field exit restores the original paid deadline and cannot manufacture another free cast. Interruption before the grant leaves it unborn; applied DoTs continue. The input accepts no caller-selected Magic/Force ability or packet override. Existing Skill tests independently check the two consecutive variants, a rejected third cast, both sides of expiry and interruption/field-exit cleanup.

## Managed Hunt, Delusion and self CI

Hunt and Delusion are actual managed stat carriers under `zankou.form_modifiers_v1`. A0 Reality Normal/held grants self damage+25% for that action; Illusion Normal/held grants the team DoT critical damage+25% for that action. A legally adjacent Skill carries the earned action effect through setup. Successful entry retains Hunt for eight World seconds; successful Soulcross retains an earned Delusion for eight World seconds. An idle gap can end the action effect before Skill and cannot invent retention. Contact pauses extend the action component only; rejected inputs preserve the predecessor, interruption discards only its unborn/action component, and A0 field exit clears both retained components. These duration/recipient contracts agree with the current carriers and [published kit description](https://neverness.app).

Selected A1 installs permanent living-source Hunt40% on Zankou and Delusion50% on the team, including off field. Their damage and critical effects pass through normal stat projection once; the old owner callback no longer adds Hunt a second time. Native checks exercise overlap at13.833889s, Hunt expiry at13.833890s, and Delusion expiry at21.409662s for a complete Reality/Illusion chain. Ten level-eight Heartwrench stacks at ATK1000 with Hunt give325 noncritical damage; adding Delusion with zero authored critical bonus gives1625/4 critical damage. A0 field exit and actual source defeat remove their contributions; selected A1 can restore its passive carriers on revival without repeating entry Cycle.

Crimson Reverie sets entry Cycle100 once and supplies **self CI100**. It gives teammates no CI. The independent Blackbird/Twinflight case is53100 Nova damage from45000*(1+108/600), with Zankou alive or defeated; the earlier60600 team-aura result is obsolete. Zankou retains her own CI100 on and off field, and no field return refills Cycle or Ultimate Energy.

This source audit corrects the packet identities in the historical inventories below. It does not complete the executable adapter or establish native playback. The saved, warning-free `GA_Zankou_Skill` export has SHA-256 `647c769c6f5416800d98282ae9e99b5795edd9ab5e63c2bce9bc499205535704`. Its `K2_GetCustomStartSection` (`#/5`, statements 248, 647, 1186..1289 and 2111..2214) reads the **player's** `bIsMagic` and `bCanReleaseStrongSkill`. False form selects Skill1/PowerSkill1; true form selects Skill2/PowerSkill2. This is distinct from both shadow selectors described below.

Joining those montage keys to their event tags and the player's `EffectContainerMap` (`#/8`) gives the following direct packets. Packet suffixes omit `GE_Player_Zankou_`; coordinates are asset seconds, before any native playback or contact-pause interpretation.

| Player selection | Montage | Direct packet counts | Direct-event coordinates | Level-8 direct ATK coefficient |
| --- | --- | --- | --- | --- |
| Ordinary Sanguine Dash | `Zankou_Skill1` | `Skill2_Damage` x4; `Skill2_1_Damage` x1 | .122519, .226651, .345053, .453684; .666135 | 4*.386+1.028 = **2.572** |
| Enhanced Sanguine Dash | `Zankou_PowerSkill1` | `Skill1_Damage` x4; `Skill1_1_Damage` x1 | .122519, .22665128, .345053, .45368373; .6661353 | 4*.451+1.200 = **3.004** |
| Ordinary Soulcross | `Zankou_Skill2` | `Skill4_Damage` x2; `Skill4_1_Damage` x4; `Skill4_2_Damage` x1 | .09001875, .1966165; .6016507, .7222599, .87049174, 1.0069857; 1.86 | 2*.226+4*.459+2.854 = **5.142** |
| Enhanced Soulcross | `Zankou_PowerSkill2` | `Skill3_Damage` x2; `Skill3_1_Damage` x4; `Skill3_2_Damage` x1 | .090019, .196617; .601651, .72226, .870492, 1.006986; 1.86 | 2*.339+4*.687+4.281 = **7.707** |

These totals exclude summoned-shadow delivery, DoTs, buffs and recipient multipliers. In particular, the executable `Skill3`/`Skill3_2` pair is neither an ordinary Sanguine Dash nor a complete enhanced Soulcross. The original montage binding also resolves the apparent [A6 packet-identity conflict](#force-cleanup-and-per-target-a6-magnitude-recovered-october-18): `Skill3_2_Damage` is the enhanced Soulcross finisher associated with Oblivion, rather than an Inferno Flamenco or Bloodfeast packet. Its located additive `DamageUpGeneralBase` consumer still requires its actual target-stack query at delivery.

Both enhanced montages emit `Event.Montage.Player.Skill.2_2`; the player's effect map contains **five** `DotDamage` applications for that event. `OnReceiveEvent` (`#/6`, statements 5..302) applies that container at the independently obtained **Normal** talent level. Enhanced Sanguine Dash emits it at .66205525 asset seconds; enhanced Soulcross emits it at 1.86, together with `Display.1`. The latter maps to `Buff_Zankou_GiveUltraMagic` on the owner. These are configured producers, not evidence that every ordinary Skill grants Heartwrench or that a transition commits at the first direct hit.

The saved `CoolDwonCurveTable` export (`793dde3db1a1b16444a74f28119bbb6ff6a779e479c9ccb37e6465714d280ff4`) supplies **5 seconds** for `GA_Zankou_Skill` and **20 seconds** for `GA_Zankou_UltraSkill`, constant at its two retained coordinates. `GA_Zankou_Skill#/8` references that Skill row and explicitly serializes `CancelCooldownDuration.Value=0`. Thus the existing twelve-second/.12-second-notify shortcut and the historical two-second cancel surcharge below are not established by these sources. The nominal five-second row is consistent with a short Skill interval; the current finite owner clock above binds its start, progress and form-dependent admission.

The saved `DT_ZankouEffectFigure` export (`e3690dae80a85faed58cf13396f375fa2e8fbeb57f37a0a63b2f8da956c59ede`) supplies `ZankouMagicDur=8`, `ZankouMagicSkillRemainTime=3.5`, its separate `_Show=3`, `ZankouGiveUltraForceTime=5`, and both retained-modifier durations=8, all in seconds. The `MagicDur` carrier explicitly binds the eight-second row and removes itself on field exit. An eight-second lifetime and a 3.5-second remaining-time threshold would place the late window 4.5 seconds after entry **as confirmed by the repaired current logic below**. Neither arithmetic nor the `_Show` row proves the missing comparison, entry timestamp or real action deadline.

### Current logic decode and rolling source selection

The older `player_036_zankou` export has a scoped **ZanKouLogicObj** decoder error at `bRecycleSummonedCharacter` (invalid bool 235). Its reported `SummonedCharacterLocationObj`, `SummonedCharacterRotationObj` and `SummonedCharacterSkill` values point to the three Ultimate classes; those are incorrect labels from the broken mapping. The current read-only extraction repairs this object using its 37 reflected fields and independently calibrated primitive/compound types. A separate player-CDO movement-array warning does not concern the repaired logic object.

The installed executable and player asset differ from the older acquisition. This is provenance information, not a product admission gate. The user explicitly selected the current client on November 14. Current native field checks repair the decoder; no historical executable, exact patch certificate or multi-version migration is required. Old bytes and findings remain available where useful.

The repaired `ZanKouLogicObj_0` binds `RealUltraSkillClass=GA_Zankou_UltraSkill`, `ForceRealUltraSkillClass=GA_Zankou_UltraSkillForce`, `MagicUltraSkillClass=GA_Zankou_UltraSkillMagic`, the two grant carriers, MagicDur, MagicMelee, Hunt, Delusion and the retain-state carrier. `MeleeSectionArrCanReleaseStrongSkill` is exactly **Melee4, Melee5, Branch1, Atk**. `StrongSkillMagicRemainTime` binds the actual **3.5-second** row; the separate three-second `_Show` value is presentation. Passive entry uses `ZankouSetMagPas2=100` and its own thirty-second cooldown carrier. The A2 reaction counter is a separate configured class.

Raw findings, checked player-event joins, direct-only arithmetic, scoped warnings and parser-layout calibration are retained under . Current asset references, not an exact game version, own these findings. No game code is executed and no client file is modified.

### Selected finite action and permission model

The following is a bounded executable contract from current player montage notifies, current owner configuration and the [published combat description](https://neverness.app). It uses one source montage second per real second before connected contact pauses. This is an explicit playback model, not a recovered native montage interpreter. Source coordinates are rounded once to the nearest microsecond. World cooldowns and deadlines continue during local montage hitlag; global mode pauses preserve their exact remaining time and periodic phase.

- Skill selects Skill1/PowerSkill1 in Reality and Skill2/PowerSkill2 in Illusion. The five-second cooldown starts on accepted input; no separate cooldown notify is present in these four montages. The form-change notify begins at **496507us** for entry and **2039354us** for return. The selected model commits at that notify's start. Cancelled unborn notifies cannot transform; delivered packets and paid costs remain committed. Entry lasts eight World seconds, and enhanced Soulcross is available with at most 3.5 seconds remaining, before the half-open expiry.
- Enhanced entry requires an eligible current Normal4/Normal5, Reality held attack or earned counter. Its permission follows that action's Skill cancel window and ends with the action or an unrelated accepted input; a failed input leaves it intact. The repaired section array supplies the allowed actions. Held attacks require Normal1/Normal2 in the matching form and a held interval of at least .2 seconds. The public authoring route is ordinary `attack`, held `attack`, then `skill`; no packet/form selector is exposed.
- A Reality Normal chain has packet multiplicities **2/1/3/5/6**. Its Skill gates are **483052/519612/992815/601789/1228513us**; ordinary recovery gates are **1084427/1127670/1117351/1958435/1474013us**. Reality held emits Branch1 at 50454 and 309436us, Branch2 at 179547us and Branch3 at 850204us; Skill/recovery gates are **1041360/1580795us**.
- Illusion Normal direct multiplicities are **1/3/4/3**, with one Heartwrench application alongside each configured direct contact. Skill gates are **337351/616365/941312/892160us**; ordinary recovery gates are **1084427/1279700/1691888/892160us**. Illusion held emits MagicBranch1/2/3/4/4 at **152536/359393/562700/927319/1090510us**, each with one Heartwrench; Skill/recovery gates are **1430433/1620425us**. Both chains retain their own progression and reset on form exit or an unrelated accepted action.
- Sanguine Dash ordinary/enhanced recovery is **998950us**, with Jump gates **737301/748330us**. Soulcross ordinary/enhanced recovery is **2267783/2267928us**, after their respective switch locks **2250017/2249976us**. A cancelled montage discards unlaunched owner events. Already delivered persistent carriers keep independent generations and real lifetimes.
- Support uses the earned shared Support admission boundary: QTE contact **791367us**, recovery **1034234us**, switch lock **1015890us**. Counter requires an earned Critical Dodge; its contact is **166172us**, Skill/Jump gate zero, ordinary recovery **387993us**. Plunge requires the existing airborne/landing input, uses AirAttackEnd contact **83928us**, recovery **281249us**, and the sourced fall-height enhancement capped at +100%. Ordinary dodge has no damage; shared stamina costs and direction bindings apply. None is fabricated in the curated rotation.
- Ordinary Inferno uses UltraSkill1/2 at **3166667/3783333us**. Free enhanced Inferno uses ForceUltraSkill1/2 at the same rounded coordinates. Bloodfeast uses MagicUltraSkill1 three times at **2991323/3338388/3664250us**, then MagicUltraSkill2 and five Vile Ash applications at **4216667us**. Bloodfeast's free-finisher grant occurs at **5675956us**. Inferno/Bloodfeast ordinary recovery is **4818122/5697196us**, with switch locks **4799427/5649767us**.

The ordinary/enhanced Skill mappings and packet coefficients above match the current export. Shadow Skill has a distinct `GA_SkillBase` parent, no configured direct effect map, and presentation-only ShadowSkill1/ShadowPowerSkill1 montages; it does not authorize an additional copy of the player's damage. The selected model emits only the configured player producers. Native spawn presentation and geometry are outside the dummy contract. Contact pauses consume explicitly recovered owner delay/duration/rate inputs; omitted native constructor defaults are not represented as measured values. Current independent tests must establish the resulting model, cancellation and continuing-state behavior before any full-kit or preset-completion claim.

### Current grants, modifiers and DoT composition

The current player Skill `OnReceiveEvent` (`#/6`, statements528/579) explicitly requires `Buff_Zankou_Level1` before `SpreadDotDamage` at715. Thus **both enhanced Skill spread notifies require selected A1**, including enhanced Soulcross in Illusion. Ordinary Illusion Normal/held spreading remains its separate A0 path. A notify alone does not bypass its receiving function's node guard. The warning-free current receipt is retained in .

The repaired class configuration identifies Magic as **Bloodfeast Reverie** and Force as the **free enhanced Inferno Flamenco**. The Magic montage emits the five Vile Ash applications and the Force-time grant; its activation removes GiveUltraMagic and MagicDur. Force activation consumes GiveUltraForce and installs the negative-priority HideUltraForce until ability end. The published description supplies the shared-input priority and free-energy rule. The bounded owner admission model gives earned Bloodfeast priority, then an active one-use free Inferno, then ordinary paid Inferno. Paid casts require 120 energy and their twenty-second cooldown; the five-second Force grant bypasses that paid cooldown and costs zero. Expiry is half-open. A rejected input consumes nothing. Field exit cancels Illusion and pending action-derived opportunities; it cannot extend or recreate the real free-cast deadline.

Enhanced Soulcross grants Bloodfeast at its configured Display event. A4 additionally unlocks Bloodfeast when the **paid** Inferno prerequisites are ready; a free Force grant is not a second paid-ready event. This prevents an infinite A4 grant loop. No-A4 Bloodfeast requires enhanced Soulcross.

Current `GC_Zankou_Level2:OnActivateAbilityEvent` tests the repaired Real/Force/Magic class fields. The capture predicate is **Real OR (Magic AND A4) OR (Force AND NOT A4)**, followed by copying positive ReactionCnt to Lv2UltraDamUp and removing ReactionCnt. The reaction counter itself has cap **one**. The end callback removes the captured damage carrier after Real or Force, not Magic, so A4's Magic/Force pair shares one capture. Without A4, Magic leaves the stored stack for Force to capture. New Cycle participation while the A4 pair is in flight may store another stack but cannot overwrite the pair's capture. The selected model clears an abandoned pair on interruption/expiry/field cleanup, preserving already paid costs and independently stored new Kindling. R6 Scorch defeat grants its timed Scorch enhancement, not an extra Kindling or an invented cooldown refund.

`Buff_Zankou_ListenBattle` supplies the previously missing **additive CoefModify** consumer of `Calc_ZankouDotStackCoef`, with coefficient one and no extra application requirement. The helper accepts only the two exact DoT classes, bypasses its stack read when the class-default `fExpandMult` is positive, and otherwise returns `clamp((class_stack_count-1)*1,0,10)`. The selected packet composition is native coefficient times **1+CoefModify**. Thus ordinary one-/five-/ten-stack carriers give exactly **1/5/10 times** their native coefficient; multiplying by the stack count again would be incorrect. Both classes are source-aggregated, cap ten, period one second, duration thirty seconds, without immediate execution and with NeverReset phase. The current model preserves phase on live reapplication, refreshes duration, uses half-open expiry, and invalidates old generations on removal/replacement. This replaces the old Vile Ash phase-reset/inclusive-endpoint shortcut.

Eligible Illusion contacts apply their configured Heartwrench before spreading up to four DoT carriers. Enhanced entry/exit each apply five Heartwrench stacks; Bloodfeast applies five Vile Ash stacks. The selected spread model copies each carrier's source, current stack count and native coefficient to eligible living recipients including the original target, merges within that carrier's cap, refreshes duration without resetting its next phase, and suppresses recursive spread. A1 additionally enables the named Reality/held/counter/enhanced-Skill producers. Shared Cycle/resource/contact channels remain deduplicated per causal direct packet, regardless of recipient count.

Hunt contributes 25% ordinary damage, replaced by 40% at A1; Delusion contributes 25% team DoT critical damage, replaced by 50% at A1. Without A1, the appropriate admitted form action activates the modifier, with the eight-second retained overlap on its configured form transition and cleanup on field exit. A1 makes both living-source modifiers permanent, including off field. A3 adds its Hunt resistance coefficient .5 and Delusion's periodic self-heal of 4% own Max HP per second. A5 applies its once-per-target/combat enhanced-entry Break with the source/summon and mode/boss gates, and supplies team Break damage +300%. Three selected nodes separately add talent levels and +20% Ultimate ratios; A6 independently affects only Skill3_2/Oblivion's final hit, at .12 per target DoT stack capped at 2.4. Six nodes separately grant refreshable +40% base-ATK scaling for twenty seconds on damage and the sixty-second Scorch-defeat enhancement. These source rules do not by themselves claim their runtime implementation is complete.

**Crimson Reverie recipient correction:** current `player_036_zankou/ZanKouLogicObj_0.PlayerAwakenBuffMap` binds Passive2 to Zankou's own `Buff_Zankou_Passive2`; that carrier adds MagBase=100 with no global/team component. The published description likewise specifies Zankou's CI. The selected recipient is therefore **SelfActor**, including off field. The old accepted team-aura model is superseded by the complete owning delta before changing runtime code. Independent recipient expectation: off-field Zankou no longer adds 100 CI to Blackbird; Twinflight T3 alone adds 108, so a 45000-base Nova gives **45000*(1+108/600)=53100**, while Zankou retains her own 100-point bonus. Entry Cycle remains a once-only set-to-100, not an additive grant or an Ultimate Energy refill.

## Consumed packet and resource bindings

The current rolling [typed packet rows](../../../../crates/data/src/native_damage/zankou.rs) own all51 direct and periodic packet definitions, exact talent coefficients, categories, native ChargeAdd and HeterochromeAdd. The [owner profile](../../../../crates/content/src/characters/zankou/profile.rs) owns causal multiplicity and coordinates. Reality has five Normal stages with17 contacts; Illusion has four stages with11 contacts. Reality stage five alternates its two cross-cut and two pierce contacts before the heavy finish and demon slam. Held inputs emit four Reality or five Illusion contacts.

Each positive-HP direct causal packet pays its native energy and Cycle once across recipients; periodic ticks pay neither. Ordinary energy distributes1/1 to its source and3/5 to other living actors, with individual Charge Efficiency, capacity, carry and overflow. The five Reality stages independently grant2.405+1.958+6.193+9.905+14.339=34.8 Cycle. Qualifying switches spend the continuing100-point gauge; the entry passive never refills it on return. Source `.resources.kindling` is integral0..1. The current lifecycle and public observations are documented below and in [the DSL reference](../../../NTE_DSL.md).

## Current action timing contract

The executable rate-one profiles live in `zankou/profile.rs`, `forms.rs` and `burst.rs`. Native coordinates are rounded once to microseconds; the owner integrates only sourced positive contact pauses, once per causal packet across dummy recipients. Missing duration adds none. Cooldown, form, DoT, free-grant expiry and DPS denominator keep real time. Rejected inputs leave the predecessor clock and its permissions intact. Replacement/field exit/interrupt removes unborn montage contacts and notifies, preserving already applied DoTs and committed costs/cooldowns.

| Input | Direct source progress | Base recovery / separate switch gate |
| --- | --- | --- |
| Reality Skill, ordinary/enhanced |122519,226651,345053,453684,666135 |998950; no additional montage switch lock |
| Illusion Skill, ordinary/enhanced |90019,196616/196617,601651,722260,870492,1006986,1860000 |2267783/2267928;2250017/2249976 |
| Inferno, ordinary/free enhanced |3166667,3783333 |4818122;4799427 |
| Bloodfeast |2991323,3338388,3664250,4216667 |5697196;5649767 |
| Support |791367 |1034234;1015890 |
| Critical Riposte |166172 |387993 |
| Plunge |landing+83928 |landing+281249 |

Both Reality Skill finishers add27000us lost progress; Support adds320000us and Riposte297000us. Reality full-chain recoveries are1084427,1157370,1117351,1978235,1474013us including their connected pauses, totaling6811396us. Illusion Normal2 adds58800us lost progress; all exact packet-specific gates and other pauses are consumed in the profile. Movement, Skill, Jump and switch gates may differ from ordinary recovery. Forced interruption at a contact's timestamp occurs after that contact; an input admission at a notify timestamp precedes the notify. Explicit Skill commitment starts five World seconds of cooldown, with no cancellation surcharge. Paid Ultimate commitment costs120 energy and starts twenty World seconds; a live earned free enhanced Inferno has its own zero-cost admission without replacing the paid deadline.

## Recipient and continuing-cycle contract

Direct area actions reach every eligible living declared dummy in ascending index order. Normal targeting retains its owner-selected primary. Heartwrench/Vile Ash retain independent source/target aggregates and the donor-specific spread rules below. No enemy geometry or summoned-target catalog is inferred.

The [live source checks](../../../../crates/content/src/simulation/tests/source_forms.rs) execute two full held-entry, Illusion, enhanced-Soulcross and paid/free Ultimate sequences, funding the later paid cast from intervening actual hits. They verify live readiness, consumed energy, form transitions and exact native replay. The curated [Insight source](../../../../presets/studio/zankou-insight.nte) adds actual Scorch handoffs, equipped supports and a bounded120-second tail; its measured result belongs to the current engine/source identity.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain and DoT scheduling

The chain loops after its final stage and resets on a different admitted ability or field exit. The two-second continuation allowance begins after recovery; exactly two seconds continues. Every contact retains its current finite owner clock and contact pause. Ordinary and enhanced Skills, Ultimates and secondary actions use the same scheduling boundary described above.

`character.zankou.vile_ash_dot_scheduling_v2` uses Bloodfeast-only five-stack application, independent capped source/target trains, phase-preserving live refresh and half-open expiry. Both DoT categories retain normal-talent coefficients. Snapshot coefficient expansion and live owner/modifier evaluation are explicit bounded models; the [coefficient card](vile-ash-dot-coefficient.md) separates decoded native inputs from those policies.

### Silent Sunset execution and causal replacement

`character.zankou.silent_sunset_v1` now binds the A0 prerequisite for ordinary Skill/Heartwrench producers. With a living team Zankou, a qualified Scorch application installs the source's three-stack carrier rather than the ordinary cap-one carrier. Initial application is one stack; further qualified applications or connected ordinary DoT deliveries on that enhanced living target add one, cap at three, refresh15s and retain the original.99s phase without immediate ticks. Scorch's own ticks cannot reenter the listener. Killing hits clear the target's status after defeat callbacks and create no replacement on the dead target. Explicit Scorch tags without a bound runtime generation still reject the unsupported legacy fixture path.

The selected interpretation of the tooltip's newly inflicted effect is **new Scorch by Zankou**, consistent with the reached one-stack-add/attribution-replacement request and the native1036 carrier's15s/.99s inputs. All current stacks recapture his level, live CI, Incantation element and ordinary plus DoT crit-damage bonuses. They retain the shared Scorch base and fixed50% crit; they do not copy the ally's applied HP damage, ATK coefficient or30s Heartwrench duration. This is an explicit bounded interpretation, not a certified native damage reader. Dead Zankou cannot initiate further replacements, while an already armed snapshot continues independently.

`simulation/tests/zankou_scorch.rs` checks an Adler/Lacrimosa switch with third-party Zankou. At level80/CI100/DEFfactor.5/RES0 one stack gives1575 noncrit. Adler DoT at2s raises two stacks before the first Scorch tick2.19s, producing3150; the next DoT caps it at three, producing4725. Last DoT at10s refreshes expiry25s, so the original1.2s phase gives24 ticks through24.96s. Only nine DoT callbacks replace the payload; the24 Scorch ticks do not recurse. Increasing Adler's ATK does not change these Scorch numbers. Tests also preserve two continuing Skill cycles, source death and exact earlier-hit ownership links. Native/WASM/Worker compare complete reports, aggregation and replay, including rejected cyclic/missing/invalid ownership links. [ADR-0011](../../../decisions/0016-spread-replacement-causality.md) owns the narrow report extension. Ordinary Skill does not apply Heartwrench; R6 and Kindling use the separate current lifecycle below.

### Current Heartwrench producers

Ordinary Sanguine Dash and ordinary Soulcross do not invent Heartwrench. Enhanced entry applies five stacks at662055us; enhanced Soulcross applies five at1860000us. Each configured Illusion Normal/held contact adds one. Native Normal talent owns the Heartwrench coefficient even when Skill or an owner-specific spread initiates it. The shared thirty-second, one-second, half-open, phase-preserving train uses the same coefficient composition as Vile Ash while retaining a separate cap and metadata. Independent tests cover two applications through actual continuing Reality/Illusion actions, refresh around expiry, native A versus Q talent, one-/multi-target resource deduplication and a real zero-HP pause-free admission.

## Configured form and removal ability grants


**Serialized source facts, not executable form transitions.** The compact effect table (effect_parameters.jsonl) retains the following ordered `AbilitiesGameplayEffectComponent.GrantAbilityConfigs` entries, each at `/4/Properties` of its named record:

| Carrier | Ability | Explicit level parameter | InputID | Removal-policy label |
| --- | --- | ---: | ---: | --- |
| `Buff_Zankou_GiveUltraForce` | `GA_Zankou_UltraSkillForce` | 0 | 3 | `RemoveAbilityOnEnd` |
| `Buff_Zankou_GiveUltraMagic` | `GA_Zankou_UltraSkillMagic` | 0 | 3 | `RemoveAbilityOnEnd` |
| `Buff_Zankou_GrantRemoveGSkill` | `GA_Zankou_RemoveGSkill` | 1 | 35 | `CancelAbilityImmediately` |
| `Buff_Zankou_MagicDur` | `GA_Zankou_MeleeMagic` | 0 | 0 | `RemoveAbilityOnEnd` |

Zero is the serialized `LevelScalableFloat.Value`, not an omitted multiplier, zero energy cost or demonstrated effective talent level. Native form prerequisites, grant/replace/removal ordering, action admission and packets remain in [unknown](#unknown-execution-policies). These declarations do not bind the separate Vile Ash executor.

### Configured Force ability override

The compact record `Buff_Zankou_HideUltraForce`, `/2/Properties`, now also retains `AssignAbilityExtraInfoGEComponent.AbilityExtraInfo`: `AbilityClass=GA_Zankou_UltraSkillForce` and **`OverridePriority=-1`**. The cooldown-ignore field is omitted; do not copy Lacrimosa's explicit true value or synthesize false. This is a configured override, separate from the Force ability grant above. The asset's name does not prove native hiding, disablement or cancellation. Native priority arbitration, admission, lifetime and cleanup remain in [unknown](#unknown-execution-policies). Original export SHA-256: `1230ee3ae43cca0e5d916ef5cd1e8e64aa1589305e2cd3800dbc3be2a827701c`.

## Ordinary-hit Ultimate Energy

The current `zankou.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; each delivered native packet uses the consumed current ChargeAdd, independently of Cycle. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`fadia-zankou-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/fadia-zankou-ordinary-hits.json).

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/zankou-action-binding-model.json`](../../../../reviews/q10-game-action-binding/zankou-action-binding-model.json).
- Owner supplements: [`zankou-dot-producer-model.json`](../../../../reviews/q10-game-action-binding/zankou-dot-producer-model.json), [owning Rust checks](../../../../crates/content/src/simulation/tests/zankou_actions.rs), [`zankou-normal-chain.json`](../../../../reviews/q10-game-action-binding/zankou-normal-chain.json).
- Frozen inputs: `data/extracted/neverness_zankou.json`, `DT_SkillDamageData` (cache-only source; [consumed Zankou damage rows](../../../../crates/data/src/native_damage/zankou.rs)), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), `1036AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed EffectFigure parameters remain typed in [parameters.rs](../../../../crates/content/src/characters/zankou/parameters.rs); raw inputs and generated acquisition receipts remain cache-only. The existing `zankou_*` owner tests independently cover direct packet trains, both DoTs, spreading, exact expiry, interruption, target/source defeat, foreign carriers, all six selected nodes, resonance and continuing resource conservation. [Nest Bird cycles](../../../../crates/content/src/simulation/tests/nest_bird_cycles.rs) exercise two actually funded Bloodfeasts, off-field ticks and all five equipment refinements.

The combat adapter does not model overworld invisibility, enemy AI, geometry or native armor capacities. Explicit armor-pressure fixtures use the declared normalized model below. Source facts retained in the historical investigation sections explain acquisition and conflicts; they do not override the current executable contract.

## Full-kit source investigation (October 17–18, 2026)

Full extraction across client asset data, ability blueprints, and combat tables establishes Zankou's complete native damage packet inventory, passives, and awakening modifiers:

### Serialized damage-row inventory (reachability unverified)

| Ability ID | Gameplay Effect Class | Cat | Base Ratio (Lv 1/8/9) | ChargeAdd | Hetero | Unbalance | Description / Inferred Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee1_Damage` | A | 0.158 / 0.271 / 0.292 | 0.529 | 0.881 | 0.159 | Normal 1 hit 1 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee1_1_Damage` | A | 0.274 / 0.470 / 0.507 | 0.915 | 1.524 | 0.276 | Normal 1 hit 2 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee2_Damage` | A | 0.352 / 0.603 / 0.652 | 1.176 | 1.958 | 0.355 | Normal 2 hit |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee3_Damage` | A | 0.578 / 0.991 / 1.070 | 1.930 | 3.214 | 0.582 | Normal 3 main strike |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee3_1_Damage` | A | 0.183 / 0.314 / 0.339 | 0.611 | 1.018 | 0.184 | Normal 3 sub-hit 1 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee3_2_Damage` | A | 0.353 / 0.605 / 0.653 | 1.178 | 1.961 | 0.355 | Normal 3 sub-hit 2 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee4_Damage` | A | 0.682 / 1.169 / 1.262 | 2.276 | 3.789 | 0.686 | Normal 4 main strike |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee4_1_Damage` | A | 0.247 / 0.423 / 0.457 | 0.824 | 1.372 | 0.248 | Normal 4 flurry hit 1 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee4_2_Damage` | A | 0.228 / 0.391 / 0.422 | 0.760 | 1.266 | 0.229 | Normal 4 flurry hit 2 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee4_3_Damage` | A | 0.522 / 0.895 / 0.966 | 1.743 | 2.903 | 0.526 | Normal 4 flurry hit 3 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee4_4_Damage` | A | 0.103 / 0.177 / 0.191 | 0.345 | 0.575 | 0.104 | Normal 4 flurry hit 4 |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee5_Damage` | A | 0.364 / 0.624 / 0.674 | 1.215 | 2.023 | 0.366 | Normal 5 start hit |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee5_1_Damage` | A | 0.126 / 0.216 / 0.233 | 0.421 | 0.701 | 0.127 | Normal 5 intermediate flurry |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee5_2_Damage` | A | 0.965 / 1.654 / 1.786 | 3.222 | 5.365 | 0.971 | Normal 5 heavy slash |
| `GA_Zankou_Melee` | `GE_Player_Zankou_Melee6_Damage` | A | 0.634 / 1.087 / 1.173 | 2.117 | 3.526 | 0.638 | Normal finisher slash |
| `GA_Zankou_PerfectEvade` | `GE_Player_Zankou_PerfectEvadeAttack_Damage` | Z | 1.750 / 2.999 / 3.239 | 2.500 | 4.200 | 2.500 | Perfect Evade counter |
| `GA_Zankou_QTE` | `GE_Player_Zankou_QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 5.200 | 0.000 | 2.500 | Switch-in QTE attack |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill1_Damage` | E | 0.263 / 0.451 / 0.487 | 0.570 | 0.945 | 0.900 | Skill variant 1 hit 1 |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill1_1_Damage` | E | 0.700 / 1.200 / 1.296 | 1.520 | 2.520 | 2.400 | Skill variant 1 hit 2 |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill2_Damage` | E | 0.225 / 0.386 / 0.416 | 0.570 | 0.945 | 0.120 | Skill variant 2 hit 1 |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill2_1_Damage` | E | 0.600 / 1.028 / 1.111 | 1.520 | 2.520 | 0.320 | Skill variant 2 hit 2 |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill3_Damage` | E | 0.198 / 0.339 / 0.366 | 1.101 | 0.833 | 0.101 | Sanguine Dash / Skill 3 rapid hit |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill3_1_Damage` | E | 0.401 / 0.687 / 0.742 | 2.230 | 1.686 | 0.205 | Sanguine Dash / Skill 3 mid strike |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill3_2_Damage` | E | 2.498 / 4.281 / 4.624 | 13.877 | 10.491 | 1.277 | Sanguine Dash finisher slash |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill4_Damage` | E | 0.132 / 0.226 / 0.244 | 0.502 | 0.833 | 0.101 | Soulcross / Skill 4 early hit |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill4_1_Damage` | E | 0.268 / 0.459 / 0.496 | 1.017 | 1.686 | 0.205 | Soulcross / Skill 4 mid hit |
| `GA_Zankou_Skill` | `GE_Player_Zankou_Skill4_2_Damage` | E | 1.665 / 2.854 / 3.082 | 6.328 | 10.491 | 1.277 | Soulcross finisher slash |
| `GA_Zankou_Skill` | `GE_Player_Zankou_SkillForce_Damage` | E | 0.000 / 0.000 / 0.000 | 0.000 | 0.000 | 999999.0 | Force stance unbalance/interrupt impulse |
| `GA_Zankou_UltraSkill` | `GE_Player_Zankou_UltraSkill1_Damage` | Q | 1.251 / 2.144 / 2.316 | 0.000 | 0.000 | 0.744 | Burst: Oblivion hit 1 |
| `GA_Zankou_UltraSkill` | `GE_Player_Zankou_UltraSkill2_Damage` | Q | 2.949 / 5.054 / 5.458 | 0.000 | 0.000 | 1.756 | Burst: Oblivion final blast |

### Passives, Awakenings, and Reactions

Source descriptions and selected IDs: `1036AwakenEffect#/rows/1036/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **Passives:**
  - `GA_Zankou_Passive1`: Binds `Buff_Zankou_Passive1_C` (`/Game/Blueprints/Abilities/Player/Ability_036_Zankou/PassiveEffect/Buff_Zankou_Passive1`).
  - `GA_Zankou_Passive2`: Binds `Buff_Zankou_Passive2_C` (`/Game/Blueprints/Abilities/Player/Ability_036_Zankou/PassiveEffect/Buff_Zankou_Passive2`).
- **Awakenings:**
  - A1 (`Effect1`): Wildfire, Flickering Shadow, Voidstep, enhanced Sanguine Dash, and enhanced Soulcross spread DoT effects upon Awakening. Grants permanent Hunt (+40% DMG) and Delusion (+50% DoT Crit DMG to whole team). Never removed on swap or form switch; removed only on death.
  - A2 (`Effect2`): Accumulates Kindling when participating in a Cycle triggering Scorch or Hexed, boosting Final DMG of next Inferno Flamenco by 150% (up to 1 stack).
  - A3 (`Effect3`): Hunt increases Interruption Resistance by +50%; Delusion regenerates 4% Max HP per second.
  - A4 (`Effect4`): Activates Bloodfeast Reverie immediately when Inferno Flamenco is ready. Shares Kindling bonus.
  - A5 (`Effect5`): Guarantees Break on first breakable target hit by enhanced Sanguine Dash (once per battle per target). Increases all allies' Break DMG by 300% while Zankou is on the team.
  - A6 (`Effect6`): Increases final hit of Oblivion by 12% per DoT stack on target, up to 240% (calculated independently per target).
  - Resonance 3 (`resonance_3`): +1 level to Wildfire, Nightmare Waltz, Sanguine Dash, Soulcross, Inferno Flamenco, and Bloodfeast Reverie; +20% DMG Ratios to Bloodfeast Reverie and Inferno Flamenco.
  - Resonance 6 (`resonance_6`): +40% ATK for 20s after dealing damage. Defeating a target with Scorch grants a 60s enhancement that maxes out target's Scorch stacks on hit and refreshes on further Scorch kills.
- **Attribute scaling:**
  - Scaled strictly via `AtkRateBaseArray` under Incantation element.
- **Reactions:**
  - Binds Reaction 5 (Scorch / Incantation) via `Buff_Reaction_5_new_1036_Add_C`.

### Force cleanup and per-target A6 magnitude recovered, October 18

The initial bundle's `GA_Zankou_UltraSkillForce:ExecuteUbergraph_GA_Zankou_UltraSkillForce` (`#/2`) is entered from `K2_ActivateAbility` at statement 217: parent activation, successful avatar-player cast and authority check precede removal of `Buff_Zankou_GiveUltraForce` (382) and request of `Buff_Zankou_HideUltraForce` (429). `K2_OnEndAbility` enters at 576, calls the parent end callback, then reaches the authority-gated removal of HideUltraForce (165). The callback does not branch on its cancellation boolean before that cleanup. This establishes the requested activation/end lifecycle; native ability arbitration, delivery and in-flight packet survival remain open.

The separate warning-free **GA_Zankou_UltraSkillMagic#/3** enters `/2` at 264, calls the montage parent, and on a valid avatar-player/authority path immediately requests removal of **Buff_Zankou_GiveUltraMagic** and **Buff_Zankou_MagicDur**, both all stacks (-1), statements 165/212. Its reached OnReceiveEvent only calls the parent. This locates Magic's activation cleanup request rather than delaying both removals to Burst end or borrowing Force's Hide carrier. Native removal/grant arbitration, active MagicMelee cancellation and pending DoT survival remain open. The compact rendering is .

`Calc_ZanKouLv6DamUp:CustomCalculateBaseMagnitude` (`#/2`, statements 101..856) requires the owner's `Buff_Zankou_Level6`, obtains **HitActor** ASC, and asks for the total stacks of effects matching its configured tag container. It reads `ZankouPerStackDamUpLv6=0.12` and `ZankouDamUpMaxLv6=2.4` at coordinate zero and returns **clamp(target stack total * 0.12, 0, 2.4)** on that path. Five/twenty/twenty-one supplied stacks yield **0.6/2.4/2.4**. This is a per-target helper value, not a team stack pool or automatically a separate 1.6 multiplier. Native tag-query eligibility, read time and modifier destination remain necessary. The inspected functions have no decode warnings; source locations/lifecycle requests and algebra are no longer missing.

The actual tag literal at statement 399 is **State.Damage.Dot**. The initial `GE_Player_Zankou_Skill3_2_Damage#/2/Properties/CustomApplicationModifierInfos` binds this helper with multiplier 1 to additive **DamageUpGeneralBase**. The [frozen player Skill audit](#frozen-player-skill-audit-november-14-2026) identifies that packet as enhanced Soulcross's final direct hit, resolving the earlier apparent mismatch with Oblivion's final hit. This does not install A6 on every Skill or either Ultimate. Native capture/reachability and any further consumer remain open; the selected destination and tag are located.

The new inventory's first two Normal rows have been corrected against their exact `DT_SkillDamageData` identities, including ChargeAdd/HeterochromeAdd/UnbalValue. They are different packets, not two copies of an averaged ratio. Form reachability and remaining alternate rows are not established by that inventory.

### Located A5 Force impulse admission

The initial `GE_Player_Zankou_SkillForce_Damage#/2` requires `Con_Zankou_Lv5`. Its warning-free `#/2` function first requires a player SourceActor, active ZanKouLogicObj with **bIsSummonedBySkill=false**, and `Buff_Zankou_Level5`. The hybrid-server branch then explicitly returns true. Its other branch requires a valid ability-character target without `Buff_Zankou_Lv5TargetBeenUnbal` and a valid ASC; it rejects only the conjunction **IsStroyMonster() AND IsBossCharacter(target)** at the final classification check. Do not simplify this to rejecting every boss, and do not bypass the earlier owner/summon gates. Target marking/clearing, native impulse/Break settlement and callback mode remain open; the configured 999999 UnbalValue is not proof of unconditional live Break.

### Passive description coverage, reviewed October 18

Saved description source: `guide_zankou.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Silent Sunset describes a three-stack Scorch limit, one Scorch grant when an ally applies a DoT to a Scorched target, replacement of prior damage/type/duration by the new effect, and recursion suppression. Crimson Reverie describes Cycle Energy set to 100 on combat entry, once per battle with a 30s cooldown, plus 100 Cycle Intensity while on the team. A set-to-100 operation must not be modeled as adding 100, and native combat reset, stack replacement, trigger ownership and event recursion remain open.

### Crimson Reverie stat source recovered, October 18

The initial `Buff_Zankou_Passive2#/1/Properties/Modifiers` supplies additive **MagBase**, multiplier 1, through `DT_ZankouEffectFigure:ZankouMagUpPas2=100`. This locates the stated Cycle Intensity contribution as 100 stat points, not a percent ATK or flat Cycle Energy grant. The Infinite, source-aggregated, limit-one carrier has NeverRefresh/NeverReset; it does not contain the separately described combat-entry set-to-100 Energy mutation or its 30-second/once-per-battle guard. Those operations remain with the native combat-entry consumer.

### Hunt damage bonus and repaired dynamic-armor source, October 18

The warning-free `ZankouRealDamUp:CustomCalculateBaseMagnitude` (`#/1`) validates instigator ASC, then returns **ZankouRealDamUpLv1=0.4** with `Buff_Zankou_Level1`, otherwise **ZankouRealDamUp=0.25**, at coordinate 0. `Buff_Zankou_RealDamUp#/2/Properties/CustomApplicationModifierInfos` binds it with multiplier 1 to additive **DamageUpGeneralBase**. This is a 25%/40% source selector, without adding both values. The original expansion export stopped on **DynamicSuperArmorCoefficient**; a targeted schema recovery now decodes that same raw asset through the remaining fields without an object-read warning. Prior recovered properties compare equal.

The separate warning-free `Calc_ZanKouLv3SuperArmor#/2` requires valid owner ASC, `Buff_Zankou_RealDamUp` and `Buff_Zankou_Level3`; it returns **ZankouSuperArmorLv3=0.5** on that path and explicit **1.0** otherwise. The repaired **DynamicSuperArmorCoefficient** explicitly selects this calculator as CustomCalculationClass, with bEnableDynamicSuperArmor=true. The now-visible tail declares **Infinite**, AggregateByTarget, limit **1**; serialized DurationMagnitude=120 is not a 120-second lifetime. This closes the binding and configured lifecycle fields. Native interpretation of the armor coefficient, actual Hunt activation/removal and callback clocks remain open; no game function or constructor was executed to recover these fields.

### Shadow-Skill form selectors and event forwarding recovered, October 18

Initial warning-free **GA_Zankou_Shadow_Skill:K2_GetCustomStartSection#/4** requires a valid non-real-player AbilityCharacter, its active **ZanKouLogicObj**, a player leader and that leader's active logic. Its successful mapping reads **the shadow's bIsMagic**, but **the leader's bCanReleaseStrongSkill**:

| Shadow bIsMagic | Leader strong=false | Leader strong=true |
| --- | --- | --- |
| true | ShadowSkill1 | ShadowPowerSkill1 |
| false | ShadowSkill2 | ShadowPowerSkill2 |

The separate warning-free **MS_ZankouShadowSkill:K2_GetStartSectionName#/1**, selected by this ability's AdditionalStartSections (`#/6`), first requires the owning shadow logic's **bIsSummonedBySkill**. It then reads **the leader's bIsMagic**, mapping true to **ShadowSkill2/ShadowPowerSkill2**, false to **ShadowSkill1/ShadowPowerSkill1**, with the same leader strong flag. These are different state owners and opposite numbered mappings; they cannot be merged by assuming a shared bIsMagic. Native shadow-form initialization and precedence between custom/Additional selectors remain open. The reached CDO explicitly lists only ShadowSkill1/ShadowPowerSkill1 montage keys, so numbered section selection alone does not recover effective montage inheritance or every delivered packet.

The ability's activation (`#/3 → /2`) calls its montage parent, then requests **HideWithDuration(20,.1)** only for a valid avatar player with **IsRealPlayerCharacter=false**. Its **OnReceiveEvent#/5** similarly forwards to GA_MontageBase only for that non-real-player path. The real-player or failed-avatar branch returns without forwarding. The .1 hide input is not a damage delay, and this shadow override is not the ordinary real player's Skill dispatcher. The player Blueprint's OnCanReleaseStrongAbilityChanged merely refreshes ability extra info; no resource mutation is recovered there. Sources: `alternate-selector-bindings.txt` and `alternate-action-config.txt`. Actual shadow spawn/form admission, shared resource use, hit counts and DoT lifecycle remain native dependencies.

### Shadow Skill target fallback recovered, October 18

Warning-free **LockRange_Zankou_ShadowSkill:GetTargets#/1**, bound by **GA_Zankou_Shadow_Skill#/6/Properties/LockTargetInfo/TargetType**, tries OwningActor as the player, gets its leader and the leader's GetAttackTarget. With those validity checks passing it returns **OutActors=[leader attack target]**, empty HitResults and TargetData. Failure of any check reaches the **native GATargetType:GetTargets** parent and forwards its outputs; it does not unconditionally return no target. This is a concrete preferred-target/fallback source route alongside the own-shadow and additional-MS packet selectors above. The native fallback's recipient rules, leader lifetime and effective damage ownership remain open. Source: ; caller references in `spawn-target-caller-refs.txt`.

### Summoned-player and invisible-Skill overrides checked, October 18

Warning-free **GA_Zankou_InvisibleSkill:K2_CanActivateAbility#/2** returns **NOT avatar-player.GetPlayerFightState()** on a successful cast, with empty RelevantTags; failed cast leaves its unassigned return. No external binding for this ability was found in the selected saved Properties/bytecode, so it does not establish ordinary in-combat Shadow admission. **SummonedZankou:BPOnBuildCharacterMeshFinish#/4 → /5** calls the native player's mesh-finish parent then **SetAutoCombat(false)**; BeginPlay calls player_036_zankou's parent implementation. Its four object-read warnings remain scoped. These functions supply neither a shared-resource constructor nor an autonomous attack scheduler; native Shadow initialization remains open. Source:  and `overlap-and-invisibility-callers.txt`.

### A2 conditional final-zone and source/summon predicates recovered, October 18

Warning-free **Buff_Zankou_Lv2UltraDamUp#/1/CustomApplicationModifierInfos** binds additive **FinalDamageUp** to **ZankouFinalUltraDamUpLv2=1.5**, with required Source tag **State.Damage.UltraSkill** and condition **Con_ZanKouLv2_ApplyDamUp#/2**. That function accepts a valid GameplayEffect tagged **Ability.Player.Zankou.RealUltra**; a **MagicUltra** effect instead additionally needs valid SourceActor ASC with **individual Level4**. Other/failure paths return an unassigned local. The carrier's own **#/2** requires **SourceActor Level2 AND SourceHasAuthority**; the latter's complete function validates SourceActor then calls HasAuthority. Thus A2 carrier admission, Burst-tag eligibility and the extra A4 Magic-route gate are distinct predicates. The carrier is Infinite, target-cap1; its duration magnitude 600 is not ten minutes of confirmed uptime. The A2 ReactionCnt carrier uses the same source/authority admission, but these declarations alone do not recover Kindling accumulation/consumption or either carrier's producer. Under the shared final-zone rule, isolated input 1.5 corresponds to a 2.5 multiplier, not adding 1.5 ATK coefficients or multiplying damage by 1.5.

**Con_SourceIsZankou#/2**, bound by **MagicDotDamCritUp#/3**, requires SourceActor player, active valid **ZanKouLogicObj**, and **NOT bIsSummonedBySkill**. This is the named marker/logic test, not a general character-ID or IsRealPlayerCharacter comparison. The carrier's conditional **CritDamageBase** contribution additionally requires **State.Damage.Dot**; its separate MaxHPRecoverHPExecution remains with the existing heal calculation. **Con_Zankou_Skill2Stop#/2**, bound by Skill2Stop#/2, simply returns that SourceActor logic's **bIsMagic**. Omitted DurationPolicy for Skill2Stop does not turn its 2.5 magnitude into a confirmed expiry rule.

The **LevelExtra** family also preserves different contexts: **LvExtraAtkUp** requires SourceActor player has LevelExtra; **LvExtra_Update** tests TargetActor player instead; **LvExtra** accepts IsHybridMode_Server(TargetActor), otherwise requires SourceActor player has LevelExtra. Their bound carriers are **LevelExtra_AtkUp#/2**, **LevelExtra_Update#/2** and **LevelExtra_Target#/2**. Configured ATK is additive **AtkUp 0.4 for 20**, while the target carrier's **DamageResistIncantationBase** row is **0**, not a nonzero inferred resistance shred; Update duration is 60. Target-side hybrid bypass is not a substitute for a selected resonance on every source. Native producers, refresh/recipient behavior and snapshot timing remain open. For the previously located A5 Force impulse, **Lv5TargetBeenUnbal#/1** is Infinite, target-cap1/DeadRemove, with both ENTER_BATTLE and LEAVE_BATTLE self-removal requests; its actual mark producer remains unlocated, so the presence predicate alone does not enforce a complete once-per-battle ledger.

Sources: `zankou-shared-condition-callers.txt`, `zankou-shared-conditional-carriers.txt`, `zankou-source-authority-function.txt`, `zankou-conditional-carrier-producers.txt` and `DT_ZankouEffectFigure` named rows. All cited predicate/carrier exports are warning-free; live application is not established by a class name.

### Scorch passive and R6 refresh cue consumers recovered, October 18

The shared supplement resolves the misleading names: **Buff_Zankou_Passive1#/1 → GameplayCue.Display.Reaction.3.1008X → GC_Reaction_ZhouAn_1033**, whose warning-free CDO **#/5** configures **In Character ID=1036**, AcceptCustomDilation=false and bDestroyAtEnd=false. This is a Zankou carrier/class join despite 1008X/1033 in those identifiers. **OnWhileActiveEvent#/4 → graph#/1:1312** casts MyTarget to player, validates its player state, caches it, then subscribes **AsyncTaskCharacterAttacked(player,player,true,true)**. The [native callback parameter metadata](../../shared/buffs-and-callbacks.md#native-task-parameter-declarations-recovered-october-18) now identifies these inputs as **bGlobal=true, bAutoEnd=true**; actual global scope and task lifetime remain native dependencies.

The damage callback **#/2 → 10** validates DamageGE and DamagedCharacter, requires the latter have **Buff_Reaction_5_new_1036**, requires DamageGE's asset tags contain **State.Damage.Dot**, and rejects **ObjectIsA(DamageGE, Buff_Reaction_5_new_1036)**. It then looks up party1036 through the cached player state, calls **RefreshActiveGameplayEffectDuration** on the damaged actor's ASC for that Scorch class with literal argument **1** and party1036 supplied in the three trailing actor slots, then requests **Buff_Reaction_5_new_server** on DamagedCharacter attributed to party1036. No positive-DamageAmount, allied-instigator identity or source-filtered existing-Scorch test appears in this callback. OnRemove enters an empty branch; no explicit task EndTask/unsubscribe is present. The recovered [native parameter declarations](../../shared/action-timing.md#time-function-parameter-metadata) identify **AddStackCount=1, OptionalInstigator=null**, and **NewInstigator/NewEffectCauser/NewSourceObject=party1036**. This is an explicit one-stack-add and attribution-replacement request; accepted stacks, snapshot replacement and native recursion suppression remain unresolved.

The server marker (`#/2`) is OnlyServer, HasDuration **Reaction_ZhouAn_BuffTime=15**, target cap1 and grants **Ability.Player.Buff.Reaction.ZhouAn**; it has no explicit damage execution or stack-add payload. The separate Scorch carrier **Buff_Reaction_5_new_1036#/2** has **bApplyGEStackCount=true**, target cap **3**, duration input **15**, period input **0.99 × Reaction_ZhouAn_Period(1)=0.99**, no immediate periodic execution and NeverReset period. Its **ZhouAnReactionSixGEComp** has no recovered serialized body. The literal 1 is now bound to the integer AddStackCount parameter, not seconds; native cap/overflow, refresh and accepted mutation still require the implementation.

Both **GE_Player_Zankou_DotDamage / DotUltraDamage#/2** declare `GameplayCue.Display.Reaction.5.1036`, joining **GC_Reaction_5_New_1036**. **OnWhileActiveEvent#/2 → graph#/1:486** obtains incoming EffectContext **EffectCauser as player**, requires that player have **Buff_Zankou_LevelExtra_Update**, then resolves MyTarget as ability character. If the target already has Scorch1036, it requests **RefreshActiveGameplayEffectDuration** with literal **5** and four null trailing objects; otherwise it requests Scorch1036 there attributed to EffectCauser, level1/stack0. The parameter metadata binds this to **AddStackCount=5** with OptionalInstigator/NewInstigator/NewEffectCauser/NewSourceObject all null. Five requested stacks must remain distinct from the carrier cap3 and actual accepted stacks; this does not prove every DoT tick reaches OnWhile. The update carrier (`#/1`) is HasDuration via its named curve, target cap1 and requires **Con_Zankou_LvExtra_Update**; its own producer/admission and the refresh method remain separate. The parent LevelExtra FINISH requests removal of Update and AtkUp, independently of this cue's target branch.

Sources: `shared-reaction-cue-carriers.txt`, `zankou-scorch-update-carriers.txt`, `shared-cue-extraction.json` and `cue-supplemental-joins.json`. These warning-free requests narrow the open Scorch chain without replacing the separate Vile Ash coefficient owner or native Kindling/combat-entry questions.

### A2 activation and cleanup consumers recovered, October 18

The repaired **Buff_Zankou_Level2#/1 → GameplayCue.Display.Zankou.Level2 → GC_Zankou_Level2** join locates Kindling's consumption-side caller. OnWhile **#/7 → graph#/1:15** caches MyTarget as player and, after **HasAuthority**, binds that player's activation, ability-end and fight-state delegates, plus ASC OnRemoveAbilityDelegate. **OnActivateAbilityEvent#/3** requires PlayAbility cast with **Ability.UltraSkill**, active **ZanKouLogicObj**, and this exact class/marker expression:

```text
class == RealUltraSkillClass
OR (class == MagicUltraSkillClass AND player has Level4)
OR (class == ForceRealUltraSkillClass AND player lacks Level4)
```

After valid ASC, it reads **Lv2_ReactionCnt**; count >0 requests **Lv2UltraDamUp** on the cached player with nStackCount=count, then removes all ReactionCnt. This is ordered copy/removal, not a proved atomic reservation or native Cycle-generation route. ReactionCnt is Infinite, target cap1 and DeadRemove; its duration magnitude 600 is not ten minutes of confirmed uptime. The already recovered 1.5 FinalDamageUp and packet-tag predicates remain separate from this ability-class gate.

**OnAbilityEndEvent#/2** removes all Lv2UltraDamUp only for an UltraSkill whose exact class equals **ForceRealUltraSkillClass OR RealUltraSkillClass**. It does not include the MagicUltraSkillClass arm of the activation expression or branch on cancellation in its reached body. **OnRemoveAbilityCallback#/5** instead resolves the ability from its spec and removes the bonus only for exact **ForceRealUltraSkillClass**, after the UltraSkill/logic checks. **OnFightStateChanged_Event#/4** ignores its bInFight value and removes both ReactionCnt and damage bonus on either delivered transition. OnRemove unbinds all four delegates on the authority path, removes both carriers and clears the player. Native class configuration, event order, effect settlement and the apparent Magic cleanup asymmetry remain explicit; no universal “consume on any Burst and expire on any Burst end” rule follows.

### A5 target marker and R6 damage/slay producers recovered, October 18

**Buff_Zankou_Level5#/1 → GC_Zankou_Level5** installs AsyncTaskCharacterAttacked(self,player,false,true) after MyTarget's player cast and **IsAuthorityOrAutonomousHybridMode**. The callback requires valid DamageGE with exact class **GE_Player_Zankou_SkillForce_Damage**, InstigatorCharacter cast to player and an active ZanKouLogicObj on that instigator. If DamagedCharacter lacks **Lv5TargetBeenUnbal**, it requests that marker there attributed to the callback instigator; otherwise it only prints a recache diagnostic. No positive-DamageAmount or confirmed native Break-result check appears. OnRemove clears its player and ends a valid task. The marker (`#/1`) is Infinite, target cap1, DeadRemove, and requests self-removal on **both ENTER_BATTLE and LEAVE_BATTLE**. Together with the previously recovered SkillForce admission predicate this locates target marking/reset requests, while successful Break, hybrid bypass, native battle-event delivery and multiple-source scope remain open.

**Buff_Zankou_LevelExtra#/1 → GC_Zankou_LevelExtra** installs damage and slay-to-actor tasks after the same player/mode gate. Native parameter declarations identify both this damage subscription and A5's as **bGlobal=false, bAutoEnd=true**; the slay factory has only WorldContextObject/InPlayer inputs, so its kill attribution cannot be inferred from an absent global boolean. The damage callback adds each valid DamagedCharacter to **Damaged CharacterSet**, creates **K2_SetTimerForNextTickDelegate(NextTickApplyTargetBuff)** only when its saved handle is invalid, and does not explicitly test DamageGE, damage amount or instigator identity. The next-tick callback iterates a local counter up to the set length without reading/applying to individual target objects; if the set is nonempty it makes **one LevelExtra_AtkUp request on the cached player**, then clears the set and timer handle. This is one request for the collected nonempty batch, not one ATK stack per target. Native task routing, re-entry and accepted stat-effect refresh remain separate from that coalescing graph.

The slay callback validates DeathCharacter and the cached player, then accepts **either Buff_Reaction_5_new_1036 OR ordinary Buff_Reaction_5_new on the dead character's ASC**, without a source filter. It requests **LevelExtra_Update on the cached player**. That carrier's constant duration input is **60**, target cap1; **Con_Zankou_LvExtra_Update#/2** separately requires TargetActor be player and have **LevelExtra**, with no reached explicit false assignment on failed paths. This closes the missing request route to the [R6 Scorch refresh cue](#scorch-passive-and-r6-refresh-cue-consumers-recovered-october-18), while native kill attribution, application/refresh and refresh-method settlement remain open; the refresh argument names are now recovered above. OnRemove ends valid damage/slay tasks, clears the set/timer and cached player. The parent LevelExtra FINISH independently removes Update/AtkUp; these are distinct cleanup routes.

Sources: `zankou-awakening-cue-carriers.txt`, `zankou-extra-update-condition.txt`, `zankou-reaction-count-refs.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`; all cited functions/carriers are warning-free. The selected owner-folder reference scan finds no additional serialized ReactionCnt producer, so native Cycle-generation remains open rather than inferred from consumption.

### Invisibility state-change removal request checked, October 18

The repaired **Buff_Zankou_Invisible#/2 → GC_Zankou_InvisibleSkill** join supplies a nonstandard native call missed by the broad gameplay-call name filter: **ZanKouLogicObj.ServerRemoveInvisibleGE**. OnWhile skips its own body on **IsHybridMode_Server**, rejects an already valid cached player, otherwise caches MyTarget player and applies mesh/material presentation; only **IsLocalClientRole** installs the character-state task. Entered-state callbacks call **ConditionallyRemoveInvisibleGE#/1**; broken-state callbacks have an empty reached branch.

That helper starts CanRemove=false and accepts state bytes **39/14/24/21**. State 17 instead sets it true but can clear it when the valid controller/context/dialogue-target chain resolves a target class derived from the configured **ShoopGoodBaseClass**. The execution-flow continuation then requires CanRemove and active ZanKouLogicObj before ServerRemoveInvisibleGE; it does not directly apply damage, consume Kindling or enforce a native invisibility duration. OnRemove restores presentation, ends a valid state task and clears the cache; world/dialogue presentation is not a combat recovery profile. Native state labels, server RPC admission, effect removal and current action arbitration remain separate. Sources: `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`; the cue and carrier are warning-free.

### ChangeState wrapper separated from form/resource grants, October 18

Warning-free **Buff_Zankou_ChangeState#/1 → GC_Zankou_ChangeState** is an Infinite presentation marker; the scalar 120 is not a 120-second form expiry. OnWhile **#/4 → graph#/1:3110** selects fashion material maps, reads native logic **InvisibleSkillValue** into material opacity and calls **UpdateEmissiveColor(ChangeState,configured color)**. OnRemove **#/3 → 3645** restores the selected materials, requests an appearance emitter, resets its ChangeState color entry and reapplies the native combined emissive color to the mesh/outline. The color/opacity reads and writes do not locate Kindling generation, a Force/Magic ability grant or a second damage packet. The reached wrapper has no explicit such operation; native logic and the already recovered form/grant consumers remain required. Sources: `form-wrapper-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`; all cited reads are warning-free.

## Unknown execution policies

Native protected executor bodies, exact native armor capacities and live-client playback traces remain unavailable. Current consumed coordinates, recovered Blueprint requests and explicit scheduling/stack/recipient policies support the bounded combat model above. The optional armor fixture declares pressure rather than inventing enemy attacks. Overworld invisibility, non-player/summoned actor logic and geometry remain outside the simulator's stationary-dummy scope.

## Responsibility cards

- [vile-ash-dot-coefficient](vile-ash-dot-coefficient.md#unknown-execution-policies) contains the findings and questions for that distinct subsystem.

## Prior findings

- [`reviews/q10-game-action-binding/zankou-action-binding-model.json`](../../../../reviews/q10-game-action-binding/zankou-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [`zankou-dot-producer-model.json`](../../../../reviews/q10-game-action-binding/zankou-dot-producer-model.json), [owning Rust checks](../../../../crates/content/src/simulation/tests/zankou_actions.rs), [`zankou-normal-chain.json`](../../../../reviews/q10-game-action-binding/zankou-normal-chain.json).

## Research history

October17-20 investigations recovered packet rows, form/removal grants, cue calculations and shadow selectors. November14's current-client decode repaired the native logic layout and superseded the old fixed Skill, twelve-second stance, unconditional Vile Ash Burst and unbounded Kindling shortcuts. Raw hashes/inputs and receipts stay outside Git; game-version certification is not required. The current mechanics and named owner checks above supersede historical runtime limitations without relabeling bounded policies as recovered executor code.

### Current A1, Kindling and A4 lifecycle

A1 is an exact selected node, not a node-count threshold: permanent living-source Hunt replaces .25 with .40 and Delusion replaces .25 with .50. Both enhanced Skill spreading notifies require A1 (`GA_Zankou_Skill.OnReceiveEvent` object6, offsets528/579/715); ordinary Illusion Normal and held spreading does not. A2 stores at most one Kindling from actual Scorch/Hexed participation. Without A4, Bloodfeast leaves that stack for the free Inferno. With A4, paid-ready Bloodfeast captures it and both that Bloodfeast and its associated free Inferno get the additional150% final multiplier. A4 Bloodfeast also removes Illusion immediately. Free consumption/expiry, interruption, field exit and defeat discard captured permissions; actual defeat clears stored Kindling too. Two continuing paid/free pairs with renewed Cycle grants and two targets reproduce the exact5/2 multiplier on all24 eligible contacts, without AoE overgrant.

### Current A3 and A5 owner bindings

A3 Delusion heals only the living owner for four percent of live MaxHP each real second, through shared outgoing/incoming healing modifiers. An armed periodic phase survives action refresh; inactive Delusion at the tick, field cleanup without A1, or actual defeat stops healing. MaxHP10000 gives400 base,600 with50% healing bonus; the off-field A1+A3 owner receives1800 over three seconds while an injured ally receives zero.

`Calc_ZanKouLv3SuperArmor` returns.5 with Hunt+A3, otherwise1. The native armor pool/regen is not recovered. `zankou.combat_passives_v1` therefore exposes **explicit native fixtures** using `interrupt_events[].armor_damage`: one normalized armor unit per admitted action, incoming pressure multiplied by the live coefficient, and cancellation at accumulated1. Two unit contacts cancel A3 Hunt on the second, whereas no A3 cancels on the first. Omission retains unconditional forced cancellation. No rotation infers incoming attacks; the DSL gains no armor syntax. This finite model does not claim native armor capacity or regeneration.

A5 enhanced entry schedules its separate zero-HP SkillForce impulse at657891 montage progress. The owner records a target once per combat before applying999999 through the existing gauge/efficiency/Break settlement, including distinct AoE recipients. Both explicit combat enter/leave reset the ledger. Anonymous authored dummies are nonsummoned and default to a non-story context. Optional native `zankou.story_mode_v1` selects the recovered story-mode predicate: only story-mode bosses are excluded, not bosses in every context. A declared gauge/Break profile remains necessary for a Break result; the impulse does not fabricate HP damage or a gauge on an unmodeled target. Living A5 grants team Break damage+300%, including self; independent shared settlement gives exactly four times each living contributor's baseline and loses the contribution on source defeat.

### Current resonance and live-query bindings

Three selected nodes, independently of node3 identity, enable the native talent increment and.20 Ultimate coefficient addition. Six selected nodes, independently of merely selecting node6, enable the refreshing.40 ATK/20-World-second damage callback and60-second Scorch-defeat enhancement. R6 no longer invents Kindling on defeat or a50% Skill-cooldown refund. The enhanced Scorch policy sets the existing recipient carrier to its three-stack cap only before the real deadline and through a qualifying Zankou DoT application.

A6 binds **Skill3_2**, enhanced Soulcross's final contact, and no preceding/ordinary-return packet. It adds.12 per live target DoT stack, capped at20 stacks/2.40. Heartwrench/Vile Ash and Scorch use their actual tagged statuses; attached Nightmare uses its original owner's lazy one-stack decay and Adler uses its live per-target deadline. Living-source checks prevent counting defeated foreign owners. Two real AoE enhanced returns with only node6 versus no nodes reproduce the final-contact multiplier49/25: `(1+.25 Hunt+10*.12)/(1+.25 Hunt)`, with every other contact unchanged. Independent target fixtures retain differing counts and the cap.

The public queries are documented in `docs/NTE_DSL.md`: Reality/Illusion, remaining Illusion, enhanced entry/Soulcross, Bloodfeast, free Inferno and its remaining real time. A4 Bloodfeast availability includes real paid readiness; free expiry and readiness agree at the exact microsecond. Readiness selects with current combat time rather than descriptor timestamp0, and its read-only resource check has no fabricated authored command. Admitted native commands still require canonical Skill2/Ultimate1 identities and real costs/cooldowns; variants are selected by the owner. Source/native replay covers two complete held-entry, late-return and paid/free pairs, without script-owned form flags.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
