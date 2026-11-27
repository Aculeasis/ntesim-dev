# ntesim agent guide

Source and documentation stay English. Human instructions take precedence. Continue the selected saved change after interruptions; ask only about material ambiguity or an actual permission boundary.

If `.local/AGENTS.md` exists, read it as an optional machine-specific supplement to this guide. Keep it short; do not duplicate this file or use `AGENTS.override.md`. Private paths, credentials, proxy endpoints and permission grants belong only in ignored `.local/` files or environment variables. Tracked code and instructions must work from any checkout location on supported platforms; see [local configuration](docs/DEVELOPMENT.md#local-configuration).

## Workflow

Use OpenSpec for planned behavior, architecture and tooling changes. `openspec/specs/` owns accepted requirements; `openspec/changes/<name>/` owns a change's proposal, design, deltas and tasks. Read the selected change, relevant capability and only the code, tests and references needed for it. [Workflow](docs/DEVELOPMENT.md#openspec-workflow) explains setup, selection, implementation and archive. Do not create a parallel queue, readiness ledger or completion log.

Keep artifacts current when implementation changes the approach. Requirements describe observable behavior; design explains implementation choices; tasks state how completion is verified. Pure refactors or documentation work can declare `skip_specs: true`; do not invent behavior changes. Small corrections need proportionate documentation. Existing specs are requirements, not evidence that every scenario currently passes.

Use `python tools/openspec.py` for the pinned CLI, including the CLI examples in generated skills: replace the example's `openspec` executable with this wrapper and retain its arguments. This preserves the version and CPU limit without editing generated skills. [Setup](docs/DEVELOPMENT.md#openspec-workflow) also installs the plain command for skill discovery and manual use. `python tools/openspec.py view` shows changes and task progress. Resume the selected change; if several changes are open and none is selected, ask which one. Do not invent work when none is open.

## Product boundaries

One Rust engine serves the native CLI and React Studio through WASM. Rust owns all domain logic and mechanics tests; TypeScript owns presentation, Worker transport and integration checks. Core is independent of content; content owns character and equipment behavior. See [architecture](docs/ARCHITECTURE.md) and the [capability index](openspec/README.md).

Targets are anonymous, stationary, script-authored dummies. Preserve owner-specific recipients and caps; all eligible dummies are in range. No enemy catalogs, enemy AI or geometry. Console stats come from `add stats`; only declared set effects apply automatically. Preserve one shareable script for builds, targets, options and actions.

Use consumed typed Rust constants, exact units and explicit assumptions. Keep raw acquisition inputs and generated receipts in `cache/`; JSON is transport/export, not a runtime content catalog. [Mechanics references](docs/mechanics/README.md) retain sources, rationale, limits and independent calculations. Do not execute game code or untrusted acquisition scripts.

No backward compatibility: update current contracts, consumers and tests together; do not add obsolete-input shims. Preserve the real combat-time axis, deterministic causality/RNG, continuing state and explicit action admission. Unsupported reached damage dependencies and incomplete runs cannot become successful comparisons.

## Checks and delivery

Preserve unrelated edits. Local tools and children share at most four logical CPUs through `tools/cpu_limit.py`; run one validator at a time. Only the GitHub Pages deployment build opts into available runner CPUs with both `GITHUB_ACTIONS=true` and `NTESIM_BUILD_CPUS=all`; smaller caller limits survive. Use `tools/cargo.py`, `tools/web.mjs`, `tools/openspec.py` or `python tools/check.py -- COMMAND ARGS`. Scratch belongs in `cache/`; builds in `target/`. Read bounded excerpts and logs. The workspace holds large raw data, scratch scripts, and caches, so never run full-workspace text searches; search tracked code with `git grep`.

Inspect existing coverage before adding tests. Rust tests own mechanics and independent numeric/causal checks; extend the relevant existing boundary. Integration checks need a distinct transport, package, replay or UI risk. Do not introduce test-count or line-count quotas, copied kit suites or new `tools/verify_*.py` scripts.

Run focused checks, review the stable diff, then `python tools/checkpoint.py run` before committing. [Check selection](docs/DEVELOPMENT.md#choosing-checks) owns native, WASM/Worker smoke and browser boundaries. OpenSpec validation checks documents, not simulator correctness. Reuse valid receipts and rerun affected checks after fixes.

Commit coherent changes. Archive completed OpenSpec changes after checks, keeping specs and code consistent. Delete obsolete documentation and repair its consumers rather than creating forwarding stubs. Publication or deployment requires explicit authorization. Report behavior, checks and material limits.

When merging a completed branch into `main`, if the branch has only a single commit with a proper message, fast-forward merge it (`--ff-only`). If the branch has multiple commits, use squash merge and create one descriptive commit unless the user explicitly requests another strategy.

For explicitly requested subagents, use the latest available Luna model with `reasoning_effort="xhigh"`.
