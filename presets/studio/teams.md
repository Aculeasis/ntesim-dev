# Curated team comparison assumptions

## Shared profile

New teams use 1,000 seeds starting at 42, random crit and 120 relative
combat seconds. Characters and Arcs are level 80/80. Purchased Normal, Skill
and Burst levels are 8/8/8. Every Team list character, including S5, declares
`qte=8`. Rust applies earned Resonance upgrades to the applicable talents. Baseline awakening counts
are A0 limited S, A1 standard S and A6 A-rank/Chiz/Zero. Individual selected
nodes are explicit in each script. Arcs use T1 unless an entry declares an
exception. A count of one does not require selecting awakening node 1.

The common target is a level-80 stationary Boss dummy: current/max HP
1,000,000,000; DEF/base DEF 1080; all elemental resistances 20%; original ATK
1000 for Haniel's Nova extraction; initial/original Break gauge 30, Broken
duration 10 seconds and Discord reduction fraction 0.1. Control susceptibility
is an explicit scenario assumption, independent of Boss eligibility. There are
no incoming attacks, movement, forced successful parries or Critical Dodges.
Shared starting Fons is explicitly 250000, including Chiz's Arc dependency.
Ordinary Ultimate energy starts full; special resources retain owner defaults.
All resources and cooldowns continue between rotation rounds. Hotori starts
with zero Timepiece. Absolute playback continues during Time Stop; the
relative duration and ordinary World timers pause under the current timer policy.

S5 remains a separate historical control. Its duplicate support
sets, missing Arcs and different HP assumptions are intentionally preserved;
its score is the outcome of its own script. Its rotation uses actual switch
reaction readiness and the ensuing owner recovery.

## Fifteen useful lines

The [original comparison contract](https://github.com/Aculeasis/ntesim-dev/blob/c1fcb41119b963dec26cf6aac68e6acce1f35281/docs/ACCEPTANCE_DELIVERABLES_CONTRACT.md#L30)
specifies fifteen useful substat lines **per character**, across the whole
console. The [Mint reference](../../docs/mechanics/characters/mint/build-candidate-context.md)
retains that budget. Values below recover the historical published-value
hypothesis from the [then-consumed exact Rust values](https://github.com/Aculeasis/ntesim-dev/blob/c1fcb41119b963dec26cf6aac68e6acce1f35281/crates/data/src/equipment/substats.rs#L60).
They are comparison inputs, not a certification of native roll-pool legality.

| One useful line | S cartridge | Type II | Type III | Type IV |
| --- | ---: | ---: | ---: | ---: |
| HP | 1000 | 200 | 300 | 400 |
| HP or ATK percentage | 12.5% | 2.5% | 3.75% | 5% |
| Flat ATK or DEF | 80 | 16 | 24 | 32 |
| DEF percentage | 17.5% | 3.5% | 5.25% | 7.5% |
| Crit Rate | 10% | 2% | 3% | 4% |
| Crit DMG | 20% | 4% | 6% | 8% |
| General damage | 10% | 2% | 3% | 4% |
| Cycle or Break Intensity | 60 | 12 | 18 | 24 |

Use the same authored contribution convention throughout: one S cartridge,
two Type II, four Type III and one Type IV equivalent. Their module main
contributions sum to flat ATK 420 and HP 5600. This twenty-cell-equivalent
budget names stat categories only; the simulator neither packs a layout nor
proves its obtainability. Per-character specialization is included explicitly
from the frozen `DT_CharacterEquipmentSlotsData`/`DT_EquipmentModifySlotsEffect`
source values: two matching Type II or four matching Type III contributions.
Character, Arc and declared set stats remain automatic and are excluded from
these authored additions.

| Recipe | Cartridge's three lines | Each of four Type III | Type IV | The two Type II | Total substat additions |
| --- | --- | --- | --- | --- | --- |
| `crit` | CR, CD, general damage | CR, CD | CR, CD | CR / CD | CR 28%, CD 56%, damage 10% |
| `support` | ATK%, CR, general damage | ATK%, CR | ATK%, CR | ATK% / CR | ATK 35%, CR 28%, damage 10% |
| `ci` | CI, CR, general damage | CI, ATK% | CI, ATK% | CI / ATK% | CI 168, ATK 22.5%, CR 10%, damage 10% |
| `bi` | BI, CR, general damage | BI, ATK% | BI, ATK% | BI / ATK% | BI 168, ATK 22.5%, CR 10%, damage 10% |

Every row allocates `3 + 4*2 + 2 + 1 + 1 = 15` lines. A cartridge main is
additional to those lines: CD 60%, CR 30%, ATK 37.5%, DEF 52.5%, an elemental
bonus 37.5%, or CI/BI 180. These level-20 orange main values come from the
frozen `CT_EquipmentBaseAttribute` source. The chosen main and recipe appear
beside every character declaration. Specialization, mains and automatic set
bonuses do not consume or duplicate the fifteen useful lines.

The script's `add stats` is the sum of the chosen recipe, cartridge main,
module main contributions and explicit specialization. For example, a
Cycle-built Zero has authored CI `168+180=348`, ATK `420`, HP `5600`, CR `0.1`,
general damage `0.1` and ATK percentage `0.225+4*0.1=0.625`. Rust then adds his
character/Arc/set values once. Flat `ci`/`bi` are intensity points, not damage
percentages. Each team has at most one Speedy Hedgehog wearer.

Repeated personal sets are intentional: Fireflies stacks, Diabolos's Chaos
resistance ignore, Lost Radiance's DEF ignore and Shadow Creed's ATK belong to
their wearer. Two Diabolos damage dealers do not duplicate a shared party buff.
Speedy Hedgehog supplies the shared ATK effect and has one assigned owner.

## Research and rotation interpretation

## Research and rotation interpretation

The reference team roster and character kits were evaluated on 2026-11-13. They supply starting
team/build ideas; native owner models and actual complete runs decide what the
scripts can execute. Labels describe playstyles, not a claim of global
optimality. Scores remain subject to the native model assumptions retained in
the generated catalog and the owning [mechanics references](../../docs/mechanics/README.md).

- Nanally, Hotori and Jiuyuan:
  use their complementary Blossom passives, Zero's instant cycling and
  sustainable personal-damage windows. Zero owns Speedy Hedgehog; Hotori can
  use Lost Radiance without duplicating the team buff.
- Zankou and Daffodill:
  combine persistent Scorch/DoTs, Sakiri/Iroi buffs and Insight before Break.
  The mandatory Daffodill is explicitly A3 with nodes `[3,4,5]` and a BI build.
  Sakiri selects exactly node4: ordinary Burst grants30% of her source base ATK to other living teammates, and A4 separately adds another30% for twenty seconds.
- Lacrimosa and Baicang:
  retain off-field damage, alternate damage windows and earned Support
  switches; no automatic successful Phantom Step parry is assumed.
- Shinku, Chaos, Chiz and Aurelia:
  build real Remora/Blossom/Charge chains and check continuing energy.
  Dummy-target scripts do not claim the dodge opportunities of real combat.
- Blackbird and Linko:
  distinguish form windows, Nova/Stain support and follow-up ownership from
  the main damage dealer's field time.

Excluded throughout: Elyms and Akane Rin; Elyms's Echoes On Hold is absent from the catalog.
Akane Rin's Far Side of the Moon is canonical `weapon:fork_prism` and is excluded even though
the engine has an implementation. Alternative Arcs recommended on their pages are not
automatically excluded if they belong to another available character.

Raw source acquisitions, exact-source trial outputs and before/after
comparisons stay in `cache/team-curation/`. Final Team list values are
generated through the current Rust/WASM trial session and include source and
engine identities; no separate hand-authored score table is used.

## Composition and rotation choices

Each linked source declares all four builds. A timed support pass uses available
Skills/Bursts, then leaves the main actor on field for the authored window.
Readiness guards preserve continuing energy and cooldowns. The final seconds
use a shorter Normal-attack tail and wait to the exact 120-second horizon.

| Source / roster | Equipment roles and repeating plan |
| --- | --- |
| [Nanally Blossom](nanally-blossom.nte): Nanally / Hotori / Zero / Jiuyuan | Zero carries CI and Hedgehog. Zero Skill → Jiuyuan funds Buds; Jiuyuan/Nanally use Fireflies for personal damage. Record a paid Zero Skill with Hotori once per Time Stop; preserve Nanally's Underboss window, five Normals and a held attack. Exit Time Stop after its real-time recovery boundary. |
| [Zankou Insight](zankou-insight.nte): Zankou / Iroi / Sakiri / Daffodill | Zankou's real entry Cycle enables the Daffodill Scorch opener. Daffodill `[3,4,5]` spends earned Phantom Step returns around Sakiri `[4]` and Iroi's Burst-before-Skill setup. Held-enhanced entry leads through uninterrupted Illusion to enhanced Soulcross, Bloodfeast and the free Inferno; another Illusion chain spreads Vile Ash to ten stacks. Only observed Support entries remove Black Tome seals. Iroi owns Hedgehog; builds and the comparison profile are unchanged. |
| [Zankou Blackbird](zankou-blackbird.nte): Zankou / Blackbird / Sakiri / Daffodill | Replace Iroi with Blackbird. Cycle 1 casts Blackbird Skill and activates transformation Burst. Subsequent cycles switch to Blackbird when energy suffices for finisher -> skill -> Burst reactivation, granting team ATK buff and coordinated bird attacks. Sakiri owns Hedgehog; Daffodill A3 `[3,4,5]` BI and Zankou crit build. |
| [Nanally Hexed](nanally-hexed.nte): Nanally / Sakiri / Linko / Zero | Sakiri enables Hexed, Linko supplies persistent follow-ups, CI Zero funds Blossom. Keep Nanally on field during Underboss; Zero owns Hedgehog and Linko uses personal Fireflies. |
| [Nanally Dream](nanally-dream.nte): Nanally / Sakiri / Iroi / Zero | Replace Linko with Iroi's Dream support; use Shadow Creed on Iroi because Zero already owns Hedgehog. Preserve the same earned Blossom/Hexed windows. |
| [Hotori Blossom](hotori-blossom.nte): Hotori / Iroi / Jiuyuan / Zero | Jiuyuan → Zero → Iroi creates Buds before Hotori's Time Stop. Lost Radiance supports Hotori's damage, Zero supplies CI/Hedgehog, Iroi uses Shadow Creed. No prepaid Timepiece. |
| [Mint Blossom](mint-blossom.nte): Mint / Hotori / Jiuyuan / Zero | Accessible A6 Mint is the normal-field driver. CI Zero → Jiuyuan creates Buds; use Hotori's available Time Stop between Mint windows. Zero alone holds Hedgehog. |
| [Zankou Break](zankou-break.nte): Zankou / Daffodill / Adler / Sakiri | BI Zankou `[5]` and BI Daffodill `[3,5]` are explicit low-awakening investment exceptions. Adler uses BI/Kingdom's Guard; Sakiri carries Hedgehog. Prioritize the Daffodill setup before the Break-damage window. |
| [Zankou Hexed](zankou-hexed.nte): Zankou / Sakiri / Iroi / Linko | Switch Zankou's earned Cycle into Anima Linko for Hexed, then Iroi/Sakiri buffs and sustained Zankou damage. Linko keeps Fireflies; Iroi owns Hedgehog. |
| [Lacrimosa Quickswap](lacrimosa-quickswap.nte): Lacrimosa / Haniel / Sakiri / Daffodill | Lacrimosa's naturally earned Chaos Cycle → Haniel enables Nova. Haniel's Shadow Creed and Sakiri's Hedgehog support Lacrimosa/Daffodill Diabolos damage. No assumed Phantom Step counter. |
| [Lacrimosa Scorch](lacrimosa-scorch.nte): Lacrimosa / Zankou / Sakiri / Haniel | Alternate Lacrimosa's field window with Zankou's Incantation damage and persistent Scorch; Haniel enables Nova after Chaos Cycle is earned. Sakiri owns Hedgehog. |
| [Baicang Scorch](baicang-scorch.nte): Baicang / Adler / Sakiri / Daffodill | Baicang → Daffodill is the Scorch handoff; Adler's BI/Kingdom's Guard adds Break pressure. Sakiri refreshes Hedgehog. Build Power Words with actual Normals and spend only available Bursts. |
| [Baicang Linko](baicang-linko.nte): Baicang / Sakiri / Linko / Daffodill | Daffodill first receives Baicang's Cycle for Scorch; Linko adds persistent personal damage during Power Words/Domain windows. Sakiri owns Hedgehog. |
| [Blackbird Nova](blackbird-nova.nte): Blackbird / Daffodill / Haniel / Sakiri | Blackbird's instant Cycle → Chaos Daffodill enables Nova before Haniel/Sakiri buffs. Preserve transformed Blackbird field time; Sakiri carries Hedgehog. |
| [Blackbird Stain](blackbird-stain.nte): Blackbird / Iroi / Daffodill / Chaos | Blackbird → Lakshana Chaos spends real Cycle on Stain; Daffodill and Iroi then support the next transformed window. Iroi is the Hedgehog owner. |
| [Blackbird Hathor](blackbird-hathor.nte): Blackbird / Sakiri / Lacrimosa / Hathor | Blackbird → Hathor enables Stain, followed by Lacrimosa's persistent Chaos damage and Sakiri/Hedgehog. Hathor uses a supported five-second held Skill; never switch out of Blackbird's form prematurely. |
| [Chaos Remora](chaos-remora.nte): Chaos / Zero / Hathor / Haniel | Chaos → Haniel enables Stain; Zero Skill → Hathor supplies Remora, then return to Chaos for actual Crime/held-attack conversions. Zero carries CI/Hedgehog; Haniel Shadow Creed. |
| [Shinku Charge](shinku-charge.nte): Shinku / Zero / Hathor / Iroi | Zero → Hathor creates Remora, a second paid Zero Skill → Iroi creates Buds while Remora lives, enabling Charge directly to Shinku. Lost Radiance Shinku drives damage with full Surging Crimson alignment; Zero owns Hedgehog and Iroi Shadow Creed. |
| [Chiz Charge](chiz-charge.nte): Chiz / Zero / Jiuyuan / Hathor | The same two-Skill chain with Jiuyuan's Bud support and A6 Chiz on field. Contemplative Cat's dependency is funded by explicit shared starting Fons; Zero owns Hedgehog. |
| [Aurelia Charge](aurelia-charge.nte): Aurelia / Hathor / Jiuyuan / Zero | CI Zero → Jiuyuan, then Zero → Hathor enables Charge for Aurelia's Devil's Blood Curse damage window. Jiuyuan keeps personal Fireflies; no duplicate Hedgehog. |
| [Skia Charge](skia-charge.nte): Skia / Zero / Haniel / Jiuyuan | Haniel buffs, Zero → Jiuyuan makes Buds, second Zero → Skia creates Remora/Charge. Street Boxer Skia takes the long field window; Zero CI/Hedgehog, Haniel Shadow Creed. |

The mandatory Daffodill A3 selection buys three nodes, not a prefix A5. The
separate Break variant deliberately buys Zankou A1 node 5 and Daffodill A2
nodes 3/5 to exercise their Break interactions. These are the only departures
from the shared awakening-count baseline. Every Arc remains T1, including the
accessible characters; no hidden refinement advantage is used.

Pilot comparisons use the same first 25 seeds, builds and target. Support-refresh
intervals (12/15 versus 20 seconds, or 16 versus 20 for the mandatory Blossom
team), measured after the support pass, screen longer personal-damage windows
against more frequent buff setup. For Blackbird, interval/finisher-threshold
changes produced identical outcomes in the pilot; the useful alternative was
an ordinary Skill before the first handoff and before transformation. That
enabled earlier reaction spending and was retained in all three rosters. These
screens are not proof of a global optimum or statistical significance.
Additional retained comparisons cover the mandatory Daffodill BI versus direct
build, Hotori's paid recording window, Nanally's Underboss held attack and
reaction-routing order. Invalid or incomplete variants cannot win. Final
1,000-seed runs use the selected exact sources and current engine; diagnostic
samples verify actual reactions rather than inferring them from roster names.

### Historical Zankou Insight measurements

The earlier owner audit found a missing ordinary Sakiri Burst carrier: it grants
30% of her source base ATK to other living teammates for twenty seconds. A4
adds a separate 30% carrier. The preset keeps one selected node, replacing
`[6]` with `[4]`; Daffodill stays exactly `[3,4,5]`.

These receipts predate `relative-combat-v1` / `paused-world-v1`. Their DPS and
absolute packet times are historical, not current rankings. All bundled old
calculations are unavailable until an explicit current refresh.

Matched native trials over seeds 42–49 on that engine gave 18496.03 mean
DPS for the new rotation versus 15546.86 for the old rotation. On the new
rotation, Sakiri `[]` and `[6]` each gave 18191.29; `[4]` added 304.74 DPS
(1.68%). This tap-Skill rotation does not invoke A6's multi-target Eating Mode;
the six-node resonance bonus is separate from selecting node6 alone. These are
an eight-seed comparison, not a required uplift or an optimum.
At seed 42, source base ATK 1094 grants 328.2 ATK per carrier to Zankou, Iroi and
Daffodill, excluding Sakiri. Two twenty-second windows cover 40 of 120 seconds;
direct and ATK-scaled periodic damage change, while Break damage is unchanged.
The first ten-stack Vile Ash tick lands at 45.463464s. Two paid Phantom Step
returns and one unsealed Black Tome activation execute without a parry success.
Exact sources, engine identities, complete paired receipts and damage-channel
calculations were retained in `cache/complete-zankou/`. That historical source
completed1000 trials at120 absolute seconds with18722.78 mean DPS. The current
relative-time source separately guards Black Tome's 20-World-second summon.
The late-support correction removes duration reserves and terminal carry-only
fillers from the twenty curated compositions. Their normal team decisions
continue until the shared engine stops at 120 relative seconds, including when
the final action or wait would otherwise continue longer. Support handoffs still
respect form boundaries. In Insight seed 42, the old final Iroi Skill at relative
70.070 (absolute 119.292) is followed by further team windows, including Skills
from all three supports after relative 90 seconds. Builds and resources retain
their original assumptions. Loading or editing the source does not regenerate
catalog scores.

## Model limits

All scores use explicitly selected research hypotheses, not game measurements.
Zero's cast-time Skill fill and Lacrimosa's native connected-packet Cycle binding
were missing and are corrected with owner tests. The latter follows the frozen
Global values, which supersede the older sampled Cycle column in its mechanics
reference. No incoming attacks means no assumed successful counter, dodge or Phantom Step parry finisher. Daffodill's paid Burst can still fund her ordinary charged-entry returns. Support reaction admission requires an actually full outgoing
Cycle pool.

Zankou's owner derives both forms, enhanced Soulcross and paid/free Ultimates from actual inputs and continuing state. Its finite clocks, spread/stack rules and normalized armor fixture retain explicit bounded assumptions in the mechanics reference. Shinku's full public Rage input sequence exceeds the current bindings. These compositions are executable research candidates
within those limits. Their numbers must not be presented as complete live-game
ceilings, and their ordering may change when those owner models expand. S5 has
different historical assumptions and remains a control rather than a normalized
twenty-first candidate.
