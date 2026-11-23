# Design

## Context

See proposal.md for motivation. Cargo and web launchers already derive their roots from their own files. The remaining concrete leaks are a tracked machine permission rule, documentation and research source paths, and the native acquisition binary directory shared by four consumers. Checkpoint already owns repository input enumeration and static validation.

## Goals / Non-Goals

**Goals:** preserve ordinary setup without private files; provide one optional supplement and an explicit data-only environment file; prevent accidental reintroduction at checkpoint.

**Non-Goals:** new platform support, rewriting Git history or commit authorship, provisioning secrets, loading local files implicitly, or changing Rust mechanics, WASM, Worker or UI contracts.

## Decisions

- Use ignored `.local/AGENTS.md` as a short supplement named in the root guide. Keep permission rules and `.env` alongside it. Do not use an override file or duplicate the shared guide. Ignore environment files and Python virtual environments; tracked examples must contain variable names and placeholders only.
- Extend `tools/check.py` with opt-in `--env-file`. Parse plain `KEY=VALUE`, comments and optional matching quotes without evaluation, interpolation or multiline syntax. Inherited environment wins. Invalid lines fail before child execution without including their value in errors. Existing commands without the option retain their behavior. No dotenv dependency or automatic secret discovery is needed.
- Native acquisition resolves `NTESIM_CLIENT_BINARIES` lazily when an executable is needed. All current consumers call the same resolver. Pure PE parsing and saved-evidence audits remain usable without a client installation. Missing or nonexistent configuration fails before client I/O.
- Move the reviewed machine formatter rule into `.local/`, remove the tracked rule and repair its documentation. Preserve existing local permissions without installing or broadening grants.
- Replace recorded source file paths with logical source identifiers while retaining public URLs, retrieval dates and hashes. Remove machine layout and one-session networking narratives from documentation. Standard environment variable names and optional platform adapters remain portable; no address or credential values belong in source.
- Extend checkpoint static checks over tracked and nonignored commit-eligible files, including forced tracked ignored files. Reject private configuration paths, concrete Windows/Unix home or checkout paths, proxy endpoint literals and recognizable private keys/tokens. Keep synthetic path-traversal fixtures and public third-party attribution valid. Report category and source location only. Run this boundary for every checkpoint and never reuse its success cache.

## Risks / Trade-offs

- Pattern matching cannot prove absence of all personal data or every secret format; keep the checks conservative, cover concrete regressions, and review diffs. Public attribution and upstream license identities must remain intact.
- A private environment file can contain credentials; do not execute it or print its values, and do not load it during ordinary checks. Local configuration is not included in distributions.
- Existing optional acquisition scripts share imports and a required acquisition workspace. Lazy client resolution avoids breaking saved-evidence tools or parser tests at import time.
- Frozen source metadata changes its package identity. Run the existing package audit and selected native replay checks without claiming new mechanics verification.

## Migration Plan

Preserve this machine's existing formatter rule and client location under `.local/`, then remove their tracked copies. Update consumers and documentation together, test missing/private configuration and explicit environment loading, and run tooling smoke checks in a copied checkout containing spaces with private configuration absent. Run focused regressions and checkpoint, synchronize and archive this change, then checkpoint the final tree before committing. No publication or history rewrite occurs.
