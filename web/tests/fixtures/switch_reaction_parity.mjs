import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { stringifyWasmInput } from '../../src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../../src/infrastructure/storage/lossless-json.ts';
import { createReplay } from '../../src/domain/replay/model.ts';
import { continuingEquipmentSources } from './equipment_scenarios.mjs';
import { verifyImplicitSupportParity } from './implicit_support_parity.mjs';

export function equippedCycleSources(tier) {
  const example = (name) =>
    readFileSync(new URL(`../../../examples/${name}.nte`, import.meta.url), 'utf8').replace(
      'refine=1',
      `refine=${tier}`,
    );
  const suppression = readFileSync(
    new URL('../../../examples/sakiri-suppression-cycles.nte', import.meta.url),
    'utf8',
  );
  const goodBoyControl = example('good-boy-control-cycles');
  const goodBoy = example('good-boy-energy-cycles');
  const mamen = example('contemplative-cat-cycles');
  const mindRoyale = example('mind-royale-energy-cycles');
  const koinobori = example('koinobori-shield-cycles');
  const kite = example('kite-marked-cycles');
  const film = example('film-break-cycles');
  const silentGarden = example('silent-garden-cycles');
  const nakupeda = example('nakupeda-support-cycles');
  const oraora = example('oraora-puncher-cycles');
  const prism = example('prism-gravitide-cycles');
  const snowman = example('snowman-blessing-cycles');
  const spider = example('spider-knowledge-cycles');
  const thiefCandy = example('thief-candy-dodge-cycles');
  const tigerTally = example('tiger-tally-cycles');
  const marchingTime = example('marching-beyond-time-cycles');
  const beHappy = example('be-happy-defeat-cycles');
  const whale = example('whale-break-defeat-cycles');
  const worldrain = example('worldrain-cycle-intensity');
  const jiuyuanPact = example('jiuyuan-natural-pact-cycles');
  const realityRefuge = example('reality-refuge-cycles');
  const zero = example('zero-support-cycles');
  const bopu = example('bopu-support-cycles');
  const bitgame = example('bitgame-hootie-cycles');
  const ace = example('haniel-ace-ensemble');
  const castle = example('castle-healing-cycles');
  const door = example('wrong-gate-healing-cycles');
  const voyager = example('gold-record-support-cycles');
  const catSupport = example('nanally-cat-support');
  return [
    goodBoyControl,
    ...(tier === 1 ? [suppression] : []),
    goodBoy,
    mamen,
    mindRoyale,
    koinobori,
    kite,
    film,
    ...continuingEquipmentSources(tier),
    silentGarden,
    nakupeda,
    oraora,
    prism,
    snowman,
    spider,
    thiefCandy,
    tigerTally,
    tigerTally.replace('boss=true', 'boss=false'),
    marchingTime,
    beHappy,
    beHappy.replace('target hp=2 ', 'target hp=5805 '),
    whale,
    whale.replaceAll(' break_gauge=1/2 max_break_gauge=1 broken_duration=6', ''),
    worldrain,
    realityRefuge,
    realityRefuge.replace('awakening=0 talent=1,1,1', 'awakening=[2] talent=8,1,1'),
    ...(tier === 1 ? [jiuyuanPact] : []),
    worldrain.replace('wait_until(25)', 'wait_until(27.25)'),
    voyager,
    catSupport,
    voyager.replace(
      'nanally char lvl=80/80 awakening=0 talent=1,1,1',
      'nanally char lvl=80/80 awakening=3 talent=1,8,10',
    ),
    door,
    door.replace('awakening=0 talent=1,1,1', 'awakening=[3] talent=1,8,10'),
    castle,
    ace,
    bitgame,
    bitgame.replace('awakening=0 talent=1,1,1', 'awakening=4 talent=1,8,10'),
    bopu,
    bopu.replace(
      'sakiri char lvl=80/80 awakening=0 talent=1,1,1',
      'sakiri char lvl=80/80 awakening=3 talent=1,8,1',
    ),
    zero,
    zero.replace(
      'zero char lvl=80/80 awakening=0 talent=1,1,1',
      'zero char lvl=80/80 awakening=5 talent=1,8,1',
    ),
    `options iteration=1 duration=14 crit=fixed;
    lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    blackbird add weapon=fork_twinbirds refine=${tier};
    target hp=100000000 def=1080 resist=0;
    active lacrimosa; blackbird swap; wait(3); lacrimosa swap;`,
    `options iteration=1 duration=30 crit=fixed;
    blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    blackbird add weapon=fork_twinbirds refine=${tier};
    blackbird add cartridge=Suit3 count=4;
    hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
    target hp=100000000 def=1080 resist=0;
    active blackbird; hathor swap; wait(3); blackbird swap;
    blackbird attack; wait(12); blackbird attack;`,
  ];
}

export function reactionSetSources() {
  return ['blackbird', 'adler']
    .map(
      (other) => `options iteration=1 duration=40 crit=fixed;
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      lacrimosa add cartridge=Suit1 count=4;
      ${other} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      target hp=100000000 def=1080 resist=0.2;
      active lacrimosa; ${other} swap; wait(3); lacrimosa swap;
      lacrimosa attack; wait(20); lacrimosa attack;`,
    )
    .concat([
      `options iteration=1 duration=40 crit=fixed;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=100000000 def=1080 resist=0;
      active mint; sakiri swap; wait(0.6); mint swap;
      wait(14.4); sakiri swap; wait(0.6); mint swap;
      wait(14.4); sakiri swap; wait(0.6);`,
      `options iteration=1 duration=40 crit=fixed;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      mint add cartridge=Suit5 count=4;
      zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      target hp=100000000 def=1080 resist=0;
      active mint; zero swap; wait(3); hathor swap;
      wait(3); blackbird swap; wait(3); hathor swap;
      wait(3); mint swap; mint attack; wait(17); mint attack;`,
      `options iteration=1 duration=15 crit=fixed;
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      blackbird add weapon=fork_twinbirds refine=3;
      zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=100000000 def=1080 resist=0;
      active lacrimosa; blackbird swap; wait(3); lacrimosa swap;`,
      `options iteration=1 duration=40 crit=fixed;
      zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=40;
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=100000000 def=1080 resist=0;
      active zankou; lacrimosa swap; wait(3); zankou swap;
      wait(31); lacrimosa swap;`,
      `options iteration=1 duration=45 crit=fixed;
      adler char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      zankou char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=100000000 def=1080 resist=0;
      active adler; adler skill; lacrimosa swap; wait(10); adler swap; adler skill;`,
    ]);
}

export function verifyReplacementReplay(request, report) {
  const grouped = report.hit_receipts.findIndex((hit) => hit.energy_group_id);
  if (grouped >= 0) {
    const invalid = structuredClone(report);
    invalid.hit_receipts[grouped].energy_group_id = 'packet:invalid';
    assert.throws(() => createReplay(request, invalid), /energy-group identity/);
  }
  const index = report.hit_receipts.findIndex(
    (hit) => hit.switch_origin?.kind === 'esper_cycle_replacement',
  );
  if (index < 0) return;
  const cyclic = structuredClone(report);
  cyclic.hit_receipts[index].switch_origin.trigger_hit_index = index;
  assert.throws(() => createReplay(request, cyclic), /earlier hit link/);
  const wrongOwner = structuredClone(report);
  wrongOwner.hit_receipts[index].switch_origin.owner_actor_index = request.actors.length;
  assert.throws(() => createReplay(request, wrongOwner), /initiating participants/);
  const missing = structuredClone(report);
  delete missing.hit_receipts[index].switch_origin;
  assert.throws(() => createReplay(request, missing), /command identity/);
}

export async function verifySwitchReactionParity({ engine, simulate, input, cli }) {
  await verifyImplicitSupportParity({ engine, simulate, input, cli });
  const swap = (timestamp_us, target_actor_index) => ({
    kind: 'switch_actor',
    timestamp_us,
    target_actor_index,
    cooldown_us: 0,
  });
  const scenarios = [
    ['mint', 'sakiri', 2, 'support'],
    ['zero', 'mint', 12, 'blossom'],
    ['sakiri', 'lacrimosa', 20, 'scorch'],
    ['sakiri', 'lacrimosa', 20, 'partial-policy'],
    ['lacrimosa', 'aurelia', 8.2, 'nova'], // include the second Nova generation-end at8.1s
  ];
  for (const [first, second, duration, kind] of scenarios) {
    const { request } = engine.compileScript(`options iteration=1 duration=${duration} crit=fixed;
      ${first} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      ${second} char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      target hp=100000000 def=1080 resist=0.2;
      active ${first}; wait(1);`);
    request.actions = [swap(0, 1)];
    if (kind !== 'support' && kind !== 'blossom') {
      request.actions.push(swap(3000000, 0));
    }
    if (kind === 'partial-policy') {
      request.stochastic_crit = {
        roll_policy: 'independent_per_target_hit_v1',
        hypothesis: 'Transport fixture: ordinary crit eligibility for incoming actor only.',
        eligible_effects: { [request.actors[1].character_id]: ['*'] },
        actor_sequences: { [request.actors[1].character_id]: 999 },
      };
    }
    for (const seed of [0, 42]) {
      request.seed = seed;
      writeFileSync(input, `[${stringifyWasmInput(request)}]`);
      const native = spawnSync(cli, ['run-game-suite', input], {
        encoding: 'utf8',
        maxBuffer: 8 * 1024 * 1024,
        timeout: 15000,
      });
      assert.ifError(native.error);
      assert.equal(native.status, 0, native.stderr || native.stdout);
      const report = await simulate(request);
      assert.deepEqual(report, parseLosslessJson(native.stdout).reports[0]);
      assert.deepEqual(report, engine.simulateGame(request));
      assert.ok(report.execution_completed && report.invariants_passed, kind);
      const followupIndex = report.hit_receipts.findIndex(
        (hit) => hit.parent_hit_index !== undefined,
      );
      if (kind === 'support')
        assert.ok(followupIndex >= 0, 'Native Hexed follow-up retains its actual damage parent.');
      if (followupIndex >= 0) {
        const wrongParent = structuredClone(report);
        wrongParent.hit_receipts[followupIndex].parent_hit_index = followupIndex;
        assert.throws(() => createReplay(request, wrongParent), /parent/);
      }
      const replay = createReplay(request, report);
      assert.equal(replay.stateAt(report.absolute_elapsed_us).hits, report.hits_settled);
      for (const event of replay.events.filter((event) => event.hit)) {
        const frame = report.state_timeline[event.frameIndex];
        const previous = report.state_timeline[event.frameIndex - 1];
        const hitIndex = report.hit_receipts.indexOf(event.hit);
        assert.ok(
          hitIndex >= previous.hits_end && hitIndex < frame.hits_end,
          'Each engine hit retains its native settlement frame and exact receipt identity.',
        );
      }
      for (const hit of report.hit_receipts) {
        assert.ok(
          hit.switch_origin || hit.autonomous_event_index !== undefined,
          'Switch consequences carry an explicit switch or autonomous origin.',
        );
      }
      const missingOrigin = structuredClone(report);
      delete missingOrigin.hit_receipts[0].switch_origin;
      assert.throws(() => createReplay(request, missingOrigin), /command identity/);
      const wrongParticipant = structuredClone(report);
      wrongParticipant.hit_receipts[0].switch_origin.incoming_actor_index = 0;
      assert.throws(() => createReplay(request, wrongParticipant), /initiating participants/);
      if (kind === 'support') {
        const missingAdmissionOrigin = structuredClone(report);
        delete missingAdmissionOrigin.action_metadata_ledger[0].switch_origin;
        assert.throws(() => createReplay(request, missingAdmissionOrigin), /command identity/);
        const wrongCost = structuredClone(report);
        wrongCost.trace.find((event) => event.event_kind === 'invoke_ability').energy_change
          .spent++;
        assert.throws(() => createReplay(request, wrongCost), /energy cost receipt/);
      }
      if (report.reaction_rng) {
        const wrongSeed = structuredClone(report);
        wrongSeed.reaction_rng.seed++;
        assert.throws(() => createReplay(request, wrongSeed), /RNG audit header/);
      }
      if (report.final_target_states[0].statuses?.length) {
        const wrongOwner = structuredClone(report);
        wrongOwner.final_target_states[0].statuses[0].source_actor_index = request.actors.length;
        assert.throws(() => createReplay(request, wrongOwner), /target status receipt/);
      }
    }
  }
}
