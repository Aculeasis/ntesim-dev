# Native source baselines and calibration

These are executable A0 recipes for the frozen package, using the ordinary
source/build path. Each runs at least two continuing cycles, with an equipped
primary actor, complete ordinary chains, Skill and Burst. The saved script is
the rotation artifact: no internal packet IDs, injected costs or caller timing
tables are required. Resources, buffs, cooldowns, pending effects, dummies and
seeded streams continue across the repeated body.

The new recipes use a compatible blue Arc at refinement 1, an explicit two-piece
Shadow Creed set and authored additive stats. The set supplies its own static
ATK bonus exactly once. They fund Bursts from empty energy with ordinary chains;
150s periods deliberately leave room for natural expiry and continuing effects.
The existing Mint, Chaos and Nanally recipes retain their own equipped builds,
initial resources, horizons and input choices. Blackbird explicitly adds charge
efficiency to fund the evolved finisher as well as the transformation. These are legal integration
baselines, not optimized rotations or measurements of game DPS.

## Executable recipes

All target stats and combat assumptions are in the source. Most new recipes use
two anonymous dummies; existing one-dummy Mint/Chaos runs and multi-dummy owner
checks retain single/area recipient distinctions.

| Actor | Script | Reached choices beyond ordinary chains |
| --- | --- | --- |
| Adler | [adler.nte](../examples/baselines/adler.nte) | Skill DoT, Karma and shield |
| Aurelia | [aurelia.nte](../examples/baselines/aurelia.nte) | Cadenza/Crescendo, Burst and autonomous effects |
| Baicang | [baicang.nte](../examples/baselines/baicang.nte) | Stored Words, HP cost and continuing domain |
| Blackbird | [blackbird.nte](../examples/baselines/blackbird.nte) | Ordinary/evolved Skill, evolved chains/Plunge and finisher |
| Chaos | [chaos-native-cycles.nte](../examples/chaos-native-cycles.nte) | Crime, Dread Echo and Verdict across form expiry |
| Chiz | [chiz.nte](../examples/baselines/chiz.nte) | Reserved held Skill, paid third input, market and separate Fons |
| Daffodil | [daffodil.nte](../examples/baselines/daffodil.nte) | Resonance, paid Finale, Zero Support and Phantom entry |
| Edgar | [edgar.nte](../examples/baselines/edgar.nte) | Aimed held channel, keys and Finnegan's Wake |
| Fadia | [fadia.nte](../examples/baselines/fadia.nte) | Skill effects and Lilith attacks/expiry |
| Haniel | [haniel.nte](../examples/baselines/haniel.nte) | Hootie, Ace/Symphony and held Cannon |
| Hathor | [hathor.nte](../examples/baselines/hathor.nte) | Power, emergency mode and explicit Branch landing |
| Hotori | [hotori.nte](../examples/baselines/hotori.nte) | Rewind, regenerated Timepiece and guarded Katana finisher |
| Iroi | [iroi.nte](../examples/baselines/iroi.nte) | Moon, held attacks, Dream and field transitions |
| Jiuyuan | [jiuyuan.nte](../examples/baselines/jiuyuan.nte) | PowerBullets, Pact and autonomous ticks |
| Lacrimosa | [lacrimosa.nte](../examples/baselines/lacrimosa.nte) | Nightmare and both persistent forms |
| Linko | [linko.nte](../examples/baselines/linko.nte) | Oversync and off-field Xiaozhen continuation |
| Mint | [mint-native-cycles.nte](../examples/mint-native-cycles.nte) | Compiled Mind Royale/Fireflies and paid continuing periods |
| Nanally | [nanally-full-kit.nte](../presets/native/nanally-full-kit.nte) | Authority/Underboss, held/secondary/Plunge, paid Dodge/Riposte and Support |
| Sakiri | [sakiri.nte](../examples/baselines/sakiri.nte) | Suppression and paid Burst effects |
| Shinku | [shinku.nte](../examples/baselines/shinku.nte) | Surging Crimson activation, refresh and expiry |
| Skia | [skia.nte](../examples/baselines/skia.nte) | Gnaw activation and continuing effects |
| Zankou | [zankou.nte](../examples/baselines/zankou.nte) | Heartwrench DoT and Hunt |
| Zero | [zero.nte](../examples/baselines/zero.nte) | Ordinary Skill/Burst and shield effects |

Run, save the compiled program, or export and replay the executed result:

```sh
python tools/cargo.py run -p ntesim-cli -- run-script examples/baselines/blackbird.nte
python tools/cargo.py run -p ntesim-cli -- compile-script examples/baselines/blackbird.nte > blackbird-program.json
python tools/cargo.py run -p ntesim-cli -- run-compiled-script blackbird-program.json
python tools/cargo.py run -p ntesim-cli -- export-script examples/baselines/blackbird.nte target/blackbird-export
python tools/cargo.py run -p ntesim-cli -- replay-script target/blackbird-export/report.json
python tools/cargo.py run -p ntesim-cli -- verify-tabular .
```

Export destinations must be new directories. The self-contained report can be
relocated and replayed after the original source changes. Engine/content/input
identity mismatches reject; replay also reconciles saved failed prefixes without
turning them into completed DPS or ranking inputs. See [CLI usage](RUNBOOK.md)
and the [report contract](../openspec/specs/combat-execution/reference.md#reports-failures-and-horizon).

## Independent checks and limits

The existing Rust owner tests remain the mechanics oracle. The
[source baseline tests](../crates/content/src/simulation/tests/source_baselines.rs)
verify these saved recipes for every canonical owner, serialize/reload compiled
inputs and compare complete native reports. This equality checks source
integration, costs, causal events, recipients, resources and continuing state;
agreement between two callers of one engine is not independent game evidence.
Existing [Mint checks](../crates/content/src/simulation/tests/source_mint.rs) pin
the independent first hit `190.65046`, exact paid energy/carry and multi-dummy
packet counts. [Chaos checks](../crates/content/src/simulation/tests/source_forms.rs)
retain independent first damage, paid Verdicts and captured-form expiry.

The same integration boundary compiles and executes every supported weapon at
all five refinements and all twelve sets at two/four pieces. It checks canonical
IDs, refinement retention and disclosed model quality through source. Charge
efficiency uses the canonical shared hypothesis description, preserving its
bounded assessment instead of treating a stale duplicate description as a custom
model. Existing Rust equipment tests own
independent trigger/expiry, recipient, static-bonus and five-rank numbers; the
access grid does not claim every proc is triggered by its first ordinary hit.
Representative continuing source gear recipes remain in `examples/`, including
Mind Royale, Gold Wool, Hootie, incoming shields, HP/healing, Break and Support.
No gear/character cross-product or duplicated CLI mechanics harness is needed.

[Formula calibration](../openspec/specs/calibration/reference.md) retains complete
reference inputs, applicable source context and explicit tolerances. Negative
RES models remain distinguishable hypotheses; Nightmare/Scorch discrepancies
retain their selected executable rules and unresolved game interpretation.
No applicable captured game rotation was acquired. Seed variance describes RNG,
not model uncertainty. Strict readiness and native/browser agreement remain
separate from completed disclosed source execution. [Native performance and
resumable lifecycle acceptance](NATIVE_PERFORMANCE.md) is complete at its declared
boundaries; [browser calibration](BROWSER_CALIBRATION.md) now retains complete
native/WASM/Studio Worker executions of these continuing baselines. Browser lifecycle/performance and distribution have separate current-artifact checks.
