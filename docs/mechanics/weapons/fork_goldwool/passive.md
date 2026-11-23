# What's Desired — passive

- Native identity: `weapon:fork_GoldWool`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

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

**Passive Name**: Golden Fleece

> Increases the wearer's Lakshana DMG by <lv>{0}</>. Increases the wearer's CRIT DMG by <lv>{1}</> for <lv>{2}</>s when they cast a Redirect Skill or Ultimate. Resets the duration when triggered again.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_GoldWool_Up | buff_GoldWool_CritDamageUp | buff_GoldWool_CD |
|---|---|---|---|
| T1 | 20.00% | 40.00% | 20.00 |
| T2 | 25.00% | 50.00% | 20.00 |
| T3 | 30.00% | 60.00% | 20.00 |
| T4 | 35.00% | 70.00% | 20.00 |
| T5 | 40.00% | 80.00% | 20.00 |

## Activation and trigger mechanics

`weapons.gold_wool_v1` adds the permanent20/25/30/35/40% wearer Lakshana
bonus to the ordinary damage zone. Admitted Redirect Skill and Ultimate
casts grant wearer-only40/50/60/70/80% CRIT DMG for20s, before their impacts.
Either trigger refreshes the same window without stacking. Other actions
and failed admissions grant nothing; field exit preserves the deadline.
The CRIT DMG window is independent of packet element and affects only
critical damage, while the permanent bonus requires Lakshana.

The [continuing source](../../../../examples/gold-wool-refresh-cycles.nte)
uses legal A0 Chaos/Mint builds, a paid Burst and two Skills. Rust fixes
ATK1000, general bonus25% and base CRIT DMG50%, comparing every native
ordinary/transformed Normal1, bite and Suppression packet against the same source without the proc.
For a coefficient `c`, permanent Lakshana adds exactly `1000*c*ratio` to
noncritical damage alongside Chaos's existing Warrant zone; it never
multiplies that zone. Critical damage multiplies the resulting value by
1.9/2/2.1/2.2/2.3 inside a live window and1.5 outside it. The source reaches
100% ordinary critical chance and uses seeded rolls; actual rounded critical
damage is checked alongside both exact damage alternatives. The early Burst
refreshes the first Skill grant at1.054994s, the30s Skill opens its next
window at31.054994s, and
off-field waits leave expired Normal attacks with only the permanent bonus.
No resources or clocks reset. Native source execution and replay compare all
ranks through full reports. Browser transport parity is checked at the affected integration boundary. Existing Rust cases retain alternate
elements, noncritical damage, exact expiry, failed admissions and source
bindings; native action clocks remain modeled.

### Chaos successful-finish profile

`weapons.gold_wool_chaos_realfinish_v1` selects the callback distinction in
the [Chaos handoff](../../characters/chaos/action-damage.md#complete-implementation-handoff)
when that owner's ordinary clock is accepted. Redirect Skill grants at its
successful full recovery; Burst still grants at accepted commit. Other owners
and independent caller-timed owner fixtures keep `weapons.gold_wool_v1` admission
timing. This is a selected callback model, not an observed native dispatch.

The completion callback follows the montage's reached positive-hit pauses,
including an inherited pause. Interruption, field exit, replacement or source
death before completion cancels that unearned grant. Earlier granted windows
remain intact. A fully completed montage with zero applied damage still grants,
without inventing hitstop. The same self-only key refreshes for20 World seconds;
the first Skill's bites cannot use its own later bonus.

Independent Rust checks cover all five ranks, exact expiry, Burst refresh,
interruption on either side of completion, source death, zero-damage recovery
and inherited Riposte pause. At ATK1000, Warrant+.30, permanent+.20 and DEF.5,
T1 Skill bites have noncritical618.75/1631.25 and critical928.125/2446.875;
the following Normal1 has136.5/259.35. Successful connected Skill grants at
1054994us; no-damage Skill grants at955994us. Legal native two-period builds
retain all-rank alternatives, two recipients, exact channels and seeded
aggregation/replay.

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (570.0).
  - Secondary stat `CritBase` adds into its corresponding percentage stat zone.
  - Passive modifiers add into their corresponding formula zones (`AtkUp` in ATK%, `DamageUp` in general/elemental damage bonus `M_bonus`).
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 What's Desired (Base ATK 570.0):
    - Base ATK = 600 + 570.0 = 1170.0.
    - Secondary Stat `CritBase` contributes +24.00% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
