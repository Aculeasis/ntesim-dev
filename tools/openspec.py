"""Run the pinned OpenSpec CLI; `setup` installs it into the ignored project cache."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]
VERSION = "1.16.1"
CACHE = ROOT / "cache" / "openspec-cli"
PACKAGE = CACHE / "node_modules" / "@fission-ai" / "openspec"


def installed():
    try:
        return (json.loads((PACKAGE / "package.json").read_text(encoding="utf-8"))
                .get("version") == VERSION and (PACKAGE / "bin" / "openspec.js").is_file())
    except (OSError, ValueError):
        return False


def main(argv=None):
    limit_cpus()
    args = sys.argv[1:] if argv is None else argv
    node = shutil.which("node")
    if not node:
        print("OpenSpec requires Node.js >=22.12.0 on PATH.", file=sys.stderr)
        return 1
    version = subprocess.run([node, "--version"], capture_output=True, text=True, check=True)
    if tuple(int(part) for part in version.stdout.strip().lstrip("v").split(".")) < (22, 12, 0):
        print("OpenSpec requires Node.js >=22.12.0.", file=sys.stderr)
        return 1
    env = dict(os.environ)
    env.setdefault("OPENSPEC_TELEMETRY", "0")
    env.setdefault("OPENSPEC_CONCURRENCY", "4")
    if args == ["setup"]:
        if not installed():
            npm = shutil.which("npm.cmd" if os.name == "nt" else "npm")
            if not npm:
                print("OpenSpec setup requires npm on PATH.", file=sys.stderr)
                return 1
            code = subprocess.call([npm, "install", "--prefix", str(CACHE), "--ignore-scripts",
                                    "--no-audit", "--no-fund", f"@fission-ai/openspec@{VERSION}"],
                                   cwd=ROOT, env=env)
            if code:
                return code
        if not installed():
            print("OpenSpec setup did not install the pinned CLI.", file=sys.stderr)
            return 1
        print(f"OpenSpec {VERSION} ready in cache/openspec-cli")
        return 0
    if not installed():
        print("Run `python tools/openspec.py setup` to install the pinned CLI.", file=sys.stderr)
        return 1
    return subprocess.call([node, str(PACKAGE / "bin" / "openspec.js"), *args], cwd=ROOT, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
