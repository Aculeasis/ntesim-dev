"""ntetool - Unified CLI tool for verification, linting, syntax, and link checks in ntesim."""
from __future__ import annotations

import argparse
from concurrent.futures import as_completed, ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import tomllib
from urllib.parse import unquote, urlsplit

from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]

# Known weapon verifier mappings to friendly short names
WEAPON_ALIASES = {
    "far_side_moon": "verify_far_side_moon.py",
    "koinobori": "verify_koinobori_sources.py",
    "oraora": "verify_oraora_source.py",
    "spider": "verify_spider_sources.py",
    "forgotten": "verify_forgotten_sources.py",
    "gold_record": "verify_gold_record_sources.py",
    "bopu": "verify_bopu_sources.py",
    "bit_game": "verify_bit_game_sources.py",
    "raging_flames": "verify_raging_flames_sources.py",
    "passives": "verify_weapon_passive_values.py",
    "weapon_passives": "verify_weapon_passive_values.py",
    "groups": "verify_weapon_groups.py",
    "weapon_groups": "verify_weapon_groups.py",
    "curves": "verify_universal_weapon_curves.py",
    "weapon_curves": "verify_universal_weapon_curves.py",
    "refinement": "verify_refinement_binding.py",
    "cli": "verify_weapon_cli.py",
    "weapon_cli": "verify_weapon_cli.py",
}


def discover_verifiers() -> list[Path]:
    tools_dir = ROOT / "tools"
    return sorted(
        p for p in tools_dir.glob("verify_*.py")
        if p.name not in ("verify_all.py", "ntetool.py")
    )


def categorize(verifier: Path) -> str:
    name = verifier.stem
    if name.endswith("_kit"):
        return "kits"
    if "sources" in name or name.endswith("_source"):
        return "sources"
    if re.match(r"verify_s\d_", name):
        return "scenarios"
    if "weapon" in name or "gear" in name or "skill" in name or name in [
        Path(f).stem for f in WEAPON_ALIASES.values()
    ]:
        return "mechanics"
    return "runtime"


def discover_kits() -> dict[str, Path]:
    tools_dir = ROOT / "tools"
    kits = {}
    for p in sorted(tools_dir.glob("verify_*_kit.py")):
        char = p.stem.removeprefix("verify_").removesuffix("_kit")
        kits[char] = p
    return kits


def discover_weapons() -> dict[str, Path]:
    tools_dir = ROOT / "tools"
    weapons = {}
    for alias, filename in sorted(WEAPON_ALIASES.items()):
        target = tools_dir / filename
        if target.is_file():
            weapons[alias] = target
    for p in sorted(tools_dir.glob("verify_*.py")):
        if p.name in ("verify_all.py", "ntetool.py"):
            continue
        if "weapon" in p.name:
            short = p.stem.removeprefix("verify_")
            weapons[short] = p
    return weapons


def run_single(verifier: Path, verbose: bool = False) -> tuple[Path, bool, float, str]:
    start = time.perf_counter()
    result = subprocess.run(
        [sys.executable, str(verifier)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    duration = time.perf_counter() - start
    passed = result.returncode == 0
    output = result.stdout + ("\n" + result.stderr if result.stderr else "")
    return verifier, passed, duration, output


def run_suite(
    verifiers: list[Path],
    *,
    workers: int = 4,
    fail_fast: bool = False,
    verbose: bool = False,
) -> int:
    workers = max(1, min(4, workers))
    print(f"Running {len(verifiers)} verifier(s) with {workers} worker(s)...")

    start_all = time.perf_counter()
    passed_count = 0
    failed_count = 0
    failures: list[tuple[Path, str]] = []

    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(run_single, v, verbose): v for v in verifiers}
        for future in as_completed(futures):
            verifier, passed, duration, output = future.result()
            status = "PASS" if passed else "FAIL"
            print(f"  [{status}] {verifier.name} ({duration:.2f}s)")
            if passed:
                passed_count += 1
            else:
                failed_count += 1
                failures.append((verifier, output))
                if fail_fast:
                    print("\n--fail-fast triggered, aborting remaining tests.")
                    executor.shutdown(wait=False, cancel_futures=True)
                    break
            if verbose and output.strip():
                print(output.strip())

    total_time = time.perf_counter() - start_all
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


# --- Markdown link & anchor checking ---

def without_fences(text: str) -> str:
    return re.sub(r"^(`{3,}|~{3,}).*?^\1\s*$", "", text, flags=re.MULTILINE | re.DOTALL)


def markdown_anchors(text: str) -> set[str]:
    anchors, counts = set(), {}
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", without_fences(text), re.MULTILINE):
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        count = counts.get(slug, 0)
        anchors.add(slug + (f"-{count}" if count else ""))
        counts[slug] = count + 1
    return anchors


def repository_markdown_files(root: Path) -> list[Path]:
    excluded = {"cache", ".cache", ".git", "node_modules", "target"}
    results = []
    for directory, children, files in os.walk(root):
        children[:] = [child for child in children if child not in excluded]
        for filename in files:
            if filename.lower().endswith(".md"):
                results.append((Path(directory) / filename))
    return sorted(results)


def check_markdown_links(root: Path, paths: list[str | Path] | None = None) -> list[str]:
    if paths:
        md_paths = [Path(p) if Path(p).is_absolute() else root / p for p in paths]
        md_paths = [p for p in md_paths if p.suffix.lower() == ".md"]
    else:
        md_paths = repository_markdown_files(root)

    errors = []
    anchor_cache: dict[Path, set[str]] = {}

    for path in sorted(md_paths):
        if not path.exists():
            continue
        rel = path.relative_to(root).as_posix()
        try:
            text = without_fences(path.read_text(encoding="utf-8-sig"))
        except OSError as e:
            errors.append(f"{rel}: failed to read: {e}")
            continue

        for link in re.findall(r"!?\[[^\]]*\]\(([^\n]+?)\)", text):
            destination = link[1:link.find(">")] if link.startswith("<") else link.split()[0]
            parsed = urlsplit(destination)
            if parsed.scheme or parsed.netloc:
                continue
            target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not target.exists():
                errors.append(f"{rel}: missing local link target {destination}")
                continue
            if parsed.fragment and target.suffix.lower() == ".md":
                if target not in anchor_cache:
                    try:
                        anchor_cache[target] = markdown_anchors(target.read_text(encoding="utf-8"))
                    except OSError as e:
                        errors.append(f"{rel}: cannot read target {destination}: {e}")
                        continue
                if unquote(parsed.fragment) not in anchor_cache[target]:
                    errors.append(f"{rel}: missing local anchor {destination}")

    return errors


# --- Fast JSON/TOML syntax checking ---

def check_file_syntax(root: Path, paths: list[str | Path]) -> list[str]:
    errors = []
    for p in paths:
        path = Path(p) if Path(p).is_absolute() else root / p
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix() if path.is_relative_to(root) else str(path)
        try:
            text = path.read_text(encoding="utf-8-sig")
            if path.suffix == ".json":
                json.loads(text)
            elif path.suffix == ".toml":
                tomllib.loads(text)
        except Exception as e:
            errors.append(f"{rel}: syntax error: {e}")
    return errors


# --- CLI dispatch ---

def cmd_verify_kit(args: argparse.Namespace) -> int:
    kits = discover_kits()
    if getattr(args, "list", False):
        print(f"Available character kits ({len(kits)}):")
        for name in sorted(kits):
            print(f"  {name}")
        return 0

    target = getattr(args, "character", None)
    run_all = getattr(args, "all", False) or target in ("all", None)

    if run_all:
        verifiers = list(kits.values())
        return run_suite(verifiers, workers=getattr(args, "jobs", 4), verbose=getattr(args, "verbose", False))

    target = target.lower().strip()
    if target not in kits:
        print(f"Error: Unknown character kit '{target}'. Available: {', '.join(sorted(kits))}", file=sys.stderr)
        return 1

    path = kits[target]
    v, passed, duration, output = run_single(path, verbose=True)
    status = "PASS" if passed else "FAIL"
    print(f"[{status}] {v.name} ({duration:.2f}s)")
    if output.strip():
        print(output.strip())
    return 0 if passed else 1


def cmd_verify_weapon(args: argparse.Namespace) -> int:
    weapons = discover_weapons()
    if getattr(args, "list", False):
        print(f"Available weapons / mechanics ({len(weapons)}):")
        for name in sorted(weapons):
            print(f"  {name:<25} -> {weapons[name].name}")
        return 0

    target = getattr(args, "weapon", None)
    run_all = getattr(args, "all", False) or target in ("all", None)

    if run_all:
        unique_paths = sorted(set(weapons.values()))
        return run_suite(unique_paths, workers=getattr(args, "jobs", 4), verbose=getattr(args, "verbose", False))

    target = target.lower().strip()
    matched = None
    if target in weapons:
        matched = weapons[target]
    else:
        # Search substring
        candidates = [p for alias, p in weapons.items() if target in alias]
        if candidates:
            matched = candidates[0]

    if not matched:
        print(f"Error: Unknown weapon verifier '{target}'. Available aliases: {', '.join(sorted(weapons))}", file=sys.stderr)
        return 1

    v, passed, duration, output = run_single(matched, verbose=True)
    status = "PASS" if passed else "FAIL"
    print(f"[{status}] {v.name} ({duration:.2f}s)")
    if output.strip():
        print(output.strip())
    return 0 if passed else 1


def cmd_check_links(args: argparse.Namespace) -> int:
    paths = getattr(args, "paths", None)
    errors = check_markdown_links(ROOT, paths)
    if errors:
        print(f"Markdown link validation failed ({len(errors)} error(s)):", file=sys.stderr)
        for err in errors:
            print(f"  {err}", file=sys.stderr)
        return 1
    target_count = len(paths) if paths else len(repository_markdown_files(ROOT))
    print(f"Markdown links OK: {target_count} document(s) verified.")
    return 0


def cmd_check_syntax(args: argparse.Namespace) -> int:
    paths = getattr(args, "paths", [])
    if not paths:
        print("No paths specified for syntax check.", file=sys.stderr)
        return 0
    errors = check_file_syntax(ROOT, paths)
    if errors:
        print(f"Syntax validation failed ({len(errors)} error(s)):", file=sys.stderr)
        for err in errors:
            print(f"  {err}", file=sys.stderr)
        return 1
    print(f"Syntax OK: {len(paths)} structured file(s) verified.")
    return 0


def main(argv: list[str] | None = None) -> int:
    limit_cpus()
    parser = argparse.ArgumentParser(
        prog="ntetool",
        description="ntetool - Unified CLI for verification, linting, syntax, and links in ntesim.",
    )
    sub = parser.add_subparsers(dest="command")

    # verify
    verify_p = sub.add_parser("verify", help="Run verification oracles")
    verify_sub = verify_p.add_subparsers(dest="verify_subcommand")

    # verify kit
    kit_p = verify_sub.add_parser("kit", help="Verify character kits")
    kit_p.add_argument("character", nargs="?", default=None, help="Character name (e.g. mint, aurelia) or 'all'")
    kit_p.add_argument("--all", action="store_true", help="Run all character kits")
    kit_p.add_argument("--list", "-l", action="store_true", help="List available character kits")
    kit_p.add_argument("--jobs", "-j", type=int, default=4, help="Max workers (1-4, default: 4)")
    kit_p.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    # verify weapon
    wep_p = verify_sub.add_parser("weapon", help="Verify weapon mechanics")
    wep_p.add_argument("weapon", nargs="?", default=None, help="Weapon name or alias (e.g. far_side_moon, koinobori) or 'all'")
    wep_p.add_argument("--all", action="store_true", help="Run all weapon verifiers")
    wep_p.add_argument("--list", "-l", action="store_true", help="List available weapon verifiers")
    wep_p.add_argument("--jobs", "-j", type=int, default=4, help="Max workers (1-4, default: 4)")
    wep_p.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    # verify all / suite options directly on verify_p
    verify_p.add_argument("target", nargs="?", default=None, help="Category or verifier target")
    verify_p.add_argument("--category", choices=["kits", "sources", "scenarios", "mechanics", "runtime", "all"],
                          default=None, help="Run only verifiers in this category")
    verify_p.add_argument("--filter", "-f", type=str, default=None, help="Filter verifiers by regex")
    verify_p.add_argument("--all", action="store_true", help="Run all discovered verifiers")
    verify_p.add_argument("--list", "-l", action="store_true", help="List verifiers without executing")
    verify_p.add_argument("--fail-fast", action="store_true", help="Stop immediately on first failure")
    verify_p.add_argument("--jobs", "-j", type=int, default=4, help="Max workers (1-4, default: 4)")
    verify_p.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    # Top-level direct aliases
    kit_direct = sub.add_parser("verify-kit", help="Verify character kits")
    kit_direct.add_argument("character", nargs="?", default=None, help="Character name or 'all'")
    kit_direct.add_argument("--all", action="store_true", help="Run all character kits")
    kit_direct.add_argument("--list", "-l", action="store_true", help="List available character kits")
    kit_direct.add_argument("--jobs", "-j", type=int, default=4, help="Max workers (1-4, default: 4)")
    kit_direct.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    wep_direct = sub.add_parser("verify-weapon", help="Verify weapon mechanics")
    wep_direct.add_argument("weapon", nargs="?", default=None, help="Weapon name or 'all'")
    wep_direct.add_argument("--all", action="store_true", help="Run all weapon verifiers")
    wep_direct.add_argument("--list", "-l", action="store_true", help="List available weapon verifiers")
    wep_direct.add_argument("--jobs", "-j", type=int, default=4, help="Max workers (1-4, default: 4)")
    wep_direct.add_argument("--verbose", "-v", action="store_true", help="Verbose output")

    # check-links
    links_p = sub.add_parser("check-links", help="Validate local markdown links and anchors")
    links_p.add_argument("paths", nargs="*", default=None, help="Optional markdown files or directories")

    # check-syntax
    syntax_p = sub.add_parser("check-syntax", help="Fast syntax check for JSON and TOML files")
    syntax_p.add_argument("paths", nargs="+", help="JSON or TOML files to check")

    args = parser.parse_args(argv)

    if args.command == "verify-kit":
        return cmd_verify_kit(args)
    elif args.command == "verify-weapon":
        return cmd_verify_weapon(args)
    elif args.command == "check-links":
        return cmd_check_links(args)
    elif args.command == "check-syntax":
        return cmd_check_syntax(args)
    elif args.command == "verify":
        if args.verify_subcommand == "kit":
            return cmd_verify_kit(args)
        elif args.verify_subcommand == "weapon":
            return cmd_verify_weapon(args)
        # General verify runner
        verifiers = discover_verifiers()
        if args.target:
            target = args.target.lower().strip()
            if target in ("kits", "sources", "scenarios", "mechanics", "runtime"):
                verifiers = [v for v in verifiers if categorize(v) == target]
            elif target == "all":
                pass
            else:
                exact = [v for v in verifiers if v.name == f"verify_{target}_kit.py"]
                if exact:
                    verifiers = exact
                else:
                    verifiers = [v for v in verifiers if target in v.name.lower()]
        elif args.category and args.category != "all":
            verifiers = [v for v in verifiers if categorize(v) == args.category]

        if args.filter:
            pattern = re.compile(args.filter, re.IGNORECASE)
            verifiers = [v for v in verifiers if pattern.search(v.name)]

        if args.list:
            print(f"Discovered {len(verifiers)} matching verifier(s):")
            for v in verifiers:
                print(f"  [{categorize(v):<9}] {v.name}")
            return 0

        if not verifiers:
            print("No verifiers matched the given target or filters.")
            return 1

        if len(verifiers) == 1 and not args.all:
            v, passed, duration, output = run_single(verifiers[0], verbose=True)
            status = "PASS" if passed else "FAIL"
            print(f"[{status}] {v.name} ({duration:.2f}s)")
            if output.strip():
                print(output.strip())
            return 0 if passed else 1

        return run_suite(verifiers, workers=args.jobs, fail_fast=args.fail_fast, verbose=args.verbose)
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
