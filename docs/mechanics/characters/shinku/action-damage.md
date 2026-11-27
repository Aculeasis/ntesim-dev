# Shinku action and damage model

Unchecked means not yet accepted under this checklist, not absence of existing data. For closure format and acceptance rules see the [progress workflow](../../README.md).

Native identity: `character:1076`.

**Responsibility:** Shinku outgoing action packets, modeled action sequencing, and owner-specific damage effects. Frozen cutoff: Global the Global source snapshot of 2026-10-04.

**Model state:** complete selected 8/8 research specification with checked inferences below; current executable partial adapters are narrower. Research readiness, native-fidelity certainty and runtime capability are distinct.

**Open questions:** [owner execution and implementation gaps](#unknown-execution-policies).

## Selected complete packet specification

**Mixed source/checked inference, October 20.** Normative over historical partial runtime models below. Source inputs: `DT_SkillDamageData`, `DT_ShinkuEffectFigure`, `1076AwakenEffect.json`, `DT_CharacterAbilityConfig.json`, permanent owner inputs (optional `shinku.json`), and verification cases in [owning Rust checks](../../../../crates/content/src/simulation/tests/shinku_actions.rs). All selected outgoing damage is **Cosmos** (`CHARACTER_ELEMENT_TYPE_COSMOS`), scales with Shinku's **live ATK**, and evaluates standard crit, DEF, and RES multipliers within each source category. Native 13-coordinate talent arrays are retained; sampled below at **levels 1 / 8 / 9** (coordinate = effective talent minus one). Category A represents Normal/hold/branch/switch, E represents Skill, Q represents Burst, H represents Support QTE, X represents plunge, and Z represents Critical Riposte.

| Suffix after `GE_Player_Shinku_` | Action multiplicity and category | Coefficient 1 / 8 / 9 | Raw energy / Cycle / Break per packet |
| --- | --- | --- | --- |
| `Melee1_1_Damage` | 1 stage 1 slash / A | .182 / .312 / .337 | 0.760 / 1.266 / 0.151 |
| `Melee1_2_Damage` | 1 stage 2 first slash / A | .308 / .528 / .570 | 1.285 / 2.140 / 0.256 |
| `Melee1_2_1_Damage` | 1 stage 2 follow-up slash / A | .205 / .351 / .379 | 0.853 / 1.419 / 0.170 |
| `Melee1_3_Damage` | 1 stage 3 flurry 1 / A | .294 / .504 / .544 | 1.225 / 2.039 / 0.244 |
| `Melee1_3_1_Damage` | 1 stage 3 flurry 2 / A | .174 / .298 / .322 | 0.724 / 1.206 / 0.144 |
| `Melee1_3_2_Damage` | 1 stage 3 flurry 3 / A | .197 / .338 / .365 | 0.820 / 1.365 / 0.163 |
| `Melee1_3_3_Damage` | 1 stage 3 flurry 4 / A | .421 / .722 / .779 | 1.753 / 2.918 / 0.349 |
| `Melee1_4_Damage` | 1 stage 4 dual strike 1 / A | .815 / 1.397 / 1.509 | 3.399 / 5.658 / 0.676 |
| `Melee1_4_1_Damage` | 1 stage 4 dual strike 2 / A | .498 / .853 / .922 | 2.075 / 3.454 / 0.413 |
| `Melee1_5_Damage` | 1 stage 5 heavy finish / A | 1.129 / 1.935 / 2.090 | 4.706 / 7.834 / 0.936 |
| `Melee1_1_Rage_Damage` | 1 Rage stage 1 strike / A | .276 / .473 / .511 | 1.066 / 1.771 / 0.213 |
| `Melee2_Rage_Damage` | 1 Rage stage 2 strike / A | .408 / .699 / .755 | 1.573 / 2.614 / 0.315 |
| `Melee3_Rage_Damage` | 1 Rage stage 3 sweep / A | .254 / .435 / .470 | 0.981 / 1.629 / 0.196 |
| `Melee3_1_Rage_Damage` | 1 Rage stage 3 slice / A | .078 / .134 / .144 | 0.302 / 0.502 / 0.060 |
| `Melee4_Rage_Damage` | 1 Rage stage 4 burst 1 / A | .536 / .919 / .992 | 2.069 / 3.438 / 0.414 |
| `Melee4_1_Rage_Damage` | 1 Rage stage 4 burst 2 / A | .687 / 1.177 / 1.272 | 2.654 / 4.410 / 0.531 |
| `Melee5_Rage_Damage` | 1 Rage stage 5 finisher / A | .465 / .797 / .861 | 1.796 / 2.984 / 0.359 |
| `Watch_Damage` | 1 Menacing Gaze Instant Strike (scaled by stacks) / A | 1.000 / 1.714 / 1.851 | 3.750 / 6.250 / 0.500 |
| `WatchEx_Damage` | 1 A4 16-stack Instant Strike bonus (scaled by 16) / A | .200 / .343 / .370 | 0.000 / 0.000 / 0.000 |
| `ReactionAOE_Damage` | 1 Passive 1 Lone Stride Cosmos AoE / A | 4.000 / 6.855 / 7.404 | 10.000 / 0.000 / 0.000 |
| `Skill1_1_Damage` | 1 ordinary Skill opening / E | .173 / .296 / .320 | 0.750 / 1.255 / 0.144 |
| `Skill1_2_Damage` | 1 ordinary Skill mid slash / E | .231 / .396 / .428 | 1.000 / 1.673 / 0.192 |
| `Skill1_3_Damage` | 1 ordinary Skill finisher / E | .796 / 1.364 / 1.473 | 3.450 / 5.772 / 0.663 |
| `Skill1_Rage_Damage` | 1 Rage Skill strike 1 / E | .224 / .384 / .415 | 0.000 / 0.832 / 0.103 |
| `Skill2_Rage_Damage` | 1 Rage Skill strike 2 / E | 2.176 / 3.729 / 4.028 | 0.000 / 8.068 / 0.997 |
| `UltraSkill1_Damage` | 1 Burst initial slash / Q | .185 / .317 / .342 | 0.000 / 0.000 / 0.046 |
| `UltraSkill2_Damage` | 1 Burst second cut / Q | 1.200 / 2.057 / 2.221 | 0.000 / 0.000 / 0.300 |
| `UltraSkill3_Damage` | 5 Burst rapid slashes / Q | .550 / .943 / 1.018 each | 0.000 / 0.000 / 0.138 each |
| `UltraSkill4_Damage` | 1 Burst heavy cleave / Q | 1.440 / 2.468 / 2.665 | 0.000 / 0.000 / 0.360 |
| `UltraSkill5_Damage` | 1 Burst ground slam / Q | .425 / .728 / .787 | 0.000 / 0.000 / 0.106 |
| `UltraSkillPre_Rage_Damage` | 1 Rage Burst setup / Q | .600 / 1.028 / 1.111 | 0.000 / 3.333 / 0.125 |
| `UltraSkill_Rage_Damage` | 1 Rage Burst flurry / Q | .200 / .343 / .370 | 0.000 / 0.000 / 0.050 |
| `UltraSkill2_Rage_Damage` | 1 Rage Burst detonation / Q | 4.000 / 6.855 / 7.404 | 0.000 / 0.000 / 1.000 |
| `QTE_Damage` | 1 Support QTE strike / H | 2.000 / 3.428 / 3.702 | 5.300 / 0.000 / 2.500 |
| `PerfectEvadeAttack_Damage` | 1 Critical Riposte slash / Z | 1.425 / 2.442 / 2.638 | 2.900 / 4.800 / 2.500 |
| `PerfectEvadeAttack_Rage_Damage` | 1 Rage Critical Riposte / Z | 1.675 / 2.871 / 3.100 | 3.400 / 5.600 / 2.500 |
| `AirAttack1_Damage` | 1 air plunge landing / X | .770 / 1.320 / 1.425 | 1.560 / 2.560 / 0.800 |
| `AirAttack_Rage_Damage` | 1 Rage air plunge / X | .600 / 1.028 / 1.111 | 1.200 / 2.000 / 0.600 |

**Action packet totals:** Normal 5-input chain (10 packets: 1 + 2 + 4 + 2 + 1) delivers **7.238 ATK / 17.600 ChargeAdd / 29.299 Cycle / 3.502 Break** at Level 8. Modeled ordinary Skill (3 packets: Skill1_1 + Skill1_2 + Skill1_3) delivers **2.056 ATK / 5.200 ChargeAdd / 8.700 Cycle / 0.999 Break**. Modeled direct Burst (9 packets: 1x 1, 1x 2, 5x 3, 1x 4, 1x 5) delivers **10.285 ATK / 0 ChargeAdd / 0 Cycle / 1.502 Break**. Scarlet Descent (`Skill1_Rage` + `Skill2_Rage`) delivers **4.113 ATK / 0 ChargeAdd / 8.900 Cycle / 1.100 Break**. Critical Riposte delivers **2.442 ATK / 2.900 ChargeAdd / 4.800 Cycle / 2.500 Break**. Support QTE delivers **3.428 ATK / 5.300 ChargeAdd / 0.000 Cycle / 2.500 Break**.

## Selected complete timing specification

**Mixed native coordinates and real-clock execution:** All action and buff timings progress along one continuous real combat time axis. Source montages advance at 1.0 source units per second, modified by hitstop and pause intervals on positive contact. Overlapping pauses on the same caster take the minimum progress rate rather than serially stacking. Cooldowns and the DPS denominator continue uninterrupted during pauses.

- **Ordinary Skill (`GA_Shinku_Skill`):** Client `CoolDwonCurveTable` cooldown is **3.0s** (`GA_Shinku_Skill: 3.0`, `GA_Shinku_Skill_Rage: 3.0`). Montage asset notifies start hit 1 (`Skill1_1`) at **0.150s**, hit 2 (`Skill1_2`) at **0.350s**, hit 3 (`Skill1_3`) at **0.742s**, and cancel windows from **0.850s–1.234s** (`shinku.skill_animation_profile_v1`). Does **not** enter Surging Crimson. In Surging Crimson, Skill redirects to **Scarlet Descent (`GA_Shinku_Skill_Rage`)** on its own **3.0s** cooldown, spending **12 Fading Reason**, granting +1 **Crimson Judgment** stack (cap 3, up to 5 casts per stance), and at A5 healing `3.0 * BaseATK`.
- **Burst (`GA_Shinku_UltraSkill`):** Client `CoolDwonCurveTable` cooldown is **20.0s** (`GA_Shinku_UltraSkill_Rage: 15.0`). `GA_Shinku_UltraSkill` has **`GACost = 0.0`** ordinary `Ultimate Energy` and requires **60.0 `Defiant Spirit`** (`ShinkuQEnergy`, `ShinkuUltraEnergyMax = 60.0`), earned only on Shinku's own connected hits (`18.0` Normal chain, `5.0` Skill, `5.3` QTE, `3.75` Watch — unaffected by `Charge Efficiency`, `Charge` reaction, or off-field ally hits). Nine direct hits span **3.283s–6.433s** (`shinku.burst_animation_profile_v1`) under Boss / default selection (`UltraSkill7` single-hit variant is unmodeled).
- **Normal Attack chain (`GA_Shinku_Melee`):** Five ordinary stages progress with 10 hits (`Melee1_1..1_5`). During **Surging Crimson**, Normal attacks execute 5-stage **Tidal Assault** (`Melee1_Rage..Melee5_Rage`, 2 + 1 + 6 + 2 + 1 = 12 hits), building **Fading Reason** (`13 * ChargeAdd / 12.998`, capped at `12`). Branch attacks (`Branch1_1..1_4`, `Branch1..5_Rage`) and plunge attacks are not yet routed.
- **QTE (`GA_Shinku_QTE`):** The selected bounded Support model uses a 0.100-second impact and 0.600-second recovery. The native packet grants 5.300 `ChargeAdd`, 0.000 Cycle, and 2.500 `UnbalValue` under their respective delivery rules. A qualified reaction switch admits this Support with no independent cooldown.
- **Critical Riposte (`GA_Shinku_Dodge` / `PerfectEvadeAttack`):** Client packet data gives 2.900 `ChargeAdd` / 4.800 Cycle / 2.500 Break (3.400 / 5.600 / 2.500 in Rage); unmodeled in `ntesim`.

## Selected complete state and effect specification

### Live Surging Crimson binding

Source `.shinku.mods.surging_crimson` reads the native damage modifier's remaining World seconds; `.shinku.surging_crimson.active` reads whether that deadline is strictly after the settled combat time. `.shinku.energy`, `.shinku.resources.fading_reason`, `.shinku.resources.crimson_judgment`, `.shinku.resources.watch`, and `.shinku.menacing_gaze.active` expose Shinku's exact gauges and stacks. The native Burst callback (`GA_Shinku_UltraSkill`) spends **60 Defiant Spirit** (via `.energy`, default `start_energy=60`), enters Surging Crimson for 13s, and clears Surging Crimson immediately on field exit (`CHANGE_ROLE_OUT`) or after **Dragonflame Verdict**'s hits settle. [Rust source checks](../../../../crates/content/src/simulation/tests/source_forms.rs) check continuing Burst cycles, field-exit expiry, one microsecond before expiry, exact expiry, native replay, and unknown-name diagnostics.

**Mixed source facts and checked gameplay rules:** Resources, status effects, passives, and forms are modeled as an integrated state machine:

- **Energy and gauges:** Burst (`GA_Shinku_UltraSkill`, `GACost = 0.0`) consumes **60.0 Defiant Spirit** (`ShinkuQEnergy`, `ShinkuUltraEnergyMax = 60.0`, `ShinkuUltraEnergyMaxL1 = 60.0`), earned only on Shinku's own connected hits (`18.0` scaled across Normal `Melee1_1..1_5`, `5.0` scaled across `Skill1_1..1_3`, `5.3` on `QTE`, `3.75` on `Watch` outside Rage; unaffected by Charge Efficiency, `Charge` reaction, or off-field ally hits). Shinku's ordinary `ChargeAdd` values still distribute `0.6x` `Ultimate Energy` to living off-field allies (`Energy.rs`). Inside Surging Crimson, `Tidal Assault` (`Melee1_Rage..Melee5_Rage`, `13 * ChargeAdd / 12.998`) and `Watch` (`3.75`) fill **Fading Reason** (`ShinkuSkillEnergyMax = 12.0`). Esper Cycle (`HeterochromeAdd`) has capacity `100` and is granted by Normal, Tidal Assault, Skill, Scarlet Descent, Crimson Judgment, and Watch packets (`0` on QTE, Burst, Dragonflame Verdict, ReactionAOE, and WatchEx).
- **Surging Crimson / Rage State (`Shinku_Rage_Duration` = 13.0s):** Activated by committing Ultimate (`GA_Shinku_UltraSkill`); cleared on field exit (`CHANGE_ROLE_OUT`) or after `Dragonflame Verdict`'s hits settle (`GA_Shinku_UltraSkill_Rage.EndAbilityRemoveGEs`). Grants base +30% DMG (`Shinku_Rage_DmgUp = 0.30`) and +30% additional DMG at R3 (`Shinku_Rage_DmgUpEx_L3 = 0.30`); `Watch`, `WatchEx`, and `ReactionAOE` receive `+0%` base Rage DMG and `+30%` at R3 (`Calc_Shinku_RageDmgUp`).
- **A1:** Ignores 12% of the target's DEF (`Shinku_DefIgnore_L1` = 0.12). Modeled.
- **A2:** Reduces the target's Cosmos RES by 10% (`Shinku_DmgResistDownCosmo` = -0.10) for 20.0s (`Shinku_DmgResistDownCosmo_Dur`) after Ultimate hits. Modeled.
- **A3:** Retains Menacing Gaze stacks and continues ticking off-field across actor switches (`Con_Shinku_NoLevel3` bypassed). Modeled.
- **A4:** Increases Menacing Gaze max stacks to 16 (`Shinku_WatchExStackL4 = 16.0`) and triggers `GE_Player_Shinku_WatchEx_Damage` (`0.2 * Watch`, `0` Charge/Cycle/Break) when Instant Strike fires at 16 stacks. Modeled.
- **A5:** Increases Shinku's DEF by +50% (`Shinku_RageDefUpL5 = 0.5`) during Surging Crimson and heals `3.0 * BaseATK` (`Shinku_DmgRecoverHpL5`) after Scarlet Descent. Modeled.
- **A6:** Increases skill damage ratio by +30% (`Shinku_RageSkillDmgCoefL6` = 0.30) for Scarlet Descent, Crimson Judgment, and Dragonflame Verdict packets only (with vacuum pull). Modeled.
- **Resonance 3 / 6:** Resonance 3 grants +1 talent level to Melee/Skill/Burst and +30% DMG bonus in Surging Crimson; Resonance 6 grants +15% DEF ignore (`Shinku_DefIgnore_L6Extra` = 0.15, total 27%) and +10% Crit Rate (`Shinku_CritUp_L6Ex` = 0.10). Modeled.
- **Passives 1 / 2 / 3 (`Lone Stride` BT2 / `Wrathscale` BT4 / `Menacing Gaze`):**
  - **Passive 1 (`Lone Stride`, BT >= 2):** `BackChargeMul = 1.0` grants living off-field teammates equal Charge reaction energy (`10 * their_CE`). When Shinku is the active Charge recipient (`1.0s` ICD), emits `GE_Player_Shinku_ReactionAOE_Damage` across all living targets (Basic talent + R3; `10.0` `ChargeAdd` to teammates) and applies `shinku.lone_stride_atk` (`+5% ATK` per stack, `30.0s`, max 10 stacks). Modeled.
  - **Passive 2 (`Wrathscale`, BT >= 4):** Grants `+50%` DMG (`Shinku_DmgUp_Pas2 = 0.50`) on `UltraSkill` packets against non-boss targets (`!is_boss`). Modeled.
  - **Passive 3 (`Menacing Gaze`):** After `2.0s` without active Shinku attacks (`UnAttackDuration = 2.0`), enters Menacing Gaze and gains `+1` stack/s (cap 8, 16 at A4) and `+1` on Critical Dodge; cleared off-field unless A3. Next connected active hit consumes all stacks and emits `Watch_Damage` (`rate * stacks`, granting `3.75` Defiant Spirit / Fading Reason, `6.25` Cycle, `0.5` Break, `3.75` `ChargeAdd` fanout). Modeled.

## Selected complete recipient specification

**All-in-range dummy contract and single-target rules:**
- **Burst AoE (`GA_Shinku_UltraSkill`):** Under the all-in-range dummy contract, all declared scenario dummies receive the 9 Burst hits in ascending target_index order at each hit's scheduled timestamp.
- **Normal attacks and Skill slashes:** Single-target cleaves focused on the primary target (dummy index 0). Defeated dummies do not absorb hits.
- **A2 Resistance Shred:** Applied to the target struck by Burst; persists for 20.0s.

## Model implementation boundaries (current adapter vs client)

Under the `ntesim` adapter (`character:1076`), Shinku executes Normal chain (`Melee1_1..1_5`), Tidal Assault (`Melee1_Rage..Melee5_Rage`), Ordinary Skill (`Skill1_1..1_3`, 3.0s CD), Scarlet Descent (`Skill1_Rage`/`Skill2_Rage`, 3.0s CD, 12 Fading Reason), QTE (`QTE_Damage`), Crimson Fury Burst (`60` `Defiant Spirit` / `ShinkuQEnergy`, `0` ordinary `Ultimate Energy`), Crimson Judgment (`UltraSkillPre_Rage`), Dragonflame Verdict (`UltraSkill_Rage`/`UltraSkill2_Rage`, clearing `Surging Crimson` after hits settle), Passive 1 (`Lone Stride`: `BackChargeMul = 1.0`, `1.0s` ICD `ReactionAOE_Damage`, `30s` `+5% ATK` max 10 stacks), Passive 2 (`Wrathscale`: `+50%` `UltraSkill` DMG vs non-boss), Passive 3 (`Menacing Gaze`: `Watch_Damage`, `+1` stack/s after `2.0s`, Critical Dodge `+1`, off-field clear unless A3), and Awakenings A1–A6 / R3 / R6 (`WatchEx_Damage` at A4 16 stacks, `+50% DEF` + `3.0 * BaseATK` Scarlet Descent heal at A5). Remaining unmodeled client actions: `Branch` attacks, `AirAttack`, `PerfectEvadeAttack`, and Boss single-hit `UltraSkill7`.

## Source facts and executable model
Frozen table rows and published descriptions establish native IDs, coefficient strings, categories, and stated awakening effects. The rules below separate those inputs from the current executable model; reset and delivery timing marked as a hypothesis are tested implementation policies, not native gameplay evidence. Verification receipts establish only their named scenarios.

## Ordinary Normal chain (current model)
Five admitted ordinary Normal inputs advance stages Melee1 through Melee5 with multiplicities 1,2,4,2,1 (ten hits). The chain loops, retaining native effective talent projection including resonance_3 and defense ignore passives (A1 12%, A6 27%). Rage mode variants (Melee 6-10) are not inferred from ordinary inputs.

Execution policy: A different admitted ability or field exit resets the chain; a same-actor switch does not. A gap greater than 2 seconds between input starts resets the chain; the exact boundary continues. This is a tested continuity convention pending animation/cancel evidence. Each stage delivers ordered coincident packets at the research primary impact timestamp; travel, AoE and animation intervals remain unresolved. Native animation/cancel evidence is not established by this model. Evidence: [`shinku-normal-chain.json`](../../../../reviews/q10-game-action-binding/shinku-normal-chain.json).

## Accepted bounded product models (October 17, 2026)

Following client asset extraction and bounded modeling:
- **Action timing profiles (`shinku.skill_animation_profile_v1`, `shinku.burst_animation_profile_v1`):** Action timing uses asset notify coordinates. For Skill (`Shinku_Skill`), `AnimNotify_TriggerCooldown` is at 0.133s, Hit 1 (Skill1_1) is at 0.150s (`Event.Montage.Player.Skill.1`), Hit 2 (Skill1_2) at 0.350s (`Event.Montage.Player.Skill.2`), and Hit 3 (Skill1_3) at 0.742s (`Event.Montage.Player.Skill.3`); cancel windows activate at 0.850s (`BreakSkill_Jump`), 0.950s (`BreakSkill_AnySkill`), and 1.234s (`BreakSkill_Move`). For Burst (`Shinku_UltraSkill`), direct hits: Hit 1 (UltraSkill1) at 3.283s (`Event.Montage.Player.UltraSkill.1`), Hit 2 (UltraSkill2) at 3.933s (`Event.Montage.Player.UltraSkill.2`), Hits 3 (UltraSkill3 x5) at 4.433s, 4.683s, 4.900s, 5.184s, 5.417s (`Event.Montage.Player.UltraSkill.3`), Hit 4 (UltraSkill4) at 6.199s (`Event.Montage.Player.UltraSkill.4`), and Hit 5 (UltraSkill5) at 6.433s (`Event.Montage.Player.UltraSkill.5`); cancel windows for Move, AnySkill, and Jump activate simultaneously at 7.254s (`BP_CanBreakSkill_C`). Follow-up Rage actions: Crimson Judgment (`Shinku_UltraSkillPre_Rage`) hits at 0.167s and 0.267s, cancels at 0.517s (AnySkill) and 0.701s (Move/Jump); Dragonflame Verdict (`Shinku_UltraSkill_Rage`) hits at 3.450s and 4.450s, cancels at 5.300s (Move/AnySkill/Jump).
- **Multi-target AoE (`shinku.aoe_all_in_range_dummy_v1`):** Shinku High-Speed Breach and Crimson Fury deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending `target_index` order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.

## Action packets and owner effects


- GA_Shinku_Melee public primary packet is GE_Player_Shinku_Melee1_1_Damage. Native coefficients at Normal levels 1/8/9 are 0.182/0.312/0.337; category A/COSMOS metadata stays source-bound.  Does not activate Surging Crimson; DEF ignore applies at awakening >= 1 (12%) and awakening >= 6 (27%).
- GA_Shinku_Skill public primary packet is GE_Player_Shinku_Skill1_1_Damage. Native coefficients at Skill levels 1/8/9 are 0.173/0.296/0.320; category E/COSMOS metadata stays source-bound. Does not activate Surging Crimson; Rage Skill packets (`Skill1_Rage`, `Skill2_Rage`) are separate.
- GA_Shinku_UltraSkill public primary packet is GE_Player_Shinku_UltraSkill1_Damage. Native coefficients at Burst levels 1/8/9 are 0.185/0.317/0.342; category Q/COSMOS metadata stays source-bound. on_cast activates Surging Crimson (13s duration) granting +30% DMG bonus (+60% at three-selection resonance_3), and +30% ratio at A6 applies to Scarlet Descent / Crimson Judgment / Dragonflame Verdict packets only.
- Ordinary GA_Shinku_Skill delivers Skill1_1, Skill1_2, Skill1_3 once each in listed order; native coefficients at Skill levels 1/8/9 are 0.173/0.231/0.796, 0.296/0.396/1.364, and 0.320/0.428/1.473. Child packets use the cast's effective talent coordinate and primary impact time as a tested timing hypothesis; animation intervals and travel are unresolved. Rage Skill packets remain separate.
- Initial Crimson Fury GA_Shinku_UltraSkill delivers UltraSkill1 once, UltraSkill2 once, UltraSkill3 five times, UltraSkill4 once, and UltraSkill5 once in listed coefficient order, totaling 6.000/10.285/11.105 ratio at levels 1/8/9. Native GA_Shinku_UltraSkill.GALevelUpArray[0].LevelUpData.DamageGEList lists those five IDs in order, and its coefficient string gives the third term *5. Children use the cast's effective talent coordinate and primary impact time as a tested timing hypothesis. In the same frozen ability row, GA_Shinku_UltraSkill2_des maps to GE_Player_Shinku_UltraSkillPre_Rage_Damage and requires repeated Ultimate presses for Crimson Judgment dashes, with Scarlet Descent granting up to three uses; GA_Shinku_UltraSkill3_des maps to GE_Player_Shinku_UltraSkill_Rage_Damage and explicitly requires another press after Judgment uses are consumed or Surging Crimson ends. These later phases are explicit inputs, not automatic children of the initial cast; their resource, timing, and AoE rules remain unmodeled.
- The separate Crimson Judgment and Dragonflame Verdict descriptions map to `GE_Player_Shinku_UltraSkillPre_Rage_Damage` and `GE_Player_Shinku_UltraSkill_Rage_Damage`, with UltraSkill coefficients 0.600/1.028/1.111 and 0.200/0.343/0.370 at levels 1/8/9. Both damage rows retain native `GAName=GA_Shinku_UltraSkill`; the configured character action list exposes only this base Ultimate input, while `GA_Shinku_UltraSkill2_des` and `GA_Shinku_UltraSkill3_des` are description keys. An explicit base-Ultimate command can select either effect as one direct packet under `shinku.direct_ultimate_followups_v1`. It does not emit the initial Burst train or activate/refresh Surging Crimson; an existing Surging Crimson state and its live modifiers still apply. Phase prerequisites, use limits, costs, cooldowns, timing, travel, target selection, AoE, callbacks, and other Rage packets remain unresolved.
- Resonance 3 activates at awakening >= 3 and adds +1 level to Melee, Skill, and UltraSkill, selecting native coordinate level + 1 without manual overrides.
- Existing A1/A6 DEF ignore (12%/27%), A2 Cosmos RES down on UltraSkill hit (10%), A6 Crit Rate (+10%) and rage skill ratio bonus (+30%) remain preserved as partial kit models.
- Canonical arbitrary awakening selections: Effect1 supplies 12% DEF ignore and Effect6 supplies the modeled ratio addition independently. Resonance_6 adds 15% DEF ignore and 10% crit at six selections; resonance_3 Surging damage and skill-level bonuses use count. Effect2 target resistance uses identity. Existing unsupported owner mechanics remain pending.

## Configured Rage ability grants

**Serialized source facts, not natural Rage admission.** The compact effect table (effect_parameters.jsonl) retains three native `AbilitiesGameplayEffectComponent` declarations. Every listed grant explicitly serializes `LevelScalableFloat.Value=1`:

| Carrier / component pointer | Ordered ability and InputID entries | Removal-policy label |
| --- | --- | --- |
| `Buff_Shinku_Rage` / `/3/Properties` | `GA_Shinku_Melee_Rage`: 0; `GA_Shinku_Evade_Rage`: 1; `GA_Shinku_Skill_Rage`: 2; `GA_Shinku_PerfectEvade_Rage`: **-1** | `CancelAbilityImmediately` for all four |
| `Buff_Shinku_UltraRage` / `/4/Properties` | `GA_Shinku_UltraSkill_Rage`: 3 | `RemoveAbilityOnEnd` |
| `Buff_Shinku_UltraRagePre` / `/3/Properties` | `GA_Shinku_UltraSkill_Rage_Pre`: 3 | `RemoveAbilityOnEnd` |

InputID -1 is a literal configured value, not a certified public input or a missing field. The regular Rage grants and the two Ultimate grants have different removal labels. Source availability does not settle input arbitration, effective levels, Rage spending, prerequisites, use limits, cancellation/removal delivery or packet timing. These remain in [unknown](#unknown-execution-policies), independently of the existing direct follow-up packet hypothesis.

## Ordinary-hit Ultimate Energy

The current `shinku.ordinary_hit_energy_v1` adapter uses frozen per-packet ChargeAdd values; its independent packet totals are Melee 17.6, Skill 5.2 units for the listed modeled packet trains. The modeled distribution is 1/1 to the source and 3/5 to each living ally, with recipient Charge Efficiency, carry, capacity, and overflow applied separately. Caller eligibility, positive-HP-loss gating, delivery-time recipients, and off-field continuation are tested hypotheses, not source-table facts. The two explicit follow-up packet rows have frozen ChargeAdd 0 and remain outside this ordinary-hit adapter.

Receipt: [`shinku-ordinary-hits.json`](../../../../reviews/q10-energy-ledger/shinku-ordinary-hits.json).

The [continuing-energy receipt](../../../../reviews/q10-energy-ledger/shinku-continuing-energy.json) checks an A0 zero-start sequence of six complete Normal chains, two Skills, the initial 100-energy Burst train, and one post-spend Normal chain. It applies the modeled 13-second Surging Crimson bonus to every later packet. This does not include Rage follow-up inputs or Charge reaction behavior.

## Provenance

- Model and checks: [`reviews/q10-game-action-binding/shinku-action-binding-model.json`](../../../../reviews/q10-game-action-binding/shinku-action-binding-model.json).
- Owner supplements: [owning Rust checks](../../../../crates/content/src/simulation/tests/shinku_actions.rs), [`shinku-normal-chain.json`](../../../../reviews/q10-game-action-binding/shinku-normal-chain.json).
- Frozen inputs: `data/extracted/guide_shinku.json`, `data/extracted/neverness_shinku.json`, [consumed typed damage rows](../../../../crates/data/src/native_damage/shinku.rs) (original `DT_SkillDamageData` export is cache-only), `1076AwakenEffect` (cache-only awakening source), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)), `DT_GameplayAbilityTipsData` (cache-only source; [consumed action titles](../../../../crates/data/src/ability_titles.rs)).
- The owner verifier checks only the scenarios named in the model; a passing check does not promote unresolved rules to facts.

## Implementation boundary

Consumed EffectFigure parameters are typed in [`shinku/parameters.rs`](../../../../crates/content/src/characters/shinku/parameters.rs); unconsumed source rows are cache-only. The existing owner model and its remaining implementation limits are preserved.

- The explicit Crimson Judgment/Dragonflame Verdict one-packet inputs are executable under the hypothesis above. Their native admission/prerequisites and Rage resource transitions, other Rage variants, Branch1–Branch5, and Evade attacks remain unmodeled.

## Full-kit source investigation (October 17–18, 2026)

Source notation below names an asset stem and JSON export ordinal (`#/N/Properties` or a named Function's `ScriptBytecode`). Installed source reports PC_140 / logic 566459. Description references: [Reference guide](https://neverness.app) and [Neverness](https://neverness.app/characters/shinku). Specific current-page comparisons are identified in the findings below.

### Alternate actions and packet inputs

The existing `DT_SkillDamageData` rows establish native coefficients at talent levels 1 / 8 / 9, categories, and resource channels for Shinku (`GA_Shinku_*`):

| Action / damage row (`GE_Player_Shinku_` prefix) | Native category | Coefficient at levels 1 / 8 / 9 | ChargeAdd / HeterochromeAdd / UnbalValue |
| --- | --- | --- | --- |
| Normal chain `Melee1_1_Damage` .. `Melee1_5b_Damage` | A | 0.174–1.129 / 0.298–1.935 / 0.322–2.090 | 0.724–4.706 / 1.206–7.834 / 0.144–0.936 per hit |
| Rage Normal `Melee1_Rage` .. `Melee5_Rage_Damage` | A | 0.078–0.687 / 0.134–1.177 / 0.144–1.272 | 0.302–2.654 / 0.502–4.410 / 0.060–0.531 per hit |
| Menacing Gaze `Watch_Damage` | A | 1.000 / 1.714 / 1.851 | 3.750 / 6.250 / 0.500 |
| Menacing Gaze extra `WatchEx_Damage` | A | 0.200 / 0.343 / 0.370 | 0.000 / 0.000 / 0.000 |
| Critical Riposte `PerfectEvadeAttack_Damage` | Z | 1.425 / 2.442 / 2.638 | 2.900 / 4.800 / 2.500 |
| Critical Riposte Rage `PerfectEvadeAttack_Rage_Damage` | Z | 1.675 / 2.871 / 3.100 | 3.400 / 5.600 / 2.500 |
| Support `QTE_Damage` | H | 2.000 / 3.428 / 3.702 | 5.300 / 0.000 / 2.500 |
| Skill Normal `Skill1_1_Damage` .. `Skill1_3_Damage` | E | 0.173–0.796 / 0.296–1.364 / 0.320–1.473 | 0.750–3.450 / 1.255–5.772 / 0.144–0.663 per hit |
| Skill Rage `Skill1_Rage_Damage` | E | 0.224 / 0.384 / 0.415 | 0.000 / 0.832 / 0.103 |
| Skill Rage `Skill2_Rage_Damage` | E | 2.176 / 3.729 / 4.028 | 0.000 / 8.068 / 0.997 |
| Burst Normal `UltraSkill1` .. `UltraSkill7_Damage` | Q | 0.185–3.065 / 0.317–5.253 / 0.342–5.673 | 0.000 / 0.000 / 0.046–0.766 per hit |
| Crimson Judgment `UltraSkillPre_Rage_Damage` | Q | 0.600 / 1.028 / 1.111 | 0.000 / 3.333 / 0.125 |
| Dragonflame Verdict `UltraSkill_Rage_Damage` | Q | 0.200 / 0.343 / 0.370 | 0.000 / 0.000 / 0.050 |
| Burst Rage Finisher `UltraSkill2_Rage_Damage` | Q | 4.000 / 6.855 / 7.404 | 0.000 / 0.000 / 1.000 |
| Reaction AoE `ReactionAOE_Damage` | A | 4.000 / 6.855 / 7.404 | 10.000 / 0.000 / 0.000 |

Critical Riposte uses GAName `GA_Shinku_Skill` (Normal) or `GA_Shinku_Melee` (Rage), Support QTE uses `GA_Shinku_QTE`. All damage rows deal COSMOS damage. Ordinary Burst `GA_Shinku_UltraSkill` enters Surging Crimson (13s duration) granting +30% DMG bonus (+60% at three-selection resonance_3); ordinary `GA_Shinku_Skill` does not enter Surging Crimson. A6 adds +30% ratio to Scarlet Descent, Crimson Judgment, and Dragonflame Verdict packets during Surging Crimson.

### Passives and reaction enhancements

- **Passive 1 (Breakthrough 2):** `GA_Shinku_Passive_1` binds `Buff_Shinku076_Passive1`. `DT_ReactionExtendDataTable` row 1076 binds `ReactionAbilityID=GA_Shinku_Passive_1` with `Buff_Shinku_ReactionAOE` (Cosmos reaction 7 AoE damage).
- **Passive 2 (Breakthrough 4):** `GA_Shinku_Passive_2` binds `Buff_Shinku076_Passive2`.

### Awakenings

Source descriptions and selected IDs: `1076AwakenEffect#/rows/1076/AwakenEffectStructList`. These summaries describe configured effects; delivered trigger, recipient and clock rules require the consumers identified below.

- **A1:** Shinku's damage ignores 12% of the target's DEF (`TargetDefIgnore = 0.12`).
- **A2:** Reduces the target's Cosmos Resistance by 10% for 20s after the Ultimate hits.
- **A3:** Keeps Menacing Gaze active after switching characters and allows Shinku to gain Menacing Gaze stacks while off-field.
- **A4:** Increases the Menacing Gaze stack cap to 16. Enhances the triggered Instant Strike when Menacing Gaze reaches 16 stacks with forced knockback, stun, and +20% additional damage.
- **A5:** Increases Shinku's DEF by +50% and Interruption RES by +50% during Surging Crimson, and recovers HP equal to 300% Base ATK after casting Scarlet Descent.
- **A6:** Increases damage ratios for Scarlet Descent, Crimson Judgment, and Dragonflame Verdict by +30%, with vacuum pull.
- **Resonance 3:** Increases the skill level of Basic Attack, Redirect Skill, and Ultimate by +1; increases DMG dealt by an additional +30% while in Surging Crimson.
- **Resonance 6:** Increases Shinku's Crit Rate by +10% and causes her damage to ignore 15% of the target's DEF.

### Menacing Gaze requirement bodies recovered, October 18

The initial bundle's `Con_Shinku_AddWatch:CanApplyGameplayEffectContent` (`#/2`) reads the **instigator ASC**. Presence of `Buff_Shinku076_Level4` explicitly returns true; otherwise successful lookup of `Shinku_Watch_Stack_L0=8` accepts only **current `Buff_Shinku_Watch` count < 8**. This locates the base add predicate; it does not prove the final A4 storage cap or the number of grants per tick.

`Con_Shinku_WatchEx#/2` accepts **IsUsingEvade OR `Buff_Shinku_ParryMark` present** on its first valid-character branch. Otherwise it requires both Level4 marker and **Watch count >= `Shinku_WatchExStackL4=16`**. These are alternatives, not a requirement to accumulate 16 stacks for every enhanced Instant Strike. The expansion's `Calc_Shinku_WatchStack#/2` returns `max(instigator Watch count,0)` and does not consume it. Actual event frequency, off-field delivery, subsequent consumption/reset and Rage input cleanup remain open; a requirement query must not mutate the count merely to predict eligibility. All cited functions are warning-free.

Individual Effect3 describes off-field Watch persistence. Three-selection resonance_3 describes an additional **30%** during Surging Crimson; the source below explains how eligible ordinary Rage packets reach **60% total**, rather than treating that as a single 60% resonance grant. This research does not change runtime awakening selection.

### Rage bonus class split recovered, October 18

`Buff_Shinku_Rage#/2/Properties/CustomApplicationModifierInfos` binds additive DamageUpGeneralBase to `Calc_Shinku_RageDmgUp`, multiplier 1, gated by `Con_Shinku_IsRageDamage`. The warning-free calculator (`#/2`) explicitly treats class-display strings **GE_Player_Shinku_WatchEx_Damage_C**, **GE_Player_Shinku_Watch_Damage_C** and **GE_Player_Shinku_ReactionAOE_Damage_C** separately: they receive only `Shinku_Rage_DmgUpEx_L3=0.3` when the instigator has `Buff_Shinku076_Level3_extra`, otherwise initialized zero. Other packets receive `Shinku_Rage_DmgUp=0.3`, plus the same 0.3 resonance increment when present. Both figure rows are sampled at coordinate 0. Thus ordinary admitted Rage packets give **0.3/0.6**, but those three special classes give **0/0.3**, without/with resonance. A blanket 60% bonus on every Rage-associated packet would be wrong.

`Con_Shinku_IsRageDamage#/2` accepts the same three class strings directly; other effects must satisfy **AssetTagHasAllMatchingGameplayTags([Ability.Player.ShinkuRage])**. This locates packet admission and zone, but the native current-damage getter/capture, effect callback context and Rage state lifecycle remain open.

### Rage logic-object configuration recovered, October 18

The expansion's `player_076_shinku#/28/Properties`, `ShinkuLogicObj`, sets **UnAttackDuration=2.0**, `ObservationStatusGE=Buff_Shinku_Watch`, and trigger event **Ability.Player.ShinkuWatchDamage**. It binds separate `UltraSkillCostValue`, `AwakeningUltraSkillCostValue` and `SkillCostValue` to **ShinkuUltraEnergyMax**, **ShinkuUltraEnergyMaxL1** and **ShinkuSkillEnergyMax**, each with explicit multiplier 1. Native admission must choose the applicable branch; no cost is inferred from the similarly named ordinary Ultimate Energy capacity.

Configured mesh-state events are `Ability.Player.Shinku.Rage` and `...RageMove`. `EnterStateBreakChangeMeshStates` lists Swimming, Dialogue, Sequence, LevelTransfer and DrivingWaterVehicle. Those declarations are not proof that every switch or two-second attack gap exits Rage. The player export has three warnings; its logic-object fields and warning-free Watch helpers above narrow the source boundary, while the native ShinkuLogicObj still controls resource mutation, callback clocks and input arbitration.

### Rage healing basis recovered, October 18

`GE_Player_Shinku_Skill1_Rage_Recover#/1` selects `CustomReplyHealthExecution`, supplying a Source/snapshot SkillCoefficient custom magnitude from `Calc_Shinku_Recover`. The latter (`#/2`, warning-free) returns `instigator GetAtkBase * DT_ShinkuEffectFigure#/rows/Shinku_DmgRecoverHpL5`, evaluated at coordinate 0, whose value is **3**. With base ATK 1000 and final ATK 1500 the supplied heal magnitude is **3000**, before native healing modifiers/clamp; it is not 4500 and does not read recipient MaxHP. Native execution recipient, A5 event admission, actual application frequency and rounding remain open.

### Located target-state selectors and Rage movement gate

`JumpSection_Con_Shinku_Melee5_Check:CanJumpSection` (`#/1`) explicitly returns true for invalid targets/failed ability-character casts or **IsBalancedingState()**; on its remaining valid path it returns **NOT SkillEffectComponent.IsInFloatingEffect(0)**. `JumpSection_Con_Shinku_UltraSkill_Check#/1` instead explicitly rejects invalid targets/casts and returns **NOT IsBossCharacter() AND LimitFloating_Floating==0**. These warning-free selectors make different failure and airborne decisions; their native callers/section precedence remain required before deriving a universal finisher policy.

The expansion's `Con_Shinku_CanBreakMoveRage#/2` returns the valid instigator player's **IsUsingAnyMontageAbility()** directly. `Buff_Shinku_RageMove#/2/Properties/BuffEventEffectData/0` uses it specifically to gate **removal of RageMove on SKILL_BEGIN**. Separate CHANGE_ROLE_OUT and INTERACTENTRY entries request removal without that requirement, and DeadRemove=true. This is not a gate on granting RageMove or a two-second timeout, and must not be transferred to the distinct Rage carrier. Actual callback delivery and native Rage logic remain open.

### Passive description coverage, reviewed October 18

Saved description source: `guide_shinku.json`, `skills[4..5]` (the saved extractor labels these passive records as Support Skill). These are description-level constraints, not recovered native callbacks.

Lone Stride describes off-field characters sharing the active characters Ultimate Energy gain during Charge. While Shinku is active in Charge, it also describes one area packet starting at 400% ATK, scaled by Normal talent, and self ATK +5% for 30s up to ten stacks, with 1s cooldown. Wrathscale describes Ultimate damage +50% against non-Bosses. Native recipient efficiency/carry, Charge event ownership, recursion, ICD capture and Boss classification remain open; these cannot be replaced by ordinary per-hit energy assumptions.

### Wrathscale damage zone and predicate recovered, October 18

`Buff_Shinku_DmgUp_Pas2#/1/Properties/CustomApplicationModifierInfos` supplies additive **DamageUpGeneralBase**, multiplier 1, through `Shinku_DmgUp_Pas2=0.5`. `/2` requires `Con_Shinku_Passive2`, whose warning-free function checks the instigator's `Buff_Shinku076_Passive2`. The modifier's own `Con_Shinku_DmgUp_Pas2#/2` requires a valid effect cast, **AssetTagHasAllMatchingGameplayTags([State.Damage.UltraSkill])**, and **NOT IsBossCharacter(TargetActor)**. This is a located per-packet/non-Boss 50-point bonus in the ordinary damage zone. Existing 20% general plus this 50% gives a 1.70 bonus factor, not 1.80 from multiplying 1.2 by 1.5. Native Boss classification/capture remains open; the tag condition does not grant every Rage or Watch packet an Ultimate bonus.

### Rage duration mode split and A6 coefficient consumers recovered, October 18

Initial `Buff_Shinku_Rage#/2/Properties/DurationMagnitude` selects `Calc_Shinku_RageDuration`. Its warning-free `#/2` function evaluates `Shinku_Rage_Duration=13` at coordinate 0 when **IsHybridMode(OwnerActor)** is true; otherwise it explicitly returns **22.2**. The current 13-second model corresponds to one source branch, without recovering mode classification or the live pause/expiry clock.

`Calc_Shinku_RageSkillDmgCoef#/2` tests instigator `Buff_Shinku076_Level6`, returns **Shinku_RageSkillDmgCoefL6=0.3** on the successful selected path and explicit zero without it. It is bound with multiplier 1 to **CoefModify** by `GE_Player_Shinku_Skill1_Rage_Damage#/2`, `Skill2_Rage_Damage#/3`, and `UltraSkill2_Rage_Damage`, `UltraSkillPre_Rage_Damage`, `UltraSkill_Rage_Damage`, each `#/2/Properties/CustomApplicationModifierInfos`. This locates A6 on five exact packet classes, including the pre-hit; it is not a universal extra 30% DamageUpGeneralBase or a change to the separately recovered healing basis. Native CoefModify composition and actual packet dispatch remain open.

### RageMove input-release and cleanup source recovered, October 18

Initial warning-free **GA_Shinku_RageMove#/2** enters `/1` at 1253, calls GA_TriggerGEBase activation, then requests **Buff_Shinku_RageMove** on its avatar only on the authority path. A separate **WaitIntervalTick(self,0.1,0,false)** task is registered directly on the non-authority path, or after the grant path when **IsOnlyStandalone** is true. Thus the reached polling route is not exclusively standalone, and the 0.1 input is not evidence for a universal real-time cadence.

The tick callback (`#/5` → `/1`, 549..1252) reads local player-controller **CacheAbilityInputHistory[1]**, validates its nested AbilityInputHistory index **TailIdx**, and requests end when the selected record's **bPressOrRelease is false**. Invalid controller/history indices return without that end request; no elapsed-duration threshold is tested. The finished callback merely stores its elapsed value. **K2_OnEndAbility** (`#/3` → `/1`, 1344..1505) calls the parent and requests removal of all RageMove stacks after a valid avatar-player cast, without a cancellation branch. This locates a release-history gate and cleanup request, separate from the existing SKILL_BEGIN/role-out carrier events. Native history-slot meaning, event delivery, tick clock and movement/Rage state arbitration remain open; it does not establish a two-second timeout or an end of Rage itself. 
The ordinary **GA_Shinku_Evade** supplies a separate entry route (warning-free initial export, `#/3` → `/2`, 746..761): after its montage parent it registers **WaitInputRelease(self,0.25,false)**. The [release-task parameter metadata](../../shared/action-timing.md#release-task-and-latent-delay-declarations) identifies these inputs as **AutoTriggerTime=0.25, bTestAlreadyReleased=false**. The OnRelease callback (`#/4` → `/2`, 29..574) requires **bAuto=true**, start section **FrontEvade**, a valid avatar player, **IsInCharacterState(4)=false** and absence of **Buff_FT_Shinku_CantQ**. Only that path requests **Buff_Shinku_RageMove** on the avatar and ends the Evade ability. TimeHeld is stored but not compared in this Blueprint, so the 0.25 task input must not be substituted for a second explicit elapsed-time test; the native task's automatic/release semantics and state-4 meaning remain open. This is not a grant on every ordinary dodge. 
### Rage pre-action limit and Bean removal request recovered, October 18

Initial warning-free **GA_Shinku_UltraSkill_Rage_Pre:K2_CanActivateAbility** (`#/8`) accepts the valid owning-player path only while the ASC count of **Buff_Shinku_UltraSkillPreRageLimit < 3**. Its **TriggerJumpSectionImpl** (`#/10` → `/6`, 10..455) requests that limit carrier, calls the montage parent for the requested section, rereads its count, and at **>=3** requests removal of all **Buff_Shinku_UltraRagePreBean** stacks. The carrier request is on section transitions, not in the reached activation override, which only calls the montage parent. Counts 2/3 distinguish admission; the limit reread is not proof of settled mutation before the subsequent check.

The limit carrier (`#/1`) is HasDuration **7**, AggregateByTarget, cap **4**, NeverRefresh/NeverReset, with server/standalone requirement. The separate Bean (`#/2`) is HasDuration **100**, cap **3**, DeadRemove=true, and requests **Buff_Shinku_UltraRage** on FINISH. Do not merge the two counters or infer three pre-action hits from their caps: native grant/stack conventions, whether deliberate removal delivers FINISH, clocks, refresh and already queued attack cleanup remain open. 
### Damage-trigger caller joins recovered, October 18

The [shared actor-table investigation](../../shared/action-timing.md#damage-trigger-actor-configuration-recovered-october-18) joins the saved ability/projectile subobject to its **DamageTriggerActorDataTable** row and its actual effect container. Below, tuple order is **(DelaySpawnTime, DelayTriggerTime, Period, duration, MaxTriggerCount, TriggerOnSpawn)**. These are serialized scheduler inputs, not measured hit times/counts. source ordinals below refer to the saved original bundle objects. Control/support effects are discussed separately from the damage projection.

| Caller subobject / event suffix | DamageTriggerConfigID | Configuration tuple | Selected damage effect class(es) |
| --- | --- | --- | --- |
| GA_Shinku_Skill_Rage#/10; Skill.3 | Shinku_RageGather | (0.0, 0.0, 0.0, 1.0, 1, true) | no explicit damage class |
| GA_Shinku_UltraSkill_Rage#/10; UltraSkill.1 | ShinkuUltraSkillRage | (0.0, 0.0, 0.1, 1.3, 10, true) | GE_Player_Shinku_UltraSkill_Rage_Damage |
| GA_Shinku_UltraSkill_Rage#/11; UltraSkill.2 | ShinkuUltraSkillRage1 | (0.0, 0.0, 0.0, 0.3, 1, true) | GE_Player_Shinku_UltraSkill2_Rage_Damage |
| GA_Shinku_UltraSkill_Rage_Pre#/13; UltraSkill.2 | Shinku_RageGather | (0.0, 0.0, 0.0, 1.0, 1, true) | no explicit damage class |
| GA_Shinku_UltraSkill_Rage_Pre#/14; UltraSkill.1 | ShinkuUltraSkillPreRage | (0.0, 0.0, 0.06, 0.4, 4, true) | GE_Player_Shinku_UltraSkillPre_Rage_Damage |

Rage Burst separates UltraSkillPre_Rage_Damage (period 0.06/cap 4), UltraSkill_Rage_Damage (period 0.1/cap 10), and a distinct single-count UltraSkill2_Rage_Damage follow-up. Gather entries have empty payloads and do not establish another damage packet. These are actor settings, not a proved full Rage rotation or measured hit schedule.

### Rage phase timers and removal cleanup recovered, October 18

The repaired **Buff_Shinku_Rage#/2 → GameplayCue.Display.Shinku.Rage → GC_Shinku_Rage** join supplies the phase callers missing from the grant declarations. **OnWhileActiveEvent#/6 → graph#/1:622** caches MyTarget as player. Its **HasAuthority** branch creates two AsyncTaskSetTimer tasks; a separate **IsLocalClientRole** branch creates the attempted-Burst task. The graph explicitly assigns mode-specific inputs rather than relying on its CDO defaults:

| Task / role gate | IsHybridMode=true input | false input | Timer callback request |
| --- | ---: | ---: | --- |
| RageUltraPre / HasAuthority | 9 | 13 | UltraRagePre, then UltraRagePreBean on cached player |
| RageUltra / HasAuthority | 12 | 16 | UltraRage on cached player |
| TryRageUltra / IsLocalClientRole | 12.9 | 16.9 | If NOT player.IsInPeace, TryActivateAbilityByClass(GA_Shinku_UltraSkill_Rage) |

Each timer receives **(self,player,time,false,false,-1,true)** and is saved even after an invalid-task path. The [native callback parameter metadata](../../shared/buffs-and-callbacks.md#native-task-parameter-declarations-recovered-october-18) identifies **bLooping=false, bMaxOncePerFrame=false, InFirstDelay=-1, bInPauseTimerWhenGamePause=true**. This is an explicit game-pause request, while the native first-delay sentinel, pause domain/restoration and task clocks remain open. The callback bodies do not recheck Rage presence or the creation-time role/mode gate; the attempted-activation result is not tested. Thus these are three requested phases on native task clocks, not a guaranteed Rage rotation, a universal 13-second state or confirmed successful automatic Burst. The input difference between Ultra permission and local attempt is 0.9 in either mode; native pause/age/dispatch determine actual elapsed time.

OnRemove **#/2 → 3720** restores movement/animation state and ends valid **RageUltraPre/RageUltra** tasks, requests removal of **all UltraRagePre, then UltraRagePreBean, then UltraRage**, and ends a valid TryRageUltra task. No explicit authority gate encloses that cleanup. The previously recovered Bean FINISH→UltraRage request and UltraRage BEGIN→remove Pre/Bean remain separate; successful removal-event re-entry and ordering cannot be assumed atomic. The parent Rage carrier separately requests removal on CHANGE_ROLE_OUT and INTERACTENTRY; its native duration calculator, input arbitration and already active/queued attacks remain open.

### Charge AoE cue-removal request and hit-break animation checked, October 18

**Buff_Shinku_ReactionAOE#/1 → GC_Shinku_ReactionAOE** supplies **OnRemoveEvent#/2 → graph#/1:10**, rather than an OnWhile damage call. It casts MyTarget to player, builds a container with **DamageRange_Shinku_ReactionAOE** and CDO **GE_Player_Shinku_ReactionAOE_Damage**, makes event data with Instigator=Target=that player, obtains **GA_Shinku_Melee level** through that player's net ID, then calls **MakeEffectContainerSpecFromContainer → ApplyEffectContainerSpec** on its ASC. No explicit role/positive-damage/removal-reason gate appears. The range (`#/1`) declares HalfSize(1500,1500,1500), MaxCount 30 and BindSocketName=BlueprintType_713277695; these do not override the all-in-range scenario's recipient policy. The carrier has HasDuration 0.5, target cap1, NeverRefresh/NeverReset; BEGIN independently requests Reaction_AtkUp with server/standalone admission. Native producer, removal delivery, recipient resolution and energy/damage settlement remain separate from this request.

**WatchEvadeAtk_HitBroken#/2 → GC_Shinku_WatchEvadeAtk_HitBroken** instead rotates a MonsterCharacterBase target toward incoming EffectContext instigator and plays ArmorBrokenMontage at section Default. A native timer input 3 then tests **IsBalancedingState**: true ends the task; otherwise it plays **EndWeak** if montage section index 2 has that name, or stops the montage. OnRemove ends a valid task. This callback contains no explicit native balance/Break-resource write or GE removal. It is not evidence for a universal three-second Break duration, despite the Slomo tag/name. All cited cue reads are warning-free; sources: , `sakiri-shinku-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

### Evade-hit slow cue clock inputs recovered, October 18

Warning-free **Buff_Shinku_EvadeAtk_slomo#/2 → GC_Shinku_EvadeAtk_Slomo** joins the target carrier to a separate timeline-driven dilation request. OnWhile **#/6 → graph#/2:235** casts MyTarget to Character and starts its timeline; Update **#/8 → 144** requests **SetCustomTimeDilation(curve output,3,false,true)**. Both serialized curve keys, at 0 and 0.083333, have value **0.66666**, while TimelineLength is **0.5**. The Finished callback **#/7 → 139** returns without restoring dilation; **OnRemoveEvent#/5 → 10** requests **1,3,false,true**. The carrier's HasDuration input is **0.066666**, distinct from either timeline coordinate. Type 3 is TimeDilationBuffEffect under the [native declarations](../../shared/action-timing.md#time-function-parameter-metadata); false/true name bReplicate/bUsedParticle. No role/authority predicate or Mint-specific timer-pause call appears in this body. Native cue delivery/removal, timeline scheduling, clock/dilation composition and restoration must establish actual slow time; do not replace all three source inputs with one assumed real-time deadline. Sources: `remaining-control-cue-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

### RageMove wrapper's movement and visual cleanup checked, October 18

The warning-free **Buff_Shinku_RageMove#/2 → GC_Shinku_RageMove** join is separate from the Rage phase/timer cue. OnWhile **#/8 → graph#/5:81** caches MyTarget player, requests configured entry gaits **Running/Sprinting**, saves current movement/wall/vine settings, changes animation mode and overrides those movement inputs. It also changes the weapon's special unequip montage and subscribes camera-view transition presentation. OnRemove **#/6 → 4101** requests configured end gaits **Walking/Running/Sprinting**, restores the saved movement fields, clears that weapon montage and handles mesh/emitter presentation. Its **RemoveEntryStateHideMesh#/9** scans Dialogue/Sequence states and hides the rage mesh; it does not remove a gameplay state or spend a gauge despite the helper's name. These complete functions contain no explicit damage, charge, Rage grant/removal or forced ability activation. The linked carrier's Infinite policy still makes its serialized 200 magnitude/foreign Hathor duration row unsuitable as a RageMove deadline. Native movement/animation arbitration and the separately recovered release-history/end-ability carrier removal remain open; all-in-range combat does not turn traversal speeds or visual timeline lengths into damage timing. Sources: `form-wrapper-carriers.txt`, `roster-cue-halo-extraction.json` and `cue-supplemental-joins.json`.

## Unknown execution policies

- **Real timing/admission:** Montage notify coordinates for Skill (`Shinku_Skill`), Burst (`Shinku_UltraSkill`), and follow-up Rage actions (`Shinku_UltraSkillPre_Rage`, `Shinku_UltraSkill_Rage`) provide serialized markers. Real-time conversion, hitstop, and cancel windows remain subject to the shared 3-method timing barrier.
- **Surging Crimson & Rage stance transitions:** The [Rage grant declarations](#configured-rage-ability-grants) now locate ability mappings and InputIDs (including -1). Resolve native input arbitration, effective talent mapping, and cancellation/cleanup upon form exit.
- **Menacing Gaze stack accumulation:** Off-field gain rate, stack consumption, and A4 16-stack Instant Strike detonation conditions.
- **Spatial geometry & targeting:** Skill AoE and Burst delivery use the all-in-range dummy contract; native geometry is outside product scope.


Earlier unresolved policies retained pending an evidenced answer (the new source inventory does not close them):

- The [Rage grant declarations](#configured-rage-ability-grants) are now portable. Resolve native grant/input arbitration, handling of the negative InputID, effective talent mapping and removal during an active Rage action. The base Rage and Ultimate grant removal labels differ; neither supplies natural prerequisites or use limits.
- Resolve Rage state/cost transitions and Crimson Judgment/Dragonflame Verdict admission prerequisites and use limits, branch timing, and Charge reaction eligibility/recipient/stacking behavior. Their two direct one-packet damage inputs are executable under the explicit hypothesis in the known card; this does not establish when those phases can be reached.
- Ordinary-hit energy is explicitly outside Charge reaction state; the ordinary caller rules remain a tested hypothesis.

## Prior findings

- [`reviews/q10-game-action-binding/shinku-action-binding-model.json`](../../../../reviews/q10-game-action-binding/shinku-action-binding-model.json) records the current modeled boundary and pending findings.
- Related owner investigations: [owning Rust checks](../../../../crates/content/src/simulation/tests/shinku_actions.rs), [`shinku-normal-chain.json`](../../../../reviews/q10-game-action-binding/shinku-normal-chain.json).

## Research history

- **Current question:** RageMove task/input history interpretation, native Rage resource mutation and state transitions, and natural packet clocks.
- **Useful source locations:** `GA_Shinku_Evade`, `GA_Shinku_RageMove`, `GA_Shinku_UltraSkill_Rage_Pre`, RageMove/Bean/Limit carriers and ShinkuLogicObj.
- **Methods already tried:** Watch/Rage magnitude gates and source carriers, RageMove release-history polling/cleanup and PreRage counter requests are located. The ordinary Evade callback now identifies the bAuto/FrontEvade/state-4/CantQ gate for granting RageMove; its TimeHeld value is not compared.
- **Next concrete step:** Rage cue now supplies authority-gated phase inputs 9/12 or 13/16 and local attempt 12.9/16.9, ordered cleanup and ReactionAOE OnRemove dispatch. Resolve native timer clocks/lifetime, bean-removal re-entry, ability admission and Watch/Charge producer/recipient/resource settlement. The hit-break cue changes montage state, not the Break ledger; continue remaining independent cue dependencies.

## Relative combat timer binding

Consumed owner Ultimate action-completion boundary. Explicit modeled fallback: reliable end-of-cinematic notify timing is unavailable. Positive authored owner recovery is authoritative; a zero-recovery research descriptor consumes the shared typed two-second action-recovery fallback, so it cannot silently receive a zero pause. The consumed typed binding is `CINEMATIC_RELEASE` in this owner's Rust module. Successful commitment opens a generation-owned cinematic pause; rejection opens none. Cancellation or defeat clips the executed interval. This ends the cinematic pause independently of continuing buffs, fields or summons. Global timer composition and the source/assumption boundary are defined in [shared clocks](../../shared/clocks-and-action-timing.md).
