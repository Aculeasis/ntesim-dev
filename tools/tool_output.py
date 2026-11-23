from pathlib import Path
import os
import re
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
LOGS = ROOT / "cache/tool-logs"
MAX_BYTES = 12_000
MAX_LINES = 60
ERROR = re.compile(
    r"\berror(?:\[|:| TS)|\b[A-Za-z]*Error\b|Exception:|^FAIL[: ]|^ERROR[: ]"
    r"|^Traceback|panicked at|^not ok|^✖|FAILED", re.IGNORECASE)
PASS = re.compile(
    r"^\s*(?:ok \d|test .+ \.\.\. ok$|✔|✓|PASS\b|# Subtest:|# tests|# pass"
    r"|# duration_ms|\.{2,}$|Ran \d+ tests|OK$"
    r"|ℹ (?:tests|suites|pass|cancelled|skipped|todo|duration_ms)\b)| passed[.!]?$",
    re.IGNORECASE)


def failure_excerpt(log):
    """Retain diagnostics even when later passing tests bury the error."""
    selected = []
    context = 0
    with log.open(encoding="utf-8", errors="replace") as output:
        while line := output.readline(MAX_BYTES):
            if ERROR.search(line):
                context = 16
            if context:
                if not PASS.search(line):
                    selected.append(line.rstrip())
                context -= 1
                if len(selected) == MAX_LINES:
                    break
    if selected:
        return "\n".join(selected).encode("utf-8")[:MAX_BYTES].decode("utf-8", errors="replace")
    with log.open("rb") as output:
        output.seek(max(0, log.stat().st_size - MAX_BYTES))
        return "\n".join(output.read().decode("utf-8", errors="replace").splitlines()[-MAX_LINES:])


def run_logged(command, *, cwd=ROOT, env=None, label="check", verbose=False):
    """Preserve exit status; successful commands produce no terminal output.

    Use a file rather than capture_output so large reports cannot fill memory.
    Verbose mode streams directly; quiet mode retains stdout and stderr together.
    """
    if verbose:
        try:
            return subprocess.call(command, cwd=cwd, env=env)
        except OSError as error:
            print(f"ntesim: {error}", file=sys.stderr)
            return 1
    LOGS.mkdir(parents=True, exist_ok=True)
    name = re.sub(r"[^a-zA-Z0-9_-]", "-", label)
    log = LOGS / f"{name}-{uuid.uuid4().hex}.log"
    # Nested project wrappers must retain their complete output in this log.
    child_env = {**(os.environ if env is None else env), "NTESIM_TOOL_VERBOSE": "1"}
    with log.open("wb") as output:
        try:
            result = subprocess.call(command, cwd=cwd, env=child_env,
                                     stdout=output, stderr=subprocess.STDOUT)
        except OSError as error:
            output.write(f"{error}\n".encode("utf-8"))
            result = 1
    if result:
        print(f"ntesim: {label} failed (exit {result}); log: {log}", file=sys.stderr)
        excerpt = failure_excerpt(log)
        if excerpt:
            print(excerpt, file=sys.stderr)
    return result if result >= 0 else 1
