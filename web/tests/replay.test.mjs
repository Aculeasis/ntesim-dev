import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import { createReplay } from '../src/domain/replay/model.ts';
import { compileNte, getPresetSource, editOptions } from './native-compiler.ts';
import { stackedBars } from './render-helpers.tsx';
import { eventLocation } from '../src/domain/replay/source.ts';
import {
  actorSummaryFromDelta,
  applyActorDelta,
  applyBuffDiff,
} from '../src/domain/reports/timeline-fold.ts';
import { engine } from './native-compiler.ts';
import { SCRIPT_PRESETS } from '../src/domain/presets';
import { checkReplayClocks } from './replay-clock-cases.mjs';
import { checkReplayStateValidation } from './replay-state-cases.mjs';
const { NtesimWasm } = createRequire(import.meta.url)('../../crates/wasm/js/ntesim.js');
const wasm = await NtesimWasm.create(
  fs.readFileSync(new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url)),
);
let reports = 0;
for (const preset of ['s1', 's2', 's3'])
  for (const crit of ['fixed', 'random']) {
    const { request } = compileNte(editOptions(getPresetSource(preset), { crit }));
    for (const seed of [0, 42, 999]) {
      const report = wasm.simulateGame({ ...request, seed }),
        before = JSON.stringify(report);
      assert.ok(report.invariants_passed && report.execution_completed);
      if (reports === 0) {
        const hitWithPrimary = report.hit_receipts.find(
          (hit) => report.metadata_catalog?.[hit.metadata_id]?.action.primary_slot,
        );
        const action = report.metadata_catalog[hitWithPrimary.metadata_id].action;
        assert.equal(
          report.evidence_catalog[action.identity_evidence_id].table,
          'typed_character_actions',
        );
        for (const table of ['typed_unknown_actions', 'DT_SkillDamageData']) {
          const invalid = structuredClone(report);
          invalid.evidence_catalog[action.identity_evidence_id].table = table;
          assert.throws(() => createReplay({ ...request, seed }, invalid));
        }
        const badMetaId = structuredClone(report);
        badMetaId.hit_receipts[0].metadata_id = badMetaId.metadata_catalog.length;
        assert.throws(() => createReplay({ ...request, seed }, badMetaId), /metadata catalog/);
        const badEvidenceId = structuredClone(report);
        badEvidenceId.metadata_catalog[hitWithPrimary.metadata_id].action.identity_evidence_id =
          badEvidenceId.evidence_catalog.length;
        assert.throws(
          () => createReplay({ ...request, seed }, badEvidenceId),
          /evidence reference/,
        );
        const emptyIndices = structuredClone(report);
        emptyIndices.action_metadata_ledger[0].command_indices = [];
        assert.throws(
          () => createReplay({ ...request, seed }, emptyIndices),
          /at least one command index/,
        );
        const dupIndices = structuredClone(report);
        const firstIdx = dupIndices.action_metadata_ledger[0].command_indices[0];
        dupIndices.action_metadata_ledger[0].command_indices = [firstIdx, firstIdx];
        assert.throws(
          () => createReplay({ ...request, seed }, dupIndices),
          /duplicate command indices/,
        );
        const unreferencedAssumption = structuredClone(report);
        unreferencedAssumption.resolved_model_assumptions['test.unreferenced_hypothesis'] =
          'Unused assumption entry.';
        assert.throws(
          () => createReplay({ ...request, seed }, unreferencedAssumption),
          /Resolved model assumptions differ/,
        );
      }
      const replay = createReplay({ ...request, seed }, report),
        end = replay.stateAt(report.absolute_elapsed_us);
      assert.equal(replay.events.filter((e) => e.hit).length, report.hit_receipts.length);
      assert.equal(end.hits, report.hits_settled);
      assert.equal(end.damage, report.total_damage_dealt);
      for (const actual of report.final_actor_states) {
        const replayed = end.actors.find((a) => a.slot === actual.slot);
        for (const [key, field] of [
          ['energy', 'energy'],
          ['spent', 'spent_energy'],
          ['granted', 'granted_energy'],
          ['overflow', 'overflow_energy'],
        ])
          assert.equal(replayed[key], actual[field]);
      }
      assert.deepEqual(end.native.actors, report.final_actor_states);
      assert.deepEqual(end.native.targets, report.final_target_states);
      assert.deepEqual(end.native.buffs, report.final_buff_state.instances);
      assert.equal(end.native.healing_end, report.healing_receipts.length);
      for (const event of replay.events.filter((e) => e.kind === 'state_boundary')) {
        const boundary = replay.stateAt(event.time, event.index + 1);
        assert.equal(boundary.native.frameIndex, event.frameIndex);
        const recorded = report.state_timeline[event.frameIndex];
        const beforeBoundary = replay.stateAt(event.time, event.index);
        assert.equal(beforeBoundary.native.frameIndex, event.frameIndex - 1);
        if (recorded.boundary.kind === 'event') {
          assert.equal(recorded.buffs, undefined);
          for (const actor of recorded.actors ?? []) {
            assert.deepEqual(
              boundary.native.actors[actor.slot],
              applyActorDelta(beforeBoundary.native.actors[actor.slot], actor),
            );
          }
          if (recorded.buff_diff)
            assert.deepEqual(
              boundary.native.buffs,
              applyBuffDiff(beforeBoundary.native.buffs, recorded.buff_diff),
            );
        } else {
          for (const actor of recorded.actors ?? [])
            assert.deepEqual(boundary.native.actors[actor.slot], actorSummaryFromDelta(actor));
          assert.deepEqual(boundary.native.buffs, recorded.buffs);
        }
        for (const target of recorded.targets ?? [])
          assert.deepEqual(boundary.native.targets[target.target_index], target);
      }
      for (const target of end.targets) {
        const input = request.targets.find((t) => t.target_index === target.index);
        const hpLoss = report.hit_receipts
          .filter((h) => h.target_index === target.index)
          .reduce((n, h) => n + h.applied_hp_loss, 0);
        assert.equal(target.hp, input.hp_current - hpLoss);
      }
      assert.equal(
        replay.segments.reduce((n, s) => n + s.end - s.start, 0),
        report.absolute_elapsed_us,
      );
      for (const time of [0, 500000, 1000000, 2500000, 4000000, 5000000, 6500000, 20000000]) {
        const state = replay.stateAt(time),
          t = Math.min(time, report.absolute_elapsed_us);
        const receipts = report.hit_receipts.filter((h) => h.time_us <= t);
        assert.equal(state.hits, receipts.length);
        assert.equal(
          state.damage,
          Number(receipts.reduce((sum, hit) => sum + BigInt(hit.rounded_damage), 0n)),
        );
        const switches = report.trace.filter(
          (e) => e.event_kind === 'switch_actor' && e.time_us <= t,
        );
        assert.equal(state.active, switches.at(-1)?.active_actor ?? request.active_actor_index);
        replay.stateAt(report.absolute_elapsed_us);
        assert.deepEqual(replay.stateAt(time), state);
      }
      for (let i = 0; i < replay.events.length; i++) {
        const state = replay.stateAt(replay.events[i].time, i + 1);
        assert.equal(state.count, i + 1);
        assert.equal(state.last.index, i);
      }
      assert.equal(replay.stateAt(-1, 0).count, 0);
      assert.equal(replay.stateAt(Infinity).time, report.absolute_elapsed_us);
      assert.equal(JSON.stringify(report), before);
      reports++;
    }
  }
for (const preset of SCRIPT_PRESETS) {
  const compiled = engine.compileSource(preset.source).compiled;
  assert.ok(compiled, preset.id);
  const execution = engine.runSourceSample({ compiled, seed: 42 });
  assert.equal(execution.report.execution_completed, true, preset.id);
  let playback;
  assert.doesNotThrow(() => {
    playback = createReplay(execution.request, execution.report);
  }, preset.id);
  assert.equal(playback.stateAt(playback.horizon).hits, execution.report.hits_settled, preset.id);
  if (preset.id === 'lacrimosa-quickswap') {
    const metadata = execution.report.metadata_catalog.find((entry) =>
      entry.packet.semantic_tags?.includes('shatter'),
    );
    assert.ok(metadata, 'Broken-target Shatter must be reached');
    const missingSemantic = structuredClone(execution.report);
    delete missingSemantic.metadata_catalog.find((entry) =>
      entry.packet.semantic_tags?.includes('shatter'),
    ).packet.semantic_provenance;
    assert.throws(
      () => createReplay(execution.request, missingSemantic),
      /Packet semantic tags have no provenance/,
    );
    const missingBinding = structuredClone(execution.report);
    delete missingBinding.metadata_catalog.find((entry) =>
      entry.packet.semantic_tags?.includes('shatter'),
    ).binding.provenance;
    assert.throws(
      () => createReplay(execution.request, missingBinding),
      /Non-direct action\/packet binding has no provenance/,
    );
  }
  if (preset.id === 'nanally-blossom') {
    const orphanCost = structuredClone(execution.report);
    const utility = orphanCost.trace.find((event) => event.event_kind === 'utility_ability');
    assert.ok(utility, 'Nanally removal must reach a utility admission');
    utility.event_kind = 'not_an_admission';
    assert.throws(
      () => createReplay(execution.request, orphanCost),
      /Energy cost outside action admission/,
    );
  }
  if (preset.id === 'zankou-blackbird') {
    const missingOrigin = structuredClone(execution.report);
    const supportBreak = missingOrigin.hit_receipts.find(
      (hit) => hit.break_contribution && hit.switch_origin?.kind === 'support',
    );
    assert.ok(supportBreak, 'Persistent Support damage must reach Break');
    delete supportBreak.switch_origin;
    assert.throws(
      () => createReplay(execution.request, missingOrigin),
      /Hit receipt command identity differs/,
    );
  }
  reports++;
}
const { request } = compileNte(getPresetSource('s2'));
const report = wasm.simulateGame(request);
const offField = structuredClone(report);
offField.trace.forEach((e) => {
  if (
    e.event_kind === 'hit_damage' ||
    (e.energy_change && e.energy_change.source !== 'ability_cost') ||
    e.energy_fanout
  )
    e.active_actor = 1;
});
const replay = createReplay(request, offField);
const mintHits = report.hit_receipts.filter((h) => h.actor_index === 0).length;
assert.equal(replay.stateAt(replay.horizon).actors.find((a) => a.slot === 0).hits, mintHits);
assert.equal(replay.events.filter((e) => e.hit && e.owner === 0).length, mintHits);
assert.match(replay.events.find((e) => e.admitted && e.owner === 0).action, /Containment/);
const fanoutIdx = report.trace.findIndex((e) => e.energy_fanout);
assert.ok(fanoutIdx >= 0);
assert.ok(replay.events.some((e) => e.fanout));
const bothChangeAndFanout = structuredClone(report);
bothChangeAndFanout.trace[fanoutIdx].energy_change = {
  source: bothChangeAndFanout.trace[fanoutIdx].energy_fanout.source,
  source_actor_index: 0,
  recipient_actor_index: 0,
  energy_before: 0,
  gross: 0,
  retained: 0,
  overflow: 0,
  energy_after: 0,
};
assert.throws(
  () => createReplay(request, bothChangeAndFanout),
  /both energy_change and energy_fanout/,
);
const emptyFanout = structuredClone(report);
emptyFanout.trace[fanoutIdx].energy_fanout.recipients = [];
assert.throws(() => createReplay(request, emptyFanout), /Invalid typed energy fanout/);
const dupFanout = structuredClone(report);
dupFanout.trace[fanoutIdx].energy_fanout.recipients.push({
  ...dupFanout.trace[fanoutIdx].energy_fanout.recipients[0],
});
assert.throws(() => createReplay(request, dupFanout), /Duplicate recipient in typed energy fanout/);
const badFanoutAfter = structuredClone(report);
badFanoutAfter.trace[fanoutIdx].energy_fanout.recipients[0].energy_after++;
assert.throws(() => createReplay(request, badFanoutAfter), /does not reconcile with its pool/);
const badFanoutBasis = structuredClone(report);
badFanoutBasis.trace[fanoutIdx].energy_fanout.base = '-1/1';
assert.throws(() => createReplay(request, badFanoutBasis), /Invalid packet energy basis/);
const missing = structuredClone(report);
const keptTracePrefix = [0];
for (const entry of missing.trace)
  keptTracePrefix.push(keptTracePrefix.at(-1) + (entry.event_kind !== 'hit_damage' ? 1 : 0));
missing.trace = missing.trace.filter((e) => e.event_kind !== 'hit_damage');
for (const frame of missing.state_timeline) frame.trace_end = keptTracePrefix[frame.trace_end];
assert.equal(
  createReplay(request, missing).stateAt(report.absolute_elapsed_us).hits,
  report.hits_settled,
);
const multiRequest = compileNte(getPresetSource('s1')).request;
for (const a of multiRequest.actors) {
  a.gear_procs = a.gear_procs.filter((p) => p.kind !== 'fireflies_crit_damage');
}
multiRequest.targets.push({ ...multiRequest.targets[0], target_index: 1 });
const multi = wasm.simulateGame(multiRequest);
assert.equal(multi.execution_completed, true);
assert.equal(
  createReplay(multiRequest, multi).events.filter((e) => e.hit).length,
  multi.hit_receipts.length,
);
assert.throws(() => createReplay(request, { ...report, hits_settled: 99 }), /hit count/);
const liveSource =
  '# 🦀\r\noptions duration=5 iteration=2 crit=random;\r\nmint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;\r\ntarget hp=100000000 def=0 resist=0; active mint;\r\nrepeat 2 { mint attack; wait(1); }';
const liveCompiled = engine.compileSource(liveSource).compiled;
const liveExecution = engine.runSourceSample({ compiled: liveCompiled, seed: 43 });
const liveReplay = createReplay(liveExecution.request, liveExecution.report);
for (const event of liveReplay.events.filter(
  (e) => e.admitted || e.hit?.command_index !== undefined,
)) {
  const loc = eventLocation(liveExecution, event);
  assert.ok(loc);
  assert.match(liveSource.slice(loc.offset, loc.offset + loc.length), /attack/);
}
assert.equal(eventLocation(liveExecution, null), null);
const markup = stackedBars(
  [{ name: '<script>', values: [2, 3] }],
  [
    { name: 'Skill', color: '#fff' },
    { name: 'Burst', color: '#000' },
  ],
);
assert.doesNotMatch(markup, /<script>/);
assert.match(markup, /width:40%/);
assert.match(markup, /width:60%/);
checkReplayStateValidation(request, report);
checkReplayClocks(engine, report);
console.log(
  `Replay: ${reports} actual WASM reports, exact receipt ownership, simultaneous event stepping, rewind, energy/overflow, target HP, field time and immutable reports passed`,
);
