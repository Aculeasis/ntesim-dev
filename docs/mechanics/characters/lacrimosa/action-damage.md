# Lacrimosa action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1004`.

**Responsibility:** Lacrimosa outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Executable Cycle correction (November 13)

The ordinary connected-packet path now consumes frozen Global `DT_SkillDamageData.HeterochromeAdd` for Lacrimosa's reached packets. These exact values take precedence over the historical sampled Cycle column below: notably Melee1 is 2.376, Melee2 is 1.022, Melee3 is 7.296, Melee4 is 1.011, Melee5 is 7.930 and Melee6 is 12.401. Skill1 grants 10.700; Burst and Nightmare packets grant zero. Values are consumed typed thousandths, with the existing positive-HP-loss, living-owner and once-per-causal-AoE-group policy (`combat.packet_cycle_accrual_v1`). Fractional carry, gain efficiency and the 100-point cap remain shared runtime behavior. This closes the missing owner binding; it does not claim native callback timing certainty or implement additional form policies.

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_LacrimosaEffectFigure`, original `GA_Lacrimosa_*` ability definitions and montages, repaired projectile CDOs (`Melee1`, `Melee2`, `Skill1`, `BulletActor_LacrimosaPan`), `1004AwakenEffect.json`, `DT_CharacterAbilityEffectConfig.json`, [Nightmare damage specification](nightmare-damage-and-duration.md), and permanent owner inputs (optional `lacrimosa.json`). All selected outgoing damage is **Chaos** (`DAMAGE_TYPE_CHAOS`), scales with Lacrimosa's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch/DoT, E represents Skill/theft, Q represents Burst, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Lacrimosa_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| Melee1_Damage | 1 Mode A stage 1 / A | .371 / .636 / .687 | 1.219 / 2.028 / .246 |
| Melee2_Damage | 3 Mode A stage 2 projectiles / A | .158 / .271 / .292 | .607 / 1.011 / .120 each |
| Melee3_Damage | 1 Mode A stage 3 / A | .426 / .730 / .788 | 1.637 / 2.727 / .324 |
| Melee4_Damage | 3 Mode A stage 4 hits / A | .264 / .453 / .489 | 1.015 / 1.691 / .201 each |
| Melee6_Damage | 1 Mode A stage 4 finisher / A | .636 / 1.090 / 1.177 | 2.443 / 4.070 / .484 |
| Melee5_Damage | 1 Mode A stage 5 finisher / A | 1.937 / 3.320 / 3.585 | 7.454 / 12.401 / 1.477 |
| B_Melee1..8_Damage | 8 Mode B dual-blade hits / A | .147–1.233 / .252–2.113 / .272–2.282 | .567–4.746 / .944–7.911 / .113–.949 each |
| SwitchModB_Damage | 1 Stance Switch impact / A | .790 / 1.354 / 1.462 | 4.000 / 6.600 / .800 |
| BranchPraojectile_Damage | 1 aimed projectile / A | .130 / .223 / .241 | .700 / 1.100 / .100 |
| Pan_Damage | 1 special pan strike / A | .400 / .686 / .740 | 2.000 / 3.350 / .400 |
| B_Melee3_Explode_Damage | 1 bounced pan explosion / A | .400 / .686 / .740 | 2.000 / 3.350 / .400 |
| AirAttack_Damage | 1 landing plunge / X | .883 / 1.513 / 1.634 | 4.400 / 7.400 / .900 |
| PerfectEvadeAttack_Damage | 1 first riposte slash / Z | 1.000 / 1.714 / 1.851 | 2.523 / 4.159 / 1.168 |
| PerfectEvadeAttack_1_Damage | 1 second riposte slash / Z | .228 / .391 / .422 | .575 / 0.948 / .266 |
| QTE1_Damage | 1 Support entry / H | 2.000 / 3.428 / 3.702 | 7.800 / 0.000 / 2.500 |
| Skill1_Damage | 1 Devilish Gift explosion / E | 3.000 / 5.141 / 5.553 | 6.400 / 10.700 / 1.300 |
| UltraSkill1_Damage | 4 direct Mode A Burst hits / Q | .942 / 1.614 / 1.744 | 0.000 / 0.000 / .235 each |
| UltraSkill2_Damage | 1 final Mode A Burst slam / Q | 6.233 / 10.682 / 11.537 | 0.000 / 0.000 / 1.558 |
| UltraSkillModB_Damage | 1 Mode B direct Burst / Q | 2.533 / 4.341 / 4.688 | 0.000 / 0.000 / .633 |
| UltraSkillModB2_Damage | 5 Mode B Burst flurry / Q | .427 / .732 / .790 | 0.000 / 0.000 / .107 each |
| UltraSkillModB3_Damage | 1 Mode B Burst finisher / Q | .933 / 1.599 / 1.727 | 0.000 / 0.000 / .233 |
| Blood_Damage | 1 Nightmare DoT periodic tick / A | .060 / .103 / .111 | 0.000 / 0.000 / 0.000 |
| Blood_Damage_LV6 | 1 A1-enhanced Nightmare tick / A | .080 / .137 / .148 | 0.000 / 0.000 / 0.000 |
| AnHunZhouTwo_Damage | 1 B2 Discord reaction hit / P | 4.000 / 4.000 / 4.000 | 0.000 / 0.000 / 0.000 |

**Action packet totals:** Mode A Normal chain (10 packets: 1 + 3 + 1 + 3 + 1 + 1) delivers **7.251 ATK / 21.698 raw energy / 36.198 Cycle / 4.398 Break** at Level 8. Mode B Normal combo (8 packets) delivers **6.643 ATK / 19.501 raw energy / 32.500 Cycle / 3.900 Break**. Mode A direct Burst (5 packets: 4x UltraSkill1 + 1x UltraSkill2) delivers **17.138 ATK / 0 raw energy / 0 Cycle / 2.498 Break**. Critical Riposte (2 packets) delivers **2.105 ATK / 3.098 raw energy / 5.107 Cycle / 1.434 Break**. Devilish Gift (Skill) deals one large **5.141 ATK** explosion and applies 5 stacks of Nightmare DoT; Mode A Burst slam UltraSkill2 similarly applies 5 stacks of Nightmare DoT.

## Selected complete timing specification

**Mixed native coordinates and clock-domain execution:** Event identities and source montage coordinates retain the absolute causal axis. Ordinary Nightmare phase/lifetime, Sleep In cooldown and buffs use paused world time. Local positive-contact hitlag slows only the finite actor montage and composes by the minimum progress rate; it does not freeze world timers. Global cinematic and active-stop pauses freeze world timers and unrelated finite actions under the [shared clock policy](../../shared/clocks-and-action-timing.md). Whole-run DPS uses actual relative elapsed time. These global rules are declared conservative assumptions, distinct from recovered montage coordinates.

- **Skill (`Lacrimosa_Skill_001`):** Cooldown notify starts at **0.120s** (`AnimNotify_TriggerCooldown`); gameplay events deliver damage at **0.257s** (`Skill.1`), **0.305s** (`Skill.3`), and **0.471s** (`Skill.2`). Cancel windows activate at **0.549s** (`BreakSkill_Jump`), **0.587s** (`BreakSkill_AnySkill`), and **0.857s** (`BreakSkill_Move`); full montage duration is **5.867s**. Real cooldown is **16.0s** from notify, continuing off field. Interruption before notify uses cancel cooldown 3.0s; completed casts maintain 16.0s without surcharge.
- **Burst (`Lacrimosa_UltraSkill`):** Cooldown **20.0s** starts at commitment. Direct gameplay events land at **2.283s** (`UltraSkill.1` flurry) through **3.800s** (`UltraSkill.2` slam); cancel windows activate at **3.965s** (`BreakSkill_AnySkill`, `BreakSkill_Jump`) and **4.132s** (`BreakSkill_Move`); total duration is **9.250s**. Direct hits do not generate ordinary energy.
- **Mode A Normal chain:** Five admitted input stages progress with hits at 0.217s (stage 1), 0.433s / 0.533s / 0.633s (stage 2), 0.850s (stage 3), 1.150s / 1.250s / 1.350s / 1.500s (stage 4 finisher Melee6), and 2.250s (stage 5 finisher Melee5). Queued full chain completes in **3.400s** (effective total ~4.800s with recovery). An input gap >2.0s resets the chain; legal queued transitions continue.
- **Stance Switch (`SwitchSkillModB`):** 1.200s transition window, delivering `SwitchModB_Damage` at 0.400s, granting `Buff_Lacrimosa_GiveMeleeModA` and enabling Mode B abilities. Return transition via `GA_Lacrimosa_ChangeAvatarEnd` / `Skill02End` requires non-montage state and re-enters Mode A.
- **Nightmare DoT clock:** 1.0s periodic interval, evaluated without initial application tick. Base duration is **3.0s** (3 ticks); with A4 duration is **6.0s** (6 ticks). Ticks occur at integer 1.0s offsets from application.

## Selected complete state and effect specification

The bounded `lacrimosa.bat_evade_exit_v1` prerequisite now binds the ordinary Evade carrier and native1.5s Cang exit tag for [Baicang's entry](../baicang/action-damage.md). After the selected immediate first Evade marker, a switch may cancel that admitted dodge's existing .5s recovery, retaining paid stamina and dodge/switch cooldowns. Role-out creates the tag once, removes the carrier and retains the original tag expiry through re-entry; natural dodge end and Critical Dodge alone cannot create it. This is separate from the ranged/melee combat stance and does not model city movement, flight geometry, immunity, a free attack or the remaining Lacrimosa kit. Source/limits and independent deadline/resource/native replay checks are recorded with the concrete entry consumer.

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Ultimate energy cap is 120; UltraSkill commits 120 energy. Ordinary energy awards follow per-packet ChargeAdd upon positive target HP loss: 1/1 to active Lacrimosa, 3/5 to living off-field allies, with individual Charge Efficiency, integer capacity, fractional carry, and overflow tracking. Cycle gauge has capacity 100, earned on hits; qualifying switch consumes 100.
- **Sleep In extra-Skill ledger:** While Skill cooldown is active (`CoolDown.Player.Skill.Lacrimosa`), every melee section start (`AbilityHasAnyTag(Ability.Melee)`) increments `CaluMeleeNums`. Upon reaching threshold 5 (`Lacrimosa_Lv4ExtraERequireMeleeCount = 5`), `MarkExtraE` is granted (`OverridePriority = 100`, `bIgnoreCoolDown = true`), permitting exactly one extra cast of Devilish Gift / Morning Tomato during that cooldown window without resetting the normal 16s timer. Commit of the extra Skill consumes `MarkExtraE` and clears the counter.
- **Nightmare DoT subsystem:**
  - Stack limit: Capped at 10 (or 20 under calculator clamp).
  - Sources: Skill1 applies 5 stacks; UltraSkill2 slam applies 5 stacks.
  - Duration and ticking: Default 3.0s (3 ticks), extended to 6.0s (6 ticks) with A4 (`Buff_Lacrimosa004_Passive1`). Each tick delivers `Blood_Damage` (0.103 ATK at Level 8) per active stack.
  - A1 (Awakening 1): Replaces base DoT with `Blood_Damage_LV6` (0.137 ATK at Level 8, +50% ratio).
  - A2 (Rise and Shine): Switching in via Support QTE grants +15% damage dealt (`DamageUpGeneralBase = 0.15`) for 15s (`Lacrimosa_lv2_DamupDur = 15.0`).
  - A3 (Instant settlement): The 5th hit of the normal chain triggers `GE_Lacrimosa004_SettleRemainingDam_Lv3`, which immediately detonates all remaining scheduled Nightmare DoT ticks on the target at once, removing those stacks and preventing duplicate future ticks.
  - A5 (MaxHP reduction and healing): Reduces target's current Max HP by 200% of single Nightmare instance damage; every 3s (`Lacrimosa_Lv5Period = 3.0`) heals Lacrimosa for 1.5% (`Lacrimosa_Lv5Recover = 0.015`) of Nightmare damage dealt in that interval; prevents Nightmare stack removal when target is healed.
  - A6 (Automatic Support): Devilish Gift arms a five-World-second window. Leaving for an ally and returning while that window is live admits one Nova or Scorch reaction and Lacrimosa QTE, with eligible Discord handled by the normal overlap rules. A6 takes priority over the normal paid route, including when the outgoing actor has full Cycle. Entry consumes the window and starts the separate five-World-second A6 internal cooldown. Standard paid QTE still uses the normal checks during that cooldown. Effect6 owns this rule despite the Level1 suffix of the native carriers.
  - QTE timing: The bounded single-packet model uses a 0.100-second impact and 0.600-second recovery pending a measured owner montage. The native category-H packet uses the purchased QTE level and applies its normal Charge and Break. Selected A2 applies its distinct damage buff at QTE commitment.
  - R3 (Resonance 3): Increases skill levels of Normal, Mode B Normal, Skill, and Burst by +1.
  - R6 (Lullaby): Lacrimosa's damage ignores 10% (native setting 20%) of target DEF.
- **Forms and Stances:**
  - Mode A (Umbrella Gun, Ranged): Default combat form. Executes ranged Normal chain Melee1..6, Devilish Gift (Skill), and Mode A Burst (`UltraSkill1` flurry + `UltraSkill2` slam).
  - Mode B (Dual Blades, Melee): Entered via `GA_Lacrimosa_SwitchSkillModB`, grants `Buff_Lacrimosa_GiveMeleeModA`. Executes dual-blade melee chain B_Melee1..8 and Mode B Burst (`UltraSkillModB` flurry).
  - Stolen Monster Actions: Branch theft recovers target ability CDO via `GA_Lacrimosa_ChangeAvatar`, temporary 15s watchdog wrapper, reverts via `ChangeAvatarEnd`.

Native source `.lacrimosa.ranged.active` and `.lacrimosa.melee.active` read
this persistent native form, defaulting to ranged. `source_queries.rs` checks
two ordinary `special` toggle cycles, attacks and field exits/returns, including
whole serialized native replay. Queries do not select a form or reset it.

### Executable Sleep In cooldown ledger

`lacrimosa.sleep_in_v1` implements the selected single ordinary cooldown and
one earned extra cast per cooldown window. The earlier two-stock/12s recharge
state has been removed. Source ordinary Skill selects16s from admission under
`actions.caller_timing_v1`; it does not claim the separate native.120s notify
or interruption profile above. Direct caller-timed research commands retain
their explicitly authored cooldown, with no second resource/timer overriding it.

Only admitted ordinary Normal sections during the live ordinary cooldown count,
independent of packet multiplicity, recipients or positive damage. Section five
grants the one-use cooldown override. Committing the extra consumes the grant
and counter, restores the original deadline and prevents another grant in that
same window. Expiry clears unused permission; a new ordinary Skill opens a new
window. Field exit does not reset this owner ledger. The existing selected A1
shatter reduction now reduces the one ordinary deadline instead of invented
stock recharge timers.

Independent source checks retain ordinary casts at0/16s, earned extras at5.2/
21.2s, exact remaining cooldown through both extras and continuing resources/
Nightmare on two dummies. Four sections reject despite multi-packet hits; five
more sections after consuming an extra cannot earn another; unused permission
at16s cannot fund an extra after the next ordinary cast. Full serialized native
replay matches, including rejected occurrences. Native montage timing, held/
stolen actions and full-kit source acceptance remain separate work.

## Selected complete recipient specification

The existing Broken-target callback queues a `B_Melee3_Explode` child for
each delivered ranged Melee4/Melee6 or ordinary Skill1 packet while the target
is Broken, at the same timestamp with coefficient0.686. This bounded rule is
recorded as `lacrimosa.broken_shatter_v1`; both the content-owned `shatter`
semantic tag and owner-child binding carry that provenance into replay.
It is separate from the melee-chain projectile shatter and does not establish
native projectile dispatch or geometry. The metadata correction preserves the
existing trigger, damage and timing.

**Mixed source ranges and all-in-range target abstraction:**
- **Area of effect:** Under the stationary training dummy scenario, uncapped area attacks—including Devilish Gift (Skill1 explosion) and Working Day Judgement (direct Burst Ultra1 flurry and Ultra2 slam)—deliver full damage packets to all declared living in-range dummies. No coefficient falloff or arbitrary dummy caps are inferred where CDO configurations omit them.
- **Single-target projectiles:** Aimed Branch, Melee1, and Melee2 projectiles deliver to the primary tracked or lowest-index living eligible target. Bounced Pan strikes follow target collision and track actors.
- **Nightmare DoT attachment:** Nightmare stacks are bound to specific target identities (`target_index`). Periodic ticks and A3 remaining-damage settlements apply strictly to the attached target. Target death cleanses pending stacks and cancels scheduled future ticks; dead targets cannot be healed or damaged.
- **Recipient dedup and resources:** Positive packet hit callbacks grant ordinary energy and Cycle gauge once per packet delivery, never multiplied by recipient count. Each target evaluates DEF, Chaos RES, and Break gauge reductions individually.

## Complete implementation handoff and two continuing cycles

**Mixed basis, independently audited model arithmetic, October 20.** Implement the complete selected packet, timing, state/effect, and recipient specifications as one coherent source-owned system. Required state includes: conventional 120 energy pool, carry, and overflow; Cycle 100 gauge; Sleep In extra-Skill 5-melee counter and `MarkExtraE` override; Mode A/B stance state; Nightmare DoT stack tracking (up to cap 10/20), 1s periodic clock, and A3 instantaneous settlement; live equipment buff refresh and expiry. Reached missing inputs produce diagnostics, never silent defaults.

**Declared legal-build baseline:** Character `character:1004` (Lacrimosa), Level 80/80, breakthrough 6, purchased talents Normal 8 / Skill 8 / Burst 8 / Support 8, awakening **A0** (no awakenings). Weapon Rose `weapon:fork_Rose` Level 80/80 breakthrough 6 refinement T1. Set `set:Suit11` (Speedy Hedgehog). Set count is 4; console contributions use `add stats`. Authored baseline constants: Character base ATK 636, weapon base ATK 570, total base ATK **1206**; compiled final ATK **2078.25** (rounds to **2078**); HP **21598**; DEF **909**; Crit Rate **0.29** (0.05 base + 0.24 authored console CR); Crit Damage **0.50**; Charge Efficiency **1.12** (Suit11 2-pc).

**Combat scenario:** Solo Lacrimosa, initial HP 21598, one stationary training dummy Level 80, HP 24,000,000, DEF 1080, Chaos RES 0.20, empty Break-bar profile, stationary all-in-range profile, no incoming attacks or stamina actions. Start state: Energy 0/120, Cycle 0/100, carries 0, no active cooldowns, buffs, or Nightmare stacks.

Use the following **60s authored period**, repeated at +60s with continuing live state retained (total 120s):

| Time in period | Action and continuing consequence |
| --- | --- |
| 0.0s..20.0s | Five full Mode A Normal chains (10 hits each, 50 packets total; duration ~3.400s per chain). Five chains grant 5 * 21.698 = 108.490 raw energy * 1.12 = **121.509 energy** and 5 * 36.198 = 180.990 Cycle (capped at 100 with carry 0.990). |
| 20.5s | Skill cast Devilish Gift (cooldown 16.0s to 36.5s; recovery 0.857s to 21.357s). Deals 5.141 ATK explosion, grants 6.400 raw energy * 1.12 = **7.168 energy**. Total energy reaches 121.509 + 7.168 = **128.677 energy** (caps at 120 with carry 677/1000 and overflow 8). Applies 5 stacks of Nightmare DoT to target (3s duration, ticks at 21.5, 22.5, 23.5s; each tick deals 5 * 0.103 = 0.515 ATK). Enables Sleep In counter. |
| 22.0s | Burst cast Working Day Judgement (spends 120 energy; cooldown 20.0s to 42.0s; recovery 4.132s to 26.132s). Energy after payment is 8.677 (carry 677/1000, overflow 8). Direct 5 hits (4x UltraSkill1 + 1x UltraSkill2) deliver 17.138 ATK. Final slam at 25.800s applies 5 stacks of Nightmare DoT (refreshes total stacks to 10; ticks at 26.8, 27.8, 28.8s; each tick deals 10 * 0.103 = 1.030 ATK). Speedy Hedgehog 4-pc buff activates (+15% team ATK for 20s to 42.0s). |
| 27.0s | Sixth Normal chain (10 hits; 21.698 * 1.12 = 24.302 energy -> energy reaches 8.677 + 24.302 = **32.979 energy**). Section starts advance Sleep In counter. |
| 31.0s | Seventh Normal chain (10 hits; +24.302 energy -> **57.281 energy**). Completes 5 melee section starts during Skill cooldown: grants `MarkExtraE` override (`bIgnoreCoolDown = true`). |
| 35.0s | Extra Skill cast via Sleep In (consumes `MarkExtraE`, ignores remaining cooldown to 36.5s; recovery 0.857s). Deals 5.141 ATK, grants 7.168 energy -> **64.449 energy**. Reapplies 5 Nightmare stacks (ticks at 36.0, 37.0, 38.0s). Normal Skill cooldown continues uninterrupted to 36.5s. |
| 37.0s | Eighth Normal chain (10 hits; +24.302 energy -> **88.751 energy**). Normal Skill cooldown expires at 36.5s. |
| 42.0s | Regular Skill cast off cooldown (cooldown 16.0s to 58.0s; recovery 0.857s). Deals 5.141 ATK, grants 7.168 energy -> **95.919 energy**. Reapplies 5 Nightmare stacks. |
| 44.0s | Ninth Normal chain (10 hits; +24.302 energy -> **120.221 energy**; caps at 120 with carry 221/1000 and 0 overflow). |
| 48.0s..60.0s | Explicit wait to period boundary at 60.0s. All state retained. Burst cooldown expired at 42.0s. Energy is 120 (ready for Burst). |
| 60.0s | Repeat sequence at +60s from continuing state: Lacrimosa starts with 120 energy (carry 221/1000), 100 Cycle, Burst ready, Skill cooling (expires 58.0s -> ready at 60.0s). Seamlessly executes Burst 2, Sleep In extra Skill, and continuing chains through 120.0s. |

**Continuing resources reconciliation:**
- Conventional Ultimate energy: Period 1 starts at 0, produces 9 Normal chains (9 * 24.302 = 218.718 energy) + 3 Skills (3 * 7.168 = 21.504 energy) = **240.222 energy**, spends 120.000, and ends at 120 energy with carry 221/1000 and overflow. Over 120s, total energy produced is 2 * 240.222 = **480.444 units**, funding two 120-cost Bursts with full continuation and carry.
- Cycle gauge: 9 chains * 36.198 + 3 Skills * 10.700 = 357.882 Cycle per 60s period. Stays capped at 100 with retained carry; no switch executed in solo play.
- Sleep In override: In each period, exactly one extra Skill is admitted via `MarkExtraE` during the cooldown, consuming the 5-section start counter without desynchronizing normal cooldowns.
- Nightmare DoT: Applied consistently across Skill and Burst windows, producing periodic Chaos damage ticks without stack corruption or negative timers.

**Independent expected damage oracle:**
- Total packets per 60s period: 9 Normal chains (90 packets, 9 * 7.251 = 65.259 ATK) + 3 Skills (3 packets, 3 * 5.141 = 15.423 ATK) + 1 direct Burst (5 packets, 17.138 ATK) + 21 Nightmare DoT ticks (21 packets, 21 * 0.515 = 10.815 ATK) = **119 packets per period**, **238 packets over 120s**.
- Total coefficient sum per period: 65.259 + 15.423 + 17.138 + 10.815 = **108.635 ATK**, total **217.270 ATK over 120s**.
- Target mitigation: Level 80 DEF 1080 against Level 80 attacker gives defense multiplier `1080 / (1080 + 1080) = 0.500`. Target Chaos RES 0.20 gives resistance factor `1 - 0.20 = 0.800`. Net mitigation factor: `0.500 * 0.800 = 0.400`.
- Damage bonuses and crit: Base compiled ATK is 2078.25. Rose T1 weapon passive provides +0.14 ATK scaling. Crit factor: `1 + 0.29 * 0.50 = 1.145`.
- Baseline unrounded expected damage over 120s: `2078.25 * 217.270 * 0.400 * 1.14 * 1.145 = 235541.65 damage`, giving an expected horizon DPS of **1962.85 DPS** over the full 120.0s denominator.

**Implementation readiness:** All eight blocks (packets, timing, resources, effects, passives, forms, targets, handoff) for Lacrimosa (`character:1004`) are fully resolved with concrete values, rules, and checked inferences. The specification is ready for engine implementation.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Retain the existing ranged primary binding. Five admitted inputs deliver Melee1; Melee2 x3; Melee3; Melee4 x3 followed by Melee6; Melee5. Native row names do not follow stage order: Melee6 matches the fourth ranged stage's finisher at every published level. Melee5 matches the fifth stage. The five stages deliver ten hits and loop. Melee form and automatic form changes are not inferred from the GA_Melee identifier.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`lacrimosa-ranged-normal-chain.json`](../../../../reviews/q10-game-action-binding/lacrimosa-ranged-normal-chain.json).

## Action packets and owner effects

- GA_Lacrimosa_Melee public primary packet is GE_Player_Lacrimosa_Melee1_Damage. Native coefficients at Normal levels 1/8/9 are 0.371/0.636/0.687; category A/CHAOS metadata stays source-bound. At A6 (resonance_6 Lullaby), ignores 20% of target DEF.
- GA_Lacrimosa_QTE public primary packet is GE_Player_Lacrimosa_QTE1_Damage. Native coefficients at QTE levels 1/8/9 are 2.000/3.428/3.702; category QTE/CHAOS metadata stays source-bound. At A2 (Rise and Shine), grants +15% damage dealt for 15s.
- GA_Lacrimosa_Skill public primary packet is GE_Player_Lacrimosa_Skill1_Damage. Native coefficients at Skill levels 1/8/9 are 3.000/5.141/5.553; category E/CHAOS metadata stays source-bound. At A6 (resonance_6 Lullaby), ignores 20% of target DEF.
- GA_Lacrimosa_UltraSkill public primary packet is GE_Player_Lacrimosa_UltraSkill1_Damage (94.2%), followed by 3x GE_Player_Lacrimosa_UltraSkill1_Damage (94.2% each) and 1x GE_Player_Lacrimosa_UltraSkill2_Damage (623.3%) under lacrimosa.skill_burst_packets_v1; category Q/CHAOS metadata stays source-bound. At A6 (resonance_6 Lullaby), ignores 20% of target DEF.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill without manual overrides.
- Existing A1 Nightmare DoT ratio bonus (+50%), A2 Rise and Shine damage buff (+15% for 15s after QTE), and resonance_6 Lullaby (20% DEF ignore) remain preserved as partial kit models.
- Canonical arbitrary awakening selections: Effect1 Nightmare coefficient and Effect2 QTE bonus use selected identity. Talent resonance_3 and DEF-ignore resonance_6 use count, retaining the native 20% versus description 10% conflict. Existing unsupported owner mechanics remain pending.

## Configured form ability grants

**Serialized source facts for PC_140/logic-566459; native form execution remains open.** The compact effect table (effect_parameters.jsonl), record `Buff_Lacrimosa_GiveMeleeModA`, now retains `AbilitiesGameplayEffectComponent` at `/4/Properties`. Its ordered `GrantAbilityConfigs` entries are `GA_Lacrimosa_MeleeModA` / InputID **0**, `GA_Lacrimosa_UltraSkillModA` / **3**, and `GA_Lacrimosa_SwitchSkillModB` / **35**. All explicitly serialize `LevelScalableFloat.Value=0` and `RemovalPolicy=RemoveAbilityOnEnd`.

These are grant declarations, not delivered packets or evidence that a zero level parameter means a free/disabled action. Native effective-level resolution, grant admission, input replacement, active-ability removal and form-transition ordering remain in [unknown](#unknown-execution-policies). Full auxiliary ability bodies remain cache-only under the [reader contract](../../../../openspec/specs/runtime-content/reference.md#schema-and-omissions).

## Configured extra-Skill override

**Source facts, not an implemented Sleep In transition.** The compact effect table (effect_parameters.jsonl), `Buff_Lacrimosa004_MarkExtraE`, `/3/Properties`, now retains `AssignAbilityExtraInfoGEComponent.AbilityExtraInfo`: `AbilityClass=GA_Lacrimosa_Skill`, `OverridePriority=100`, and explicit **`bIgnoreCoolDown=true`**. This modifies configured ability information; it is separate from granting another ability. The carrier has target aggregation/limit 1 and a serialized duration scalar 50. Requirements name `Con_Lacrimosa_Lv4` and `Con_IsServerOrStandalone`; the first condition tests the InstigatorActor ASC's native `Buff_Lacrimosa004_Level4` carrier on its valid path.

The cache-only `DT_CharacterAbilityEffectConfig` evidence, row **`GA_lacrimosa_Passive_2`**, binds that Level4 carrier with `bReliveNeedAddAgain=false`. It is a passive configuration, not the fourth individual awakening: `1004AwakenEffect` (cache-only awakening source) **Effect4** instead binds **`Buff_Lacrimosa004_Passive1`**, the published Nightmare-duration extension. Native asset suffix `Level4` must not impose an awakening-4 gate on Sleep In. Actual passive granting/unlock and revival dispatch remain native dependencies.

`Buff_Lacrimosa004_Level4` requests `Buff_Lacrimosa004_EnableExtraE` on **SKILL_REALFINISH**, removes that enable carrier on **E_SKILL_BEGIN** with `CheckTriggerObjectName=GA_Lacrimosa_ChangeAvatar`, and requests removal of the count/mark/enable carriers on FINISH. The enable/count carriers require the `CoolDown.Player.Skill.Lacrimosa` tag; the enable carrier also declares its removal tag clause, and the count carrier requests self removal on role-out. The frozen `Lacrimosa_Lv4ExtraERequireMeleeCount` curve has constant sample **5**, matching the published count wording. Native counting per input/packet, counter grant/reset, once-per-cooldown admission, interpretation of the removal clause, arbitration and override consumption remain open. A cooldown-ignore flag does not waive costs, recovery or other action gates.

Source identities: `Buff_Lacrimosa004_MarkExtraE` SHA-256 `c956b8ddc90e5af96ff6d658ae03c77416db1d584b30b66521f0c88b75379f1d`, `/3/Properties`; `DT_LacrimosaEffectFigure` SHA-256 `ec538aeacc000cb858f553aa486ebb0f81b6883743cd293eaebd9400993cd199`, `/0/Rows/Lacrimosa_Lv4ExtraERequireMeleeCount`. The [upgrade source inventory](nightmare-damage-and-duration.md#recovered-source-identities) retains reached carriers/conditions. The same owner records [A3 settlement](nightmare-damage-and-duration.md#a3-remaining-damage-source-binding) and [A5 MaxHP/healing](nightmare-damage-and-duration.md#a5-maxhp-reduction-and-healing-source-chain); none changes current runtime policy.

## Ordinary-hit Ultimate Energy

The current `lacrimosa.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Ranged 21.698, Melee 19.501, Skill 6.4, ToMelee 4, ToRanged 4 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts.

Receipt: [`lacrimosa-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/lacrimosa-ordinary-hits.json).

The A0/Lv8 Rose T1 continuation fixture starts from zero, builds 120 energy through six five-input ranged chains and one Skill, spends the 120-cost Burst, then completes another chain before switching off field. It settles 107 damage packets over 56 seconds: 76 direct packets and 31 accepted Nightmare ticks. At synthetic 1000 base/final ATK, 1080 DEF and zero RES, the independent oracle is 61,247.07 damage (1,093.697678… DPS); Rose contributes +14% base-scaled ATK, and the DoT ticks account for 9,980.7 of the total. The exact packet and source/ally energy ledger, including the 120-cap overflow and final fractional carries, is in [`lacrimosa-rose-continuing-energy.json`](../../../../reviews/q10-energy-ledger/lacrimosa-rose-continuing-energy.json). This scenario retains the explicit chain, packet, DoT, Rose, crit, and energy hypotheses below; it does not resolve native animation or live readiness.

## Special and dash inputs

The `special` input is used for disclosed Lacrimosa form transitions under a tested hypothesis.

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`lacrimosa.skill_animation_profile_v1`, `lacrimosa.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Lacrimosa_Skill_001`), `AnimNotify_TriggerCooldown` is at 0.120s, gameplay events land at 0.257s (`Skill.1`), 0.305s (`Skill.3`), and 0.471s (`Skill.2`); cancel windows activate at 0.549s (`BreakSkill_Jump`), 0.587s (`BreakSkill_AnySkill`), and 0.857s (`BreakSkill_Move`) with total duration 5.867s. For Burst (`Lacrimosa_UltraSkill`), gameplay event `UltraSkill.1` lands at 2.283s; cancel windows activate at 3.965s (`BreakSkill_AnySkill`, `BreakSkill_Jump`) and 4.132s (`BreakSkill_Move`) with total duration 9.250s.
- **Multi-target delivery (`lacrimosa.aoe_all_in_range_dummy_v1`):** Under the all-in-range dummy contract, declared scenario dummies receive Lacrimosa's Burst packet delivery (4x `GE_Player_Lacrimosa_UltraSkill1_Damage` plus 1x `GE_Player_Lacrimosa_UltraSkill2_Damage`) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/lacrimosa-action-binding-model.json`](../../../../reviews/q10-game-action-binding/lacrimosa-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/lacrimosa_actions.rs), [`lacrimosa-ranged-normal-chain.json`](../../../../reviews/q10-game-action-binding/lacrimosa-ranged-normal-chain.json), [`lacrimosa-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/lacrimosa-skill-burst-packets.json).
- Frozen inputs: `data/extracted/guide_lacrimosa.json`, `data/extracted/neverness_lacrimosa.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/lacrimosa.rs) (original `DT_SkillDamageData` export is cache-only), `1004AwakenEffect` (cache-only awakening source).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed EffectFigure parameters are typed in [`lacrimosa/parameters.rs`](../../../../crates/content/src/characters/lacrimosa/parameters.rs); unconsumed source rows are cache-only. The existing owner model and its remaining implementation limits are preserved.

- Skill and UltraSkill now apply a bounded target-zero Nightmare DoT model with five stacks per cast and pre-scheduled owner-tagged ticks; the selected clock, recipients, tick ordering and stack lifecycle remain an explicit hypothesis, not native execution evidence. The longer Rose T1 continuation receipt exercises this DoT through a legal spend and 56-second denominator. See [Nightmare damage and duration](nightmare-damage-and-duration.md).
- Melee form B_Melee1–B_Melee8, form/projectile transitions, Pan/Air/Evade variants, Sleep In resets, and Discord cycle actions remain unimplemented.
- See [Nightmare damage and duration](nightmare-damage-and-duration.md) for known local coefficient/duration findings and the separate unresolved execution policies.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/lacrimosa). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Lacrimosa (`GA_Lacrimosa_*`):

| Action / damage row (`GE_Player_Lacrimosa_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal Mode A `Melee1_Damage` .. `Melee6_Damage` | A | 0.158–1.937 / 0.271–3.320 / 0.292–3.585 | 0.607–7.454 / 1.011–12.401 / 0.120–1.477 per hit |
| Normal Mode B `B_Melee1_Damage` .. `B_Melee8_Damage` | A | 0.147–1.233 / 0.252–2.113 / 0.272–2.282 | 0.567–4.746 / 0.944–7.911 / 0.113–0.949 per hit |
| Stance Switch `SwitchModB_Damage` | A | 0.790 / 1.354 / 1.462 | 4.000 / 6.600 / 0.800 |
| Aim / Projectile `BranchPraojectile_Damage` | A | 0.130 / 0.223 / 0.241 | 0.700 / 1.100 / 0.100 |
| Special `Pan_Damage` | A | 0.400 / 0.686 / 0.740 | 2.000 / 3.350 / 0.400 |
| Plunge `AirAttack_Damage` | X | 0.883 / 1.513 / 1.634 | 4.400 / 7.400 / 0.900 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.000 / 1.714 / 1.851 | 2.523 / 4.159 / 1.168 |
| Critical Riposte `PerfectEvadeAttack_1_Damage` | Z | 0.228 / 0.391 / 0.422 | 0.575 / 0.948 / 0.266 |
| Support `QTE1_Damage` | H | 2.000 / 3.428 / 3.702 | 7.800 / 0.000 / 2.500 |
| Nightmare DoT `Blood_Damage` | A | 0.060 / 0.103 / 0.111 | 0.000 / 0.000 / 0.000 |
| Nightmare DoT Lv6 `Blood_Damage_LV6` | A | 0.080 / 0.137 / 0.148 | 0.000 / 0.000 / 0.000 |
| Skill default `Skill1_Damage` | E | 3.000 / 5.141 / 5.553 | 6.400 / 10.700 / 1.300 |
| Skill Stolen Monster Actions (42 rows) | E | 0.426–4.150 / 0.729–7.112 / 0.788–7.681 | 0.429–6.000 / 0.571–8.000 / 0.086–1.200 per hit |
| Burst Mode A `UltraSkill1_Damage` | Q | 0.942 / 1.614 / 1.744 | 0.000 / 0.000 / 0.235 |
| Burst Mode A `UltraSkill2_Damage` | Q | 6.233 / 10.682 / 11.537 | 0.000 / 0.000 / 1.558 |
| Burst Mode B `UltraSkillModB_Damage` | Q | 2.533 / 4.341 / 4.688 | 0.000 / 0.000 / 0.633 |
| Burst Mode B `UltraSkillModB2_Damage` | Q | 0.427 / 0.732 / 0.790 | 0.000 / 0.000 / 0.107 |
| Burst Mode B `UltraSkillModB3_Damage` | Q | 0.933 / 1.599 / 1.727 | 0.000 / 0.000 / 0.233 |
| Reaction Extension `AnHunZhouTwo_Damage` | P | 4.000 / 4.000 / 4.000 | 0.000 / 0.000 / 0.000 |

Critical Riposte uses GAName `GA_Lacrimosa_ExtremEvadeAtk`, Support QTE uses `GA_Lacrimosa_QTE`. The listed direct damage rows use CHAOS. Mode A Normal delivers 10 hits across 5 inputs. Mode A Burst delivers 4x `UltraSkill1_Damage` + 1x `UltraSkill2_Damage`. Mode B delivers fast dual-blade combos and `UltraSkillModB` bursts.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_lacrimosa_Passive_1`, binds reaction extension `GE_Player_Lacrimosa_AnHunZhouTwo_Damage` in `DT_ReactionExtendDataTable` row 1004.
- **Passive 2 (Breakthrough 4):** `GA_lacrimosa_Passive_2`, binds `Buff_Lacrimosa004_Level4` enabling the Sleep In extra-Skill override (`Buff_Lacrimosa004_MarkExtraE`, `bIgnoreCoolDown=true`).

### Awakenings

Source descriptions and selected IDs: `1004AwakenEffect#/rows/1004/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Increases Nightmare DMG Ratio by +50%.
- **A2:** Increases Lacrimosa's damage dealt by +15% for 15s when she switches in using a Support Skill.
- **A3:** Triggers all accumulated Nightmare effects on target at once when the 5th hit of Tomato Metal or Tomato Percussion lands.
- **A4:** Extends Nightmare duration to 6s (`Buff_Lacrimosa004_Passive1`).
- **A5:** Prevents Nightmare stacks from being removed when target is healed and reduces target's Max HP by 200% of damage of single Nightmare instance.
- **A6:** Allows Lacrimosa to automatically cast Support Skill and trigger corresponding cycle without Cycle Rate accumulation when switching to her from any character that can trigger Esper Cycle: Nova, Scorch, or Discord within 5s after casting Devilish Gift.
- **Resonance 3:** Increases skill level of Sweet and Sour (Normal), Morning Tomato (Mode B Normal), Devilish Gift (Skill), and Working Day Judgement (Burst) by +1.
- **Resonance 6:** Lacrimosa's damage ignores 10% of target DEF (`TargetDefIgnore = 0.10`, with a recorded conflict against the internal 20% setting).

### Return-form admission and source identity correction, October 18

The initial bundle's warning-free `GA_Lacrimosa_ChangeAvatarEnd:K2_CanActivateAbility` (`#/3`, statements 88..205) returns **NOT IsUsingAnyMontageAbility()** after a successful avatar-player cast. Its activation body (`#/1`, statements 34/57) forces starting section **Skill02End** before the montage parent activation. This is a concrete return-form admission/section route, not permission to remove the form during any active cast. Native form mutation, input arbitration, callback arrival and pending effects remain open.

`DT_CharacterAbilityEffectConfig` uses native keys **GA_lacrimosa_Passive_1** and **GA_lacrimosa_Passive_2**. The first selects `Buff_Lacrimosa004_Passive2`; the second selects `Buff_Lacrimosa004_Level4`. Preserve the native key spelling and distinguish the configured passive from individual awakening Effect4. The existing extra-Skill and [Nightmare](nightmare-damage-and-duration.md) findings already own their override, internal healing ceiling and stack-limit/coefficient-clamp distinctions; those previously located values are not new research gaps. Their named native consumers, effective defaults and settlement/clock questions remain unresolved.

### Passive description coverage, reviewed October 18

Saved description source: `guide_lacrimosa.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Tomato Banquet describes a 400% ATK extra packet when Discord triggers on an already Broken target. This is a Discord condition, not a Psyche-only passive inferred from a legacy name. Sleep In describes one extra Morning Tomato attempt after five Tomato Metal/Tomato Percussion uses during the current Skill cooldown, limited to once per cooldown. The known override and Nightmare subsystem do not by themselves establish that counter reset, trigger admission or extra-attempt consumption.

### Stolen ATK snapshot and flat cap recovered, October 18

The expansion's warning-free `Cau_Lacrimosa_StealTargetAtk:CustomCalculateBaseMagnitude` (`#/2`) reads **InstigatorActor player's StolenAbility.TargetAtk** only after a valid TargetAvatarClass check. It does not read current HitActor ATK. Successful coordinate-zero curves provide multiplier **0.3**, lower bound **10** and upper bound **200**, returning **clamp(stored TargetAtk*0.3,10,200)**. Its initialized StolenAtk=0 path still reaches this clamp after the failed player/avatar checks; with successful curves, supplied snapshots 0/100/1000 yield **10/30/200**, not 0/30/300. Failed curve defaults are separate paths and must not be overwritten with these successful inputs.

`Buff_Lacrimosa_AddAttack#/1/Properties/CustomApplicationModifierInfos` binds the result with multiplier 1 to additive **AtkAdd**. This resolves a flat ATK amount and stored basis rather than a 30% final-ATK multiplier. Native StolenAbility capture/expiry, recipient, change-avatar admission and pending removal remain open; the helper does not recover monster-skill theft itself.

### Repaired projectile configuration, October 18

The ProjectileBase schema repair corrects a missing first field in the public map. Use the latest cache exports under , not the old shifted projectile CDO fields. The raw assets are unchanged. Recovered `/2/Properties/ProjectileFuncArray` now explicitly selects ProjectileHitExplodeEffect in **Melee1** (`40d5b5b99887c5f2.json`, subobject `/10`), **Melee2** (`5c60cf77360354ff.json`, `/13`) and **Skill1** (`3383aa70b23c60db.json`, `/32`). Their containers respectively name DamageRange_Lacrimosa_Melee1/Melee2/Skill1 and GE_Player_Lacrimosa_Melee1_Damage/Melee2_Damage/Skill1_Damage, each alongside a separate HitBack effect. This resolves active function membership, not just the presence of unattached subobjects. Configured DefaultDisappearTime is respectively **0.3/0.5/0.6**; native clock, hit/disappear ordering, omitted repeat defaults and effective lifetime remain open. These three exports have no object-read warning; unrelated inherited Actor fields are not certified by that fact.

The repaired **BulletActor_LacrimosaPan** (`57cec42319e76e09.json`, `/3`) explicitly sets bOverlapHitDamage=true, bOverlapDamage=true, OverlapDamageInterval=**0.1**, bOverlapedCheckDamage=true and OverlapDamageMaxCount=**3**. Its active functions include ProjectileDamagedBounce (`/13`, bCanBeParriedByOwner=true), requesting DamageActorLacrimosaPanExplode / PlayerLacrimosaMelee3 with GE_Player_Lacrimosa_B_Melee3_Explode_Damage, and ProjectileOverrideEffectContainer (`/14`), requesting B_Melee3_Damage with DelayOverrideTime=**0.5**. **Pan1** (`f1613cd6eb1cc4eb.json`, `/2`) instead sets interval **0.3**, bOverlapedCheckDamage=true and max count **2**, while omitting those two overlap-enable flags. Do not infer their effective defaults or turn 3/2 into proven per-target packet counts: the native counter scope, deduplication, enablement, parry/bounce dispatch and override clock are missing. The subsequent PrimitiveComponent repair removes Pan/Pan1's earlier component-read warnings without changing these CDO properties. Neither reached component supplies an explicit collision delegate; native collision dispatch remains open.

The previously saved Melee1/Melee2 function graphs contain a **0.2** AsyncTaskSetTimer, but the reached timer callback ends without applying damage; the separate BeginPlay/Destroyed route calls waterInteractionAdd. This is not a recovered 0.2 damage interval. Branch now recovers bHitStealTargetAbility=true. Its earlier BoxComponent read error is resolved by the subsequent PrimitiveComponent repair; no explicit collision delegate is recovered from that component. This does not establish successful theft admission or the native StolenAbility writer.

### Stolen-ability wrapper admission, activation and exit requests, October 18

The initial bundle's warning-free **GA_Lacrimosa_ChangeAvatar** contains an explicit wrapper state graph, separately rendered in . Its **K2_CanActivateAbility** (`#/4`, statements 10..456) returns true only on the valid avatar-player path with **both StolenAbility.TargetAvatarClass and StolenAbility.AbilityCfg.AbilityClass valid** and **no Buff_Lacrimosa_ChangeAvatar**. Failed cast/class checks and an existing carrier return false. This is the Blueprint predicate, not the complete native cost/cooldown/input gate.

Activation (`#/2`, 3768..4256) calls the montage parent, clears HasActivatedEndAbility/HasActivatedStolenAbility/IsEndingAbility, stores the current **StolenAbility.AbilityCfg.AbilityClass**, and registers a **15** WaitDelay watchdog, ability-commit/end tasks and native input subscription **[1,6,3,35]**. **OnReceiveEvent** (`#/15`) calls the parent and requests TriggerChangeAvatar only for **Event.Montage.Player.Skill.1**. That route is local-client-role gated, waits one frame, then attempts the form only if HasActivatedStolenAbility is still false (6722..6736 → 5636..5661). Thus activation alone does not immediately prove a changed avatar or an extra damage packet.

The attempt (4803..5635) revalidates both class inputs and the change-character object, clears StopMontageEndAbility, calls **SetChangeCharacterCDO(TargetAvatarClass,true)**, and only after the object's current class compares equal sends **ServerSetChangeCharacterCDO** and **TryActivateAbilityByClass(stored class)**. It stores that boolean in HasActivatedStolenAbility. The caller ends the wrapper on unsuccessful activation; native class mutation, RPC delivery and the stolen ability's own admission remain separate. On a commit of the stored stolen class, 2599..2609 requests **K2_TriggerAbilityCooldown**, then the valid-player/authority route requests Buff_Lacrimosa004_Lv1TriggerReaction. A different committed montage ability, excluding ChangeAvatarEnd, requests wrapper end. This locates cooldown and reaction-marker request timing relative to the reported commit, without measuring elapsed time or proving mutation success.

On the stored stolen ability's end, **RevertAndTriggerEnd** (`#/6` → 3097..3246 → `#/18`) enters a local-client-role, IsEndingAbility-guarded one-frame route. Its callback (618..1879) restores the player's class locally if different, requires equality before the server request, and attempts **GA_Lacrimosa_ChangeAvatarEnd** unless the death montage is playing; it then ends the wrapper on either return-ability success or failure. **K2_OnEndAbility** (`#/5` → 4257..4586) separately clears the stored class and requests **SafeSetChangeCharacterCDOLocally(player class)**. The 15-delay callback only requests wrapper end; it is not evidence for a 15-second form lifetime. The reached **Buff_Lacrimosa_ChangeAvatar#/2** instead has HasDuration scalar **10**, with an immunity query over State.ControlState/State.AniState; its admission, clock and native immunity scope are still independent.

The input callback (3456..3767) requires **HasActivatedStolenAbility AND NOT HasActivatedEndAbility**. On a successful avatar-player cast it requests wrapper end only when **GetIsControlledCharacter() is true**; failed cast also reaches end. The received InputType/press/jump values are stored but not tested in that branch. This is not proof that every physical press/release reaches the callback, or that all monitored inputs are legal cancels: native input-task filtering, queued packets, server/local reconciliation and parent end cleanup remain open.

### Stance-switch predicates and extra-Skill consumption request, October 18

Initial warning-free **GA_Lacrimosa_SwitchSkill:K2_CanActivateAbility** (`#/11`) returns **NOT Has(Buff_Lacrimosa_GiveMeleeModA)** on its valid ability-character path; **SwitchSkillModB#/11** returns presence of the same carrier. SwitchSkillModB's priority function (`#/12`) returns DefaultPri with the marker, otherwise **-1**. Its follow-level component (`#/17`; SwitchSkill `/16`) explicitly names **GA_Lacrimosa_Melee**, rather than inheriting Skill talent by assumption. These resolve selected predicates and source level; native grant/input arbitration, effective DefaultPri and cleanup remain open.

Both switch CDOs bind **Event.Montage.Player.Melee.5_1** to a direct SwitchModB_Damage container and to a spawned DamageActorLacrimosaSwitchModB whose container also names that damage class (SwitchSkill `/12..13`; ModB `/13..14`). Branch.2/Branch.3 similarly have direct Pan_Damage and Pan1 projectile spawn entries. Do not count both channels as automatically delivered independent hits or deduplicate them solely by class name: native event dispatch, source-container construction and recipient identity must decide that. These callers now connect the repaired projectile configuration to the stance actions.

Ordinary **GA_Lacrimosa_Skill:K2_CommitExecute** (`#/6` → `/4`, 10..156) explicitly requests removal of **Buff_Lacrimosa004_MarkExtraE**, all stacks (-1), on the valid avatar-player path. This locates a commit-time consumption request; it does not recover the five-input counter, once-per-cooldown ledger, alternate wrapper consumption or successful native removal.

### A6 reaction-ready marker and guard chain, October 18

`1004AwakenEffect/1004/AwakenEffectStructList` maps **Effect6** to the internally Level1-named carrier. The wrapper's commit route above requests **Buff_Lacrimosa004_Lv1TriggerReaction**, whose expansion `/3` requires **Con_Lacrimosa_Lv1** and **Con_Lacrimosa_Lv1_NotHavCD**. Their warning-free `#/2` functions check instigator ASC for **Buff_Lacrimosa004_Level1** and absence of **Buff_Lacrimosa004_Level1CD**, respectively. Do not relabel this as public A1 merely from the native suffix.

The ready carrier (`#/2`) is HasDuration through **Lacrimosa_Lv1TriggerReactionDur=5**, granting **State.QTETriggerReaction**. At **QTE_BEGIN** it requests self-removal with Con_1072_IsNotInDomain, and addition of the CD carrier with Con_IsOnlyServer. The latter names **Lacrimosa_Lv1CD=5**, but omits its own duration policy; effective inherited policy and clock are not recovered. These locate readiness, exclusion and consumption requests; they do not establish automatic Support selection, Cycle-cost bypass, the domain requirement's effect, actual cooldown expiry or native event ordering.

### Normal-form predicates, Branch self-cost and A2 caller recovered, October 18

Initial warning-free **GA_Lacrimosa_Melee:K2_CanActivateAbility** (`#/7`) returns absence of **Buff_Lacrimosa_GiveMeleeModA** on its valid ability-character path; **MeleeModA#/22** returns presence, matching the separately located switch predicates. Their reached OnReceiveEvent overrides (`#/10` / `/25`) call the montage parent, then only for **Event.Montage.Player.Branch.1** request **Buff_Lacrimosa_BranchBurningLife** on the valid avatar player. That carrier (`#/2`) selects **CurrentHPDamageExecution**, Source/snapshot **SkillCoefficientMult Override**, through **Lacrimosa_AimHurtHPPercent=0** at coordinate 0. Its self-damage/server/no-fight tags do not create a nonzero HP cost; conversely the zero configured input does not prove every aimed/Branch action is globally free. Native executor interpretation, other costs, callback frequency and admission remain separate.

**GA_Lacrimosa_QTE#/3** calls GA_QTEBase activation then, with valid avatar player and **Buff_Lacrimosa004_Level2**, requests **Buff_Lacrimosa004_Lv2DamUp**. That warning-free carrier (`#/1`) binds additive **DamageUpGeneralBase** through Lacrimosa_lv2_Damup, with duration Lacrimosa_lv2_DamupDur and target-aggregated limit one. This locates the A2 grant at the reached activation callback rather than a hit/death trigger. Actual Support admission, successful grant/refresh and clock remain open. None of these reached Normal/QTE functions increments the Sleep In five-melee counter; its native/event ledger is still required. 
### Normal-form start-section priority recovered, October 18

Warning-free initial **LacrimosaMeleeJumpSelection#/1** and **...SelectionB#/1** share this ordered successful-player predicate: **IsFalling → AirAttack**, else **Buff_EvadeAttackSection → Melee4**, else **Buff_Lacrimosa_SkillJumpMelee5Section → Melee5**, otherwise compare GetDistanceTo(GetAttackTarget()) to a threshold and choose **MeleePre at >= threshold, Melee1 below**. The threshold is **570** for Selection, **400** for SelectionB. Source calls GetAttackTarget/GetDistanceTo without a separate valid-target branch; missing-target behavior remains with those native APIs. Their constructed Ability.Evade.PerfectEvade literal is unused in the returned selection.

**GA_Lacrimosa_MeleeModA#/27/AdditionalStartSections** selects the 570 helper; ordinary **GA_Lacrimosa_Melee#/12** selects the 400 helper. Thus the helper suffix cannot be used to swap the two actual form bindings. These locate entry priority and distance boundaries, not an elapsed two-second combo reset, input arbitration or successful AirAttack admission. Source:  and `spawn-target-caller-refs.txt`.

### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| BulletActor_LacrimosaPan#/13; projectile/spawn entry | PlayerLacrimosaMelee3 | (0.0, 0.05, 0.0, 1.0, 1, true) | GE_Player_Lacrimosa_B_Melee3_Explode_Damage |
| GA_Lacrimosa_MeleeModA#/28; Melee.5_1 | PlayerLacrimosaMelee3 | (0.0, 0.05, 0.0, 1.0, 1, true) | GE_Player_Lacrimosa_Melee3_Damage |
| GA_Lacrimosa_MeleeModA#/29; Melee.3 | playerLacrimosaMelee3 | case-only lookup variant | GE_Player_Lacrimosa_Melee3_Damage |
| GA_Lacrimosa_MeleeModA#/30; Melee.4 | PlayerLacrimosaMelee4 | (0.0, 0.0, 0.05, 2.0, 3, true) | GE_Player_Lacrimosa_Melee4_Damage |
| GA_Lacrimosa_MeleeModA#/30; Melee.4 | PlayerLacrimosaMelee4_1 | (0.5, 0.05, 0.0, 2.0, 1, true) | GE_Player_Lacrimosa_Melee6_Damage |
| GA_Lacrimosa_PerfectEvade#/6; Melee.1 | PlayerLacrimosaPerfectEvade | (0.0, 0.0, 0.0, 1.5, 1, true) | GE_Player_Lacrimosa_PerfectEvadeAttack_Damage |
| GA_Lacrimosa_PerfectEvade#/6; Melee.1 | PlayerLacrimosaPerfectEvade1 | (0.0, 0.15, 0.08, 1.5, 5, true) | GE_Player_Lacrimosa_PerfectEvadeAttack_1_Damage |
| GA_Lacrimosa_SwitchSkill#/13; Melee.5_1 | PlayerLacrimosaMelee3 | (0.0, 0.05, 0.0, 1.0, 1, true) | GE_Player_Lacrimosa_SwitchModB_Damage |
| GA_Lacrimosa_SwitchSkillModB#/14; Melee.5_1 | PlayerLacrimosaMelee3 | (0.0, 0.05, 0.0, 1.0, 1, true) | GE_Player_Lacrimosa_SwitchModB_Damage |
| GA_Lacrimosa_UltraSkillModA#/10; UltraSkill.1 | PlayerLacrimosaUltraSkill | (0.0, 0.125, 0.1, 3.0, 4, true) | GE_Player_Lacrimosa_UltraSkill1_Damage |
| GA_Lacrimosa_UltraSkillModA#/10; UltraSkill.1 | PlayerLacrimosaUltraSkill1 | (0.93, 0.0, 0.0, 2.0, 1, true) | GE_Player_Lacrimosa_UltraSkill2_Damage |

MeleeModA's Melee.4 requests two differently delayed actors: Melee4_Damage and Melee6_Damage. PerfectEvade and Normal-form Burst likewise have separate primary/repeated or delayed packet classes. The same PlayerLacrimosaMelee3 configuration is reused for Melee3_Damage, SwitchModB_Damage and the Pan's B_Melee3_Explode_Damage; the caller container selects damage, not the row name. The lowercase Melee.3 lookup is the shared case-only variant, not a second proven train.

### Projectile end callbacks and long-evade overrides checked, October 18

Initial **BulletActor_LacrimosaMeleeBase:K2_NextTickDestroySelf#/4 → ExecuteUbergraph#/3** stores fDelayTime but only requests **Velocity=ClampVectorSize(Velocity,10,20)**. It supplies no destroy call, delay consumption or packet cancellation in this override. The native caller/final cleanup remains required; the event name is not proof that pending damage ends on the next tick. The original inherited read errors were separately repaired where selected; complete function projections are unchanged.

**BulletActor_LacrimosaMelee1/2** register **AsyncTaskSetTimer(...,0.2,...)**, but their OnTimerExecute bodies immediately return. ReceiveDestroyed requests only water-interaction visuals, not a manual damage application. Melee2 and Melee2_1 separately make the owner's valid capsule ignore this projectile's BoxComponent while moving; their Tick overrides add no attack. These empty/visual timers do not establish 0.2-period damage or a repeat cap. **Branch, Pan1 and Skill1** also expose scene/field-interaction visual callbacks and must not be promoted to extra damage trains independently of their already recovered projectile effect arrays.

**BulletActor_LacrimosaPan1:ReceiveBeginPlay#/6 → ExecuteUbergraph#/5**, 421..754, additionally obtains its owner player and uses that player's existing valid **GetAttackTarget** as ProjectileTrackActor. If absent it requests **owner.SearchAndLockTarget(LockRange_Props_Hammer)**, re-reads GetAttackTarget, stores Current Aggro Target and passes it to SetProjectileTrackActor without another explicit validity check. Tick reads the track actor and, when valid, requests a look-at rotation; it does not repeat the target search. This is an owner-lock-mutating fallback request, not a private nearest-projectile-target selector or a guarantee that the fallback finds an enemy. Native lock policy, recipient deduplication and travel remain open. Its fork_rose branch changes Niagara appearance only. The original three read warnings are separate from the complete function stream; repaired projectile-prefix data supplies the effect-array boundary.

Warning-free **GA_Lacrimosa_Evade** calls the montage parent at activation and handles long-evade section transitions: exact **LongFrontEvadeEndHasMoveInput** requests StartSprint and K2_EndAbility, **LongFrontEvadeEnd** requests StopMovementImmediately before the parent section jump, and **LongFrontEvadeEndAir** requests HideWithDuration(12,0.05) before that parent. EndAbility calls GameplayAbility's parent and resets its loose HideInteractUI state. Its CantTransformFly check requests a HUD message, not a CanActivate return or a damage/resource rule. **GA_Lacrimosa_Appear** only forwards activation to GA_ChangePlayerAppear. These locate source callbacks without supplying real recovery, flight/cancel admission, Sleep In counters or source clocks. Sources: `remaining-projectile-and-field-functions.txt`, `additional-evade-and-pan-functions.txt` and `remaining-action-and-target-functions.txt`.

### Sleep In counter ledger recovered, October 18

The repaired **Buff_Lacrimosa004_Level4#/1 → GameplayCue.Display.Lacrimosa.ExtraESkill → GC_Lacrimosa_ExtraESkill** join supplies the missing counter caller. OnWhile **#/9 → graph#/1:1986** rejects an already valid cached player with a diagnostic, otherwise caches MyTarget player, resets **ExtraECount=0**, evaluates **Lacrimosa_Lv4ExtraERequireMeleeCount=5** at coordinate0 and truncates it into the threshold (CDO also declares 5). Server/standalone setup installs listeners for the Skill-CD tag, EnableExtraE add/remove, post-play montage events and CaluMeleeNums stack changes. CDO **#/10** supplies **MaxExtraECount=1** and section names **Melee1..Melee5, MeleePre, AppearMelee**.

**OnPostPlayAbilityMontage#/7 → 15** requires the animating ability, **AbilityHasAnyTag(Ability.Melee)**, StartSection in that list, a valid cached player and **EnableExtraE present**, then requests **CaluMeleeNums**, level1/stack0. This is a montage-start-section request, with no hit/damage callback or positive-damage test. Native notification delivery and stack conventions must establish actual increments; five landed hits are not the recovered trigger.

**Stack-change callback #/6 → 1355** requires **OldStackCount < threshold AND NewStackCount >= threshold**, **ExtraECount <1**, valid player and EnableExtraE. It removes all EnableExtraE, requests **MarkExtraE**, then increments ExtraECount. Thus supplied4→5 or4→6 cross the threshold;5→6 does not. It does not grant another mark at every subsequent multiple of five. Both **EnableExtraE add** and **Skill-CD-tag removal** reset ExtraECount and remove all MarkExtraE/CaluMeleeNums. EnableExtraE removal and CD addition callbacks only store parameters and return. OnRemove clears the cached player without explicitly ending the installed tasks. Re-entry, synchronous removal callbacks, native stack settlement and task lifetime remain open.

Carrier data now closes the rest of the declared request chain: Level4's **SKILL_REALFINISH** requests EnableExtraE; **E_SKILL_BEGIN** with CheckTriggerObjectName=GA_Lacrimosa_ChangeAvatar removes it; FINISH removes Enable/Mark/Count. Enable and Count require **CoolDown.Player.Skill.Lacrimosa**, Level4 and server/standalone; Count is source cap 20, duration 50 and removes itself at CHANGE_ROLE_OUT. Enable is target cap1, duration 50, with the explicit removal-tag IgnoreTags entry for that cooldown. Mark is target cap1, duration 50 and supplies **AssignAbilityExtraInfoGEComponent: AbilityClass=GA_Lacrimosa_Skill, OverridePriority=100, bIgnoreCoolDown=true**. These values locate permission/count/reset/input-override requests; native cost/commit/readiness, removal-tag semantics and input arbitration remain separate. The 50 magnitudes are not the ordinary Skill cooldown.

### Separate ChangeAvatar cue candidate checked, October 18

Warning-free **GC_Lacrimosa_ChangeAvatar** has a readable body, but no saved carrier GameplayCues join or exact tag reference was found in the selected Ability_004 sources. Its CDO declares Dialogue/LevelTransfer/InteractSmartObject as ForbidChangeStates. The function validates the cached player/state manager and rejects any active listed state; allowed entry subscribes state changes and requests **AsyncTaskChangeAvatar** using the player's StolenAbility TargetAvatarClass/AbilityClass. The entered task's successful end requests a reverse task using the player's current class and GA_Lacrimosa_ChangeAvatarEnd, then an authority/autonomous-hybrid A1-reaction-carrier request followed by next-tick removal of ChangeAvatar. Reverse success/failure and entered failure callbacks have empty reached branches. Entering a forbidden state schedules next-tick carrier removal. OnRemove ends a valid saved entered task and clears its cache; no explicit state-task cleanup is reached. This is an unbound candidate route, not an additional admitted A1 proc or proof of Mode A/B switching. The 0.0167 HideWithDuration input is cosmetic.

The separately joined MeleeTotal damage listener and its Nightmare requests belong to the [Nightmare owner](nightmare-damage-and-duration.md#blood-damage-listener-and-runtime-stack-map-recovered-october-18). Sources for this section: , `lacrimosa-extra-skill-cue-carriers.txt`, `lacrimosa-avatar-cue-tag-refs.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Lacrimosa_Skill_001`) and Burst (`Lacrimosa_UltraSkill`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Form-grant configuration & stance switching:** The Blueprint stance predicates and stolen-wrapper routes are now located above. Native input replacement/arbitration, cost/cooldown admission, successful class mutation, server/local reconciliation and cleanup ordering remain open.
- **Projectile delivery:** Recovered Pan repeat/limit inputs and packet-function membership still require native counter scope, deduplication, callback delivery, overlap defaults, bounce/override ordering and clock selection. The remaining component/inherited-field decode failures are recorded above; do not replace them with inferred hit counts or lifetimes.
- **Sleep In extra-Skill override:** The cue now locates section-start count requests, threshold crossing and reset/override payloads. Resolve native post-play callback and stack delivery, zero-stack conventions, reset/removal re-entry and actual extra-Skill admission/cost.
- **Nightmare DoT & A3/A5 settlement:** Owned by [nightmare-damage-and-duration](nightmare-damage-and-duration.md).
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.

Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The [form-grant configuration](#configured-form-ability-grants) is now portable. Resolve native grant admission/effective level, old-versus-new input arbitration, and how removing the carrier treats an already active alternate ability or queued form switch. The configured removal label does not establish the delivered cleanup order.
- The [extra-Skill override](#configured-extra-skill-override) is also portable. Resolve counting per input/packet, counter grant/reset, once-per-cooldown eligibility, override priority/consumption and real-finish/switch cleanup; the configured cooldown-ignore flag does not prove a complete Sleep In transition or remove other costs/readiness gates.
- Resolve native Blood DoT executor dispatch, clock, recipients beyond the bounded target-zero hypothesis, and ordering; resolve form transition input/return rules, Sleep In extra-use/cooldown lifecycle and Discord cycle triggers. See the linked Nightmare card for the executable boundary and separate local/public stack-lifecycle conflicts.
- Ordinary-hit energy retains tested form-transition and projectile assumptions; native caller, timing, and recipient evidence remains limited.

## Responsibility cards

- [nightmare-damage-and-duration](nightmare-damage-and-duration.md#unknown-execution-policies) contains the findings and questions for that distinct subsystem.

## Prior findings

- [`reviews/q10-game-action-binding/lacrimosa-action-binding-model.json`](../../../../reviews/q10-game-action-binding/lacrimosa-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/lacrimosa_actions.rs), [`lacrimosa-ranged-normal-chain.json`](../../../../reviews/q10-game-action-binding/lacrimosa-ranged-normal-chain.json), [`lacrimosa-skill-burst-packets.json`](../../../../reviews/q10-game-action-binding/lacrimosa-skill-burst-packets.json).

## Research history

- **Current question:** Stance transition arbitration (Mode A to Mode B), Sleep In 5-melee counter tracking, and A3 instantaneous Nightmare detonation.
- **Useful source locations:** `GA_Lacrimosa_Skill`, `GA_Lacrimosa_UltraSkill`, `GA_Lacrimosa_MeleeModA`, `Buff_Lacrimosa004_MarkExtraE`, `1004AwakenEffect.json`, `DT_LacrimosaEffectFigure.json`, `DT_ReactionExtendDataTable#/1004`.
- **Methods already tried:** Saved asset/notify tracing, Sleep In override and Nightmare curves, stored-ATK calculator/consumer, and the October 18 ProjectileBase/PrimitiveComponent field repair. Source-backed projectile membership and repeat inputs are above; the common native-body barrier is unchanged. The attempted Actor parent override worsened decoding and was excluded. Coincident packet tests in `lacrimosa-action-binding-model.json` remain model checks, not gameplay evidence.
- **Next concrete step:** Sleep In now has a section-start counter/threshold/reset graph, and MeleeTotal has exact-class Blood requests with runtime E/Q map replacement; the separate ChangeAvatar cue remains unbound. Resolve native callback/stack settlement, effective source/talent capture, extra-Skill input/cost arbitration and Nightmare delivery. Do not repeat the located counter ledger as a missing graph; continue other roster cue consumers.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
