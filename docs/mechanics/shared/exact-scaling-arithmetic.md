# Exact scaling formulas

The in-memory `formula` module evaluates nonnegative rational scaling expressions. It provides technical arithmetic for later reviewed adapters; it does not assert an NTE damage formula, percentage convention, crit policy, defense/resistance curve or game rounding rule.

Game stat and damage-zone semantics belong to the [combat formula contract](damage-and-stat-zones.md); ordered technical stages do not imply that each gear bonus is an independent multiplier.

Inputs explicitly declare an output `unit`, a rational `flat` amount, ordered `terms` (`stat` name and rational `coefficient`), ordered `stages` (`id` and rational `multiplier`), and one final `rounding` policy. A separate map supplies the referenced stat values as rational quantities. Numerators and denominators in inputs are unsigned 64-bit integers. No denominator scale, rounding policy, coefficient or stat defaults are supplied.

```text
base  = flat + sum(stats[term.stat] * term.coefficient)
exact = base * stages[0].multiplier * ... * stages[last].multiplier
amount = quantize(exact, rounding)
```

Supported final policies are `floor`, `ceil` and `nearest_ties_even`. All terms and stages retain exact reduced fractions; rounding occurs once, after the entire expression. Multiplication cancels common numerator/denominator factors before checked products. Addition reduces shared denominator factors before forming the denominator product. Intermediate arithmetic is bounded by `u128`; a checked intermediate overflow rejects evaluation even if a different symbolic rearrangement might fit. Final amounts must fit `u64` for the existing pool/health quanta.

Evaluations include the flat value, each scaled term, summed base, each ordered stage, final exact fraction, rounding policy, output unit and quantized amount. Exact numerators/denominators serialize as decimal strings so a JSON consumer need not support `u128` or convert them to floating point. The unit is a caller declaration; dimensional consistency and stat/target capture rules remain the adapter's responsibility.

Empty units/stat names/stage IDs, missing referenced stats, duplicate stage IDs, zero denominators, overflow and unknown JSON fields fail. A referenced stat is required even when its coefficient is zero. Later stages are validated even if an earlier multiplier is zero. Unused stat values remain outside this formula's dependency closure. Explicit zero values are valid; unknown values have no representation as implicit zero. Signed stat derivation and per-stage quantization are outside this initial contract.

Seven formula tests cover an independent decimal example, final-only quantization, required stat dependencies, zero-denominator/stage identity errors, cancellation and overflow edges, JSON requirements, 23,040 generated scaling cases and 3,751 fractions under all three rounding modes. These checks establish arithmetic behavior, not a validated game adapter.

## Synthetic damage binding

Both synthetic runners accept an effect with `kind=formula_damage`, `target`, `formula`, `stats` and `hit_delay_us`. The formula must explicitly declare `unit=health_quantum`, meaning the same caller-chosen quanta used by the target's HP/shield values. Unit mismatch or an unknown target is rejected before simulation. Stats are serialized caller-provided constants for this effect; no game snapshot/dynamic-stat policy is inferred.

The effect evaluates the complete expression when its effect event executes. On success, its causal trace contains the formula evaluation and it schedules a hit using the quantized amount and explicit local hit delay. That hit settles against the target's current HP/shields, preserving the existing command -> effect -> hit cause links. A hit beyond the inclusive real horizon remains pending; evaluating a formula does not prematurely damage the target.

Evaluation or hit scheduling failures emit `FORMULA_EFFECT` at `commands/INDEX`, preserve the failed partial trace and halt without creating that hit or changing the target. Rotation source/effect/iteration mappings identify the failing command. Previously completed effects stay applied; the runner does not roll back the whole plan.

The [synthetic fixture](../../../examples/synthetic-formula.json) and [rotation](../../../examples/synthetic-formula.nte) exercise two continuous formula hits: `(3.4 + 10.1 * 3.1) * 1.1 * 0.8 = 30.5448`, rounded to 31. A five-quantum shield absorbs five from the first hit; HP falls from 100 to 43 after both hits. These are artificial numbers and timings. Runner/rotation/CLI checks cover exact audit/causality, failures before target mutation, unit/target/horizon boundaries and repeat-state settlement. No game DPS is computed.
