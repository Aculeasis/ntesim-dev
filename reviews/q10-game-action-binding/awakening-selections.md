# Individual awakening selection integration

The native scenario, build/preview projection and public DSL accept canonical selected effect arrays for all 23 released owners at `global-2026-10-04`. Integer DSL awakening remains prefix sugar. An ordinary `EffectN` requires its own selected index; resonance_3 and resonance_6 require three and six selected effects respectively. Selection order cannot change execution. This migrates the existing partial models, not whole-kit coverage.

The owner mappings were checked against each frozen `{id}AwakenEffect` table (Mint uses `1019AwakenEffect1`), including separate ordinary and resonance rows. Existing owner model files retain their source hashes, hypotheses, producer gaps and applicability limits. No frozen native tables, saved source documents or verification receipts were rewritten.

| Owner | Important identity/count distinction |
| --- | --- |
| Adler | Effect5 refund and Effect6 Blessing duration are independent of R3 levels and R6 Incantation bonus |
| Aurelia | Effect1/5 crit, Effect2 Cadenza extension, Effect3 Burst bonus; R6 team Psyche bonus uses count |
| Baicang | Effect1/2 Power Words, Effect5 domain duration, Effect6 crit; R6 team ATK uses count |
| Blackbird | Effect1/2 coefficient additions, Effect4 duration/team damage, Effect5 resistance, Effect6 Psyche bonus; Burst coefficient additions and team Crit DMG belong to R3/R6 |
| Chaos | Effect2 DEF ignore, Effect3 timed Burst Crit DMG and Effect5 damage; resistance reduction belongs to R6 |
| Chiz | Effect1 capacity, Effect4 acquisition, Effect5 initial Grain; target Cosmos resistance belongs to R6 |
| Daffodil | Effect1 Skill bonus, Effect3 Insight duration, Effect6 DEF ignore; R3 levels use count |
| Edgar | Effect1 teammate QTE keys, Effect4 Skill bonus, Effect5 Max HP, Effect6 team DEF; HP percentages add before one settlement in native builds and Studio |
| Fadia | Normal R3 level projection uses count; Skill and nonzero-awakening Burst remain rejected under the existing owner boundary |
| Haniel | Effect2 extends Hootie, Effect3 Ace crit, Effect5 Ace Psyche bonus; team Crit DMG belongs to R6 |
| Hathor | Effect1 initial Power, Effect3/5 damage, Effect4 crit, Effect6 Crit DMG; Emergency Delivery DEF ignore belongs to R6 |
| Hotori | Effect1/2 Time Stop damage, Effect5 team single-target bonus, Effect6 Time Stop DEF ignore; team ATK belongs to R6 |
| Iroi | Effect2 assist, Effect4 Skill bonus, Effect5 team ATK, Effect6 callback receipt; Crit DMG belongs to R6 and revival resources remain unsupported |
| Jiuyuan | Effect1 Pact ATK and Effect5 Pact damage; Anima resistance belongs to R6; Pact settlement remains pending |
| Lacrimosa | Effect1 Nightmare coefficient and Effect2 QTE bonus; DEF ignore belongs to R6 with the existing 20% native/10% description conflict |
| Linko | Effect2 Burst coefficient, Effect5 modeled Skill bonus, Effect6 modeled Skill crit; ATK belongs to R3 and field duration/team bonus to R6; coordination eligibility remains partial |
| Mint | Effect2/4/5/6 packet/cast modifiers; skill levels and team cycle energy belong to R3/R6 |
| Nanally | Ordinary effects gate Authority, callbacks and Burst modifiers by identity; R6 damage and R3 levels use count |
| Sakiri | Effect3 defeat stacks, Effect4 Feast bonus, Effect5 airborne reset; R6 debuff-conditioned damage uses count |
| Shinku | Effect1 DEF ignore, Effect2 resistance and Effect6 ratios; Surging Crimson damage belongs to R3 and additional crit/DEF ignore to R6 |
| Skia | Effect1 coefficient, Effect2 timed crit, Effect3 marked damage, Effect4 mark duration; resistance belongs to R6 |
| Zankou | Effect1 permanent Hunt/Delusion, Effect2/4 Kindling, Effect6 final-hit bonus; ratio bonus belongs to R3 and timed ATK/Scorch enhancement to R6 |
| Zero | Effect1 child, Effect2 refund, Effect3 Burst crit, Effect4 Burst damage, Effect5 QTE buff, Effect6 child upgrade; Cosmos-cycle team ATK is separate R6 pending work |

Independent native tests distinguish selected effects from selection count using packet arithmetic, energy, timed buffs, Grain and HP projection. Public selection tests exercise continuing native scenarios, source normalization, observations/replay and complete CLI/WASM/Worker report parity. These checks establish only their implemented inputs; all reports retain incomplete readiness and unsupported owner dependencies remain visible.

Historical owner verifier scripts now match already implemented combat behavior: Baicang's equipped Crimson stacks are reconstructed from hit deadlines; Chaos, Linko and Shinku assert complete packet trains; Mint, Nanally and Sakiri independently account for native hit charge, recipient distribution, Charge Efficiency and fractional carry. Sakiri's A5 restores missing capacity after first-packet charge rather than granting an unconditional 100 units. Chiz and Zankou explicitly assert their reached unsupported producers before checking supported action slices. Saved verification receipts remain unchanged and do not establish current complete-kit readiness.
