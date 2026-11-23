# Thief's Candy — passive

- Native identity: `weapon:fork_ThiefCandy`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 11.00% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | AtkUp: 13.75% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | AtkUp: 16.50% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | AtkUp: 19.25% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | AtkUp: 22.00% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | AtkUp: 24.75% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | AtkUp: 27.50% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **AtkUp 27.50%**.

## Refinement scaling

**Passive Name**: Thief's Candy

> Increases wearer's damage by <lv>{0}</> for <lv>{1}</>s after performing a Critical Dodge, up to <lv>3</> stacks. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_ThiefCandy_Up | buff_ThiefCandy_CD |
|---|---|---|
| T1 | 8.00% | 10.00 |
| T2 | 9.60% | 10.00 |
| T3 | 11.20% | 10.00 |
| T4 | 12.80% | 10.00 |
| T5 | 14.40% | 10.00 |

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
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (512.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Thief's Candy (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

Continuing native cycles (`examples/thief-candy-dodge-cycles.nte`, `thief_candy_cycles.rs`): the native lvl80 Sakiri wearer (16483 HP, the group-compatible owner) runs seven assumed-success Critical Dodges and one plain dash with a Mint peer (12120 HP). The three opening `dash[critical=true]` inputs at 0/0.5/1.0s cap the stacks at three, and both chain visits (ten packets each at 1.5-4.7s and 6.5-9.7s) ride the resulting M_bonus zone — every tier settles all 51 packets against exact native ATK-1000 coefficients (403/138/757/16/716/331x3/35/1062) times 1 + 3 x 8/9.6/11.2/12.8/14.40 percent. The plain dash at 5.5s records no weapon hypothesis and procs nothing: the retriggering Critical Dodge at 6s resets the duration to 16s without adding a fourth stack, which the returning hits pin from both sides — the swap-in Support packet (14.05s) and the 15.6s chain stage stay boosted inside (15.5, 16] while the 16.4s stage lands bare past the expiry, excluding the plain-dash window that would have ended at 15.5s. Three fresh Critical Dodges at 17.2/17.7/18.2s reopen the window, the second chain rides it, and the 28.2s attack lands exactly on the second expiry boundary at bare 403 with no buff instance surviving to the final state. Dash inputs deal no damage and grant no energy; an independent ChargeAdd oracle (owner x1, peers x3/5) settles 96+387/5000 wearer and 69+2989/5000 peer Ultimate energy with zero spent, and every tier replays identically from its serialized request.
