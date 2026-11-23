# Eternal Waltz — passive

- Native identity: `weapon:fork_Arachne`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 28.0 | Stage 0 | Max Lv 20 | +0.0 | HPMaxUp: 16.50% |
| Lv 20 | 96.0 | Stage 1 | Max Lv 30 | +18.0 | HPMaxUp: 20.62% |
| Lv 40 | 168.0 | Stage 2 | Max Lv 40 | +36.0 | HPMaxUp: 24.75% |
| Lv 60 | 240.0 | Stage 3 | Max Lv 50 | +55.0 | HPMaxUp: 28.87% |
| Lv 80 | 312.0 | Stage 4 | Max Lv 60 | +74.0 | HPMaxUp: 33.00% |
| Lv 80 | 312.0 | Stage 5 | Max Lv 70 | +93.0 | HPMaxUp: 37.12% |
| Lv 80 | 312.0 | Stage 6 | Max Lv 80 | +112.0 | HPMaxUp: 41.25% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **424.0** (312.0 + 112.0); Secondary Stat = **HPMaxUp 41.25%**.

## Refinement scaling

**Passive Name**: Arachne

> Increases Max HP by <lv>{0}</>. Increases Mental DMG dealt by <lv>{1}</> for <lv>{2}</>s after the wearer casts an Ultimate.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Arachne_Hp | buff_Arachne_Up | buff_Arachne_CD |
|---|---|---|---|
| T1 | 20.00% | 10.00% | 10.00 |
| T2 | 23.00% | 12.00% | 10.00 |
| T3 | 26.00% | 14.00% | 10.00 |
| T4 | 29.00% | 16.00% | 10.00 |
| T5 | 32.00% | 18.00% | 10.00 |

## Activation and trigger mechanics

`weapons.eternal_waltz_v1` adds the permanent MaxHP percentage during build
compilation, once against character base HP. An admitted Ultimate grants one
wearer-only Mental damage bonus for 10s; recasting refreshes the same instance.
Switching preserves its deadline. Expiry precedes equal-time hits. Failed
admission grants nothing. Mental is distinct from Psyche; Nova's separate
reaction policy does not inherit this ordinary Mental damage bonus.

The [continuing script](../../../../examples/eternal-waltz-mental-cycles.nte)
funds two paid A0 Fadia initial Bursts from zero energy through real Normal
chains and switches to Mint between cycles. Rust checks all refinements with
static base/final HP10000 and additional general bonus25%. Fadia's native B4
membership adds10% of base HP once, so live MaxHP is11000. Level8 initial Burst
coefficients .067 + 4*.021 + .108 sum to .259, giving
**3846.15/3903.13/3960.11/4017.09/4074.07**
noncritical damage per activation. The general and Mental bonuses add; DEF1080
does not mitigate these Mental packets. Guaranteed critical checks multiply
each exact packet by1.5 before rounding. Ordinary Psyche packets and Mint's
attacks remain equal to a passive-free control. Both windows expire and the
240 total energy spent is retained, without replenishing or resetting state.
Native source compilation, WASM and actual Worker compare complete reports,
aggregation and replay at T1..T5. Existing tests retain exact expiry, refresh,
failed admission and cross-element cases. The fixture uses the native finite
owner clock and ordinary Burst branch, spends120 energy each time and exits
Lilith before its follow-ups. Other owner branches retain their independent
Fadia checks; this fixture does not establish frame-measured gameplay timing.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (424.0).
  - Secondary stat `HPMaxUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Eternal Waltz (Base ATK 424.0):
    - Base ATK = 600 + 424.0 = 1024.0.
    - Secondary Stat `HPMaxUp` contributes +41.25% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
