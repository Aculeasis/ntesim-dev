# Tears Beneath the Mask — passive

- Native identity: `weapon:fork_NestBird`.
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

**Passive Name**: Nestbound Bird

> Applies Warning Gaze on enemies hit by the wearer's Ultimate. Marked enemies deal <lv>18%</> reduced damage for <lv>20</>s. Effect does not stack.

The description is bound through `DT_ForkUpgradeStarDataTable`; damage reduction
and duration use [typed sparse curves](../../../../crates/data/src/weapon_curves/mod.rs).
`buff_NestBird_down` retains signed native ratios which the owner explicitly
converts to positive reduction magnitudes. `buff_NestBird_CD1` retains keys at
T1/T5; T2–T4 select the Constant left key without expanding the source curve.

| Rank | Outgoing damage reduction | Mark duration |
|---|---|---|
| T1 | 18% | 20s |
| T2 | 21% | 20s |
| T3 | 24% | 20s |
| T4 | 27% | 20s |
| T5 | 30% | 20s |

The original export also contains `buff_NestBird_CD2`, an observed 30s curve
without an evidenced gameplay role. That observation stays in the cache-only
source; it is not a runtime constant, report field or inferred cooldown.

## Activation and trigger mechanics

- **Field State & Trigger Conditions**:
  - Evaluated based on the wearer's active vs off-field state and damage delivery.
  - Only the reduction and mark duration above are bound; CD2 supplies no cooldown.
  - `weapons.nest_bird_v2` marks the actual recipient of each connected category-Q packet after damage settlement, including off-field Vile Ash ticks. It supersedes v1's target-zero limitation. Each source/target mark remains independent, has one stack and refreshes its twenty-second expiry. Dummy targets do not acquire automatic attacks.
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
  - Declared Lv 80 Character with Base ATK 600 equipped with Lv 80 Tears Beneath the Mask (Base ATK 512.0):
    - Base ATK = 600 + 512.0 = 1112.0.
    - Secondary Stat `AtkUp` contributes +27.50% to the baseline stat pool.
    - Passive effect contributions scale with refinement rank T1 through T5 without double-counting base terms.

The [Vile Ash integration tests](../../../../crates/content/src/simulation/tests/zankou_dot.rs) exercise every compatible refinement with two declared dummies. Thirty periodic hits per target refresh both marks to 50s; a recast at 10.5s moves the last tick to 40.5s and both expiries to 60.5s. Exact-boundary expiry removes the marks without changing completed damage. Native/WASM/Worker transport covers a legal A0 build, two continuing Bursts with ordinary-hit energy refill and subsequent off-field delivery. These checks establish the selected model, not native game execution.

Continuing native cycles (`examples/nestbird-gaze-cycles.nte`, `nest_bird_cycles.rs`): an A0 Zankou wearer runs two Burst casts across the 20-second recast window against two declared dummies, refilling energy only through the interleaved 30-input melee chain before leaving the field. Every tier asserts the exact frozen reduction ratios (18..30%), the per-target tick schedules (1..30s and47..76s), one refreshed non-stacking mark per dummy expiring at96s, the exact 96s expiry boundary with unchanged settled damage, an independent ChargeAdd oracle for the102 settled chain packets (125 whole +97/250 charge),200 spent energy and deterministic replay. The same shareable source replaces the former inline transport scenario in `equipped_dot_parity.mjs`.
