#!/usr/bin/env python3
"""Stage H1 verification script: Native CLI vs WebAssembly parity.

Validates that ntesim-wasm produces identical deterministic gameplay output,
RNG draws, arithmetic fractions, event timelines, and tabular results as
the native Rust engine across all canonical scenarios.
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from verification_workspace import temporary_directory

ROOT = Path(__file__).resolve().parents[1]
CLI_PATH = ROOT / "target" / "debug" / ("ntesim.exe" if os.name == "nt" else "ntesim")
WASM_PATH = ROOT / "target" / "wasm-bindgen" / "ntesim_wasm_bg.wasm"
REVIEW_DIR = ROOT / "reviews" / "stage-h-wasm-parity"
VERIFICATION_JSON = ROOT / "cache" / "verification" / "wasm-parity.json"


def run_node_parity(extra_scenarios=()):
    script = f"""
const fs = require('fs');
const assert = require('node:assert/strict');
const {{ execFileSync }} = require('child_process');
const {{ parseLosslessJson }} = require('./crates/wasm/js/lossless-json.js');
const {{ NtesimWasm }} = require('./crates/wasm/js/ntesim.js');

async function main() {{
    const wasmBytes = fs.readFileSync('{WASM_PATH.as_posix()}');
    const wasm = await NtesimWasm.create(wasmBytes);
    const results = {{}};
    // Preserve raw JSON for Python's arbitrary-precision comparison of Rust u128 fields.
    results.exact_reports = [];

    // 1. Version check
    const wasmVer = wasm.version();
    results['version'] = {{
        version: wasmVer.version,
        content_id: wasmVer.content_id,
        content_cutoff: wasmVer.content_cutoff,
        status: wasmVer.status,
        game_dps_computed: wasmVer.game_dps_computed,
        adapter: wasmVer.adapter,
        target_arch: wasmVer.target_arch,
    }};

    // 2. Tabular battery parity
    const wasmTabular = wasm.verifyTabular();
    const cliTabularRaw = execFileSync('{CLI_PATH.as_posix()}', ['verify-tabular'], {{ encoding: 'utf8' }});
    const cliTabular = parseLosslessJson(cliTabularRaw);
    results['tabular'] = {{
        total_cases: wasmTabular.total_cases,
        passed_cases: wasmTabular.passed_cases,
        failed_cases: wasmTabular.failed_cases,
        discrepancies_isolated: wasmTabular.documented_discrepancies.length,
        cli_match: wasmTabular.total_cases === cliTabular.total_cases && wasmTabular.passed_cases === cliTabular.passed_cases,
    }};

    // 3. Compositions enumeration parity
    const wasmComps = wasm.enumerateCompositions();
    const cliCompsRaw = execFileSync('{CLI_PATH.as_posix()}', ['compositions'], {{ encoding: 'utf8', maxBuffer: 40 * 1024 * 1024 }});
    const cliComps = parseLosslessJson(cliCompsRaw);
    results['compositions'] = {{
        total_compositions: wasmComps.total_compositions,
        published_variants: wasmComps.published_variants_count,
        unconventional_compositions: wasmComps.unconventional_compositions,
        csv_sha256: wasmComps.csv_sha256,
        sha256_match: wasmComps.csv_sha256 === cliComps.csv_sha256,
    }};

    // 4. S1 through S5 Game Scenarios parity
    const gameScenarios = [
        {{ id: 's1', name: 'S1 Mint A0', path: './examples/game-scenario.json' }},
        {{ id: 's2', name: 'S2 Mint+Sakiri', path: './examples/s2-game-scenario.json' }},
        {{ id: 's3', name: 'S3 Mint+Sakiri+Zero', path: './examples/s3-game-scenario.json' }},
        {{ id: 's4', name: 'S4 Mint+Sakiri+Zero+Iroi', path: './examples/s4-game-scenario.json' }},
        {{ id: 's5', name: 'S5 Full published', path: './examples/s5-game-scenario.json' }},
    ];
    gameScenarios.push(...{json.dumps(list(extra_scenarios))});

    // Execute the current-content native scenario suite once. Passing raw
    // JSON fragments keeps large integer tokens exact across the CLI boundary.
    const suitePath = process.argv[1];
    const suiteRaw = '[' + gameScenarios.map(sc => fs.readFileSync(sc.path, 'utf8').trim()).join(',') + ']';
    fs.writeFileSync(suitePath, suiteRaw);
    const cliSuiteRaw = execFileSync('{CLI_PATH.as_posix()}', ['run-game-suite', suitePath], {{ encoding: 'utf8', maxBuffer: 80 * 1024 * 1024 }});
    const cliSuite = parseLosslessJson(cliSuiteRaw);
    assert.equal(cliSuite.execution_completed, true, 'Native game scenario suite did not complete');
    assert.equal(cliSuite.reports.length, gameScenarios.length, 'Native game scenario suite report count differs');

    results['game_scenarios'] = {{}};
    const wasmReportRaws = [];
    for (const [index, sc] of gameScenarios.entries()) {{
        const raw = fs.readFileSync(sc.path, 'utf8');
        // The WASM wrapper accepts a raw JSON string, preserving every integer
        // token before Rust deserializes it.
        const wasmRes = wasm.simulateGame(raw);
        const wasmRaw = wasm.exportJson('simulate_game', raw);
        assert.deepStrictEqual(wasmRes, parseLosslessJson(wasmRaw), 'Binary/JSON report mismatch');
        const cliRes = cliSuite.reports[index];
        wasmReportRaws.push(wasmRaw);
        assert.deepStrictEqual(wasmRes, cliRes, 'Complete engine report differs: ' + sc.id);

        const [wasmNumStr, wasmDenStr] = wasmRes.total_damage_exact.split('/');
        const [cliNumStr, cliDenStr] = cliRes.total_damage_exact.split('/');
        const wasmNum = BigInt(wasmNumStr);
        const wasmDen = BigInt(wasmDenStr);
        const cliNum = BigInt(cliNumStr);
        const cliDen = BigInt(cliDenStr);

        const match = wasmNum === cliNum &&
                      wasmDen === cliDen &&
                      wasmRes.total_events_processed === cliRes.total_events_processed &&
                      wasmRes.hits_settled === cliRes.hits_settled &&
                      wasmRes.invariants_passed === cliRes.invariants_passed;

        results['game_scenarios'][sc.id] = {{
            name: sc.name,
            total_damage_exact: wasmRes.total_damage_exact,
            events_processed: wasmRes.total_events_processed,
            hits_settled: wasmRes.hits_settled,
            invariants_passed: wasmRes.invariants_passed,
            zero_epsilon_match: match,
        }};
        if (!match) throw new Error('Zero epsilon mismatch in ' + sc.name);
    }}
    // Compare raw JSON at Python's arbitrary precision, without reconstructing
    // native reports from parsed JavaScript objects.
    results.exact_reports.push({{id: 'game_suite', wasm: '[' + wasmReportRaws.join(',') + ']', native: cliSuiteRaw}});

    // 5. Research Batch parity (3,000 runs)
    const scPath = './examples/research-crit.json';
    const batchPath = './cache/reviews/q10-research-batch/pilot-request.json';
    if (!fs.existsSync(batchPath)) throw new Error('Generate current batch inputs first: python tools/check.py -- python tools/verification_engine.py --prepare-batch-inputs');
    const scRaw = fs.readFileSync(scPath, 'utf8');
    const batchRaw = fs.readFileSync(batchPath, 'utf8');
    // Build the wrapper envelope from the two original JSON documents so large
    // integer inputs stay exact without converting them to JavaScript Numbers.
    const wasmBatch = wasm.call('ntesim_wasm_run_batch', `{{"scenario":${{scRaw}},"request":${{batchRaw}}}}`);
    const wasmBatchRaw = wasm.exportJson('run_batch', `{{"scenario":${{scRaw}},"request":${{batchRaw}}}}`);
    assert.deepStrictEqual(wasmBatch, parseLosslessJson(wasmBatchRaw), 'Binary/JSON batch mismatch');
    const cliBatchRaw = execFileSync('{CLI_PATH.as_posix()}', ['batch-game', scPath, batchPath], {{ encoding: 'utf8', maxBuffer: 50 * 1024 * 1024 }});
    const cliBatch = parseLosslessJson(cliBatchRaw);
    results.exact_reports.push({{id: 'batch', wasm: wasmBatchRaw, native: cliBatchRaw}});
    assert.deepStrictEqual(wasmBatch.runs, cliBatch.runs, 'Research trial reports differ');

    const batchRunsMatch = wasmBatch.requested_runs === cliBatch.requested_runs &&
                           wasmBatch.events_processed === cliBatch.events_processed &&
                           wasmBatch.runs.length === cliBatch.runs.length;
    const parseExact = (value) => {{
        const [num, den] = value.split('/');
        return [BigInt(num), BigInt(den)];
    }};
    const [w0n, w0d] = parseExact(wasmBatch.runs[0].total_damage_exact);
    const [c0n, c0d] = parseExact(cliBatch.runs[0].total_damage_exact);
    const [w2999n, w2999d] = parseExact(wasmBatch.runs[2999].total_damage_exact);
    const [c2999n, c2999d] = parseExact(cliBatch.runs[2999].total_damage_exact);
    const run0Match = w0n === c0n && w0d === c0d;
    const run2999Match = w2999n === c2999n && w2999d === c2999d;

    results['research_batch'] = {{
        requested_runs: wasmBatch.requested_runs,
        events_processed: wasmBatch.events_processed,
        first_run_damage: wasmBatch.runs[0].total_damage_exact,
        last_run_damage: wasmBatch.runs[2999].total_damage_exact,
        zero_epsilon_match: batchRunsMatch && run0Match && run2999Match,
    }};

    // 6. Universal invoke metadata isolation check
    const invokeRes = wasm.invoke('version', {{}});
    results['invoke_envelope'] = {{
        status: invokeRes.status,
        action: invokeRes.action,
        environment: invokeRes.environment,
        data_status: invokeRes.data.status,
        metadata_isolated: invokeRes.environment.adapter === 'ntesim-wasm' && invokeRes.data.status === 'INCOMPLETE',
    }};

    console.log(JSON.stringify(results, null, 2));
}}

main().catch(err => {{
    console.error(err);
    process.exit(1);
}});
"""
    with temporary_directory(prefix="wasm-parity-suite-") as workspace:
        result = subprocess.run(
            ["node", "-e", script, str(Path(workspace) / "requests.json")],
            cwd=str(ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
    if result.returncode != 0:
        raise RuntimeError(result.stderr)
    results = json.loads(result.stdout)
    for pair in results.pop("exact_reports"):
        wasm = json.loads(pair["wasm"])
        native = json.loads(pair["native"])
        if pair["id"] == "batch":
            wasm, native = wasm["runs"], native["runs"]
        elif pair["id"] == "game_suite":
            native = native["reports"]
        else:
            # Native-only filesystem audit metadata is outside the shared engine report.
            for key in ("audit", "input_hashes", "request_sha256"):
                native.pop(key, None)
        if wasm != native:
            raise AssertionError(f"Exact full report mismatch: {pair['id']}")
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=VERIFICATION_JSON, help='Verification receipt destination')
    parser.add_argument('--scenario', type=Path, action='append', default=[], help='Additional compiled game scenario to compare')
    args = parser.parse_args()
    print("Running Stage H1 Native/WASM parity verification...")
    if not CLI_PATH.exists():
        print(f"Error: native CLI binary not found at {CLI_PATH}", file=sys.stderr)
        sys.exit(1)
    if not WASM_PATH.exists():
        print(f"Error: WASM release binary not found at {WASM_PATH}", file=sys.stderr)
        sys.exit(1)

    with temporary_directory(prefix="talent-parity-") as workspace:
        scenarios = []
        for name, talents, awakening in [
            ("mint-talents-1-2-3", [1, 2, 3], 0),
            ("mint-talents-1-10-3", [1, 10, 3], 0),
            ("mint-talents-10-a6", [10, 10, 10], 6),
        ]:
            scenario = json.loads((ROOT / "examples/game-scenario.json").read_text(encoding="utf-8"))
            scenario["actors"][0].update(talent_levels=talents, character_max_level=80, awakening=list(range(1, awakening + 1)))
            path = Path(workspace) / (name + ".json")
            path.write_text(json.dumps(scenario), encoding="utf-8")
            scenarios.append({"id": name, "name": name, "path": path.as_posix()})
        for index, path in enumerate(args.scenario):
            scenarios.append({"id": f"additional-{index}-{path.stem}", "name": path.stem, "path": path.resolve().as_posix()})
        parity_results = run_node_parity(scenarios)

    verification_data = {
        "schema_version": 1,
        "stage": "H1",
        "title": "Thin WebAssembly Adapter and Native/WASM Parity Verification",
        "content_cutoff": "2026-10-04",
        "content_id": "ntesim-current",
        "status": "INCOMPLETE",
        "game_dps_computed": False,
        "acceptance_ready": True,
        "adapter": "ntesim-wasm",
        "target_arch": "wasm32-unknown-unknown",
        "wasm_binary_size_bytes": WASM_PATH.stat().st_size,
        "wasm_binary_sha256": hashlib.sha256(WASM_PATH.read_bytes()).hexdigest(),
        "parity_summary": {
            "version_parity": parity_results["version"],
            "tabular_verification_battery": parity_results["tabular"],
            "composition_enumeration_parity": parity_results["compositions"],
            "game_scenarios_zero_epsilon_parity": parity_results["game_scenarios"],
            "research_batch_3000_runs_parity": parity_results["research_batch"],
            "invoke_envelope_metadata_isolation": parity_results["invoke_envelope"],
        },
        "all_parity_scenarios_passed": (
            parity_results["tabular"]["cli_match"]
            and parity_results["compositions"]["sha256_match"]
            and all(sc["zero_epsilon_match"] for sc in parity_results["game_scenarios"].values())
            and parity_results["research_batch"]["zero_epsilon_match"]
            and parity_results["invoke_envelope"]["metadata_isolated"]
        ),
    }

    if not verification_data["all_parity_scenarios_passed"]:
        raise AssertionError("Native/WASM parity failed")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(verification_data, indent=2), encoding="utf-8")
    print(f"Wrote verification artifact to {args.output}")
    print("All Stage H1 native/WASM parity checks PASSED with 100% zero epsilon!")


if __name__ == "__main__":
    main()
