"""Native source lifecycle/performance measurements; no combat reconstruction.

Called by benchmark_native_wasm.py --native-only. Native stdout, profiles and
portable checkpoints stay in the requested cache directory for reproduction.
"""
import argparse
import hashlib
import json
import math
import os
import platform
import subprocess
import sys
import time
from pathlib import Path

from cargo import PIN, engine_source_sha256
from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]
BUDGET = {
    "wall_seconds": 30,
    "cpu_seconds": 30,
    "peak_resident_bytes": 512 * 1024 * 1024,
    "serialization_ms": 5000,
    "serialized_report_bytes": 64 * 1024 * 1024,
    "events_processed": 100_000,
}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def windows_usage(process):
    import ctypes
    from ctypes import wintypes

    class Memory(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in (
                "peak_working_set", "working_set", "peak_paged", "paged",
                "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile", "private")]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    counters = [wintypes.FILETIME() for _ in range(4)]
    if not kernel.GetProcessTimes(process._handle, *(ctypes.byref(value) for value in counters)):
        raise ctypes.WinError(ctypes.get_last_error())
    seconds = sum((value.dwHighDateTime << 32) | value.dwLowDateTime for value in counters[2:]) / 10_000_000
    memory = Memory()
    memory.cb = ctypes.sizeof(memory)
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Memory), wintypes.DWORD]
    if not psapi.GetProcessMemoryInfo(process._handle, ctypes.byref(memory), memory.cb):
        raise ctypes.WinError(ctypes.get_last_error())
    return {"cpu_seconds": seconds, "peak_resident_bytes": memory.peak_working_set}


def wait_usage(process, timeout):
    if os.name == "nt":
        try:
            process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise RuntimeError("native measurement exceeded wall-clock timeout") from None
        return windows_usage(process)
    deadline = time.monotonic() + timeout
    while True:
        pid, status, usage = os.wait4(process.pid, os.WNOHANG)
        if pid:
            process.returncode = os.waitstatus_to_exitcode(status)
            return {"cpu_seconds": usage.ru_utime + usage.ru_stime,
                    "peak_resident_bytes": usage.ru_maxrss * (1 if sys.platform == "darwin" else 1024)}
        if time.monotonic() >= deadline:
            process.kill()
            _, status, _ = os.wait4(process.pid, 0)
            process.returncode = os.waitstatus_to_exitcode(status)
            raise RuntimeError("native measurement exceeded wall-clock timeout")
        time.sleep(0.01)


def collect_process(command, output, stderr, *, cwd=ROOT):
    start = time.perf_counter()
    with output.open("wb") as stdout, stderr.open("wb") as errors:
        process = subprocess.Popen(command, cwd=cwd, stdout=stdout, stderr=errors)
        usage = wait_usage(process, BUDGET["wall_seconds"] * 2)
    return {**usage, "wall_seconds": time.perf_counter() - start,
            "exit_code": process.returncode}


def validate_sample(profile, usage, stdout_size, identity, expected_exit):
    if profile.get("schema_version") != 1 or profile.get("engine_source_sha256") != identity:
        raise ValueError("native profile engine/schema identity mismatch; rebuild the CLI")
    if usage["exit_code"] != expected_exit or profile.get("operation_success") != (expected_exit == 0):
        raise ValueError("native measurement failed or returned an unexpected incomplete outcome")
    if profile.get("serialized_report_bytes") != stdout_size:
        raise ValueError("native stdout and measured serialized bytes differ")
    metrics = {**usage, **profile}
    for key in ("command_ms", "serialization_ms", "wall_seconds", "cpu_seconds",
                "peak_resident_bytes", "serialized_report_bytes", "events_processed"):
        value = metrics.get(key)
        if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value) or value < 0:
            raise ValueError(f"missing/invalid native metric: {key}")
    exceeded = [key for key, limit in BUDGET.items() if metrics[key] > limit]
    return exceeded


def sample(cli, directory, name, args, identity, expected_exit=0):
    stdout = directory / f"{name}.stdout.json"
    stderr = directory / f"{name}.stderr.txt"
    profile_path = directory / f"{name}.profile.json"
    command = [str(cli), *args, "--profile", str(profile_path)]
    usage = collect_process(command, stdout, stderr)
    if usage["exit_code"] != expected_exit:
        raise ValueError(f"{name}: unexpected exit {usage['exit_code']}; see {stderr}")
    profile = json.loads(profile_path.read_bytes())
    exceeded = validate_sample(profile, usage, stdout.stat().st_size, identity, expected_exit)
    return {"name": name, "command": command, **usage, **profile,
            "budget_exceeded": exceeded, "stdout_sha256": sha(stdout),
            "profile_sha256": sha(profile_path)}, stdout


def run(cli, output, iterations):
    output.mkdir(parents=True, exist_ok=False)
    identity = engine_source_sha256()
    artifact_sha256 = sha(cli)
    workloads = {
        "daffodil-team": ["run-script", "examples/baselines/daffodil.nte"],
        "nanally-team": ["run-script", "presets/native/nanally-full-kit.nte"],
        "incoming-shields": ["run-script", "examples/incoming-shield-cycles.nte"],
        "bounded-search": ["search-scripts", "examples/native-source-search.json"],
        "mint-full": ["run-script", "examples/mint-native-cycles.nte"],
    }
    input_paths = [ROOT / args[1] for args in workloads.values()]
    search = json.loads((ROOT / "examples/native-source-search.json").read_bytes())
    input_paths.extend(ROOT / "examples" / candidate["source"] for candidate in search["candidates"])
    inputs = {path.relative_to(ROOT).as_posix(): sha(path) for path in input_paths}
    rows = []
    full = None
    for iteration in range(iterations):
        for name, args in workloads.items():
            row, stdout = sample(cli, output, f"{name}-{iteration}", args, identity)
            rows.append(row)
            if name == "mint-full":
                full = stdout
    prefix, prefix_path = sample(cli, output, "mint-prefix", [
        "run-script", "examples/mint-native-cycles.nte", "--stop-after-inputs", "13"], identity, 2)
    saved = json.loads(prefix_path.read_bytes())
    if saved["execution"]["report"]["execution_completed"] or not saved["checkpoint"]:
        raise ValueError("cancelled source did not retain an incomplete prefix/checkpoint")
    checkpoint = output / "relocated-checkpoint.json"
    checkpoint.write_text(json.dumps(saved["checkpoint"]), encoding="utf-8")
    resumed, resumed_path = sample(cli, output, "mint-resumed", [
        "run-script", "examples/mint-native-cycles.nte", "--resume", str(checkpoint)], identity)
    if json.loads(resumed_path.read_bytes())["execution"] != json.loads(full.read_bytes())["execution"]:
        raise ValueError("native resumed output differs from uninterrupted output")
    rows.extend([prefix, resumed])
    if (sha(cli) != artifact_sha256 or engine_source_sha256() != identity
            or any(sha(ROOT / name) != digest for name, digest in inputs.items())):
        raise ValueError("native measurement inputs/artifact changed during collection")
    report = {
        "schema_version": 1,
        "passed": all(not row["budget_exceeded"] for row in rows),
        "environment": {"os": platform.platform(), "machine": platform.machine(),
                        "processor": platform.processor(), "python": platform.python_version(),
                        "rust_pin": PIN, "logical_cpus_available": os.cpu_count(), "cpu_limit": 4},
        "artifact": {"path": str(cli), "sha256": artifact_sha256, "engine_source_sha256": identity},
        "tool_sha256": {path.name: sha(path) for path in [Path(__file__), ROOT / "tools/benchmark_native_wasm.py"]},
        "input_sha256": inputs,
        "iterations": iterations,
        "budgets_per_process": BUDGET,
        "resumed_equal": True,
        "measurements": rows,
        "limits": "Process CPU/peak residency include startup, stdout IO and opt-in profiling. command_ms includes source compilation/execution and conversion to a JSON value. serialization_ms measures one additional pretty-JSON pass to a counting sink. Serialized bytes are retention/transport size, not live heap. Replay repeats saved prefix CPU without duplicating combat receipts. This is native acceptance, not browser or game-fidelity evidence.",
    }
    (output / "measurement.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=ROOT / "target/debug" / ("ntesim.exe" if os.name == "nt" else "ntesim"))
    parser.add_argument("--output", type=Path, required=True, help="New cache directory for measured artifacts")
    parser.add_argument("--iterations", type=int, default=3)
    args = parser.parse_args()
    if not 1 <= args.iterations <= 20:
        parser.error("iterations must be in 1..=20")
    limit_cpus()
    try:
        report = run(args.cli.resolve(), args.output.resolve(), args.iterations)
    except (OSError, ValueError, RuntimeError) as error:
        print(f"native benchmark failed: {error}", file=sys.stderr)
        return 1
    print(f"Native measurement {'passed' if report['passed'] else 'exceeded budget'}: {args.output / 'measurement.json'}")
    return 0 if report["passed"] else 1
