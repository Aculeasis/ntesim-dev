#!/usr/bin/env python3

import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path
from verification_workspace import review_receipt_path

ROOT = Path(__file__).resolve().parents[1]
CLI_PATH = ROOT / "target" / "release" / ("ntesim.exe" if os.name == "nt" else "ntesim")
if not CLI_PATH.exists():
    CLI_PATH = ROOT / "target" / "debug" / ("ntesim.exe" if os.name == "nt" else "ntesim")
WASM_PATH = ROOT / "target" / "wasm-bindgen" / "ntesim_wasm_bg.wasm"
SCENARIO_PATH = ROOT / "examples" / "s5-game-scenario.json"
REVIEW_DIR = review_receipt_path(ROOT / "reviews/stage-h7-wasm-benchmark")
VERIFICATION_JSON = REVIEW_DIR / "verification.json"


def get_runtime_identities():
    rustc_ver = subprocess.check_output(["rustc", "--version"], text=True).strip()
    node_ver = subprocess.check_output(["node", "--version"], text=True).strip()
    return {
        "os": platform.system(),
        "os_release": platform.release(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "rustc": rustc_ver,
        "node": node_ver,
    }


def benchmark_wasm(iterations=200):
    script = f"""
const fs = require('fs');
const {{ performance }} = require('perf_hooks');
const {{ NtesimWasm }} = require('./crates/wasm/js/ntesim.js');

async function bench() {{
    const wasmBytes = fs.readFileSync('{WASM_PATH.as_posix()}');
    const loadStart = performance.now();
    const wasm = await NtesimWasm.create(wasmBytes);
    const loadTimeMs = performance.now() - loadStart;

    const raw = fs.readFileSync('{SCENARIO_PATH.as_posix()}');
    const jsonReq = JSON.parse(raw);

    const memBefore = process.memoryUsage().heapUsed;
    const start = performance.now();
    let totalEvents = 0;
    let sampleResult = null;

    for (let i = 0; i < {iterations}; i++) {{
        const res = wasm.simulateGame(jsonReq);
        totalEvents += res.total_events_processed;
        if (i === 0) sampleResult = res;
    }}
    const elapsedSec = (performance.now() - start) / 1000.0;
    const memAfter = process.memoryUsage().heapUsed;

    const runsPerSec = {iterations} / elapsedSec;
    const eventsPerSec = totalEvents / elapsedSec;

    console.log(JSON.stringify({{
        runtime: 'wasm32 (Node.js)',
        iterations: {iterations},
        elapsed_sec: elapsedSec,
        runs_per_sec: runsPerSec,
        total_events: totalEvents,
        events_per_sec: eventsPerSec,
        startup_ms: loadTimeMs,
        heap_diff_bytes: memAfter - memBefore,
        sample_damage_exact: sampleResult.total_damage_exact,
        sample_events: sampleResult.total_events_processed,
    }}));
}}

bench().catch(e => {{ console.error(e); process.exit(1); }});
"""
    res = subprocess.run(["node", "-e", script], cwd=str(ROOT), capture_output=True, text=True, check=True)
    return json.loads(res.stdout)


def benchmark_native(iterations=50):
    raw = SCENARIO_PATH.read_bytes()
    start = time.perf_counter()
    sample_res = None
    total_events = 0

    for i in range(iterations):
        cmd = [str(CLI_PATH), "run-game", str(SCENARIO_PATH)]
        out = subprocess.check_output(cmd, text=True)
        if i == 0:
            sample_res = json.loads(out)
        total_events += sample_res.get("total_events_processed", 28)

    elapsed_sec = time.perf_counter() - start
    runs_per_sec = iterations / elapsed_sec
    events_per_sec = total_events / elapsed_sec

    return {
        "runtime": "native (Rust binary)",
        "iterations": iterations,
        "elapsed_sec": elapsed_sec,
        "runs_per_sec": runs_per_sec,
        "total_events": total_events,
        "events_per_sec": events_per_sec,
        "sample_damage_exact": sample_res.get("total_damage_exact", {}),
        "sample_events": sample_res.get("total_events_processed", 0),
    }


def main():
    print("Running Stage H7 Benchmark: Native CLI vs WebAssembly...")
    identities = get_runtime_identities()
    print(f"Host: {identities['os']} {identities['machine']}, Rust: {identities['rustc']}, Node: {identities['node']}")

    wasm_results = benchmark_wasm(iterations=200)
    print(f"WASM / Node throughput: {wasm_results['runs_per_sec']:.1f} runs/sec ({wasm_results['events_per_sec']:.1f} events/sec)")
    print(f"WASM startup / compilation: {wasm_results['startup_ms']:.1f} ms")

    # Native CLI benchmark (process-per-invocation overhead included)
    native_results = benchmark_native(iterations=30)
    print(f"Native CLI invocation throughput: {native_results['runs_per_sec']:.1f} runs/sec")

    wasm_num, wasm_den = map(int, wasm_results["sample_damage_exact"].split("/"))
    cli_num, cli_den = map(int, native_results["sample_damage_exact"].split("/"))
    parity_preserved = (wasm_num == cli_num) and (wasm_den == cli_den) and (wasm_results["sample_events"] == native_results["sample_events"])

    report = {
        "schema_version": 1,
        "stage": "H7",
        "title": "Native Execution vs WebAssembly Performance Benchmark",
        "content_id": "ntesim-current",
        "content_cutoff": "2026-10-04",
        "status": "INCOMPLETE",
        "game_dps_computed": False,
        "parity_preserved": parity_preserved,
        "hardware_identities": identities,
        "workload": {
            "scenario": "S5 Full published (Mint+Sakiri+Zero+Iroi, 120s)",
            "events_per_run": wasm_results["sample_events"],
        },
        "wasm_performance": wasm_results,
        "native_performance": native_results,
    }

    REVIEW_DIR.mkdir(parents=True, exist_ok=True)
    VERIFICATION_JSON.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"Wrote benchmark report to {VERIFICATION_JSON}")
    print(f"Parity preserved across benchmark: {parity_preserved}")


if __name__ == "__main__":
    if "--native-only" in sys.argv:
        from native_benchmark import main as native_main
        sys.argv.remove("--native-only")
        raise SystemExit(native_main())
    main()
