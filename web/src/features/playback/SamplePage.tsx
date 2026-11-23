import { useState } from 'react';
import type { TrialResult } from '../../domain/contracts/trials';
import type { SourceExecution } from '../../domain/contracts/source';
import type { SimulationController } from '../../infrastructure/worker/controller';
import { errorMessage } from '../../components/ui/format';
import { validateObservations } from '../../domain/reports/validation';
import { usePlayback } from './usePlayback';
import { ReplayStage } from './ReplayStage';
import { EventLog } from './EventLog';
import { SourceView } from './SourceView';
import { StatePanel } from './StatePanel';
import { ReportTools } from '../results/ReportTools';
import type { ExportFile } from '../import-export/ExportDialog';
export function SamplePage({
  result,
  visible,
  running,
  controller,
  onBusy,
  onExport,
}: {
  result: TrialResult;
  visible: boolean;
  running: boolean;
  controller: () => SimulationController;
  onBusy: (value: boolean) => void;
  onExport: (file: ExportFile) => void;
}) {
  const [execution, setExecution] = useState<SourceExecution>({
      source: result.compiled.input.source,
      request: result.request,
      report: result.report,
      action_locations: result.actionLocations,
      occurrences: result.occurrences,
      diagnostic: null,
      failure_context: null,
    }),
    [seed, setSeed] = useState(result.report.seed),
    [busy, setBusy] = useState(false),
    [error, setError] = useState('');
  const { request, report } = execution;
  const playback = usePlayback(request, report, visible);
  const load = async () => {
    if (running || busy || !result.samples.some((s) => s.seed === seed)) return;
    setBusy(true);
    onBusy(true);
    setError('');
    try {
      const nextExecution = await controller().execute('SOURCE_SAMPLE', {
        compiled: result.compiled,
        seed,
      });
      const next = nextExecution.report;
      if (!next.execution_completed || !next.invariants_passed)
        throw new Error('Sample did not complete.');
      if (
        next.seed !== seed ||
        next.content_id !== result.request.content_id ||
        next.clock_contract !== result.request.clock_contract ||
        next.timer_policy !== result.request.timer_policy ||
        next.absolute_limit_us !== result.request.absolute_limit_us ||
        next.relative_duration_us !== result.request.relative_duration_us ||
        next.engine_version !== result.report.engine_version ||
        next.engine_source_sha256 !== result.report.engine_source_sha256
      )
        throw new Error('Sample identity differs from the declared trial run.');
      if (nextExecution.source !== result.compiled.input.source || nextExecution.diagnostic)
        throw new Error('Sample differs from the executed source.');
      validateObservations(nextExecution.request, next);
      const observation = result.samples.find((sample) => sample.seed === seed)!;
      if (
        next.relative_elapsed_us !== observation.relative_elapsed_us ||
        next.absolute_elapsed_us !== observation.absolute_elapsed_us ||
        next.total_damage_dealt !== observation.damage ||
        JSON.stringify(next.available_shield_hp) !== JSON.stringify(observation.shp) ||
        next.game_dps_computed !== (observation.dps.relative !== null)
      )
        throw new Error('Regenerated sample differs from its native scalar clock observation.');
      setExecution(nextExecution);
    } catch (error) {
      setError(errorMessage(error));
    } finally {
      setBusy(false);
      onBusy(false);
    }
  };
  return (
    <>
      <section className="panel sample-panel" id="sample">
        <div className="panel-heading">
          <h2>Sample · Combat playback</h2>
          <label>
            Seed{' '}
            <select id="sample-seed" value={seed} onChange={(e) => setSeed(Number(e.target.value))}>
              {result.samples.map((s) => (
                <option key={s.seed} value={s.seed}>
                  {s.seed}
                </option>
              ))}
            </select>
          </label>
          <button id="load-sample" disabled={running || busy} onClick={load}>
            Load sample
          </button>
        </div>
        {error && <p role="alert">{error}</p>}
        <ReplayStage playback={playback} />
      </section>
      <EventLog playback={playback} />
      <SourceView execution={execution} event={playback.state.last} />
      <StatePanel playback={playback} />
      <ReportTools
        key={report.seed}
        compiled={result.compiled}
        execution={execution}
        controller={controller}
        disabled={running || busy}
        onBusy={onBusy}
        onExport={onExport}
      />
    </>
  );
}
