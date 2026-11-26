# characters/elyms Specification

## Purpose
Defines observable combat mechanics, Connection Strength states, Skill redirects, `On the Line` / `Remembrance` off-field attacks, Passive 1 (`Charge Enhancement` & `Remora Enhancement`), `Passive 2`, `A1`–`A6` / `R3` / `R6`, world-clock pauses, and DSL queries for Elyms (`character:1090`), grounded in [action-damage.md](../../../../docs/mechanics/characters/elyms/action-damage.md).

## Requirements

### Requirement: Elyms identity and base stats
The simulator SHALL register Elyms (`character:1090`, DSL slug `elyms`, script alias `Elyms`) as an S-Class `Cosmos` character in `WeaponGroup::One` (Solid Arc) starting at Lv1 S0 with `1270` HP, `83` ATK, `75` DEF, `100` `ChargeMax`, `5%` Crit Rate, and `50%` Crit DMG, projecting to `15392` HP, `660` ATK, and `909` DEF at Lv80 S6.

#### Scenario: Compile Elyms Lv1 S0 and Lv80 S6 base stats
- **WHEN** an actor is compiled from `character:1090` at Lv1 S0 and Lv80 S6 without external equipment stats
- **THEN** Lv1 S0 resolves `1270` HP, `83` ATK, `75` DEF, `100` `ChargeMax`, `5%` Crit Rate, and `50%` Crit DMG
- **AND** Lv80 S6 resolves `15392` HP, `660` ATK, and `909` DEF, pairing legally with Solid Arc `weapon:fork_elyms` (`Echoes on Hold`)

#### Scenario: Resolve Elyms in DSL scripts
- **WHEN** a DSL script declares `elyms char ...;` or `Elyms char ...;`
- **THEN** the compiler binds the actor to `character:1090` with element `Cosmos`

### Requirement: Basic chains, secondary attacks, and Support Skill
Elyms's `attack` (`GA_Elyms_Melee`) SHALL execute the 5-stage `Instant Glitch` chain (`22.4%x2`, `31.4%x2`, `66.8%x2 + 33.2%`, `27.6%x3 + 28.4%x2`, `182.9%` @Lv10) at Weak Connection and switch to 3-stage `Crosstalk` (`GA_Elyms_MeleeB`: `24.8% + 27.0%`, `18.6%x2 + 19.0%x2`, `21.2%x3 + 94.0%`) at Medium or Strong Connection. `Interference` hold, `Memory Anchor` (`Aim`), `Falling Dream` plunge, Riposte, and `GA_Elyms_QTE` SHALL emit their frozen Cosmos packets (`DamageTypeEX = Cosmos`).

#### Scenario: Basic chain switches between Weak and Medium/Strong Connection
- **WHEN** Elyms executes repeated `attack` inputs while at Weak Connection vs Medium/Strong Connection
- **THEN** Weak Connection advances through `Instant Glitch` N1–N5 (`Melee1` through `Melee5`)
- **AND** Medium or Strong Connection dispatches `Crosstalk` B1–B3 (`MeleeB1` through `MeleeB3`)

#### Scenario: Secondary, Aim, Plunge, Riposte, and Support packet admission
- **WHEN** Elyms casts held `Interference`, manual `Aim` (`Memory Anchor`), `Falling Dream` plunge, critical `Riposte`, or `GA_Elyms_QTE` (`I'm Here`)
- **THEN** held attack emits `Branch1 x2 + Branch2 + Branch3` (`89.8%` @Lv10, with `19_800 us` HitPause on `Branch3`), Plunge emits `AirAttack_Damage` (`59.2%`), Riposte emits `PerfectEvadeAttack_Damage` (`199.9%`), Support emits `QTE_Damage` (`399.8%` at `747_546 us`, `5.2` Charge, `0` Cycle)
- **AND** `Memory Anchor` (`Aim`) repeats on the `0.146353s` (`146_353 us`) `BP_CanBreakSkill_C` `AnySkill`/`Jump` cancel cadence (`24.0%`, `0.8` Charge, `1.3` Cycle)

### Requirement: Connection Strength states and Optical Illusion stacking
Combat SHALL initialize Elyms at `Weak` Connection (`0`) with `Buff_Elyms_PowerE1Dur` (`12.0s`) active. `Intermittent Existence` (`Skill2`) and `I'm Here` (`QTE`) advance `Weak` to `Medium` (`1`) (or directly to `Strong` (`2`) with `R3`), clearing `PowerE1Dur`. `Crosstalk` stage 3 (`0.617396s`) grants `PowerE2Dur` (`6.0s`), allowing `Goodbye to You` (`Skill3`) to advance `Medium` to `Strong`. Entering `Medium` from `Weak` grants `+1` stack of `Optical Illusion` (`15.0s`, base cap `1`).

#### Scenario: Combat starts with Intermittent Existence immediately available
- **WHEN** combat begins at `t = 0` with Elyms at Weak Connection
- **THEN** `PowerE1Dur` is active for `12.0s` without waiting for `Skill1`'s cooldown
- **AND** casting `Skill2` or `QTE` transitions `Weak -> Medium`, removes `PowerE1Dur`, and grants `1` stack of `Optical Illusion`

#### Scenario: Crosstalk finisher unlocks Goodbye to You
- **WHEN** Elyms is at Medium Connection and executes `Crosstalk` B1 -> B2 -> B3
- **THEN** at the B3 `AddE` timing (`0.617396s`), `PowerE2Dur` becomes active for `6.0s`, enabling `Skill3` on the next `skill` input

### Requirement: Skill redirect and selective cooldown bypass
`skill` (`GA_Elyms_Skill`) SHALL cast `Intermittent Existence` (`Skill2`: `20.4%x5 + 97.6%`, `8.3` Charge) when `PowerE1Dur` is active at `Weak`; `Goodbye to You` (`Skill3`: `26.2%x5 + 124.3%`, `8.3` Charge) when `PowerE2Dur` is active at `>= Medium`; and `The Ones Who Left` (`Skill1`: `53.8% + 47.2%x4 + 97.6%`, `5.1` Charge, `8.399` packet Cycle plus `+100` Cycle refill via `Buff_Elyms_QRecoverEnergy`) otherwise. Only `Skill1` SHALL start or respect the `16.0s` Skill cooldown.

#### Scenario: Intermittent Existence bypasses and preserves Skill cooldown
- **WHEN** Elyms casts `Skill2` (`PowerE1Dur` active) at `t = 0`, or casts `Skill3` (`PowerE2Dur` active) while `Skill1` is on its `16.0s` cooldown
- **THEN** neither `Skill2` nor `Skill3` waits on or starts the `16.0s` cooldown
- **AND** `Skill1` places `GA_Elyms_Skill` on `16.0s` cooldown, refills `+100` Cycle immediately via `Buff_Elyms_QRecoverEnergy`, and grants `PowerE1Dur` (`12.0s`) at `0.818769s` only if Elyms is still at `Weak` Connection

### Requirement: Optical Illusion consumption on Goodbye to You
Casting `Goodbye to You` (`Skill3`) SHALL consume all active `Optical Illusion` stacks, increasing `Skill3` DMG Ratio by `+100%` (`CoefModify = 1.0`) per consumed stack and storing the total consumed stack count in `TotalPowerE2ConsumedStack` (capped at `4`) until the next `Do You Get Deja Vu` cast.

#### Scenario: Consume stacked Optical Illusion on Skill3
- **WHEN** Elyms holds `k` active `Optical Illusion` stacks and casts `Goodbye to You` (`Skill3`)
- **THEN** `Skill3` resolves with `CoefModify = k * 1.0` (`+100%` per stack), active `Optical Illusion` stacks clear to `0`, and `TotalPowerE2ConsumedStack` increments by `k` (capped at `4`)

### Requirement: Non-Ultimate Skill2/Skill3 and Ultimate camera pauses
Elyms's `Skill2` and `Skill3` SHALL admit `ModePauseKind::Cinematic` on relative world time for `1_719_655 us` (`EndAbilitySkill`), while `Skill1` SHALL incur no camera pause. Both `GA_Elyms_UltraSkill` and `GA_Elyms_UltraSkillB` SHALL admit `ModePauseKind::Cinematic` for `5_499_266 us`.

#### Scenario: Skill2/Skill3 and UltraSkill pause world timers while Skill1 does not
- **WHEN** Elyms casts `Skill1`, `Skill2`, `Skill3`, and `Do You Get Deja Vu` (`UltraSkill` or `UltraSkillB`)
- **THEN** `Skill1` advances relative and absolute clocks equally with zero `ModePauseKind::Cinematic` pause
- **AND** `Skill2` and `Skill3` each pause world/buff timers for `1_719_655 us`, while `UltraSkill`/`UltraSkillB` pause for `5_499_266 us`

### Requirement: Do You Get Deja Vu targeting and scaling
`burst` (`100` Energy cost, `20.0s` cooldown) SHALL snapshot pre-cast Connection Strength (`+0%` Weak, `+50%` Medium, `+100%` Strong `CoefModify`), select ST `GA_Elyms_UltraSkillB` when living targets equal `1` and Connection is `>= Medium` or AoE `GA_Elyms_UltraSkill` otherwise, and reset Connection to `Weak`.

#### Scenario: AoE vs ST Ultimate variant selection and Connection scaling
- **WHEN** Elyms casts `burst` at Strong Connection against `1` living dummy vs `2` living dummies
- **THEN** `1` dummy dispatches `GA_Elyms_UltraSkillB` (`71.2%x3 + 139.5%x4 + 127.9%` @Lv10) and `2` dummies dispatches `GA_Elyms_UltraSkill` (`64.0%x3 + 617.7%` @Lv10), both scaled by `+100%` `CoefModify`
- **AND** Connection resets to `Weak` upon casting

### Requirement: Do You Get Deja Vu team AtkBase and Follow-up buff
Casting `Do You Get Deja Vu` SHALL grant all living allies, including Elyms, flat ATK equal to `20%` of Elyms's `AtkBase` (character plus equipped weapon base ATK) plus `+10%` Follow-up Attack DMG for `20.0s`.

#### Scenario: Whole-team AtkBase ATK and Follow-up DMG buff lifetime
- **WHEN** Elyms casts `Do You Get Deja Vu` with Lv80 S6 base ATK and equipped `weapon:fork_elyms`
- **THEN** every living ally including Elyms gains `0.20 * AtkBase` flat `AtkAdd` (excluding console ATK and ATK% buffs) and `+10%` Follow-up Attack DMG for `20.0s` on relative world time

### Requirement: On the Line and Remembrance off-field coordinated attacks
Casting `Do You Get Deja Vu` SHALL arm `On the Line` (`6` triggers, `3.0s` interval) bound to that cast's AoE or ST mode and Connection snapshot. Trigger `1` SHALL fire immediately upon switching Elyms off-field, repeating every `3.0s` while off-field up to `6` triggers total and pausing while active.

#### Scenario: Immediate off-field fire, pause while on-field, and packet attribution
- **WHEN** Elyms casts `burst`, switches off-field at `t = T`, returns on-field after `1` trigger, and leaves the field again
- **THEN** trigger `1` fires immediately at `T`, repeating triggers `2..=6` at `3.0s` off-field intervals (`0.254680s` AoE / `0.326896s` ST first hit, `4` hits at `0.12s` spacing) without ticking while Elyms is active
- **AND** AoE (`XieTongSkillA1`, `54.0%x4`) and ST (`XieTongSkillB1`, `60.0%x4`) hits carry `State.Damage.Followup`, grant `0.425` Charge + `0.7` Cycle per hit, satisfy `weapon:fork_elyms`, and keep the Ultimate Connection snapshot

### Requirement: Relay Node Passive 1 — Charge Enhancement
While Elyms is in the team and alive (`hp > 0`), whenever the currently active character receives a positive Ultimate Energy grant during combat (from `ChargeAdd` packet hits, `Charge` reaction, or kit/weapon grants), the whole living team SHALL gain `1` stack of `Charge Enhancement` (`+3.75%` Cosmos DMG per stack, up to `8` stacks = `+30%`) for `30.0s`, resetting duration on each trigger.

#### Scenario: Active-actor energy gains build and refresh team Cosmos DMG stacks
- **WHEN** Elyms is on the team and alive while the active character triggers positive Ultimate Energy grants across hits and reactions
- **THEN** each grant event adds `1` stack (capped at `8`, max `+30%` Cosmos DMG) and refreshes the `30.0s` buff deadline across all living teammates
- **AND** initial combat start energy does not pre-stack the buff, and off-field-only energy grants to inactive members do not trigger it when the active character's energy does not increase

### Requirement: Relay Node Passive 1 — Remora Enhancement
While Elyms is alive, whenever any ally applies `Remora`, she SHALL schedule a `Remora` reapplication to that same target `7.5s` later on `ClockDomain::World` (`ElymsPas1Time`), subject to a `7.0s` global ICD across targets (`ElymsPas1CD`). `Remora` applied by this passive SHALL NOT re-trigger `Remora Enhancement`, and a pending trigger SHALL skip if Elyms or the target dies beforehand.

#### Scenario: Delayed Remora cooperates with Akane Perfect Pitch without infinite recursion
- **WHEN** an ally triggers `Remora` at `t = 10s` with Akane Rin in the party outside ICD
- **THEN** Elyms triggers `Remora` on the same target at `t + 7.5s` on relative world time (honoring Akane `Perfect Pitch` if off Akane's `1.0s` ICD)
- **AND** that delayed `Remora` does not schedule another `7.5s` trigger, and any second team `Remora` within `7.0s` does not reset or duplicate the global ICD

### Requirement: Pity System, Awakenings A1–A6, and Resonances R3 and R6
Elyms's `Passive 2` SHALL grant her unconditional `+10%` Crit DMG. Individual awakenings `A1`–`A6` and resonance thresholds `R3` / `R6` SHALL activate strictly when selected or reached by awakening count, with R3 enabling direct `Weak -> Strong` transition via `QTE` or `Skill2` and immediate `Skill3` unlock.

#### Scenario: Individual awakenings and R3/R6 rotation acceleration
- **WHEN** Elyms is tested with A1 (`+10% * consumed` team Crit DMG for `20s` on Q), A2 (`+30%` Q `CoefModify`, `5% * consumed` Skill3 DEF Ignore), A3 (`+3` Optical Illusion on `Medium -> Strong`, max `4`), A4 (`+20%` team Crit DMG while alive), A5 (`+100%` Skill1 `CoefModify` + `100` combat-start Cycle once per `30s`/battle), A6 (`+25% * consumed` Elyms Follow-up DMG for `20s` on Q), R3 (`+1` skill levels and `Weak -> Strong` jump on `QTE`/`Skill2`), and R6 (`12%` team All-RES Ignore + `Skill3Extra` `335.8%/362.7% * consumed` @Lv10/11)
- **THEN** each effect applies only under its native gate, A1/A6 clear `TotalPowerE2ConsumedStack` on Q cast while granting Q the A1 bonus, and R3 lets `Skill2 -> Skill3 -> Q` build `4` stacks with A3 without needing `Crosstalk`

### Requirement: Elyms DSL queries
Authored scripts SHALL support read-only Elyms state queries for Connection Strength tier, `Optical Illusion` stack count, `TotalPowerE2ConsumedStack` count, `PowerE1Dur` (`Skill2`) and `PowerE2Dur` (`Skill3`) readiness, and `On the Line` remaining trigger count without mutating simulation state.

#### Scenario: Observe Elyms resources mid-rotation
- **WHEN** a DSL script inspects `.elyms.connection`, `.elyms.optical_illusion`, `.elyms.power_e1.active`, `.elyms.power_e2.active`, or `.elyms.on_the_line`
- **THEN** each read returns the exact live runtime count or active status with zero RNG or state side effects
