# Time Bandit — passive

- Native identity: `weapon:fork_Crowbar`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | HPMaxUp: 10.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | HPMaxUp: 12.50% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | HPMaxUp: 15.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | HPMaxUp: 17.50% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | HPMaxUp: 20.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | HPMaxUp: 22.50% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | HPMaxUp: 25.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **HPMaxUp 25.00%**.

## Refinement scaling

**Passive Name**: Picky

> Increases Break Intensity by <lv>{0}</> for <lv>{1}</>s after the wearer casts a Redirect Skill. Effect does not stack. Unlocks Arc: Picky. Arc: Picky — Summons Picky to unlock all nearby lockable objects.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Crowbar_Unbal | buff_Crowbar_CD |
|---|---|---|
| T1 | 90.00 | 50.00 |
| T2 | 108.00 | 50.00 |
| T3 | 126.00 | 50.00 |
| T4 | 144.00 | 50.00 |
| T5 | 162.00 | 50.00 |

## Activation and trigger mechanics

`weapons.crowbar_v1` grants the wearer 90/108/126/144/162 Break Intensity
points for 50s on an admitted Redirect Skill. Recasts inside the half-open
window neither stack nor refresh it; a later cast after expiry starts another
window. Switching keeps the buff and its original deadline. Support, Burst,
Normal and Critical Riposte do not substitute for Redirect. The non-combat
lock-opening Arc has no simulator action.

The [continuing script](../../../../examples/time-bandit-break-cycles.nte)
uses legal A0 Haniel/Time Bandit and Mint builds with explicit anonymous gauge
capacity 1/10 and one-second Broken recovery. Grants at 0/52s survive the
wearer's off-field periods; Skills after native entry at25.896381/76.896381s
cannot refresh them. Entry/continuation Breaks at50.279633/51.426627s and
102.279633/103.426627s capture zero weapon BI after expiry. All ten Break
generations include the living off-field teammate, who never receives the
wearer's bonus. No resources, clocks or pending effects reset between windows.

Independent Rust checks use level80 base3603 and original capacity.1:
`3603*(.1/3)*(1+BI/300)` gives **156.13/163.336/170.542/177.748/184.954**
wearer Break HP at T1..T5, and **120.1** after expiry or for the teammate.
An ordinary +900% damage bonus does not enter this formula. Native source execution and replay compare all refinements through complete reports.
Browser transport parity is checked at the affected integration boundary. Existing owner checks retain exact expiry, failed
admission, group/unit and strict-assumption coverage. Trigger/refresh and
target Break policies remain explicit product models, not native-game proof.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Time Bandit (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `HPMaxUp` contributes +25.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
