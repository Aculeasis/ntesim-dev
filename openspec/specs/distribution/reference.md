# Release contract

Detailed interface and model reference for this [capability](spec.md). Requirements and scenarios live in that spec; this reference retains formats, bounds and technical rationale.

Version 0.1.0 is package metadata, not a completed release. [Requirements](../product-scope/reference.md) own scope and the [first usable baseline](../product-scope/reference.md#first-usable-baseline); [progress](../../README.md) owns actual readiness. Apply the [modeling policy](../../../docs/DEVELOPMENT.md).

Releases follow the [rolling data policy](../product-scope/reference.md#rolling-data-policy). A fixed Global cutoff, exact patch certification, per-source hash receipts and historical multi-version replay are not release prerequisites. Existing artifact-integrity checks describe the current tools, not a demand to expand archival infrastructure before useful gameplay coverage.

## Release requirements

- Every released character and verified gear owner meets the mandatory DPS baseline under the [relevance policy](../product-scope/reference.md#dps-relevance-and-modeling-boundary), with saved legal configurations, ordinary continuing rotations and equipped-effect cases. Remaining damage-relevant branches are outside those configurations and diagnosed if requested/reached; irrelevant simplifications/omissions are declared with causal rationale and applicability; baseline readiness does not claim full-kit/full-domain completion.
- Practical independent damage/build/event checks from calculations, compatible spreadsheets and combat logic; invalid configuration rejection, deterministic replay and native/WASM parity.
- Usable rotation/build/report/import/export/browser flows; representative performance/resource limits and interruption behavior. Failed/truncated runs cannot enter successful rankings; assumptions stay attached to valid comparisons.
- Fresh portable checkout builds/tests using available product inputs, pinned software dependencies, native/browser behavior parity and retained notices. Existing package audits remain relevant while those package readers are in use.
- Artifact-specific evidence/dependency/artwork distribution review; exclude unresolved distributable material. Original code uses MIT, which does not cover third-party game assets.
- Actual keyboard/focus/contrast/mobile/reduced-motion/error/offline checks and concise user/reproduction docs with executable configs and explicit supported build/action selections.
- Tags/publication/deployment only within explicit user authorization.

Reuse LICENSE/NOTICE/build/WASM/browser tools. Run relevant checks through [the runbook](../../../docs/RUNBOOK.md); historical test counts/model throughput are not release verification. Remaining product work is tracked in the [OpenSpec changes](../../README.md).
