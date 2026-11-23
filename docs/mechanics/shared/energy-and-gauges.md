# Unbound native energy and gauges


## Declared-build resource model for the collection handoff

The consumed six-by-six `DT_SkillTeamChargeRelationData` matrix is typed in [`energy/relations.rs`](../../../crates/content/src/simulation/energy/relations.rs): every eligible peer receives exactly 3/5, including a peer of the same element. Owner identity receives 1 independently of that matrix. The export is cache-only; an exhaustive six-element Rust check and the continuing resource/recipient tests preserve coefficients, carry and overflow. This storage migration does not expand packet eligibility or alter Cycle energy.

**Mixed source/inference, October 18; implementation target, not a report of current runtime coverage.** Mint's previously checked ordinary-hit receipts already select a positive-HP-loss, per-packet Ultimate grant and the source six-by-six team factor .6. Extend this explicitly to her selected alternate damage rows rather than giving them a hidden zero grant. A packet grants once when at least one eligible living target loses positive HP; recipient count does not multiply it. A fatal hit counts as positive HP loss. Misses, invulnerability and rounded-to-zero damage do not. Settle owner ChargeAdd with owner Charge Efficiency, and .6*ChargeAdd with each other living configured recipient's own Charge Efficiency. Keep exact fractional carry per recipient, floor accumulated grant to integer pool units, clamp integer balance to capacity, record discarded integer overflow and retain fractional carry. Switching never resets pools/carry. Source references: `DT_SkillDamageData`, `DT_SkillTeamChargeRelationData`, and the [Mint continuing receipt](../../../reviews/q10-energy-ledger/mint-continuing-energy.json).

**Narrow native carrier exception:** accepted `iroi.action_clock_v1` selects
`EffectApplication` for the bound tiny tap-Skill carrier only. Its source10Charge
must survive integer HP rounding (`.05` exact damage at1000ATK/halfDEF becomes0
integer loss); this is an explicit source/rounding inference. It grants once
per delivered causal group to eligible living targets through the same native
row/team/efficiency/carry/overflow service, with the actual hit basis retained.
No delivery means no grant; positive integer loss does not add a second grant.
All other current policies explicitly retain `AppliedHpLoss`. See the
[Iroi native tap checks](../characters/iroi/action-damage.md#executable-native-tap-skill-prerequisite).

Cycle/Heterochrome is a **separate per-owner** pool, nominal maximum100 from the [public mechanics calculation](https://www.reddit.com/r/NevernessToEverness/comments/1tlwa1w/translated_info_from_china_on_damage_formulas/) and source ASC ownership. Successful selected damage grants its HeterochromeAdd once per packet to that owner, with declared Cycle gain efficiency, cap100 and exact fractional carry; it does not use the Ultimate team's .6 matrix. Source-mutating calculators such as Mint R6 add only their explicit owner's amount once at their bound event; do not run them once for prediction and again for application. This scope is inferred because the source getters/mutations name owner ASC while the Ultimate matrix belongs to another channel. Example: ten dummies hit by Mint's two Skill packets give **4.9** raw Ultimate and **8.2** raw Cycle, not49/82; R6 adds2 Cycle once, giving10.2 Cycle, not Ultimate.

**Qualified Support switch (inferred from the [cycle guide](https://neverness.app)):** require a living outgoing actor with Cycle>=100, a distinct living incoming actor, a living locked target, legal switch timing, and a compatible adjacent element pair. Spend outgoing100 atomically on admission, retain incoming pool, invoke the incoming Support once and create the pair's reaction on the locked target. Same-element/nonadjacent switches are ordinary switches and do not spend/trigger Support. A failed switch spends nothing; a committed Support miss does not refund. A later third-element switch must satisfy the same live admission and the already active pair-state prerequisite; it is not a free fabricated third reaction. No separate numeric Support cost is inferred from an omitted `AbilityHeterochromeCost` scalar. Countercheck: outgoing100/incoming37 becomes0/37, not outgoing100/incoming0 and not both0. A same-element switch preserves100/37.

**Account stamina is scenario data, not a Mint talent constant.** The [combat-stamina guide](https://gamewith.net/nte/75694) distinguishes the white combat bar from Pixels/City Stamina and says its cap changes through VitalMax/account progression. Therefore an action using it requires declared `stamina_capacity`, `initial_stamina`, `stamina_use_efficiency` and `stamina_recovery_rate` (units/real second). None has a fabricated universal account default. Validate finite nonnegative initial/cap/rate, positive capacity, initial<=cap and positive use efficiency; record the chosen account/profile assumptions in results. Natural recovery is `min(cap, previous + rate*eligible_elapsed)` after the source recovery-pause model below; do not silently substitute rate0 or reset at cycle boundaries. A profile without rate may still execute a rotation that provably never reads/changes stamina, but must reject a later stamina-dependent action before execution.

**Inferred recovery gate:** spending stamina pushes `recovery_ready_at` to now+**2s**, using the located recovery-pause carrier's explicit duration; spending again refreshes that deadline. Recover only after that deadline and while no consuming action/StrengthGetEfficiency=0 carrier is active. Mint's hold overrides the recovery efficiency to0 from spin entry until exit; exit restores the preceding value, not an unconditional1. Off-field recovery uses the same account pool and gate. This preserves natural recovery as a modeled process while keeping its account-rate input explicit. The native generic carrier's universal application/clock is not proven; this is the selected source-based recovery model.

**Why a required profile is not a guessed gameplay number:** cap changes with account progress and furniture can affect consumption; a single hardcoded200 or an arbitrary refill speed would falsely certify a build. Complete conditional formulas plus mandatory scenario values make the branch implementable without assuming one account state. For capS, startingS and recoveryr>=0, two Mint full spins costing60 each leave at leastS-120 before recovery, so any S>=120 admits both; exact remaining stamina is obtained from the recovery equation. A concrete audit fixture may declare S=200,r=20, but20 is a **fixture choice**, never a discovered game rate or a general default. At spend t0 leaving198, with no further drain, it stays198 through t2, then reaches200 at t2.1 for that fixture; adding a spend at t1 moves the ready deadline to3. This counterexample distinguishes delay refresh from instant refill and preserves state across cycles.

## Native packet Cycle ledger

`combat.packet_cycle_accrual_v1` binds Baicang's consumed native rows when
his Word model is selected, and Blackbird's reached ordinary primary rows. The shared engine applies positive-HP and packet-group
eligibility, exact `cycle_gain_efficiency` (native request actor field, default
one), carry, cap and overflow. This multiplier is separate from Charge Efficiency;
there is no team distribution. The existing qualified switch spends the same
pool and records the spend without resetting carry. Other owners still require
their own concrete row bindings; metadata presence alone grants no resource.

Nanally now binds its consumed Normal/heavy/Aim/plunge/Riposte/Skill rows to
this same ledger. Its native zero Burst/Support/Cat/A3/Fair Duel channels do
not fabricate grants. Independent checks retain27.701 per full Normal chain,
110.804 across four continuing chains, qualified100 spend and a subsequent1.9
grant: final2, granted112, spent100, overflow10, carry88/125. The source fixture
also reaches two legitimate enhanced Bud generations after the actual handoffs;
Nanally starts with zero Cycle. Ultimate charge/efficiency/distribution stay
separate. The [owner contract](../../../openspec/specs/characters/nanally/reference.md#native-cycle-generation)
owns packet amounts and model limits.

Zankou's ordinary Normal and Heartwrench direct packets now bind the same
ledger using their frozen `HeterochromeAdd`, separately from Ultimate charge.
The five Normal inputs grant2.405/1.958/6.193/9.905/14.339 Cycle, totaling34.8;
two continuing chains grant69 whole units with carry3/5. Heartwrench grants
.833+10.491=11.324 once across AoE recipients; its periodic packets grant zero.
The combat-entry passive establishes the actual initial100 in both pool and
ledger before any spend. Source checks then spend that pool on qualified
switches, replenish it through actual hits and create/consume Kindling across
two continuing paid Burst cycles. No elapsed wait refills Cycle.

Actors using an accrual binding expose `final_actor_states[].cycle` with initial,
current/capacity, whole granted/spent/overflow, exact carry and conservation.
Independent Rust checks cover 37.372 per eleven-packet chain, three-chain
112.116 to pool100/overflow12/carry.116, 1.5 gain efficiency, AoE with a dead
primary recipient, zero damage/efficiency and a qualified spend followed by
continued generation. Explicit damage overlays retain missing-versus-zero
`HeterochromeAdd`; only concretely bound owner rows carry that stored field.

Blackbird's ordinary Skill also binds `blackbird.skill_cycle_fill_v1` at
successful admission: fill the integer Cycle pool to100 once, independently
of positive HP application and gain efficiency, without changing fractional
carry or Ultimate Energy. Record only the missing whole amount as granted;
connected Skill packets continue their separate accrual and overflow. Failed
admission and evolved Skill do not execute this ordinary activation binding.

## Located stamina-recovery carrier, October 18

### Executable declared account pool

`combat.stamina_v1` now consumes a complete optional profile from the shared
script options or typed request. `simulation/stamina.rs` owns exact account
balance, capacity, actual recovered/spent totals and per-command spend receipts.
Configured `dash` spends15 times the declared use-efficiency multiplier,
starts a .9s per-actor cooldown and refreshes the2s recovery pause. Recovery
advances analytically between actual events and at the final report time,
including off-field time, with no scheduled polling ticks. The .5s dash recovery
is still the disclosed public input model, not a native cancel measurement.
Missing profile fields, nonpositive capacity/efficiency, initial above cap and
invalid ratios diagnose; explicit zero recovery is preserved.

Native source `.team.stamina` and `.team.staminamax` read this same settled
account and exact capacity. Missing profile diagnoses when a query is reached;
explicit zero and fractional values are preserved. Queries add no recovery,
spend, events or RNG draws. `simulation/tests/source_queries.rs` independently
checks spends at0/.9/6s, zero at2.9s, .5 at3s, 15.5 before6s spending, .5 at8s,
1 at8.1s and20.5 at12s, including a field change and whole native replay. A
600s zero-rate fixture retains exact0/.125 balances with no recovery events.

The [continuing fixture](../../../examples/stamina-dodge-cycles.nte) declares
cap40, initial30, efficiency1 and recovery5 as scenario inputs. Spends at0,
.9 and6s total45. The refreshed pause ends2.9s, so the6s balance before
spending is15.5. At12s actual recovery totals35.5 and final stamina is20.5:
`30+35.5=45+20.5`. A600s horizon reaches cap40 with the same event count.
Rust checks exact fractional costs, cooldown/shortage atomicity, pause endpoint,
capacity and conservation; native/WASM/actual Worker compare full reports,
aggregation and replay. Unconfigured historical dash inputs and unbound
hold/channel/airborne consumers retain their separate limits.

The expansion bundle's `Buff_Player_StrengthDelayRecover#/1/Properties` declares HasDuration with explicit duration scalar **2.0** and owned tag `State.Property.ZeroPauseRecoverStrength`; `/2` grants that same tag. No Blueprint body or resource modifier establishes the tag consumer, application caller, refresh rule or resulting recovery amount. This is a located recovery-pause carrier, not proof that every attack pauses stamina recovery for two real seconds. Mint's hold-specific `StrengthGetEfficiency=0` carrier is a separate path in her owner card. Native stamina settlement/recovery still requires the reached consumer or an attributable independent ledger.

## Support activation metadata

The initial `GA_QTEBase#/9` activation component has **ActivationCondition=20**. Native enum metadata now establishes the activation-condition labels/ordinals, including LockedTarget=2 and QTE=4, but 20 is not an ordinal. A bit-position mask is a candidate interpretation only. The named `GAComponent_CheckActivateCondition` native consumer must establish decoding, AND/OR policy, target availability and QTE readiness. Empty `AbilityHeterochromeCost` Properties in base/Mint QTE still do not supply effective cost, threshold, deduction or refund. The metadata probe resolves labels, not the still-open Support legality/ledger.

## Resource calculators with mutation, October 18

The initial bundle's warning-free `Cau_ParryAddCharge` and `_boss`, both `CustomCalculateBaseMagnitude#/2`, cast **OwnerActor** to a player and directly call that owner's ASC **AddHeterochromeValue(50)** and **(100)** respectively, then explicitly return **0**. `Buff_Parry_CommonPlayerProperty` and `_boss`, both `#/2/Properties/Executions`, select these helpers as Source/snapshot PParameter1 magnitudes for **HeterochromeExecution**. Zero returned magnitude therefore does not mean zero resource request, and calling the calculator twice to predict a result would issue two mutation requests. Native boss/ordinary carrier selection, caller frequency, efficiency/clamp, deduplication and delivery are still unknown; these are not universal rewards for every dodge or an Ultimate Energy grant.

`Cau_Female051SkillAddCharge#/2` has the same valid-player/successful-curve mutation shape, but reads `DT_Female051EffectFigure:Female051_SkillEnergy` at coordinate 0 (**100**), then calls AddHeterochromeValue and returns zero. Both `Buff_Female051_AddEnergy#/2` and the separately named `Buff_BlackBird_AddEnergy#/2` reference this same calculator in HeterochromeExecution. Blackbird's [hybrid-server activation component](../characters/blackbird/malice-and-coordinated-damage.md#reached-hybrid-server-activation-and-execution-fields) already locates that carrier's caller; actual callback multiplicity, effective context and settled 100-unit write remain open. The local initialized 50 is unused. The corresponding [Mint](../characters/mint/action-damage.md#awakening-conditions-recipients-and-conflicts) and [Fadia](../characters/fadia/action-damage.md#skill-resource-callback-source-boundary) consumers already document their distinct curve arguments; do not generalize these constants across owners.

Responsibility: native resource source, recipients, capture/consumption and timing for still-unbound forms, summons, reactions and special owner branches.

Open questions include missing source delivery events, per-cast versus per-connected-hit versus per-recipient scope, miss/death/switch behavior, native duplicate suppression, source-specific efficiency/carry/fill policy, gauge clamping and read-before/after-consumption order. A saved `ChargeAdd`/`HeterochromeAdd` scalar or missing cost row does not supply these policies.

Exact direct-packet energy policies are now modeled as separate tested hypotheses for the inputs listed in the [direct-packet owner receipt](../../../reviews/q10-energy-ledger/direct-packet-energy-owners.json). Native eligibility and natural Branch, Entry, AirAttack, plunge, EvadeBranch or Chaos QTE dispatch remain unresolved; do not generalize the direct-input policies to those actions.

Consequence: energy/gauge decisions alter continuing rotations and affordability. Self grants cannot become generic team generation; a missing cost is not a free action. Known ordinary branches and exact per-owner native values are in character resource cards and their energy receipts; final integer ledger interfaces remain in [runner](../../../openspec/specs/combat-execution/reference.md#execution-and-checked-arithmetic).

Checked leads: `DT_SkillTeamChargeRelationData`, native damage/cost/figure tables, owner graphs, [native damage-value binding](native-damage-values.md), [owning Rust checks](../../../crates/content/src/simulation/tests/energy_observations.rs). Current typed grant/conservation services establish implemented arithmetic, not missing game dispatch.

The permanent owner tables resolve base-cost and configured-component availability. In particular, [Hotori's native cost/pause consumers](../characters/hotori/time-stop-resource-and-cooldown.md#reached-native-cost-and-pause-boundary) are identified despite omitted cost-subobject Properties; [Blackbird's hybrid-server activation and HeterochromeExecution declaration](../characters/blackbird/malice-and-coordinated-damage.md#reached-hybrid-server-activation-and-execution-fields) are available. Resume at those native consumers, not by searching again for the already-saved cost/gauge scalar rows. A known activation reference is not proof of final resource mutation.

Return on a compatible reached producer/consumer graph or a reviewed explicit source policy. Verify an empty-energy continuing rotation, independent recipient efficiency/fractional carry, overflow, delayed off-field hits and later Ultimate spending. Fractional timeline/report expansion for already-known rules is implementation work, not research evidence.

## Custom charge mutation parameter boundary

Read-only function metadata identifies **ModifyLogicCustomChargeValue(ModifyChargeValue, GEClassName)**, a float and FName with no separate ReturnValue. Linko's joined **GC_Linko_SkillDam:OnRemove** calls the main character's LinkoLogicObj with **ModifyChargeValue=0, GEClassName=Clean**. The source supplies a zero modification input and named discriminator; it does not by itself establish an absolute-set operation, a successful reset, the special Clean branch, Sync Frequency ownership/cap or an Ultimate Charge write. Preserve the main-character recipient rather than substituting party1072. The cue is warning-free; the native logic implementation remains the return condition.

## Complete energy and resource handoff

- **Resource pools and state**:
  1. `Ultimate Energy`: Cap 100 integer units. On-field hits grant `ChargeAdd * ChargeEfficiency`; off-field team members receive `0.6 * ChargeAdd * TeammateEfficiency`. Fractional carry is preserved across events and switches.
  2. `Cycle / Heterochrome`: Individual per-character pool (cap 100). Generated strictly by the owner's hits; qualified Support Switch costs 100 points atomically.
  3. `Combat Stamina`: Scenario-defined capacity (e.g. 200). Spending initiates a 2.0s pause before natural recovery resumes at `stamina_recovery_rate`.
- **Engine contract**: Declared in `crates/core/src/combat/resources.rs` and `crates/content/src/simulation/energy/mod.rs`.
- **Verification case**:
  - Actor with 0 Energy hits a dummy with a Skill packet (15.0 base charge, 120% efficiency).
  - Active actor gains floor(15.0 * 1.2) = 18 integer Energy (remainder +0.0).
  - Off-field teammate (100% efficiency) receives 15.0 * 0.6 * 1.0 = 9.0 -> 9 integer Energy.
  - Ten simultaneous dummies hit by one splash packet award 18 and 9 energy once (per-event cap, not multiplied by 10).
