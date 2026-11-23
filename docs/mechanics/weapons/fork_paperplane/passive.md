# Clear Skies — passive

- Native identity: `weapon:fork_PaperPlane`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 31.0 | Stage 0 | Max Lv 20 | +0.0 | AtkUp: 10.00% |
| Lv 20 | 108.0 | Stage 1 | Max Lv 30 | +21.0 | AtkUp: 12.50% |
| Lv 40 | 189.0 | Stage 2 | Max Lv 40 | +41.0 | AtkUp: 15.00% |
| Lv 60 | 270.0 | Stage 3 | Max Lv 50 | +62.0 | AtkUp: 17.50% |
| Lv 80 | 351.0 | Stage 4 | Max Lv 60 | +82.0 | AtkUp: 20.00% |
| Lv 80 | 351.0 | Stage 5 | Max Lv 70 | +103.0 | AtkUp: 22.50% |
| Lv 80 | 351.0 | Stage 6 | Max Lv 80 | +124.0 | AtkUp: 25.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **AtkUp 25.00%**.

## Refinement scaling

**Passive Name**: Paper Squadron

> Increases Anima DMG dealt by the wearer's Redirect Skill and Ultimate by <lv>{0}</>.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_PaperPlane_Up |
|---|---|
| T1 | 20.00% |
| T2 | 23.75% |
| T3 | 27.50% |
| T4 | 31.25% |
| T5 | 35.00% |

## Activation and trigger mechanics

- **Field State & Trigger Conditions**:
  - Evaluated based on the wearer's active vs off-field state and damage delivery.
  - Parameters specify explicit cooldowns and thresholds.
- **Stacking and Refresh Behavior**:
  - Stacks are bounded by the explicit limits declared in the passive description.
  - Reapplication adheres to the identified ICD and expiration/reset rules.
- **Reset / State Transitions**:
  - Switching characters or transitioning field state triggers resets where explicitly stated in the skill description.

### Continuing native execution

`weapons.paper_plane_v1` uses each delivered packet's native Anima element and
E/Q source category, independently of the initiating action's category. The
bonus adds to the existing damage zone and belongs only to the wearer; Normal,
other elements and allied packets do not inherit it. There is no proc clock,
stack or field-entry reset for this unconditional passive.

The [shared Mint script](../../../../examples/clear-skies-mint-cycles.nte)
retains legal A0 builds, two paid Bursts, forty intervening Normal inputs and
explicit Support recovery waits. Its two Skills and two seven-packet Bursts
deliver 36 eligible hits across two anonymous dummies. Rust independently
checks T1–T5 bonuses of .20/.2375/.275/.3125/.35 added to the authored .25
damage bonus: the relative factors are 1.16/1.19/1.22/1.25/1.28, with every
Normal and allied packet unchanged. Earlier tests retain wrong-element,
unknown-category and off-field delivery boundaries. Native/WASM/actual Worker
compare source compilation, reports, aggregation and replay at every rank.
Native Mint timing remains the disclosed owner model; this does not close its
remaining kit branches.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `AtkUp` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Clear Skies (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `AtkUp` contributes +25.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
