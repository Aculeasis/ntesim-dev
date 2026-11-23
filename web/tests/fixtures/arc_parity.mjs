import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createReplay } from '../../src/domain/replay/model.ts';

const sources = ['black-tome-cycles', 'officer-whisker-cycles'].map((name) =>
  readFileSync(new URL(`../../../examples/${name}.nte`, import.meta.url), 'utf8'),
);

export function arcCycleSources(tier) {
  const variants = sources.map((source) => source.replace('refine=1', `refine=${tier}`));
  return variants.concat(
    variants[1].replace(
      'target hp=100000000 def=0 resist=0 boss=false;',
      'target hp=1 def=0 resist=0 boss=false;',
    ),
  );
}

export function verifyArcOrigins(request, report) {
  if (!report.action_metadata_ledger?.length) return;
  const index = report.hit_receipts.findIndex((hit) =>
    report.player_input_receipts?.some((input) => input.ability_id === hit.ability_id),
  );
  if (index < 0) return;
  const parent = report.player_input_receipts.findIndex(
    (input) => input.command_index === report.hit_receipts[index].command_index,
  );
  const wrongCommand = structuredClone(report);
  wrongCommand.hit_receipts[index].command_index = request.actions.length;
  assert.throws(() => createReplay(request, wrongCommand), /command identity/);
  const wrongAbility = structuredClone(report);
  wrongAbility.player_input_receipts[parent].ability_id = 'weapon:unrelated:arc';
  assert.throws(() => createReplay(request, wrongAbility), /admitted action metadata occurrence/);
  const missing = structuredClone(report);
  delete missing.player_input_receipts[parent].ability_id;
  assert.throws(() => createReplay(request, missing), /admitted action metadata occurrence/);
  const future = structuredClone(report);
  future.player_input_receipts[parent].time_us = report.hit_receipts[index].time_us + 1;
  assert.throws(() => createReplay(request, future), /admitted command/);
}
