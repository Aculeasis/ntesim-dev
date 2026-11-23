# Synthetic seed ensembles

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`batch-rotation-synthetic FIXTURE.json ROTATION.nte SEEDS.json` runs the prepared
synthetic rotation once per explicitly supplied seed. The request is schema 1
with a `seeds` array of 1..1,000 distinct unsigned 64-bit integers. No seed is
chosen from wall time or implicitly incremented. Request order is retained;
duplicates, unknown fields, invalid schema/range and requests over 1 MiB fail
before execution. Batch version is `synthetic-seed-batch-v1`.

Compilation is shared; execution starts from a fresh copy of the input state
for every seed. Actor readiness, pools, targets, shields, clocks, queues and
streams never carry between independent runs. Rotation repeats inside a run
remain continuous. The implementation is serial and uses no ambient entropy,
threads, IO or system time in core.

## Completeness and work limits

Each row reports execution success, pending events, clocks, diagnostics,
seed, effective input hash, full report hash and per-target exact metrics.
`complete` requires both successful execution and zero pending events. A
successful truncated horizon with future/paused commands or hits is not a
complete sample. The report counts requested/eligible runs and retains all
rows, including failures and their partial totals.

Aggregate statistics are all-or-none: any failed or pending run makes
`target_statistics` null. This avoids a success-only mean concealing a
seed-dependent failure. All complete runs are required; no retry, exclusion
or seed replacement is performed. Batch `success` describes execution alone;
batch `complete` additionally describes pending events. CLI exit 0 requires
batch completeness; incomplete batches return a JSON report and exit 2.
IO/schema/compilation/preflight errors exit 1.

Before simulation, selected commands produce a conservative upper bound on
causal events: two per root command (command/effect), plus each actor leaf
effect and each possible damage hit. Multiplying by seed count must not exceed
1,000,000 events. Unknown actor actions and nested invocations fail preflight.
The estimate does not lower itself for a shorter horizon or likely failures.
This is an event-work limit, not a benchmark or bound on input allocation.

## Metrics and uncertainty

Per-target metrics are `final_hp`, cumulative `hp_lost`, `shield_absorbed`,
`overkill` and `offered_damage`. The latter is HP loss + shield absorption +
overkill over successful settled hit receipts. Failed hits and pending hits
contribute nothing; successful earlier hits remain in partial rows. Healing
does not subtract previous HP loss. These are synthetic health quanta; no
game DPS, units conversion or duration denominator is inferred.

Totals accumulate with checked u128 arithmetic and serialize as decimal
strings. Complete batches report exact count, min, max, sum and mean as an
unreduced numerator/denominator. Floating fields are explicitly `approximate`:
mean, unbiased sample variance (denominator n-1) and standard error
`sqrt(sample_variance / n)`. The latter two are null at n=1. Integer centering
before floating conversion preserves small differences between large totals;
large differences and final floats still have finite precision. Variance is
clamped to zero if floating accumulation introduces a tiny negative result.
The standard error's statistical interpretation assumes independent samples;
distinct seeds alone do not certify that assumption for game events. There is
no confidence interval, significance/ranking assertion or validated game RNG.

## Hashes and replay

The batch input hash is SHA256 of the ASCII batch-version tag followed by
three inputs in order: fixture, rotation and seed request. Each input is
prefixed with its unsigned 64-bit big-endian byte length. Separate hashes
record the exact original bytes of all three inputs. Seed order, formatting
and comments are part of identity.

Replay one row with
`run-rotation-synthetic FIXTURE.json ROTATION.nte --seed U64`. The effective
rotation input identity appends ASCII `seed-override-v1` and the seed's unsigned
64-bit big-endian bytes to the ordinary rotation identity. Metadata adds
`rotation.seed_override`; original fixture/source hashes remain unchanged.
Even overriding with the original seed produces a distinct effective identity.
Ordinary runs without the flag retain the previous hash and JSON structure.

Each row's `report_sha256` hashes the full seeded rotation report after
conversion to serde JSON Value with sorted object keys, serialized compactly
without whitespace. This canonicalization is reproducible from the individual
CLI JSON result; it is not a hash of pretty terminal bytes. Full traces are
available through replay rather than repeated in the batch summary. The hash
and engine source fingerprint also bind the seed override and source origins.

## Verification and scope

Current checks cover an independent variance oracle, adjacent large
integer values, exact totals/replay for every seed, mixed failures, pending
horizons, gross damage/HP/shield/overkill separation, request/work limits and
byte/order identities. One CLI test checks batch reporting and seeded replay.
Native process verification checks exit codes
0/1/2, byte-identical batches, individual canonical report hashes, independent
PCG damage totals and Python exact mean/sample-variance oracles; results are
saved locally in `target/validation/batch/verification.json`.

This is a synthetic ensemble foundation. Native gameplay coverage and continuing cycles remain separate from these synthetic batches; explicit partial-model research RNG/batches already exist. Legacy Reference guide study is optional. The existing browser is a research interface; these checks do not establish gameplay readiness.

The [paired comparison service](../synthetic-comparison/reference.md) now uses this foundation
for two variants with the same context/seeds and one combined event budget.
