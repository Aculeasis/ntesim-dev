# Castle — passive

- Native identity: `weapon:fork_Castle`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 25.0 | Stage 0 | Max Lv 20 | +0.0 | HPMaxUp: 15.00% |
| Lv 20 | 88.0 | Stage 1 | Max Lv 30 | +18.0 | HPMaxUp: 18.75% |
| Lv 40 | 154.0 | Stage 2 | Max Lv 40 | +36.0 | HPMaxUp: 22.50% |
| Lv 60 | 220.0 | Stage 3 | Max Lv 50 | +54.0 | HPMaxUp: 26.25% |
| Lv 80 | 286.0 | Stage 4 | Max Lv 60 | +72.0 | HPMaxUp: 30.00% |
| Lv 80 | 286.0 | Stage 5 | Max Lv 70 | +90.0 | HPMaxUp: 33.75% |
| Lv 80 | 286.0 | Stage 6 | Max Lv 80 | +109.0 | HPMaxUp: 37.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **395.0** (286.0 + 109.0); Secondary Stat = **HPMaxUp 37.50%**.

## Refinement scaling

**Passive Name**: Cardboard Castle

> Increases Healing Bonus by <lv>{0}</> for <lv>{1}</>s after the wearer casts a Redirect Skill. Effect does not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Castle_HealUp | buff_Castle_CD |
|---|---|---|
| T1 | 12.00% | 10.00 |
| T2 | 14.00% | 10.00 |
| T3 | 16.00% | 10.00 |
| T4 | 18.00% | 10.00 |
| T5 | 20.00% | 10.00 |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (395.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Castle (Base ATK 395.0):
    - Base ATK = 600 + 395.0 = 995.0.
    - Secondary Stat `HPMaxUp` contributes +37.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

## Executable integration boundary

`weapons.call_of_the_twisted_city_v1` grants one wearer-owned outgoing healing
bonus after admitted Redirect Skill; it refreshes rather than stacks, survives
switching and expires after ten World seconds. It is read from Edgar at each
Wake tick even when the active recipient is Mint. It does not grant a recipient
healing bonus, extend Wake or modify its one-second cadence.

The [A0 continuing source](../../../../examples/castle-healing-cycles.nte) uses
two natural Skill/Burst cycles, refilling the second Burst with thirty-five ordinary
inputs. Each Skill grants one new Key and Burst consumes it at field spawn,
so both Wakes produce11 ticks without resetting
energy, cooldowns or owner state. The second field continues after switching.
Independent Rust cases fix source MaxHP at20000: `.011 * 20000 + 350 = 570`
base healing becomes638/650/661/673/684 at T1–T5, rounded once after the
outgoing bonus. Five ticks in each cycle receive Castle; subsequent ticks
return to570 while Wake remains alive. The second tap independently selects
Mint's lower absolute HP: `.014 * 20000 + 300 = 580` before Castle. Mint then
receives three buffed and six unbuffed field ticks, ending at
`1000 + buffed_skill + 3 * buffed_field + 6 * 570` HP in the controlled oracle.
Complete native/WASM/actual Worker reports,
aggregation and replay use the same legal equipped source at every rank.

The source/native tests use Edgar's selected finite owner clock, including
the2.750956s field spawn marker and explicit extra waits;
it neither fabricates a healing event nor supplies a caller-selected multiplier.
