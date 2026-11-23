import assert from 'node:assert/strict';
import { createReplay } from '../src/domain/replay/model.ts';
import { applyActorDelta } from '../src/domain/reports/timeline-fold.ts';

export function checkReplayStateValidation(request, report) {
  assert.ok((report.buff_catalog?.length ?? 0) > 0);
  const badBuffCatalog = structuredClone(report);
  badBuffCatalog.buff_catalog[0].id = '';
  assert.throws(() => createReplay(request, badBuffCatalog), /buff_catalog contains invalid/);
  const snapshotOnEvent = structuredClone(report);
  snapshotOnEvent.state_timeline[1].buffs = [];
  assert.throws(() => createReplay(request, snapshotOnEvent), /buff_diff instead of buffs/);
  const emptyBuffDiff = structuredClone(report);
  emptyBuffDiff.state_timeline[1].buff_diff = { upsert: [], remove: [] };
  assert.throws(() => createReplay(request, emptyBuffDiff), /Empty buff_diff/);
  const badBuffRemove = structuredClone(report);
  badBuffRemove.state_timeline[1].buff_diff = { upsert: [], remove: [999999] };
  assert.throws(() => createReplay(request, badBuffRemove), /inactive instance_id/);
  const removeFrameIdx = report.state_timeline.findIndex(
    (frame, index) => index > 0 && (frame.buff_diff?.remove?.length ?? 0) > 0,
  );
  assert.ok(removeFrameIdx > 0);
  const conflictingDiff = structuredClone(report);
  const removedId = conflictingDiff.state_timeline[removeFrameIdx].buff_diff.remove[0];
  conflictingDiff.state_timeline[removeFrameIdx].buff_diff.upsert = [
    { instance_id: removedId, buff_def_id: 0, stacks: 1, duration: { kind: 'permanent' } },
  ];
  assert.throws(() => createReplay(request, conflictingDiff), /cannot both remove and upsert/);
  const lastEnergyFrameIdx =
    report.state_timeline
      .slice(1, -1)
      .findLastIndex((frame) => frame.actors?.some((a) => a.energy !== undefined)) + 1;
  assert.ok(lastEnergyFrameIdx > 0);
  const badActorDelta = structuredClone(report);
  const energyDelta = badActorDelta.state_timeline[lastEnergyFrameIdx].actors.find(
    (a) => a.energy !== undefined,
  );
  energyDelta.energy++;
  assert.throws(() => createReplay(request, badActorDelta), /final.*state/);
  assert.equal(
    applyActorDelta(
      { ...report.final_actor_states[0], charge_remainder: '1/2' },
      { slot: 0, charge_remainder: null },
    ).charge_remainder,
    undefined,
  );
}
