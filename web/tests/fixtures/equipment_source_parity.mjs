import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { Worker } from 'node:worker_threads';
import { NtesimWasm } from '../../src/infrastructure/wasm/client.ts';
import { SimulationController } from '../../src/infrastructure/worker/controller.ts';
import { parseLosslessJson } from '../../src/infrastructure/storage/lossless-json.ts';
import { runSourceFixture, assertNativeParity } from '../native-suite.mjs';
import {
  equippedCycleSources,
  reactionSetSources,
  verifyReplacementReplay,
} from './switch_reaction_parity.mjs';
import { equippedDotSources } from './equipped_dot_parity.mjs';
import { arcCycleSources, verifyArcOrigins } from './arc_parity.mjs';
import { equipmentScenarios, verifyEquipmentReceipts } from './equipment_scenarios.mjs';

export async function verifyEquipmentSourceParity(tiers) {
  const bytes = readFileSync(
    new URL('../../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url),
  );
  const engine = await NtesimWasm.create(bytes);
  const cache = new URL('../../../cache/verification/', import.meta.url);
  mkdirSync(cache, { recursive: true });
  const directory = mkdtempSync(new URL('equipment-source-', cache));
  assert.ok(directory.startsWith(fileURLToPath(cache)));
  const cli = fileURLToPath(
    new URL(
      process.platform === 'win32'
        ? '../../../target/debug/ntesim.exe'
        : '../../../target/debug/ntesim',
      import.meta.url,
    ),
  );
  const controller = new SimulationController({
    wasmSource: bytes,
    timeoutMs: 15000,
    workerScript: new URL('../../src/infrastructure/worker/worker.ts', import.meta.url).href,
    workerFactory: (script) => {
      const worker = new Worker(new URL('../node_worker_bridge.mjs', import.meta.url), {
        workerData: { script },
      });
      const transport = {
        onmessage: null,
        onerror: null,
        postMessage: (message) => worker.postMessage(message),
        terminate: () => worker.terminate(),
      };
      worker.on('message', (data) => transport.onmessage?.({ data }));
      worker.on('error', (error) => transport.onerror?.({ message: error.message }));
      return transport;
    },
  });
  const simulate = (request) => controller.execute('SIMULATE_GAME', request);
  const nativeCases = [];
  try {
    const equipmentSources = [];
    if (tiers.includes(1)) {
      equipmentSources.push(
        ...[2, 4].map(
          (count) => `options iteration=1 duration=2 crit=fixed;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      mint add cartridge=Suit3 count=${count}; mint add stats dmg%=0.25;
      target hp=100000000 def=0 resist=0; active mint; mint attack;`,
        ),
      );
      equipmentSources.push(`options iteration=1 duration=8 crit=fixed;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    target hp=100000000 def=1080 resist=0.2 boss=true break_gauge=1/1000 max_break_gauge=1/1000 broken_duration=0.1;
    active mint; mint attack; wait(2); mint attack; wait(2); mint attack;`);
      equipmentSources.push(...reactionSetSources());
    }
    for (const tier of tiers) {
      for (const [actor, weapon] of [
        ['lacrimosa', 'fork_BlackBook'],
        ['edgar', 'fork_jiaojuan'],
        ['baicang', 'fork_dustbin'],
      ]) {
        equipmentSources.push(`options iteration=1 duration=8 crit=fixed;
        ${actor} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        ${actor} add weapon=${weapon} refine=${tier};
        target hp=100000000 def=1080 resist=0.2 break_gauge=1/1000 max_break_gauge=30 broken_duration=0.1;
        active ${actor}; ${actor} attack; wait(2); ${actor} attack;`);
      }
      equipmentSources.push(...equippedDotSources(tier));
      equipmentSources.push(...equippedCycleSources(tier), ...arcCycleSources(tier));
      equipmentSources.push(`options iteration=1 duration=50 crit=fixed;
      baicang char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      baicang add weapon=fork_dustbin refine=${tier};
      target hp=100000000 def=1080 resist=0.2 break_gauge=1/1000 max_break_gauge=1/1000 broken_duration=0.1;
      active baicang; baicang attack; wait(10); baicang attack; wait(10);
      baicang attack; wait(10); baicang attack; wait(10); baicang attack;`);
      equipmentSources.push(`options iteration=1 duration=310 crit=fixed;
      haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
      haniel add weapon=fork_rishi refine=${tier};
      target hp=100000000 def=0 resist=0;
      active haniel; haniel arc; wait(300); haniel arc;`);
      equipmentSources.push(`options iteration=1 duration=10 crit=fixed;
      haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=120;
      haniel add weapon=fork_rishi refine=${tier};
      target hp=1 def=0 resist=0; active haniel; haniel arc, burst;`);
    }
    const scenarios = equipmentSources.map((source) => ({ source, request: undefined }));
    if (tiers.includes(1)) scenarios.push(...equipmentScenarios(engine));
    for (const scenario of scenarios) {
      const { source } = scenario;
      const request = scenario.request ?? runSourceFixture(engine, source).request;
      if (
        source.includes('broken_duration=') ||
        /type=|boss=|incoming /.test(source) ||
        /fork_(Rose|Arachne)/.test(source) ||
        /fork_(twinbirds|PaperPlane|Prokaryon|appliance|nonos|moon|DemonBlade)/.test(source) ||
        source.includes('zankou char') ||
        source.includes('cartridge=Suit')
      ) {
        const scriptPath = `${directory}/break-profile.nte`;
        writeFileSync(scriptPath, source);
        const nativeCompile = spawnSync(cli, ['compile-script', scriptPath], {
          encoding: 'utf8',
          maxBuffer: 8 * 1024 * 1024,
          timeout: 15000,
        });
        assert.ifError(nativeCompile.error);
        assert.equal(nativeCompile.status, 0, nativeCompile.stderr);
        assert.deepEqual(
          engine.invoke('compile_source', { source }).data,
          parseLosslessJson(nativeCompile.stdout),
        );
      }
      const report = await simulate(request);
      if (source.includes('while !.zankou.enhanced_soulcross.active')) {
        assert.equal(report.execution_completed, true);
        assert.equal(report.final_actor_states[0].spent_energy, 240);
        assert.equal(
          report.trace.filter((event) => event.event_kind === 'zankou_form_transition').length,
          4,
        );
        for (const effect of [
          'GE_Player_Zankou_Skill3_2_Damage',
          'GE_Player_Zankou_MagicUltraSkill2_Damage',
          'GE_Player_Zankou_ForceUltraSkill2_Damage',
        ]) {
          const hit = report.hit_receipts.find((hit) => hit.damage_effect_id === effect);
          assert.ok(hit);
          const metadata = report.metadata_catalog[hit.metadata_id];
          assert.equal(metadata.packet.damage_effect_id, effect);
          assert.equal(metadata.packet.provenance.kind, 'native');
          assert.ok(report.evidence_catalog[metadata.packet.source_evidence_id]);
        }
      }
      if (scenario.expectBreakReceipt || source.includes('max_break_gauge=30')) {
        assert.ok(report.hit_receipts.some((hit) => hit.break_contribution));
      }
      if (source.includes('fork_dustbin') && source.includes('duration=50')) {
        assert.equal(
          report.trace.filter((event) => event.event_kind === 'dustbin_break_intensity_granted')
            .length,
          3,
        );
        assert.ok(report.hit_receipts.filter((hit) => hit.break_contribution).length >= 5);
      }
      nativeCases.push({ request, report });
      assert.deepEqual(report, engine.simulateGame(request));
      try {
        verifyEquipmentReceipts(request, report);
      } catch (error) {
        error.message += `; source context: ${source.slice(0, 400)}`;
        throw error;
      }
      verifyReplacementReplay(request, report);
      verifyArcOrigins(request, report);
    }
    assertNativeParity(nativeCases);
  } finally {
    controller.terminateWorker();
    rmSync(directory, { recursive: true });
  }
}
