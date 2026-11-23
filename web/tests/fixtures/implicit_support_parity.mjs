import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { stringifyWasmInput } from '../../src/infrastructure/wasm/client.ts';
import { parseLosslessJson } from '../../src/infrastructure/storage/lossless-json.ts';
import { createReplay } from '../../src/domain/replay/model.ts';

export async function verifyImplicitSupportParity({ engine, simulate, input, cli }) {
  for (const owner of ['sakiri', 'zero']) {
    for (const action of ['attack', 'dash']) {
      const { request } = engine.compileScript(`options duration=2 crit=fixed;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        ${owner} char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=0 resist=0;
        active mint; mint attack; ${owner} ${action};`);
      writeFileSync(input, `[${stringifyWasmInput(request)}]`);
      const native = spawnSync(cli, ['run-game-suite', input], {
        encoding: 'utf8',
        maxBuffer: 8 * 1024 * 1024,
        timeout: 15000,
      });
      assert.ifError(native.error);
      assert.equal(native.status, 2, native.stderr);
      const report = await simulate(request);
      assert.deepEqual(report, parseLosslessJson(native.stdout).reports[0]);
      assert.deepEqual(report, engine.simulateGame(request));
      assert.equal(report.execution_completed, false);
      const rejected = report.rejected_action.command_index;
      const occurrence = report.action_metadata_ledger
        .filter(
          (entry) =>
            entry.switch_origin?.kind === 'support' &&
            entry.command_indices.some((idx) => idx < rejected),
        )
        .at(-1);
      assert.ok(occurrence);
      const supportCommand = occurrence.command_indices.find((idx) => idx < rejected);
      assert.equal(request.actions[supportCommand].kind, 'switch_actor');
      assert.ok(
        !report.action_metadata_ledger.some((entry) => entry.command_indices.includes(rejected)),
      );
      assert.equal(occurrence.switch_origin.kind, 'support');
      assert.equal(report.metadata_catalog[occurrence.metadata_id].action.input_role, 'qte');
      const replay = createReplay(request, report);
      assert.equal(
        replay.stateAt(report.absolute_elapsed_us).active,
        request.actions[rejected].actor_index,
      );
      assert.throws(
        () =>
          engine.invoke('aggregate_trials', {
            request,
            total: 1,
            reports: [report],
            elapsed_ms: 0,
          }),
        /execution failed or incomplete/,
      );
      const missing = structuredClone(report);
      delete missing.action_metadata_ledger.find((entry) =>
        entry.command_indices.includes(supportCommand),
      ).switch_origin;
      assert.throws(
        () => createReplay(request, missing),
        /command identity|rejected action identity/,
      );
      const wrongParticipant = structuredClone(report);
      wrongParticipant.action_metadata_ledger.find((entry) =>
        entry.command_indices.includes(supportCommand),
      ).switch_origin.incoming_actor_index = 0;
      assert.throws(() => createReplay(request, wrongParticipant), /initiating participants/);
      const wrongRole = structuredClone(report);
      const roleOccurrence = wrongRole.action_metadata_ledger.find((entry) =>
        entry.command_indices.includes(supportCommand),
      );
      wrongRole.metadata_catalog[roleOccurrence.metadata_id].action.input_role = 'normal';
      assert.throws(() => createReplay(request, wrongRole), /role|primary/);
    }
  }
}
