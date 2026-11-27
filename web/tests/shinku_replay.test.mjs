import assert from 'node:assert/strict';
import { createReplay } from '../src/domain/replay/model.ts';
import { engine } from './native-compiler.ts';

// Shinku Watch / WatchEx (triggered from Skill & Attack) and Passive 1 ReactionAOE
// must carry valid OwnerChild provenance through createReplay.
const shinkuSource =
  'options duration=35 crit=fixed;\n' +
  'shinku char lvl=80/80 awakening=5 talent=8,8,8 qte=8 start_energy=60;\n' +
  'skia char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;\n' +
  'mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;\n' +
  'target hp=100000000 def=1080 resist=0;\n' +
  'active skia;\n' +
  'shinku swap;\n' +
  'mint swap;\n' +
  'wait(1);\n' +
  'shinku swap;\n' +
  'wait(3.2);\n' +
  'shinku skill;\n' +
  'wait(19);\n' +
  'shinku burst;\n' +
  'shinku attack:5;\n' +
  'shinku skill;\n' +
  'if .shinku.mods.surging_crimson > 4 { wait(.shinku.mods.surging_crimson - 4); }\n' +
  'shinku burst;\n' +
  'shinku burst;';
const compiled = engine.compileSource(shinkuSource).compiled;
assert.ok(compiled);
const execution = engine.runSourceSample({ compiled, seed: 42 });
assert.equal(execution.report.execution_completed, true, execution.report.halt_diagnostic);
const replay = createReplay(execution.request, execution.report);
assert.equal(replay.stateAt(replay.horizon).hits, execution.report.hits_settled);
assert.ok(
  execution.report.hit_receipts.some((h) => h.damage_effect_id === 'GE_Player_Shinku_Watch_Damage'),
);
assert.ok(
  execution.report.hit_receipts.some(
    (h) => h.damage_effect_id === 'GE_Player_Shinku_WatchEx_Damage',
  ),
);
assert.ok(
  execution.report.hit_receipts.some(
    (h) => h.damage_effect_id === 'GE_Player_Shinku_ReactionAOE_Damage',
  ),
);
console.log('Shinku replay: Watch/WatchEx and Passive 1 ReactionAOE OwnerChild provenance passed');
