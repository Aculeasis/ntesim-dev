import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { stringifyWasmInput } from '../../src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../../src/infrastructure/storage/lossless-json.ts';
import { createReplay } from '../../src/domain/replay/model.ts';

export async function verifyWeaponUpgradeParity({ engine, simulate, input, cli }) {
  const result = spawnSync(cli, ['studio-build-catalog'], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const catalog = parseLosslessJson(result.stdout);
  const characters = JSON.parse(
    readFileSync(new URL('../../src/generated/catalog.json', import.meta.url), 'utf8'),
  ).characters;
  assert.equal(Object.keys(catalog.weapons).length, 51);
  const progressionCases = ['1/20', '30/30', '40/40', '50/50', '60/60', '70/70', '80/80'];
  let index = 0;
  for (const [weaponId, weapon] of Object.entries(catalog.weapons)) {
    const progression = progressionCases[index++ % progressionCases.length];
    const character = characters.find(({ id }) => catalog.characters[id].group === weapon.group);
    assert.ok(character, weaponId);
    // Explicit zero permits Fons-dependent passive reads without inventing funds.
    const source = `options iteration=1 duration=4 crit=fixed start_fons=0;
      ${character.slug} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      ${character.slug} add weapon=${weaponId.split(':')[1]} lvl=${progression} refine=1;
      target hp=100000000 def=0 resist=0 boss=false;
      active ${character.slug}; ${character.slug} attack;`;
    const compiled = engine.compileScript(source);
    const sourcePath = `${input}.nte`;
    writeFileSync(sourcePath, source);
    const nativeCompile = spawnSync(cli, ['compile-script', sourcePath], {
      encoding: 'utf8',
      maxBuffer: 8 * 1024 * 1024,
    });
    assert.ifError(nativeCompile.error);
    assert.equal(nativeCompile.status, 0, nativeCompile.stderr);
    assert.deepEqual(
      engine.invoke('compile_source', { source }).data,
      parseLosslessJson(nativeCompile.stdout),
    );
    const { request } = compiled;
    writeFileSync(input, `[${stringifyWasmInput(request)}]`);
    const native = spawnSync(cli, ['run-game-suite', input], {
      encoding: 'utf8',
      maxBuffer: 8 * 1024 * 1024,
    });
    assert.ifError(native.error);
    assert.equal(native.status, 0, native.stderr);
    const report = await simulate(request);
    assert.ok(report.execution_completed && report.invariants_passed, weaponId);
    assert.deepEqual(report, parseLosslessJson(native.stdout).reports[0]);
    assert.deepEqual(report, engine.simulateGame(request));
    assert.equal(
      createReplay(request, report).stateAt(report.absolute_elapsed_us).hits,
      report.hits_settled,
    );
  }
}
