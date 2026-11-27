# Elyms (`character:1090`, `090_elyms` / `曳尔沐`) Action, Mechanics, and Damage Dossier

Client acquisition target: `character:1090` (`090_elyms` / Chinese: `曳尔沐` / English: `Elyms`).  
Signature weapon dossier (already documented): [Echoes on Hold (`weapon:fork_elyms`)](../../weapons/echoes-on-hold/passive.md).

---

## 1. Identity, Base Stats, and Ability Progression

- **Canonical identity:** `1090` (`090_elyms`), S-rank **Cosmos** (`<Guang>` / 光属性) character wielding a **Solid Arc** (`CHARACTER_GROUP_TYPE_ONE`, matching `weapon:fork_elyms`).
  - *Provenance note:* `DT_Character.json` exported empty rows because USMAP lacks `FCharacterData` struct layout, while `DT_Character` binary `NameMap` links `1090` -> `player_090_elyms_C`, `elyms_base`, `elyms_lv`, `elyms_Breakthrough`, and `Elyms_SkillDes` / Chinese localization confirm Cosmos DMG (`<Guang>`), S-rank, and Solid Arc weapon family.
- **Base combat stats (`DT_PlayerPackData["elyms_base"]` & `DT_PlayerModifyPackData`):**
  - Base (Lv1 Stage 0): `HPMaxBase = 1270.0`, `AtkBase = 83.0`, `DefBase = 75.0`, `CritBase = 0.05` (`5%`), `CritDamageBase = 0.50` (`+50%`, before `Passive2`), `ChargeMax = 100.0`, `StrengthUseEfficiency = 1.0`, `StrengthGetEfficiency = 1.0`, `UnbalAccrueEfficiencyBase = 1.0`, `UnbalReduceNatur = 100.0`, `UnbalReduceReset = 200.0`, `TenacityRecoverSpeed = 10.0`, `TenacityResetTime = 3.0`, `SumCosmosEfficiency = 1.0`.
  - `elyms_lv_80` adds: `HPMaxBase +8026.0`, `AtkBase +328.0`, `DefBase +474.0`.
  - `elyms_stage_6` adds: `HPMaxBase +6096.0`, `AtkBase +249.0`, `DefBase +360.0`.
  - **Lv80 Stage 6 innate totals (before `Passive2` and weapon):** `HP = 15,392`, `ATK = 660`, `DEF = 909`, `Crit Rate = 5%`, `Crit DMG = +50%` (`+60%` with `Passive2` unlocked at Stage 4). With `weapon:fork_elyms` Lv80 S6 (`570 ATK`, `24% Crit Rate`), pre-buff Lv80 totals are `ATK = 1,230`, `Crit Rate = 29%`, `Crit DMG = +60%`.
- **Ability slots (`DT_CharacterAbilityConfig["1090"]` & `DT_CharacterAbilityEffectConfig`):**
  - Proactive (`AbilityIndex 1..4`): `GA_Elyms_Melee` (Normal Attack / Crosstalk / Branch / Aim / Plunge; tag `Ability.Melee`), `GA_Elyms_Skill` (Counting Electric Sheep: *The Ones Who Left*, *Intermittent Existence*, *Goodbye to You*; tag `Ability.Skill`), `GA_Elyms_UltraSkill` (+ `GA_Elyms_UltraSkillB` ST override; *Do You Get Deja Vu*; tag `Ability.UltraSkill`), `GA_Elyms_QTE` (*I'm Here*, Support Skill).
  - Passive: `GA_Elyms_Passive1` -> `Buff_Elyms_Passive1` (*Relay Node* / `中继节点`), `GA_Elyms_Passive2` -> `Buff_Elyms_Passive2` (*Pity System* / `保底机制`, unlocked at `RequireTupoLevel = 4`).
  - Non-combat Peculiarity: `GA_Elyms_Passive3` (*You There?* / `在吗在吗`).

---

## 2. Damage Packets (`DT_SkillDamageData`) and Action Totals

All Elyms damage GEs set legacy `DamageElementType = None` in `DT_SkillDamageData`, while their active runtime metadata in `tables_extracted.json` populates explicit **`DamageTypeEX = EExecutionDamageTypeEX::DAMAGE_TYPE_COSMOS`** and **`DamageSourceCategory`** (`A` for `Melee` / `MeleeB` / `Branch` / `Aim`, `X` for `AirAttack`, `E` for `Skill1` / `Skill2` / `Skill3` / `Skill3Extra`, `Q` for `UltraSkillA` / `UltraSkillB` / `XieTongSkillA1` / `XieTongSkillB1`, `Z` for `PerfectEvadeAttack`, `H` for `QTE`), with `FixedCrit = 0.0` and `ShieldBreakValue = None`. *(Note: `tables_summary.txt` printed `Type=None | Cat=None` due to a parser key mismatch; use `tables_extracted.json` for `native_damage/elyms.rs`.)* Exact 13-level `AtkRateBaseArray` indices `0..12` represent Skill Level `1..13` (index `9` = Lv10, `10` = Lv11 for R3).

### 2.1 Normal & Secondary Actions (`GA_Elyms_Melee`, `GA_Elyms_PerfectEvade`, `GA_Elyms_QTE`)

| Damage GE | Skill Label | Asset Tags | Hit Count / Action | Lv1 | Lv10 | Lv11 (R3) | Lv13 | Charge | Cycle (`Hetero`) | Unbal |
| :--- | :--- | :--- | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `GE_Player_Elyms_Melee1_Damage` | Instant Glitch 1 | `State.Damage`, `TriggerParry`, `Melee` | 2 (`BulletActor_ElymsMelee1` x2) | 11.2% | 22.4% | 24.2% | 28.2% | 0.704 | 1.170 | 0.139 |
| `GE_Player_Elyms_Melee2_Damage` | Instant Glitch 2 | `State.Damage`, `Melee` | 2 (`Melee.2` x2) | 15.7% | 31.4% | 33.9% | 39.5% | 0.982 | 1.633 | 0.194 |
| `GE_Player_Elyms_Melee3_Damage` | Instant Glitch 3a | `State.Damage`, `Melee` | 2 (`Melee.3` x2) | 33.4% | 66.8% | 72.1% | 84.1% | 2.094 | 3.482 | 0.414 |
| `GE_Player_Elyms_Melee3_1_Damage` | Instant Glitch 3b | `State.Damage`, `Melee` | 1 (`Melee.3_1` x1) | 16.6% | 33.2% | 35.8% | 41.8% | 1.039 | 1.728 | 0.206 |
| `GE_Player_Elyms_Melee4_Damage` | Instant Glitch 4a | `State.Damage`, `Melee` | 3 (`Melee.4` x3) | 13.8% | 27.6% | 29.8% | 34.8% | 0.865 | 1.438 | 0.171 |
| `GE_Player_Elyms_Melee4_1_Damage` | Instant Glitch 4b | `State.Damage`, `Melee` | 2 (`Melee.4_1` x2) | 14.2% | 28.4% | 30.7% | 35.8% | 0.888 | 1.476 | 0.176 |
| `GE_Player_Elyms_Melee5_Damage` | Instant Glitch 5 | `State.Damage`, `Melee` | 1 (`Melee.5` x1; HitPause) | 91.5% | 182.9% | 197.5% | 230.4% | 5.733 | 9.534 | 1.134 |
| `GE_Player_Elyms_Branch1_Damage` | Interference a | `State.Damage`, `Melee` | 2 (`Branch.1` x2) | 10.7% | 21.4% | 23.1% | 26.9% | 0.667 | 1.119 | 0.143 |
| `GE_Player_Elyms_Branch2_Damage` | Interference b | `State.Damage`, `Melee` | 1 (`Branch.2` x1) | 10.7% | 21.4% | 23.1% | 26.9% | 0.667 | 1.119 | 0.143 |
| `GE_Player_Elyms_Branch3_Damage` | Interference c | `State.Damage`, `Melee` | 1 (`Branch.3` x1; HitPause) | 12.8% | 25.6% | 27.6% | 32.2% | 0.800 | 1.343 | 0.171 |
| `GE_Player_Elyms_MeleeB1_Damage` | Crosstalk 1a | `State.Damage`, `TriggerParry`, `Melee` | 1 (`Melee.1` x1) | 12.4% | 24.8% | 26.8% | 31.2% | 0.775 | 1.289 | 0.157 |
| `GE_Player_Elyms_MeleeB1_1_Damage` | Crosstalk 1b | `State.Damage`, `TriggerParry`, `Melee` | 1 (`Melee.1_1` x1) | 13.5% | 27.0% | 29.1% | 34.0% | 0.845 | 1.406 | 0.171 |
| `GE_Player_Elyms_MeleeB2_Damage` | Crosstalk 2a | `State.Damage`, `Melee` | 2 (`Melee.2` x2) | 9.3% | 18.6% | 20.1% | 23.4% | 0.580 | 0.964 | 0.117 |
| `GE_Player_Elyms_MeleeB2_1_Damage` | Crosstalk 2b | `State.Damage`, `Melee` | 2 (`Melee.2_1` x2) | 9.5% | 19.0% | 20.5% | 23.9% | 0.596 | 0.992 | 0.121 |
| `GE_Player_Elyms_MeleeB3_Damage` | Crosstalk 3a | `State.Damage`, `Melee` | 3 (`Melee.3` x3) | 10.6% | 21.2% | 22.9% | 26.7% | 0.664 | 1.104 | 0.134 |
| `GE_Player_Elyms_MeleeB3_1_Damage` | Crosstalk 3b | `State.Damage`, `Melee` | 1 (`Melee.3_1` x1) | 47.0% | 94.0% | 101.5% | 118.4% | 2.936 | 4.882 | 0.594 |
| `GE_Player_Elyms_Aim_Damage` | Memory Anchor | `State.Damage`, `Branch.AimAttack`, `Bullet`, `Melee` | 1 (`Branch.7` projectile) | 12.0% | 24.0% | 25.9% | 30.2% | 0.800 | 1.300 | 0.200 |
| `GE_Player_Elyms_AirAttack_Damage` | Falling Dream | `State.Damage`, `Branch.AirAttack`, `Melee` | 1 (`AirAttack.1`, +up to 100% height) | 29.6% | 59.2% | 63.9% | 74.5% | 1.900 | 3.100 | 0.400 |
| `GE_Player_Elyms_PerfectEvadeAttack_Damage` | Okay! Shake on It! | `State.Damage`, `PerfectEvadeDamage`, `TriggerParry` | 1 (`Melee.1`) | 100.0% | 199.9% | 215.9% | 251.8% | 2.500 | 4.200 | 2.500 |
| `GE_Player_Elyms_QTE_Damage` | I'm Here | `State.Damage`, `TriggerParry`, `QTE` | 1 (`QTE.1`; HitPause) | 200.0% | 399.8% | 431.8% | 503.6% | 5.200 | 0.000 | 2.500 |

### 2.2 Redirect Skills, Ultimates, and Coordinated Attacks (`GA_Elyms_Skill`, `GA_Elyms_UltraSkill*`, `GA_Elyms_XieTongSkill*`)

| Damage GE | Skill Label | Asset Tags | Hit Count / Action | Lv1 | Lv10 | Lv11 (R3) | Lv13 | Charge | Cycle | Unbal |
| :--- | :--- | :--- | :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `GE_Player_Elyms_Skill1_Damage` | The Ones Who Left a | `State.Damage`, `Skill.Elyms.Skill1` | 1 (`Skill.1`) | 26.9% | 53.8% | 58.1% | 67.7% | 0.808 | 1.331 | 0.158 |
| `GE_Player_Elyms_Skill1_1_Damage` | The Ones Who Left b | `State.Damage`, `Skill.Elyms.Skill1` | 4 (`Skill.1_1` x4) | 23.6% | 47.2% | 51.0% | 59.4% | 0.707 | 1.164 | 0.139 |
| `GE_Player_Elyms_Skill1_2_Damage` | The Ones Who Left c | `State.Damage`, `Skill.Elyms.Skill1` | 1 (`Skill.1_2`) | 48.8% | 97.6% | 105.4% | 122.9% | 1.464 | 2.412 | 0.287 |
| `GE_Player_Elyms_Skill2_Damage` | Intermittent Existence a | `State.Damage`, `Skill.Elyms.PowerSkill1` | 5 (`Skill.2` x5) | 10.2% | 20.4% | 22.0% | 25.7% | 0.850 | 0.000 | 0.000 |
| `GE_Player_Elyms_Skill2_1_Damage` | Intermittent Existence b | `State.Damage`, `Skill.Elyms.PowerSkill1` | 1 (`Skill.2_1`) | 48.8% | 97.6% | 105.4% | 122.9% | 4.050 | 0.000 | 0.000 |
| `GE_Player_Elyms_Skill3_Damage` | Goodbye to You a | `State.Damage`, `Skill.Elyms.PowerSkill2` | 5 (`Skill.3` x5) | 13.1% | 26.2% | 28.3% | 33.0% | 0.850 | 0.000 | 0.000 |
| `GE_Player_Elyms_Skill3_1_Damage` | Goodbye to You b | `State.Damage`, `Skill.Elyms.PowerSkill2` | 1 (`Skill.3_1`) | 62.2% | 124.3% | 134.3% | 156.6% | 4.050 | 0.000 | 0.000 |
| `GE_Player_Elyms_Skill3Extra_Damage` | R6 Final Extra Hit (per stack `n`) | `State.Damage`, `Skill` | 1 (`Skill3_1` on `Con_Elyms_Lv6ExtraDam`) | 168.0% | 335.8% | 362.7% | 423.1% | 0.000 | 0.000 | 0.000 |
| `GE_Player_Elyms_UltraSkillA1_Damage` | Do You Get Deja Vu (AoE) a | `State.Damage`, `UltraSkill`, `Elyms.UltraskillA` | 3 (`UltraSkill.1` x3) | 32.0% | 64.0% | 69.1% | 80.6% | 0.000 | 0.000 | 0.119 |
| `GE_Player_Elyms_UltraSkillA2_Damage` | Do You Get Deja Vu (AoE) b | `State.Damage`, `UltraSkill`, `Elyms.UltraskillA` | 1 (`UltraSkill.2`; HitPause inside pause) | 309.0% | 617.7% | 667.1% | 778.1% | 0.000 | 0.000 | 1.144 |
| `GE_Player_Elyms_UltraSkillB1_Damage` | Do You Get Deja Vu (ST) a | `State.Damage`, `UltraSkill`, `Elyms.UltraskillB` | 3 (`UltraSkill.1` x3) | 35.6% | 71.2% | 76.9% | 89.6% | 0.000 | 0.000 | 0.119 |
| `GE_Player_Elyms_UltraSkillB2_Damage` | Do You Get Deja Vu (ST) b | `State.Damage`, `UltraSkill`, `Elyms.UltraskillB` | 4 (`UltraSkill.2` x4) | 69.8% | 139.5% | 150.7% | 175.8% | 0.000 | 0.000 | 0.233 |
| `GE_Player_Elyms_UltraSkillB3_Damage` | Do You Get Deja Vu (ST) c | `State.Damage`, `UltraSkill`, `Elyms.UltraskillB` | 1 (`UltraSkill.3`) | 64.0% | 127.9% | 138.2% | 161.2% | 0.000 | 0.000 | 0.213 |
| `GE_Player_Elyms_XieTongSkillA1_Damage` | Remembrance (AoE) | `State.Damage`, `Elyms.XieTongA`, `Nanally.XieTongDamage` | 4 per trigger (x6 max = 24) | 27.0% | 54.0% | 58.3% | 68.0% | 0.425 | 0.700 | 0.075 |
| `GE_Player_Elyms_XieTongSkillB1_Damage` | Remembrance (ST) | `State.Damage`, `Elyms.XieTongB`, `Nanally.XieTongDamage` | 4 per trigger (x6 max = 24) | 30.0% | 60.0% | 64.8% | 75.5% | 0.425 | 0.700 | 0.075 |

### 2.3 Per-Action Aggregate Totals

| Action | Lv1 | Lv10 | Lv11 (R3) | Charge | Cycle | Unbal |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| `Instant Glitch` N1 / N2 / N3 / N4 / N5 | 22.4% / 31.4% / 83.4% / 69.8% / 91.5% | 44.8% / 62.8% / 166.8% / 139.6% / 182.9% | 48.4% / 67.8% / 180.0% / 150.8% / 197.5% | 1.408 / 1.964 / 5.227 / 4.371 / 5.733 | 2.340 / 3.266 / 8.692 / 7.266 / 9.534 | 0.278 / 0.388 / 1.034 / 0.865 / 1.134 |
| `Instant Glitch` Full N1–N5 | 298.5% | 596.9% | 644.5% | 18.703 | 31.098 (`~31` UI) | 3.699 |
| `Interference` (Hold Basic, Weak) | 44.9% | 89.8% | 96.9% | 2.801 | 4.700 | 0.600 |
| `Crosstalk` B1 / B2 / B3 | 25.9% / 37.6% / 78.8% | 51.8% / 75.2% / 157.6% | 55.9% / 81.2% / 170.2% | 1.620 / 2.352 / 4.928 | 2.695 / 3.912 / 8.194 | 0.328 / 0.476 / 0.996 |
| `Crosstalk` Full B1–B3 | 142.3% | 284.6% | 307.3% | 8.900 | 14.801 (`~15` UI) | 1.800 |
| `The Ones Who Left` (`Skill1`) | 170.1% | 340.2% (A5: 680.4%) | 367.5% (A5: 735.0%) | 5.100 | 8.399 (+100 via Buff) | 1.001 |
| `Intermittent Existence` (`Skill2`) | 99.8% | 199.6% | 215.4% | 8.300 | 0.000 | 0.000 |
| `Goodbye to You` (`Skill3` base) | 127.7% | 255.3% | 275.8% | 8.300 | 0.000 | 0.000 |
| `Do You Get Deja Vu` AoE Base (Weak / Med +50% / Strong +100%) | 405.0% / 607.5% / 810.0% | 809.7% / 1214.55% / 1619.4% | 874.4% / 1311.6% / 1748.8% | 0.000 | 0.000 | 1.501 |
| `Do You Get Deja Vu` ST Base (Weak / Med +50% / Strong +100%) | 450.0% / 675.0% / 900.0% | 899.5% / 1349.25% / 1799.0% | 971.7% / 1457.55% / 1943.4% | 0.000 | 0.000 | 1.502 |
| `Remembrance` AoE 1 proc (x4 hits) / 6 procs | 108.0% / 648.0% (Strong: 1296.0%) | 216.0% / 1296.0% (Strong: 2592.0%) | 233.2% / 1399.2% (Strong: 2798.4%) | 1.700 / 10.200 | 2.800 / 16.800 | 0.300 / 1.800 |
| `Remembrance` ST 1 proc (x4 hits) / 6 procs | 120.0% / 720.0% (Strong: 1440.0%) | 240.0% / 1440.0% (Strong: 2880.0%) | 259.2% / 1555.2% (Strong: 3110.4%) | 1.700 / 10.200 | 2.800 / 16.800 | 0.300 / 1.800 |

---

## 3. Cooldowns, Costs, and `DT_ElymsEffectFigure`

- **Cooldowns (`CoolDwonCurveTable`):**
  - `GA_Elyms_Skill = 16.0s` (Lv1..7). Only base `Skill1` (*The Ones Who Left*) pays this cooldown (`CoolDown.Player.Skill.ShortPress`). Both `Buff_Elyms_PowerE1Dur` and `Buff_Elyms_PowerE2Dur` set `bIgnoreCoolDown = true` on `GA_Elyms_Skill` via `HTAssignAbilityExtraInfoGEComponent`, so `Skill2` (*Intermittent Existence*) and `Skill3` (*Goodbye to You*) ignore and do not wait on the 16s cooldown.
  - `GA_Elyms_UltraSkill = 20.0s`, `GA_Elyms_UltraSkillB = 20.0s`.
- **Costs (`DT_GameplayAbilityCostData`):**
  - `GA_Elyms_Evade`: `GAStrengthCost = 15.0` stamina.
  - `GA_Elyms_UltraSkill` / `GA_Elyms_UltraSkillB`: `GACost = 100.0` Ultimate Energy.
- **`DT_ElymsEffectFigure` (all 50 keys extracted):**
  - `ElymsPowerE1Dur = 12.0s` (`Buff_Elyms_PowerE1Dur` window for `Skill2`).
  - `ElymsPowerE2Dur = 6.0s` (`Buff_Elyms_PowerE2Dur` window for `Skill3`).
  - `ElymsERecoverEnergy = 100.0` (`Buff_Elyms_QRecoverEnergy` Cycle Energy gain on `Skill1` cast).
  - `ElymsP1ToP2Stack = 1.0`, `ElymsP2ToP3Lv2Stack = 3.0` (Optical Illusion gained on Weak->Med, and Med->Strong when **A3** is unlocked; base stack limit without A3 is `1`, raised to `4` by A3).
  - `ElymsPowerE2DamUpDur = 15.0s`, `ElymsPowerE2CoefModify = 1.0` (`+100%` DMG Ratio multiplier via `CoefModify` per Optical Illusion stack for `Skill3`), `Elyms_Buff_Elyms_PowerE2DamUpCnt = 4.0`, `Elyms_Buff_Elyms_TotalPowerE2ConsumedStack = 4.0`.
  - `ElymsXieTongCoefModify1 = 0.5` (Med AoE Q), `ElymsXieTongCoefModify2 = 0.5` (Med ST Q), `ElymsXieTongCoefModify3 = 1.0` (Strong AoE Q), `ElymsXieTongCoefModify4 = 1.0` (Strong ST Q), `ElymsXieTongCoefModify5 = 0.5` (Med AoE Remembrance), `ElymsXieTongCoefModify6 = 0.5` (Med ST Remembrance), `ElymsXieTongCoefModify7 = 1.0` (Strong AoE Remembrance), `ElymsXieTongCoefModify8 = 1.0` (Strong ST Remembrance).
  - `ElymsQAtkUp = 0.2` (`20%` of Elyms's `AtkBase`), `ElymsQAtkUpLevelExtra = 0.0` (`Cau_Elyms_QAtkBuff`), `ElymsQZhuijiUp = 0.1` (`+10%` Follow-up Attack DMG), `ElymsQAtkUpDur = 20.0s`.
  - `ElymsXieTongAPer = 3.0s`, `ElymsXieTongAMax = 6.0`, `ElymsXieTongBPer = 3.0s`, `ElymsXieTongBMax = 6.0`.
  - `ElymsPas2Def = 0.1` (`Buff_Elyms_Passive2` -> `+10% CritDamageBase`).
  - Passive 1 (`Relay Node`): `Reaction_GuangLingXiang5_DamageUpCosmosBase_1090 = 0.0375` / `Time = 30.0s` and Remora `7.5s` / `7.0s` ICD are confirmed from localized descriptions (`Elyms_SkillDes`) and `Buff_Elyms_Passive1` / native `ElymsLogicObj`; note `DT_ReactionEffectFigure` / `DT_ReactionExtendDataTable` row `1090` is **not yet extracted** into `tables_extracted.json`.
  - Awakenings (with `.uexp` attribute-name traps verified): `ElymsLv1DamUp = 0.1` (`ElymsLv1Dur = 20.0s`), `ElymsLv2CritDamageUp = 0.05` (actually modifies **`DefIgnore`** on `Buff_Elyms_PowerE2DamUp`, gated by `Con_Elyms_Lv2`), `ElymsLv3ExtraQCoefModify = 0.3`, `ElymsLv4Def = 0.2` (`ElymsLv4DefDur = 20.0s`, actually modifies **`CritDamageBase`** on `Buff_Elyms_Level4_1`), `ElymsLv5ECoefModify = 1.0`, `Elyms_Lv5CD = 30.0s`, `ElymsLv6XieTongDamUp = 0.25` (`ElymsLv6XieTongDamUpDur = 20.0s`), `ElymsLv6ExtraXiShu = 1.0`, `ElymsLvExtraPenetrateGeneral = 0.12` (`DamagePenetrate*`). (Unused/zero rows: `ElymsLv1Mag = 0.0`, `ElymsLv3Reaction1/6 = 0.0`, `ElymsLv6ExtraXieTongCoefModify = 0.0`.)

---

## 4. Montage Timing, Timer Pauses (`HTGAComponent_AbilityGamePaused`), HitPause, and Ranges

### 4.1 Timer-Pause Architecture (`HTGAComponent_AbilityGamePaused`)

Unlike standard characters that only attach `HTGAComponent_AbilityGamePaused` to Ultimate abilities, Elyms attaches it to **both** `GA_Elyms_UltraSkill`/`GA_Elyms_UltraSkillB` **and** `GA_Elyms_Skill`:
1. **`GA_Elyms_UltraSkill` (AoE Q) & `GA_Elyms_UltraSkillB` (ST Q):**
   - `HTGAComponent_AbilityGamePaused_0` triggers with `TriggerEndEventTags = ["Event.Montage.Player.Common.EndAbilitySkill"]`.
   - `Elyms_UltraSkill` & `Elyms_UltraSkill2` both hold `BP_CantChangePlayerNS_C`, `NotifyState_LimitPassiveSkillEffect`, `BP_ANS_PlayCameraSequence_C` (at `3.0s`, `BP_CantChangePlayerNS_C` at `0.0s`), and emit `EndAbilitySkill` at **`5.499266s`** (`5_499_266 us`) (`BP_TriggerEndAbilityEffect_C` at `5.569568s` / `5.566667s`).
   - *Model note:* This long `5.499266s` pause covers the full `Do You Get Deja Vu` animation/cinematic (`3.0s` sequence inside the montage, first hit at `3.124s`).
2. **`GA_Elyms_Skill` (`Skill2` / `Skill3` ONLY):**
   - `HTGAComponent_AbilityGamePaused_0` has `ActiveRequirement` = `AnyRequirementMet` with 2 sub-requirements (section checks corresponding to `Skill2` and `Skill3`), and `TriggerEndEventTags = ["Event.Montage.Player.Common.EndAbilitySkill"]`.
   - **`Elyms_Skill1` (*The Ones Who Left*)** has **NO** `BP_ANS_PlayCameraSequence_C`, **NO** `BP_CantChangePlayerNS_C`, and **NO** `EndAbilitySkill` notify -> **does NOT pause the timer**.
   - **`Elyms_Skill2` (*Intermittent Existence*)** and **`Elyms_Skill3` (*Goodbye to You*)** BOTH carry `BP_ANS_PlayCameraSequence_C`, `BP_CantChangePlayerNS_C`, `NotifyState_LimitPassiveSkillEffect`, `BP_AddLooseGameplayTagsNS_C` (`State.CantbreakSkill`, `State.Property.ImmuneDamage`, `State.Property.ImmuneAniState`, `State.Property.CantUseEvade`), and emit `Event.Montage.Player.Common.EndAbilitySkill` at **`1.719655s`** (`1_719_655 us`), ~`50ms` after `BP_TriggerEndAbilityEffect_C` (`1.669059s` on `Skill2` / `1.666238s` on `Skill3`) and `BP_CanBreakSkill_C(Move/AnySkill/Jump)` (`1.666549s` / `1.666667s`).

### 4.2 HitPause (`HitPauseData`), Projectiles, and Frame Table

- **HitPause (`HitPauseData`):**
  - `GE_Player_Elyms_Branch3_Damage`: `0.02s @ rate 0.01` (`19_800 us` lost progress on final `Branch.3` hit at `0.533002s`, before `0.649976s` `EndAbility`).
  - `GE_Player_Elyms_Melee5_Damage`: `0.03s @ rate 0.01` (`29_700 us` lost progress on `Melee.5` hit at `0.605117s`, before `0.632180s` `AddE` / `1.305152s` `EndAbility`). Note `19_800` / `29_700 us` are lost montage-clock progress if hits connect.
  - `GE_Player_Elyms_QTE_Damage`: `DelayTime = 0.0167s`, `0.02s @ rate 0.1` (`18_000 us` lost progress after `0.747546s` hit, before `1.246321s` `EndAbility`).
  - `GE_Player_Elyms_UltraSkillA2_Damage`: `0.05s @ rate 0.03` (`48_500 us`, occurs at `4.616667s` inside the Ultimate game-pause window).
- **Projectiles:**
  - `Melee1` (`AppearMelee` / `Appear_Parry` / `MeleePre`): spawns `BulletActor_ElymsMelee1` at `0.131303s` (`Melee.1`) and `0.307749s` (`Melee.1_1`); `ProjectileNormalAccSpeed` has `DelayTime = 0.068s`, `NormalAccSpeed = 1,000,000.0`, `MaxSpeed = 16,000.0`, homing `InitOpenTrack = true`.
  - `Aim` (`Elyms_AimFire`): spawns `ElymsBranchProjectile` at `0.048313s` (`InitialSpeed = 40,000.0`, `MaxSpeed = 50,000.0`).
- **Montage Notify & Cancel Timing Summary (`Animation/Skill/*.json`):**

| Montage | `SeqLen` | Damage / Effect Notifies (montage seconds) | Cancel / Switch / Recovery Gates |
| :--- | ---: | :--- | :--- |
| `Elyms_Melee1` / `MeleePre` / `AppearMelee` | `4.8333335s` | `0.131303` (`Melee.1` projectile), `0.307749` (`Melee.1_1` projectile) | `JumpSection` `Melee2` at `0.084138` (`nFrame=23`); Skill `0.421018`, Move `0.599339`, EndAbility `0.657555`, AnySkill `1.084427` |
| `Elyms_Melee2` | `10.0s` | `0.163746`, `0.261461` (`Melee.2` x2) | `JumpSection` `Melee3` at `0.083026` (`nFrame=25`); Skill `0.459984`, Move `0.779794`, EndAbility `0.788579`, AnySkill `1.084427` |
| `Elyms_Melee3` | `4.1666665s` | `0.337874`, `0.624525` (`Melee.3` x2), `0.835709` (`Melee.3_1`) | `JumpSection` `Melee4` at `0.084604` (`nFrame=65`); Skill `1.274223`, AnySkill `1.598623`, Move `1.598705`, EndAbility `2.018442` |
| `Elyms_Melee4` | `7.8333335s` | `0.086976`, `0.307586`, `0.516182` (`Melee.4` x3); `0.194800`, `0.411490` (`Melee.4_1` x2) | `JumpSection` `Melee5` at `0.081980` (`nFrame=28`); AnySkill `0.812723`, Skill `0.955727`, Move `1.966649`, EndAbility `2.104604` |
| `Elyms_Melee5` | `4.3333335s` | `0.605117` (`Melee.5` + HitPause), **`0.632180` (`Melee.Elyms.AddE` -> grants `PowerE1Dur` 12s)** | Jump `0.769057`, AnySkill `1.038993`, Move `1.295037`, EndAbility `1.305152` |
| `Elyms_Branch1` | `4.1666665s` | `0.070101` (`Branch.1`), `0.180677` (`Branch.2`), `0.314019` (`Branch.1`), `0.533002` (`Branch.3` + HitPause) | Jump `0.549787`, EndAbility `0.649976`, AnySkill/Move `0.650035` |
| `Elyms_MeleeB1` / `MeleeBPre` / `AppearMeleeB` | `3.0s` | `0.164788` (`Melee.1`), `0.355115` (`Melee.1_1`) | `JumpSection` `MeleeB2` at `0.084604` (`nFrame=22`); Skill `0.452851`, AnySkill `0.570405`, Move `0.984953`, EndAbility `1.001050` |
| `Elyms_MeleeB2` | `3.0s` | `0.165958`, `0.301134` (`Melee.2` x2); `0.232924`, `0.365986` (`Melee.2_1` x2) | `JumpSection` `MeleeB3` at `0.081980` (`nFrame=18`); Skill `0.383873`, AnySkill `0.549281`, Jump `0.561142`, EndAbility `1.060205`, Move `1.067342` |
| `Elyms_MeleeB3` | `4.116667s` | `0.148066`, `0.247854`, `0.380905` (`Melee.3` x3), **`0.617396` (`Melee.Elyms.AddE` -> grants `PowerE2Dur` 6s)**, `0.665314` (`Melee.3_1`) | AnySkill/Jump `0.933511`, Skill `0.943447`, Move `1.415258`, EndAbility `1.508954` |
| `Elyms_AimFire` | `0.75s` | `0.048313` (`Branch.7` projectile) | Jump/AnySkill `0.146353` |
| `Elyms_AirAttackEnd` | `4.4s` | `0.050169` (`AirAttack.1`) | Jump `0.123557`, AnySkill `0.255141`, Move `0.749160`, EndAbility `0.766767` |
| `Elyms_PerfectEvadeAtk` | `4.95s` | `0.182751` (`Melee.1`), `0.651798` (`EndAbilitySkill` buffer cleanup) | Jump/Skill `0.0`, AnySkill `0.387993`, EndAbility `0.651818`, Move `0.652065` |
| `Elyms_EvadeForward` / `Back` | `1.4833s` / `2.6667s` | None | Forward: AnySkill `0.332778`, EndAbility `0.431326`; Back: AnySkill `0.383426`, EndAbility `0.499987`, Move `0.583301` |
| `Elyms_QTE` | `4.1666665s` | `0.747546` (`QTE.1` + HitPause `18_000 us`), `1.368357` (`EndAbilitySkill`) | `CantChangePlayerNS` from `0.0s`; AnySkill/Jump `1.034234`, Move `1.240934`, EndAbility `1.246321` |
| `Elyms_Skill1` | `3.3333333s` | `0.163746` (`Skill.1`), `0.461007 / 0.523651 / 0.625732 / 0.711157` (`Skill.1_1` x4), **`0.818769` (`AddE` -> `PowerE1Dur`)**, `0.821274` (`Skill.1_2`) | **No camera sequence / no pause**; Skill `0.483052`, Move `1.003819`, AnySkill `1.084427`, EndAbility `1.309221` |
| `Elyms_Skill2` | `3.8333333s` | `0.097422 / 0.288095 / 0.469234 / 0.664674 / 0.844009` (`Skill.2` x5), `0.604148` (`Manage` if R3) + `0.605117` (`Manage`), `1.186104` (`Skill.2_1`) | **`CantChangePlayerNS` & CameraSequence & `AbilityGamePaused`** -> Move/AnySkill/Jump `1.666549`, EndAbility `1.669059`, **`EndAbilitySkill` `1.719655s`** |
| `Elyms_Skill3` | `4.1666665s` | `0.097422 / 0.288095 / 0.469234 / 0.664674 / 0.844009` (`Skill.3` x5), `1.137738` (`Manage`), `1.186104` (`Skill.3_1` + R6 Extra Hit) | **`CantChangePlayerNS` & CameraSequence & `AbilityGamePaused`** -> EndAbility `1.666238`, Move/AnySkill/Jump `1.666667`, **`EndAbilitySkill` `1.719655s`** |
| `Elyms_UltraSkill` (AoE) | `13.5s` | `3.124098 / 3.328750 / 3.544312` (`UltraSkill.1` x3), `4.616667` (`UltraSkill.2` x1) | **`CantChangePlayerNS` & `AbilityGamePaused`** -> AnySkill/Jump `5.322640`, Move `5.484152`, **`EndAbilitySkill` `5.499266s`**, EndAbility `5.569568` |
| `Elyms_UltraSkill2` (ST) | `13.5s` | `3.126049 / 3.328750 / 3.544312` (`UltraSkill.1` x3), `4.628571 / 4.741408 / 4.845375 / 4.987917` (`UltraSkill.2` x4), `5.166667` (`UltraSkill.3` x1) | **`CantChangePlayerNS` & `AbilityGamePaused`** -> AnySkill/Jump `5.316667`, Move `5.483333`, **`EndAbilitySkill` `5.499266s`**, EndAbility `5.566667` |
| `Elyms_XieTongA` / `B` | `1.9166666s` / `1.25s` | Spawn `DamageTriggerActor` config `PlayerElymsXieTongA` (`Period=0.12s`, `MaxTriggerCount=4`, `TriggerOnSpawn=true`, `duration=2.0s`); montage `UltraSkill.1` at `0.254680` / `0.326896` | Off-field entity (`DamageRange_Elyms_XieTongA`: sphere `r=1000`, `MaxCount=10`; `DamageRange_Elyms_XieTongB`: sphere `r=1000`, `MaxCount=1`) |

- **All DamageRange Dimensions (for dummy count / multi-target eligibility):**
  - `Skill1`: Box `HalfSize=(600, 400, 300)`, `Offset/Trace=(600, 0, 0)`.
  - `Skill2` & `Skill3`: Box `HalfSize=(700, 800, 600)`, `Offset/Trace=(700, 0, 0)`.
  - `UltraSkill` (both AoE & ST): Box `HalfSize=(1500, 1500, 700)`, `Offset/Trace=(1500, 0, 0)`.
  - `RefreshUltraSkillRange` (target scan for ST vs AoE): Box `HalfSize=(2000, 2000, 2000)`, `ObjectTypes=[3, 12]`.
  - `XieTongA`: Sphere `r=1000`, `MaxCount=10`; `XieTongB`: Sphere `r=1000`, `MaxCount=1`.
  - `SearchRange_Elyms_XieTong`: Sphere `r=1500`, `MaxCount=1`.

---

## 5. Core State Machine, Buffs, Passives, and Awakenings

### 5.1 Connection Strength (`Buff_Elyms_Authority`), Optical Illusion, and Skill Redirect Flow

1. **Three Connection Strength states (`Buff_Elyms_Authority` stacks):**
   - **Weak (`0` stacks):** Default on battle start, after casting `Do You Get Deja Vu`, or upon leaving combat (`BUFF_LEAVE_BATTLE` removes `Buff_Elyms_Authority`). Basic Attack is 5-hit `Instant Glitch` (`GA_Elyms_Melee`). **Observed simulator rule:** combat entry grants no `Buff_Elyms_PowerE1Dur`, so the first Skill is `The Ones Who Left` (`Skill1`); its `AddE` at `0.818769s` grants the `12.0s` `Skill2` window if Elyms is still Weak. This intentionally differs from `CharacterTeachGuide` / CN tutorial text (*"入战时，强化技能「不连续的存在」直接处于可施放状态"*), which claims `Skill2` is immediately available for `12s` on entry.
   - **Medium (`1` stack):** Reached from Weak via `Intermittent Existence` (`Skill2`) or on-field `I'm Here` (`GA_Elyms_QTE`). Basic Attack switches to 3-hit `Crosstalk` (`GA_Elyms_MeleeB`). Entering Medium from Weak grants **`Optical Illusion`** (`Buff_Elyms_PowerE2DamUp`, `1` stack, `15.0s` duration; `Buff_Elyms_Authority` `BUFF_EVENT_BEGIN` also removes `Buff_Elyms_PowerE1Dur`).
   - **Strong (`2` stacks):** Reached from Medium via `Goodbye to You` (`Skill3`, unlocked for `6.0s` by the 3rd hit of `Crosstalk` at `0.617396s` or instantly via R3).
2. **`GA_Elyms_Skill` Redirect Logic:**
   - **At Weak without `PowerE1Dur`:** Casts **`The Ones Who Left` (`Skill1`)** -> puts `GA_Elyms_Skill` on `16.0s` CD -> `Buff_Elyms_QRecoverEnergy` adds **`+100` Cycle Energy** (instant Cycle readiness), `AddE` (`0.818769s`) grants `Buff_Elyms_PowerE1Dur` (`12.0s`, `bIgnoreCoolDown=true`).
   - **With `PowerE1Dur` (`12s`, only valid while NOT having `Authority`):** Casts **`Intermittent Existence` (`Skill2`)** -> removes `PowerE1Dur`, triggers `AbilityGamePaused` (`1.719655s`), `0.605117s` applies `Authority` (`Weak -> Medium`, or `-> Strong` at R3) and calls `RefreshUltraSkill`.
   - **At Medium or Strong without `PowerE2Dur`:** Can still cast **`The Ones Who Left` (`Skill1`)** if off its 16s CD to refill `100` Cycle Energy (however `PowerE1Dur` cannot apply while `Authority` is active, per `Con_Elyms_NotHaveAuthorityBuff`), which explains why C0 rotation is `QTE (Weak->Med) -> E (Skill1, +100 Cycle) -> Crosstalk B1-B3 -> E (Skill3, Med->Strong) -> Q`!
   - **With `PowerE2Dur` (`6s`, requires `Authority`):** Casts **`Goodbye to You` (`Skill3`)** -> removes `PowerE2Dur`, triggers `AbilityGamePaused` (`1.719655s`), consumes all `Optical Illusion` stacks (`+100%` `CoefModify` per stack), adds consumed count to `TotalPowerE2ConsumedStack` (`max 4`), advances `Medium -> Strong`, and calls `RefreshUltraSkill`.
3. **Ultimate (`Do You Get Deja Vu`) & Coordinated Attack (`On the Line` / `Remembrance`):**
   - Skill2, Skill3, and QTE invoke `RefreshUltraSkill`, which checks `RefreshUltraSkillRange` at that action. A scan of one living target selects single-target `GA_Elyms_UltraSkillB`; any other count selects AoE `GA_Elyms_UltraSkill`. The Ultimate uses the last scan and always uses AoE at Weak Connection.
   - `Cau_Elyms_Authoriy_SkillCoef` scales both `Do You Get Deja Vu` and subsequent `Remembrance` DMG Ratios by Connection Strength snapshot at cast time (`LastAuthorityOnUltraSkillActivated`): Weak `+0%`, Medium `+50%` (`+0.5`), Strong `+100%` (`+1.0` `CoefModify`).
   - Grants team `Buff_Elyms_QAtkBuff` for **`20.0s`**: **`+20%` of Elyms's `AtkBase`** (`+132` ATK at Lv80 S6 without weapon) + **`+10%` Follow-up Attack DMG** (note `ATK` bonus uses Elyms's `GetAtkBase()`, which includes Base Character ATK + Weapon Base ATK!).
   - Grants **`On the Line`**: while Elyms is **off-field**, automatically spawns `GA_Elyms_XieTongSkillA` (AoE) or `B` (ST) every **`3.0s`** up to **`6` triggers**; each trigger spawns `DamageActor` (`Period=0.12s`, `4` hits). Ends once 6 triggers occur (see Section 6 for initial swap/re-entry timing). Deals `Follow-up Attack` DMG (`Ability.Player.Nanally.XieTongDamage`). Connection Strength resets to Weak on Ultimate cast.

### 5.2 Passives and Awakenings (`1090AwakenEffect.json` & `Upgrade/*`)

- **Passive 1 (`Relay Node` / `中继节点`):**
  1. *Charge Enhancement:* Whenever the **active character** gains Ultimate Energy, whole team gains `+3.75%` Cosmos DMG for `30s`, max `8` stacks (`+30%`), refreshes duration (`DT_ReactionEffectFigure`).
  2. *Remora Enhancement:* When any ally applies **Remora**, Elyms reapplies **Remora** to the same target **`7.5s`** later (`7.0s` ICD; self-applied Remora cannot re-trigger this). Both the `7.5s` delay and `7.0s` ICD operate on relative combat time (`ClockDomain::World`, pausing during `Skill2`/`Skill3`/`UltraSkill` pauses alongside the Remora status itself, rather than advancing on `ClockDomain::Absolute`).
- **Passive 2 (`Pity System` / `保底机制`):** Unconditional `+10%` Crit DMG (`CritDamageBase += 0.10`) for Elyms.
- **Awakenings & Resonances:**
  - **A1 (`404: Island Out of Reach`):** Casting `Do You Get Deja Vu` grants whole team `+10%` Crit DMG for `20s` per Optical Illusion stack consumed before casting Q (`max 4` = `+40%`), clears accumulated count and refreshes on next Q.
  - **A2 (`200: Nodes Connected`):** `Do You Get Deja Vu` DMG Ratio `+30%` (`CoefModify += 0.30`); `Goodbye to You` ignores `5%` target DEF per Optical Illusion stack consumed (`Buff_Elyms_PowerE2DamUp` `DefIgnore += 0.05/stack`).
  - **A3 (`504: Gateway Timeout`):** Upgrading Medium -> Strong grants `3` stacks of `Optical Illusion`, after Skill3 consumes the existing stacks if it triggered the upgrade. Another Crosstalk finisher can unlock Skill3 at Strong to consume those three stacks. `Optical Illusion` max stacks raised to `4` (and `TotalPowerE2ConsumedStack` max `4`).
  - **A4 (`302: Wish Redirected`):** While Elyms is on the team and alive, whole team gains `+20%` Crit DMG (`Buff_Elyms_Level4` / `4_1`).
  - **A5 (`304: Nothing Has Changed`):** `The Ones Who Left` DMG Ratio `+100%` (`CoefModify += 1.0`); on combat entry, Cycle Energy set to `100` (`30s` CD, max `1` time per battle).
  - **A6 (`301: Moved Permanently`):** Casting `Do You Get Deja Vu` grants Elyms `+25%` Follow-up Attack DMG for `20s` per Optical Illusion stack consumed before Q (`max 4` = `+100%`).
  - **R3 (`For Auld Lang Syne, My Dear`):** `+1` level to Basic / Redirect Skill / Ultimate (Lv10 -> Lv11). Casting `I'm Here` (QTE) or `Intermittent Existence` (`Skill2`) at Weak advances **directly to Strong**, triggering both Weak->Med (`+1` Optical Illusion) and Med->Strong (at A3: `+3` -> `4` stacks total!), and immediately unlocks `Goodbye to You` (`Skill3`) -> enables fast `E (Skill2) -> E (Skill3, 4 stacks) -> Q` rotation!
  - **R6 (`Until Next Time, New Friend`):** Whole team ignores `12%` All Resistances while Elyms is alive (`Buff_Elyms_LevelExtra6_1`). On the final hit of `Goodbye to You`, if Optical Illusion was consumed, deals `1` extra hit of `GE_Player_Elyms_Skill3Extra_Damage` (`Lv10 335.8%`, `Lv11 362.7%`, with `CoefModify = 1.0 * (stacks - 1)`, i.e. `335.8% * n_stacks`, or `1450.8%` at Lv11 & 4 stacks).

- **`Buff_Elyms_QAtkBuff` whole-team & Follow-up verification:** Although `GA_Elyms_UltraSkillB_ef_ElymsQAtkUp_des` says *"Other Allies' ATK Bonus"*, both `GA_Elyms_UltraSkill_des` and `Cau_Elyms_QAtkBuff` + `.uexp` properties confirm `Buff_Elyms_QAtkBuff` grants `+20%` of Elyms's `AtkBase` (`AtkAdd`) **and** `+10%` Follow-up Attack DMG (`ElymsQZhuijiUp = 0.1`, checked via `FollowUpDamage` and `Con_Elyms_isXieTongDamage` matching `Ability.Player.Nanally.XieTongDamage`) to **the whole team including Elyms** for `20.0s` (`OwnerActor == HitActor` only adds `ElymsQAtkUpLevelExtra = 0.0`). On cast (`K2_ActivateAbility` -> `GA_ElymsUltraSkillBase:UltraSkillAddBuffFun`), Elyms also converts `TotalPowerE2ConsumedStack` into `Buff_Elyms_Lv1_1` / `Buff_Elyms_Lv6_1` and clears `TotalPowerE2ConsumedStack` immediately so `Do You Get Deja Vu` itself receives `A1` Crit DMG.
- **Asset & guide caveats verified:**
  - `GE_Player_Elyms_XieTongSkillA1_Damage` / `B1_Damage` carry native asset tag `Ability.Player.Nanally.XieTongDamage` (not `State.Damage.Followup`); `crates/content` must bind both packets to `FOLLOW_UP_DAMAGE_TAG` (`State.Damage.Followup`) in `packet_semantics.rs` (just like Haniel) so `weapon:fork_elyms` (*Echoes on Hold*) counts `Remembrance` as Follow-up Attack DMG.
  - `GA_Elyms_PerfectEvade` and `GA_Elyms_XieTongSkillB` contain dormant cloned `Zankou` (`Buff_Zankou_Level1_C` / `SpreadDotDamage`) checks in `OnReceiveEvent`; they never trigger on Elyms and fall through cleanly to `GA_MontageBase_C::OnReceiveEvent`.
  - Several community web guides misnumber or swap awakenings (e.g., `neverness.gg` places `R3`'s Weak -> Strong jump in `A2`, while some guides mix up `R3` and `R6`). Always follow `1090AwakenEffect` + `elyms_locres_en.json`. *(Also note `A1` / `A6` stack tracking and `R3` Weak -> Strong without `A3` follow canonical individual awakening rules: `R3` without `A3` triggers Weak -> Med `+1` Optical Illusion and Med -> Strong `+0`, remaining capped at `1`.)*
  - `Elyms_Appear` (`1.45s`) transitions to `AppearMelee` / `AppearMeleeB` at `0.049422s` (`nFrame=12`); `Elyms_PerfectEvadeFront` (`1.6333s`) / `Back` (`3.4833s`) jump to `PerfectEvadeAtk` (`4.95s`, `fDelayTriggerTime=0.033s`). `Buff_Elyms_BackendManagement` also carries a `BUFF_EVENT_APPLY_REACTION` / `BUFF_LEAVE_BATTLE` self-remove entry in JSON alongside its 6-trigger limit.

---

## 6. Selected Modeling Decisions and Engine Prerequisites

### 6.1 Confirmed Modeling Decisions

1. **`GA_Elyms_Skill` & `GA_Elyms_UltraSkill*` Timer Pause (`HTGAComponent_AbilityGamePaused`):** Release strictly at the `Event.Montage.Player.Common.EndAbilitySkill` notify: **`1_719_655 us`** on `Skill2` (*Intermittent Existence*) and `Skill3` (*Goodbye to You*), and **`5_499_266 us`** on `UltraSkill` / `UltraSkillB` (*Do You Get Deja Vu* AoE/ST). `Skill1` (*The Ones Who Left*) has no camera sequence or pause.
2. **Observed Start-of-Combat Rule & Skill Cooldown:** Battle starts at Weak Connection without `Buff_Elyms_PowerE1Dur`; the first `skill` cast is `Skill1` (*The Ones Who Left*) and starts its `16.0s` cooldown. `Skill1`'s `AddE` at `0.818769s` grants `Buff_Elyms_PowerE1Dur` for `12.0s` if still Weak, unlocking `Skill2`; Weak Normal 5's `AddE` also grants this window. `Skill2` and `Skill3` ignore and preserve the existing Skill cooldown. This is the current simulator rule, not the tutorial's immediate-entry `Skill2` readiness claim.
3. **`On the Line` / `Remembrance` Off-Field Coordinated Attacks (`3.0s` x `6`):** Trigger logic resides in native C++ `ElymsLogicObj` and is undocumented in Blueprints/guides beyond the `3.0s` interval and `6` max limit. Selected model: first *Remembrance* trigger fires immediately upon switching Elyms off-field (`swap_off_field + 0s`), then every `3.0s` while off-field (up to `6` triggers total, `0.12s * 4` hits each); returning on-field pauses the timer until she leaves the field again.
4. **`Optical Illusion` Stack Accounting (`A1` / `A6` / `R6`):** Track accumulated `TotalPowerE2ConsumedStack` strictly from stacks actually consumed by casting `Goodbye to You` (`Skill3`), capped at `4`, matching the localized `A1` / `A6` / `R6` descriptions.
5. **Projectile Delivery & `AtkBase` Scope:** Treat `Melee1` (`BulletActor_ElymsMelee1`) and `Aim` (`ElymsBranchProjectile`) as zero-flight or nominal-hit delivery unless range latency is explicitly modeled; `Cau_Elyms_QAtkBuff` reads Elyms's `GetAtkBase()` (Character Base ATK + equipped Weapon Base ATK, excluding flat/percent ATK buffs).

### 6.2 Simulator Implementation & Resolved Design Decisions

- **Non-Ultimate `ModePauseKind::Cinematic` admission:** Implemented in `crates/content/src/characters/cinematics.rs` (`action_binding`) and `crates/content/src/simulation/{admission,pause}.rs` so `Skill2` and `Skill3` admit `1_719_655 us` `Cinematic` pauses (`Skill1` admits none, `UltraSkill`/`UltraSkillB` admit `5_499_266 us`), shifting HitPause deadlines uniformly without altering existing characters.
- **Active-actor Ultimate Energy observer (Passive 1 *Charge Enhancement*):** Implemented via `notify_active_actor_energy_gain` -> `registry::on_active_actor_energy_gain` across combat hit grants, `Charge` reaction, and kit/weapon grants (excluding `t = 0` start energy and off-field-only grants). Grants `+3.75%` team Cosmos DMG stacks (`30.0s`, max `8` = `+30%`, resets duration).
- **Passive 1 *Remora Enhancement* anti-recursion:** Implemented via `apply_from_owner_with_retrigger_control(..., allow_elyms_retrigger: bool)` in `crates/content/src/simulation/reactions/remora.rs`. Ally `Remora` triggers schedule a `7.5s` delayed `Remora` on `ClockDomain::World` under a `7.0s` global ICD (`ElymsPas1CD`), skipping cleanly if Elyms or the target dies and setting `allow_elyms_retrigger = false` while cooperating with Akane `Perfect Pitch`, Hathor `B2`, and Chaos `B2`.
- **`Aim`, `State.Damage.Followup`, and 2-cycle oracle verification:** `Memory Anchor` (`GE_Player_Elyms_Aim_Damage`) uses the `146_353 us` `AnySkill`/`Jump` cancel cadence (`48_313 us` hit). Both `XieTongSkillA1_Damage` and `XieTongSkillB1_Damage` carry `FOLLOW_UP_DAMAGE_TAG` (`State.Damage.Followup`), satisfying `weapon:fork_elyms` (*Echoes on Hold*), and are covered alongside full C0/C6 mechanics and a 2-cycle oracle in `crates/content/src/simulation/tests/elyms_actions.rs`.
- **Remaining external note:** `DT_ReactionExtendDataTable` / `DT_ReactionEffectFigure` row `1090` remains unextracted in `tables_extracted.json`, with Passive 1 numeric values anchored directly from `Elyms_SkillDes` and `Buff_Elyms_Passive1`.
