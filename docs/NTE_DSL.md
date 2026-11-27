# NTE DSL syntax

The script declares builds and targets, executes a rotation and measures combat damage. Fields and behavior marked **pending** define the accepted implementation contract, not current runtime support. Native `run-script` and the Rust/WASM `run_script` API interpret exact expressions and control flow separately for each trial on the combat clock. Native `compile-script`/WASM `compile_source` retain executable live source; Studio Run and seed selection use that same live executor; `compile_script` is the explicit scheduled API for structured-scenario consumers. See [native source execution](../presets/native/README.md). Unsupported requested inputs fail explicitly until their owning adapter exists.

Contemplative Cat reads one explicit party balance declared as `options start_fons=250000;`. The value is available as `.team.fons` and is compiled into the shared native scenario request. It is static for the run; no combat gain or spend is inferred. An absent value is allowed until a reached Contemplative Cat passive needs it, when execution fails with the owning diagnostic. Chiz Grains and Grain Loans remain separate resources.

Rust [script inspection](../openspec/specs/script-inspection/reference.md) returns the complete party with canonical IDs, exact static stats and initial resources without executing rotation instructions. Compiler diagnostics expose typed codes, one-based line/column and UTF-16 offset/length. Native CLI/live execution is completed in native workflow; full Studio integration follows in final browser integration.

## Grammar

EBNF: brackets mean optional, braces mean repetition, quoted tokens are literal.

```ebnf
script       = options { declaration | constant } { statement } ;
options      = "options" pair { pair } ";" ;
declaration  = character | weapon | cartridge | stats | target | active | hurt | incoming ;
character    = name "char" character_field { character_field }
               [ "+params=[" pair { "," pair } "]" ] ";" ;
character_field = "lvl=" level | "awakening=" awakening | "talent=" triple | "qte=" expression ;
awakening    = integer | "[" [ integer { "," integer } ] "]" ;
weapon       = name "add" "weapon=" name pair { pair } ";" ;
cartridge    = name "add" "cartridge=" name "count=" integer ";" ;
stats        = name "add" "stats" pair { pair } ";" ;
target       = "target" pair { pair } ";" ;
active       = "active" name ";" ;
hurt         = "hurt" "every" pair { pair } ";" ;
incoming     = "incoming" name pair { pair } ";" ;
pair         = identifier [ "%" ] "=" value ;
value        = expression | name | level | triple | range | boolean ;
level        = integer "/" integer ;
triple       = expression "," expression "," expression ;
range        = expression "," expression ;
boolean      = "true" | "false" ;
statement    = action_list | variable | constant | assignment | call ";"
             | conditional | while_loop | for_loop | repeat_loop
             | "break;" | "continue;" ;
action_list  = name action { "," action } ";" ;
action       = ( "attack" | "skill" | "burst" | "swap" | "special"
               | "dash" | "arc" | "plunge" | "jump" )
               [ "[" pair { "," pair } "]" ] [ ":" integer ] ;
variable     = "let" identifier "=" expression ";" ;
constant     = "const" identifier "=" expression ";" ;
assignment   = identifier "=" expression ";" ;
block        = "{" { statement } "}" ;
conditional  = "if" expression block { "else if" expression block } [ "else" block ] ;
while_loop   = "while" expression block ;
for_loop     = "for let" identifier "=" expression ";" expression ";"
               identifier "=" expression block ;
repeat_loop  = "repeat" expression block ;
expression   = logical_or ;
logical_or   = logical_and { "||" logical_and } ;
logical_and  = equality { "&&" equality } ;
equality     = comparison { ( "==" | "!=" ) comparison } ;
comparison   = sum { ( "<" | "<=" | ">" | ">=" ) sum } ;
sum          = product { ( "+" | "-" ) product } ;
product      = unary { ( "*" | "/" | "%" ) unary } ;
unary        = ( "+" | "-" | "!" ) unary | primary ;
primary      = number | time_literal | identifier | field | call | "(" expression ")" ;
field        = "." identifier { "." identifier } ;
call         = identifier "(" [ expression { "," expression } ] ")" ;
time_literal = number ( "s" | "ms" | "us" ) ;
number       = integer [ "." digit { digit } ] ;
integer      = digit { digit } ;
identifier   = ( letter | "_" ) { letter | digit | "_" } ;
name         = identifier ;
letter       = "A".."Z" | "a".."z" ;
digit        = "0".."9" ;
```

Configuration precedes rotation and cannot occur inside blocks. Fields may appear in any order. One options declaration and one active declaration are required. Keywords are lowercase; catalog names and documented aliases are used without quotes. LF/CRLF and whitespace are equivalent. # and // begin line comments. Unknown names, fields and duplicate keys fail. Engine paths, JSON objects, ability/effect IDs and user functions are outside the language.

Periodic `hurt` remains **pending**. Native source supports the owner-specific `parry` flag below. Fixed `incoming` and the documented live fields below are supported. `jump` and `plunge` execute for Baicang, Blackbird, Chaos, Chiz, Daffodil, Edgar, Fadia, Haniel, Hathor, Nanally and Zankou with the explicit airborne profile below; other owners remain pending. Boolean literals belong to typed flag parameters; numeric variables and logical query results retain numeric semantics. Documented flags also accept exact numeric0/1. Catalog names, target names and recipients resolve according to their parameter type, not as numeric variables. Two-value ranges are only accepted by scenario fields documenting them.

## Options

| Field | Default | Meaning |
| --- | --- | --- |
| iteration | 1000 | Positive integer trial count |
| duration | Required for time mode | Positive real combat-time horizon in seconds; optional victory safety horizon |
| seed | 42 | Nonnegative exact integer; trial i uses seed+i |
| mode | research | research or strict evidence policy |
| crit | fixed | fixed non-critical damage or random research rolls |
| stop | time | time runs to duration; victory runs until every target is defeated |
| start_fons | Explicit when reached by a Fons-dependent model | Nonnegative exact integer party balance, including explicit zero; shared by every reached consumer |
| stamina_capacity | Explicit profile, no default | Positive account combat-stamina capacity |
| initial_stamina | Explicit with the profile | Nonnegative starting amount, no greater than capacity |
| stamina_use_efficiency | Explicit with the profile | Positive multiplier of the source action cost |
| stamina_recovery_rate | Explicit with the profile | Nonnegative units recovered per real second; explicit zero is valid |

Supply all four stamina fields together. The optional `combat.stamina_v1`
model makes `dash` spend `15*stamina_use_efficiency` from one party account pool,
with a 0.9-second actor dodge cooldown and existing modeled 0.5-second recovery.
Each accepted spend refreshes a two-second recovery pause. Recovery integrates
exactly on the combat clock, continues off field and clamps to capacity;
rejected inputs do not spend or postpone it. The report records the profile,
exact current/spent/recovered amounts and each spend's command, actor and time.
The [shared script](../examples/stamina-dodge-cycles.nte) checks fractional
recovery across switches. Legacy unconfigured dash timing remains the earlier
partial model below; stamina-dependent owner branches must require a profile.
Held/channel/airborne consumers remain owner-specific pending work.

Nanally's paid `dash[critical=true]` now arms one following Normal as Critical
Riposte under `nanally.critical_riposte_v1`. Waiting preserves the opportunity;
another admitted action or field exit clears it. The authored `attack` remains
the source artifact, while runtime receipts/replay carry the actual response
packet and role. The [two-cycle source](../examples/nanally-riposte-cycles.nte)
declares the stamina account and explicit recovery waits. Critical Dodge and
Riposte do not themselves imply an assumed successful Parry.

In time mode, defeated targets stop receiving damage, without changing the DPS denominator. Victory mode reports elapsed time to victory; duration, if supplied, is a safety horizon and reaching it without victory gives an incomplete result. Script exhaustion keeps pending combat events running, but never invents additional inputs. Strict rejects unaccepted model hypotheses; both evidence modes reject unsupported damage dependencies. Documented irrelevant omissions follow the [DPS scope policy](../openspec/specs/product-scope/reference.md#dps-relevance-and-modeling-boundary); this does not enable currently rejected inputs.

The party Fons balance is readable as `.team.fons` when `start_fons` is declared. It is distinct from Chiz's combat Grains and Grain Loans. Combat changes to Fons require an owner-defined rule; no spend/gain is inferred from those combat resources. Missing Fons fails when a reached dependency needs it, rather than silently supplying zero. In victory mode, `.duration` exposes the independent absolute execution safety limit, including its default when no duration is authored. An authored duration does not replace that victory safety limit.

## Characters and equipment

| Declaration / field | Meaning |
| --- | --- |
| char lvl=current/cap | Legal character progression |
| awakening=X | Select first X awakening effects; X is 0–6 |
| awakening=[A,B,...] | Select only listed distinct effects 1–6; current level is their count; [] selects none |
| talent=normal,skill,burst | Three purchased talent levels; effective bonuses follow the selected effects |
| qte=X | Required purchased Support Skill level. X must evaluate to a legal positive integer for the declared character progression. Resonance bonuses apply separately. |
| add weapon=name refine=X | Equip one compatible weapon; optional lvl=current/cap defaults to 80/80 |
| options start_fons=X | Optional nonnegative exact integer; required when a reached Fons-dependent model needs it |
| add cartridge=name count=X | Equip set pieces; set bonuses follow their thresholds |

awakening=3 equals awakening=[1,2,3]. Selection order does not affect execution, and compiled requests sort effect indices while source roundtrips preserve the authored text. All 24 released characters accept arbitrary selections for their currently modeled actions and effects. Individual awakenings require their exact effect index, resonance_3 activates at three selected effects, and resonance_6 activates at six. This selection support preserves each owner's partial mechanics and does not complete unsupported effect producers. Multiple cartridge sets require legal total piece capacity and are **pending**.

Console/module geometry, layouts, automatic console stats and module specialization bonuses are outside simulator scope. The script supplies cartridge/module stat contributions through add stats; only declared 2-/4-piece set effects apply automatically.

Optional character +params initial state:

| Parameter | Default | Constraint |
| --- | --- | --- |
| start_hp | Full HP | Positive and at most projected max HP |
| start_hp% | Full HP | Fraction in (0,1]; mutually exclusive with start_hp; exact decimal conversion then floor against projected max HP |
| start_energy | Full energy | 0 through model-derived capacity |
| start_cycle | 0 | 0 through the nominal 100 Esper Cycle capacity; spent by qualified adjacent-element switches |
| start_timepiece | 0 for Hotori | Exact Timepiece units in 0..120; Hotori only, separate from Ultimate energy |
| start_grains, start_loans | 0 before combat-entry grants | Chiz exact savings/drawn principal within level/A1 capacities; separate from Fons |
| start_malice | 0 before entry | Blackbird exact Malice in 0..60; the living entry grant20 applies afterward |
| start_crime | 0 | Chaos exact Crime in 0..1000 |
| start_power | 0 | Hathor integer Power in 0..6; preload does not grant energy or count as earned Power |
| start_moon | 0 before entry | Iroi exact Moon in 0..100; A1 fills to100 at combat entry after the authored preload |
| start_plot | 0 | Haniel integer Plot Armor in 0..4, or 0..6 with A1; preload triggers no Harmony/Ensemble |
| start_bullets | 0 | Jiuyuan integer PowerBullets in 0..6; independent of energy/Cycle, preserved on switch and cleared on death |
| start_resonance | 0 | Daffodil integer Resonance in 0..2; persistent Support pool, without self Eyes or Phantom Step grants |
| start_string_energy | 0 | Akane Rin integer String Energy in 0..30 (R3 raises battle-entry floor to 30) |
| start_energy | char `0`, Shinku `60` | Initial Ultimate Energy (`0..capacity`, Shinku `0..60` Defiant Spirit) |
| start_<resource> (**other owners pending**) | Owner-defined starting value | Document each resource name, capacity and default in its character model |

`start_hp%` keeps its evaluated expression as an exact ratio and floors the product with final projected integer Max HP once. For example, `start_hp%=0.58` with 100 final Max HP starts at 58 HP. A positive fraction that floors to zero is rejected. Equipment and additive HP contributions are included before this conversion.

The shared Rust compiler parses decimal and time literals as bounded exact rationals. Arithmetic, comparisons, logical values and `min`, `max`, `abs`, `floor`, `ceil` and `clamp` retain exact results. Time converts to whole microseconds at the request boundary; resistance and summed additive stats convert to exact request ratios; `start_hp%` floors once against projected integer Max HP. Thus `0.1+0.2==0.3` is true, and `start_hp%=0.3-0.2` matches `start_hp%=0.1`. For bounded resource use, numeric literals are limited to 1,000 decimal digits and reduced numerator/denominator values to 4,096 bits; public numeric settlements remain in the safe-integer range. Integer fields, including equipment refinement/levels/piece counts and target HP/Max HP/DEF, reject a nonintegral exact result before conversion to a JavaScript number; a fraction that rounds to an integer in floating point is still invalid. Live per-trial interpretation preserves these exact results. Source parsing bounds nested constructs and expression sizes before recursive processing; arithmetic retains its own node/depth budget.

## Additional stats

Repeated add stats rows sum contributions. Omitted keys contribute zero. Already derived character, weapon, set and specialization contributions are not added again. Values are nonnegative finite numbers; ratios are fractions.

| Keys | Meaning |
| --- | --- |
| hp, atk, def | Flat additions |
| hp%, atk%, def% | Percentage additions; base × (1 + percentage sum) + flat sum |
| cr, cd | Critical rate and critical damage bonus additions |
| dmg% | General damage bonus, applicable to eligible damage of any element |
| anima%, chaos%, cosmos%, incantation%, lakshana%, psyche% | Bonus for damage of that element |
| ci, bi | Flat Cycle Intensity and Break Intensity points; repeated additions sum exactly |
| ce | Nonnegative additive contribution to the native Charge Efficiency multiplier; ce=0.12 adds 0.12. Applies to implemented numeric grants that support efficiency, with fractional carry. Ordinary hit generation executes for supported owner branches; enhanced forms, summons and reaction grants remain incomplete. |
| heal% | Outgoing healing bonus for Edgar Ultimate, Be Happy and Nacupeda healing callbacks |
| mental% | Additive bonus for eligible Mental damage; distinct from Psyche |
| shield% | Additive outgoing ShieldEfficiency contribution at shield grant; not incoming absorption or a capacity override |

Percentage `ci%` and `bi%` additions remain unsupported. Flat intensity additions
retain their separate native properties and are not damage-bonus percentages.

The static contribution is read from the healing source at each callback and adds to live outgoing healing bonuses. For example, `edgar add stats heal%=0.1;` adds 10% to Edgar's healing output without changing recipient incoming bonuses.

`mental%` and `shield%` use the shared native stat projector: their contributions sum with equipment into `DamageUpPsychicallyBase` and `ShieldEfficiency`. Repeated `mental%=0.1` and `mental%=0.15` add 25 percentage points; Quiet Manor's two-piece bonus makes the static Mental total 35%. General, static Mental and eligible live Mental bonuses share the additive damage-bonus zone. Repeated `shield%=0.1` and `shield%=0.15` with Kingdom's Guard four-piece produce 45% outgoing shield bonus, sampled from the granter at each modeled shield grant before one final round. Granted capacity then remains fixed. Unsupported damage/healing/shield producers still reject under their owner contracts; these stat declarations do not implement those producers.

General and all six elemental bonuses retain separate native identities. At delivery, eligible packets receive dmg% plus the bonus of their own element, including delayed/off-field hits. Other-element additions remain available for packets of that element. Physical/mental packets retain dmg%; true damage bypasses the bonus zone. Zero CR/CD additions preserve innate stats. Unavailable stat mechanics fail explicitly.

## Targets

Targets receive t0, t1, etc. in declaration order. Manual targets require hp, def and resist.

| Field | Default | Meaning |
| --- | --- | --- |
| hp | Required | Positive starting HP |
| max_hp | hp | Maximum HP, at least hp |
| def | Required | Nonnegative total authored defense |
| base_def | def | Optional nonnegative integer base DEF for base-scaled target debuffs |
| original_atk | omitted | Optional nonnegative integer initial dummy ATK, required by reached Haniel B2 Nova extraction; no incoming damage is inferred |
| control_immune | false (scenario assumption) | Explicit boolean dummy immunity to modeled owner control, independent of `boss` |
| resist | Required | Common resistance ratio |
| boss | false (scenario runner default) | Explicit `true`/`false` Boss assumption for this target; omission means false only in the runner, not native non-Boss classification |
| lvl | Model-dependent | Level required by level-dependent mechanics |
| anima, chaos, cosmos, incantation, lakshana, psyche | resist | Optional element-specific resistance overrides; omitted elements use `resist` |
| break_gauge, max_break_gauge | None | Explicit exact current/maximum Break gauge; require both plus `broken_duration`, with positive maximum and current in `[0, maximum]` |
| broken_duration | None | Positive Broken duration in seconds, exactly representable in whole microseconds; required with the two gauge fields |
| discord_fraction | None | Explicit fraction in `(0,1]` of original capacity removed by a reached Scorch/Nova overlap; requires the complete Break profile and is mandatory when that breakable scenario reaches Discord |

For example, `target hp=10000000 def=1080 resist=0.2 break_gauge=1/3 max_break_gauge=2/3 broken_duration=3;` supplies the bounded [content Break model](mechanics/shared/content-break-model.md). Omission keeps the dummy without a modeled gauge. A partial profile rejects. Targets have no native identity, pack or profile lookup; all stats and assumptions are explicit script inputs. For `target hp=10000000 def=1500 base_def=1000 resist=0 control_immune=false;`, a successful modeled Sakiri Suppress can apply its 10% base-DEF reduction, leaving 1400 DEF. Control does not add movement or target AI; immunity prevents successful-control receipts and their conditional effects.



## Rotation

| Input | Meaning |
| --- | --- |
| character attack | One ordinary attack input |
| character skill | Skill input |
| character burst | Ultimate input |
| character swap | Switch to that character; same-character swap is a no-op |
| character special | Press the special/Aim button (G); the character model owns its effect |
| character arc | Activate the equipped weapon's supported Arc (Silent Garden, Eclipse, Justice Executioner or Black Tome); the Rust owner supplies costs, cooldown, packets or field lifecycle |
| character dash | Dash input with the modeled 500ms research recovery; `dash[critical=true]` explicitly assumes Critical Dodge success for reached owner effects |
| character plunge | Supported owners listed above require explicit `airtime` and `height_bonus`; other owners remain unsupported |
| character jump | Native grounded Jump/cancel for Akane Rin, Baicang, Blackbird, Chaos, Chiz, Daffodil, Edgar, Elyms, Fadia, Haniel, Hathor, Nanally and Zankou; requires the owner's current input gate and preserves costs/pending effects |

Comma-separated actions run left to right. :N repeats an input N times; N is a positive integer literal. attack:5 is five consecutive attack inputs; the character model handles the chain. An off-field action implicitly swaps first. Initial active selects a character without a swap event. Native models own costs, cooldowns, recovery and all resulting damage/effects.

`special` currently executes for Adler, Elyms, Nanally, Jiuyuan and Lacrimosa; other owners fail explicitly. For the first four, `special` enters persistent aiming and each `attack` fires one native aimed shot. A second `special` exits aiming. Admitted `dash`, `skill`, `burst` and field exit also exit aiming; waits and same-character swaps preserve it. These mode transitions are disclosed hypotheses pending native transition evidence.

`dash` is a non-damaging input, admitted with zero energy cost and a modeled 500ms recovery. `dash[critical=true]` records an explicit assumed-success Critical Dodge and dispatches reached owner effects; it does not invent dodge timing windows, Riposte packets or dash damage. The Critical Dodge and input timing assumptions remain runner policies, not native behavior.

Lacrimosa starts in the existing ranged Tomato Percussion model. Each `special` deals Flavor Variation damage and toggles between ranged and melee forms. `attack` selects the current form's five-stage chain, and `burst` selects the car or truck packet train. Dash resets the chain but preserves form; skill, burst and field exit preserve form. The melee chain models one stage-two projectile explosion on uninterrupted grounded stage three, under an explicit nearby-target hypothesis. Model timings and initial form remain disclosed hypotheses.

Lacrimosa's ordinary `skill` has one16s cooldown in the disclosed caller-timed
source profile. Five admitted Normal sections during that cooldown earn one
Sleep In extra `skill`; ordinary `skill` automatically consumes it without
restarting the original timer. Multiple hits/targets do not multiply the
counter, and further sections cannot earn a second extra in the same window.
Unused permission expires with the ordinary cooldown. `.lacrimosa.skill.ready`
and `.lacrimosa.skill.cd` expose current admission eligibility, including the
earned extra. There are no public stock controls or invented charge resources.
The [continuing example](../examples/lacrimosa-sleep-in.nte) retains ordinary
Skill deadlines0-to16 and16-to32 while extra casts occur at5.2 and21.2.
Native montage/notify timing and interruption remain a separate owner binding.

Lacrimosa `skill` and `burst` also activate the [shared owner Nightmare model](mechanics/characters/lacrimosa/nightmare-damage-and-duration.md#multi-target-execution-extension). Public descriptors attach its disclosed scheduling, all-target and fixed-critical-probability assumptions. Blood uses the effective Normal talent independently of Skill/Burst talents, applies at impact to living declared dummies, and continues off field. No equipped weapon or extra source directive is required; owner-specific unsupported branches remain outside this model.

Zankou's Crimson Reverie initializes Cycle to100 once and grants only herself100 CI points. Normal/held, Skill and Burst inputs derive actual Reality/Illusion actions from continuing owner state. A legal held entry is `zankou attack; zankou attack[hold=200ms]; zankou skill;`. Skill has a five-world-second cooldown; entry/return and contact pauses use the selected native montage clock. Enhanced Soulcross requires Illusion's final3.5 paused-world seconds. It unlocks Bloodfeast, whose completed activation grants one free Inferno; A4 also unlocks Bloodfeast when the paid Ultimate is ready. Both Ultimates remain ordinary `burst` inputs. Heartwrench and Vile Ash retain their separate ten-stack trains; ordinary Illusion attacks spread at A0, and A1 additionally enables Reality/counter/enhanced-Skill spreading. See the [owner contract](mechanics/characters/zankou/action-damage.md).

```text
nanally special, attack:4, dash, attack:3;
lacrimosa special, attack:5, burst, special, attack:5;
```

The research adapter schedules `special` with 800ms recovery and `dash` with 500ms recovery. Without the explicit stamina profile above, both retain zero energy cost and no cooldown under `inputs.special_dodge_timing_v1`; this historical partial model supplies no stamina-dependent owner permission. Configured dash uses the shared stamina cost/cooldown model. Animation cancellation, invulnerability and dash damage remain unmodeled. Player-input commands carry no caller timings, coefficients or desired mode. A failed admission preserves the mode and reports the blocked input. There is no `attack[mode=ranged]` override.

Action parameters (owner support is required):

| Parameter | Meaning |
| --- | --- |
| target=tN | Select the aim target, default t0; modeled area hits still cover eligible targets |
| window=attack/hold/jump/dash/skill | Static input-kind identifier on attack/skill/burst/dash/plunge. Yield the producer at its native input gate; omission uses full recovery. This neither cancels the producer nor admits the following input |
| plunge[airtime=expression,height_bonus=expression] | Supported owners: positive time in seconds to landing and exact dimensionless height bonus in0..1; both required. Mint additionally requires a new admitted `jump` before each flight. No geometry or guessed jump velocity is supplied |
| hold=expression | Exact nonnegative hold/channel duration in seconds on an ability or owner-supported entry switch; `hold=1.2s` is a duration, not a flag, stage or frame count. Omission/zero is tap where supported; the owner validates duration bounds and derives stages, costs, hits and cooldown. Native live execution retains the typed duration on each actual command and observes owner recovery. Nanally's continuous held Normal is covered; unsupported reached owner bindings diagnose |
| moving=expression | Movement intent for a held ability, requiring an explicit `hold`; boolean or exact0/1. Native owner admission selects the branch and checks its prerequisites. This does not model distance or geometry |
| approach=expression | Mint `attack` only: boolean or exact0/1. True selects the native approach prefix in place of Normal1 and advances to2. False/omission uses the current grounded chain or pending entry prefix; no movement distance is inferred |
| stocks=expression | Whole per-input reservation for an explicit `hold`; Chiz's native held Skill accepts2 or3. Omission keeps her native two-stock route. This reserves existing stocks and does not set/refill the initial pool |
| launchable=expression,landing=expression[,height_bonus=expression] | Hathor held `attack` or explicit entry `swap`; requires `hold` and both choices. `launchable` is boolean/exact0 or1; `landing` is positive real seconds from admission. Height bonus defaults to0 and accepts0..1 for the nonlaunchable falling route. These explicit dummy assumptions do not imply geometry or motion |
| parry=expression | Assume success after an eligible admitted input: paid `dash` for Blackbird/Hathor/Haniel/Mint, or a distinct charge-backed Daffodil `swap`. Omission/false adds no parry assumption |
| dash[back=expression] | Explicit Back Dodge when true, Front Dodge when false; omission preserves native Front. The owner must support the selected direction and native model. Direction does not supply Critical Dodge success or waive stamina, cooldown or recovery |

`window` follows [ADR 0018](decisions/0018-authored-input-windows.md).
The chosen family uses a read-only native gate and recomputes after real queued
events, including contact hitlag. Without an early override, ordinary recovery
remains. The next authored input still checks native permissions, cooldowns,
resources, field ownership and costs. No retry, hidden readiness wait, frame
constant or automatic cancellation is introduced. A `jump` does not supply
Plunge's explicit airborne/landing profile or model geometry.

Native Mint scripts initialize the complete owner clock before initial entry,
including when Mint begins off field. Ordinary Normal/Skill/Burst, held Normal,
qualified Support, entry/approach, paid Dodge/Riposte and Jump/landing use the
same engine gates and continuing accounts. Skill's6s cooldown starts at its
reached notify; Burst pays100 energy and starts15s cooldown. The
[equipped continuing example](../examples/mint-native-cycles.nte) retains two
periods with Mind Royale and Fireflies through this path.

Mint `jump` opens one flight; `plunge[airtime=500ms,height_bonus=1/2]` declares
its landing. Before landing, another grounded input rejects unless an explicit
interrupt or field exit clears the flight. The conservative model does not
infer an ordinary Jump's landing. Plunge additionally accepts `hold=seconds`,
measured from plunge admission; it must outlast flight and the native landing
hold threshold to enter Hold. Default completion retains that pending branch
and the real End recovery, including remaining caused pause.

Mint `window=jump|dash|skill` yields at the native family gate. Full recovery
can precede a later movement gate, so `mint skill[window=dash]; mint dash;`
explicitly yields to Dodge admission. `dash[critical=true]` arms only one
replacement Riposte; `dash[parry=true,window=skill]` instead requests Back after
20ms and yields at its native Skill gate, without a free damage response or
second stamina payment. Mint has no second-input `window=hold` continuation;
its duration belongs to the original input. Back Jump conservatively waits for
full recovery. Source tenacity/incoming interruption declarations remain Q4-L5.
Approach selection and landing hold require native live execution; the frozen
explicit scheduled API rejects these choices; Studio Run executes them live.

Chaos also supports `window=hold`. Its native combo predicate requires the
held duration to fit inside the preceding montage: minimum200ms for ordinary
Normals/Plunge,250ms for an Appear Normal, and200ms/100ms for continuation of an
ordinary/transformed full Verdict. The window yields at the minimum legal
native gate; a longer authored hold still has to satisfy its own gate. It does
not insert an idle lead-in for a combo. A short hold without an eligible live
producer rejects; an idle Verdict still requires at least1s. Full Verdict's
continuation source can outlive ordinary next-action recovery, and preserves
its captured form predicate while the new branch selects the live form. The
[two-cycle script](../examples/chaos-verdict-continuations.nte) funds and spends
1000 Crime twice, preserving native recovery, hitlag and later resource state.

Daffodil's `window=hold` observes the same Normal Jump gate as her native
held-combo admission; outside an unfinished Normal it retains ordinary
recovery. Her ordinary source model also enables paid Displaced Edge without
requiring an unrelated held input: Front Dodge at its native gate suspends
the existing Normal and later delivers only its remaining contacts. Back
Dodge uses its separate gate and cancels the Normal. A `window=dash` observes
the default Front gate; it does not promise that a following Back Dodge is
already legal. Jump during Displaced Edge releases the remaining suspension
and cancels unreached Normal contacts. The [continuing source example](../examples/daffodil-input-windows.nte)
exercises held replacement, suspension and Jump cancellation twice, preserving
one paid stamina account and native replay.

Chiz's explicit `attack[hold=0]` selects tap Barren, retaining its2.5s
admission limiter; omission is still an ordinary Normal. A positive
`attack[hold=200ms]` selects combo-held Barren after a live Normal continuation.
Its duration is the native branch choice rather than added charging time.
The locked first request excludes that dummy from the secondary request;
the final AoE still reaches every living eligible dummy. The [Barren script](../examples/chiz-barren-inputs.nte)
uses both routes across two periods with an explicit lock on `t1`.

Chiz's held Skill can yield with `window=skill` for a separate paid third
`skill` input. The existing native model buffers that third input until the
held-loop gate, retains its command identity and spends its actual stock;
it does not cash out the unused Grain remainder. The [buffered Full script](../examples/chiz-buffered-held-full.nte)
continues two funded loops, exact outstanding loans and3s stock refills.
Ordinary tap `skill[window=skill], skill[window=skill], skill` likewise
uses native continuation gates and cancels unreached Half packets.

The [Blackbird window script](../examples/blackbird-input-windows.nte) performs
two ordinary Roostcalls, then two continuing Grace-to-Tidings replacements.
`attack[window=hold], attack[hold=200ms]` pays the real hold after yielding;
`hold=0` is still a tap and fails if ordinary recovery prohibits it. Other
families use the ten implemented owner control profiles above; native branch
permissions remain owner-specific. `SourceOccurrence.action_window` records
the choice; `completed_us` is its source yield time, while pending montage or
spawned packets may continue. Saved source roundtrips and native replay retain
the actual input timestamps. The explicit scheduled API rejects these windows; Studio Run executes them live.

`target=tN` is a typed dummy identifier in declaration order, beginning at `t0`.
It is neither a numeric expression nor a variable/string alias. Unknown or
malformed identifiers diagnose at the parameter expression even in an unvisited
branch. Each repeated input retains its own selection after any implicit switch;
an explicit targeted `swap` also supplies the aim for automatic eligible Support.
Omission selects `t0`. This selects the primary and owner-locked continuations;
area coverage, recipient caps, retargeting and autonomous selection remain with
the owner. An explicit targeted switch also supplies the locked target for its
qualified Cycle reaction and the owner entry selector. Scorch/Nova generations
and Hexed/Stain/Remora statuses retain that recipient; Blossom's autonomous
pistil selection and area coverage remain unchanged. `.support.ready` inspects
the default `t0` context; admission rechecks an explicitly selected different
dummy. [Source target checks](../crates/content/src/simulation/tests/source_targets.rs)
retain native replay/roundtrip, ordinary trains for all23 owners and Edgar's
single-target channel, two paid generations of all six Cycle pairs, independent
Remora removals on two dummies and entry while only the selected dummy lives.
The Plunge checks also retain both area recipients when the primary is `t1`.

Except for Mint's and Zankou's explicit Jump prerequisites, Plunge declares airborne
entry through its positive landing time; it does not
insert a Jump, infer geometry or set an owner combo stage. The selected owner
clock determines landing contacts, recovery, AoE and resource opportunities.
Hathor additionally requires her explicit positive shared Strength profile and
retains native airborne spending. [Source Plunge checks](../crates/content/src/simulation/tests/source_plunge.rs)
exercise two continuing inputs across two dummies for Blackbird, Chiz,
Daffodil, Edgar, Fadia, Haniel and Hathor, retaining independent first-contact
times and complete native replay. Blackbird selects her ordinary or transformed
airborne row from the same settled owner form as her other source actions.

The optional `critical=true` dash flag defaults to false. It supplies the missing external success condition in a dummy-target scenario; it does not waive input recovery, owner restrictions or create arbitrary damage. Reached effects and the assumed-success provenance appear in the input receipt. Critical Riposte remains a separate owner transition and is only modeled where an owner contract supplies its legal window and packet.

`moving`, `back`, `critical` and `parry` accept boolean literals or exact numeric0/1, including
logical expressions and numeric variables. Other numeric values and text fail.
Closed invalid flags diagnose at their literal/operator UTF-16 range during
compilation; dynamic invalid flags fail at the reached input without leaving a
command or orphaned direction entry. Live execution evaluates flags for each
actual repetition. The saved native source artifact and explicit scheduled export
both preserve explicit Front/Back choices, keyed to the actual command after an
implicit switch. [Rust source checks](../crates/content/src/simulation/tests/source_directions.rs)
exercise two paid Critical Dodge/Riposte cycles for Edgar, Hathor, Fadia,
Blackbird, Chaos, Chiz, Daffodil and Haniel, a live direction change, native
replay, saved-source roundtrip, rejected owners and source ranges. Authored dash
inputs select these supported native secondary clocks before initialization;
the default Front direction uses the same model and requires the same explicit
stamina profile as an explicit direction. A direction flag does not supply an
initial resource or an assumed successful Critical Dodge.

Chaos's party projection initializes native ordinary/form clocks, Crime,
Verdict, secondary actions, qualified Support, Warrant transfer and B2 Remora
models. `burst` spends100 and enters twelve World seconds of Dread Echo at
2.718888s; final-inclusive Crime ticks retain their real .2s cadence. Native
`attack` selects the settled ordinary/transformed rows. The
[continuing A0 script](../examples/chaos-native-cycles.nte) funds two Bursts
from zero energy and four1000-Crime Settlements without resetting resources;
captured transformed Verdict stages continue after natural form expiry.
Gold Wool grants at successful Skill recovery and refreshes at Burst commit.
Source cancellation windows and shorter combo/continuation holds remain Q4-L3.

Chaos `attack[hold=1s]` declares the selected neutral Verdict model and its
ordinary clock/Crime dependencies. Shorter nonzero holds still need a supported
owner combo/continuation window; zero remains a tap. Daffodil's explicitly held
Normal selects her nine-contact held montage, including when the authored hold
is zero: her owner binding uses presence of the held input, not a shared heavy
threshold. Haniel `attack[hold=0.2s]` selects her ordinary clock and charged
Cannon; holds below0.2s are taps. Native Plot Armor, launch timing and ricochet
selection remain owner-owned. These declarations occur before initial state,
without editing request maps. [Source hold checks](../crates/content/src/simulation/tests/source_moving_holds.rs)
retain two continuing held inputs, implicit entry, independent first-contact
delays and whole native replay. Blackbird's unfinished-montage held branches use the explicit input windows
above; remaining owner continuation/cancellation choices stay in Q4-L3.

Daffodil's party projection initializes her native clock, cancellation,
secondary and Phantom Step models. Ordinary `burst` pays120 energy and applies
the owner's separate Eyes, two-charge handle and target Insight at their native
events. An ordinary return can select the charge-funded Step; authors do not
name its packets or set its stages. The
[continuing A0 script](../examples/daffodil-skill-burst-packets.nte) retains two
paid Finales, chains/Skills, actual Zero Supports and two Phantom returns across
two dummies. [Source checks](../crates/content/src/simulation/tests/source_resonance.rs)
preserve native entry/Finale contact times and direct replay; the exact30s
handle-expiry check also runs without internal overrides. Charge-backed entry
parry is source-bound; remaining cancellation admission stays in the queue.

Haniel's party projection initializes the ordinary clock, Cannon, full-kit,
Hootie and Nova-extraction models. `skill` retains its20s cooldown from the
.684620s notify; `burst` pays120 and starts Ace at2.869166s. Speaker time pauses
only during actual Ace. An appear is a queued Normal1; an explicit positive
idle, such as `wait(1us)`, ends that queue before a new neutral Cannon.
The [continuing A0 script](../examples/haniel-continuing-energy.nte) funds two
Bursts from zero energy, retains Plot0 then4 for its fast Cannons and exercises
Ensemble/Symphony/extra packets across two dummies. A200ms early release during
Ace contacts at.2+.400001+.044596=.644597s. Source checks retain real Hootie/Ace
deadlines and native replay. A reached Nova extraction requires declared dummy
`original_atk`; the source adapter supplies no guessed target ATK. Parry during
a paid Dodge is source-bound; remaining cancellation inputs stay in Q4-L3/L5.

Blackbird's party projection declares the selected ordinary/evolved clock,
Malice, coordinated bird, transformation and qualified Support models before
initialization. Native source `attack`, `skill`, `burst` and `plunge` select the
current form's native ability and packet through the owner registry. Skill and
Burst readiness/cooldown queries use that same selection, including earned
Evolve Skill permission and the paid finishing Burst. Ordinary Skill retains
the selected16s cooldown. The executed native request saves each actual input
identity for replay; authors keep ordinary action names. The live Studio Worker uses the same source execution path. [Source form checks](../crates/content/src/simulation/tests/source_forms.rs)
retain two continuing transformation/finisher periods, two full evolved Normal
chains per period, evolved Skill/Plunge,400 total paid energy and native replay. The existing
five-stage query/timeout check also runs directly from source without internal
request or descriptor edits.

Baicang's held Normal binds `moving` to his native post-Dodge Formless/Unrestrained
selection. Moving intent requires a real preceding admitted dash and the native
heavy threshold; a flag cannot invent that permission. Parameter order is
irrelevant. [Rust source checks](../crates/content/src/simulation/tests/source_moving_holds.rs)
retain two paid Dodge/heavy cycles across two dummies and sixteen branch
deliveries. Their two connected inputs carry into the following third Normal,
which generates one Word and pays one HP cost; target/packet multiplicity does
not multiply the input count. Native replay and scheduled typed export agree.

Chiz's ordinary Skill selects her existing native finance/pressed/held models
before initialization, including default zero savings/debt and three stocks.
`skill[hold=10s,stocks=2]` keeps the finite two-stock loop;
`skill[hold=10s,stocks=3]` reserves the extra stock for native held Full.
Native admission checks capacities before payment. [Rust source checks](../crates/content/src/simulation/tests/source_stocks.rs)
exercise both routes across two dummies and two continuing cycles, preserving
stock refills, the existing15000 loan debt and explicit shared Fons. Stock types,
missing duration and native capacity errors remain distinct; scheduled export
retains the typed reservation. Initial-stock authoring remains Q4-L4.

Hathor's `attack[hold=1.5s,launchable=true,landing=2s]` and
`attack[hold=1.5s,launchable=false,landing=3s,height_bonus=0.5]` select her
native Branch and falling fallback. The reached Normal stage owns its hold
gate, delivered/undelivered contacts, landing impact, recovery and paid fallback
stamina. `hathor swap[hold=0.6s,launchable=true,landing=2s]` attaches the same
choices to an actual entry switch. An implicit switch preceding `attack` keeps
the choices on the attack; a same-actor swap remains a native no-op. These paths
require the explicit shared stamina profile. Hathor's party projection selects
her native clock/Power/Gaze/held Skill/hybrid Burst/Cyclone/secondary/Remora dependencies
before initialization, including off-field startup without an owner instruction.
Declaring a model does not declare proximity or an incoming attack. Ordinary
no-hold actions use the same selected native clocks; Branch adds its AirAttack
and Branch dependencies only when authored.
[Rust source checks](../crates/content/src/simulation/tests/source_branch.rs)
retain both Branch routes across two continuing cycles/two dummies,40 total
fallback stamina spent, entry binding, native replay and scheduled typed export.
They additionally cover two complete finite five-second held Skills. Native
owner checks retain independent packet coefficients, timing and resource values.

[Ordinary source checks](../crates/content/src/simulation/tests/source_directions.rs)
retain two five-stage Hathor Normal chains (26 contacts; second-chain first
contact8.441361s;44 energy plus301/500 carry), implicit entry and two paid
Front/Back Critical Dodge/Riposte cycles. The native five-second Riposte reward
carrier admits one Power reward for closely spaced cycles and two after longer
authored waits; neither the input flag nor a switch resets it. Power initially
reads native zero without requiring an explicit start field.

Native source `dash[parry=true]` supports Blackbird, Hathor and Haniel with the
same explicit paid stamina and native Dodge admission as an unflagged input.
It does not set `critical=true`. Ordinary Blackbird receives the existing20
Malice grant; transformed Blackbird receives one30s Goner permission, consumed
by the actual Skill. Hathor/Haniel use their existing following-Riposte transition.

Daffodil `swap[parry=true]` requires a distinct eligible entry with a live Phantom
Step charge. It retains native Step or qualified Support selection, one charge
consumption, conditional finisher/recharge and actual extended Support recovery.
A no-charge, same-character or otherwise ineligible flagged entry rejects before
switching. The [two-cycle example](../examples/daffodil-parry-cycles.nte) retains
two120-energy Bursts and four charge-funded counters on two dummies.

The assumption is bound to the actual accepted command, including after an
implicit switch and within repeaters. Its native callback runs at that input's
admission time before the next sequential input. Saved native replay retains
that causal reference, receipts and source locations without inferring incoming
damage. Unsupported owner/action pairs and closed invalid flags diagnose even
in unvisited branches. Scheduled Studio export awaits its browser integration migration.

The parry flag defaults to false. It supplies the missing external success condition in a dummy-target scenario; it does not waive action costs, transition windows or owner restrictions, or create arbitrary damage. Unsupported owners/parameters fail. Assumed successes retain source locations and explicit scenario provenance in results. No separate `qte`, `riposte` or `parry` command is required. Support Skill eligibility and automatic dispatch on a qualifying switch are native combat mechanics; native QTE identities remain internal to those owner bindings. `.support.ready` is a read-only query of that mechanic, not a command to trigger it.

The executor submits each action or switch to the shared combat engine and respects its admission result. Transition windows, the departing actor's cast and eligible Support Skills follow the [combat switching rules](mechanics/shared/actor-lifecycle.md#combat-switching-mechanics). A qualifying reaction switch into Sakiri admits her Support Skill with 0.6 seconds of recovery. Support Skills have no separate cooldown. An implicit switch dispatches through the same engine path, and the following input remains subject to its ordinary admission rules. A same-character swap leaves combat state unchanged.

Character-specific choices use documented typed bracket parameters with defaults. No caller damage coefficients, cooldowns, animation timing overrides or generic proc objects are accepted; hold duration is a player choice, not a recovery override.

Automatic Break and manual scenario inputs share the native target gauge/lifecycle:

```text
target t1 break_gauge=29.5;
target t1 break;
```

`break_gauge` sets an exact nonnegative absolute gauge, at most its live value;
`break` sets zero. Both require a declared `break_gauge`, `max_break_gauge` and
`broken_duration` profile. Invalid targets/missing profiles and closed invalid
scalars diagnose even in unvisited branches. Increasing the live gauge, changing
an already Broken target or changing a defeated target rejects the occurrence.
The expression samples live state when reached. These external scenario inputs
consume no combat time, actor resources or cooldowns, and do not wait for actor
recovery. They settle after existing same-time events, preserve pending attacks,
and neither apply outgoing Break Efficiency nor count as actor gauge damage
for Decomposer. Zero enters the ordinary Broken callbacks, living-party Break
contributions and authored recovery lifecycle. See the executable
[three-cycle example](../examples/manual-break-cycles.nte).

 Damage-relevant shield, summon and reaction effects trigger automatically when their owning adapters exist; unsupported dependencies reject. Irrelevant omissions retain scoped model notes. Any needed manual trigger must bind to an existing owner model and a documented player/scenario choice; no universal fake-effect command.

## Scenario damage (fixed incoming implemented; periodic hurt pending)

These declarations precede rotation and schedule independently of the interpreter's blocked input:

```text
hurt every interval=100ms,300ms amount=100,300 actor=active;
incoming adler at=250ms damage=1000;
```

Fixed `incoming <declared actor> at=... damage=...;` declarations are executable.
They precede rotation, accept configuration constants/exact arithmetic and time
literals, and require both fields. `at` is a nonnegative whole-microsecond
absolute time within the independent execution ceiling; `damage` is a nonnegative exact
safe integer (including zero). Events sort by time, preserving declaration
order at equal times. Named recipients remain fixed after switching. The
compiled `incomingDamageLocations` array follows the sorted event indices,
which are retained by report/replay receipts. The existing explicit incoming
settlement hypothesis is included in the compiled request. No damage is
inserted as an ordinary action or inferred from a dummy's behavior.

Periodic `hurt every` remains unsupported by the compiler; its contract below
specifies the later binding.

`hurt every` requires `interval` and `amount`, each a fixed value or inclusive two-value range. Intervals are positive whole microseconds expressed in seconds/time literals; amounts are nonnegative exact integers. `actor` defaults to `active`, resolved at delivery, or names a declared actor. Draw bounded uniform interval/amount values from a seeded scenario-owned stream, with the next interval measured from the preceding event; fixed values need no draw. `incoming` names a declared actor and requires a nonnegative representable absolute `at` and integer `damage`. Both supply delivered damage before shared shield/effect absorption and HP loss; enemy mitigation/AI is not inferred. Event ordering, deaths, thresholds and proc recipients use the existing incoming-damage owner model. Periodic damage does not guarantee a parry/dodge window; use explicit action flags for an assumed successful trigger. Retain the declarations, sampled events and seed provenance for replay.

## Time

`wait(expression)` advances by an explicit interval; `wait_until(expression)` waits until an absolute timestamp and is a no-op if already reached. `wait_for_energy(integer)` suspends the authored suffix until the active actor's settled whole-energy pool reaches the declared threshold, which cannot exceed its capacity. Native events continue during waits. Under `stop=time`, the simulation completes at the exact relative duration even during an action or ordinary wait; later source inputs and expressions are not evaluated, and later effects stay pending without damage. Rotations need no remaining-time reserve or special final tail. An unreachable explicit energy threshold or the independent absolute safety ceiling still leaves a source-linked incomplete execution. Live branches and multiple energy gates compose in the same lexical program. Time values use seconds; s/ms/us suffixes convert to seconds. Negative/nonrepresentable time fails. Native CLI, Studio Run and selected seeds share the `relative-combat-v1` clock contract and `paused-world-v1` timer policy. `.time` reads relative combat seconds and freezes during ordinary Ultimate cinematics and active Hotori Time Stop. `duration=120` targets 120 relative seconds. `wait(seconds)`, `wait_until(...)`, scenario timestamps, receipts and playback stay on the absolute axis. Whole-run DPS uses actual relative elapsed time by default, or actual absolute elapsed time when the Absolute basis is selected in Results; zero relative duration makes relative DPS, EPS, HPS and SHP unavailable while preserving raw completion, damage and absolute rates. Native statistics and reports deliver paired `{ relative, absolute }` rates for DPS, EPS, HPS and time-weighted SHP. Field-time statistics retain their absolute combat-seconds definitions. The independent default absolute safety ceiling is 3600 seconds and is never enlarged to fit pauses. Scheduled `incoming`, `dummy`, `interrupt` and `combat` declarations are bounded by that absolute ceiling rather than the relative target. Caller execution limits remain authoritative. World-clock enemy schedules preserve their remaining delay through pauses; explicit interruptions and combat transitions retain their authored absolute instants.

The explicit scheduled `compile_script` API retains its precompiled clock and narrower branch adapter: at most one `.target.hp`/`.actor.energy` branch with a positive wait in each arm, or ordered energy gates; it cannot combine these mechanisms. Those limits do not apply to Studio's live source execution.

An ordinary action with unmet cooldown, energy, recovery or state requirements immediately stops the run with its source location and typed failure reason. This is the required admission policy, including for the future shared source interpreter; there is no automatic retry or inserted wait. Readiness queries and conditional skips remain **pending**. The explicit `wait_for_energy` gate is supported as a caller-threshold wait, not native cost admission. Impossible actions and loops that cannot progress are diagnosed automatically. No try, assert, generic `wait_for` or per-action timeout is needed. Failed admission remains a failed run and is excluded from successful aggregates. Stall/idle warnings are executor/report behavior owned by the [runner contract](../openspec/specs/combat-execution/reference.md#waiting-and-idle-diagnostics), not language fields or commands.

Hotori's native live source execution preserves the timer/resource, Katana combo, finisher and process-specific pause rules. `attack[window=attack]` yields at her native queued-next-Normal gate, permitting both five-stage Katana chains and the eleventh-input finisher. Ordinary full recovery also respects a later native next-Normal gate; it can consume enough time to reach timeout before the finisher. Each affected action, cooldown, buff, summon/projectile and target process declares whether/how it progresses in that state; paused lifetimes retain their remaining duration on exit. Real combat time continues. `hotori burst; mint swap;` fails immediately if the switch is still prohibited; use an explicit authored wait to reach state exit without inventing attacks, while eligible flowers/summons continue. Repeated `hotori attack` inputs use the owner-selected Katana mode until exit; any remaining inputs then use the current mode, and an immediate following switch still obeys admission. The `.hotori.time_stop.active` query permits guarded loops to avoid spillover. `wait(...)` between attacks consumes the same real combat time. There are no additional clock-query or `wait_real` commands. An absolute wait can finish while `.time`, ordinary cooldowns and independent modifier durations remain frozen. Deployed-object lifetimes advance during active Time Stop, while ordinary emissions pause; a living Hotori B2 permits Vita Bud attacks. Another cinematic still freezes both. Finite current cinematic and permitted Hotori direct action contexts have explicit exceptions; they do not grant immunity to persistent producers.

## Live fields

All fields are read-only and evaluate against current settled engine state in each trial, after preceding causal work. Logical fields return numeric 0/1; durations/cooldowns use seconds. No future RNG result or compiler-maintained form flag is exposed.

The documented native owner query surface is accepted as Q4-L2/R11. Only the
closed names below are public; unsupported names diagnose. Remaining action,
scenario and start authoring belong to Q4-L3/L4/L5. Existing native form inputs
remain available to Rust integration fixtures until those public action bindings
land; query acceptance does not certify unbound source actions.

| Field | Meaning |
| --- | --- |
| .time, .duration | Elapsed combat time and declared horizon in seconds; exact live values in native source execution |
| .target.hp | Current HP of target t0; general exact expressions in native source execution |
| .actor.energy | Settled whole energy of the active actor; general exact expressions in native source execution |
| .name.skill.ready, .name.burst.ready | Owner cooldown, resource and mode prerequisites, inspectable off field; on-field/recovery/switch admission is still checked by the executor |
| .name.skill.cd, .name.burst.cd | Remaining owner cooldown |
| .name.support.ready | Availability of the Support Skill on the next legal switch; distinct from Critical Riposte |
| .name.normal | Next one-based normal-chain input for the current native form; one after an owner-defined reset/end |
| .name.hp, .name.hpmax, .name.hpratio | Current effective vitals and HP fraction |
| .name.energy, .name.energymax | Current energy and capacity |
| .name.onfield | Whether this actor currently controls the field |
| .name.resources.resource_name | Owner-documented special resource, with units, capacity and matching start parameter |
| .name.mods.mod_name | Remaining duration of a documented timed modifier; absent/expired is zero, unknown names fail |
| .name.state_name.active | Owner-documented state flag, including .hotori.time_stop.active |
| .team.fons | Explicit immutable party starting Fons balance |
| .team.stamina, .team.staminamax | Exact shared combat stamina and capacity; require a declared complete account profile |

Resource/state names and defaults belong to each character model, not raw engine property paths. Readiness is advisory: admission rechecks after any implicit switch or intervening event. Native source execution supports `.team.stamina`/`.team.staminamax` from the declared shared account profile (missing profile diagnoses), `.time`, `.duration`, `.target.hp`, `.actor.energy`, named actor `.hp`, `.hpmax`, `.hpratio`, `.energy`, `.energymax`, `.onfield`, `.skill.ready`, `.skill.cd`, `.burst.ready`, `.burst.cd`, `.support.ready`, `.normal` and an explicitly declared immutable `.team.fons`. Named vitals use the native effective Max HP, including live set stacks and their expiry; HP ratios remain exact. Energy reads whole settled pools and capacities, including off-field actors. Skill/Burst readiness checks living state, native cooldown/energy and owner ability/packet/resource prerequisites, without waiving later field or recovery admission. Support readiness reads whether a switch from the current actor can trigger a reaction on the default dummy `t0` and admit the incoming QTE. It uses outgoing Cycle or an applicable owner rule such as Lacrimosa A6, plus the incoming actor's own admission requirements. An explicit targeted switch checks its authored dummy at admission. Reading availability never applies entry effects or dispatches a packet. `.normal` shares native continuation selection: exact gap boundaries, queued-input boundaries, wraps and field-exit resets. It follows evolved Blackbird, Fadia Lilith and Hotori Katana selection; the Katana finisher is input six. It never overrides a stage or admits an input. `.name.resources.cycle` reads the settled whole Esper Cycle pool for every actor, matching `start_cycle`. Hotori additionally supports `.resources.timepiece`, `.rewind.active`, `.time_stop.active` and `.mods.momentary_gaze`; see the [owner bindings](mechanics/characters/hotori/time-stop-resource-and-cooldown.md#selected-implementation-transaction-and-clock-specification) for exact units, defaults and intervals. Chiz binds `.resources.grains`, `.resources.loans`, `.debt.active`, `.surplus.active`, `.market.active` and `.mods.market` under the [financial model](mechanics/characters/chiz/grain-and-market-source-findings.md#selected-complete-financial-model), retaining wide exact interest and separate Fons. Blackbird binds `.resources.malice`, `.transformed.active`, `.out_of_combat.active` and `.mods.skill_permission`; Chaos binds `.resources.crime` and `.dread_echo.active`. Hathor binds `.resources.power` and `.emergency_delivery.active`, reading the native integer pool and mode admission predicate. Iroi binds `.resources.moon`, `.dream.active`, `.ivory_dream.active`, `.horn_dream.active` and `.mods.dream`, including exact released-clock drain and captured mode. Haniel binds `.resources.plot`, `.resources.harmony`, `.hootie.active`, `.paranormal_ace.active`, `.symphony_token.active` and the three corresponding `.mods` lifetimes. Jiuyuan binds `.resources.bullets` to her integer PowerBullets counter. Daffodil binds `.resources.resonance`, `.resources.phantom_steps`, `.eyes.active`, `.phantom_handle.active`, `.mods.eyes` and `.mods.phantom_handle`, separating the persistent Support pool from native timed Burst handles and target Insight. Nanally additionally binds `.authority.active`, `.underboss.active`, `.mods.authority` and `.mods.underboss` to native half-open deadlines under her [selected model](../openspec/specs/characters/nanally/spec.md). Adler, Nanally and Jiuyuan bind `.aiming.active` to native aiming; Lacrimosa binds complementary `.ranged.active`/`.melee.active` to her persistent form. These queries do not toggle modes or waive admission. Explicit resource starts declare their native resource models before initialization. Only the documented owner names are public; remaining action/start authoring stays in Q4-L3/L4. Queries consume no RNG and short-circuiting avoids unvisited fields. The explicit scheduled API retains its narrower one-branch restrictions; Studio Run uses live source execution. Unknown live fields diagnose rather than returning fabricated values.

```text
for let i=0; i<4; i=i+1 {
    if .edgar.support.ready { edgar swap; }
    if .mint.skill.ready { mint skill[target=t0]; }
    if .mint.burst.ready { mint burst; }
    mint attack[hold=1.2s], dash;
}
```

Adler exposes `.resources.karma`: whole held stacks, initially zero, capped at20, persistent across time/field exit, consumed by the native Skill callback. Karma has no timed modifier. Aurelia exposes `.resources.crescendo` (whole unexpired stacks, cap10 or20 with A6), `.mods.cadenza`, `.mods.crescendo` (remaining World seconds) and `.cadenza.active`/`.crescendo.active`. Known absent and equal-time expired effects read zero; reads never clean up or refresh state. Ordinary Aurelia source attacks declare the existing powered-projectile and Crescendo models so native hit callbacks populate the observed pool. Remaining owner action timing/hold integration remains Q4-L3. [Rust source tests](../crates/content/src/simulation/tests/source_queries.rs) check two continuing cycles, persistent Karma, exact expiry, unknown-name diagnostics and native replay.

Adler also exposes `.mods.blessing` and `.blessing.active` from his actual positive shield pool, including its living party carrier after a switch. Depletion, expiry or owner death returns zero immediately. Skill source inputs select the existing party-interception model. Edgar exposes `.resources.keys` (whole Keys of Truth, default0, cap3), `.mods.finnegans_wake`, `.finnegans_wake.active` and `.out_of_combat.active`. His native full-kit model initializes from party membership, including an idle off-field A1 observer. Burst captures and clears keys at field spawn; one key gives11 ObjectLifetime seconds from spawn, not from cast admission. Explicit nonzero key starts and remaining action choices remain L4/L3 work.

Baicang exposes `.resources.words` (unexpired stored Words, default0, cap3 or4 with A3), `.mods.domain` (remaining ObjectLifetime seconds) and `.domain.active`. Activated pending Words no longer occupy the stored pool; individual15s expiries do not share a refreshed pool timer. An announced future domain reads inactive/zero until its native spawn; A0/A5 lifetimes are12/16s. Native build projection selects the existing Word, hitlag and cancellation models. Burst costs120 Ultimate energy; Skill's shared cooldown is14s. Word generation pays its actual nonlethal current-HP cost and notifies equipped HP-consumption consumers once, separately from incoming damage. Remaining owner action choices/start authoring stay in L3/L4.

Fadia exposes `.mods.lilith`, `.lilith.active` and `.out_of_combat.active`. Lilith reads native permission, which may end before already committed hits finish; expiry, fifth-stage commitment and field exit do not create extra stocks or restore consumed stages. Native party projection selects her full-kit model for passive membership, Skill and Burst-only scripts alike. This preserves her existing incoming HP-floor rules and the9.42s permission, instead of selecting the historical partial Burst model based on which source actions appear.

Linko exposes `.mods.oversync` (remaining World seconds), `.oversync.active` and `.xiaozhen.active`, all initially zero/false. Burst grants 15s of Oversync from commitment, with permission absent at the exact deadline; this is separate from the 8/13s Resonance Field. Field exit preserves Oversync and an already deployed Xiaozhen; owner defeat clears both. Source Skill selects the [Xiaozhen model](mechanics/characters/linko/action-damage.md#live-oversync-and-xiaozhen-bindings): four canonical rockets and one companion deployment, followed by autonomous 8s pulses (6s at A1) from primary delivery. Recasting replaces the pulse generation. The separate explicit Throw route retains two packets and is not appended to ordinary Skill. There is no selected independent pet energy pool or timed pet modifier. Remaining alternate action authoring and measured action timing stay in L3.

Shinku (`start_energy` in `0..=60`, default `60`) uses `.energy` as Defiant Spirit (`ShinkuQEnergy`, max `60`, spent by Crimson Fury `burst`, ignores Charge Efficiency, `Charge` reaction, and off-field ally hits), `.resources.fading_reason` (`0..12`, filled in `Surging Crimson` by 5-stage `Tidal Assault` `Melee_Rage` hits and `Watch`, spent `12` by 3.0s-CD `Scarlet Descent`), `.resources.crimson_judgment` (`0..3`), `.resources.watch` (`0..8`, or `0..16` with A4), `.menacing_gaze.active`, `.mods.surging_crimson` (remaining World seconds), and `.surging_crimson.active`. Switching off-field (`CHANGE_ROLE_OUT`) ends `Surging Crimson`, clears `fading_reason`/`crimson_judgment`/A5 DEF, and clears `Menacing Gaze` unless A3 is unlocked (A3 keeps stacks/ticking off-field). `Dragonflame Verdict` clears `Surging Crimson` and A5 DEF after its hits settle. Passive 1 (`Lone Stride`, BT >= 2) grants `BackChargeMul = 1.0` Charge energy to living off-field teammates and (`1.0s` ICD when Shinku is the active Charge recipient) emits `ReactionAOE_Damage` + `+5% ATK` (`30s`, max 10 stacks); Passive 2 (`Wrathscale`, BT >= 4) adds `+50%` `UltraSkill` DMG vs non-boss targets; A5 grants `+50% DEF` in `Surging Crimson` and heals `3.0 * BaseATK` on `Scarlet Descent`. See [Native source checks](../crates/content/src/simulation/tests/source_forms.rs) and [Shinku mechanics reference](mechanics/characters/shinku/action-damage.md).

Skia exposes `.mods.gnaw` (remaining World seconds) and `.gnaw.active` from actual native Skill control. Public Skill selects the native six-contact schedule, enters Gnaw at1.048984s and grants5s. Field exit or death clears control immediately; returning does not restore it. Queries do not grant control or consume Fang Thrust. Mint exposes `.mods.riposte_guard` and `.riposte_guard.active`: the3s A1 Skill-reset gate created by a connected paid Critical Riposte, initially absent. It is separate from the opportunity to perform that Riposte. Zankou exposes `.resources.kindling` (initially0, integral0..1), `.reality.active`, `.illusion.active`, `.mods.illusion` (World seconds remaining), `.enhanced_entry.active`, `.enhanced_soulcross.active`, `.bloodfeast.active`, `.free_inferno.active` and `.mods.free_inferno`. Permissions and half-open deadlines use live owner selection; `.skill.ready`/`.burst.ready` use the selected variant's actual energy/cooldown gates. Readiness still requires a legal on-field input/cancel boundary when submitting an action. `.hunt.active`/`.delusion.active` and `.mods.hunt`/`.mods.delusion` reflect actual action/retained modifiers; A1 makes both living-source permanent (the modifier-time convention is `u64::MAX` microseconds). `.mods.scorch_enhancement`/`.scorch_enhancement.active` expose the six-node resonance's60s Scorch-defeat enhancement; defeating a target does not create another Kindling stack or refund Skill cooldown. [Rust source checks](../crates/content/src/simulation/tests/source_forms.rs) cover two complete legal rotations, exact expiry and native replay.

Sakiri exposes `.mods.defeat_bonus`/`.defeat_bonus.active` from the latest unexpired native A3 defeat-stack deadline, and `.mods.team_atk`/`.team_atk.active` from actual20s ordinary Burst buff instances on living allies, excluding herself. With no living carrier the latter reads zero. The ordinary and additional A4 carriers are managed snapshots, so source death alone does not end an ally's timer. Zero exposes `.mods.support_atk`/`.support_atk.active` from her20s A5 Support buff on the living owner. These names initially read zero without the effect/awakening. Switching does not refresh them; expiry is half-open. Queries read native instances without applying, refreshing or removing buffs, and unknown names diagnose. [Source checks](../crates/content/src/simulation/tests/source_managed_modifiers.rs) cover continuing entries/Bursts, exact expiry, source/carrier defeat and native replay.

Chiz additionally exposes `.resources.skill_stocks` (whole pressed Skill stocks, default/cap3), `.hoarding.active` (native on-field Charge Efficiency passive) and `.out_of_combat.active`. Stocks require the selected pressed Skill model, declared by explicit financial starts or Skill inputs; an unavailable model diagnoses rather than fabricating a pool. Spending and serial3s refills follow native events, including off-field and exact refill boundaries. These stocks are separate from Grains, Loans and Fons; remaining stock-start authoring stays in L4. [Rust source checks](../crates/content/src/simulation/tests/source_chiz_resources.rs) cover two continuing spends/refills and field transitions without resets.

Mint's managed `.mods.attendance`/`.attendance.active` and `.mods.crit_damage`/`.crit_damage.active` expose the native A3 reaction ATK buff (15s) and A5 Skill CRIT DMG buff (6s) on the living owner. Their own native triggers refresh independent timers; switches and queries do not refresh them. [Source checks](../crates/content/src/simulation/tests/source_managed_modifiers.rs) preserve two qualified Blossom participations, two Skill casts, exact expiry and native replay.

Hathor and Haniel also expose `.out_of_combat.active` from their native combat
flag, separately from field control. Authored combat transitions remain Q4-L5.

Akane Rin (`akane` / `Akane` / `"Akane Rin"`) accepts `start_string_energy` in `0..=30` and exposes `.resources.string`/`.string` (String Energy `0..30`), `.mode` (`0` = none, `1` = `Overtone`, `2` = `Reverb Field`), `.overtone`/`.overtone.active`/`.mods.overtone`, `.reverb`/`.reverb.active`/`.mods.reverb`, `.cadenza`/`.cadenza.active`/`.mods.cadenza`, `.anthem`/`.anthem.active`/`.mods.anthem`, `.perfect_pitch`/`.perfect_pitch.active`, `.mods.{a1,a4,r6}`, `.skill.ready`, `.skill.cd`, `.burst.ready`, `.burst.cd`, and `.energy`. `Rhythm Wave` Press/Hold (`hold >= 0.2s`) mutually replaces `Overtone`/`Reverb Field` on a shared `16s` cooldown, redirects to Press-only `Synced Resonance` at `30` String Energy without altering mode timers or Skill cooldown, and `Sonic Boom` unlocks manual Stage 2 in `Solo Cadenza` `[4s, 20s)` without auto-casting. See the [Akane Rin mechanics reference](mechanics/characters/akane/action-damage.md).

Elyms (`elyms` / `Elyms`) exposes `.elyms.connection` (`0` = `Weak`, `1` = `Medium`, `2` = `Strong`), `.elyms.optical_illusion` (`0..=4`), `.elyms.consumed_stacks`/`.elyms.total_consumed_stacks` (`0..=4`), `.elyms.power_e1.active`, `.elyms.power_e2.active`, `.elyms.on_the_line` (`0..=6` remaining off-field coordinated triggers), `.elyms.normal`, `.elyms.skill.ready`, `.elyms.skill.cd`, `.elyms.burst.ready`, `.elyms.burst.cd`, and `.elyms.energy`. Combat initializes at `Weak` Connection with `PowerE1Dur` active (`12s`); `Skill2` (`Intermittent Existence`) and `Skill3` (`Goodbye to You`) bypass and preserve `Skill1`'s `16s` cooldown while admitting `1.719655s` `Cinematic` world-clock pauses (`Skill1` has no pause and refills `100` Cycle); `Do You Get Deja Vu` (`5.499266s` pause) selects ST `UltraSkillB` when living targets equal `1` and Connection is `>= Medium` (else AoE), grants whole-team `20% AtkBase` + `10%` Follow-up Attack DMG (`20s`), and arms `6` off-field `Remembrance` (`3.0s` interval) coordinated attacks. See the [Elyms mechanics reference](mechanics/characters/elyms/action-damage.md).

## Expressions and control flow

let/const require a numeric initializer and use lexical scope. Assignment updates the nearest mutable binding. Same-scope redeclaration and assignment to const fail. Comparisons/logical operations return 0 or 1; zero is false. && and || short-circuit. Operators follow grammar precedence and associate left to right. Division/remainder by zero fail.

| Built-in | Arguments |
| --- | --- |
| min, max | One or more numbers |
| clamp | value, lower, upper; lower ≤ upper |
| abs, floor, ceil | One number |
| wait_for_energy | One nonnegative whole-energy threshold no higher than the active actor's declared capacity; multiple ordered gates shift each following action by the actual accumulated wait duration |

if/else evaluates conditions in order. while tests before each iteration. for initializes once, tests before each iteration and assigns after it. repeat evaluates a nonnegative integer count once. break exits the innermost loop; continue proceeds to its next iteration, including the for update. Native source loops preserve combat state and lexical bindings across actions/waits. Cumulative work and input budgets diagnose empty loops and zero-time yields. Branches and explicit energy waits compose in the same live program. Explicit scheduled export expands static control flow; Studio Run preserves live per-seed branches and loops. Priority lists, scripted enemy behavior and user functions remain outside the language.

## Design references

[gcsim fields](https://docs.gcsim.app/reference/fields/) provide readiness/resources/next-normal-index conventions; [scenario hurt configuration](https://docs.gcsim.app/reference/config/#set-hurt-generation) informs seeded external damage. [Bennett action transitions](https://github.com/genshinsim/gcsim/blob/main/internal/characters/bennett/skill.go) demonstrate next-action cancel windows. Hold semantics intentionally use seconds rather than gcsim's owner-dependent flag/stage/frame meanings. These are language/runtime design references, not NTE combat evidence.

## Example

```text
options iteration=1000 duration=20 seed=42;

mint char lvl=80/80 awakening=6 talent=8,8,8 qte=8;
mint add weapon=fork_lingganzhongjiezhe refine=5;
mint add cartridge=firefliesandtheforest count=4;
mint add stats hp=5600 atk=1168.15 def=0 cr=0.52 cd=0.44 dmg%=0;

sakiri char lvl=80/80 awakening=1 talent=8,8,8 qte=8;
sakiri add weapon=fork_mofeikesi refine=5;
sakiri add cartridge=speedyhedgehog count=4;
sakiri add stats hp=5600 atk=830.25 def=0 cr=0 cd=0 dmg%=0;

zero char lvl=80/80 awakening=1 talent=8,8,8 qte=8;
zero add weapon=fork_worldrain refine=5;
zero add cartridge=speedyhedgehog count=4;
zero add stats hp=15150 atk=2376 def=909 cr=0.05 cd=0.5 dmg%=0.1;

iroi char lvl=80/80 awakening=1 talent=8,8,8 qte=8;
iroi add weapon=fork_Door refine=5;
iroi add cartridge=speedyhedgehog count=4;
iroi add stats hp=17210 atk=2332 def=909 cr=0.05 cd=0.5 dmg%=0.1;

target hp=3500000 max_hp=10000000 def=1080 resist=0.2 psyche=0.1 anima=0.3;

active sakiri;

sakiri burst;

zero swap;
zero skill;

iroi swap;
iroi skill;

mint swap;
mint attack;
mint skill;
mint burst;
```
