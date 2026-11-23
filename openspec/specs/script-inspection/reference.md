# Rust script inspection and diagnostics

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

The shared `ntesim_content::dsl` frontend owns parsing, configuration/build validation, stat derivation and source coordinates. CLI and WASM expose the same typed results. Studio editor/team cards consume this contract after the accepted [affected-boundary checks](../../../docs/DEVELOPMENT.md) gates.

## Inspect without running a rotation

Native `ntesim inspect-script SCRIPT.nte` and WASM `invoke("inspect_script", {"source": source})` return `InspectionResponse { party, diagnostics }`. Rust accepts the same monolithic script as compilation. It parses the entire source and validates bindings and configuration, then projects every actor through the existing native build compiler. It does not lower or execute rotation instructions, expend resources, sample RNG or advance time. A complete configuration without rotation instructions is valid. Inspection success establishes configuration support, not a legal executable rotation, complete kit coverage or calibrated DPS.

`party` is null on syntax, binding or configuration failure; no incomplete roster is presented as a valid party. Syntax errors anywhere in the script prevent inspection; there is no speculative prefix recovery. A syntactically valid rotation that later fails action lowering can still be inspected. Existing source-size/expression budgets apply; inspection never unrolls an unbounded rotation loop.

`PartyInspection` contains:

- `schema_version` (currently 1), `content_id`, zero-based `active_actor_index`, optional `start_fons` (null and zero differ), and ordered `actors` in authored slot order.
- `model_assumption_ids`: reached configuration/equipment assumption IDs, without descriptions. They do not certify unsupported future runtime branches.
- Each actor's zero-based `slot`, canonical `character_id`, declaration `loc`, `level`, `max_level`, explicit awakening selection, three purchased `talent_levels` in Normal/Skill/Burst order and separate `purchased_qte_level`.
- Optional `weapon` (`id`, effective declared/default `level`, `max_level`, `refinement`) and `set` (`id`, `count`); absence is null. No character/equipment names, descriptions, catalogs or asset payloads are embedded.
- `static_stats`: the same typed `GameActorStaticStats` supplied to native execution, including base/final ATK, HP/DEF, crit, damage/element/healing/shield and intensity fields. Existing serde defaults for omitted zero-valued fields apply. Exact `Ratio { numerator, denominator }` values retain their units; HP/DEF keep engine-owned integer settlement. `charge_efficiency` is the projected full multiplier, not an additive bonus.
- `build_decomposition`: typed native attribute contributions from character,
  weapon, set, authored additions and unconditional character/weapon passives.
  Contributions add in order; a base override replaces its preceding attribute
  sum. Exact attribute totals and pre-quantization ATK/HP/DEF reconcile with the
  compiled static stats. Canonical attribute IDs and source kinds carry no display
  names or localized descriptions. Conditional runtime effects are excluded.
- `initial_hp`, `initial_energy`, `energy_capacity`, `initial_cycle`, `initial_resources` (typed owner overrides; optional exact `timepiece` for Hotori, `grains`/`loans` for Chiz, `malice` for Blackbird, `crime` for Chaos integer `power` for Hathor, exact `moon` for Iroi, integer `plot` for Haniel integer `bullets` for Jiuyuan and integer `resonance` for Daffodil, before owner combat-entry grants; empty when omitted and explicit zero retained), exact authored `additive_stats` keyed by canonical stat ID, and `unmodeled_attribute_ids`. Unknown retained attributes are explicitly identified, not silently claimed as executable effects.

Static stats include authored additions and declared unconditional equipment effects exactly once. They are not a live combat snapshot: timed, conditional and team-triggered effects remain runtime-owned. UI code must not add equipment again, recalculate derived stats, infer missing combat rules or turn successful inspection into a successful simulation. Names/translations and formatting are separate presentation concerns. Consumers use the existing lossless integer/binary transport before approximate chart formatting.

## Compile and retain valid party information

Native `compile-script` and WASM `compile_source` return `SourceCompileResponse { compiled, party, options, diagnostics }`. Both use one parsed configuration. Successful compilation supplies the same party as inspection. If configuration succeeds but rotation validation fails, `compiled` is null, `party` and effective typed `options` remain available and diagnostics explain the failure. Configuration failure leaves all three null. A retained party never authorizes running a failed compilation. Native `validate-script` and WASM `validate_source` return diagnostic arrays.

`options` uses the Rust `CompiledOptions` contract: exact integer trial count,
seed and microsecond horizon, mode/critical/stop policies, optional Fons/stamina,
and `duration_specified`. An unspecified victory safety horizon is not an
authored duration. Compiled artifact loading verifies these options against the
source together with party and executable identity.

WASM `edit_script_options` edits parsed configuration ranges without lowering or
executing the rotation. Syntactically valid live loops and rotation preflight
errors remain editable. Unchanged effective values preserve authored expressions;
changed values replace their UTF-16 expression ranges. Blank duration removes an
optional victory horizon; time mode still requires duration. All candidate
configuration changes validate atomically, preserving unrelated source text.

`CompiledSourceProgram` retains schema/engine/source-build identities, the exact source and work limits, requested trial count and canonical initial scenario. Its command list is empty: actions and live control flow execute per trial, rather than being unrolled into timestamps. Compilation checks bindings, configuration, supported action models and closed scalar wait/repeat inputs without executing loops or sampling RNG. Dynamic arithmetic, admission and nonprogress errors remain runtime diagnostics.

Native `run-compiled-script FILE.json` and WASM `run_compiled_source` consume the entire successful compile response. They reject failed compilations, schema/engine identity mismatches and scenario/trial/party metadata inconsistent with the authored source. They reprepare the source under the saved work limits and use the same live runner as `run-script`/`run_script`; there is no internal scenario editing or legacy artifact fallback.

Studio editor/import validation uses `compile_source`, and team cards consume `PartyInspection`. The live Worker uses shared Rust source execution and exact report/replay interfaces; native live commands use the same domain owners.

## Source diagnostics

Diagnostics are typed `Diagnostic { code, message, loc }`. Frontend codes are `lexical`, `syntax`, `binding`, `configuration`, `evaluation`, `arithmetic`, `work_limit` and `action_limit`. Live execution additionally distinguishes `event_limit`, `admission`, `horizon` and `energy_gate`. Messages are human-readable; consumers use codes and coordinates without parsing text. Every location uses one-based `line`/`column` and zero-based UTF-16 `offset`/`length`, matching browser editors. CRLF and supplementary Unicode before a token preserve correct positions. EOF may have a zero-length range.

Source execution retains `failure_context { occurrence_index, command_index }` alongside `diagnostic`. Repeated ranges have distinct occurrence indices; a rejected implicit switch points to its actual native command. An expression/work/input-budget failure before yielding another input uses the attempted next occurrence index and no command. Admission details remain typed in `report.rejected_action`; an event-budget stop retains `report.limit_reached = {kind: event_count, limit}`. Neither a failed admission nor an exhausted work budget inserts elapsed wait time. A terminal safety-horizon/global drain failure without a current input points to the actual options declaration.

Known configuration failures are atomic and source-linked. The existing frontend may return the first syntax/configuration error or several binding errors; this API does not promise recovery after arbitrary malformed syntax. File/JSON-envelope errors are adapter failures without an invented script line.

Compiled `actionLocations` and `incomingDamageLocations` retain authored locations alongside lowered command/event identities. They do not make a raw JSON request source-aware. General live-execution failures retain source-linked typed causes under [DSL execution](../script-execution/reference.md) and [runner diagnostics](../combat-execution/reference.md#waiting-and-idle-diagnostics); inspection itself does not execute or certify a rotation.

## Verification

Rust tests check exact independent stat values, all authored party slots, equipment/active identity, zero versus absent resources, no execution of loops, atomic invalid configurations, retained party on rotation failure and diagnostic positions after Unicode/CRLF. CLI and WASM adapter tests compare the same typed responses. Bounded current-artifact WASM/Worker smoke checks compare complete native results and diagnostic ranges without reimplementing domain arithmetic in TypeScript. GUI rendering and full browser replay/aggregation are checked at the browser integration boundary.
