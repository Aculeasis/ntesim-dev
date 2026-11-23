import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import { createReplay } from '../../src/domain/replay/model.ts';
import { daffodilScenarios } from './daffodil_resonance.mjs';

export function continuingEquipmentSources(tier) {
  const example = (name) =>
    readFileSync(new URL(`../../../examples/${name}.nte`, import.meta.url), 'utf8').replace(
      'refine=1',
      `refine=${tier}`,
    );
  const danzaburou = example('danzaburou-break-cycles');
  return [
    example('time-bandit-break-cycles'),
    example('bohemian-rose-break-cycles'),
    example('forgotten-hp-cycles'),
    example('rainman-shield-cycles'),
    example('bittercake-defense-cycles'),
    example('blast-candy-burst-cycles'),
    example('boxing-candy-hp-cycles'),
    example('gold-wool-refresh-cycles'),
    example('knight-candy-critical-cycles'),
    example('motor-candy-field-cycles'),
    example('clear-skies-mint-cycles'),
    example('prokaryon-normal-cycles'),
    example('real-music-redirect-cycles'),
    example('first-step-pact-cycles'),
    example('stellar-veil-critical-cycles'),
    example('ravenous-blade-critical-cycles'),
    example('eternal-waltz-mental-cycles'),
    example('raging-flames-redirect-cycles'),
    example('blushing-mirage-cosmos-cycles'),
    example('twinbirds-critical-cycles'),
    ...(tier === 1
      ? [
          example('incoming-shield-cycles'),
          example('stamina-dodge-cycles'),
          example('nanally-riposte-cycles'),
          example('nanally-riposte-underboss'),
        ]
      : []),
    danzaburou,
    danzaburou.replace('aurelia char', 'zero char'),
    danzaburou.replace('active haniel;', 'incoming fadia at=5 damage=100000000; active haniel;'),
  ];
}

function verifyStamina(request, report) {
  if (!request.combat_stamina) return;
  if (report.combat_stamina.spends.length) {
    const wrongOwner = structuredClone(report);
    wrongOwner.combat_stamina.spends[0].actor_index = request.actors.length;
    assert.throws(() => createReplay(request, wrongOwner), /stamina spend/);
  }
  const wrongProfile = structuredClone(report);
  wrongProfile.combat_stamina.profile.initial =
    wrongProfile.combat_stamina.profile.initial === '0/1' ? '1/1' : '0/1';
  assert.throws(() => createReplay(request, wrongProfile), /stamina profile/);
}

export function verifyEquipmentReceipts(request, report) {
  const preventedIndex = report.incoming_damage_receipts?.findIndex(
    (row) => row.prevented_hp_loss > 0,
  );
  if (preventedIndex !== undefined && preventedIndex >= 0) {
    const malformed = structuredClone(report);
    malformed.incoming_damage_receipts[preventedIndex].prevented_hp_loss++;
    assert.throws(() => createReplay(request, malformed), /does not conserve/);
  }
  verifyStamina(request, report);
  const replay = createReplay(request, report);
  assert.equal(replay.stateAt(report.absolute_elapsed_us).hits, report.hits_settled);
  for (const [role, label] of [
    ['critical_riposte', 'Critical Riposte'],
    ['parry_attack', 'Parry Attack'],
  ]) {
    if (
      report.hit_receipts.some(
        (hit) => report.metadata_catalog?.[hit.metadata_id]?.action.input_role === role,
      )
    )
      assert.ok(replay.events.some((event) => event.inputRole === role && event.action === label));
  }
  assert.ok(
    replay.events
      .filter((event) => event.hit?.break_contribution)
      .every((event) => {
        const frame = report.state_timeline[event.frameIndex];
        const previous = report.state_timeline[event.frameIndex - 1];
        return (
          event.action === 'Break' &&
          report.trace
            .slice(previous.trace_end, frame.trace_end)
            .some((trace) => trace.event_kind === 'break_damage')
        );
      }),
  );
}

export function equipmentScenarios(engine) {
  const scenarios = daffodilScenarios(engine).map((scenario) => ({
    ...scenario,
    expectBreakReceipt: true,
  }));
  const template = readFileSync(
    new URL('../../../examples/koinobori-shield-cycles.nte', import.meta.url),
    'utf8',
  ).replace('adler char lvl=80/80 awakening=0', 'adler char lvl=80/80 awakening=[4]');
  for (const tier of [1, 2, 3, 4, 5]) {
    const source = readFileSync(
      new URL('../../../examples/boxing-candy-hp-cycles.nte', import.meta.url),
      'utf8',
    ).replace('refine=1', `refine=${tier}`);
    const { request } = engine.compileScript(source);
    request.targets[0].hp_current = 500000;
    scenarios.push({ source, request });
    for (const qualified of [false, true]) {
      let source = template.replace('refine=1', `refine=${tier}`);
      if (!qualified) source = source.replace('zero char', 'sakiri char');
      const { request } = engine.compileScript(source);
      request.model_assumptions['combat.incoming_damage_fixture_v1'] = engine.invoke(
        'model_assumption',
        'combat.incoming_damage_fixture_v1',
      ).data;
      request.incoming_damage_events = [
        { timestamp_us: 1000000, actor_index: 0, delivered_damage: 3000 },
        { timestamp_us: 21000000, actor_index: 0, delivered_damage: 3000 },
      ];
      scenarios.push({ source, request });
    }
  }
  return scenarios;
}
