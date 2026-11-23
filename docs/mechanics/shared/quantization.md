# Native stat and settlement quantization

Responsibility: the native boundary, precision and rounding rule for projected stats and delivered HP/shield/damage amounts.

Open questions: does native HP/DEF become integral after all additive contributions, and under what rule? At which boundary does native damage/healing/shield settlement round? Are display-floor traces distinct from actual HP settlement? Are any intermediate stages quantized or capped?

Runtime boundary: native build projection, Studio and explicit target stat packs use the explicit `build.stat_quantization_v1` product policy. They derive HP/DEF from exact base, percentage and flat contributions and quantize once to nearest integer, with exact halves upward. Receipts retain this assumption. Target projections preserve exact layered values alongside settled `runtime_stats`; zero rounded Max HP and overflow reject. The layered target pack `20001` settles DEF756.5 to757, giving `1080000/1837` damage for a noncritical1000 base packet at attacker level80 with zero resistance. This is not verified native behavior and does not resolve the open question.

[Lacrimosa's A5 graph](../characters/lacrimosa/nightmare-damage-and-duration.md#a5-maxhp-reduction-and-healing-source-chain) now supplies a specific internal ceiling stage before returning an additive healing amount. That stage is source-known; its native getter inputs, subsequent bonuses/clamp and final HP settlement remain separate questions. Do not treat it as a universal rounding policy or keep that particular graph operation classified as unrecovered.

Consequence: fractional HP/DEF builds now execute under that declared policy, including Haniel/Nakupeda combinations. Guessing truncation can alter later scaling and full/damaged HP preservation. A display integer cannot prove native settlement rounding.

The current exact arithmetic and explicit preview policies are in [damage/stat zones](damage-and-stat-zones.md#6-implementation-and-verification-boundaries), [exact arithmetic](exact-scaling-arithmetic.md) and [build projection](native-build-projection.md). They are implemented choices, not additional game evidence.

Checked evidence: [formula audit](../../../reviews/combat-formula-audit/review.md), original-export/Fraction build/combat oracles linked in the known cards, and source-bound build receipts. Synthetic half-tie tests establish their declared policy only.

Return on applicable fractional/tie native evidence to resolve the game rule, or on a change to the bounded runtime policy. Check full/damaged HP and additive awakening/gear combinations independently; do not multiply already rounded stats or retrofit historical receipts. Damage, healing and shield settlement remain separate unanswered boundaries.
