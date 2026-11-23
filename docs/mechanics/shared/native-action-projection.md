# Native damage-coordinate projection into research execution

This service extends [research build-to-run binding](native-build-projection.md) with the optional `action_projection` in `examples/native-built-game-research.json`. The policy is `native_atk_coordinate_v1`, with a required nonempty hypothesis. Omitting it preserves caller action coefficients. Enabling it additionally binds the consumed damage/action tables. Typed constants are bound through the engine identity, rather than retired source-file hashes. The same `run-game-built` command audits selected inputs and binds the combined request and engine identity.

For every ability invocation, the existing native damage-values compiler checks character configured-action membership, effect GAName ownership and the explicit zero-based array coordinate. The projection replaces the manual ATK rate with the exact selected `AtkRateBaseArray` receipt, without display-float conversion. Ability/effect IDs become canonical. Per-command receipts preserve the old rate, new rate and full source-bound damage report. Switch/wait commands retain their caller data. At least one invocation is required when the projection is enabled. Scenario action count is checked against the execution budget before compiling builds.

The shared projector preserves native ATK/DEF scaling and the explicitly supported Fadia Burst HP packets. A single nonzero term keeps its scalar stat/rate representation. Multiple nonzero terms compile to exact `mixed` ATK/DEF/HP weights and a whole-base rate of one; `compiled_atk_rate` retains the ATK component. Other HP owners still diagnose. A selected literal zero remains executable with its selected stat; an empty array remains `declared_empty`/null in source receipts and cannot establish a zero coefficient. If another term is nonzero, explicit zeros do not mask it. A row with no selected supported scaling coordinate diagnoses. Fixed-crit overrides require the declared probability model; packet damage channels retain their native identity independently from the actor's element. Unknown IDs and invalid coordinates fail before execution. These capability limits do not assert game rules.

Mixed weights survive preview profiling, action-list compilation, scenario export and native replay without reducing the packet to ATK or splitting it into extra hits. Source-overlay counterexamples at coordinates 0/7/12 independently produce bases 350/750/1050 with ATK1000 and DEF500. These tests validate selected coefficient consumption; they do not declare new baseline kit packets or native reachability.

Native CLI integration tests reconcile two continuing mixed inputs across two
dummy recipients: four target packets consume four continuing critical draws,
with exact total `8625/14` for the checked seed. Seed-batch input/report hashes
replay against the exported weighted request. Failed admission and malformed
weights cannot enter successful statistics. This acceptance covers coefficient
transport and shared packet arithmetic, while each kit still owns its dispatch,
captures, resource rules and executable branches.

Shared primary-AoE expansion copies the admitted packet's exact weights,
metadata, time and parent to every living eligible recipient. Only the target
and causal sequence differ; all copies share one energy opportunity. It does
not recompile secondary recipients from baseline coefficients after an explicit
source projection selected different weights.

Rust projection tests independently check literal-zero versus empty coordinates, zero ATK alongside DEF/HP, and two continuing zero-damage invocations: no target HP loss or hit-energy award. Explicit research overlays supply these counterexamples without adding speculative runtime constants.

Action-list projection is transactional. A late packet, skill selection or strict-policy failure leaves every original coefficient, coordinate and action identity intact. Retrying after correcting the input produces the same compiled scenario, receipts and seeded native report as a fresh compile.

This binds serialized coefficients, not gameplay reachability. Purchased skill level to array coordinate, passive/awakening modifications, dispatch/geometry/hit count, percentage/unit interpretation, costs/resource recipients and timings remain unresolved. Unconsumed source cost observations are omitted; caller energy costs remain explicit and are never inferred from absence. ChargeAdd, UnbalValue and FixedCritRate remain exact; executable application belongs to their owner policies. HeterochromeAdd is retained only for reached owner Cycle consumers and their explicit overlay checks; unbound values and StroyBlanceGERate stay cache-only. The all-hit element hypothesis and research crit/proc hypotheses remain separate explicit choices. This scalar projection does not establish gameplay completion; pending A/B/C/R/E work is scenario-specific.

Native build and runtime talent projection use the same typed coordinate and
resonance hypothesis descriptions. Applying runtime talent selection to an
already compiled build must preserve those identities; a caller-provided
conflicting description still rejects. Native ability identity stays separate
from effective talent coordinate: Baicang's triggered Words retain the configured
Burst slot, tags and evidence while their coefficient captures the effective
Burst talent. Their binding records an owner child rather than a new cast.

Reached Daffodil ordinary clock packets also expose exact typed HeterochromeAdd
through the separate shared Cycle ledger: one chain33.196, ordinary Skill10.098,
Echoes10.100, Finale0. AoE recipients do not multiply a causal opportunity;
Cycle efficiency, carry, cap and overflow stay independent of Ultimate Charge.
Unreached Branch/Entry Cycle values remain cache-only until their owner bindings.

Native action tags default to [typed consumed constants](../../../crates/data/src/ability_tags.rs)
for the baseline package. The 66 present rows and 26 missing configured rows
preserve native category and absence independently from `AbilityIndex` input
roles, including Chaos Skill/Burst's literal `Ability.Melee` tags. ASCII case
aliases resolve without rewriting native row IDs, including `GA_lacrimosa`.
Evidence names
`typed:ability_tags` and hashes only consumed tag fields. Activation assets and
relive flags remain cache-only. Explicit tag overlays replace the default rather
than filling their missing/empty rows; other package identities require an
explicit overlay and receive no baseline fallback.

## Authored held Normal inputs

Native requests preserve `hold_inputs`, keyed by action index, alongside
`plunge_inputs`. A hold supplies `duration_us` and boolean `moving` intent;
it cannot specify the branch, coefficient, cooldown or recovery. Baicang's
owner selects a heavy from the current Normal/dodge state and binds its native
coefficient and metadata before admission. Invalid selectors produce a failed
ordinary action without spending or scheduling a new packet. Invalid indices,
foreign owners/packets and deadline overflow reject during request validation.

The [owner model](../characters/baicang/action-damage.md#current-rust-execution-boundary)
documents the .2s threshold and release clock. Native build export, batch and
seed replay preserve the full input map and continuing state. Short holds retain
ordinary tap/Riposte selection. Public source hold lowering remains native workflow
work; the transport binding does not claim that unimplemented syntax executes.

The optional owner cancel-window model changes the native recovery gate for
the requested input only. Baicang's Normal→Skill and `player_input`/`jump`
windows require the Word clock. Jump is an explicit native cancellation input,
with no supplied coefficient, implicit airborne profile or stamina refund.
Already emitted packets preserve their original command and live native talent
binding. Unsupported/expired windows fail; general source jump authoring remains
native workflow work. Native and WASM report comparisons retain the jump input receipts.

Native `interrupt_events` contain sorted absolute `timestamp_us`/`actor_index`
pairs. They are explicit scenario assumptions, independent of damage amounts,
stagger inference or player input. Baicang's separate forced-interruption model
checks native producer emission before cancelling pending deliveries, retains
committed costs and exposes typed `interrupt_receipts` with event and original
command identities. Equal-time hits/procs settle first; ordinary action admission
still precedes interruption. Unbound owners/models reject before execution.
The events and receipts survive native build/batch/replay and WASM transport;
their public source syntax and final Worker presentation remain phase work.

Zankou additionally accepts optional exact `armor_damage` pressure under the
declared [normalized A3 armor model](../characters/zankou/action-damage.md#current-a3-and-a5-owner-bindings).
Omitting it keeps forced cancellation; supplied pressure uses the live Hunt/A3
coefficient and a per-action threshold. This fixture does not infer enemy hits
or native armor capacity, and other owners reject the additional pressure.

## Mint purchased/effective levels

`native_mint_skill_level_v1` replaces both the caller coefficient and coordinate. Each actor build must provide `purchased_skill_levels`, a map from configured Mint ability IDs to purchased levels; each invoked ability needs an entry. All entries, including unused selections, are validated and ASCII case aliases cannot duplicate an ability. The legacy homogeneous `scenario.actors[].purchased_skill_level` is retained as caller data and is not the source of this policy's per-ability levels. Other characters use the separate `native_skill_level_v1` policy described below; this Mint-specific policy does not bind their bonuses.

The checked model assumes initial purchased level 1, native `LevelsCostItems.Level = L` buys level `L+1`, and effective level selects damage-array coordinate `level-1`. Mint's nine contiguous upgrade coordinates support purchased levels 1..10. Every reached upgrade must meet native breakthrough/awakening prerequisites using the compiled build breakthrough. Missing/malformed prerequisites and out-of-range levels fail. This is a visible hypothesis, not new proof of native dispatch; thirteen damage coefficients alone do not establish thirteen purchasable levels.

Mint owns resonance_3: the frozen awakening row adds one level to Melee, Skill and UltraSkill, and the model activates it at awakening 3..6. QTE receives no bonus. The compiler checks the complete three-entry modification list, binds `1019AwakenEffect1` as an additional audited input, and records purchase steps, resonance source pointer, effective level, replaced coordinate and coefficient. Awakening >6, missing skills, source changes outside the supported model and extra selections under the old coordinate policy are errors. Separate levels are demonstrated in `examples/skill-level-built-game-research.json` and CLI tests.

`tools/verify_built_skill_levels.py` checks original hash-bound exports, purchased levels 1/8/10 at awakenings 0/2/3/6, exact Fraction damage/rounding/HP, PCG draws, replay and rejection. It also compares complete execution reports for the compiled scenarios against freshly built WASM, parsing raw WASM JSON in Python to preserve u128. The [model decision](../../../reviews/q10-game-action-binding/skill-level-model.json) records the rationale; the verifier is the current check. WASM currently consumes the compiled scenario; this research request path does not itself provide the Studio stat-projection/editor service or a WASM equipment-build compiler.

Only coefficient/progression selection is implemented here. Mint's full kit, automatically reached awakening/passive behavior, action packet counts, costs and timing remain partial; full-combat and strict flags remain false.

The output retains incomplete status and false strict/DPS/acceptance flags. Native compiler reports continue to show null hit count/reachability and their unresolved gaps inside each command receipt. CLI exit 0 indicates report generation, with execution success/completion inspected separately.

Rust tests exercise exact replacement/canonicalization, scaling/crit/type rejection, reached packet arithmetic and CLI table hashes/ownership/coordinate failures. `tools/verify_game_action_binding.py` independently reconstructs selected values from hash-checked exports at coordinates 0, 7 and 12 for three Mint effects, then checks successful built execution, replay, all consumed input hashes and six invalid request mutations. Packet trains follow their owner contracts rather than a three-hit fixture assumption. Run outputs go under `target/`. These checks establish partial arithmetic and source binding only.

## Reviewed owner packet identities

The shared packet projector now applies the exact owner relationship declared
in `characters/action_packet_bindings.rs` to coefficient selection as well as
metadata. `damage_values::compile_owner_packet` still validates the configured
initiating action, but checks the selected row against its declared native
child GAName. It does not add that child as a configured proactive action or
infer relationships from suffixes. Unbound mismatches keep the ordinary error;
strict execution still requires the owner's exact accepted model provenance.

Owner-edge and HP-eligibility checks use source-resolved canonical action and
packet identities. ASCII case aliases therefore preserve the same coefficient,
causal parent, receipts and strict hypothesis requirements, including Nanally
Support children and Fadia's supported HP Burst packets.

Nanally's Underboss consumes this path with the actual initiating Normal,
Skill or Support identity and the separately captured Burst coordinate. The
Support-to-Cat row remains native `GA_Nanally_UltraSkill`, while its causal
parent stays `GA_Nanally_QTE`. At coordinates0/7 the independent Cat QTE
coefficients are **1.5/2.571**; another configured Skill cannot claim that
Support-only relationship. Native/WASM/actual Worker continuing cases preserve
full reports, unequal talent capture, aggregation and replay. Native ability
dispatch, timing and summon limits retain their owner contracts.

## Universal released character skill levels

`native_skill_level_v1` extends per-ability purchased/effective level and coordinate projection to all released characters. Each actor in 1..4 slots provides `purchased_skill_levels` for 1..4 configured proactive abilities, compiled against typed `character_actions` breakthrough prerequisites. The 92 action identities, native indices and purchase requirements preserve the consumed fields of the cache-only `DT_CharacterAbilityConfig` export. Currency/item costs and noncombat groups are omitted. Explicit configuration overlays replace the baseline and retain prerequisite validation; they never fill missing fields from it.

Resonance bonuses use typed `resonance_levels`: 66 consumed skill identities preserve the level additions from the cache-only awakening exports. At least three selected awakening effects activate +1 effective level for listed abilities (`Melee`, `Skill`, `UltraSkill` or `QTE`); this depends on the selected count, including sparse selections. Edgar and Linko include QTE rather than normal; Chiz declares no skill-level additions. Unlisted abilities and lower selected counts receive no bonus. Array coordinates equal `effective_level - 1`. Receipts use `table:typed_resonance_levels` and pointers into the consumed transport. Explicit owner overlays replace the baseline and retain schema, effect and modification validation.

`examples/universal-skill-level-built-game-research.json` binds a multi-actor team with Mint and Sakiri. Verification in `tools/verify_universal_skill_levels.py` checks 4 multi-actor awakening configurations, rejections and complete compiled-scenario WASM report parity. Ordinary runs write to `target/verification/universal-skill-levels/`; optional run receipts also belong under `target/`.

## Universal weapon curves and refinement procs

`native_weapon_curve_v1` projects equipped native parameters from `DT_ForkUpgradeStarDataTable` and `CT_ForkBuff`. Item-specific values, trigger/recipients and lifetime rules are owned by [Mind Royale](../weapons/mind-royale/passive.md) and [Good Boy's Grand Adventure](../weapons/good-boy/passive.md); this card owns the compilation interface and provenance, not another passive specification.

`examples/universal-weapon-curve-built-game-research.json` exercises the multi-actor projection. `tools/verify_universal_weapon_curves.py` checks selected refinements, invalid requests, replay and complete compiled-scenario WASM parity. [owning Rust checks](../../../crates/content/src/simulation/tests) retains its original inputs/model boundaries.
