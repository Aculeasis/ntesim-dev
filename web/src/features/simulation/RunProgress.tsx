import type { SimulationState } from './useSimulation';
import { format } from '../../components/ui/format';
export function RunProgress({ run }: { run: SimulationState }) {
  if (run.status === 'idle') return null;
  return (
    <div
      id="run-status-panel"
      className={`run-status-panel ${run.status === 'running' ? 'running' : ''}`}
    >
      <div className="run-message">
        <span className="spinner" />
        <strong id="run-status" role="status">
          {run.stage}
        </strong>
        <span id="run-clock">
          {run.result
            ? `${format(run.result.aggregate.elapsed_ms, 1)} ms · ${format(run.result.aggregate.trials_per_second)} trials/s`
            : ''}
        </span>
      </div>
      <progress id="run-progress" max="100" value={run.progress} aria-label="Completed trials" />
    </div>
  );
}
