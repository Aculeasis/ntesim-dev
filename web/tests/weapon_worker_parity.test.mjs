import { verifyResonanceParity } from './fixtures/resonance_parity.mjs';
import { verifyWeaponUpgradeParity } from './fixtures/weapon_upgrade_parity.mjs';
import { verifySwitchReactionParity } from './fixtures/switch_reaction_parity.mjs';
import assert from 'node:assert/strict';
import { readFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { Worker } from 'node:worker_threads';
import { NtesimWasm } from '../src/infrastructure/wasm/client.ts';
import { createReplay } from '../src/domain/replay/model.ts';
import { runSourceFixture, assertNativeParity } from './native-suite.mjs';

const bytes = readFileSync(
  new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
);
const engine = await NtesimWasm.create(bytes);
const cache = new URL('../../cache/verification/', import.meta.url);
mkdirSync(cache, { recursive: true });
const directory = mkdtempSync(new URL('weapon-worker-', cache));
const input = `${directory}/scenario.json`;
const cli = fileURLToPath(
  new URL(
    process.platform === 'win32' ? '../../target/debug/ntesim.exe' : '../../target/debug/ntesim',
    import.meta.url,
  ),
);
const worker = new Worker(new URL('./node_worker_bridge.mjs', import.meta.url), {
  workerData: { script: new URL('../src/infrastructure/worker/worker.ts', import.meta.url).href },
});
let id = 0;
const nativeCases = [];
function simulate(request) {
  return new Promise((resolve, reject) => {
    const requestId = ++id;
    const timeout = setTimeout(() => failure(new Error('Worker response timeout')), 15000);
    const cleanup = () => {
      clearTimeout(timeout);
      worker.off('message', receive);
      worker.off('error', failure);
    };
    const failure = (error) => {
      cleanup();
      reject(error);
    };
    const receive = (message) => {
      if (message.id !== requestId) return;
      if (message.type === 'ERROR') failure(new Error(message.error));
      if (message.type === 'SUCCESS') {
        cleanup();
        resolve(message.result);
      }
    };
    worker.on('message', receive);
    worker.once('error', failure);
    worker.postMessage({
      id: requestId,
      type: 'SIMULATE_GAME',
      payload: request,
      wasmSource: bytes,
    });
  });
}
try {
  await verifySwitchReactionParity({ engine, simulate, input, cli });
  const assumption = (name) => engine.invoke('model_assumption', name).data;
  assert.throws(() => assumption('unknown.model'), /unknown model assumption/);
  for (const tier of [1, 2, 3, 4, 5]) {
    const { request } = engine.compileScript(`options iteration=1 duration=12 crit=fixed;
      baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      baicang add weapon=fork_jingmotingyuan refine=${tier};
      target hp=100000000 def=0 resist=0;
      active baicang; baicang attack; wait(6); baicang attack;`);
    request.model_assumptions['combat.hp_cost_fixture_v1'] = assumption(
      'combat.hp_cost_fixture_v1',
    );
    request.hp_cost_events = [
      { timestamp_us: 0, actor_index: 0, source: 'scenario.hp_consumption', amount: 50 },
      { timestamp_us: 1000000, actor_index: 0, source: 'scenario.hp_consumption', amount: 100 },
    ];
    if (tier === 5)
      request.hp_cost_events.push({
        timestamp_us: 2000000,
        actor_index: 0,
        source: 'scenario.wide_hp_consumption',
        amount: 18446744073709551615n,
      });
    const report = await simulate(request);
    nativeCases.push({ request, report });
    assert.deepEqual(report, engine.simulateGame(request));
    const replay = createReplay(request, report);
    assert.equal(replay.stateAt(report.absolute_elapsed_us).hits, report.hits_settled);
    assert.equal(
      replay.events.filter((event) => event.kind === 'actor_hp_cost').length,
      report.hp_cost_receipts.length,
    );
  }
  for (const tier of [1, 2, 3, 4, 5]) {
    const { request } = engine.compileScript(`options iteration=1 duration=30 crit=random;
      baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      baicang add weapon=fork_jingmotingyuan refine=${tier};
      target hp=100000000 def=0 resist=0;
      active baicang; baicang arc; wait(25); baicang arc;`);
    const report = await simulate(request);
    nativeCases.push({ request, report });
    assert.deepEqual(report, engine.simulateGame(request));
    const replay = createReplay(request, report);
    assert.equal(replay.stateAt(report.absolute_elapsed_us).hits, report.hits_settled);
    assert.ok(replay.events.filter((event) => event.hit).every((event) => event.action === 'Arc'));
    assert.equal(report.player_input_receipts.length, 2);
    if (tier === 1) {
      const changed = structuredClone(report);
      const binding = changed.metadata_catalog[changed.hit_receipts[0].metadata_id].binding;
      binding.provenance = {
        kind: 'accepted_model_hypothesis',
        hypothesis_id: 'test.invalid_child_binding',
      };
      changed.resolved_model_assumptions['test.invalid_child_binding'] =
        'Mismatched packet provenance.';
      assert.throws(() => createReplay(request, changed), /Owner child action\/packet binding/);
    }
  }
  for (const tier of [1, 2, 3, 4, 5])
    for (const boss of [false, true]) {
      const source = `options iteration=1 duration=20 crit=fixed;
      hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
      hathor add weapon=fork_TigerTally refine=${tier};
      hathor add stats dmg%=0.25;
      target hp=100000000 def=0 resist=0 boss=${boss};
      active hathor; hathor skill, burst; wait(1); hathor attack; wait(10); hathor attack;`;
      const { request } = runSourceFixture(engine, source);
      const report = await simulate(request);
      nativeCases.push({ request, report });
      assert.deepEqual(report, engine.simulateGame(request));
      assert.equal(
        createReplay(request, report).stateAt(report.absolute_elapsed_us).hits,
        report.hits_settled,
      );
    }
  for (const owner of ['mint', 'haniel'])
    for (const dead of [false, true]) {
      const weapon = owner === 'mint' ? 'fork_lingganzhongjiezhe' : 'fork_rishi';
      const action = owner === 'mint' ? 'skill' : 'arc';
      const { request } = engine.compileScript(`options iteration=1 duration=2 crit=fixed;
        ${owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=${owner === 'mint' ? 100 : 120};
        ${owner} add weapon=${weapon} refine=1;
        target hp=100000000 def=0 resist=0;
        active ${owner}; ${owner} ${action};`);
      if (dead) {
        request.model_assumptions['combat.incoming_damage_fixture_v1'] = assumption(
          'combat.incoming_damage_fixture_v1',
        );
        request.incoming_damage_events = [
          { timestamp_us: 0, actor_index: 0, delivered_damage: request.actors[0].initial_hp },
        ];
      }
      const report = await simulate(request);
      nativeCases.push({ request, report });
      assert.deepEqual(report, engine.simulateGame(request));
      assert.equal(
        createReplay(request, report).stateAt(report.absolute_elapsed_us).hits,
        report.hits_settled,
      );
    }
  assertNativeParity(nativeCases);
  await verifyResonanceParity({ engine, simulate, input, cli });
  await verifyWeaponUpgradeParity({ engine, simulate, input, cli });
} finally {
  await worker.terminate();
  rmSync(directory, { recursive: true });
}
console.log(
  'Equipped T1–T5 Boss/HP scenarios, 23 owners at A0/A3 and 51 weapons across seven progression stages: native/WASM/Worker parity, aggregation and replay',
);
