import { useEffect, useRef, useState } from 'react';
import {
  SimulationController,
  SimulationCancelledError,
  SourceTrialError,
} from '../../infrastructure/worker/controller';
import type { FailedSourceTrial } from '../../domain/contracts/source';
import type { TrialResult } from '../../domain/contracts/trials';
import { errorMessage } from '../../components/ui/format';
export type RunStatus = 'idle' | 'running' | 'completed' | 'stopped' | 'failed';
export function useSimulation() {
  const client = useRef<SimulationController | null>(null);
  const [result, setResult] = useState<TrialResult | null>(null),
    [status, setStatus] = useState<RunStatus>('idle');
  const [progress, setProgress] = useState(0),
    [stage, setStage] = useState('Ready'),
    [failure, setFailure] = useState<string | null>(null);
  const [source, setSource] = useState(''),
    [runId, setRunId] = useState(0);
  const [failedTrial, setFailedTrial] = useState<FailedSourceTrial | null>(null);
  const busy = useRef(false);
  const controller = () => (client.current ??= new SimulationController());
  useEffect(
    () => () => {
      client.current?.cancel();
      client.current?.terminateWorker();
    },
    [],
  );
  const accept = (next: TrialResult) =>
    setResult((old) =>
      old
        ? {
            ...next,
            compiled: old.compiled,
            party: old.party,
            request: old.request,
            report: old.report,
            occurrences: old.occurrences,
            actionLocations: old.actionLocations,
          }
        : next,
    );
  const run = async (source: string, retained?: TrialResult) => {
    if (busy.current) return;
    busy.current = true;
    setSource(source);
    if (!retained) {
      setRunId((v) => v + 1);
      setResult(null);
    }
    setFailure(null);
    setFailedTrial(null);
    setProgress(retained ? Math.floor((100 * retained.samples.length) / retained.trials) : 0);
    setStatus('running');
    setStage(
      retained ? 'Verifying retained trials in a fresh Worker' : 'Loading WebAssembly engine',
    );
    try {
      const final = await controller().execute(
        'SIMULATE_NTE',
        { source, resume: retained },
        {
          onUpdate: accept,
          onProgress: (pct, stage) => {
            setProgress(pct);
            setStage(stage);
          },
        },
      );
      accept(final);
      setStatus('completed');
      setStage(`Completed ${final.aggregate.completed} / ${final.trials} iterations`);
    } catch (error) {
      if (error instanceof SourceTrialError) setFailedTrial(error.failure);
      const message = errorMessage(error),
        cancelled = error instanceof SimulationCancelledError;
      setStatus(cancelled ? 'stopped' : 'failed');
      setFailure(cancelled ? null : message);
      setStage(cancelled ? 'Simulation stopped' : 'Execution failed: ' + message);
    } finally {
      busy.current = false;
    }
  };
  const loadReport = (
    nextSource: string,
    nextResult: TrialResult,
    nextFailure?: string | null,
    nextFailedTrial?: FailedSourceTrial | null,
  ) => {
    setSource(nextSource);
    setRunId((v) => v + 1);
    setResult(nextResult);
    setStatus(nextFailure ? 'failed' : nextResult.partial ? 'stopped' : 'completed');
    setFailure(nextFailure ?? null);
    setFailedTrial(nextFailedTrial ?? null);
    setProgress(100);
    setStage(
      nextFailure
        ? 'Execution failed: ' + nextFailure
        : `Completed ${nextResult.aggregate.completed} / ${nextResult.trials} iterations`,
    );
  };
  return {
    result,
    status,
    progress,
    stage,
    failure,
    failedTrial,
    source,
    runId,
    run,
    loadReport,
    resume: () => result && run(source, result),
    cancel: () => client.current?.cancel(),
    controller,
  };
}
export type SimulationState = ReturnType<typeof useSimulation>;
