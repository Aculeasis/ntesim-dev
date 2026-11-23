# Native performance acceptance

F1-F3 native acceptance uses the ordinary CLI, continuing source recipes and a
bounded authored search. [Lifecycle](NATIVE_LIFECYCLE.md) declares cooperative
cancellation boundaries and deterministic replay costs. Browser CPU/memory and Worker lifecycle have their own [integration checks](BROWSER_PERFORMANCE.md).

## Reproduce

Build the current native executable, then choose a new cache output directory:

```powershell
python tools/cargo.py build -p ntesim-cli --locked --offline
python tools/check.py -- python tools/benchmark_native_wasm.py --native-only --iterations 3 --output target/reviews/native-performance-new
```

The measurement fails for stale engine/schema identities, failed/incomplete
ordinary workloads, changed inputs/executables during collection, corrupt output,
resume mismatch or an exceeded budget. A deliberately cancelled prefix is a
separately labelled exit-2 sample, never a completed scoring input. Native output,
per-process profiles, a relocated checkpoint and `measurement.json` stay outside
Git. The receipt binds executable, engine source, tools and input hashes.

`--profile NEW_JSON_PATH` on the CLI records command time and JSON encoding time;
source/search commands also expose processed events and report/checkpoint bytes. Existing profile files
are refused without truncation. Command time includes input IO, compilation,
simulation and conversion to a JSON value. Encoding time measures an additional
pretty-JSON pass to a counting sink, excluding stdout IO and a second full buffer.
Measured output bytes match actual stdout, including the trailing newline.
Whole-process CPU/peak residency include startup, ordinary stdout IO and this
opt-in instrumentation. Windows uses process kernel/user times and peak working
set; POSIX uses per-child `wait4` CPU/peak RSS (platform-adjusted units).
Serialized retention bytes describe portable output size, not the live Rust heap;
peak residency supplies the separate process-memory measure.

## Recorded workload and budgets

Measured 2026-11-06 on Windows 11 build 26300, AMD64 Family 26 Model 68 Stepping 0
(AuthenticAMD), 12 logical CPUs available, at most four assigned through
`tools/cpu_limit.py`. Python 3.13.1, pinned Rust 1.98.1, default development build
with repository dependency optimizations and debug assertions. These are local
native acceptance measurements, not portable throughput guarantees.

- Executable: `target/debug/ntesim.exe`, SHA-256
  `f518acbba27ad017d9ac0777f46b4fad046c00319e140eed8b356f92e82c5487`.
- Engine source SHA-256:
  `67ec2ff3a442e141da28869d9684a8f9faa3b4e6465a28c3924484ffa61bc577`.
- Receipt directory: `target/reviews/native-performance-f1-accepted/`.
  `measurement.json` retains all 17 samples, input/tool hashes, exact byte counts
  and per-process budget results. Raw stdout/report payloads remain outside Git.

All samples passed these per-process practical budgets, fixed in the measurement
tool: wall and CPU each <=30s, peak residency <=512 MiB, JSON encoding <=5s,
serialized report <=64 MiB and <=100,000 processed events. A 60s collection timeout
kills/reaps a stuck child and fails the measurement. These budgets apply to the
declared bounded workloads; they do not promise bounded memory for arbitrary
authored horizons or enumerate every team/build.

The table takes the worst of three independent processes per ordinary workload;
MiB means 1,048,576 bytes. Source/event counts are stable across those repetitions.

| Workload | Events | Wall s | CPU s | Peak MiB | Command ms | Encoding ms | Report MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| [Daffodil/Zero](../examples/baselines/daffodil.nte), two continuing 150s periods/two dummies | 1,246 | 1.900 | 1.484 | 99.9 | 480.6 | 94.5 | 13.27 |
| [Nanally/Zero](../presets/native/nanally-full-kit.nte), two continuing full-kit cycles | 369 | 0.555 | 0.547 | 50.5 | 203.2 | 39.5 | 5.01 |
| [Incoming shields](../examples/incoming-shield-cycles.nte), two shield periods/named off-field delivery | 58 | 0.056 | 0.047 | 19.3 | 24.7 | 2.2 | 0.32 |
| [Bounded search](../examples/native-source-search.json), two legal refinements x two seeds | 2,012 | 2.713 | 2.703 | 149.2 | 1,093.3 | 137.5 | 22.88 |
| [Mint](../examples/mint-native-cycles.nte), two continuing 60s periods | 503 | 0.582 | 0.547 | 54.7 | 183.7 | 32.6 | 4.95 |

The search retains 12,330,806 compact receipt bytes; its serialized checkpoint is
22,912,579 bytes. These differ from the full report because summaries/request
metadata and pretty whitespace have separate costs. Daffodil's 13.27 MiB report
and 100 MiB peak show that causal observation retention is material; bounded
events/receipt budgets should remain explicit for larger workloads.

The additional Mint cancellation sample stops after 13 complete source inputs,
retaining 60 processed events and a 225-byte serialized checkpoint token. It costs
0.111s wall/0.094s CPU, 27.7 MiB peak and a 0.63 MiB incomplete prefix. Resume costs
0.563s wall/0.563s CPU and 55.0 MiB peak, yielding the same complete executed source
receipt as uninterrupted execution: 503 events, unchanged RNG/resources/causal
output. Replay CPU is real additional work; prefix and resume together cost more
than one full run. Rust checks additionally cover repeated chunks, lexical state,
Hotori Time Stop, incoming/shields and incompatible identity/prefix rejection.

No game-fidelity, release-build speed, browser lifecycle or wall-clock cancellation
latency claim follows from these native results. In-progress inputs, lexical work
and final event draining remain bounded by their declared engine limits.
