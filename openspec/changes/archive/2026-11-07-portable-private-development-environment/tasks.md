# Tasks

## 1. Private configuration boundary

- [x] 1.1 Ignore local instructions, permissions, environment files and virtual environments; reference `.local/AGENTS.md` from the shared guide, migrate the existing machine formatter rule locally and repair its documentation. Verify Git ignores local files and the shared tree contains no machine rule.
- [x] 1.2 Add explicit data-only `--env-file` loading to `tools/check.py`, document precedence and syntax, and verify literal values, malformed-input errors, inherited overrides and child execution using focused runner tests.

## 2. Portable acquisition and provenance

- [x] 2.1 Resolve client binaries from `NTESIM_CLIENT_BINARIES` in every native acquisition consumer, preserve the local setting privately, and document invocation. Extend existing acquisition/parser coverage for missing, invalid and explicit directories without executing game code.
- [x] 2.2 Remove concrete developer paths from source metadata and acquisition docs; replace session-specific networking narratives with portable setup guidance. Verify source URLs/hashes remain intact and the frozen package audit passes.

## 3. Repository regression boundary

- [x] 3.1 Extend checkpoint static validation and existing checkpoint tests to reject commit-eligible private files, machine paths, proxy endpoints and recognizable credentials without echoing values; preserve synthetic path-rejection fixtures and public attribution. Verify every checkpoint plan selects the uncached privacy boundary.

## 4. Integration acceptance

- [x] 4.1 Run focused tooling tests and a smoke run from a relocated checkout with spaces and no private configuration; verify explicit environment loading and actionable missing-client errors there.
- [x] 4.2 Review the stable diff, validate OpenSpec and run `python tools/checkpoint.py run`; resolve failures and retain valid receipts before completing tracked tasks.

## Workflow follow-up

- Synchronize requirements and archive the completed change.
- Validate archived artifacts, run checkpoint on the final tree, and commit the coherent changes without publishing or rewriting Git history.
