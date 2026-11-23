# Danzaburou — passive

- Native identity: `weapon:fork_tuansanlang`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE`.

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

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **475.0** (351.0 + 124.0); Secondary Stat = **UnbalIntensityBase 120 points**.

## Refinement scaling

**Passive Name**: Danzaburou

> Increases Break Intensity by <lv>{0}</> for all characters of the same Esper Type as the wearer (including the wearer) when the team has <lv>3</> or more such characters. This effect does not stack.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_tuansanlang_Unbal |
|---|---|
| T1 | 70.00 |
| T2 | 84.00 |
| T3 | 98.00 |
| T4 | 112.00 |
| T5 | 126.00 |

## Activation and trigger mechanics

`weapons.tuansanlang_v1` supplies 70/84/98/112/126 Break Intensity
points to matching-element recipients when at least three deployed roster
slots share the carrier's element. The carrier counts toward that threshold.
Only living carriers supply an aura; switching does not disable it. Multiple
eligible carriers use the strongest bonus once, never their sum. A defeated
matching peer remains a deployed roster slot but cannot supply an aura or
contribute a new Break packet. These eligibility choices are explicit product
hypotheses, not recovered native execution.

The [continuing script](../../../../examples/danzaburou-break-cycles.nte)
uses legal A0 Fadia/Danzaburou, Haniel, Aurelia and Mint builds. Three native
Haniel Normal hits cause three real Break generations/recoveries against an
authored capacity1/10 dummy. The off-field carrier buffs the three Psyche
characters; Mint receives no aura. Companion cases replace Aurelia with Zero
to miss the roster gate. A legal Adler/Danzaburou, Sakiri, Baicang and Mint
party supplies the actual carrier-death counterexample at5s, removing the aura
from the next two generations. Fadia's full native incoming model preserves
her last HP while allies live, so that test does not disable her protection to
manufacture a death. All cases use live source scheduling. No resources or
clocks reset between generations.

Independent Rust checks retain the weapon's **120 BI-point** secondary stat
separately from its aura. Level80 Break uses `3603*(.1/3)*(1+BI/300)`:
Haniel/Aurelia at T1 receive **44437/300**, at T5 **170.542**, versus
**120.1** without the aura. Fadia includes secondary120 once, giving
**58849/300** at T1 and **218.582** at T5. Each intermediate refinement is
checked exactly. Existing owner checks retain strongest-duplicate selection,
strict acceptance, source binding and post-capture source death. Native
compilation/WASM/actual Worker compare all ranks and eligibility cases through
complete reports, aggregation and replay. Target gauge/HP rules remain owned
by the explicit [Break model](../../shared/content-break-model.md#live-break-hp-contributions).

## Complete implementation handoff

- **Formula Zone Placements**:
  - Base ATK contributes to character `base_atk` in the additive stat zone: `base_atk = character_base_atk + weapon_base_atk` (475.0).
  - Secondary stat `UnbalIntensityBase` adds 120 Break Intensity points at max progression.
  - The eligible aura adds BI points to `1 + BI/300 + Break_bonus`; it adds no ordinary ATK or damage bonus.
- **Independent Verification Case**:
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Danzaburou (Base ATK 475.0):
    - Base ATK = 600 + 475.0 = 1075.0.
    - Secondary Stat `UnbalIntensityBase` contributes +120 BI points to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.
