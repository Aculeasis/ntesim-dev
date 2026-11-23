# Paired synthetic comparison

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`compare-rotation-synthetic BASE.json BASE.nte CANDIDATE.json CANDIDATE.nte SEEDS.json`
compares two synthetic variants on the same explicit distinct seed list. It uses
the [batch service](../synthetic-batches/reference.md), preserves both complete batch reports
and adds signed per-seed differences. Version: `synthetic-paired-compare-v1`.
All output remains synthetic, regardless of the fixture names or damage values.

## Comparable context and preflight

The variants must have equal requested duration, mode clock, initial clock ID/rate
map, target IDs/initial HP/max HP/shield order and stream-name/sequence map.
Clock declaration order is normalized into a sorted map. The fixture's original
seed can differ because the common request explicitly overrides it. Actors,
action rules, effect values, resources and rotation plans may differ: they are
variant inputs. This is a fixed synthetic comparison context, not a declaration
of legal game builds or a universal comparison of arbitrary scenarios.

Both variants are preflighted before either is simulated. Their combined upper
bound on causal events must fit the same 1,000,000-event budget, rather than
allowing each side to spend that budget separately. Invalid seed lists, context
differences, compilation errors and work limits produce exit 1 without a result.
The ordinary batch input and execution checks still apply.

## Pairing, completeness and direction

Rows pair in request order by the exact seed. Each independent variant starts
from its own declared initial state for each seed. A pair is complete only when
both runs succeed and have zero pending events. Complete pairs retain decimal
signed differences for the batch target metrics, always **candidate minus
baseline**. Failed or pending pairs have null differences; each side's partial
state metrics, diagnostics and replay hashes remain visible in its batch rows.

`eligible_pairs` counts complete pairs, but aggregate `paired_statistics` is
null unless every requested pair completes. The comparison does not silently
drop failed seeds. CLI exit 0 requires completeness; failed/pending ensembles
produce the full comparison report with exit 2. Execution `success` and full
`complete` are separate fields as in batches.

All signed totals use checked i128 arithmetic; overflow rejects aggregation.
Exact count/min/max/sum/mean numerator/denominator accompany explicitly approximate
mean/sample variance/standard error. Variance is computed over the **per-seed
differences**, not by adding side variances or treating the sides as unpaired.
Centered integer arithmetic preserves small differences at large magnitudes
before floating conversion. At one pair, variance/standard error are null.
The standard error assumes independent seed pairs; matching stream selectors
alone does not certify game independence or aligned rolls after a changed call
order. No confidence interval, automatic winner, significance claim, sustainable
cycle claim or validated game DPS is emitted. Final HP and damage have different
interpretations; a positive sign is simply a larger candidate metric.

## Audit and example

The comparison identity hashes the ASCII version tag followed by five exact
inputs in order: baseline fixture/source, candidate fixture/source, seed request.
Each uses an unsigned 64-bit big-endian byte-length prefix. Both nested batch
reports retain their own input hashes, per-seed canonical full-report hashes and
replay metadata. Swapping sides changes comparison identity and reverses deltas.

The example uses `synthetic-random.json` on both sides, with the three-action
`synthetic-random.nte` baseline and the two-action `synthetic-candidate.nte` plan.
They share the same 30-microsecond horizon and initial context. Every candidate
has one fewer settled hit, so per-seed damage differences are -10 or -25 health
quanta. This illustrates paired accounting, not a recommended rotation.

Current tests cover identical variants, signed differences,
constant per-hit changes, mixed failures/pending horizons, context/work rejection,
negative/large-value variance oracles, input order and deterministic replay. Native CLI verification independently recomputes per-seed signed deltas, exact
means, sample variance/standard error, five-input hashes and exit codes 0/1/2.
Saved local results: `target/validation/compare/verification.json`.

The [explicit synthetic candidate search](../synthetic-search/reference.md) now ranks finite
provided lists under a declared objective. Automatic game build/parameter search,
game formula/kit/equipment adapters and useful checked combat comparisons remain
pending. Legacy Reference guide study is optional; synthetic checks alone do not establish
combat-model correctness.
