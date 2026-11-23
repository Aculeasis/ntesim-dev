import assert from 'node:assert/strict';
import { createReplay } from '../src/domain/replay/model.ts';
import { ClockIndex } from '../src/domain/reports/clocks.ts';

export function checkReplayClocks(engine, report) {
  const stoppedClocks = new ClockIndex({
    ...report,
    absolute_elapsed_us: 50_000_000,
    relative_elapsed_us: 40_000_000,
    absolute_limit_us: 60_000_000,
    timer_bindings: { 0: 'Explicit presentation fixture' },
    timer_pauses: [
      {
        owner: { owner: 0, generation: 1 },
        kind: 'active_stop',
        start_absolute_us: 35_000_000,
        end_absolute_us: 45_000_000,
      },
    ],
  });
  const incomingSource = `options duration=3 iteration=1 crit=fixed;
  mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;
  target hp=100000000 def=0 resist=0;
  incoming mint at=1 damage=1;
  incoming mint at=4 damage=1;
  active mint; mint burst;`;
  const incomingCompiled = engine.compileSource(incomingSource).compiled;
  assert.ok(incomingCompiled);
  const incomingExecution = engine.runSourceSample({ compiled: incomingCompiled, seed: 42 });
  assert.equal(incomingExecution.report.execution_completed, true);
  const incomingClocks = new ClockIndex(incomingExecution.report);
  assert.equal(incomingExecution.report.incoming_damage_receipts.length, 1);
  const incomingReceipt = incomingExecution.report.incoming_damage_receipts[0];
  assert.ok(incomingReceipt.time_us > 1_000_000);
  assert.equal(incomingClocks.relativeAt(incomingReceipt.time_us), 1_000_000);
  assert.ok(incomingExecution.report.absolute_elapsed_us > 4_000_000);
  createReplay(incomingExecution.request, incomingExecution.report);
  const earlyIncoming = structuredClone(incomingExecution.report);
  earlyIncoming.incoming_damage_receipts[0].time_us = 1_000_000;
  assert.throws(
    () => createReplay(incomingExecution.request, earlyIncoming),
    /Incoming damage receipt/,
  );
  for (const cursor of [38_000_000, 41_000_000, 38_000_000]) {
    assert.equal(stoppedClocks.relativeAt(cursor), 35_000_000);
    assert.equal(stoppedClocks.objectAt(cursor), cursor);
    assert.equal(
      stoppedClocks.elapsedAt({ kind: 'world' }, cursor) - stoppedClocks.relativeAt(35_000_000),
      0,
    );
  }
  assert.equal(stoppedClocks.relativeAt(46_000_000), 36_000_000);
  assert.equal(stoppedClocks.relativeAt(50_000_000), 40_000_000);
  for (const edit of [
    (value) => {
      value.relative_elapsed_us++;
    },
    (value) => {
      value.timer_pauses[0].end_absolute_us = value.absolute_elapsed_us + 1;
    },
    (value) => {
      delete value.clock_contract;
    },
    (value) => {
      value.timer_bindings = {};
    },
  ]) {
    const value = structuredClone(report);
    if (value.timer_pauses.length === 0)
      value.timer_pauses.push({
        owner: { owner: 0, generation: 99 },
        kind: 'cinematic',
        start_absolute_us: 0,
        end_absolute_us: 1,
      });
    edit(value);
    assert.throws(() => new ClockIndex(value), /clock|pause|duration/i);
  }
}
