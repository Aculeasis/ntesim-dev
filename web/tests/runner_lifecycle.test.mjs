import assert from 'node:assert/strict';
import { SimulationController } from '../src/infrastructure/worker/controller.ts';
const timers = [],
  originalTimer = globalThis.setTimeout;
globalThis.Worker = class {
  postMessage() {}
  terminate() {
    this.terminated = true;
  }
};
globalThis.setTimeout = (callback, delay) => {
  timers.push(delay);
  return originalTimer(callback, delay);
};
try {
  const c = new SimulationController();
  assert.equal(c.timeoutMs, 0);
  const run = c.execute('SIMULATE_NTE', {});
  assert.deepEqual(timers, [], 'Ordinary browser runs must never install a 30s cutoff');
  c.handleMessage({ data: { id: c.activeTask.id - 1, type: 'ERROR', error: 'obsolete request' } });
  assert.equal(c.isBusy, true, 'Ignore obsolete Worker messages');
  c.handleMessage({ data: { id: c.activeTask.id, type: 'SUCCESS', result: { completed: 10 } } });
  assert.deepEqual(await run, { completed: 10 });
  const failure = c.execute(
      'SIMULATE_NTE',
      {},
      {
        onUpdate: () => {
          throw new Error('broken chart');
        },
      },
    ),
    worker = c.worker;
  c.handleMessage({
    data: { id: c.activeTask.id, type: 'UPDATE', result: { samples_delta: [], sample_offset: 0 } },
  });
  await assert.rejects(failure, /Report update failed: broken chart/);
  assert.equal(worker.terminated, true);
  assert.equal(c.isBusy, false);
  assert.equal(c.activeTask, null);
  const mismatch = c.execute('SIMULATE_NTE', {});
  c.handleMessage({
    data: {
      id: c.activeTask.id,
      type: 'UPDATE',
      result: { samples_delta: [{ seed: 42 }], sample_offset: 1 },
    },
  });
  await assert.rejects(mismatch, /offset mismatch/);
  const stopped = c.execute('SIMULATE_NTE', {});
  c.cancel();
  await assert.rejects(stopped, /cancelled by user/);
  const restart = c.execute('SIMULATE_NTE', {});
  c.handleMessage({
    data: { id: c.activeTask.id, type: 'ERROR', error: 'Trial 4 halted: locked' },
  });
  await assert.rejects(restart, /Trial 4 halted: locked/);
  assert.equal(c.isBusy, false);
  const crashed = c.execute('SIMULATE_NTE', {});
  const previousWorker = c.worker;
  previousWorker.onerror({ message: 'crash' });
  await assert.rejects(crashed, /Worker fatal error: crash/);
  const fresh = c.execute('SIMULATE_NTE', {});
  previousWorker.onerror({ message: 'late crash' });
  previousWorker.onmessage({ data: { id: c.activeTask.id, type: 'ERROR', error: 'late message' } });
  assert.equal(c.isBusy, true);
  c.handleMessage({ data: { id: c.activeTask.id, type: 'SUCCESS', result: 'recovered' } });
  assert.equal(await fresh, 'recovered');
} finally {
  delete globalThis.Worker;
  globalThis.setTimeout = originalTimer;
}
console.log('Runner lifecycle: timeout, callback errors, delta ordering, stop/retry passed');
