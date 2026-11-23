# Synthetic rotation timelines

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

`synthetic-rotation-v1` is the first text parser/compiler for explicit synthetic timelines. It compiles in memory and uses the existing event/state runner; there is no filesystem, network or ambient time in the core. Game rotations still require actor/action contracts, cooldowns, recovery/switch legality, stat/formula bindings and independently validated adapters. This language does not make those policies implicit.

The CLI takes two UTF-8 files:

```powershell
python tools/cargo.py run --locked --offline -p ntesim-cli -- run-rotation-synthetic examples/synthetic-rotation.json examples/synthetic-rotation.nte
```

The JSON fixture uses the same required environment fields as `run-synthetic`, with `actions` replacing `commands`. The `actions` map associates names with ordered arrays of synthetic effects (damage, formula_damage, random_damage, invoke_action, observe_state, heal, shield, gain, spend or set_rate). Formula damage has a separate [arithmetic/unit/audit contract](../../../docs/mechanics/shared/exact-scaling-arithmetic.md); optional actor definitions and invocation have an [action admission/effect contract](../../../docs/mechanics/shared/action-admission.md). Selected action names must be ASCII identifiers and their arrays must be nonempty. Unknown JSON fields fail. Only `schema_version=1`, `content_id=synthetic-v1` and `data_status=synthetic` can run. Selected effects are validated against the fixture's clocks, targets and resources before event execution; unused actions do not enter its execution dependency closure.

## Grammar

```text
program   = statement*
statement = "clock" integer ";"
          | "wait" integer ";"
          | "do" identifier ";"
          | "repeat" integer "{" statement+ "}"
integer   = ASCII decimal digits (unsigned, no unit suffix)
identifier = ASCII letter or underscore, followed by ASCII letters,
             digits, underscores, dots, colons or hyphens
```

Whitespace is ASCII; `#` comments continue to the next newline and may contain Unicode text. Keywords are case-sensitive. Semicolons are required after clock/wait/do; repeat blocks have no trailing semicolon. Positions are one-based lines and Unicode scalar columns, with tabs counting as one column. LF and CRLF inputs are accepted. A rotation must produce at least one effect command.

## Timeline semantics

- Each declared clock starts with a timeline cursor of zero. `clock ID;` selects an existing clock without changing its cursor. No default clock is supplied.
- `wait N;` advances that clock's cursor by N integer microseconds with checked arithmetic. This is a technical representation, not a claim of measured game precision.
- `do NAME;` emits one command for each defined effect at the selected clock's current cursor, in definition order. It adds no implicit time, input gap, action lock or resource wait. Damage's explicit `hit_delay_us` remains a local delay from the effect's execution.
- `repeat N { ... }` expands the body N times. Cursors and the selected clock persist through iterations and after the block. Runtime pools, shields and HP also persist throughout the run. A multi-effect action uses ordinary ordered effect transitions; it is not a transaction with rollback across effects.
- Cursors are independent local deadlines from combat start. Moving to another clock does not synchronize it to the previous one. Source order determines scheduling IDs for ties; the scheduler determines real execution order using rates and pauses. Rate changes preserve queued local deadlines. This version has no state-dependent control flow.
- The runner's real-time horizon is inclusive. Future or paused events remain visible through `pending_events`. A successful report means no execution error through that horizon; it does not certify that every planned command ran.

The parser rejects missing delimiters, unknown statements, unsigned-integer overflow, clock ID overflow, duplicate/unknown clocks, an omitted clock selection, unknown/empty selected actions, empty repeat blocks and time overflow. Hard technical limits are 1 MiB of source, 32 repeat nesting levels, 1..10,000 iterations per repeat, 100,000 parsed statements, 100,000 expanded statement visits and 100,000 generated effect commands. Expanded visits also bound repeat bodies that emit no commands. Syntax/compile errors return a line and column and do not execute a partial plan.

## Audit and quality

Results retain all synthetic runner fields, including its explicit quality, causal command/effect/hit trace, final state, engine fingerprint, seed and clock readings. `rotation.origins` maps every zero-based `command_index` to its action, zero-based effect index, source position and one-based outermost-to-innermost repeat iterations. State diagnostics such as `commands/0` join to this map. Shortages preserve the failed partial trace; they do not wait or become zero spending.

`rotation.fixture_sha256` and `rotation.source_sha256` hash the exact input bytes separately. The top-level `input_sha256` hashes the ASCII language-version tag followed by each input's unsigned 64-bit big-endian byte length and bytes, fixture first and source second. Changes to comments, line endings or fixture formatting therefore change input identity even when the timeline is unchanged. This is an input identity, not a game-data integrity or readiness claim.

Optional CLI `--seed U64` adds `rotation.seed_override` and appends ASCII `seed-override-v1` plus the big-endian unsigned seed to that identity. Original input-byte hashes remain unchanged. Ordinary runs without the flag keep the original identity/metadata. The [seed ensemble service](../synthetic-batches/reference.md) uses the same explicit override and fresh state for each seed.

CLI exit codes remain 0 for a run without execution errors, 2 for a state/scheduler failure with a partial report, and 1 for IO/schema/compilation errors. Explicit `rng_streams` and weighted `random_damage` effects use the seed, preserve stream state across repeats and retain raw draw receipts; fixtures without streams keep the unused-seed report. See [the RNG contract](../../../docs/mechanics/shared/seeded-randomness.md).

Current core/CLI checks cover parity with the independent timestamped JSON fixture and final state, nested repeats/source positions, independent cursors, pauses/rate changes, malformed syntax, overflow/expansion limits, two-input reproducibility and synthetic/shortage boundaries. These checks establish technical timeline behavior only; no game DPS is computed.
