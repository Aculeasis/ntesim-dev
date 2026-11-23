# Reproducible RNG streams

The Rust core exposes a synthetic simulation service with version
`pcg32-xsh-rr-64-32-v1`. Both synthetic runners bind explicit weighted random
damage to this service; fixtures without streams retain the unused-seed report.
Game distributions, shared/independent crit or proc rolls, draw timing and
native RNG semantics remain unknown and cannot become strict game defaults.

## Algorithm and provenance

PCG32 uses an unsigned 64-bit state and the XSH-RR 32-bit output permutation.
The transition multiplier is 6364136223846793005; the odd increment is
`(sequence << 1) | 1`. All transition arithmetic wraps modulo 2^64.
Initialization sets state to zero, transitions once, adds the seed with
wrapping arithmetic, then transitions again. These two initialization
steps do not count as user draws. Output uses the previous state, truncates
`((state >> 18) ^ state) >> 27` to 32 bits, then rotates right by `state >> 59`.

Algorithm references:

- [PCG minimal C implementation](https://github.com/imneme/pcg-c-basic/blob/master/pcg_basic.c): transition, initialization and bounded rejection algorithm.
- [Upstream demo](https://github.com/imneme/pcg-c-basic/blob/master/pcg32-demo.c): fixed seed 42 and sequence 54.
- [Published usage example](https://www.pcg-random.org/using-pcg-c-basic.html): first six raw words `a15c02b7`, `7b47f409`, `ba1d3330`, `83d2f293`, `bfa4784b`, `cbed606e`.

The algorithm above and golden words define this version, independent of
future upstream changes. Upstream source URLs were inspected, not pinned
to a Git revision. The Rust port carries the upstream Apache-2.0 attribution
and a local license copy in [the RNG notice](../../../licenses/pcg32-NOTICE.txt).
No upstream executable is used and no additional Cargo dependency is added.
Original ntesim code uses MIT; upstream RNG attribution remains separate and must accompany distributions.

## Explicit streams and audit

`Rng::new(seed, sequences)` accepts a caller-supplied ordered map of stream
names to sequence selectors. Names must be nonempty and selectors unique
and at most 2^63 - 1. Rejecting larger selectors avoids silently discarding
the high bit and aliasing streams. An empty map is valid; referencing any
undeclared stream fails. Names identify streams but do not seed/hash them.
All streams use the supplied seed and their explicit selector. Adding or
consuming another stream leaves a stream's future words unchanged. Distinct
selectors do not establish empirical independence of actual game events.

`draw(name)` returns the name, zero-based raw draw index and unsigned word.
The counter is checked before mutation. Cloning the service preserves exact
continuation; serialization records version, seed and each stream's selector,
state and consumed word count in sorted name order. It is an audit snapshot;
there is no untrusted JSON state restoration interface at this checkpoint.

## Bounded sampling

`uniform(name, bound)` accepts 1 <= bound <= 2^32 and returns an integer in
`[0, bound)`. Let threshold = 2^32 mod bound. Raw words below the threshold
are rejected; the accepted word modulo bound is the result. The returned
receipt includes every rejected and accepted word/index in consumption order.
The accepted raw range has size divisible by the bound, avoiding modulo bias.
Even bound 1 consumes one word; bound 2^32 returns the unmodified word.
No floating point conversion or probability rounding is involved.

Each request permits at most 1,024 raw attempts. This is an explicit simulator
termination policy. Counter overflow, unknown streams, invalid bounds or
attempt exhaustion return an error and preserve the entire stream, including
tentative rejections. No partial receipt/result is promoted on failure.
Changing this policy or the algorithm requires a new RNG version.

## Verification and remaining work

Six tests cover the published word sequence, isolated stream advancement,
cloned continuation/audit serialization, selector/bound validation, a known
rejection sequence, and atomic errors. A small exhaustive arithmetic oracle
checks equal residue multiplicity for all bounds 1..256 over an 8-bit domain;
44,800 generated samples check ranges, rejection receipts and draw accounting
across 64 seeds and seven bounds. These checks establish reproducibility and
mapping invariants, not NTE RNG rules or cryptographic security.

## Synthetic effect binding

Fixtures may declare `rng_streams` as a map of names to sequence selectors;
omitting it means no streams, and random effects then fail dependency checks.
`random_damage` specifies target, stream, ordered `outcomes` (each with positive
integer `weight` and nonnegative integer health-quantum `amount`) and local
`hit_delay_us`. The sum of weights must fit 1..=2^32. There is no automatic
normalization, inferred crit multiplier, shared-roll rule or floating probability.
Target/stream/weight dependencies are validated before simulation for each
selected effect, including actor leaf effects. Invalid unselected actor effects
do not enter the selected command's dependency closure.

At effect execution, one bounded sample selects the first cumulative weight
interval containing the sample value. Its receipt records all raw words,
the sampled integer, zero-based outcome index and selected amount. A causal
hit stores that amount and settles against the target's then-current state.
Selection happens at effect time even if the hit is beyond the run horizon.
RNG advances only after hit scheduling succeeds; deadline/causal/ID errors
leave it unchanged and retain a `RANDOM_EFFECT` diagnostic in the partial trace.
Earlier successful effects and an already-admitted actor's cost are retained;
no refund or transaction across an entire action is inferred.

Configured streams add a final `rng` audit snapshot and the version tag to
JSON reports, even when zero words were drawn. With no declared streams the
snapshot is omitted and `rng_version` remains `unused; no random operations`.
Same inputs and seed produce byte-identical reports. Rotation repeats retain
stream state and source/actor origins. The example's three actions consume
the first three golden words, select amounts 25, 25 and 10, and end at HP 140
with 12 causal events. This primitive schedules one target per random effect;
no multi-target or game shared-roll policy is supplied.

Five further core tests and one CLI test cover seeded causal settlement,
invalid dependencies/weights, scheduling/admission failures, pending hits,
unrelated-stream isolation and backwards-compatible deterministic reports.
The [synthetic seed ensemble service](../../../openspec/specs/synthetic-batches/reference.md) now reports explicit
seed lists, failures/pending events, exact totals and approximate uncertainty.
Real game RNG dependencies and game Monte Carlo remain subject to the strict
coverage gate.
