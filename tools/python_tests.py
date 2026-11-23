"""Run independent unittest files in isolated processes sharing four CPUs."""
import argparse
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import subprocess
import sys

from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]


def run_file(path, root=ROOT):
    result = subprocess.run(
        [sys.executable, "-m", "unittest", "discover", "-s", str(path.parent), "-p", path.name],
        cwd=root, capture_output=True,
    )
    return result.returncode, result.stdout + result.stderr


def run_files(paths, root=ROOT, workers=4):
    # Modules patch globals and environment; separate interpreters keep those patches isolated.
    with ThreadPoolExecutor(max_workers=min(4, max(1, workers))) as pool:
        results = list(pool.map(lambda path: run_file(path, root), paths))
    status = 0
    for path, (code, output) in zip(paths, results):
        if code:
            print(f"FAILED: {path.name}", file=sys.stderr)
            sys.stderr.buffer.write(output)
            status = code if not status else status
    return status


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pattern", default="test_*.py")
    args = parser.parse_args()
    limit_cpus()
    paths = sorted((ROOT / "tests").glob(args.pattern))
    if not paths:
        parser.error("no unittest files selected")
    return run_files(paths)


if __name__ == "__main__":
    raise SystemExit(main())
