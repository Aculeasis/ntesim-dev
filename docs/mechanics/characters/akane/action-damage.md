# Akane Rin (`character:1057` / `057_Akane`) mechanics and source data

Collected from live game client assets (no game binaries executed), localized English/Chinese skill tables (`DT_GameplayAbilityTipsData`, `Akane_SkillDes`), `DT_SkillDamageData`, `DT_AkaneEffectFigure`, `1057AwakenEffect`, `CoolDwonCurveTable`, `DT_GameplayAbilityCostData`, `DT_PlayerPackData`, `DT_PlayerModifyPackData`, `AnimMontage` notifies, and Blueprint bytecode/NameMaps under `Ability_057_Akane/`. Raw acquisition exports remain in `cache/acquisition/akane-current/` (gitignored).

Implementation status: **implemented in `ntesim-data` and `ntesim-content`.** Existing `openspec/specs/studio-application/spec.md` keeps Akane Rin excluded from the frozen 21-team comparison collection unless separately updated.

---

## 1. Identity, base stats, and progression

- **Native ID / Slug / Name:** `1057` (`057_Akane`) / `akane` / **Akane Rin** (script aliases: `Akane`, `Akane Rin`).
- **Rarity / Element / Weapon Group:** S-Class (`Orange`), **Lakshana** (`Element::Lakshana`), **Liquid** / `WeaponGroup::Two` (matches already-implemented exclusive Arc `weapon:fork_prism`, *Far Side of the Moon*).
- **Registered Abilities (`DT_CharacterAbilityConfig[1057]`):**
  - Proactive (`CharacterAbilityList`): `GA_Akane_Melee` (index 1, English title **`"Stringblade"`**), `GA_Akane_Skill` (2, **`"Rhythm Wave"`**), `GA_Akane_UltraSkill` (3, **`"Sonic Boom"`** — grants `GA_Akane_UltraSkill_End` `"Sonic Boom"` / `"Ultimate: Sonic Boom - Stage 2"`), `GA_Akane_QTE` (4, **`"Altered Chord"`**).
  - Native GameplayTags (`DT_CharacterAbilityEffectConfig`): `GA_Akane_Melee` -> `Ability.Melee`, `GA_Akane_Skill` -> `Ability.Skill`, `GA_Akane_UltraSkill` -> `Ability.UltraSkill`; `GA_Akane_Passive_1` activates `Buff_Akane057_Passive1`, `GA_Akane_Passive_2` activates `Buff_Akane057_Passive2`. (`GA_Akane_PerfectEvade` -> `Ability.Evade.PerfectEvade`; `GA_Akane_Evade` -> `Ability.Evade`.)
  - Passives (`PassiveAbilityList`): `GA_Akane_Passive_1` (**Perfect Pitch**), `GA_Akane_Passive_2` (**Starlit Anthem**).
  - Derived / Reaction / Dodge: `GA_Akane_PerfectEvade` (**Critical Riposte: Featherblade Reverb**), `GA_Akane_Evade`, `GA_Akane_Appear`, `GA_Akane_Skill_instead` + `GA_Akane_Skill_instead_PlayMontage` (**Synced Resonance**), `GE_Reaction_3_new_1057_Damage` (Perfect Pitch Remora proc). Overworld gimmick `GA_Akane_Skill_Gitar` (*Play It by Ear*) is non-combat.
- **Skill upgrade material family (`DT_CharacterAbilityConfig[1057]`):** `SkillUpMaterial_04_lv1..3` + `OrdinaryMonMaterial_04_lv1..3` + `weeklycloneboss_a01_01`.

### Base Stats & Level/Breakthrough Scaling (`DT_PlayerPackData`, `DT_PlayerModifyPackData`)

Matches standard `ntesim` level (`+8%` HP/DEF, `+5%` ATK per level above Lv1) and breakthrough stage (`+80%` HP/DEF, `+50%` ATK per stage) formulas:

| Snapshot | HPMaxBase | AtkBase | DefBase | ChargeMax | CritBase | CritDamageBase |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: |
| Base (`Akane_base`, Lv1 S0) | 1320 | 80 | 76 | 100 | 0.05 | 0.50 |
| Level Growth (`Akane_lv_80`) | +8342 | +316 | +480 | — | — | — |
| Stage Growth (`Akane_stage_1`) | +1056 | +40 | +61 | — | — | — |
| Stage Growth (`Akane_stage_6`) | +6336 | +240 | +365 | — | — | — |
| **Lv80 S6 Total** | **15998** | **636** | **921** | **100** | **0.05** | **0.50** |

---

## 2. Damage packet inventory (`DT_SkillDamageData`)

All 16 rows deal **Lakshana** damage (`DAMAGE_TYPE_LAKSHANA`), `FixedCritRate = 0.0`, scaling 100% with ATK (`DefRateBaseArray = []`, `HPRateBaseArray = []`). Numbers below are exact decimal ratios (`0.149 = 14.9%`, stored in Rust `ntesim_data` as integer millionths).

| Packet ID (`GE_*`) | Cat | Owner `GAName` | ChargeAdd | UnbalValue | HeterochromeAdd | Hits / Cast Role |
| :--- | :---: | :--- | ---: | ---: | ---: | :--- |
| `GE_Player_Akane_Melee1_Damage` | A | `GA_Akane_Melee` | 1.244 | 0.216 | 1.780 | N1 x2 (and `AppearMelee` x2 / `MeleePre` x2) |
| `GE_Player_Akane_Melee2_Damage` | A | `GA_Akane_Melee` | 1.920 | 0.333 | 2.748 | N2 x2 |
| `GE_Player_Akane_Melee3_Damage` | A | `GA_Akane_Melee` | 0.796 | 0.138 | 1.140 | N3 first 3 hits (`Melee.3` x2 + `Melee.3_1` x1) |
| `GE_Player_Akane_Melee3_1_Damage` | A | `GA_Akane_Melee` | 0.992 | 0.172 | 1.419 | N3 final 4 rapid hits (`Melee.3_2` x4) |
| `GE_Player_Akane_Melee4_Damage` | A | `GA_Akane_Melee` | 4.016 | 0.697 | 5.847 | N4 x1 |
| `GE_Player_Akane_Airattack_Damage` | X | `GA_Akane_Melee` | 3.100 | 0.600 | 5.200 | Stage Dive plunge x1 (+up to 100% fall-height scaling) |
| `GE_Player_Akane_Branch_Damage` | A | `GA_Akane_Melee` | 4.100 | 0.800 | 6.800 | **Unused/legacy** (`0.001..0.003`; no held-basic branch in live kit) |
| `GE_Player_Akane_PerfectEvadeAttack_Damage` | Z | `GA_Akane_PerfectEvade` | 2.500 | 2.500 | 4.200 | Critical Riposte: Featherblade Reverb x1 |
| `GE_Player_Akane_QTE_Damage` | H | `GA_Akane_QTE` | 6.500 | 2.500 | 0.000 | Switch-in QTE: Altered Chord x1 |
| `GE_Player_Akane_Skill_Damage` | E | `GA_Akane_Skill` | 4.200 | 0.700 | 6.000 | Redirect Skill Press: Overtone initial hit x1 (+A3 `CoefAdd`) |
| `GE_Player_Akane_Skill2_Damage` | E | `GA_Akane_Skill` | 2.850 | 0.500 | 4.100 | Redirect Skill Hold: Reverb Field initial hits x2 |
| `GE_Player_Akane_UpgradeSkill_Damage` | E | `GA_Akane_Skill` | 0.000 | 0.000 | 0.000 | Synced Resonance base AoE/base (`Upskill_Short_CoefAdd` added in Overtone; +R3 `CoefAdd`) |
| `GE_Player_Akane_UpgradeSkill_DisplayGE` | E | `None` | 0.000 | 0.000 | 0.000 | **Display-only UI row** (= `UpgradeSkill_Damage + Upskill_Short_CoefAdd`); do not emit as separate hit |
| `GE_Player_Akane_UltraSkill1_Damage` | Q | `GA_Akane_UltraSkill` | 0.000 | 0.150 | 0.000 | Sonic Boom Stage 1 x10 hits |
| `GE_Player_Akane_UltraSkill2_Damage` | Q | `GA_Akane_UltraSkill` | 0.000 | 1.500 | 0.000 | Sonic Boom Stage 2 x1 hit (+R3 `CoefAdd`) |
| `GE_Reaction_3_new_1057_Damage` | E | `None` | 0.000 | 0.000 | 0.000 | Perfect Pitch Remora-reapply proc x1 (counts as Skill DMG) |

### Full 13-Coordinate `AtkRateBaseArray` (Levels 1–13)

| Packet ID | Lv1 | Lv2 | Lv3 | Lv4 | Lv5 | Lv6 | Lv7 | Lv8 | Lv9 | Lv10 | Lv11 | Lv12 | Lv13 |
| :--- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `Melee1_Damage` | 0.149 | 0.161 | 0.174 | 0.188 | 0.203 | 0.219 | 0.236 | 0.255 | 0.276 | 0.298 | 0.322 | 0.347 | 0.375 |
| `Melee2_Damage` | 0.219 | 0.237 | 0.255 | 0.276 | 0.298 | 0.322 | 0.348 | 0.375 | 0.405 | 0.438 | 0.473 | 0.511 | 0.551 |
| `Melee3_Damage` | 0.103 | 0.111 | 0.120 | 0.130 | 0.140 | 0.151 | 0.163 | 0.177 | 0.191 | 0.206 | 0.222 | 0.240 | 0.259 |
| `Melee3_1_Damage` | 0.135 | 0.146 | 0.157 | 0.170 | 0.184 | 0.198 | 0.214 | 0.231 | 0.250 | 0.270 | 0.291 | 0.315 | 0.340 |
| `Melee4_Damage` | 4.247 | 4.587 | 4.954 | 5.350 | 5.778 | 6.240 | 6.739 | 7.279 | 7.861 | 8.490 | 9.169 | 9.902 | 10.695 |
| `Airattack_Damage` | 0.625 | 0.675 | 0.729 | 0.787 | 0.850 | 0.918 | 0.992 | 1.071 | 1.157 | 1.249 | 1.349 | 1.457 | 1.574 |
| `Branch_Damage` | 0.001 | 0.001 | 0.001 | 0.001 | 0.001 | 0.001 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 | 0.002 | 0.003 |
| `PerfectEvadeAttack_Damage` | 1.375 | 1.485 | 1.604 | 1.732 | 1.871 | 2.020 | 2.182 | 2.357 | 2.545 | 2.749 | 2.969 | 3.206 | 3.462 |
| `QTE_Damage` | 2.000 | 2.160 | 2.333 | 2.519 | 2.721 | 2.939 | 3.174 | 3.428 | 3.702 | 3.998 | 4.318 | 4.663 | 5.036 |
| `Skill_Damage` | 3.600 | 3.888 | 4.199 | 4.535 | 4.898 | 5.290 | 5.713 | 6.170 | 6.663 | 7.196 | 7.772 | 8.394 | 9.065 |
| `Skill2_Damage` | 1.620 | 1.750 | 1.890 | 2.041 | 2.204 | 2.380 | 2.571 | 2.776 | 2.999 | 3.238 | 3.497 | 3.777 | 4.079 |
| `UpgradeSkill_Damage` | 10.125 | 10.935 | 11.810 | 12.755 | 13.775 | 14.877 | 16.067 | 17.352 | 18.741 | 20.240 | 21.859 | 23.608 | 25.496 |
| `UpgradeSkill_DisplayGE` | 11.250 | 12.150 | 13.122 | 14.172 | 15.306 | 16.530 | 17.852 | 19.281 | 20.823 | 22.489 | 24.288 | 26.231 | 28.329 |
| `UltraSkill1_Damage` | 0.648 | 0.700 | 0.756 | 0.816 | 0.882 | 0.952 | 1.028 | 1.111 | 1.199 | 1.295 | 1.399 | 1.511 | 1.632 |
| `UltraSkill2_Damage` | 9.180 | 9.914 | 10.708 | 11.564 | 12.489 | 13.488 | 14.568 | 15.733 | 16.992 | 18.351 | 19.819 | 21.404 | 23.117 |
| `Reaction_3_new_1057` | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 | 1.500 |

### Action packet totals (Lv1 / Lv8 / Lv9 / Lv10, per single target)

| Action (Packet Train) | Raw ATK Sum (`Lv1 / Lv8 / Lv9 / Lv10`) | Raw `ChargeAdd` | `HeterochromeAdd` | `UnbalValue` | String Energy |
| :--- | :--- | ---: | ---: | ---: | ---: |
| `GA_Akane_Melee` N1–N4 (12 hits: 2 + 2 + 7 + 1) | `5.832 / 9.994 / 10.796 / 11.660` | `16.700` | `23.999` | `2.897` | `+10` (`1+1+2+6`) |
| `GA_Akane_Melee` N1–N4 under `Overtone` (`+40%` CoefAdd) | `8.1648 / 13.9916 / 15.1141 / 16.3231` | `16.700` | `23.999` | `2.897` | `+10` |
| `GA_Akane_Melee` N1–N4 under A5 `Overtone` (`+64%` CoefAdd) | `9.5646 / 16.3910 / 17.7048 / 19.1214` | `16.700` | `23.999` | `2.897` | `+10` |
| `GA_Akane_Skill` Press (`Overtone`, x1 base; `+10.0` at A3) | `3.600 / 6.170 / 6.663 / 7.196` | `4.200` | `6.000` | `0.700` | `+6` |
| `GA_Akane_Skill` Hold (`Reverb Field`, x2 hits) | `3.240 / 5.552 / 5.998 / 6.476` | `5.700` | `8.200` | `1.000` | `+6` |
| `Synced Resonance` (`Overtone`: `UpgradeSkill + Upskill_Short`; `+8.0` at R3) | `11.2500 / 19.2801 / 20.8233 / 22.4889` | `0.000` | `0.000` | `0.000` | `-30` on hit (`+10` R6) |
| `Synced Resonance` (`Reverb Field`: `UpgradeSkill`; `+8.0` at R3) | `10.125 / 17.352 / 18.741 / 20.240` | `0.000` | `0.000` | `0.000` | `-30` on hit (`+10` R6) |
| `Sonic Boom` Stage 1 (`UltraSkill1` x10) | `6.480 / 11.110 / 11.990 / 12.950` | `0.000` | `0.000` | `1.500` | `+0` |
| `Sonic Boom` Stage 2 (`UltraSkill2` x1; `+10.0` at R3) | `9.180 / 15.733 / 16.992 / 18.351` | `0.000` | `0.000` | `1.500` | `+0` |
| `Altered Chord` (`GA_Akane_QTE` x1) | `2.000 / 3.428 / 3.702 / 3.998` | `6.500` | `0.000` | `2.500` | `+4` |
| `Featherblade Reverb` (`GA_Akane_PerfectEvade` x1 base) | `1.375 / 2.357 / 2.545 / 2.749` | `2.500` | `4.200` | `2.500` | `+2` |

---

## 3. Cooldowns, costs, and `DT_AkaneEffectFigure`

- **Cooldowns (`CoolDwonCurveTable`):**
  - `GA_Akane_Skill`: **16.0s** (shared by Press `Overtone` and Hold `Reverb Field`).
  - `GA_Akane_UltraSkill`: **20.0s**.
  - `GA_Akane_UltraSkill_End`: **0.1s** (unlocked during `Solo Cadenza` from `t >= 4.0s` until `t < 20.0s`).
  - `GA_Akane_Skill_instead` (**Synced Resonance**): **0.0s** independent cooldown; does not affect `Rhythm Wave`'s 16.0s cooldown.
- **Costs (`DT_GameplayAbilityCostData`):**
  - `GA_Akane_UltraSkill`: `GACost = 100.0` Ultimate Energy (`ChargeMax = 100.0`). Note `DT_GameplayAbilityCostData` also lists `100.0` on `GA_Akane_UltraSkill_End`, whereas the active gameplay ability `GA_Akane_UltraSkill_End` omits `HTAbilityChargeCost` and `Solo Cadenza` allows free manual Stage 2 casting with zero extra energy cost.
  - `GA_Akane_Evade`: `GAStrengthCost = 15.0` stamina.

### `DT_AkaneEffectFigure` Scaling & Constant Rows

| Curve Row | Values (Lv1..13) | Consumer / Purpose |
| :--- | :--- | :--- |
| `Upskill_Short_CoefAdd` | `[1.125, 1.215, 1.3122, 1.4172, 1.5306, 1.653, 1.7852, 1.9281, 2.0823, 2.2489, 2.4288, 2.6231, 2.8329]` | `Calculation_Akane_Upskill_AtkPerUP`: additive ratio added to `UpgradeSkill_Damage` under `Overtone` (`GA_Akane_Skill` level), yielding `1012.5% + 112.5% = 1125.0%` Lv1 / `2024.0% + 224.89% = 2248.89%` Lv10 |
| `Upskill_Long_CoefAdd` | `0.0` | Reverb Field Synced Resonance additive `CoefAdd` |
| `GE_Player_Akane_Melee1_Damage_CoefAdd` | `[0.0596, 0.0644, 0.0695, 0.0751, 0.0811, 0.0876, 0.0946, 0.1021, 0.1103, 0.1191, 0.1287, 0.139, 0.1501]` | Overtone `+40%` ratio boost per Melee1 hit (`GA_Akane_Melee` level) |
| `GE_Player_Akane_Melee1_Damage_CoefAdd_level5` | `[0.0954, 0.103, 0.1113, 0.1202, 0.1298, 0.1402, 0.1514, 0.1635, 0.1766, 0.1907, 0.206, 0.2224, 0.2402]` | A5 Overtone `+64%` ratio boost per Melee1 hit |
| `GE_Player_Akane_Melee2_Damage_CoefAdd` | `[0.0876, 0.0946, 0.1022, 0.1104, 0.1192, 0.1287, 0.139, 0.1501, 0.1621, 0.1751, 0.1891, 0.2043, 0.2206]` | Overtone `+40%` ratio boost per Melee2 hit |
| `GE_Player_Akane_Melee2_Damage_CoefAdd_level5` | `[0.1402, 0.1514, 0.1635, 0.1766, 0.1907, 0.206, 0.2225, 0.2403, 0.2595, 0.2803, 0.3027, 0.3269, 0.353]` | A5 Overtone `+64%` ratio boost per Melee2 hit |
| `GE_Player_Akane_Melee3_Damage_CoefAdd` | `[0.0412, 0.0445, 0.0481, 0.0519, 0.0561, 0.0605, 0.0654, 0.0706, 0.0763, 0.0824, 0.0889, 0.0961, 0.1037]` | Overtone `+40%` ratio boost per Melee3 (`0.103..`) hit |
| `GE_Player_Akane_Melee3_Damage_CoefAdd_level5` | `[0.0659, 0.0712, 0.0769, 0.083, 0.0897, 0.0968, 0.1046, 0.1129, 0.122, 0.1317, 0.1423, 0.1537, 0.1659]` | A5 Overtone `+64%` ratio boost per Melee3 hit |
| `GE_Player_Akane_Melee3_1_Damage_CoefAdd` | `[0.054, 0.0583, 0.063, 0.068, 0.0735, 0.0793, 0.0857, 0.0925, 0.1, 0.1079, 0.1166, 0.1259, 0.136]` | Overtone `+40%` ratio boost per Melee3_1 (`0.135..`) hit |
| `GE_Player_Akane_Melee3_1_Damage_CoefAdd_level5` | `[0.0864, 0.0933, 0.1008, 0.1088, 0.1175, 0.1269, 0.1371, 0.1481, 0.1599, 0.1727, 0.1865, 0.2015, 0.2176]` | A5 Overtone `+64%` ratio boost per Melee3_1 hit |
| `GE_Player_Akane_Melee4_Damage_CoefAdd` | `[1.6988, 1.8347, 1.9815, 2.14, 2.3112, 2.4961, 2.6958, 2.9114, 3.1444, 3.3959, 3.6676, 3.961, 4.2779]` | Overtone `+40%` ratio boost per Melee4 hit |
| `GE_Player_Akane_Melee4_Damage_CoefAdd_level5` | `[2.7181, 2.9355, 3.1704, 3.424, 3.6979, 3.9938, 4.3133, 4.6583, 5.031, 5.4335, 5.8682, 6.3376, 6.8446]` | A5 Overtone `+64%` ratio boost per Melee4 hit |
| `GE_Player_Akane_PerfectEvadeAttack_Damage_CoefAdd` | `[0.55, 0.594, 0.6415, 0.6928, 0.7483, 0.8081, 0.8728, 0.9426, 1.018, 1.0995, 1.1874, 1.2824, 1.385]` | Overtone `+40%` ratio boost on Featherblade Reverb |
| `GE_Player_Akane_PerfectEvadeAttack_Damage_CoefAdd_level5` | `[0.88, 0.9504, 1.0264, 1.1085, 1.1972, 1.293, 1.3964, 1.5082, 1.6288, 1.7591, 1.8999, 2.0518, 2.216]` | A5 Overtone `+64%` ratio boost on Featherblade Reverb |
| `GE_Player_Akane_Airattack_Damage_CoefAdd` / `_level5` | `0.0` | Stage Dive gets no Overtone CoefAdd (and has no `DamageUpGeneralBase` under Reverb Field) |
| `Skill_Long_DamageUp` | `0.60` | `Calculation_Akane_Level4_Long`: `+60%` `DamageUpGeneralBase` during `Reverb Field` |
| `Skill_Long_AtkDamageUp_Level5` | `0.90` | `Calculation_Akane_Level4_Long`: `+90%` `DamageUpGeneralBase` with A5 during `Reverb Field` |
| `SoundMarkDuration` / `SoundFieldDuration` | `25.0` | Duration (world seconds) of `Overtone` and `Reverb Field` |
| `Player_Akane_Passive2_UltraskillDamageUp` / `_CD` | `0.20` / `20.0` | `Starlit Anthem`: `+20%` `AtkUp` for `20s` (`Buff_Akane_Skill_UltraSkill_passive2addDamage`) |
| `Player_Akane_Level1_Treat` / `DefUp` / `SuperArmorDuration` | `10.0` / `0.50` / `15.0` | A1: Heal `1000%` Base ATK; `+50%` DEF & `60%` Poise (`SuperArmorCoefficient`) for `15s` |
| `Player_Akane_DamageRseistBase` | `-0.12` | A2: `-12%` `DamageResistLakshanaBase` on Overtone / Reverb Field targets |
| `Skill_Short_DamageUp_Level3` | `10.0` | A3 (`Calculation_Akane_Level3`): `+1000%` fixed ratio on Press `Rhythm Wave` (`Skill_Damage`) |
| `Player_Akane_Level4_DamageUP` / `_CD` | `0.30` / `15.0` | A4 (`Buff_Akane_Skill_insteadSkill_Level3`): `+30%` `CritDamageBase` for `15s` on `Synced Resonance` |
| `Player_Akane_Level6_DefIgnore` / `GlobleDamageUP_CD` | `0.16` / `20.0` | A6 (`Buff_Akane057_Level6_GlableDamageUP`): `16%` `DefIgnore` during `Starlit Anthem` (`20s`) |
| `Player_Akane_Level3EX_UpSkill_CoefAdd` | `8.0` | R3 (`Calculation_Akane_Level3EX_UpSkill`): `+800%` fixed ratio on `Synced Resonance` (`UpgradeSkill_Damage`) |
| `Player_Akane_Level3EX_Ultra2_CoefAdd` | `10.0` | R3 (`Calculation_Akane_Level3EX_Ultra2`): `+1000%` fixed ratio on `Sonic Boom` Stage 2 (`UltraSkill2_Damage`) |
| `Awaken_6Rhythm_Value` | `10.0` | R6: `10` String Energy refunded per `Synced Resonance` cast |
| `Player_Akane_Level6EX_CritDamageUP` / `_CD` | `0.64` / `25.0` | R6 (`Buff_Akane057_Level6EX_DamageUP`): `+64%` `CritDamageBase` for `25s` on `Synced Resonance` |
| Unused legacy rows in table | — | `Player_Akane_Level2_DamageUP=0.125`, `Player_Akane_Level1_CritBase=0.12`, `Player_Akane_Level3_DamageUP=1.0`, `Player_Akane_Level6_GlobleDamageUP=0.1`, `Player_Akane_Level3EX_DamageUP=0.0` (unreferenced or zeroed by live abilities/GAs; similarly `Upgrade/Level2/Buff_Akane057_Level2_Damageup` and `Upgrade/Level3/Buff_Akane057_Level3_DamageUP` are orphaned assets). In `Cau_Reaction3_1057`, local handles `DT_FemaleEffectFigure` (`Female_Pas_2_1`, `Female_Pas_2_2`) and tag `Ability.Player.Adler.SkillDamageTag` are unused template leftovers copied from Zero (`Cau_GuangLingXiang1_Female`). |

- **String Energy Values in `DT_AkaneEffectFigure`:** `Melee1_Damage = 1.0`, `Melee2_Damage = 1.0`, `Melee3_Damage = 2.0`, `Melee3_1_Damage = 0.0`, `Melee4_Damage = 6.0`, `Airattack_Damage = 1.0`, `PerfectEvadeAttack_Damage = 2.0`, `QTE_Damage = 4.0`, `Skill_Damage = 6.0`, `Skill2_Damage = 6.0`, `UltraSkill1 = 0.0`, `UltraSkill2 = 0.0`, `UpgradeSkill = 0.0`.
- **UI Cycle Sum Constants (`*_Heterochrome_Show`):** `Melee_Heterochrome_Show = 24.0` (`1.78*2 + 2.748*2 + 1.14*3 + 1.419*4 + 5.847 = 23.999`), `Skill_Heterochrome_Show = 6.0`, `Skill2_Heterochrome_Show = 8.0` (`4.1 * 2 = 8.2`, rounded to `8` in display table; packet emits exact `4.1` per hit).

---

## 4. Montage timing, projectile delivery, HitPause, and cancel/switch gates

Extracted from `HT/Content/Characters/Player/057_Akane_fashion2/Animation/Skill/*.uasset` (exact notify start/end bounds; convert to integer microseconds in Rust) and CDO `SpawnActorMaps` / `EffectContainerMap` / `SkillEffect`:

- **HitPause (`HitPauseData`) audit & lost progress:** Only `melee_2_Montage` (`0.02s @ rate 0.01`, `19_800 us` lost progress per hit -> `39_600 us` across both `Melee.2` hits before `0.436721s` BreakJump / `0.491588s` EndAbility), `melee_3_Montage` (`0.02s @ rate 0.01` on `Melee.3` x2 and `Melee.3_1` x1 = `59_400 us`, plus `0.02s @ rate 0.1` = `18_000 us` on each of the 4 `Melee.3_2` hits; since `0.812638 / 0.835430 / 0.849551 / 0.867402s` have montage gaps `>= 0.014s` vs `0.002s` advance during `0.02s @ 0.1`, all 4 are non-overlapping -> `72_000 us` -> `131_400 us` total lost progress in N3 if all hit), and `ultraskill_Montage` (`0.02s @ rate 0.01` on `UltraSkill1_Damage`) declare `HitPauseData` (`Event.Montage.Player.HitPause`). None of the `SkillEffect/Back/PlayerAkane/AkaneHitBack*` or `HitOut` blueprints carry HitPause/Hitlag data. Whether `UltraSkill1` hitstop adds to or is subsumed by `HTGAComponent_AbilityGamePaused` (`CINEMATIC_RELEASE` `4_012_833 us`) must be declared explicitly in `akane/clock/`.
- **Serialized CDO vs raw NameMap extraction note:** In the saved CUE4Parse JSON exports, `GA_Akane_QTE`, `GA_Akane_PerfectEvade`, `GA_Akane_Appear`, and `GA_Akane_UltraSkill_End` expose `EffectContainerMap` directly, whereas `GA_Akane_Melee`, `GA_Akane_Skill`, `GA_Akane_Skill_instead_PlayMontage`, and `GA_Akane_UltraSkill` omit `EffectContainerMap` and most `Damage/*.json` / `Buff/*.json` / `Upgrade/*.json` exports omit explicit `Modifiers` / `DurationMagnitude` structs. Those container and calculation links (including `Melee.3_2` -> `Melee3_1_Damage`, `Proskill_2` `Display.2` -> `UpgradeSkill_Damage`, and `Calculation_Akane_Level3` -> `Skill_Damage`) rest on NameMaps, `DT_SkillDamageData`, `DT_AkaneEffectFigure`, `1057AwakenEffect`, and raw blueprint bytecode rather than parsed CDO `EffectContainerMap` JSON; retain explicit `HypothesisId` provenance where container routing is inferred.
- **Projectile delivery:**
  - `Melee1` / `MeleePre` / `AppearMelee`: `Event.Montage.Player.Melee.1` spawns `BulletActor_AkaneBase` (`InitialSpeed=2000.0, MaxSpeed=3500.0`), whose `ProjectileHitExplodeEffect` delivers Hit 1 (`GE_Player_Akane_Melee1_Damage`, relayed via `DamageRange_Akane_LogicObjTarget`); `Event.Montage.Player.Melee.1_1` delivers direct Hit 2 (`GE_Player_Akane_Melee1_Damage` + `LogicObjTarget`).
  - `Skill` Press (`Overtone`, `skill_1_Montage`): `Skill.3` (`0.342294s`) grants caster mode buff `Buff_Akane057_Skill_short`; `Skill.1_1` (`0.345126s`) spawns `BulletActor_Akane_SoundMark` (`InitialSpeed=4477.1436, MaxSpeed=5049.5483`), whose impact delivers `GE_Player_Akane_Skill_Damage` + `Buff_Akane_Skill_SoundMark`. (Unused `Skill.1` container in `GA_Akane_Skill` is never triggered by `skill_1_Montage`.)
- **Cinematic / Pause release (`CINEMATIC_RELEASE`):** `GA_Akane_UltraSkill` uses `HTGAComponent_AbilityGamePaused` with `TriggerEndEventTags = ["Event.Montage.Player.Common.EndAbilitySkill"]`, matching notify `[4.012833 .. 4.112832]` on `char_Akane_skill_ultraskill_Montage` (`progress_us: 4_012_833`; `LevelSequence` range is `68800 / 24000 = 2.866667s`; `End_ultraskill_Montage` fires `EndAbilitySkill` at `1.547001s`).

| Action / Montage | `SequenceLength` | Damage / Effect Events (verified `EffectContainerMap` / `SpawnActorMaps`) | Cancel Windows `[start .. end]`, Combo (`JumpSection` + `nFrame`), Switch Locks (`BP_CantChangePlayerNS_C`) |
| :--- | ---: | :--- | :--- |
| `Melee1` (`melee_1_Montage`) | `4.666667s` | `0.133639s` (`Melee.1` -> `BulletActor_AkaneBase` Hit 1), `0.465515s` (`Melee.1_1` -> Hit 2) | JumpSection `Melee2` `[0.149882 .. 0.888861]` (`nFrame=24`); Break Jump `0.527595..4.667`, Move `0.756796..4.667`, AnySkill `0.777845..4.667`, EndAbility `0.781518..4.667` (`BlockMovement` `0..0.750349`) |
| `MeleePre` (`melee_Pre_Montage`) | `4.416667s` | `0.117649s` (`Melee.1` -> projectile Hit 1), `0.449141s` (`Melee.1_1` -> Hit 2) | JumpSection `Melee2` `[0.149882 .. 0.888861]` (`nFrame=24`); Break Jump `0.527595`, Move `0.756796`, EndAbility `0.781518`, AnySkill `0.886565` |
| `AppearMelee` (`AppearMelee_Montage`) | `3.183333s` | `0.117649s` (`Melee.1` -> projectile Hit 1), `0.449141s` (`Melee.1_1` -> Hit 2) | JumpSection `Melee2` `[0.149882 .. 0.888861]` (`nFrame=24`); Break Jump `0.527595`, Move `0.756796`, AnySkill `0.777845`, EndAbility `0.781518` |
| `Melee2` (`melee_2_Montage`) | `3.266667s` | `0.183217s`, `0.353334s` (`Melee.2` x2; HitPause `0.02s @ 0.01`) | JumpSection `Melee3` `[0.171792 .. 0.493901]` (`nFrame=14`); Break Jump `0.436721`, EndAbility `0.491588`, AnySkill `0.492799`, Move `0.639644` (`BlockMovement` `0..0.646278`) |
| `Melee3` (`melee_3_Montage`) | `7.016667s` | `0.133022s`, `0.393471s` (`Melee.3`, HitPause `0.02s @ 0.01`), `0.538558s` (`Melee.3_1`); `0.812638s`, `0.835430s`, `0.849551s`, `0.867402s` (`Melee.3_2`, HitPause `0.02s @ 0.1`) | JumpSection `Melee4` `[0.073024 .. 1.036273]` (`nFrame=52`); EndAbility `0.896640`, Break Jump `0.915880`, Move `0.965366`, AnySkill `1.024168` (`BlockMovement` `0..0.970820`) |
| `Melee4` (`melee_4_Montage`) | `3.700000s` | `0.589367s` (`Melee.4` -> `Melee4_Damage` x1) | JumpSection `Melee1` `[0.500000 .. 0.760000]` (`nFrame=12`); Break Jump `0.657890`, AnySkill `0.672824`, EndAbility `0.747497`, Move `1.532618` (`BlockMovement` `0..1.532618`) |
| `AirAttackEnd` (`airattack_end_Montage`) | `3.166667s` | `0.097189s` (`AirAttack.1` -> `Airattack_Damage` x1) | Break Jump/AnySkill `0.216571..3.167`, EndAbility `0.790528`, Move `0.831852` (`airattack_entry`: `1.0s`, `SwitchRole` `0..0.221950`, `BlockMovement` `0..1.0`) |
| `PerfectEvade Atk` (`perfectattack_Montage`) | `3.333333s` | `0.251899s` (`Melee.1` -> `PerfectEvadeAttack_Damage` x1); `0.000s` (`Melee.2` -> `Buff_ExtremeEvade_Property_Long`) | Break Jump/Skill `0.0..3.333`, JumpSection `Melee1` (retaining combo map `1->2->3->4->1`, `Pre/Appear->2`) `[0.166000 .. 0.833000]` (`nFrame=15`, `fDelayTriggerTime=0.01`); Break Move/AnySkill/Jump `0.652000`, EndAbility `0.676566` |
| `QTE` (`qte_Montage`) | `3.600000s` | `0.704643s` (`QTE.1` -> `QTE_Damage` x1) | Switch lock `0.000100..0.875734`; `BlockMovement` `0.000100..1.284477`; `EndAbilitySkill1` `1.138941s`; Break AnySkill/Jump/Move `1.291110..3.600`; `EndAbilitySkill` `1.295967s`; EndAbility `1.424255` |
| `Skill` Press (`skill_1_Montage`) | `6.100000s` | `0.342294s` (`Skill.3` -> `Buff_Akane057_Skill_short`), `0.345126s` (`Skill.1_1` -> `BulletActor_Akane_SoundMark` -> `Skill_Damage` + `Buff_Akane_Skill_SoundMark`) | `TriggerCooldown` `0.276833s` (`16s`); Hold gate `HoldTimeTrigger=0.2s` into `Skillhold`; Break Jump `0.422076`, AnySkill `0.908030`, EndAbility `1.498994`, Move `1.712056` (`BlockMovement` `0.0001..1.719570`) |
| `Skillhold` (`skill_2_Montage`) | `4.750000s` | `0.070836s` (`Skill.2` -> `Skill2_Damage` Hit 1), `0.612793s` (`Skill.5` -> `Buff_Akane057_Skill_Long`), `0.629763s` (`Skill.2_1` -> spawn `SoundField_AkaneSkill`), `0.639976s` (`Skill.4` -> spawn `SoundField_AkaneSkill_Level3`), `0.641018s` (`Skill.4_1` -> `Skill2_Damage` Hit 2) | `TriggerCooldown` `0.643221s` (`16s`); `HoldToMelee_Skillhold` `>1.31s`; Break Jump `1.229058`, AnySkill `1.309354`, EndAbility `1.417949`, Move `1.422213` (`BlockMovement` `0.0001..1.421704`) |
| `Synced Resonance` Overtone (`Proskill_1_Montage`) | `4.200000s` | `0.423777s` (`Skill.1` -> owner A1/A4/R6 buffs), `0.429237s` (`Skill.1_1` -> `UpgradeSkill_Damage` + `Upskill_Short_CoefAdd` x1) | Switch lock `0.000100..0.611700`; `LimitPassiveSkillEffect` `0.000100..0.611700`; `HoldToMelee_Skill_Short` `>0.61s`; Break Jump `0.588414`, AnySkill `0.611652`, Move `0.646508`, EndAbility `1.438672` |
| `Synced Resonance` Reverb (`Proskill_2_Montage`) | `3.833333s` | `0.938194s` (`Skill.1` -> owner A1/A4/R6 buffs), `1.004348s` (`Display.2` -> `UpgradeSkill_Damage` x1 AoE) | Switch lock `0.000100..1.309400`; `LimitPassiveSkillEffect` `0.000100..1.309400`; `HoldToMelee_Skill_Long` `>1.31s`; Break Jump `1.229058`, AnySkill `1.309354`, Move `1.335262`, EndAbility `1.417949` |
| `UltraSkill` Stage 1 (`ultraskill_Montage`) | `6.233333s` | `2.694826s` (`UltraSkill.1` -> visual buff), `2.853346s` (`UltraSkill.4` -> spawn stage FX), 9x `UltraSkill.2` (`2.856699, 2.940793, 3.021996, 3.103198, 3.185962, 3.279656, 3.374913, 3.470168, 3.568547s` + HitPause), `3.804156s` (`UltraSkill.6` -> 10th `UltraSkill1_Damage` + HitOut) | Switch lock `0.000000..4.059677`; `LimitPassiveSkillEffect` `0.000000..4.008655`; `EndAbilitySkill` `4.012833s` (`CINEMATIC_RELEASE`); `HoldToMelee_Default` `>4.05s`; Break AnySkill `4.050620`, Move/Jump `4.068734`, EndAbility `4.115238` |
| `UltraSkill` Stage 2 (`End_ultraskill_Montage`) | `3.333333s` | `0.948097s` (`UltraSkill.2` -> `UltraSkill2_Damage` x1) | Switch lock `0.000000..1.540640`; `LimitPassiveSkillEffect` `0.000000..1.519570`; Break Jump `1.526601`, `EndAbilitySkill` `1.547001s`, EndAbility `1.568251`, Move/AnySkill `1.568717` (`BlockMovement` `0..1.554679`) |
| `EvadeForward` / `EvadeBack` / `PerfectEvade` | `1.483s` / `2.667s` / `2.1s`/`6.0s` | Dash / Perfect Evade windows | `EvadeForward`: Break Jump `0.082544`, AnySkill `0.332778`, EndAbility `0.431326`, JumpSection `MeleePre` `0.086042..0.436425` (`nFrame=14`); `EvadeBack`: Break Jump `0.166629`, AnySkill `0.383426`, EndAbility `0.499987`, Move `0.583301`, JumpSection `MeleePre` `0.083388..0.484589` (`nFrame=16`); `PerfectFront`/`PerfectBack`: JumpSection `Atk` `nFrame=15` (`fDelay=0.033`); `Appear`: SwitchRole `0..0.26`, JumpSection `AppearMelee` `0.049422..0.358162` (`nFrame=12`), EndAbility `0.349956` |

### `DamageTriggerActorDataTable` and `DamageRange` / multi-dummy routing

Six Akane rows exist in `DamageTriggerActorDataTable` (from the saved client expansion):

| `DamageTriggerConfigID` | `DelaySpawnTime` | `DelayTriggerTime` | `Period` | `duration` | `MaxTriggerCount` | `TriggerOnSpawn` | Caller / Role |
| :--- | ---: | ---: | ---: | ---: | ---: | :---: | :--- |
| `BulletActor_AkaneBase` | `0.0` | `0.03` | `0.0` | `1.0` | `1` | `true` | N1 / `MeleePre` / `AppearMelee` projectile |
| `BulletActor_Akane_SoundMark` | `0.0` | `0.03` | `0.0` | `5.0` | `1` | `true` | Press `Rhythm Wave` projectile (`Skill_Damage` + `Buff_Akane_Skill_SoundMark`) |
| `SoundField_AkaneSkill` | `0.0` | `0.03` | `0.0` | `0.0` | `1` | `true` | Hold `Rhythm Wave` field (`GA_Akane_Skill` `HTAbilitySpawnDamageTriggerActor_0`) |
| `SoundField_AkaneSkill_Level3` | `0.0` | `0.03` | `0.0` | `0.0` | `1` | `true` | A3 Hold `Rhythm Wave` upgraded pull field (`HTAbilitySpawnDamageTriggerActor_2`, `HTAbilityFuncExtension` pull `Radius = 1690.0`, `duration = 8.0s`) |
| `DamageActorAkane_Melee4` | `0.0` | `0.03` | `0.0` | `0.0` | `1` | `true` | N4 ground/air impact actor (`GA_Akane_Melee` `Melee4GroundTargetEffects` / `Melee4AirTargetEffects`) |
| `UltraSkillField_AkaneSkill` | `0.0` | `0.03` | `0.0` | `25.0` | `1` | `true` | `GA_Akane_UltraSkill` stage field (`UltraSkillField_AkaneSkill`) |

All 6 rows configure `DelayTriggerTime = 0.03s` (`30_000 us`), `TriggerOnSpawn = true`, `Period = 0.0`, and `MaxTriggerCount = 1` (`DamageActorAkane_ProSkill2` has no separate `DamageTriggerActorDataTable` row and carries empty `TargetGameplayEffectClasses`). Whether `DamageActorAkane_Melee4` shifts N4 hit resolution from `0.589367s` to `0.619367s` (`+30_000 us`), or resolves cleanly at the montage notify under bounded action timing, must be declared in `akane/clock/`.

- **Targeting volumes (`DamageRange_Akane_*` & `LockRange_Akane_*`):**
  - Lock ranges: `Melee` `SphereRadius = 1200` (`1200x1200x300`), `AppearMelee` `900`, `Skill_Instend_PlayMontage` `1200`, `Skill` / `UltraSkill` `2000` (`2000x2000x500`), `SoundMark` `5000`.
  - Hitbox volumes: `Melee1` box `100x100x120` (socket-bound), `Melee1_1` swept box `500x200x150` (`Offset X=500`), `Melee2` sphere `800`, `Melee3` box `600x270x200` (`Offset X=550, Z=100`, `MaxCount=12`), `PerfectAtk` box `800x300x200` (`Offset X=700, Z=100`, `MaxCount=12`), `Airattack` box `200x150x120`, `QTE` box `600x600x300` (`Offset X=600`), `Skill2` sphere `800` / box `800x800x800` (`MaxCount=99`), `Skill2_1` box `500x500x300`, `ProSkill2` sphere `800`, `UltraSkill` sphere `1500` (`MaxCount=20`), `UltraSkill2` box `1000x1200x300` (`Offset X=600`), `ShpereSpawnActor` sphere `1200` (`MaxCount=1`), `InstandSkill` (`MaxCount=0`), `LogicObjTarget` (`{}`).
  - Note `AddTargetType = DamageRange_Akane_LogicObjTarget` is attached alongside primary `TargetType` on `BulletActor_AkaneBase` (Melee1) and `GA_Akane_PerfectEvade` (`Featherblade Reverb`), while `Overtone` vs `Reverb Field` relay predicates use `Con_MMO_Akane_Mark` and `Con_MMO_Akane_Field` respectively.
- **Multi-dummy contract (implementation specification required for `ntesim`):**
  - Direct AoE actions under all-in-range contract: `UltraSkill` Stage 1 (`UltraSkill1_Damage`, up to 20 targets), `UltraSkill` Stage 2 (`UltraSkill2_Damage`), `Rhythm Wave` Hold (`Skill2_Damage` x2), `Synced Resonance` under `Reverb Field` (`UpgradeSkill_Damage`), `Stage Dive` (`Airattack_Damage`), and `QTE` (`QTE_Damage`) hit all living scenario dummies in ascending `target_index` order at each scheduled hit timestamp.
  - `Overtone` vs `Reverb Field` relay on `Stringblade` (`N1..N4`) and `Featherblade Reverb`: under `Reverb Field`, every relayed `Stringblade` / `Featherblade Reverb` stage hits **all** living dummies inside the field exactly **once per hit** (no duplicate self-hit on dummy `0`) with `+60%` / A5 `+90%` `DamageUpGeneralBase`. Under `Overtone`, `DamageRange_Akane_LogicObjTarget` routes to the marked target; `ntesim` must explicitly declare whether single-target basics hit only the marked target (with `+40%` / A5 `+64%` `CoefAdd`) or whether AoE melee geometry (`Melee2` / `Melee3` / `Melee4`) also hits nearby unmarked dummies without receiving `Overtone` `CoefAdd`.

---

## 5. Mechanics, state machine, and confirmed behavioral choices

1. **String Energy (`0..30`, confirmed behavior), `MingYinLogicObj`, & Basic Chain Preservation:**
   - Gains per qualifying attack stage/cast (managed in native C++ `MingYinLogicObj_0`; treat per-stage deduplication across multi-tick/multi-target hits as an explicit `HypothesisId`):
     - `Melee 1`: `+1`, `Melee 2`: `+1`, `Melee 3`: `+2` (`Melee3_1 = 0`), `Melee 4`: `+6` -> **10 String Energy per full N1–N4 chain**.
     - `Stage Dive`: `+1`; `Featherblade Reverb` (Crit Riposte): `+2`; `Altered Chord` (QTE): `+4`.
     - `Rhythm Wave` Press (`Overtone`): `+6`; Hold (`Reverb Field`): `+6`.
     - `Sonic Boom` / `Synced Resonance` / `Perfect Pitch`: `+0`.
   - Capped strictly at `30`. `R3` sets String Energy to `30` on battle entry (once per battle).
   - **Dodge / Riposte chain preservation (`ComboSectionMap`):** Dodging (`EvadeForward` / `EvadeBack`) transitions via `BP_JumpSection_C` into `MeleePre` (which chains to `Melee2`), and `PerfectEvade Atk` (`perfectattack_Montage`) preserves the current Basic Attack chain via `ComboSectionMap` (`Melee1->Melee2->Melee3->Melee4->Melee1`, `MeleePre/AppearMelee->Melee2`).

2. **`Rhythm Wave` Forms (`Overtone` vs `Reverb Field`) & Relay (confirmed behavior):**
   - Both forms last `25.0s` (`SoundMarkDuration` / `SoundFieldDuration`) and are mutually exclusive (`AkaneMode_RemoveOpposite`); establishing a new form replaces the old one. Both share the `16.0s` cooldown.
   - **Relay does NOT add extra hits/ticks:**
     - **Overtone (`Buff_Akane057_Skill_short` + target mark):** Single-target focus on the marked dummy; adds `GE_Player_Akane_*_CoefAdd` (`+40%` of base ratio, or `_CoefAdd_level5` `+64%` with A5) to relayed `Stringblade` (`N1..N4`) and `Featherblade Reverb` hits on that target. (`Stage Dive` receives no CoefAdd.)
     - **Reverb Field (`Buff_Akane057_Skill_Long` + field zone):** Relays all `Stringblade` (`N1..N4`) and `Featherblade Reverb` hits in full AoE across all eligible targets in the field with `+60%` (`Skill_Long_DamageUp`) or `+90%` (`Skill_Long_AtkDamageUp_Level5` with A5) additive `DamageUpGeneralBase`.

3. **`Synced Resonance` (`GA_Akane_Skill_instead`, Press-only):**
   - Available only when `String Energy == 30` AND an active `Overtone` or `Reverb Field` is present. Zero cooldown; does not start or alter `Rhythm Wave`'s 16s cooldown.
   - Under `Overtone`: deals single-target `UpgradeSkill_Damage + Upskill_Short_CoefAdd` (`1125.0%` Lv1 / `2248.89%` Lv10).
   - Under `Reverb Field`: deals AoE `UpgradeSkill_Damage` (`1012.5%` Lv1 / `2024.0%` Lv10) to all enemies in the field.
   - Consumes `30` String Energy **on hit** (0 if missed); preserves the existing `Overtone` or `Reverb Field` without refreshing its remaining 25s timer.
   - **R6 settlement order (critical):** On montage (`Proskill_1`: `Skill.1` at `0.423777s` before `Skill.1_1` at `0.429237s`; `Proskill_2`: `Skill.1` at `0.938194s` before `Display.2` at `1.004348s`), `Skill.1` fires before impact. With `R6`, the `10` String Energy refund **must** be settled after `30` energy is spent on a successful hit (`30 -> 0 -> 10`, not capped as `min(30 + 10, 30) - 30 = 0`), and must not grant `+10` if `Synced Resonance` misses; `R6` also grants `+64%` Crit DMG (`CritDamageBase`) for `25.0s`.

4. **`Sonic Boom` Two-Stage Ultimate, Passives, and Out-of-Combat Energy Regen:**
   - **Stage 1 (`GA_Akane_UltraSkill`):** Costs `100` Ultimate Energy, starts `20.0s` cooldown, delivers 10x `UltraSkill1_Damage`, enters **`Solo Cadenza`** (`20.0s`, `+75%` interruption resistance, independent of `Overtone`/`Reverb Field`), and triggers **`Starlit Anthem`** (`+20%` `AtkUp` for `20.0s`, non-stacking refresh). None of the `Solo Cadenza` buffs (`Buff_Akane_Skill_UltraSkill_Begin`, `Buff_Akane_Skill_UltraSkill_Enhanced`, `Buff_Akane_Show_UltraSkillUI`, `Buff_Akane_Auto_UseEnhancedUltraSkill`) apply `State.Property.ForbidSelfCharge` / `ForbidOthersCharge`, so Akane continues gaining Ultimate Energy from ordinary hits/skills/team energy during `Solo Cadenza` before Stage 2 is cast.
   - **Stage 2 (`GA_Akane_UltraSkill_End`):** Manually castable anytime after `Solo Cadenza` has been active for `>= 4.0s` and before `20.0s` expiration (`0.1s` CD, `0` extra Ultimate Energy). Ends `Solo Cadenza` early, deals 1x `UltraSkill2_Damage`, and triggers/refreshes `Starlit Anthem` (`+20%` `AtkUp` for `20.0s`). **Never auto-casts on expiry:** both the client tooltip (`GA_Akane_UltraSkill_des` / `GA_Akane_UltraSkill_End_des` / `AkaneCadenza_des`) and gameplay guides explicitly state that if Stage 2 is not pressed manually before `Solo Cadenza` ends at `20.0s`, the second strike is lost.
   - **Out-of-combat passive Ultimate Energy regen (no simulator implementation required; combat-only sim):** Stated only in full `GA_Akane_UltraSkill_des`. Innate character buff `Buff_Akane057_Passive3` (referenced directly by `Player_057_Akane`) adds `Buff_Akane057_BattleState` on initialization (`BUFF_EVENT_BEGIN`) and on leaving combat (`BUFF_LEAVE_BATTLE`), and **removes** `Buff_Akane057_BattleState` on entering combat (`BUFF_ENTER_BATTLE`). While `Buff_Akane057_BattleState` is active (out of combat only), its `1.0s` `BUFF_EVENT_PERIODIC` ticks apply `Buff_UltraSBuff_Akane057_AddEnergy` (`ChargeExecution`, `PParameter1 = 3.0`, `BySelf = true`) -> **+3.0 Ultimate Energy/sec to Akane herself (self-only, does not distribute to teammates, and turns off once combat starts)**. Retained here for completeness only — **does not require implementation** in `ntesim`'s combat-only simulation. In combat, Akane receives team Ultimate Energy from allies normally (`0.6 × ChargeAdd × own ChargeEfficiency` via standard `DT_SkillTeamChargeRelationData`).
   - **Passive 1 — `Perfect Pitch` (`GA_Akane_Passive_1`):** After the team triggers `Remora` (`Cosmos + Lakshana`) once in combat, Akane permanently gains Remora Enhancement until battle end: every `3.0s`, `Overtone` applies `Remora` to its marked target and `Reverb Field` applies `Remora` to all targets inside. Whenever any team member reapplies `Remora` to an enemy already carrying active `Remora`, Akane deals `GE_Reaction_3_new_1057_Damage` (`150%` ATK base, Skill DMG, Lakshana), scaled via native `GetXiangGuangReactionEightDamageRatio` up to `+160%` (`0.20` per remaining second) using independent per-target remaining Remora duration and a `1.0s` per-target ICD. Four explicit implementation boundaries must be declared in `remora.rs` / `akane/`:
     - *Remaining-time formula:* choose discrete `floor(remaining_seconds) * 0.20` (matching tooltip `"for each 1s ... remaining"`) vs continuous `min(0.20 * remaining_seconds, 1.60)` — capped at `+160%` (`remaining >= 8.0s`, relevant when paired with Hathor's `12s` Remora extension).
     - *Periodic `3.0s` phase & off-field ticking:* schedule at `t + 3.0s` after enabling condition (vs `t + 0`) and keep active while Akane is off-field as long as she is alive and `Overtone`/`Reverb Field` remains active.
     - *Self-reapply trigger:* specify whether `Overtone` / `Reverb Field`'s own `3.0s` Remora tick onto an enemy that already carries active `Remora` counts as team reapplication triggering `GE_Reaction_3_new_1057_Damage` (subject to the `1.0s` per-target ICD).
     - *Damage binding (`GAName = None`, Category `E`, tags `State.Damage.Skill`):* bind under `GA_Akane_Skill` (per `"counts as Redirect Skill DMG"`) or `GA_Akane_Passive_1` in `action_packet_bindings.rs` so `weapon:fork_prism` (which buffs Lakshana and reacts to native `E` packets) resolves consistently.

5. **Off-field persistence & switch policy (`on_field_entry` / `on_field_exit`):**
   - Persists and continues ticking on World time when Akane switches out: `Overtone` (`25s` target mark), `Reverb Field` (`25s` zone + A2 `-12%` Lakshana RES), `Perfect Pitch` permanent battle enablement (and its `3.0s` Remora ticks while she stays alive), `String Energy` (`0..30`), `Starlit Anthem` (`+20% ATK`, `20s`, + A6 `16%` DEF Ignore), `A1` DEF/Poise (`15s`), `A4` (`+30%` Crit DMG, `15s`), `R6` (`+64%` Crit DMG, `25s`), and `Solo Cadenza`'s `20s` World timer (though Stage 2 can only be pressed when Akane is active on field before expiry).
   - Resets on field exit: `normal_stage` (`N1..N4` combo index) via standard `character.reset_normal_chain()`, and finite active montage replay clocks follow `ReplayClock`.

6. **Awakenings (`1057AwakenEffect`, identity-gated `A1..A6` and count-gated `R3`/`R6`):**
   - **A1 (*After-School Tabs*):** Casting `Synced Resonance` heals Akane for `1000%` Base ATK (`Calculation_Akane_Treat`) and grants `+50%` DEF + `60%` Poise for `15.0s`.
   - **A2 (*Capo On*):** Targets affected by `Overtone` or inside `Reverb Field` suffer `-12%` Lakshana RES (`Player_Akane_DamageRseistBase = -0.12`). (`Upgrade/Level2/Buff_Akane057_Level2_Damageup` is unused.)
   - **A3 (*Shade Capsules*):** Press `Rhythm Wave` (`GE_Player_Akane_Skill_Damage`) gains `+1000%` fixed ATK ratio (`Skill_Short_DamageUp_Level3 = 10.0`); `Reverb Field` radius expands `+30%` (`SoundField_AkaneSkill_Level3`: `Radius = 1690.0` vs base `1300.0`) and pulls enemies (`duration = 8.0s`). (`Upgrade/Level3/Buff_Akane057_Level3_DamageUP` is unused.)
   - **A4 (*Headphone Static*):** Casting `Synced Resonance` grants `+30%` Crit DMG (`CritDamageBase`, `Buff_Akane_Skill_insteadSkill_Level3`) for `15.0s`. (Internal asset name ends in `_Level3` while gated by `Con_Akane_Level4`.)
   - **A5 (*Rehearsal Click*):** Increases `Overtone` relay CoefAdd from `40%` to `64%` (`*_CoefAdd_level5`) and `Reverb Field` relay DMG bonus from `60%` to `90%`.
   - **A6 (*Front-Row Supernova*):** `Starlit Anthem` (`20.0s`, triggered on Stage 1 and Stage 2) additionally grants `16%` DEF Ignore (`Player_Akane_Level6_DefIgnore = 0.16`, asset `Buff_Akane057_Level6_GlableDamageUP`).
   - **R3 (*Diatonic*, `count >= 3`):** `+1` level to `GA_Akane_Melee`, `GA_Akane_Skill`, `GA_Akane_UltraSkill`; fills `30` String Energy on battle entry; adds `+800%` fixed ratio (`Player_Akane_Level3EX_UpSkill_CoefAdd = 8.0`) to `Synced Resonance`; adds `+1000%` fixed ratio (`Player_Akane_Level3EX_Ultra2_CoefAdd = 10.0`) to `Sonic Boom` Stage 2. (`Buff_Akane057_Level3_extra`'s `Player_Akane_Level3EX_DamageUP = 0.0` adds zero `AtkUp`.)
   - **R6 (*Polyphony*, `count >= 6`):** Casting `Synced Resonance` refunds `10` String Energy (after `30` energy is spent on hit) and grants `+64%` Crit DMG for `25.0s` (coexists independently with A4's 15s `+30%` Crit DMG buff).

---

## 6. Known repository touchpoints, modeling choices, and reference 2-cycle oracle

- **Engine touchpoints (`ntesim_data` / `ntesim_content` / `web`):**
  - `crates/data`: `constants/character/definitions.rs`, `native_damage/mod.rs` + `akane.rs`, `character_actions.rs` (remove `:319` exclusion assertion), `ability_titles.rs` (`GA_Akane_Melee: "Stringblade"`, `GA_Akane_QTE: "Altered Chord"`, `GA_Akane_Skill: "Rhythm Wave"`, `GA_Akane_UltraSkill: "Sonic Boom"`; remove `:417` assertion), `ability_tags.rs` (`Ability.Melee` / `Ability.Skill` / `Ability.UltraSkill` from `DT_CharacterAbilityEffectConfig`; remove `"GA_Akane_Skill"` from `:365`), `resonance_levels.rs`, and `native_action_metadata.rs`.
  - `crates/content`: `characters/mod.rs` + `characters/akane/`, `characters/cinematics.rs`, `characters/packet_semantics.rs`, `characters/action_packet_bindings.rs`, `characters/team.rs`, `characters/action_roles.rs` (update `1057` notes now that `DT_CharacterAbilityEffectConfig` rows exist), `simulation/reactions/remora.rs` + `registry/dispatch.rs` (hook `Perfect Pitch` periodic apply & reapply trigger), `characters/replay_clock.rs`, `dsl/catalog.rs` (remove `catalog.resolve("character", "Akane").is_err()`), and `crates/contracts/src/hypotheses.rs`.
  - `web/`: regenerate catalog/names (`web/src/generated/`); `web/src/domain/artwork.ts` already maps `'akane-rin': 'characters_akane-rin.webp'`.
- **Standard `ntesim` modeling choices to declare at implementation time:**
  - **Plunge (`Stage Dive`) height scaling:** takes explicit `PlungeInput.height_bonus` in `0..=1` -> `(1 + height_bonus)` up to `2.0x`, same as other plunge owners.
  - **Projectile flight vs zero-flight & `0.03s` actor trigger delay:** `BulletActor_AkaneBase` (`InitialSpeed=2000`), `BulletActor_Akane_SoundMark` (`InitialSpeed=4477.1436`), and `DamageActorAkane_Melee4` (`DelayTriggerTime=0.03s`) use either modeled zero-flight/notify-time delivery or explicit travel/`+30_000 us` offset.
  - **Primary-target routing under `Overtone`:** dummy target index carrying the single-target mark (`target 0` default or explicit target selector), whereas `Reverb Field` covers all battle dummies without self-duplication.
  - **Out-of-combat regen (`+3/s`, no implementation required):** since `Buff_Akane057_BattleState` is removed on combat entry (`BUFF_ENTER_BATTLE`), `ntesim`'s combat-only simulation does not implement the out-of-combat `+3/s` tick and uses declared `start_energy` directly.

### Reference two continuing cycles walkthrough (solo Lv80 S6, Lv10 talents, A0, single target, zero-energy start)

Verified raw ATK-ratio, Ultimate Energy, Cycle, and String Energy ledger (ignoring external team/set buffs and `Remora` procs unless triggered by an ally; note `100` Energy cap overflow):

- **Cycle 1 (`0.0s – 60.0s`, `Overtone` single-target focus):**
  - **`0.000s – 1.500s`: Press `Rhythm Wave` (`GA_Akane_Skill`).** Inflicts `Overtone` (`25.0s`, until `25.345s`), starts `16.0s` cooldown (`ready @ 16.277s`), deals `7.196` ATK (`+4.200` Energy, `+6.000` Hetero), grants `+6` String Energy (`6/30`).
  - **`1.500s – 8.100s`: Two full `N1–N4` loops (`GA_Akane_Melee`).** Each loop under `Overtone` deals `16.3231` ATK (`+16.700` Energy, `+23.999` Hetero, `+10` String Energy) -> `32.6462` ATK total across 2 loops; Energy `37.600`, String Energy `26/30`.
  - **`8.200s – 9.700s`: `N1` + `N2` + `N3` + `N4` (loop 3).** Deals `+16.3231` ATK, `+16.700` Energy (total `54.300`), and `26 + 10 -> 30/30` String Energy cap reached on `N3` (`26 + 1 + 1 + 2 = 30`).
  - **`11.500s – 13.000s`: `Synced Resonance` (`GA_Akane_Skill_instead`, `Overtone`).** Requires `30` String Energy and active `Overtone`; consumes `30 -> 0/30` on hit (`+10/30` after spending at `R6`), `0s` CD, preserves `Overtone` remaining timer, deals `22.4889` ATK (`+0` Energy).
  - **`16.300s – 17.800s`: Press `Rhythm Wave` (`GA_Akane_Skill`, off CD).** Refreshes `Overtone` (`25.0s`), deals `7.196` ATK (`+4.200` Energy -> `58.500`), String Energy `6/30` (`16/30` at R6).
  - **`18.000s – 27.900s`: Three `N1–N4` loops.** Deals `3 * 16.3231 = 48.9693` ATK, `+50.100` Energy (reaches `100.0` cap with `8.600` overflow), String Energy reaches `30/30` cap.
  - **`28.000s – 29.500s`: `Synced Resonance`.** Consumes `30 -> 0/30`, deals `22.4889` ATK (`+0` Energy).
  - **`30.000s – 34.115s`: `Sonic Boom` Stage 1 (`GA_Akane_UltraSkill`).** Spends `100 -> 0` Energy, starts `20.0s` CD (`ready @ 50.000s`), `20.0s` *Solo Cadenza* (`30.0s..50.0s`), `20.0s` *Starlit Anthem* (`+20% ATK`, `30.0s..50.0s`; `+16%` DEF Ignore at A6), delivers `10x UltraSkill1_Damage = 12.950` ATK.
  - **`34.200s – 35.700s`: Press `Rhythm Wave` (`off CD @ ~32.577s`).** Deals `7.196` ATK (`+4.200` Energy), String Energy `6/30` (`16/30` at R6).
  - **`36.000s – 44.000s`: `Sonic Boom` Stage 2 (`GA_Akane_UltraSkill_End`, cast after `t >= 34.000s` `>= 4s` Cadenza).** Costs `0` Energy, ends *Solo Cadenza*, refreshes *Starlit Anthem* (`20.0s` from Stage 2), deals `18.351` ATK (`+10.0` ratio at R3 -> `28.351` ATK). Paired `N1–N4` fills Energy & String Energy cleanly before `60.0s`.
- **Cycle 2 (`60.0s – 120.0s`, continuing state):**
  - **`60.000s`:** Resumes with `100.0` Energy, `Overtone` / `Rhythm Wave` & `Sonic Boom` cooldowns ready (`20s` ult CD expired at `50.0s`).
  - Repeats `Rhythm Wave` -> `N1–N4` builders -> `Synced Resonance` (`30 -> 0/30`, `22.4889` ATK) -> `Sonic Boom` Stage 1 (`100 -> 0`) + Stage 2 (`>= 4s` later) with all cooldowns, *Starlit Anthem*, and `String Energy` cycling smoothly across the `60s -> 120s` boundary with zero desync. (Under `R3`, Akane opens Cycle 1 at `30/30` String Energy on battle entry, adding an immediate opening `Synced Resonance` after the first `Rhythm Wave`.)
