# Commander Tiger Talisman — passive

- Native identity: `weapon:fork_TigerTally`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | CritBase: 9.60% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | CritBase: 12.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | CritBase: 14.40% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | CritBase: 16.80% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | CritBase: 19.20% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | CritBase: 21.60% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | CritBase: 24.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **CritBase 24.00%**.

## Refinement scaling

**Passive Name**: Commander Tiger Talisman

> Increases ATK by <lv>{0}</>. Increases Basic Attack and Critical Riposte damage by <lv>{1}</> for <lv>{2}</>s when casting a Redirect Skill or Ultimate, up to <lv>2</> stacks. Grants a Left Tiger Talisman when casting Redirect Skill and a Right Tiger Talisman when casting Ultimate. Unlocks Commander Tiger Talisman if the wearer obtains the other type of talisman within <lv>{3}</>s of gaining one. Arc: Commander Tiger Talisman — Increases the wearer's damage to Bosses by <lv>{4}</> for <lv>{5}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_TigerTally_AtkUp | buff_TigerTally_NormalUp | buff_TigerTally_CD | buff_TigerTally_CD4 | buff_TigerTally_Qup | buff_TigerTally_CD3 |
|---|---|---|---|---|---|---|
| T1 | 15.00% | 15.00% | 15.00 | 15.00 | 10.00% | 10.00 |
| T2 | 18.75% | 18.75% | 15.00 | 15.00 | 12.50% | 10.00 |
| T3 | 22.50% | 22.50% | 15.00 | 15.00 | 15.00% | 10.00 |
| T4 | 26.25% | 26.25% | 15.00 | 15.00 | 17.50% | 10.00 |
| T5 | 30.00% | 30.00% | 15.00 | 15.00 | 20.00% | 10.00 |

## Activation and trigger mechanics

- **Field State & Trigger Conditions**:
  - Evaluated based on the wearer's active vs off-field state and damage delivery.
  - Parameters specify explicit cooldowns and thresholds.
- **Stacking and Refresh Behavior**:
  - Stacks are bounded by the explicit limits declared in the passive description.
  - Reapplication adheres to the identified ICD and expiration/reset rules.
- **Reset / State Transitions**:
  - Switching characters or transitioning field state triggers resets where explicitly stated in the skill description.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Commander Tiger Talisman (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable Boss eligibility

The Arc modifier reads the delivered packet's selected target `is_boss` flag,
supplied by the existing DSL `target boss=true|false` scenario field. A target
status named `ntesim.status.boss` does not change this classification. Omission
uses the runner's false default; it does not establish a native non-Boss fact.
This shares Justice Executioner's explicit scenario classification and removes
the earlier synthetic-tag requirement without adding a gameplay trigger.

Under the existing `weapons.tiger_tally_v2` timing model, Skill/Ultimate pairs
activate the ten-second Arc automatically and its bonus joins the same additive
damage zone as the two Normal stacks. With an independent 25% damage bonus,
the Boss/non-Boss Normal damage ratios at T1–T5 are respectively `33/31`,
`14/13`, `37/34`, `78/71`, and `41/37`. For example, T1 is
`(1.25 + 0.30 + 0.10) / (1.25 + 0.30) = 33/31`, rather than a separate
`1.10` multiplier. After Arc expiry the ratio is 1 even if Normal stacks remain.
Rust regressions check all five refinements through real equipped Hathor
Skill/Burst/Normal admissions, per-target classification, a misleading legacy
status, refresh and exact expiry. Native owner Riposte checks remain separate;
the automatic Arc activation and talisman consumption retain their disclosed
model limits. No roster baseline or native Boss classification is certified.

## Delivered packet categories and continuing cycles

The Basic Attack/Riposte damage zone reads the delivered packet's native source
category: A and Z qualify. The initiating input role continues to own talisman
grants, but cannot classify a child packet. In particular, Hathor's native Branch
packet is A and qualifies; her AirAttack packet is X and receives no
Normal-stack bonus even though both share `GA_Hathor_Melee`. Missing or unknown
classifications diagnose while the restricted bonus is reached. This follows the
frozen packet rows and the same category boundary used by Oraora! and Us.; it
does not create natural plunge or Riposte admission.

The [continuing script](../../../../examples/tiger-tally-cycles.nte) opens two
Skill/Ultimate pairs across two100s periods, with ordinary chains, a Mint field
visit and real connected-packet energy to fund the second100-energy Ultimate.
Explicit zero-hold taps select the native clock/Power/hybrid Burst dependencies;
the live executor exports actual input times rather than nominal intervals.
All five refinements run against declared Boss and non-Boss dummies. Independent
Rust checks isolate both timed damage zones against a control retaining the
same permanent ATK, Power snapshot and native clocks. Every wearer packet checks
the independent additive formula over a static25% damage bonus. Power2 then4
snapshots supply30% then60% of base ATK; at T1/isolated base/static1000, the first
Boss Normal is `243 * 1.45 * (1.25 + .30 + .10) = 581.3775`.

Arc's exact expiry is ten seconds after each admitted Burst; each Normal stack
expires independently fifteen seconds after its own Skill/Burst input. Authored
waits place first-Normal contacts exactly on all six boundaries using the native
216746us contact coordinate. This tests expiry before damage without overriding
packet or recovery timing. Peer damage remains unchanged. Native source execution,
JSON replay and all-tier Boss ratios retain the continuing state. The native
Branch/AirAttack category check uses the held Normal/landing and declared plunge
profiles, preserving their real gate, cost and recovery. Remaining public
controls/cancellation stay Q4-L3; this does not claim a complete source
kit or observed native Arc consumer evidence.
