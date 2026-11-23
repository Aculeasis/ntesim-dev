"""Run a development check quietly; use --verbose for full debugging output."""
import argparse
import os
from pathlib import Path
import re
import sys

from tool_output import run_logged
from cpu_limit import limit_cpus


def environment_file(path, inherited):
    """Read literal assignments; never evaluate code or override inherited values."""
    values = {}
    for number, line in enumerate(Path(path).read_text(encoding="utf-8-sig").splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        key, value = key.strip(), value.strip()
        if not separator or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key) or "\0" in value:
            raise ValueError(f"environment file line {number}: expected KEY=VALUE")
        if os.name == "nt":
            key = key.upper()
        if value[:1] in ("'", '"'):
            if len(value) < 2 or value[-1] != value[0]:
                raise ValueError(f"environment file line {number}: unmatched quote")
            value = value[1:-1]
        if key in values:
            raise ValueError(f"environment file line {number}: duplicate variable")
        values[key] = value
    return {**values, **inherited}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--env-file", type=Path, help="explicit literal KEY=VALUE file; inherited values win")
    parser.add_argument("command", nargs=argparse.REMAINDER,
                        help="command and arguments after --; no shell expansion")
    args = parser.parse_args()
    command = args.command
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("a check command is required after --")
    env = None
    if args.env_file is not None:
        try:
            env = environment_file(args.env_file, os.environ)
        except OSError:
            parser.error("cannot read selected environment file")
        except UnicodeError:
            parser.error("environment file must be UTF-8")
        except ValueError as error:
            parser.error(str(error))
    if command[0] == "python":
        command[0] = sys.executable
    limit_cpus()
    if env is not None:
        # Keep CPU limits applied above in the child environment too.
        env = {**env, **os.environ}
    return run_logged(command, label=Path(command[0]).stem,
                      verbose=args.verbose or os.environ.get("NTESIM_TOOL_VERBOSE") == "1", env=env)


if __name__ == "__main__":
    raise SystemExit(main())
