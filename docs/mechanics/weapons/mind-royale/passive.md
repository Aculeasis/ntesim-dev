# Mind Royale — passive

- Native identity: `weapon:fork_lingganzhongjiezhe`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_PURPLE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 36.0 | Stage 0 | Max Lv 20 | +0.0 | UnbalIntensityBase: 24.0 |
| Lv 20 | 124.0 | Stage 1 | Max Lv 30 | +25.0 | UnbalIntensityBase: 30.0 |
| Lv 40 | 217.0 | Stage 2 | Max Lv 40 | +51.0 | UnbalIntensityBase: 36.0 |
| Lv 60 | 310.0 | Stage 3 | Max Lv 50 | +76.0 | UnbalIntensityBase: 42.0 |
| Lv 80 | 403.0 | Stage 4 | Max Lv 60 | +102.0 | UnbalIntensityBase: 48.0 |
| Lv 80 | 403.0 | Stage 5 | Max Lv 70 | +127.0 | UnbalIntensityBase: 54.0 |
| Lv 80 | 403.0 | Stage 6 | Max Lv 80 | +152.0 | UnbalIntensityBase: 60.0 |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **555.0** (403.0 + 152.0); Secondary Stat = **60 Break Intensity points**.

## Refinement scaling

**Passive Name**: Inspiration Terminator

> Grants the wearer <lv>{0}</> Ultimate Energy when casting a Redirect Skill. Triggers up to once every <lv>{1}</>s.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_xiangdao_energy | buff_xiangdao_CD |
|---|---|---|
| T1 | 10.00 | 20.00 |
| T2 | 12.00 | 20.00 |
| T3 | 14.00 | 20.00 |
| T4 | 16.00 | 20.00 |
| T5 | 18.00 | 20.00 |

## Activation and trigger mechanics

Runtime lifecycle uses `weapons.mind_royale_v1`: an admitted Redirect Skill reserves the weapon cooldown and schedules energy for the captured wearer. Delivery requires that wearer to remain alive. Switching to another actor preserves the grant; death skips it without altering energy, overflow or fractional carry and without refunding the admitted cooldown. This living-recipient policy is an explicit inference consistent with existing ordinary energy generation, not proof of native callback eligibility. Living grants retain the existing charge-efficiency scaling and capacity accounting.

Independent counterexample in `simulation/tests/energy_observations.rs`: 18 base energy at efficiency3/2 with carry1/3 and pool95/100 yields27 whole energy, retained5, overflow22 and carry1/3 for the off-field wearer. The same queued callback after death changes none of those quantities and emits no grant receipt. A full runner case admits Skill then settles same-time fatal incoming damage before the grant; CLI/WASM/Worker consumers compare that case with a living source-projected Mint build.

The source-built `examples/mind-royale-energy-cycles.nte` exercises every refinement with two paid Bursts and continuing resources. Redirect Skills at 0, 20 and 40 seconds grant 10/12/14/16/18 base energy to Mint. The Skill after the 10-second switch is admitted inside the weapon ICD and grants no weapon energy. Switching preserves the original deadline; exact-deadline casts are eligible. The opening grant overflows an already full pool; subsequent grants participate in ordinary capacity/spending accounting. Normal attack chains fund the later Burst without a resource reset. Ordinary team energy still reaches Adler; the weapon grant itself remains wearer-only.

Native `simulation/tests/mind_royale_cycles.rs` checks these deadlines, each refinement, the opening overflow, two 100-energy spends, deterministic replay and the equipped secondary stat's actual Break consumer. The same five public sources cross CLI/WASM and the actual Worker comparison/aggregation boundary. Projection receipts mark this execution validated under the existing research policy; this does not establish native Redirect applicability or measured game timing for other owners.

## Complete implementation handoff

- The weapon's 555 base ATK joins character base ATK once: a 600-base-ATK character has 1155 combined base ATK before percentage/flat bonuses.
- The level 80/stage 6 secondary adds 60 Break Intensity points, not 6000%. At level 80 against the source's zero-DEF/RES dummy with maximum Break gauge 30, the independent Break oracle is 3603 * (1 + 60/300) * (30/3) = 43236 damage.
- The passive changes energy rather than ATK or damage zones. Its non-stacking 20-second cooldown belongs to the wearer, and a rejected action never admits the trigger. Charge efficiency, fractional carry, capacity and death handling use the shared energy ledger.
- The scalar T-coordinate interpretation, Redirect Skill classification and admitted-cast clock remain explicit model assumptions. No native callback ordering, off-field native applicability or action timing is inferred from these numeric/source-consumer checks.
