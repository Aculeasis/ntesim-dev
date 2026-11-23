# Explicit bounded synthetic search

`search-rotation-synthetic FIXTURE.json SEARCH.json` evaluates an explicitly
provided finite list of rotation sources against one synthetic fixture and
common seed list. Version: `synthetic-candidate-search-v1`. It enumerates the
entire declared list; it does not generate legal game builds, invent action
timing, search an undeclared space or calculate validated game DPS.

## Request and bounds

The schema-1 request contains `seeds`, `objective` and `candidates`. Each
candidate has a unique nonblank `id` (at most 128 UTF-8 bytes) and inline
`source` in the [synthetic rotation language](../synthetic-rotations/reference.md). There are
1..64 candidates. The common seeds follow the [batch contract](../synthetic-batches/reference.md):
1..1,000 explicit distinct u64 values. No entropy, seed replacement, mutation
operator or stopping rule is inferred. The entire request is at most 1 MiB.

The objective explicitly names a declared target, one of `hp_lost`,
`shield_absorbed`, `overkill`, `offered_damage`, `final_hp`, and direction
`minimize` or `maximize`. No default metric or interpretation of a positive
change is supplied. Unknown fields/objectives/targets, duplicate IDs/seeds,
schema/range errors and malformed candidate sources fail with exit 1.

Every candidate uses the same fixture/horizon/targets/clocks/stream selectors;
independent executions start from the original state for each seed/candidate.
All sources, seed lists and event estimates are preflighted before simulation.
The combined candidate-by-seed upper bound must fit 1,000,000 causal events.
The bound is conservative even if later resource failures would terminate a
candidate early. Simulation dependency checks still apply at execution.

## Ranking and incompleteness

Each candidate retains its source SHA256 and complete batch report, including
per-seed input/full-report hashes, diagnostics, pending events, partial metrics
and statistics. `eligible_candidates` counts candidates for which every seed
succeeds with zero pending events. `success` concerns execution; `complete`
also requires no pending events across the whole list.

If any candidate is incomplete, both `ranking` and `best_candidate_ids` are
null. No candidate or failed seed is silently dropped to name a winner.
Such a report exits 2, including successful but truncated/pending execution.
Complete searches exit 0 and return every candidate in objective order.

Scores are empirical means over the exact declared seed list. All denominators
are equal, so ranking compares exact u128 sums, never approximate floats.
The score is reported as a decimal numerator/integer denominator. Exact ties
retain request order and all tied best IDs are reported. Each candidate's
approximate variance/standard error remains available in its batch report.
Ranking is only a property of these observed synthetic means. It does not
establish statistical superiority, a global optimum beyond this list, game
build legality, sustainable cycles or a game recommendation. Use the paired
comparison service when examining difference-sample uncertainty between plans.

## Audit and example

The search input identity hashes the ASCII version tag followed by two inputs
in order: fixture and original request, each prefixed by unsigned 64-bit
big-endian byte length. Separate hashes preserve their exact original bytes.
Inline source bytes are their decoded UTF-8 string; their original JSON escaping
is also bound through request identity. Candidate order, comments, IDs and
objective are part of that identity.

Nested batches use a compact serde JSON Value seed request containing only
`schema_version: 1` and `seeds`. Their own hashes and replay receipts retain that
derived identity. To replay, save a candidate's exact source string as UTF-8 and
use the ordinary fixture with `run-rotation-synthetic --seed U64` as described
in [the batch contract](../synthetic-batches/reference.md#hashes-and-replay).

The example enumerates one, two and three synthetic actor actions over the same
30-microsecond horizon and eight seeds. Maximizing gross HP loss ranks three,
two, one; minimizing it reverses the order. This demonstrates exact finite
selection and does not make the example a legal or recommended game rotation.

Current tests cover exhaustive ranking, min/max, tied best
IDs, failure/pending gates, validation/global event budget, large exact scores
beyond floating precision, order/hash identity and deterministic replay. Native process verification independently checks PCG hit totals, exact Fraction
ordering, per-seed replay hashes, two-input identity and exit codes 0/1/2. Saved
local results are in `target/validation/search/verification.json`.

This explicit synthetic enumerator does not establish native build generation, sustainable game rotations or calibrated team DPS.
