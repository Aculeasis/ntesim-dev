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

const root = new URL('../', import.meta.url);
const output = new URL('../cache/verification/damage-bonuses/', import.meta.url);
mkdirSync(output, { recursive: true });
const bytes = readFileSync(new URL('../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url));
const wasm = await NtesimWasm.create(bytes);
const source = `options duration=2 iteration=1 crit=fixed;
mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
mint add stats dmg%=0.1 chaos%=0.01 cosmos%=0.02 incantation%=0.03 lakshana%=0.04 anima%=0.05 psyche%=0.06;
mint add stats dmg%=0.02 cosmos%=0.01;
sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
target hp=10000000 def=0 resist=0; active mint;
mint attack;
`;
const request = compileNte(source).request;
assert.equal(fraction(request.actors[0].static_stats.damage_bonus), 0.12);
assert.equal(fraction(request.actors[0].static_stats.elemental_bonuses.COSMOS), 0.03);
assert.equal(request.actors[0].static_stats.elemental_bonus, '0/1');
const native = (request, name) => {
  const path = new URL(`${name}.json`, output);
  writeFileSync(path, JSON.stringify(request));
  const cli = fileURLToPath(new URL(`target/debug/ntesim${process.platform === 'win32' ? '.exe' : ''}`, root));
  const report = parseLosslessJson(execFileSync(cli, ['run-game', fileURLToPath(path)], { encoding: 'utf8' }));
  const { audit, input_hashes, request_sha256, ...gameplay } = report;
  return gameplay;
};
const publicReport = wasm.simulateGame(request);
assert.deepEqual(publicReport, native(request, 'public-script'));
// Frozen GE_Player_Mint_Melee1_Damage_New coordinate 0 is 0.104 = 13/125.
// Source ATK 176+296+124=596; general 12% + Anima 5% -> 72.52128.
assert.equal(publicReport.hit_receipts[0].non_crit_damage, '226629/3125');
const mixed = structuredClone(request);
mixed.actors[0].static_stats.atk = '1000/1';
delete mixed.actors[0].talent_levels;
mixed.model_assumptions['verification.mixed_packets'] = 'Explicit synthetic packet metadata isolates shared bonus selection; native packet reachability is not asserted.';
mixed.actions = [];
mixed.research_action_metadata = {};
const elements = ['CHAOS', 'COSMOS', 'INCANTATION', 'LAKSHANA', 'NATURE', 'PSYCHE'];
for (const [index, element] of elements.entries()) {
  mixed.actions.push({ kind: 'invoke_ability', timestamp_us: index * 10, actor_index: 0, ability_id: 'bonus-test', array_index: 0, damage_effect_id: 'bonus-effect', atk_rate: '1/1', energy_cost: 0, hit_delay_us: 100, recovery_us: 0, cooldown_us: 0 });
  mixed.research_action_metadata[index] = { hypothesis_id: 'verification.mixed_packets', ability_id: 'bonus-test', damage_effect_id: 'bonus-effect', action_gameplay_tags: ['Ability.Skill'], damage_type_ex: `EExecutionDamageTypeEX::DAMAGE_TYPE_${element}`, fixed_crit_rate: { kind: 'no_override' } };
}
mixed.actions.push({ kind: 'switch_actor', timestamp_us: 60, target_actor_index: 1, cooldown_us: 0 });
const report = wasm.simulateGame(mixed);
assert.equal(report.execution_completed, true);
const expected = [1130, 1150, 1150, 1160, 1170, 1180];
for (const [index, hit] of report.hit_receipts.entries())
  assert.equal(hit.non_crit_damage, `${expected[index]}/1`);
assert.deepEqual(report, native(mixed, 'mixed-packets'));
assert.deepEqual(report, wasm.simulateGame(mixed));
const worker = new Worker(new URL('../web/tests/node_worker_bridge.mjs', import.meta.url), {
  workerData: { script: new URL('../web/src/infrastructure/worker/worker.ts', import.meta.url).href },
});
try {
  const workerReport = new Promise((resolve, reject) => {
    worker.on('error', reject);
    worker.on('message', (message) => {
      if (message.type === 'SUCCESS') resolve(message.result);
      if (message.type === 'ERROR') reject(new Error(message.error));
    });
  });
  worker.postMessage({ id: 1, type: 'SIMULATE_GAME', wasmSource: bytes, payload: mixed });
  assert.deepEqual(await workerReport, report);
} finally {
  await worker.terminate();
}
writeFileSync(new URL('verification.json', output), JSON.stringify({ status: 'passed', quality: 'incomplete', engine_source_sha256: report.engine_source_sha256, wasm_sha256: createHash('sha256').update(bytes).digest('hex'), public_native_packet_damage: publicReport.hit_receipts[0].non_crit_damage, mixed_packet_expected: expected, full_native_wasm_worker_parity: true, exact_replay: true }, null, 2) + '\n');
console.log('General/element bonuses: native Mint arithmetic, all six off-field packet elements, exact CLI/WASM/Worker report parity.');
