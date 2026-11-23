import type { RunCallbacks } from './protocol';
import { PIPELINE_DEPTH, type WorkerPool } from './pool';
import { SourceTrialError } from './controller';
import { throwDiagnostics } from '../wasm/script';
import { cancelParallelRequests, sendWorkerRequest } from './parallel_transport';

export async function executeParallelNte(options: {
  payload: { source: string };
  callbacks: RunCallbacks;
  pool: WorkerPool;
  wasmSource: BufferSource | string;
  isCurrent: () => boolean;
  signal: AbortSignal;
  finish: () => void;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}) {
  const { payload, callbacks, pool, wasmSource, isCurrent, signal, finish, resolve, reject } =
    options;
  const started = performance.now();

  const workers = pool.ensure(pool.count).slice(0, pool.count);
  let cancelled = false;
  const capacityWaiters: (() => void)[] = [];
  const releaseCapacity = () => {
    for (const resume of capacityWaiters.splice(0)) resume();
  };
  const isAlive = () => isCurrent() && !cancelled;
  const fail = (error: unknown) => {
    if (!isAlive()) return;
    finish();
    reject(error instanceof Error ? error : new Error(String(error)));
  };
  signal.addEventListener(
    'abort',
    () => {
      cancelled = true;
      releaseCapacity();
      cancelParallelRequests(workers);
    },
    { once: true },
  );

  callbacks.onProgress?.(5, 'Compiling source scenario');
  if (!isAlive()) return;
  let compileRaw: any;
  try {
    compileRaw = await sendWorkerRequest(
      workers[0],
      {
        type: 'INVOKE',
        payload: { action: 'compile_source', payload: { source: payload.source } },
      },
      wasmSource,
    );
  } catch (err) {
    fail(err);
    return;
  }
  if (!isAlive()) return;
  const compileRes = compileRaw?.data ?? compileRaw;
  const { compiled, party, diagnostics } = compileRes ?? {};
  if (diagnostics && diagnostics.length > 0) {
    try {
      throwDiagnostics(diagnostics);
    } catch (diagErr) {
      fail(diagErr);
      return;
    }
  }
  if (!compiled || !party) {
    fail(new Error('Missing live source compilation or party.'));
    return;
  }

  const totalTrials: number = compiled.trials;

  callbacks.onProgress?.(10, 'Initializing aggregator');
  if (!isAlive()) return;
  try {
    await sendWorkerRequest(
      workers[0],
      {
        type: 'INVOKE',
        payload: { action: 'start_source_trials', payload: compiled },
      },
      wasmSource,
    );
  } catch (err) {
    fail(err);
    return;
  }
  if (!isAlive()) return;
  callbacks.onProgress?.(10, `Simulating across ${workers.length} workers...`);

  const CHUNK_SIZE = 50;
  const computeWorkersStart = workers.length > 1 ? 1 : 0;
  const capacity = PIPELINE_DEPTH * (workers.length - computeWorkersStart);
  let nextDispatchOffset = 0;
  let uncommitted = 0;
  let failureQueued = false;

  interface ChunkResult {
    first_execution?: any;
    failure?: any;
    samples: any[];
    observations: any[];
  }
  let first: any = null;
  let nextCommitOffset = 0;
  const completedChunks = new Map<number, ChunkResult>();
  const allSamples: any[] = [];
  let lastUpdate = started;
  let lastSnapshot: any = null;
  let pendingCommit: Promise<void> | undefined;

  const drainCompleted = async () => {
    while (isAlive() && completedChunks.has(nextCommitOffset)) {
      const ready = completedChunks.get(nextCommitOffset)!;
      completedChunks.delete(nextCommitOffset);
      const endOffset = nextCommitOffset + ready.samples.length;
      if (ready.failure) {
        const diagnostic = ready.failure.diagnostic;
        fail(
          new SourceTrialError(
            `${diagnostic ? `Line ${diagnostic.loc.line}, column ${diagnostic.loc.column}: ` : ''}Trial ${endOffset + 1} halted: ${diagnostic?.message ?? ready.failure.report?.halt_diagnostic ?? 'execution is not eligible'}`,
            { compiled, execution: ready.failure },
          ),
        );
        return;
      }

      const now = performance.now();
      const isLast = endOffset === totalTrials;
      const isDue = endOffset === 1 || isLast || now - lastUpdate >= 200;
      const elapsed = now - started;

      const aggRaw: any = await sendWorkerRequest(
        workers[0],
        {
          type: 'ADD_SOURCE_OBSERVATIONS',
          payload: {
            observations: ready.observations,
            elapsed_ms: elapsed,
            snapshot: isDue,
          },
        },
        wasmSource,
      );
      if (!isAlive()) return;
      nextCommitOffset = endOffset;
      allSamples.push(...ready.samples);
      uncommitted--;
      releaseCapacity();
      const aggSnapshot = aggRaw?.data ?? aggRaw;
      if (aggSnapshot) lastSnapshot = aggSnapshot;

      if (first && isDue && lastSnapshot) {
        const resultSnapshot = {
          party,
          compiled,
          request: first.request,
          report: first.report,
          actionLocations: first.action_locations,
          occurrences: first.occurrences,
          trials: totalTrials,
          aggregate: lastSnapshot,
          warnings: [],
          partial: nextCommitOffset < totalTrials,
          samples: allSamples.slice(),
        };
        callbacks.onUpdate?.(resultSnapshot);
        if (!isAlive()) return;
        callbacks.onProgress?.(
          Math.floor((100 * nextCommitOffset) / totalTrials),
          `Running ${nextCommitOffset} / ${totalTrials}`,
        );
        lastUpdate = now;
      }
    }
  };

  const commitCompleted = () => {
    if (pendingCommit || !isAlive() || !completedChunks.has(nextCommitOffset)) return;
    pendingCommit = drainCompleted()
      .catch(fail)
      .finally(() => {
        pendingCommit = undefined;
        commitCompleted();
      });
  };

  const dispatchNext = async (workerIdx: number) => {
    while (isAlive() && !failureQueued && nextDispatchOffset < totalTrials) {
      if (uncommitted >= capacity) {
        await new Promise<void>((resume) => capacityWaiters.push(resume));
        continue;
      }
      const offset = nextDispatchOffset;
      const count = offset === 0 ? 1 : Math.min(CHUNK_SIZE, totalTrials - offset);
      nextDispatchOffset += count;
      uncommitted++;
      try {
        const res = await sendWorkerRequest<ChunkResult>(
          workers[workerIdx],
          {
            type: 'RUN_SOURCE_CHUNK',
            payload: {
              compiled,
              start_offset: offset,
              count,
            },
          },
          wasmSource,
        );

        if (!isAlive()) return;

        if (
          res.observations.length !== res.samples.length ||
          (res.failure ? res.samples.length >= count : res.samples.length !== count)
        ) {
          throw new Error('Worker returned an invalid trial chunk');
        }

        if (res.first_execution) {
          first = res.first_execution;
        }

        // Keep earlier dispatched seeds running so the reported failure has
        // the same seed regardless of which Worker answers first.
        if (res.failure) {
          failureQueued = true;
          releaseCapacity();
        }
        completedChunks.set(offset, res);
        commitCompleted();
      } catch (err) {
        fail(err);
        return;
      }
    }
  };

  const workerPromises: Promise<void>[] = [];
  // Reserve worker 0 for ordered commits when the pool has compute workers.
  for (let i = computeWorkersStart; i < workers.length; i++) {
    workerPromises.push(dispatchNext(i));
  }

  await Promise.all(workerPromises);
  while (pendingCommit) await pendingCommit;

  if (!isAlive()) return;

  if (nextCommitOffset < totalTrials) {
    fail(new Error(`Incomplete trials: ${nextCommitOffset} / ${totalTrials}`));
    return;
  }

  const elapsed = performance.now() - started;
  let finalAggregate = lastSnapshot;
  if (!finalAggregate) {
    try {
      const finalRaw: any = await sendWorkerRequest(
        workers[0],
        {
          type: 'INVOKE',
          payload: { action: 'snapshot_source_trials', payload: elapsed },
        },
        wasmSource,
      );
      finalAggregate = finalRaw?.data ?? finalRaw;
    } catch (err) {
      fail(err);
      return;
    }
  }
  if (!isAlive()) return;

  const finalResult = {
    party,
    compiled,
    request: first.request,
    report: first.report,
    actionLocations: first.action_locations,
    occurrences: first.occurrences,
    samples: allSamples,
    trials: totalTrials,
    aggregate: finalAggregate,
    warnings: [],
    partial: false,
  };
  finish();
  callbacks.onProgress?.(100, `Completed ${totalTrials} / ${totalTrials}`);
  callbacks.onUpdate?.(finalResult);
  resolve(finalResult);
}
