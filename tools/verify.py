"""Unified verification entrypoint for ntesim.

Allows running specific kit verifications, categories, parity oracles, or the complete suite:
  python tools/verify.py edgar
  python tools/verify.py --kit mint
  python tools/verify.py --parity
  python tools/verify.py --category kits
  python tools/verify.py --all
"""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import sys

from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

import verify_all


def main() -> int:
    limit_cpus()
    parser = argparse.ArgumentParser(
        description="Unified runner for ntesim verification oracles and kit contracts."
    )
    parser.add_argument(
        "target",
        nargs="?",
        default=None,
        help="Optional target name (e.g. 'edgar', 'wasm_parity', 's1', 'sources')",
    )
    parser.add_argument(
        "--kit",
        type=str,
        default=None,
        help="Run kit verification for a specific character (e.g. 'edgar', 'mint', 'sakiri')",
    )
    parser.add_argument(
        "--parity",
        action="store_true",
        help="Run WASM parity verification (tools/verify_wasm_parity.py)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Run all discovered verifiers",
    )
    parser.add_argument(
        "--category",
        choices=["kits", "sources", "scenarios", "mechanics", "runtime", "all"],
        default=None,
        help="Run all verifiers in the given category",
    )
    parser.add_argument(
        "--filter",
        "-f",
        type=str,
        default=None,
        help="Filter verifiers by substring or regex",
    )
    parser.add_argument(
        "--list",
        "-l",
        action="store_true",
        help="List matching verifiers without executing",
    )
    parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="Stop immediately after the first failure",
    )
    parser.add_argument(
        "--jobs",
        "-j",
        type=int,
        default=4,
        help="Concurrent processes (1-4, default: 4)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Print stdout/stderr of each verifier",
    )
    args = parser.parse_args()

    verifiers = verify_all.discover_verifiers()

    # Determine filter / target
    if args.parity:
        verifiers = [v for v in verifiers if "wasm_parity" in v.name]
    elif args.kit:
        verifiers = [v for v in verifiers if v.name == f"verify_{args.kit}_kit.py" or f"verify_{args.kit}" in v.name]
    elif args.target:
        target = args.target.lower().strip()
        if target in ["kits", "sources", "scenarios", "mechanics", "runtime"]:
            verifiers = [v for v in verifiers if verify_all.categorize(v) == target]
        elif target == "all":
            pass
        elif target == "parity":
            verifiers = [v for v in verifiers if "wasm_parity" in v.name]
        else:
            # Check exact kit match or general match
            exact_kit = [v for v in verifiers if v.name == f"verify_{target}_kit.py"]
            if exact_kit:
                verifiers = exact_kit
            else:
                verifiers = [v for v in verifiers if target in v.name.lower()]
    elif args.category and args.category != "all":
        verifiers = [v for v in verifiers if verify_all.categorize(v) == args.category]

    if args.filter:
        pattern = re.compile(args.filter, re.IGNORECASE)
        verifiers = [v for v in verifiers if pattern.search(v.name)]

    if args.list:
        print(f"Discovered {len(verifiers)} matching verifiers:")
        for v in verifiers:
            print(f"  [{verify_all.categorize(v):<9}] {v.name}")
        return 0

    if not verifiers:
        print("No verifiers matched the given target or filters.")
        return 1

    # Delegate execution
    if len(verifiers) == 1 and not args.all:
        v, passed, duration, output = verify_all.run_single(verifiers[0], verbose=True)
        status = "PASS" if passed else "FAIL"
        print(f"[{status}] {v.name} ({duration:.2f}s)")
        if output.strip():
            print(output.strip())
        return 0 if passed else 1

    workers = max(1, min(4, args.jobs))
    print(f"Running {len(verifiers)} verifiers with {workers} worker(s)...")

    start_all = verify_all.time.perf_counter()
    passed_count = 0
    failed_count = 0
    failures: list[tuple[Path, str]] = []

    with verify_all.ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(verify_all.run_single, v, args.verbose): v for v in verifiers}
        for future in verify_all.as_completed(futures):
            verifier, passed, duration, output = future.result()
            status = "PASS" if passed else "FAIL"
            print(f"  [{status}] {verifier.name} ({duration:.2f}s)")
            if passed:
                passed_count += 1
            else:
                failed_count += 1
                failures.append((verifier, output))
                if args.fail_fast:
                    print("\n--fail-fast triggered, aborting remaining tests.")
                    executor.shutdown(wait=False, cancel_futures=True)
                    break
            if args.verbose and output.strip():
                print(output.strip())

    total_time = verify_all.time.perf_counter() - start_all
    print(
        f"\nCompleted in {total_time:.2f}s: {passed_count} passed, {failed_count} failed, {len(verifiers)} total."
    )

    if failures:
        print("\nFailed verifiers:")
        for v, output in failures:
            print(f"\n--- {v.name} ---")
            print(output.strip())
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
