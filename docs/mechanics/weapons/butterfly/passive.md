# Reality Refuge — passive

- Native identity: `weapon:fork_Butterfly`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 37.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 12.00% |
| Lv 20 | 128.0 | Stage 1 | Max Lv 30 | +26.0 | AtkUp: 15.00% |
| Lv 40 | 224.0 | Stage 2 | Max Lv 40 | +51.0 | AtkUp: 18.00% |
| Lv 60 | 320.0 | Stage 3 | Max Lv 50 | +77.0 | AtkUp: 21.00% |
| Lv 80 | 416.0 | Stage 4 | Max Lv 60 | +102.0 | AtkUp: 24.00% |
| Lv 80 | 416.0 | Stage 5 | Max Lv 70 | +128.0 | AtkUp: 27.00% |
| Lv 80 | 416.0 | Stage 6 | Max Lv 80 | +154.0 | AtkUp: 30.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **570.0** (416.0 + 154.0); Secondary Stat = **AtkUp 30.00%**.

## Refinement scaling

**Passive Name**: Swallowtail

> Increases Anima DMG by <lv>{0}</>. Increases the wearer's Attachment DMG by <lv>{1}</>. Increases the Attachment DMG bonus to <lv>{2}</> for <lv>{3}</>s when the wearer casts an Ultimate. Effect does not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_Butterfly_DamageUpNatureBase | buff_Butterfly_attachedup | buff_Butterfly_attachedup2 | buff_Butterfly_CD |
|---|---|---|---|---|
| T1 | 15.00% | 10.00% | 20.00% | 6.00 |
| T2 | 17.50% | 11.25% | 22.50% | 6.00 |
| T3 | 20.00% | 12.50% | 25.00% | 6.00 |
| T4 | 22.50% | 13.75% | 27.50% | 6.00 |
| T5 | 25.00% | 15.00% | 30.00% | 6.00 |

## Activation and trigger mechanics

`weapons.reality_refuge_v2` closes the former inert Attachment boundary. The
wearer's Anima bonus applies to delivered Nature packets. Attachment-tagged
wearer packets additionally receive the base Attachment ratio in the same damage
zone. A successful wearer Ultimate replaces that ratio with the higher frozen
Ultimate value for six world seconds; it does not add the two Attachment ratios
or multiply either by the Anima bonus. A recast refreshes one instance without
stacking. Exact expiry precedes an equal-time hit. Switching preserves the window
and each off-field contact samples its current value. Another actor's Ultimate
and a rejected wearer admission grant no window.

[Jiuyuan's natural Pact](../../characters/jiuyuan/action-damage.md#natural-attachment-execution-october-25-2026)
supplies native Seed/Bud periodic and BudEnd packets with live wearer ATK, Normal
talent and Attachment metadata. Ordinary direct Skill/Burst/Normal packets get
only the Anima bonus; a teammate never inherits the wearer's weapon bonuses.
Periodic Pact damage is not generic DoT damage. Active Pact settlement and
other unbound character branches retain their own implementation limits.

The [shared script](../../../../examples/reality-refuge-cycles.nte) uses legal A0
equipment, two continuing Pact generations on two dummies, two real Burst spends,
35 Normal inputs to refill energy, and an off-field period with Mint Skill.
No state is reset. Rust independently fixes ATK to 1000 and removes other bonuses:
a level-1 periodic coefficient .15 gives idle damage
**187.5/193.125/198.75/204.375/210** and upgraded damage
**202.5/210/217.5/225/232.5** at T1..T5. Selected A2 with Normal8 changes only
the periodic base to .514; natural settlement uses its own 1.714 coefficient.
Tests check all 124 Attachment contacts, direct/teammate exclusions, six-second
expiry boundaries, off-field delivery, energy conservation/overflow and serialized
replay. Native/WASM/actual Worker compare complete reports, aggregation and replay
at every refinement, including the unequal Normal8/Skill1/Burst1 A2 variant.
Native cast timing and contact order remain the disclosed public/owner models;
executable coverage does not certify native callback internals or Global release.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Reality Refuge (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `AtkUp` contributes +30.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
