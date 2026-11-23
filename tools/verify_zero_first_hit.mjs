import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { Worker } from 'node:worker_threads';
import { compileNte } from '../web/tests/native-compiler.ts';
import { NtesimWasm } from '../web/src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../web/src/infrastructure/storage/lossless-json.ts';
import { fraction } from '../web/src/domain/reports/statistics.ts';
import { validateObservations } from '../web/src/domain/reports/validation.ts';

const root = new URL('../', import.meta.url);
const output = new URL('../cache/verification/zero-first-hit/', import.meta.url);
mkdirSync(output, { recursive: true });
const bytes = readFileSync(new URL('../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url));
const wasm = await NtesimWasm.create(bytes);
const source = `options duration=2 iteration=1 crit=fixed;
zero char lvl=80/80 awakening=1 talent=1,1,1 qte=1;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
target hp=10000000 def=1080 resist=0 lvl=80; active zero;
zero skill;
`;
const native = (request, name) => {
  const path = new URL(`${name}.json`, output);
  writeFileSync(path, JSON.stringify(request));
  const cli = fileURLToPath(new URL(`target/debug/ntesim${process.platform === 'win32' ? '.exe' : ''}`, root));
  const report = parseLosslessJson(execFileSync(cli, ['run-game', fileURLToPath(path)], { encoding: 'utf8' }));
  const { audit, input_hashes, request_sha256, ...gameplay } = report;
  return gameplay;
};
const hypothesis = 'zero.a1_first_hit_v1';
const publicRequest = compileNte(source).request;
const publicReport = wasm.simulateGame(publicRequest);
assert.deepEqual(publicReport, native(publicRequest, 'public-script'));
validateObservations(publicRequest, publicReport);
assert.ok(publicReport.resolved_model_assumptions[hypothesis].includes('target level <= Zero level'));
const request = structuredClone(publicRequest);
request.actors[0].talent_levels = [1, 1, 1];
request.actors[0].static_stats.atk = '1000/1';
request.actions[0] = { ...request.actions[0], atk_rate: '1/1', energy_cost: 0, recovery_us: 0, cooldown_us: 0, hit_delay_us: 100 };
const cases = [];
const skillPackets = [
  'GE_Player_Female046_Skill1_Damage',
  'GE_Player_Female046_Skill2_Damage',
  'GE_Player_Female046_Skill3_Damage',
  'GE_Player_Female046_Skill4_Damage',
];
for (const level of [79, 80, 81]) {
  const input = structuredClone(request);
  input.targets[0].level = level;
  const report = wasm.simulateGame(input);
  validateObservations(input, report);
  assert.deepEqual(report, native(input, `level-${level}`));
  assert.equal(report.hits_settled, level <= 80 ? 5 : 4);
  const receipts = report.hit_receipts;
  assert.deepEqual(receipts.slice(0, 4).map((hit) => hit.damage_effect_id), skillPackets);
  assert.equal(receipts.filter((hit) => hit.damage_effect_id.startsWith('GE_Player_Female046_Skill_Kill_Damage')).length, level <= 80 ? 1 : 0);
  const expectedCoefficients = [0.2, 0.2, 1.256, 1.344];
  const actor = input.actors[0];
  const bonus =
    1 +
    fraction(actor.static_stats.damage_bonus) +
    fraction(actor.static_stats.elemental_bonuses.COSMOS);
  const expectedBase = expectedCoefficients.map((rate) => fraction(actor.static_stats.atk) * rate * 0.5 * bonus);
  const expectedA1 = fraction(actor.static_stats.atk) * 2 * 0.8 * bonus;
  const expectedTotal = expectedBase.reduce((total, damage) => total + damage, 0) + (level <= 80 ? expectedA1 : 0);
  assert.ok(Math.abs(fraction(report.total_damage_exact) - expectedTotal) < 1e-9);
  cases.push({ level, hits: report.hits_settled, total: report.total_damage_exact });
}
const overlapping = structuredClone(request);
overlapping.actions.push({ ...overlapping.actions[0], timestamp_us: 10, hit_delay_us: 10 });
overlapping.actions.push({ kind: 'switch_actor', timestamp_us: 15, target_actor_index: 1, cooldown_us: 0 });
const report = wasm.simulateGame(overlapping);
validateObservations(overlapping, report);
assert.deepEqual(report, native(overlapping, 'overlapping-off-field'));
assert.equal(report.hits_settled, 10);
assert.deepEqual(report.hit_receipts.map((hit) => hit.command_index), [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]);
for (const commandIndex of [0, 1]) {
  const hits = report.hit_receipts.filter((hit) => hit.command_index === commandIndex);
  assert.deepEqual(hits.slice(0, 4).map((hit) => hit.damage_effect_id), skillPackets);
  assert.equal(hits.filter((hit) => hit.damage_effect_id.startsWith('GE_Player_Female046_Skill_Kill_Damage')).length, 1);
}
assert.deepEqual(report, wasm.simulateGame(overlapping));
const wrongCommand = structuredClone(report);
wrongCommand.hit_receipts[0].command_index = 2;
assert.throws(() => validateObservations(overlapping, wrongCommand), /command identity/);
const omittedAssumption = structuredClone(report);
for (const action of omittedAssumption.action_metadata_ledger)
  delete action.execution_model_assumption_ids;
assert.throws(() => validateObservations(overlapping, omittedAssumption), /resolved model assumptions/i);
const conflictingAssumption = structuredClone(report);
conflictingAssumption.action_metadata_ledger[0].execution_model_assumption_ids = ['contradictory'];
assert.throws(() => validateObservations(overlapping, conflictingAssumption), /resolved model assumptions/i);
const worker = new Worker(new URL('../web/tests/node_worker_bridge.mjs', import.meta.url), {
  workerData: { script: new URL('../web/src/infrastructure/worker/worker.ts', import.meta.url).href },
});
try {
  const result = new Promise((resolve, reject) => {
    worker.on('error', reject);
    worker.on('message', (message) => {
      if (message.type === 'SUCCESS') resolve(message.result);
      if (message.type === 'ERROR') reject(new Error(message.error));
    });
  });
  worker.postMessage({ id: 1, type: 'SIMULATE_GAME', wasmSource: bytes, payload: overlapping });
  assert.deepEqual(await result, report);
} finally {
  await worker.terminate();
}
writeFileSync(new URL('verification.json', output), JSON.stringify({ status: 'passed', quality: 'incomplete', engine_source_sha256: report.engine_source_sha256, wasm_sha256: createHash('sha256').update(bytes).digest('hex'), cases, full_native_wasm_worker_parity: true, exact_replay: true, model_assumption: report.resolved_model_assumptions[hypothesis] }, null, 2) + '\n');
console.log('Zero A1: four-packet Skills, levels 79/80/81, off-field overlapping casts, per-target first-hit gating, independent damage arithmetic, and full CLI/WASM/Worker report parity.');
