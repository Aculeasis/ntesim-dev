import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { stringifyWasmInput } from '../../src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../../src/infrastructure/storage/lossless-json.ts';
import { createReplay } from '../../src/domain/replay/model.ts';

export async function verifyResonanceParity({ engine, simulate, input, cli }) {
  const characters = JSON.parse(
    readFileSync(new URL('../../src/generated/catalog.json', import.meta.url), 'utf8'),
  ).characters;
  assert.equal(characters.length, 23);
  for (const { slug } of characters) {
    for (const awakening of [0, 3]) {
      const { request } = engine.compileScript(`options iteration=1 duration=4 crit=fixed;
        ${slug} char lvl=80/80 awakening=${awakening} talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active ${slug}; ${slug} attack;`);
      writeFileSync(input, `[${stringifyWasmInput(request)}]`);
      const native = spawnSync(cli, ['run-game-suite', input], {
        encoding: 'utf8',
        maxBuffer: 8 * 1024 * 1024,
        timeout: 15000,
      });
      assert.ifError(native.error);
      assert.equal(native.status, 0, native.stderr);
      const report = await simulate(request);
      assert.ok(report.execution_completed && report.invariants_passed, `${slug} A${awakening}`);
      assert.deepEqual(report, parseLosslessJson(native.stdout).reports[0]);
      assert.deepEqual(report, engine.simulateGame(request));
      assert.equal(
        createReplay(request, report).stateAt(report.absolute_elapsed_us).hits,
        report.hits_settled,
      );
    }
  }
}
