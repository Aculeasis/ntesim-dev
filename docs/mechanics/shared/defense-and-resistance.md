# Native defense and resistance policy


Responsibility: unresolved native reduction/cap/profile behavior, beyond the explicit damage model.

- Which negative-resistance branch applies at the Global cutoff? The known card retains selectable research models; [pinned calculator audit](../../DRIVE_CALCULATOR_RESEARCH.md) and [formula audit](../../../reviews/combat-formula-audit/review.md) disagree. A neutral/positive-RES match cannot distinguish them.
- How do multiple DEF-ignore/shred sources stack and cap natively? Current model aggregation is tested; native multi-source semantics are not independently established.
- How are boosted/flat and negative DEF handled by the gameplay adapter? Supported unbuffed target DEF does not implement an arbitrary layered profile.
- Which DEF/RES stats apply to a chosen target? Under [ADR 0009](../../decisions/0009-anonymous-targets-and-typed-content.md), target catalogs, monster profiles, pack lookups and Abyss logic manager overrides are retired. Targets are [anonymous stationary dummies](dummy-targets.md) whose base DEF and RES are explicitly authored in the shared simulation script rather than derived from native mob tables or community presets.
- How do shred, ignore and negative resistance interact with authored dummy stats? Script-defined target defense and resistance feed the core damage formula directly. Direct, alternate-array or mode-specific enemy stat overrides are out of scope; preserve explicit script assumptions and independent numeric checks.

Consequence: these choices change ordinary damage and packet-specific ignore eligibility. Mental/True bypass policies must not be used to hide an unsupported reached channel.

Current formulas and distinguishing independent numbers live only in [damage/stat zones](damage-and-stat-zones.md#3-direct-damage-crit-def-and-res). Return on applicable independent negative-RES/multiple-source/layered-defense evidence or an explicitly reviewed bounded target policy. Reuse the linked audits; do not fit a new formula to one aggregate hit.

## Complete defense and resistance formulation

1. **Target Defense Factor ($F_{\text{def}}$)**:
   - Attacker level scaling constant:
     $$A = 600 + 6 \times \text{attacker\_level} \quad (\text{at Lv80: } A = 1080)$$
   - Layered effective defense:
     $$\text{DEF}_{\text{eff}} = \left[\text{DEF}_{\text{base}} \times (1 + \text{DEF}_{\text{boost}} - \text{DEF}_{\text{shred}}) + \text{DEF}_{\text{flat}}\right] \times (1 - \text{DEF}_{\text{ignore}})$$
     For the standard unbuffed benchmark dummy ($\text{DEF}_{\text{boost}} = 0, \text{DEF}_{\text{flat}} = 0$):
     $$\text{DEF}_{\text{eff}} = \text{DEF}_{\text{base}} \times (1 - \text{shred}) \times (1 - \text{ignore})$$
   - Outgoing damage defense multiplier:
     $$F_{\text{def}} = \frac{A}{A + \text{DEF}_{\text{eff}}}$$

2. **Elemental Resistance Factor ($F_{\text{res}}$)**:
   - Net effective resistance against attacker's damage type:
     $$R_{\text{eff}} = R_{\text{base}} - \text{RES}_{\text{shred}} - \text{RES}_{\text{ignore}}$$
   - Non-negative resistance ($R_{\text{eff}} \ge 0$):
     $$F_{\text{res}} = \max(0, 1 - R_{\text{eff}})$$
   - Negative resistance ($R_{\text{eff}} < 0$, standard rational model):
     $$F_{\text{res}} = 1 - \frac{R_{\text{eff}}}{1 - R_{\text{eff}}} \quad \left(\text{alternative linear model: } 1 - \frac{R_{\text{eff}}}{1.10}\right)$$

3. **Damage Channel Bypass Rules**:
   - **Mental Damage**: Bypasses enemy defense completely ($F_{\text{def}} = 1.0$). Elemental / Mental resistance still applies.
   - **True Damage**: Bypasses both defense and resistance completely ($F_{\text{def}} = 1.0, F_{\text{res}} = 1.0$).

## Time-domain ordering and debuff cadence

- **Scope and attribution**:
  - `DEF Shred` is target-scoped (a debuff attached to the enemy), benefiting all attackers while active. Reapplication refreshes the debuff duration; multiple distinct sources stack additively.
  - `DEF Ignore` is attacker/packet-scoped (an attribute of the attacking character or skill), evaluated at packet creation or hit delivery.
- **Cadence**:
  - Debuff expiry is evaluated at discrete simulation timestamps before damage delivery.
  - Periodic DoTs (such as Scorch or bleed) evaluate target defense and resistance live on each individual tick.

## Boundary conditions and edge cases

Supported content adds applicable packet, equipment and team DEF-ignore sources
exactly, then caps the effective aggregate at 100% in both direct-hit and Break
evaluation. Zero's 75% execution, Lost Radiance's 25% and Iroi's 5% therefore use
exactly 100%, without an arithmetic failure or defense multiplier above one.
Individual exact-formula requests outside [0,1] still fail validation. This is
the bounded simulator policy; native multi-source stacking remains unverified
as described above.

- **DEF Shred Clamp**: $\text{DEF}_{\text{shred}} \in [0.0, 1.0]$. Shred cannot exceed 100%.
- **DEF Ignore Clamp**: $\text{DEF}_{\text{ignore}} \in [0.0, 1.0]$. Ignore cannot exceed 100%.
- **Non-negative DEF**: $\text{DEF}_{\text{eff}} \ge 0$. Defense cannot become negative or produce damage multipliers exceeding 1.0.
- **Benchmark Dummy Profile**:
  - Default stationary training dummy is calibrated at Level 80:
    - $\text{DEF}_{\text{base}} = 1080 \implies F_{\text{def}} = 1080 / (1080 + 1080) = 0.5000$.
    - Standard elemental base resistance: $R_{\text{base}} = 0.20$ (20% resistance $\implies F_{\text{res}} = 0.80$).

## Implementation handoff and verification case

- **Engine references**: `crates/core/src/combat/arithmetic.rs` (`evaluate_def_factor`, `evaluate_res_factor`) and `crates/core/src/combat/damage.rs`.
- **Verified numerical oracle**:
  - At attacker Lv80 ($A = 1080$), dummy DEF 1080:
    - Base: $F_{\text{def}} = 1080 / 2160 = 0.5000$.
    - With 25% DEF ignore: $\text{DEF}_{\text{eff}} = 810 \implies F_{\text{def}} = 1080 / 1890 = 4/7 \approx 0.571429$ (+14.29% DPS).
    - With 20% shred + 25% ignore: $\text{DEF}_{\text{eff}} = 1080 \times 0.80 \times 0.75 = 648 \implies F_{\text{def}} = 1080 / 1728 = 5/8 = 0.625000$ (+25.00% DPS).
    - With 100% DEF ignore: $F_{\text{def}} = 1.0000$ (+100.00% DPS).
  - Target with base RES $0.20$ subjected to $0.30$ total RES shred ($R_{\text{eff}} = -0.10$):
    - Under standard rational model: $F_{\text{res}} = 1 - (-0.10 / 1.10) = 1.090909$.
    - Under linear scaled model: $F_{\text{res}} = 1 - (-0.10 / 1.10) \approx 1.0909$.
