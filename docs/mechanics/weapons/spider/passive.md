# Failing You, Heavy in My Heart — passive

- Native identity: `weapon:fork_spider`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | UnbalIntensityBase: 48.0 |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | UnbalIntensityBase: 60.0 |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | UnbalIntensityBase: 72.0 |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | UnbalIntensityBase: 84.0 |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | UnbalIntensityBase: 96.0 |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | UnbalIntensityBase: 108.0 |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | UnbalIntensityBase: 120.0 |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **UnbalIntensityBase 12000.00%**.

## Refinement scaling

**Passive Name**: Spider Knowledge Collector

> Grants the wearer <lv>1</> stack of Spider Knowledge when dealing damage with Basic Attacks, up to <lv>8</> stacks (up to <lv>1</> stack every <lv>0.5</>s). All stacks are consumed when casting Ultimate. Increases the entire team's ATK by <lv>{1}</> for <lv>{0}</>s <lv>per</> stack consumed. Increases ATK by an additional <lv>{2}</> when <lv>8</> stacks are consumed.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_spider_CD | buff_spider_AtkUp | buff_spider_AtkUp2 |
|---|---|---|---|
| T1 | 10.00 | 1.00% | 2.00% |
| T2 | 10.00 | 1.15% | 2.30% |
| T3 | 10.00 | 1.30% | 2.60% |
| T4 | 10.00 | 1.45% | 2.90% |
| T5 | 10.00 | 1.60% | 3.20% |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `UnbalIntensityBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Failing You, Heavy in My Heart (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `UnbalIntensityBase` contributes +12000.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

Continuing native cycles (`examples/spider-knowledge-cycles.nte`, `spider_cycles.rs`): the native lvl80 Sakiri wearer (16483 HP, the group-Four owner) starts with 100 funded energy and runs two consuming cycles with a Mint peer (12120 HP). Each 0.8s chain stage's primary packet grants one Spider Knowledge stack while its same-instant siblings stay ICD-rejected without trace, so ten chain attacks produce exactly ten `spider_knowledge_stack` receipts counting 1/8 through 8/8; further capped attempts repeat the 8/8 detail. The Burst at 4s consumes five stacks for a +5.00% Team ATK window (per-tier 5 x AtkUp: 5.00/5.75/6.50/7.25/8.00 percent) that lifts the wearer's Ultimate packets, her switch-in Support packet (2100 at T1) and the whole peer chain; the peer's second chain sits between windows at bare coefficients, and the returning hit at exactly 14s lands on the expiry boundary at bare 403 while it reopens the stack count. The rebuild reaches the eight-stack cap, and the second funded Burst at 32.5s consumes 8/8 for the AtkUp2-augmented +10.00/11.50/13.00/14.50/16.00 percent window whose packets hit 1320/3300 at T1, with the third peer chain riding it. An independent ChargeAdd oracle (owner x1, peers x3/5) settles 131+247/625 wearer and 100+572/625 peer Ultimate energy; the two Ultimates spend exactly 200, the final state keeps no Spider Knowledge and still holds the second window, and every tier replays identically from its serialized request.
