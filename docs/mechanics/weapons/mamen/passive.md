# Contemplative Cat — passive

- Native identity: `weapon:fork_mamen`.
- Responsibility: this card owns the item's combat passive behavior in the frozen Global package.
- Weapon Quality: `EItemQuality::ITEM_QUALITY_ORANGE`; Group: `ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR`.

## Base stats and scaling

Base weapon ATK follows the upgrade progression curve and breakthrough stages:

| Level | Base ATK (Curve) | Breakthrough Stage | Max Level | Breakthrough ATK Add | Secondary Stat |
|---|---|---|---|---|---|
| Lv 1 | 33.0 | Stage 0 | Max Lv 20 | +0.0 | CritDamageBase: 17.60% |
| Lv 20 | 116.0 | Stage 1 | Max Lv 30 | +21.0 | CritDamageBase: 22.00% |
| Lv 40 | 203.0 | Stage 2 | Max Lv 40 | +43.0 | CritDamageBase: 26.40% |
| Lv 60 | 290.0 | Stage 3 | Max Lv 50 | +66.0 | CritDamageBase: 30.80% |
| Lv 80 | 377.0 | Stage 4 | Max Lv 60 | +89.0 | CritDamageBase: 35.20% |
| Lv 80 | 377.0 | Stage 5 | Max Lv 70 | +112.0 | CritDamageBase: 39.60% |
| Lv 80 | 377.0 | Stage 6 | Max Lv 80 | +135.0 | CritDamageBase: 44.00% |

- **Maxed Weapon Stats (Lv 80, Stage 6)**: Final Base ATK = **512.0** (377.0 + 135.0); Secondary Stat = **CritDamageBase 44.00%**.

## Refinement scaling

**Passive Name**: Mammon

> Increases Cosmos DMG by <lv>{1}</> for every <lv>{0}</> Fons the wearer holds, up to <lv>10</> stacks.

Parameters across refinement ranks T1 to T5 from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`:

| Rank | buff_mamen_fons | buff_mamen_CosmosUp |
|---|---|---|
| T1 | 100000.00 | 2.50% |
| T2 | 100000.00 | 3.00% |
| T3 | 100000.00 | 3.50% |
| T4 | 100000.00 | 4.00% |
| T5 | 100000.00 | 4.50% |

## Activation and trigger mechanics

The executable `weapons.contemplative_cat_v1` model reads one explicit party-wide `options start_fons` balance at damage delivery. It grants the wearer `min(10, floor(Fons / 100000))` stacks, each adding the selected refinement's Cosmos ratio to the ordinary damage-bonus zone. Zero is valid. A missing balance diagnoses when the passive is reached; projection never invents a weapon-local wallet. This is a continuous stat predicate with no ICD, duration, refresh or switch reset. Other elements and other party members receive no passive damage bonus.

Fons remains the authored account-currency assumption for the whole run. Chiz's combat Grain, loans and Ultimate Energy cannot replace or mutate it. No native combat currency gain/spend or account farming is inferred for the stationary-dummy scenario. Frozen Global release applicability remains unverified; the accepted model is explicit in projection and combat provenance.

## Complete implementation handoff

- The level 80 weapon contributes 512 base ATK and 44 percentage points of crit damage once through the shared build compiler. A character with 600 base ATK therefore has 1112 combined base ATK before percentage and flat bonuses.
- Independent additive-zone check: 250000 Fons means two stacks. Alongside an authored 10% general damage bonus, T1 gives `(1 + .10 + 2 * .025) / (1 + .10) = 23/22` relative damage; T2–T5 give 58/55, 117/110, 59/55 and 119/110. Existing Rust boundary checks cover below-threshold Fons, exact steps, the ten-stack cap, wearer-only scope and non-Cosmos exclusion.
- `examples/contemplative-cat-cycles.nte` equips the legal GroupFour weapon on A0 Chiz and runs two continuing Normal/Skill visits, interleaved with Edgar's two paid Skill/Burst cycles. All five refinements compare against the same source-built stats with zero Fons: only the wearer's Cosmos damage changes. Two five-input Chiz chains produce 4500 Grain while the authored 250000 Fons stays separate. Edgar retains unchanged damage and pays 200 energy without a reset.
- `simulation/tests/mamen_cycles.rs` owns the independent ratios, continuing resource ledger and deterministic replay. These five public sources also cross CLI/WASM/actual Worker comparison, aggregation and replay. Projection execution validation applies to this weapon slice. Chiz's held Skill, loans and market Burst remain separate incomplete owner work; this source does not claim a complete Chiz kit.
