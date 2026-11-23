"""Opt-in exact source/Studio throughput measurements (never a checkpoint check)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

from cargo import PIN, engine_source_sha256
from cpu_limit import limit_cpus
from native_benchmark import wait_usage

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def topology(cpus):
    """Describe only available topology; do not broaden inherited affinity."""
    try:
        if os.name == "nt":
            import ctypes
            from ctypes import wintypes
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            query = kernel.GetLogicalProcessorInformationEx
            query.argtypes = [wintypes.DWORD, ctypes.c_void_p, ctypes.POINTER(wintypes.DWORD)]
            size = wintypes.DWORD()
            query(0, None, ctypes.byref(size))  # RelationProcessorCore
            buffer = ctypes.create_string_buffer(size.value)
            if not size.value or not query(0, buffer, ctypes.byref(size)):
                return {"available": False, "reason": "processor topology query unavailable"}
            raw = buffer.raw
            offset, cores = 0, []
            pointer_size = ctypes.sizeof(ctypes.c_size_t)
            while offset < size.value:
                length = int.from_bytes(raw[offset + 4:offset + 8], "little")
                groups = int.from_bytes(raw[offset + 30:offset + 32], "little")
                logical = []
                for index in range(groups):
                    start = offset + 32 + index * (pointer_size + 8)
                    mask = int.from_bytes(raw[start:start + pointer_size], "little")
                    group = int.from_bytes(raw[start + pointer_size:start + pointer_size + 2], "little")
                    if group == 0:
                        logical.extend(cpu for cpu in cpus if mask & (1 << cpu))
                if logical:
                    cores.append(logical)
                if length < 8:
                    return {"available": False, "reason": "invalid processor topology record"}
                offset += length
            return {"available": True, "selected_logical_cpus_by_core": cores,
                    "physical_cores_in_affinity": len(cores)}
        if sys.platform.startswith("linux"):
            cores = {}
            for cpu in cpus:
                directory = Path(f"/sys/devices/system/cpu/cpu{cpu}/topology")
                key = ((directory / "physical_package_id").read_text().strip(),
                       (directory / "core_id").read_text().strip())
                cores.setdefault(key, []).append(cpu)
            return {"available": True, "selected_logical_cpus_by_core": list(cores.values()),
                    "physical_cores_in_affinity": len(cores)}
    except (OSError, AttributeError):
        pass
    return {"available": False, "reason": "topology unavailable on this host"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["native", "wasm", "workers", "chunks"])
    parser.add_argument("--source", type=Path, default=Path("presets/studio/zankou-insight.nte"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--count", type=int, default=1000)
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--workers", type=int, choices=[1, 2, 4], default=1)
    parser.add_argument("--chunk-size", type=int, choices=[25, 50, 100])
    parser.add_argument("--shared-aggregator", action="store_true")
    parser.add_argument("--profile", choices=["release", "profiling"], default="release")
    parser.add_argument("--diagnostics", action="store_true")
    parser.add_argument("--oracle", action="store_true", help="native detailed per-seed mechanics hashes and independent rounded summaries; not headline timing")
    parser.add_argument("--build", action="store_true")
    parser.add_argument("--reference", type=Path,
                        help="archived cache artifact manifest for matched historical measurements")
    parser.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()
    cpus = limit_cpus()
    source = (ROOT / args.source).resolve()
    output = (ROOT / args.output).resolve()
    if not output.is_relative_to((ROOT / "cache").resolve()):
        parser.error("output must be inside this checkout's cache/")
    if args.count <= 0 or args.repetitions <= 0 or args.timeout <= 0:
        parser.error("count, repetitions and timeout must be positive")
    if args.diagnostics and args.mode != "native":
        parser.error("native owns phase/allocation diagnostics; use profiling WASM for CPU profiles")
    if args.oracle and args.mode != "native":
        parser.error("native owns the detailed per-seed oracle")
    if args.mode != "chunks" and (args.chunk_size is not None or args.shared_aggregator):
        parser.error("chunk size and aggregator placement apply to matched chunks")
    reference = None
    reference_root = None
    if args.reference:
        manifest = (ROOT / args.reference).resolve()
        if (not manifest.is_relative_to((ROOT / "cache").resolve())
                or args.build or args.diagnostics or args.profile != "release"):
            parser.error("references must be archived release manifests in cache without build/diagnostics")
        reference = json.loads(manifest.read_text(encoding="utf-8"))
        reference_root = manifest.parent
        for name, digest in reference["files"].items():
            archived = (reference_root / name).resolve()
            if not archived.is_relative_to(reference_root) or sha(archived) != digest:
                parser.error("archived reference file identity mismatch")
    output.mkdir(parents=True, exist_ok=True)
    receipt = {"schema_version": 1, "state": "running", "mode": args.mode,
               "engine_source_sha256": engine_source_sha256(), "source_sha256": sha(source),
               "source": source.relative_to(ROOT).as_posix() if source.is_relative_to(ROOT) else "external",
               "rust": PIN, "node": subprocess.check_output(["node", "--version"], text=True).strip(),
               "host": platform.platform(), "affinity": cpus, "workers": args.workers,
               "topology": topology(cpus),
               "profile": args.profile, "diagnostics": args.diagnostics, "oracle": args.oracle,
               "count": args.count, "numeric_policy": "exact",
               "damage_total_policy": "rounded-hit-sum", "runs": []}
    if reference:
        receipt.update(engine_source_sha256=reference["engine_source_sha256"],
                       damage_total_policy=reference["damage_total_policy"],
                       reference_sha256=sha(manifest))
    if args.mode == "chunks":
        receipt.update(chunk_size=args.chunk_size or 50, shared_aggregator=args.shared_aggregator)
    receipt["tools"] = {name: sha(ROOT / name) for name in (
        "tools/throughput.py", "tools/throughput.mjs", "tools/throughput_worker.mjs",
        "crates/research/src/bin/throughput.rs", "crates/wasm/js/runtime.js",
        "crates/wasm/js/wire.js", "web/tests/node_worker_bridge.mjs")}
    receipt["tools"].update({path.relative_to(ROOT).as_posix(): sha(path)
                             for path in (ROOT / "web/src/infrastructure/worker").glob("*.ts")})
    summary = output / "receipt.json"
    summary.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    try:
        if args.build:
            command = [sys.executable, str(ROOT / "tools/cargo.py"), "build", "--locked", "--offline"]
            if args.mode == "native":
                features = "throughput-bench" + (",performance-diagnostics" if args.diagnostics else "")
                command += ["-p", "ntesim-research", "--bin", "ntesim-throughput", "--features", features]
            else:
                command += ["-p", "ntesim-wasm", "--target", "wasm32-unknown-unknown"]
            command += ["--profile", args.profile]
            started = time.perf_counter()
            subprocess.run(command, cwd=ROOT, check=True)
            receipt["build_seconds"] = time.perf_counter() - started
        binary = ROOT / "target" / args.profile / ("ntesim-throughput.exe" if os.name == "nt" else "ntesim-throughput")
        glue = ROOT / "target" / ("wasm-bindgen-profiling" if args.profile == "profiling" else "wasm-bindgen")
        if reference_root:
            binary = reference_root / binary.name
            glue = reference_root / "wasm-bindgen"
        artifact = binary if args.mode == "native" else glue / "ntesim_wasm_bg.wasm"
        if reference:
            required = [artifact]
            if args.mode != "native":
                required.append(glue / "bindings.js")
            if any(path.relative_to(reference_root).as_posix() not in reference["files"] for path in required):
                raise RuntimeError("selected artifact or its bindings are absent from the reference manifest")
        receipt["artifact_sha256"] = sha(artifact)
        if args.mode != "native":
            receipt["glue_files"] = {p.name: sha(p) for p in glue.iterdir() if p.is_file()}
        for repetition in range(args.repetitions):
            raw = output / f"run-{repetition}.json"
            log = output / f"run-{repetition}.log"
            if args.mode == "native":
                command = [str(binary), str(source), str(args.count), "1"]
                if args.oracle:
                    command.append("--oracle")
            else:
                command = ["node", "--import", (ROOT / "cache/web/node_modules/tsx/dist/loader.mjs").as_uri(),
                           str(ROOT / "tools/throughput.mjs"), args.mode, str(source),
                           str(args.count), str(args.workers), str(glue), str(output / f"profile-{repetition}")]
                if args.mode == "chunks":
                    command += [str(args.chunk_size or 50), "shared" if args.shared_aggregator else "dedicated"]
            started = time.perf_counter()
            with raw.open("wb") as stdout, log.open("wb") as stderr:
                env = {**os.environ, "TSX_TSCONFIG_PATH": str(ROOT / "web/tsconfig.runtime.json")}
                process = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr, env=env)
                usage = wait_usage(process, args.timeout)
            process_wall = time.perf_counter() - started
            if process.returncode:
                raise RuntimeError(f"measurement failed; see {log.relative_to(ROOT)}")
            value = json.loads(raw.read_text(encoding="utf-8"))
            if value["engine_source_sha256"] != receipt["engine_source_sha256"]:
                raise RuntimeError("measurement used a stale engine; rebuild the selected artifact")
            if args.mode == "native":
                measured = value["repetitions"][0]
                if (measured["diagnostics"] is not None) != args.diagnostics:
                    raise RuntimeError("native artifact diagnostics differ from the requested mode; rebuild")
                if args.oracle and len(measured["oracles"]) != args.count:
                    raise RuntimeError("detailed oracle did not cover every requested seed")
            else:
                measured = value
            samples, aggregate = measured["samples"], measured["aggregate"]
            if (len(samples) != args.count or aggregate["completed"] != args.count
                    or not aggregate["game_dps_computed"]):
                raise RuntimeError("measurement is incomplete or ineligible for damage comparison")
            receipt["runs"].append({"wall_seconds": process_wall,
                                    **usage, "raw": raw.name, "raw_sha256": sha(raw),
                                    "completed":len(samples), "failed":0,
                                    "events":sum(sample["events"] for sample in samples),
                                    "hits":sum(sample["hits"] for sample in samples)})
            summary.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        walls = [run["wall_seconds"] for run in receipt["runs"]]
        receipt.update(state="complete", wall_median=statistics.median(walls),
                       wall_min=min(walls), wall_max=max(walls))
    except Exception as error:
        receipt.update(state="failed", error=str(error))
        raise
    finally:
        summary.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(f"Exact throughput receipt: {summary.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
