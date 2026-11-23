# Browser trial lifecycle

The [application contract](../openspec/specs/studio-application/reference.md) owns acceptance;
[ADR 0021](decisions/0021-browser-trial-prefix-resume.md) records continuation.
The Worker transports the shared Rust live executor and trial collector.

Stop terminates the Worker immediately. Only delivered completed samples remain
available; an in-flight seed and undelivered completed work are discarded. The
retained result remains partial, exportable and bound to its executed source.
Resume is available for a stopped, nonempty incomplete prefix. It does not use
the editor draft, and it does not turn failed trials into resumable executions.

Resume loads a fresh Worker and asks Rust to validate the exact compiled artifact
against current engine/content/source identities. Rust reruns each retained seed,
checks eligibility and every retained sample field, and rebuilds all aggregates
from the reproduced native reports. It publishes the new session only after the
entire prefix matches. A rejected restore clears any older prepared session.
The returned result retains that validated compiled artifact, including source
limits. New work starts at the next contiguous seed. Prefix samples appear exactly once;
changing seed chunk boundaries does not change samples or native statistics.

Reconstruction costs CPU proportional to the retained trials; repeated small
stops can repeat that cost. Elapsed active wall time includes reconstruction and
subsequent work, added to the retained run time; time spent stopped is excluded.
This is trial-batch continuation, not an in-progress combat checkpoint. Each
seed still executes the complete authored live rotation with continuing state
within that seed. Native input-boundary continuation remains separately owned by
[native lifecycle](NATIVE_LIFECYCLE.md).

Task IDs reject obsolete responses, and Worker instance guards reject late fatal
errors and responses from replaced transports. Callback/offset/transport errors
terminate the Worker and remain visible failures. Only explicit cancellation
uses the cancellation error type; diagnostic prose containing "cancelled" cannot
change a failure into Stopped. The next request creates a fresh Worker.

The Rust WASM adapter tests verify prefix reconstruction, changed identities,
sample corruption, invalid prefixes and seed-chunk equality. Actual Worker tests
retain native aggregate/report agreement, Stop/resume/retry and visible failures;
controller tests cover stale transports and callback/offset recovery. Actual
production browser measurements, accessibility and offline acceptance are in
[browser performance](BROWSER_PERFORMANCE.md).
