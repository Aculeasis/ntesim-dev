import { throwDiagnostics } from '../wasm/script';
import type { NtesimWasm } from '../wasm/client';
import type { WorkerRequest, WorkerResponse } from './protocol';
import type { TrialResult, TrialSample } from '../../domain/contracts/trials';
import type { SourceExecution } from '../../domain/contracts/source';
import { MIN_FLUSH_INTERVAL, MAX_FLUSH_INTERVAL, FLUSH_COST_RATIO } from './pool';

export async function runScript(
  wasm: NtesimWasm,
  message: Extract<WorkerRequest, { type: 'SIMULATE_NTE' }>,
  post: (message: WorkerResponse) => void,
) {
  const { id, payload } = message;
  const response = wasm.compileSource(payload.source);
  throwDiagnostics(response.diagnostics);
  const { party } = response;
  let compiled = response.compiled;
  if (!compiled || !party) throw new Error('Missing live source compilation/party.');
  const started = performance.now();
  let samples: TrialSample[] = [];
  let first: SourceExecution | null = null,
    result: TrialResult | null = null;
  let lastUpdate = started,
    lastYield = started,
    delivered = 0;
  let flushInterval = MIN_FLUSH_INTERVAL;
  const retained = payload.resume;
  const elapsed = () => (retained?.aggregate.elapsed_ms ?? 0) + performance.now() - started;
  if (retained) {
    if (payload.source !== retained.compiled.input.source)
      throw new Error('Resume source differs from the executed source.');
    const restored = wasm.resumeSourceTrials(retained.compiled, retained.samples, elapsed());
    compiled = retained.compiled;
    first = restored.first_execution;
    samples = restored.samples_delta;
  } else wasm.startSourceTrials(compiled);
  for (let offset = samples.length; offset < compiled.trials;) {
    const count = offset === 0 ? 1 : Math.min(25, compiled.trials - offset);
    const update = wasm.stepSourceTrials(offset, count, elapsed());
    first ??= update.first_execution;
    samples.push(...update.samples_delta);
    offset += update.samples_delta.length;
    const now = performance.now();
    let flushed = false;
    if (first && update.aggregate) {
      result = {
        party,
        compiled,
        request: first.request,
        report: first.report,
        actionLocations: first.action_locations,
        occurrences: first.occurrences,
        samples,
        trials: compiled.trials,
        aggregate: update.aggregate,
        warnings: [],
        partial: offset < compiled.trials,
      };
      const isDue =
        offset === 1 ||
        offset === compiled.trials ||
        update.failure != null ||
        now - lastUpdate >= flushInterval;

      if (isDue) {
        const flushStart = performance.now();
        result.aggregate = wasm.snapshotSourceTrials(elapsed());
        const { samples: _, ...snapshot } = result;
        post({
          id,
          type: 'UPDATE',
          result: {
            ...snapshot,
            samples_delta: samples.slice(delivered),
            sample_offset: delivered,
          },
        });
        delivered = samples.length;
        post({
          id,
          type: 'PROGRESS',
          progress: Math.floor((100 * offset) / compiled.trials),
          stage: `Running ${offset} / ${compiled.trials}`,
        });
        const flushCost = performance.now() - flushStart;
        flushInterval = Math.min(
          Math.max(flushCost * FLUSH_COST_RATIO, MIN_FLUSH_INTERVAL),
          MAX_FLUSH_INTERVAL,
        );
        lastUpdate = performance.now();
        flushed = true;
      }
    }
    if (update.failure) {
      const failure = update.failure;
      const diagnostic = failure.diagnostic;
      post({
        id,
        type: 'ERROR',
        source_failure: { compiled, execution: failure },
        error: `${diagnostic ? `Line ${diagnostic.loc.line}, column ${diagnostic.loc.column}: ` : ''}Trial ${offset + 1} halted: ${diagnostic?.message ?? failure.report.halt_diagnostic ?? 'execution is not eligible'}`,
      });
      return;
    }
    if (!update.samples_delta.length)
      throw new Error('Live source trial session made no progress.');

    // Yield to the event loop on trial 1 (for immediate UI responsiveness/cancellation)
    // or when flushed or when at least MIN_FLUSH_INTERVAL (100ms) has passed since last yield.
    // This avoids clamping delays on every 25 trials while keeping cancellation responsive.
    if (offset === 1 || flushed || now - lastYield >= MIN_FLUSH_INTERVAL) {
      lastYield = performance.now();
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }
  if (!result) throw new Error('No completed source trials.');
  post({ id, type: 'SUCCESS', result });
}
