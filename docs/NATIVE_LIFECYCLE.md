# Native cancellation and continuation

`run-script` supports cooperative cancellation at complete authored input
boundaries. `--stop-after-inputs N` completes at most N additional inputs before
returning an incomplete retained prefix and a `checkpoint` object; N=0 stops
before a new input. `--cancel-file PATH` checks file existence at each supported
boundary. Create the file to request cancellation; remove it before continuing.
File access errors are reported. An ordinary run without these options retains
the existing source response format.

```powershell
python tools/check.py -- target/debug/ntesim.exe run-script examples/mint-native-cycles.nte --stop-after-inputs 13
```

Save the returned `checkpoint` object as JSON, then continue on the same source:

```powershell
python tools/check.py -- target/debug/ntesim.exe run-script examples/mint-native-cycles.nte --resume target/checkpoint.json
```

Controlled runs return `{execution, checkpoint}`. A cancelled prefix retains all
completed source occurrences, exact damage, resource/state receipts and pending
work counts. `execution.report.execution_completed` and `game_dps_computed` are
false; the CLI exits 2. Complete successful continuation exits 0 and has a null
checkpoint. Failed admissions, work/event limits and incomplete horizons have no
checkpoint and remain excluded from successful comparisons. IO/identity/prefix
errors exit 1. Completion does not imply strict gameplay readiness.

Supported boundaries are after initialization at t=0, between complete authored
actions/waits/target controls and before the final event drain. Native owner input
windows can yield with a montage and descendant packets still pending. An implicit
switch and its following action remain one source occurrence. Cancellation never
partially commits an authored input. Long inputs/waits, lexical control evaluation
and final draining finish or hit their configured limits before returning; there
is no wall-clock cancellation-latency guarantee. Reconstruction also finishes the
saved prefix before checking a new cancellation signal.

Checkpoints use [deterministic replay](decisions/0020-source-replay-checkpoints.md),
not serialized observation frames. Resume requires byte-identical authored source,
compiled builds/scenario, seed, engine/content identity and source/execution limits.
It recreates pending events, owner clocks, RNG streams, lexical scopes and work
counters from the beginning, verifies the saved prefix and continues that same
live trial. It neither starts a fresh combat cycle at the boundary nor clears
resources/cooldowns. A token can move directories; the source must still be supplied.
Changed inputs or corrupted/unreachable prefixes reject. Replayed work costs CPU
and is not counted twice in the resulting combat report. Many short segments can
therefore cost quadratic total execution time. Tokens are reproducibility checks,
not signed data from a trusted sender.

`PreparedSourceScenario::run_controlled` exposes the same boundary callback to
native callers; a callback may check an atomic cancellation flag. No browser
Worker lifecycle acceptance is claimed by these native checks.

[Native performance acceptance](NATIVE_PERFORMANCE.md) records the measured
prefix/resume cost, environment, artifact identities and practical budgets.
