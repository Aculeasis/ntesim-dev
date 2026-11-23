import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { NtesimWasm } from '../src/infrastructure/wasm/client.ts';
import { ScriptError } from '../src/infrastructure/wasm/script.ts';
import { parseLosslessJson } from '../src/infrastructure/storage/lossless-json.ts';
import { nativeSourceFixtures } from './native_source_fixtures.mjs';

const { NtesimWasm: Universal } = createRequire(import.meta.url)('../../crates/wasm/js/ntesim.js');
const bytes = readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);
const typed = await NtesimWasm.create(bytes);
const universal = await Universal.create(bytes);
assert.equal(typeof typed.buildStartedAt(), 'string');
assert.ok(typed.buildStartedAt().length > 0);
assert.equal(typed.buildStartedAt(), universal.buildStartedAt());
assert.equal(typed.buildStartedAt(), typed.version().build_started_at);
const path = new URL('../../presets/studio/s1.nte', import.meta.url);
const source = readFileSync(path, 'utf8');
const compiled = typed.compileScript(source);
assert.deepEqual(compiled, universal.invoke('compile_script', { source }).data.compiled);
assert.equal(compiled.language, 'nte-gcsl-1');
assert.equal(compiled.request.actions.length, compiled.actionLocations.length);
assert.equal(typeof compiled.optionRanges[0].valueStart, 'number');
assert.deepEqual(typed.validateScript(source), []);
const cliExecutable = fileURLToPath(
  process.platform === 'win32'
    ? new URL('../../target/debug/ntesim.exe', import.meta.url)
    : new URL('../../target/debug/ntesim', import.meta.url),
);
const cli = spawnSync(cliExecutable, ['compile-script', fileURLToPath(path)], { encoding: 'utf8' });
assert.ifError(cli.error);
assert.equal(cli.status, 0, cli.stderr);
const native = parseLosslessJson(cli.stdout);
assert.deepEqual(typed.compileSource(source), native);
assert.deepEqual(native, universal.invoke('compile_source', { source }).data);
for (const fixture of nativeSourceFixtures) {
  const livePath = new URL(`../../presets/native/${fixture}.nte`, import.meta.url);
  const liveSource = readFileSync(livePath, 'utf8');
  const liveCompiled = typed.compileSource(liveSource);
  if (fixture === 'intensity-control') {
    assert.equal(liveCompiled.compiled.request.actors[0].static_stats.cycle_intensity, '348/1');
    assert.equal(liveCompiled.compiled.request.actors[0].static_stats.break_intensity, '337/2');
  }
  const cliCompiled = spawnSync(cliExecutable, ['compile-script', fileURLToPath(livePath)], {
    encoding: 'utf8',
  });
  assert.ifError(cliCompiled.error);
  assert.equal(cliCompiled.status, 0, cliCompiled.stderr);
  assert.deepEqual(liveCompiled, parseLosslessJson(cliCompiled.stdout));
  assert.deepEqual(universal.invoke('validate_source', { source: liveSource }).data, []);
  const executed = universal.invoke('run_compiled_source', liveCompiled).data;
  assert.equal(executed.execution.report.execution_completed, true);
  assert.deepEqual(executed, universal.invoke('run_script', { source: liveSource }).data);
  assert.deepEqual(executed.execution.report, typed.simulateGame(executed.execution.request));
}
const inspection = typed.inspectScript(source);
assert.deepEqual(inspection.party, native.party);
assert.deepEqual(inspection.diagnostics, []);
const envelope = { action: 'inspect_script', payload: { source } };
assert.deepEqual(universal.call('invoke', envelope), universal.call('invoke', envelope, true));
const inspectCli = spawnSync(cliExecutable, ['inspect-script', fileURLToPath(path)], {
  encoding: 'utf8',
});
assert.ifError(inspectCli.error);
assert.equal(inspectCli.status, 0, inspectCli.stderr);
assert.deepEqual(parseLosslessJson(inspectCli.stdout), inspection);
const invalidRotation = source.replace(/# Rotation[\s\S]*/, 'wait(-1);');
const failed = universal.invoke('compile_script', { source: invalidRotation }).data;
assert.equal(failed.compiled, null);
assert.deepEqual(failed.party, inspection.party);
assert.ok(failed.diagnostics.length > 0);
assert.deepEqual(
  universal.invoke('inspect_script', { source: invalidRotation }).data.party,
  inspection.party,
);
const diagnostics = typed.validateScript('options duration=1; wait(1 2);');
assert.equal(diagnostics[0].code, 'syntax');
assert.equal(typeof diagnostics[0].loc.length, 'number');
const unicodeError = '# 🦀\r\noptions duration=10;\r\nwait(1 2);';
const located = universal.invoke('inspect_script', { source: unicodeError }).data;
assert.equal(located.party, null);
assert.deepEqual(located.diagnostics[0].loc, { line: 3, column: 8, offset: 35, length: 1 });
assert.throws(
  () => typed.compileScript('options duration=1; wait(1 2);'),
  (error) =>
    error instanceof ScriptError && error.code === 'syntax' && error.line === 1 && error.length > 0,
);
const request = compiled.request;
assert.deepEqual(typed.simulateGame(request), universal.simulateGame(request));
assert.throws(() => typed.simulateGame({ schema_version: 1 }), /wasm error/);
