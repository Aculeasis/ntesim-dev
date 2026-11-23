# Budgeted research seed batches and timing sensitivity

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`batch-game SCENARIO.json BATCH.json` executes explicit research crit scenarios over 1..=1000 distinct caller-supplied seeds and 1..=3 distinct timing conditions. This is partial-model experimentation; reports and rows remain incomplete and never establish game DPS or acceptance.

The versioned `scale_recovery_and_hit_delay_v1` hypothesis multiplies only declared recovery and hit-delay durations, using exact rational ceiling to integer microseconds. Multipliers must be >=1; equivalent duplicate fractions fail. Planned command timestamps, cooldowns, switches, buff durations and horizon remain fixed. The runner never silently repairs a newly illegal rotation. A timing collision produces a failed trial. The example declares 1.0, 1.1 and 1.2; this policy is an explicit experiment, not a verified conversion of game animation playback.

Caller budgets bound per-run events (1..=10000), total worst-case events (<=1000000) and sum of retained compact run-receipt bytes (<=64 MiB). Seed count times timing count times event cap must fit before execution starts. Event-cap exhaustion preserves the pending event without processing an extra event. Receipt-cap exhaustion retains the completed prefix and withholds statistics. The receipt byte cap is an output-retention bound, not an operating-system peak-RAM limit.

Each row binds canonical transformed/seeded input and canonical full runner report hashes. Failed/pending trials retain partial runner traces. Engine errors without a report retain a diagnostic and null damage/event metrics; the batch marks event counts incomplete rather than inventing zero. All timing statistics are withheld if any requested trial fails, remains pending, errors or cannot be retained. Complete experiments supply exact minimum, maximum, sum and rational mean, with explicitly approximate sample variance and standard error. These describe variation under the supplied RNG hypothesis; missing/unbounded game-model and timing evidence is separate.

Original scenario bytes, batch bytes, length-prefixed combined input, engine identity and audited package metadata bind the CLI report. Seed order is preserved within each timing condition. Full successful reports replay from transformed inputs and hashes. This batch has no worker scheduling, checkpoint restoration or continuous-state cycle proof.

Optional scenario `model_assumptions` survive timing/seed transformations and are exported at the batch boundary, including incomplete batches and failed partial reports. They describe the same declared model for every trial; sampling statistics measure its RNG variation, not confidence in those hypotheses. See [build binding](../../../docs/mechanics/shared/native-build-projection.md) for metadata bounds and strict rejection.

Exit 0 means the complete declared experiment was produced; exit 2 preserves an incomplete experiment; exit 1 rejects invalid domains/input/integrity. Even exit 0 retains `status: INCOMPLETE` and `acceptance_ready: false`.

From `ntesim/`:

```powershell
python tools/cargo.py build --locked --offline -p ntesim-cli
target/debug/ntesim.exe batch-game data/global-2026-10-04 examples/research-crit.json examples/research-game-batch.json
python tools/check.py -- python tools/verify_game_batch.py
```

Rust tests own packet expansion, damage/RNG rules, explicit domains, work/receipt caps, engine errors, pending hits, exact fraction statistics and immutable timing fields. `tools/verify_game_batch.py` checks 1000 seeds across all three conditions (3000 runs), independently recomputes aggregate statistics from serialized run values and compares unscaled seeds 0/42/999 with separate `run-game` CLI executions. It preserves byte-identical replay, two preflight rejections and six incomplete mutations, including a timing collision. It does not assume one hit per authored action or reimplement Mint damage/PCG in Python. Current run outputs under `target/reviews/q10-research-batch/` record those boundaries and measured CLI wall time including audit/output; they do not establish full gameplay acceptance. Use the verifier and Rust tests as current evidence; generated receipts are disposable run outputs.

The authored [scenario request](../../../reviews/q10-research-batch/scenario-request.json) and [pilot request](../../../reviews/q10-research-batch/pilot-request.json) remain available as verifier inputs; current run outputs stay in the cache directory above.
