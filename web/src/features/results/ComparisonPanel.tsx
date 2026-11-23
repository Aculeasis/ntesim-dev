import { useState } from 'react';
import type { SimulationController } from '../../infrastructure/worker/controller';
import type { ComparisonBundle, ExperimentRequest } from '../../domain/contracts/source-report';
import type { ExportFile } from '../import-export/ExportDialog';
import { errorMessage } from '../../components/ui/format';
import { Panel } from '../../components/ui/Panel';
import { JsonDetails } from '../../components/ui/JsonDetails';

export function ComparisonPanel({
  baseline,
  controller,
  onBusy,
  onExport,
  disabled,
}: {
  baseline: string;
  controller: () => SimulationController;
  onBusy: (busy: boolean) => void;
  onExport: (file: ExportFile) => void;
  disabled: boolean;
}) {
  const [candidate, setCandidate] = useState(baseline),
    [seedText, setSeedText] = useState('42'),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(''),
    [result, setResult] = useState<ComparisonBundle | null>(null);
  const [events, setEvents] = useState('10000'),
    [total, setTotal] = useState('100000'),
    [bytes, setBytes] = useState('16777216');
  const compare = async () => {
    if (busy || disabled) return;
    setBusy(true);
    onBusy(true);
    setError('');
    setResult(null);
    try {
      const seeds = seedText.split(',').map((s) => Number(s.trim()));
      const values = [...seeds, Number(events), Number(total), Number(bytes)];
      if (!seedText.trim() || values.some((n) => !Number.isSafeInteger(n) || n < 0))
        throw new Error('Seeds and budgets must be nonnegative safe integers.');
      const request: ExperimentRequest = {
        schema_version: 1,
        seeds,
        max_events_per_run: Number(events),
        max_total_events: Number(total),
        max_receipt_bytes: Number(bytes),
      };
      setResult(
        await controller().execute('COMPARE_SOURCE_REPORTS', { baseline, candidate, request }),
      );
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
      onBusy(false);
    }
  };
  const c = result?.comparison;
  const ratio = (v: { numerator: number | string; denominator: number | string } | null) =>
    v ? `${v.numerator}/${v.denominator}` : 'Unavailable';
  return (
    <Panel title="Paired source comparison">
      <p className="chart-note">Compare the last run with another script using the same seeds.</p>
      <label className="comparison-source-label">
        Candidate source
        <textarea
          id="comparison-source"
          value={candidate}
          onChange={(e) => setCandidate(e.target.value)}
          rows={8}
          spellCheck={false}
          style={{ width: '100%' }}
        />
      </label>
      <div className="results-toolbar">
        <label>
          Seeds (comma separated)
          <input
            aria-label="Comparison seeds"
            value={seedText}
            onChange={(e) => setSeedText(e.target.value)}
          />
        </label>
        <label>
          Events/run
          <input
            aria-label="Events per run"
            value={events}
            onChange={(e) => setEvents(e.target.value)}
          />
        </label>
        <label>
          Total events
          <input
            aria-label="Total event budget"
            value={total}
            onChange={(e) => setTotal(e.target.value)}
          />
        </label>
        <label>
          Receipt bytes
          <input
            aria-label="Receipt byte budget"
            value={bytes}
            onChange={(e) => setBytes(e.target.value)}
          />
        </label>
        <button id="compare-sources" disabled={disabled || busy} onClick={compare}>
          Compare
        </button>
      </div>
      {busy && (
        <>
          <p role="status">Comparing sources…</p>
          <button onClick={() => controller().cancel()}>Cancel comparison</button>
        </>
      )}
      {error && <p role="alert">{error}</p>}
      {c && (
        <>
          <p role="status">
            {c.complete ? 'Complete' : 'Incomplete · excluded from successful ranking'} ·{' '}
            {c.direction}
          </p>
          <p>
            Mean exact damage delta {ratio(c.mean_damage_delta)} · mean exact DPS delta{' '}
            {ratio(c.mean_dps_delta)}
          </p>
          <div className="timeline-container">
            <table className="timeline-table" aria-label="Paired comparison">
              <thead>
                <tr>
                  <th>Seed</th>
                  <th>Eligible</th>
                  <th>Damage delta</th>
                  <th>DPS delta</th>
                </tr>
              </thead>
              <tbody>
                {c.pairs.map((p) => (
                  <tr key={p.seed}>
                    <td>{p.seed}</td>
                    <td>{String(p.eligible)}</td>
                    <td>{ratio(p.damage_delta)}</td>
                    <td>{ratio(p.dps_delta)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {[
            { name: 'Baseline', batch: c.baseline },
            { name: 'Candidate', batch: c.candidate },
          ].map(({ name, batch }) => (
            <JsonDetails key={String(name)} title={`${name} run details`} value={batch} />
          ))}
          <button
            id="export-comparison"
            onClick={() =>
              onExport({
                name: 'nte-comparison.json',
                type: 'application/json',
                text: result!.report_json,
              })
            }
          >
            Export exact comparison
          </button>
        </>
      )}
    </Panel>
  );
}
