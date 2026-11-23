/** One current-content CLI process for a suite; file output avoids JSON buffer limits. */
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { parseLosslessJson } from '../src/infrastructure/storage/lossless-json.ts';
import { stringifyWasmInput } from '../src/infrastructure/wasm/client.ts';

/** Obtain actual native command times from the live Rust source executor. */
export function runSourceFixture(engine, source) {
  const result = engine.invoke('run_script', { source }).data;
  assert.deepEqual(result.diagnostics, []);
  assert.ok(result.execution, 'Missing source execution');
  assert.equal(
    result.execution.diagnostic,
    null,
    JSON.stringify({ diagnostic: result.execution.diagnostic, source }),
  );
  return result.execution;
}

export function runNativeSuite(requests, command = 'run-game-suite', expectedStatus = 0) {
  const root = fileURLToPath(new URL('../../', import.meta.url));
  const cache = path.join(root, 'cache/native-suites');
  fs.mkdirSync(cache, { recursive: true });
  const work = fs.mkdtempSync(path.join(cache, 'run-'));
  const input = path.join(work, 'requests.json');
  const output = path.join(work, 'reports.json');
  fs.writeFileSync(input, stringifyWasmInput(requests));
  const fd = fs.openSync(output, 'w');
  let result;
  try {
    result = spawnSync(
      path.join(root, 'target/debug', process.platform === 'win32' ? 'ntesim.exe' : 'ntesim'),
      [command, input],
      { encoding: 'utf8', timeout: 60000, stdio: ['ignore', fd, 'pipe'] },
    );
  } finally {
    fs.closeSync(fd);
  }
  assert.ifError(result.error);
  assert.equal(
    result.status,
    expectedStatus,
    `Native suite failed: ${result.stderr}; artifacts: ${work}`,
  );
  const native = parseLosslessJson(fs.readFileSync(output, 'utf8'));
  assert.equal(native.reports.length, requests.length);
  fs.rmSync(work, { recursive: true, force: true });
  return native;
}

export function assertNativeParity(cases) {
  // Bound retained full-report batches; every input is still compared.
  for (let offset = 0; offset < cases.length; offset += 64) {
    const batch = cases.slice(offset, offset + 64);
    const complete = batch.every(
      ({ report }) => report.success && report.execution_completed && report.invariants_passed,
    );
    const native = runNativeSuite(
      batch.map(({ request }) => request),
      'run-game-suite',
      complete ? 0 : 2,
    );
    assert.equal(native.execution_completed, complete);
    for (const [index, { request, report }] of batch.entries()) {
      assert.deepEqual(
        native.reports[index],
        report,
        `Full native/WASM parity: scenario ${offset + index}, ${request.actors.map((a) => a.character_id).join(', ')}`,
      );
    }
  }
}
