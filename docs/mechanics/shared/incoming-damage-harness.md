# Explicit incoming-damage assumptions

The DPS simulator accepts explicit incoming-damage, Critical Dodge and parry assumptions to reach character/gear mechanics. Targets remain anonymous stationary dummies. There are no enemy catalogs, native monster stats, attack lists, movement scripts, boss phases or encounter simulation.

Incoming events share the real combat timeline and preserve declared recipient selection, mitigation, shield/HP order, healing/shield callbacks, survival state and action-window costs. Ordinary source inputs must diagnose unsupported or invalid assumptions. A failed or incomplete run cannot enter a successful comparison.

The content runner owns incoming settlement and its typed receipts. Use [action admission](action-admission.md), [actor lifecycle](actor-lifecycle.md), [formula zones](damage-and-stat-zones.md) and each reached character/gear owner for exact windows, defenses, resources and callbacks. Native/WASM/actual Worker reports, aggregation and replay must agree. Independent numeric and causal checks belong to those shared and owner paths.

The former standalone enemy/boss-phase research harness and `enemy-stats` CLI were removed under the [anonymous-target decision](../../decisions/0009-anonymous-targets-and-typed-content.md). They are outside the product and must not be reintroduced.

## Fixed incoming source declarations

The Rust source compiler binds `incoming actor at=seconds damage=integer;`
to the existing named-recipient incoming event list. Constants/exact time
expressions settle once to whole microseconds and nonnegative safe integer
damage. Missing/unknown/duplicate fields, fractional damage, negative values,
sub-microsecond times, undeclared recipients, post-rotation declarations and
events beyond the horizon diagnose at source locations. Zero damage is retained.
A stable timestamp sort preserves declaration order at equal times; sorted
`incomingDamageLocations` map event indices to the authored declaration.
Report `incoming_damage_receipts[].event_index` and replay retain that identity.
The existing `combat.incoming_damage_fixture_v1` declaration records the
scenario assumption; supplied damage already passed external mitigation.

The [two-cycle shield script](../../../examples/incoming-shield-cycles.nte)
checks native Adler shield renewal and fixed off-field recipients. Independent
Rust numbers set DEF1000: a level1 grant supplies1800, then equal-time incoming
1000/5000 absorbs1000/800 and loses4200 HP; a later native grant absorbs1000
again on active Mint, following Adler's party shield. A separate named off-field
Adler hit loses1000 HP without using Mint's shield; a zero-damage peer event
remains visible. Native/WASM/actual
Worker compare source compilation, complete reports, aggregation and replay.
Periodic hurt, delivery-time active selection and sampled interval/amount
streams remain pending; fixed declarations consume no RNG.

## Captured nonrecursive incoming shares

The registry asks a reached content owner for one conserved incoming plan before
recipient shields. The root retains its authored event index, recipient and full
delivered amount; optional `redirected` receipts contain each child's own amount,
shields, actual HP loss, effect absorption, survival prevention and overflow.
Children settle directly and cannot request another distribution plan. Summing
the root's local settlement and each child's local settlement reproduces the
original delivered amount. Zero/default plans preserve existing receipts.

`fadia.incoming_share_v1` binds Fadia's selected interception/reverse-share and
captured living-ally floor. The original recipient's immunity/death suppresses
redistribution. A whole-HP redirected pool floors once; the root retains its
remainder, and equal reverse shares assign integer remainders by party slot.
This explicit quantization matches the integer incoming/shield/HP transport; it
does not imply measured native rounding or a second mitigation pass.

Damage and paid HP-cost primitives also record each actual applied loss and its
loss-time MaxHP. The existing vitals observer drains those records independently
of net HP/MaxHP changes: healing in the same event cannot erase an actual loss,
and structural MaxHP clamps create no loss-triggered resource/reflection event.
Edgar's normalized R6 ledger consumes those records. Fadia's remaining threshold,
mark and structural stat-transfer consumers remain in her open owner task.

## Explicit dummy combat observations

The structured Rust request optionally accepts ordered `dummy_combat_events`.
`attack_start` supplies `timestamp_us`, anonymous `target_index` and party
`actor_index`; it declares one qualifying party-directed skill-begin observation,
without scheduling damage or identifying an enemy/skill. `proximity` supplies
the timestamp, target index and `in_proximity` eligibility. Default owner proximity
is true in the stationary all-in-range profile. Indices, monotonic time, horizon,
event work limits and a supported content observer are validated before execution.

Proximity changes settle at ModePause; owner mark application/expiry settles at
BuffTickOrExpiry; attack starts settle at ProcCallback, after equal-time HP loss
and death. An expired mark cannot pay again for that attack, and an already dead
recipient/source/target cannot qualify it. Repeated equal eligibility does not
restart an accumulating timer. Declaration index and typed contents remain visible
in `dummy_combat` trace events and the replay request.

Hathor's ordinary Gaze is the first consumer. Its owner model declares all selected
mark/rearm/award rules; the shared event layer contains no character logic, enemy
catalog, attack automation or damage arithmetic. Public script lowering remains pending in native workflow; an unrelated incoming HP
event does not imply a dummy attack start.

`incoming_dummy_sources` optionally maps each incoming-event index to its explicitly
declared anonymous dummy damage instigator. Both indices validate before execution,
and a supported owner observer is required. This binding neither schedules damage
nor infers an attack-start event. After normal incoming/share/shield settlement,
positive actual HP loss or shield absorption on any captured recipient supplies one
contact observation. Zero delivery, full immunity/prevention and overflow on a dead
recipient do not qualify. Paid HP costs use their separate path. Source bindings
remain in the replay request, alongside the unchanged incoming event/receipts.

Hathor's held model consumes only enhanced Gaze through this route, removing its
generation before any accepted Power delta. Repeated observers and a same-time
explicit attack start cannot pay again; expiry still precedes a same-time contact.
Ordinary Gaze remains attack-start-only. No mitigation is applied a second time to
the declared externally mitigated incoming amount.

## Explicit incoming DEF stage

`combat.incoming_defense_v1` adds an optional checked fixture stage. The structured
request's `incoming_damage_basis` maps incoming-event indices to `before_defense`
or `externally_mitigated`; omission preserves the existing externally mitigated
meaning. For `before_defense`, the event's declared amount is after all externally
chosen modifiers except DEF. An explicit `incoming_dummy_sources` binding supplies
the anonymous dummy's authored positive level. No attack/skill catalog, autonomous
attack, elemental/RES/crit assumption or extra source ATK is inferred.

At delivery sample the original recipient's exact base/static/live DEF, including
living-source buffs and supported conditional gear. Apply the shared core factor
`A/(A+DEF)`, `A=600+6*source_level`, with no inferred shred/ignore, then round once
to whole HP using nearest/ties-up. The resulting delivered amount enters the
existing immunity, captured nonrecursive shares, shields and HP callbacks.
Children do not apply DEF again. Unflagged externally mitigated inputs never
receive this additional stage.

The root's optional `defense_mitigation` receipt retains `damage_before_defense`,
`source_target_index`, `source_level` and exact `recipient_defense`; its existing
`delivered_damage` remains the conserved post-DEF total. Children omit the basis.
This records the chosen assumptions and actual live sample without treating the
fixture formula as measured native incoming behavior for every attack kind.

Independent Rust checks use baseDEF1000, static+.5 and flat100: DEF1600 gives
`1000*1080/2680 ->403`. A live+100% adds base1000, giving DEF2600 and293 damage;
a200 shield then loses93 HP. Exact expiry restores403 without readiness polling.
A one-HP packet at factor.5 rounds upward; another declared source level changes
the factor. Fadia sharing receives the mitigated500 pool once, without another
pass through the child's much higher DEF. Invalid model/source/level/indices reject.

Two continuing native Adler shield periods use declared DEF1000/HP10000 and the
existing source example: raw1000/5000/1000 become519/2596/519 at dummy level80.
The first shield absorbs519+1281, loses1315 HP, and the renewed shield absorbs519;
active Mint final HP8685. A named off-field Adler1000 hit is mitigated to519
and bypasses the party shield on Mint, leaving Adler9481. Native aggregation,
seeded replay and JSON/binary adapter checks
preserve pre-DEF provenance, exact sampled stats and the unchanged external peer
zero event. This shared numeric fixture is a dependency for Hathor's Cyclone DEF
carrier, not full-kit acceptance. Public source lowering remains native workflow.
