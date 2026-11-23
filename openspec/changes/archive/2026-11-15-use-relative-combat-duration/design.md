# Design

## Context

See [proposal.md](proposal.md) for the user-visible problem and the capability deltas under `specs/` for behavior. The user explicitly authorized choosing and documenting practical assumptions where measurements are unavailable, including updating affected code and documentation during implementation.

The current integration uses `GameScenarioRequest.horizon_us` and `GameScenarioReport.elapsed_us` as absolute combat time. `runtime/source.rs::Trial::advance`, `runtime/execution.rs::advance_to` and `runtime/stopping.rs` compare absolute deadlines directly with that horizon. The live observation service returns absolute `.time`. Source experiment DPS and Studio observation/contribution statistics divide by the same absolute elapsed value.

Core already provides interval union and real/mode conversion in `crates/core/src/kernel/timing/mode_clock.rs`. It is a timing research service; `docs/mechanics/shared/clocks-and-action-timing.md` explicitly separates it from current live execution. Its current interval insertion rebuilds the union and its conversion scans intervals, so calling it unmodified for every runtime event would add avoidable work.

Hotori already has actual setup/entry/expiry/finisher/death generations in `characters/hotori/timepiece.rs`. `simulation/pause.rs` separately parks incoming enemy schedules and selected Vita Bud events. Existing `hotori_pause.rs` and `source_clock.rs` tests cover those causal boundaries. No global mode-timer clock currently follows them. Most cooldown, buff, reaction and resource deadlines currently subtract absolute timestamps; merely changing the duration/DPS denominator would leave that clock behavior wrong under the selected model.

The saved owner ability extracts in the character extracts contain `GAComponent_AbilityGamePaused` on an Ultimate for every one of the 23 represented owners. Twenty-one specify release at `Event.Montage.Player.Common.EndAbilitySkill`; Hotori specifies `EnterTimeStop`; Zero's component has no serialized fields. This supports the user's clarification that the rule covers ordinary Ultimates throughout the roster. These are retrieved component declarations, not recovered native countdown bodies or measured pause durations; runtime and ordinary tests must consume portable typed bindings without requiring the acquisition cache.

Studio trials already distinguish `run_sample` with timeline recording from `run_batch` without it. Only the first detailed execution is retained in ordinary results; loading another seed regenerates its sample. The collector currently allocates bins from requested horizon, clamps later hit times into the final bin, and `DamagePanels.tsx` uses that horizon for time-mode charts. Playback uses `report.elapsed_us` and event timestamps directly, which correctly preserves absolute spacing today.

## Goals / Non-Goals

**Goals:**

- Separate the relative stop target from absolute scheduler deadlines and safety limits, preserving exact causal execution and current action admission behavior.
- Produce both durations and one relative DPS result from each execution, with bounded observations suitable for streaming and restoration.
- Keep Sample events individually addressable during a relative-time plateau and preserve the complete absolute aggregate timeline.
- Make pause provenance and model coverage explicit; validate clock conservation independently of UI formatting.
- Give every reached ordinary timer a concrete default or named exception, preserving remaining lifetime and periodic phase without requiring new gameplay recordings to start implementation.

**Non-Goals:**

- An absolute-duration execution option, a global display-basis selector, or a second simulation for absolute statistics.
- A relative Sample axis, collapsed/expandable pause clusters, or changing an authored wait into a relative-time wait. Owner timers do change according to the selected policy below; an absolute event timestamp does not imply an absolute timer lifetime.
- New enemy behavior, geometry, acquisition infrastructure or a roster-wide animation research campaign. All currently supported Ultimates receive cinematic pause bindings from the existing owner inputs and timing profiles; unrelated future camera/menu pause systems are outside scope.
- Changing EPS, gross HPS, mean available SHP or field-time definitions. Their existing absolute-time accounting remains separate from relative DPS.

## Decisions

### 1. Maintain two elapsed clocks around one absolute scheduler

Use integer microseconds throughout:

```text
A = absolute elapsed combat time
P(A) = length of the union of actual mode-timer pauses within [0, A]
R(A) = A - P(A)
```

The event heap, receipt timestamps, authored scenario timestamps and wait durations remain absolute. A timer's semantic deadline belongs to its declared clock domain and is projected to the absolute heap only when that domain can advance. Cinematic/replay action timing and ordinary world-timer progression are distinct. `.time` reads `R(now)` from settled native state; live cooldown and modifier-duration queries read the appropriate retained timer remainder. `.duration` under `stop=time` reads the authored relative target. Under `stop=victory`, preserve the existing safety-limit value exposed by `.duration`; it is not a relative stop target in that policy. Do not add another public time-query syntax in this change.

Add a runtime clock state with generation-keyed active pause handles, an active-owner count and accumulated union duration. Settle the elapsed segment before opening/closing handles. Overlaps advance the paused union once; duplicate or stale closes are idempotent. Reuse core checked arithmetic/union semantics and its existing tests, extending that boundary for incremental lifecycle behavior rather than copying the arithmetic into content or TypeScript.

Forward execution and forward UI playback can use a running interval cursor. Arbitrary UI seeks use a prefix-sum index and binary search over native union intervals. Native clock ownership and pause classification stay in Rust; TypeScript only maps already validated intervals for presentation.

**Alternative considered:** Replace every event timestamp by relative time. A paused interval is many-to-one, losing action spacing and an unambiguous seek position. Keeping absolute identity avoids that loss and preserves existing causal ordering.

### 2. Resolve the relative boundary during event-driven advancement

Never compute the final absolute stop time by summing assumed future full animations. At each advancement boundary, use the current `R`, remaining relative budget and active pause state to choose the next real event, owner deadline, provisional relative endpoint or absolute safety ceiling. An event that opens, ends or shortens a pause causes the next bound to be recomputed before further advancement.

When the timer is running, the provisional endpoint is `now + remaining_relative_us`. While it is paused, progress to the next eligible event/release or absolute safety boundary. Empty gaps still advance in one step. Source waits remain absolute; recovery windows follow their owner's declared action-clock policy and can be suspended by another owner's cinematic or Time Stop.

Stop at the first absolute instant reaching the relative target. Preserve the existing inclusion of already scheduled equal-time work and its causal children, while preventing a new player admission at that completed boundary from opening another pause and extending the run. Later positive-time hits remain pending. Retain existing diagnostics for source inputs/waits that cannot complete, and the all-target-defeat stop policies.

Keep the default absolute ceiling at `ExecutionLimits.max_horizon_us = 3600000000`, independent of the authored target. A 3600-relative-second request passes preflight but can stop incomplete at that absolute ceiling if pauses prevent reaching it. Do not silently enlarge caller limits. Reports and diagnostics identify which bound was reached.

**Alternative considered:** Extend the old absolute horizon when a Burst is admitted. Early finisher, interruption, death and overlap would require rollback of speculative extensions. Observed clock progress provides the actual boundary directly.

### 3. Bind mode-timer pauses explicitly to content transitions

Every supported character's ordinary Ultimate opens a cinematic mode-timer pause at successful commitment and releases it at that owner's actual end-of-cinematic transition, normally the bound `Common.EndAbilitySkill` event. Resolve its absolute deadline through the owner's existing action clock, including any sourced playback progression, interruption and cancellation. Keep typed bindings with the relevant content owner and use a shared runtime pause service. Do not infer the pause length from the full lifetime of an Ultimate buff, field, transformation or summon. This roster-wide binding is required deliverable scope, rather than a follow-up after implementing Hotori.

For missing serialized defaults, including Zero, prefer the owner's bound end-of-cinematic notify from existing montage evidence; when that notify is unavailable, choose the currently consumed ordinary Burst action-completion/recovery boundary as the explicit modeled release. This is a concrete fallback, not a task to wait indefinitely for native-body recovery. Keep an owner-specific typed binding and disclose which release source was selected. Absence of a serialized end tag never means a zero-length pause. A timer covered by the declared fallback can produce a complete bounded-model result; corrupt inputs and genuinely unimplemented reached mechanics still reject. Existing owner timing references are updated alongside their consumed bindings.

Hotori extends this shared rule. The selected model pauses the mode timer at successful World's Tide commitment, hands off from cinematic setup to active Time Stop without a gap or double subtraction, and releases it at actual finisher delivery, expiry or owner defeat. Interruption before entry releases the executed setup pause. A rejected cast never opens it. Reuse the existing cast/mode generations and cancellation/death hooks so an old deadline cannot close a newer pause.

This is a selected source-informed timer model, not a claim that native countdown execution was recovered. `docs/mechanics/characters/hotori/time-stop-resource-and-cooldown.md` locates `GAComponent_AbilityGamePaused` with an end tag at `EnterTimeStop`; the active Time Stop behavior and the user's observation motivate the connected pause model. Record these distinct evidence and inference boundaries in the updated mechanics reference and a canonical consumed hypothesis. The resource's twelve-second active budget remains the existing owner rule; it is not an extra twelve-second timer pause added at commit.

A content binding identifies its actual mode-timer transition hooks and provenance. Cover every currently supported Ultimate, retaining source-defined or explicitly disclosed modeled lifecycle choices. Prefer each owner's sourced release over its selected completion fallback; never invent one shared duration for every Burst or multiply pauses by packet/target count. Bounded timer assumptions are recorded as assumptions, separately from unsupported dependencies.

Enemy scheduling, local animation clocks and friendly-effect progress use the explicit matrix below for ordinary Ultimates and Hotori. This supersedes the current Hotori documentation's claim that friendly cooldowns, buffs and reaction ticks always advance in absolute time. Finalizing combat clips an open pause to actual elapsed time for the receipt without pretending its owner naturally finished.

**Alternative considered:** Use `simulation/pause.rs::is_enemy_schedule` as the timer policy. It describes affected recipients, not challenge countdown semantics, and would conflate the two systems.

### 3a. Evidence, conflicts and selected assumptions

Research reviewed on 2026-11-15 in English, Chinese and Japanese. Source publication/server/build applicability is retained where available; no current-client measurement or frame-by-frame video verification was performed. A kit-text mirror, a guide and an individual test report have different evidentiary weight. None establishes every Global snapshot rule by itself. Do not import unrelated coefficients, kit changes or build advice from these pages.

| Source | Observation used here | Evidence boundary |
| --- | --- | --- |
| Saved owner ability components described in Context | All represented Ultimates have a pause component; most name the end-of-skill release, Hotori names Time Stop entry | Native configuration, not recovered countdown or timer-dispatch bodies |
| [Hotori kit text](https://www.neverness.app/characters/jin) | Ghost Orchid Crest at B2 permits Vita Bud attacks during Time Stop; her own attacks/resource pointer continue | Reproduced ability description; does not state that every friendly timer or summon is exempt |
| [Prydwen Hotori review](https://www.prydwen.gg/neverness-to-everness/characters/hotori) | Separately describes stage-timer pause and the Blossom enhancement allowing Vita Bud attacks during the stop | Secondary corroboration of the named attack exception, not a measurement of bud lifetime or every timer |
| [JaxiTVGaming test post and replies](https://www.reddit.com/r/NevernessToEverness/comments/1tj38oy/some_cool_hotori_interactions_to_consider_for/) | Author reports a five-second Mint Crit DMG buff surviving the stop. Scorch ticks are reported, but another tester initially finds them paused and later reproduces damage only for some application orders | First-hand community claims with clips attached; direct/reapplied reaction damage and periodic ticks are not conclusively separated. Videos were not independently measured here |
| [Buff/debuff timer discussion](https://www.reddit.com/r/NevernessToEverness/comments/1u4b9ou/does_anyone_knows_if_hotori_can_stop_the_affects/) | Players report retained buff/debuff durations, including negative effects on Hotori | Corroboration with no complete carrier-by-carrier measurement |
| [Recording-window rotation report](https://www.reddit.com/r/NevernessToEverness/comments/1uinpir/its_been_over_a_month_with_hotoris_release_what/) | A player describes fitting eight to ten absolute seconds of ally Ultimates into Hotori's five-second recording window | Supports pausing ordinary recording timers during cinematics, not a proof about every timer |
| [Mobalytics Hotori guide](https://mobalytics.gg/neverness-to-everness/characters/hotori-build-guide) | Explicit ally-cooldown pause and continuing buds; describes Charge reacting during Time Stop without an energy grant | Secondary guide, updated 2026-10-12, later than the preserved Global cutoff; use as corroboration/selected policy, not unquestioned native proof |
| [Japanese community wiki, caution section](https://wikiwiki.jp/nte/%E6%BD%AF) | Describes placed-field lifetimes continuing during Time Stop while Edgar's field cannot heal | Specific contrary observation to a universal freeze; generalizing it to other deployed objects is a selected low-confidence default |
| [Earlier Blossom discussion](https://www.reddit.com/r/NevernessToEverness/comments/1t94uuf/so_can_someone_explain_how_hotori_works/) | A commenter says buds consume their normal lifetime/shot budget during the stop instead of receiving extra free shots | Older first-hand community account; supports separating bud lifetime/count from the attack exception |

The coherent default is a paused world with a small set of progressing contexts. This preserves buffs through an unavoidable cinematic, prevents cooldown/ICD farming in frozen time, and makes Hotori's expressly described Blossom exception meaningful. It replaces the previous plan's unconditional real-time friendly progression. The following matrix is the selected implementation contract, including choices where evidence conflicts; it is not an assertion that every row was tested in-game.

### 3b. Complete timer-policy matrix

`Cinematic` means an ordinary Ultimate's actual cinematic phase, including Hotori's initial setup. `Hotori active` means her post-entry Time Stop. `Pause` retains exact remaining duration/phase and produces no catch-up at release. `Advance` uses the applicable continuing action/object clock, never host execution time.

| Timer/process | Cinematic | Hotori active | Selection and reason |
| --- | --- | --- | --- |
| Mode/challenge timer | Pause | Pause | Shared duration and DPS basis |
| Ordinary Skill/Burst/switch cooldowns, internal proc cooldowns and stack-decay clocks | Pause | Pause | Cooldown observations support Hotori; common paused-world default prevents gaining readiness or repeated procs for free |
| Independent timed buffs, shields, debuffs and reaction-status lifetimes, including ones on Hotori | Pause | Pause | Mint buff example and player reports; keep source-death/dispel removal immediate |
| Scorch, Nightmare, Heartwrench, Vile Ash and other ordinary periodic damage/heal phases | Pause | Pause | Conservative deterministic default for conflicting Scorch reports; both existing and newly applied carriers use it |
| Passive stamina/energy/custom-resource recovery, cooldown-like recharge and timed refunds | Pause | Pause | No free elapsed-time gain while the world is frozen; instantaneous supported payments/grants are separate |
| Hotori recording, reserved Skill rewind settlement, A3 refund duration and A3 ICD | Pause | Pause | Same world clock keeps the five-second recording window coherent with ally cinematics; resource conservation remains exact |
| Enemies/incoming enemy schedules and unrelated actors' in-progress action recovery/projectile delivery | Pause | Pause | Preserve remaining delays, without adding enemy AI; explicitly authored non-enemy scenario events retain their declared absolute basis |
| Current cinematic's own action progression and finite direct packet train | Advance for that cast | Hotori Burst tail/Katana advance; unrelated actions pause | A cinematic must be able to finish; exemptions are generation-owned, not global permissions for all same-owner effects |
| Hotori replay proxies' finite direct packets and immediate semantic applications | Pause if a separate cinematic dominates | Advance | Recorded actions must execute; resulting long-lived effects use their own rows, not permanent proxy immunity |
| Deployed field/summon existence lifetime and finite count | Pause | Advance | Selected object-lifetime default from the Japanese field observation, generalized explicitly; may expire without acting |
| Autonomous field/summon attack/heal production | Pause | Pause | No generic exemption for Edgar, Underboss, Officer Whisker or newly deployed fields; direct replay is distinct |
| Vita Bud existence/shot budget | Pause | Lifetime advances; actual shots consume the count | No unlimited bud lifetime or extra volley stockpile; applies with or without B2 |
| Vita Bud emission/delivery, without living Hotori B2 | Pause | Pause | Explicit passive is required; shots not emitted are not spent, but the bud can naturally expire |
| Vita Bud emission/delivery, with living Hotori B2 | Pause | Advance | B2 bypasses Hotori's active stop, not another Ultimate's cinematic |
| Hotori active Time Stop budget drain and finisher timeline | Setup uses existing reservation, with no early drain | Advance | Existing twelve-second owner budget must reach expiry and allow early finisher; it is not an ordinary buff clock |

For each timer, compose all applicable pauses: one active exemption cannot bypass a different non-exempt pause. Model world time as `R`; an object-existence clock advances during Hotori active time but stops during cinematics. A cinematic/replay action clock also composes its existing local hitlag, rather than discarding local phase or adding the same global pause twice. A newly created timer starts with its full remaining duration even inside a pause.

Independent buff duration and source existence are separate constraints. A field-granted buff can be removed when its source field expires even if its own duration clock was paused. Death, replacement, dispel and finite-count exhaustion remain immediate state transitions. Expiring an object cancels its parked emissions, including generations spawned during the stop; release never resurrects it or delivers a backlog.

**Scorch choice:** Pause both remaining tick phase and lifetime before and during the stop, regardless of application time. A valid replay may apply/refresh Scorch and cause an existing instantaneous Discord/reaction callback, but that is not an ordinary elapsed-time tick. Do not model the reported order-dependent ticking as an undocumented free-damage exploit. New evidence can revise the canonical policy later; the current choice is complete and executable now.

**Charge choice:** During Hotori active Time Stop, a qualifying Blossom/Remora interaction may retain its reaction observation but grants zero Charge energy and does not build a delayed grant or start a grant-dependent guard. Preserve unrelated, explicitly supported direct packet grants and resource payments; this is not a blanket prohibition on energy mutation during a permitted attack. Timed recovery remains paused. This follows the guide's specific statement as a disclosed assumption.

**Hotori resource choice:** Passive recovery and the Skill/A3 windows use world seconds. Her active stop drains its separate reserved budget in elapsed active-stop time, independently of local contact hitlag; a dominating separate cinematic pauses that domain. Another character's cinematic pauses passive recovery and rewind/refund clocks without resetting them. Her committed setup cannot regenerate or begin draining the active-stop budget early. Mode-scoped own buffs end by mode cleanup, not by inventing an extra paused lifetime after the mode ends.

**Missing evidence:** Choose these shared defaults for every otherwise unclassified ordinary timer. Use sourced owner overrides where available and record each real exception in consumed typed Rust bindings and model metadata. No TODO, missing recording, silently advancing legacy absolute deadline or blanket incomplete status remains merely because a timer uses a declared assumption. Truly unsupported mechanics, malformed inputs and exhausted execution budgets retain their existing failure behavior. Tests verify the selected model and must not be described as new observations of the native game.

**Alternatives considered:** A universal absolute-time policy contradicts the cooldown/buff/window observations. A universal freeze blocks the cinematic, Katana, replay and B2 buds from executing. Treating every newly applied DoT or summon as immune to Time Stop promotes inconsistent reports into an unlimited exception. A universal object-lifetime freeze ignores the specific deployed-field report; the matrix instead makes this lower-confidence distinction explicit and revisable.

### 3c. Preserve remaining timers in one causal scheduler

The clock-indexed queue retains already-due work at a pause transition: a zero remaining offset still settles in existing causal priority order, while positive remaining offsets park. A deployed producer's timer origin is its actual activation callback, not the earlier input that scheduled that callback. Baicang field and Iroi Dream activation rebase the pending lifetime and first periodic phase at birth, so pre-deployment cinematic time cannot extend the object. Expiry observation follows a permitted final equal-time emission where that owner's existing lifetime rule is inclusive.

Recorded replay uses a finite `ReplayAction` context. Its montage clock and admission controls are saved separately from the actual ally's current action, and finite descendants inherit that context across callbacks. Persistent buffs, resources and deployed producers retain the actual owner and their declared world/object domains. Another cinematic freezes the replay context; active stop alone permits its finite direct progress. Checkpoint identity retains this finite replay clock state, and completed contexts are retired after their queued finite work drains.


Store semantic deadlines and periodic phase in their clock domains, with policy/generation identity. While a domain is paused, park or unschedule its future delivery and retain the exact remaining offset; when it resumes, project that remainder onto the absolute scheduler. A carrier applied mid-pause is anchored at creation, not charged for the earlier paused portion. Preserve equal-time causal priority and refresh/replacement semantics. Settle any already-due work in the existing order at pause entry before suspending later progress; half-open endpoints remain exact.

Apply this to shared buff expiry/query dispatch, owner cooldown/readiness maps, actor recovery and pending child packets, target Break/status expiry, reaction tick generations, timed weapon/set triggers, source recording/refunds, shields, stamina and passive resource services. Existing direct `expires_at_us - current_time_us` and `ready_at_us <= current_time_us` consumers need either a declared clock-coordinate read or a correctly reprojected absolute deadline. Cover reads and delivery together so the UI cannot show a frozen timer whose effect has already expired.

Extend/refactor the existing pause service to consume this policy rather than layering new shifts over the current enemy-only queue rewrite. Use clock-indexed pending work and a single generation-owned delivery; a pause cannot generate per-frame readiness retries or scan/rewrite every unrelated event at every ordinary hit. Sample observations retain cataloged timer basis and exact remaining duration at actual state boundaries, so a seek within a pause can display the retained remainder. This remains observation data, not a substitute for a full scheduler checkpoint.

The shared cooldown map uses `ProjectedDeadlines`: semantic deadlines remain on world time, with cached absolute projections for the existing owner admission interfaces. An unpaused segment updates clock coordinates without visiting entries; a paused segment updates only future projections in that bounded timer map. New timers capture their own creation coordinate inside a pause. Expired entries are not revived, and projection overflow rejects atomically. The core API requires actual binary pause boundaries rather than accepting a mixed segment with guessed deadline placement.

### 4. Make clock semantics explicit in current contracts

Require a `clock_contract` identity for the revised request/report/source contract and update all current consumers together. Replace the overloaded public horizon with explicit relative target and absolute ceiling fields; retain absolute event `time_us` fields. Planned canonical elapsed fields are `relative_elapsed_us` and `absolute_elapsed_us`, with explicit `relative_duration_us` for the authored target and `absolute_limit_us` for the effective safety bound. Victory ignores the authored target while retaining its source declaration.

Detailed reports also retain compact mode-timer pause records: absolute start/end, owner/source generation, typed reason and evidence/hypothesis reference. Overlapping source records are unioned for time conversion; static reason/provenance text uses existing catalogs. A merged numerical interval index is derived once for lookup, rather than serializing a second event stream. Validate half-open bounds, nonnegative durations, clipping at actual elapsed time, interval union and `R + P = A`.

Distinguish cinematic and active Hotori pause kinds and carry the selected timer-policy identity in execution/model metadata. Timed state observations expose the carrier's cataloged clock basis and exact remaining duration/phase; an unresolved future absolute expiry is not invented during an open-ended pause. Validate paused cooldown/buff reads against the same state used by admission and delivery. Keep static timer metadata interned and unchanged paused state absent from event deltas, rather than adding repeated policy prose to every hit.

Compact batch observations need both elapsed totals and the applicable contract identity, but not complete interval histories. The first detailed sample and regenerated selected samples retain intervals for shading, seeking and native replay. Existing source occurrences, actions, delta state frames and RNG identity remain absolute and are not duplicated.

Update native JSON, exact exports, source artifacts/checkpoints, WASM endpoints, Worker types and stored report validation in the same change. Missing/obsolete clock identities get a readable unsupported-contract error. Keep the existing compressed container, exact rational encoding and 100000000-byte file/decoded limits; no legacy reconstruction or compatibility adapter is introduced.

### 5. Apply the relative denominator to whole-run damage consistently

For each seed, compute exact whole-run DPS as `total_damage_exact * 1000000 / relative_elapsed_us`. Actor, source and target rates, source experiments, comparisons, rankings and exports use that same seed denominator. Aggregate statistics summarize the per-seed rates; they do not divide average damage by average duration. Keep independent totals/contribution reconciliation.

Represent zero-relative-duration DPS as unavailable with a typed reason, while retaining completed combat, damage and both durations. It must not become infinity, zero, an epsilon-based number or an absolute fallback. A multi-seed DPS aggregate that requires an undefined seed rate is unavailable for the full requested set; do not silently omit that seed. Comparison/search eligibility checks consume this availability separately from whether combat completed. The UI must tolerate that state without discarding the raw sample.

EPS, HPS, SHP integrals and field time continue to use absolute elapsed time. Label their time bases in inspectable UI/export metadata. The Dur selector never changes any denominator.

**Alternative considered:** Let the duration-card selector recalculate DPS and support metrics globally. That would make a local display control change comparison semantics and contradict the confirmed scope.

### 6. Extend absolute aggregate timelines without unbounded per-seed bins

Remove clamping of hits beyond the requested duration into the old final bin and remove the time-mode chart's requested-horizon X bound. Native aggregation supplies absolute endpoints for both time and victory policies.

Use aligned absolute buckets beginning at zero and bounded adaptive resolution. Start at one-second buckets; if a completed absolute duration requires more than the existing 240-bucket maximum, double bucket width and combine adjacent buckets until it fits. Compact worker observations carry their bucket width; the collector aligns rows on the coarsest encountered power-of-two grid and coarsens retained rows as necessary. Merging preserves exact bucket membership at native microsecond boundaries and existing endpoint-inclusive final-hit handling. Promotion depends only on actual delivered endpoints, so chunking and worker scheduling cannot change the final result.

This keeps retained row storage bounded by existing per-seed bucket limits instead of allocating one bucket per absolute safety second for every seed. Each promotion conserves damage. At snapshot time, use the largest completed absolute endpoint, zero post-completion bucket damage for shorter trials and constant post-completion cumulative totals. Last-bucket widths are clipped to the actual aggregate extent.

The chart displays absolute time and absolute-bucket damage rates, labeled `damage / absolute s` in inspection. Those local temporal rates are distinct from relative whole-run DPS contribution charts. Cumulative endpoints reconcile with total damage even when absolute duration exceeds the authored relative target.

**Alternative considered:** Fix all buckets from the 3600-second safety ceiling. That would avoid rebinning but needlessly degrade ordinary 120-second chart resolution or inflate retained arrays.

### 7. Keep Sample geometry absolute and add clock presentation

Keep `usePlayback`, `createReplay`, `RotationLanes`, recent-hit cues and event stepping keyed by absolute timestamps and causal prefixes. The animation-frame loop advances absolute time at the selected speed; it is presentation only. Show labeled `Relative current / relative final` and `Absolute current / absolute final` readouts and shade clipped native timer-pause spans across the lanes. Relative display can stand still while the playhead and attacks continue normally.

Inspectors render native remaining cooldown/buff values in the declared clock basis. Seeking inside a pause retains an ordinary timer's remainder; a deployed object using the Hotori-active lifetime exception can still lose remaining life and expire. TypeScript must not infer these rules from character names or decrement every timer by the absolute cursor delta.

Add the Dur card's accessible upper-right Relative / Absolute control. A new run starts Relative; view navigation preserves the current run's card choice. All seven statistics come directly from the selected native distribution. Keep the choice independent of the playback cursor and aggregate charts. PNG export captures the selected card presentation while its attached data contains both distributions. Validate regenerated sample durations against the corresponding compact seed summary as well as existing content/engine/source identity.

### 8. Measure intrinsic extension and bookkeeping separately

For fixed permitted work, clock transitions update affected timer domains and their indexed projections; detailed receipt growth is proportional to actual pause records and changed timer observations. Per-seed duration statistics add scalar storage. This is bounded work per affected timer, not a claim that an arbitrary transition has constant cost. No timer-only frame polling, per-microsecond stepping or duplicate relative event catalogs are required. Pausing periodic systems can remove work during Hotori's stop, while a longer absolute run can add work elsewhere; measure the actual event mix.

A relative target can intrinsically execute more combat: 120 relative seconds plus 20 union pause seconds produces 140 absolute seconds. This does not predict an exact runtime or receipt-size percentage because event density, exact arithmetic and static catalogs vary. Benchmark actual events, wall/CPU time, retained memory, raw/compact/compressed bytes and playback seek costs, using the existing production browser/Worker harness. Retain baseline and changed receipts under `cache/` with source/artifact identity and four-CPU limits. Compare a no-Ultimate workload, the same bounded absolute event prefix, an ordinary multi-character Ultimate rotation and a continuing Hotori rotation reaching the relative target. Report extra events separately from per-event bookkeeping cost; do not promise constant total memory.

Retain the practical acceptance budgets and report limits documented in `docs/BROWSER_PERFORMANCE.md`. A size or execution limit remains a visible failure, never permission to truncate or promote an incomplete result.

## Risks / Trade-offs

- **Pause boundaries have incomplete native execution evidence** -> Cover every supported Ultimate, disclose selected release/default models including Zero and Hotori's handoff, and distinguish retrieved configuration from native countdown verification through independent lifecycle checks.
- **World timers and deployed-object lifetimes have conflicting observations** -> Implement the complete selected matrix with canonical assumptions; keep phase, existence and direct reaction callbacks distinct and leave no measurement-dependent TODO in the default path.
- **Absolute safety ceiling can precede relative completion** -> Keep caller bounds independent and return both progress values with an incomplete diagnostic.
- **A new pause occurs before a provisional stop deadline** -> Recompute at actual event boundaries, with tests around entry, early release, overlap and exact endpoint admission.
- **Report-contract change affects many consumers** -> Require one clock identity and update native analysis, source replay, Worker resume, generated types and import validation together; reject old data explicitly.
- **Zero relative duration can occur on an early kill** -> Preserve raw successful combat while representing relative DPS availability separately and preventing biased comparison aggregates.
- **Adaptive histogram promotion can lose tail damage or change with chunk order** -> Use aligned dyadic boundaries, conservation assertions and existing native aggregation/Worker tests with differing absolute endpoints and chunk orders.
- **Longer combat produces larger samples** -> Preserve compact batch mode, interval catalogs and existing hard bounds; measure actual production workload and export sizes before completion.
- **Old curated scores describe a different duration/timer policy** -> Include the clock/model identity in calculation fingerprints and visibly invalidate stale results; input edits do not silently trigger expensive recalculation.

## Migration Plan

Implement the current contract and all native, WASM, Worker and UI consumers in one coherent branch. Documentation is required work, not an optional cleanup after code completion:

- Update `docs/NTE_DSL.md` for relative `.time`/duration, absolute waits, paused cooldown/modifier queries and timer assumptions.
- Update affected OpenSpec requirements and references for execution, runtime content, Zankou, Studio and transport; this change includes deltas for explicit contradictory real-time Sakiri/Zankou lifetimes and 120-real-second curated defaults. Preserve complete existing requirement scenarios when synchronizing at archive.
- Rewrite the current normative clock contract in `docs/mechanics/shared/clocks-and-action-timing.md` and relevant statements in `action-timing.md`, `buffs-and-callbacks.md`, `reaction-and-break-harness.md`, `energy-and-gauges.md` and `energy-accounting.md` under that same shared directory. Keep real timestamps, support-rate integrals and local hitlag distinct from paused world timers.
- Update Hotori's resource/clock card and affected character timing cards, including Sakiri, Zankou, Edgar and any other owner whose documented unconditional absolute timer changes. Retain source URLs, field coordinates, observation limits, selected defaults and independent calculations in the existing owner references rather than creating a parallel findings ledger.
- Update affected DSL examples, report/Team list descriptions and performance acceptance documentation where their interpretation or measured receipts changes. Search live documentation and current specs for unconditional cooldown/buff/DoT real-time claims; repair the consumers, not only the central paragraph. Historical evidence remains identified as history rather than rewritten as a new measurement.

Include the clock/timer-policy identity in curated comparison fingerprints and mark obsolete calculations stale. Their existing explicit refresh/release workflows own regeneration; do not silently preserve old scores as comparable or auto-run a full catalog recalculation during planning. Update source/example guards needed to account for paused timers without fabricating energy or silently inserting waits into failed admission.

Rebuild affected native/WASM artifacts, run focused owner/aggregation/transport checks, exercise browser playback and report restoration, then run the stable-tree checkpoint and archive the completed change. No publication is included. Reverting the coherent change requires restoring its code, contracts and documentation together; old and new clock receipts are not interchangeable.

### Implemented scheduler projection boundary

The DSL checks exact timestamp units and supported declarations; shared native request validation owns the actual absolute ceiling. Scheduled exports use their default request ceiling, while live preparation validates the supplied caller ceiling, including explicit smaller or larger limits. The relative target does not cap absolute declarations. Thus an absolute interruption can occur after the numeric relative target when executed pauses extend the absolute run; World-clock enemy schedules still retain their delays through those pauses.

Live source execution stops after the last dummy's defeat and its same-time causal closure under either stop policy; its captured native replay retains that source boundary. Explicit raw time scenarios may continue their authored actions after defeat, preserving their existing ability to inspect delayed work, healing and target-unavailable callbacks. This distinction does not turn an unfinished live source into a successful comparison.

`simulation/timer_queue.rs` replaces the single content heap with clock-indexed heaps of semantic deadlines. Each live domain advances one scalar coordinate; frozen domains are excluded from selecting the next absolute event. The scheduler compares projected domain heads using the existing causal priority and sequence. Owner cancellation and hitlag reprojection preserve clock assignments when draining/reinserting events. Mode metadata stores finite owner/generation contexts, and current policy snapshots omit historical intervals. No tick polling or per-hit rewriting of unrelated queued deadlines is required. Owner APIs that consume absolute deadlines keep checked cached projections in `simulation/timer_state.rs`; BuffManager stores world coordinates. The old enemy-only park-and-shift service is removed so global pauses cannot be added twice.


### Action switch gates and endpoint teardown

The actor's shared switch cooldown uses World time. Character montage switch gates have a separate finite action deadline, so their own cinematic does not extend them; unrelated blocked action contexts retain the remaining gate. Virtual replay callbacks restore the real owner's admission gate. Chaos immunity and Blackbird's QTE gate follow the same finite action rule. Chaos's automatic Crime train instead retains its World phase anchor, and Hathor's independent Emergency/ATK carriers and saved cooldown use World time.

Deployed-object endpoint teardown uses SettlementAndDeath priority: after the final eligible damage/proc callbacks and before AfterSettledEvents source admission. It must not share Observation priority with replayed authored inputs, whose earlier enqueue sequence would otherwise change exact endpoint ordering between live source and native replay. Raw scheduled native requests retain their existing incomplete-prefix boundary for finite packet trains and explicit inputs. The explicit `continuing_tail` list covers maintenance callbacks formerly bounded by the requested horizon (for example interest, regeneration, summon teardown and buff expiry). These remain queued for clock projection and visible as pending, but do not require draining after the time endpoint. Ordinary periodic trains that previously exposed incomplete prefixes, including Adler, Scorch, Nightmare and Vile Ash, are not on that list. The existing exhausted source time-run path retains its separate completion rule.
