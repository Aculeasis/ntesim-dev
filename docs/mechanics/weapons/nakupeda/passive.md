# Nacupeda — passive

- Native identity: `weapon:fork_Nakupeda`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | DefUp: 15.40% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | DefUp: 19.25% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | DefUp: 23.10% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | DefUp: 26.95% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | DefUp: 30.80% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | DefUp: 34.65% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | DefUp: 38.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **DefUp 38.50%**.

## Refinement scaling

**Passive Name**: Nacupeda

> Increases HP by <lv>{0}</>. Grants one of the following random effects when the wearer casts an Ultimate: 1. Recovers <lv>{1}</> of Max HP for the team member with the lowest HP percentage. 2. Grants a Shield equal to <lv>{2}</> of the wearer's Max HP for <lv>{3}</>s. 3. Recovers <lv>{4}</> HP for all team members. Triggers once every <lv>{5}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Nakipeda_Hp | buff_Nakipeda_effect1 | buff_Nakipeda_effect2 | buff_Nakipeda_effect2CD | buff_Nakipeda_effect3 | buff_Nakipeda_CD |
|---|---|---|---|---|---|---|
| T1 | 24.00% | 20.00% | 20.00% | 15.00 | 10.00% | 30.00 |
| T2 | 28.00% | 25.00% | 25.00% | 15.00 | 12.50% | 30.00 |
| T3 | 32.00% | 30.00% | 30.00% | 15.00 | 15.00% | 30.00 |
| T4 | 36.00% | 35.00% | 35.00% | 15.00 | 17.50% | 30.00 |
| T5 | 40.00% | 40.00% | 40.00% | 15.00 | 20.00% | 30.00 |

## Activation and trigger mechanics

- **Source:** the passive rolls only when the wearer casts an Ultimate (input role Ultimate, admitted by the ordinary engine path). Failed admissions, ordinary hits and off-field time never draw.
- **Modeled policy:** `weapons.nakupeda_v1` draws one of the three effects uniformly from a per-wearer seeded stream (`weapon:nakupeda:{actor}`), then applies the 30-second ICD from the successful cast. Effect1 heals the living team member with the lowest exact HP ratio (first tied slot) by that member's Max HP; effect2 shields the wearer by the wearer's live Max HP for15s; effect3 heals every living member by their own live Max HP. Healing separates outgoing/incoming zones, caps overheal and never revives; shields sum static and live granter bonuses before one rounding. Owner defeat after the draw cancels settlement but retains the audited selection and ICD.
- **Equal weights** are the symmetric three-outcome prior; native weights, animation windows and any native retargeting stay unverified model limits.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `DefUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Nacupeda (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `DefUp` contributes +38.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

### Executable checks

`crates/content/src/simulation/tests/nakupeda.rs` covers the frozen T1–T5 projection (HP 24..40%, heal/shield 20..40%, team heal 10..20%, 15s shield, 30s ICD, name-bound curves and unit drift), all three seeded outcomes on an ordinary Ultimate with exact RNG audit, ICD admission at the exact deadline, per-wearer independent streams, owner-defeat cancellation after the audited draw, failed admissions paying nothing, live Max HP for shields/healing, healing-zone separation with overheal caps and no revival, shield switch persistence/overkill conservation, static-plus-live shield-granter bonuses summed before rounding, and shield expiry at the exact deadline.

Continuing native cycles (`examples/nakupeda-support-cycles.nte`, `nakupeda_cycles.rs`): an A0 Haniel wearer at 50% HP and a 10%-HP Mint peer run two native Ultimates across the 30-second ICD, funded by continuing chain energy through a `wait_for_energy(120)` gate after the field switch. Seed42 rolls the lowest-HP heal (peer +2424 = 12120*1/5 at T1) then the wearer shield (16983*1/5 = 3397, expiring15s later before the horizon) from one continuing draw stream; every tier keeps the exact refinement ratios against the tier-scaled wearer Max HP (16983..19174), spends exactly240 energy across both casts without pool resets, and replays identically through native source execution. Burst commitment, its delayed native packet and Ace recovery remain separate. Browser transport parity is checked at the affected integration boundary. Seeds0–11 roll all three outcomes in the same continuing schedule against the independent team-vital oracle (peer/wearer heals summed per outcome; the shield roll changes no HP).
